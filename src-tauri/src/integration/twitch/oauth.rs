use std::sync::{Arc, LazyLock};
use std::time::SystemTime;

use anyhow::Context;
use cached::{stores::DiskCacheBuilder, DiskCache, IOCached};
use http::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufStream};
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};
use twitch_oauth2::TwitchToken;
use twitch_oauth2::{tokens::UserTokenBuilder, AccessToken, RefreshToken, Scope, UserToken};

const HOUR: u64 = 3600;

/// The token used to authenticate with the Twitch API.
pub static TOKEN: LazyLock<Arc<tokio::sync::Mutex<Option<UserToken>>>> =
    LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(None)));

/// Refreshes the token if it exists. Returns the new token if successful.
pub async fn refresh_token() -> Option<UserToken> {
    let mut token_cont = TOKEN.lock().await;
    let token = match token_cont.as_mut() {
        Some(t) => t,
        None => return None,
    };
    let old_token = token.clone();

    let reqwest_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    match (*token).refresh_token(&reqwest_client).await {
        Ok(_) => {}
        Err(e) => {
            error!("Error refreshing token: {}", e);
        }
    };
    if old_token.access_token == token.clone().access_token {
        warn!("Token not refreshed!")
    }
    Some(token.clone())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerializableUserToken {
    pub access_token: AccessToken,
    pub refresh_token: Option<RefreshToken>,
}

impl From<UserToken> for SerializableUserToken {
    fn from(value: UserToken) -> Self {
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
        }
    }
}

pub fn token_cache(key: String) -> DiskCache<String, SerializableUserToken> {
    let config = sled::Config::new().flush_every_ms(None);
    DiskCacheBuilder::new(key)
        .set_connection_config(config)
        .set_sync_to_disk_on_cache_change(true)
        .set_disk_directory(dirs::cache_dir().unwrap().join("RCON2.0"))
        .build()
        .context("Token cache build")
        .unwrap()
}

pub fn oauth_scope_to_string(scopes: &Vec<Scope>) -> String {
    let mut scopes_list = scopes.to_owned();
    let mut scope_string: String = String::new();
    scopes_list.sort_by_cached_key(|k| k.to_string());
    for scope in scopes_list {
        scope_string += &scope.to_string()
    }
    scope_string
}

pub async fn oauth(
    scopes: Vec<Scope>,
    client_id: String,
    client_secret: String,
    redirect_url: String,
    use_cache: bool,
) -> anyhow::Result<UserToken> {
    let cache = token_cache("TWITCH_OAUTH".to_string());
    let cache_key = client_id.clone() + &oauth_scope_to_string(&scopes);
    let mut cached_token_option = match cache.cache_get(&cache_key) {
        Ok(value) => value,
        Err(e) => {
            error!("DiskCacheError: {:?}", e);
            None
        }
    };
    if !use_cache {
        cached_token_option = None;
        info!("Not using cache");
    }
    // Setup the http client to use with the library.
    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let client_secret = twitch_oauth2::ClientSecret::new(client_secret);

    if let Some(token) = cached_token_option {
        match UserToken::from_existing(
            &reqwest,
            token.access_token,
            token.refresh_token,
            client_secret.clone(),
        )
        .await
        {
            Ok(t) => return Ok(t),
            Err(e) => {
                error!("Token Validation Error: {}", e.to_string());
            }
        }
    }
    debug!(
        target = "Twitch OAuth",
        "No cached token found, generating new token."
    );
    let client_id = twitch_oauth2::ClientId::new(client_id.to_string());
    let redirect_url = twitch_oauth2::url::Url::parse(&redirect_url).unwrap();
    let response_port = redirect_url.port().unwrap_or(27934);

    // Create the builder!
    let mut builder = UserTokenBuilder::new(client_id, client_secret, redirect_url)
        .force_verify(false)
        .set_scopes(scopes);

    // Generate the URL, this is the url that the user should visit to authenticate.
    let (mut url, _csrf_code) = builder.generate_url();
    let iat = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let exp = iat + 12 * HOUR; // Unsure if this actually works.

    let current_query = url.query().unwrap_or("");
    url.set_query(Some(
        format!(
            "{current}&claims={{\"exp\":{exp}}}",
            current = current_query,
            exp = exp
        )
        .as_str(),
    ));

    println!("Generated OAuth URL: {}", &url);
    let _ = webbrowser::open(url.as_str()); // BUG: Not handling error.
    let input = response_uri(response_port).await.unwrap();
    let u = twitch_oauth2::url::Url::parse(&input)
        .context("when parsing the input as a URL")
        .unwrap(); // BUG: Not handling error.

    // Grab the query parameters "state" and "code" from the url the user was redirected to.
    let map: std::collections::HashMap<_, _> = u.query_pairs().collect();
    println!("Map: {:?}", &map);
    let token = match (map.get("state"), map.get("code")) {
        (Some(state), Some(code)) => {
            // Finish the builder with `get_user_token`
            match builder.get_user_token(&reqwest, state, code).await {
                Ok(token) => token,
                Err(e) => {
                    error!("Error getting user token: {}", e);
                    anyhow::bail!("Error getting user token: {}", e);
                }
            }
        }
        _ => match (map.get("error"), map.get("error_description")) {
            (std::option::Option::Some(error), std::option::Option::Some(error_description)) => {
                error!(
                    "twitch oauth errored with error: {} - {}",
                    error, error_description
                );
                anyhow::bail!(
                    "twitch oauth errored with error: {} - {}",
                    error,
                    error_description
                );
            }
            _ => anyhow::bail!("invalid url passed"),
        },
    };
    debug!("Twitch User Token: {:?}", &token);
    let serializable_token = SerializableUserToken::from(token.clone());
    match cache.cache_set(cache_key, serializable_token) {
        Ok(_) => {
            debug!("Stored key in cache.");
        }
        Err(e) => {
            dbg!(&e);
            debug!("Disk Cache failure: {}", e);
        }
    }
    Ok(token)
}

async fn response_uri(port: u16) -> anyhow::Result<String> {
    let host = "localhost";
    let listener_address = format!("{}:{}", host, port);
    let jh = tauri::async_runtime::spawn(async move {
        let listener = match TcpListener::bind(listener_address.clone()).await {
            Ok(l) => {
                debug! {"TcpListener started: {}", listener_address};
                l
            }
            Err(e) => {
                error!("TcpListener Error: {}", e);
                return Err::<String, String>(format!("TcpListener Error: {}", e));
            }
        };
        let _ = listener.set_ttl(30);
        // loop {
        let (stream, _address) = listener.accept().await.unwrap();
        let stream = BufStream::new(stream);
        match parse_request(stream).await {
            Ok(path) => Ok(path),
            Err(e) => {
                error!("{e:?}");
                Err(e.to_string())
            }
        }

        // }
    });
    let mut path = jh.await.context("Twitch OAuth URL").unwrap().unwrap();
    path = "http://".to_string() + host + &path;
    Ok(path)
}

async fn parse_request(
    mut stream: impl AsyncBufRead + AsyncWrite + Unpin,
) -> anyhow::Result<String> {
    let mut line_buffer = String::new();
    stream.read_line(&mut line_buffer).await?;

    let mut parts = line_buffer.split_whitespace();

    let _method: String = parts
        .next()
        .ok_or(anyhow::anyhow!("missing method"))
        .unwrap()
        .to_string();

    let path: String = parts
        .next()
        .ok_or(anyhow::anyhow!("missing path"))
        .map(Into::into)?;
    let _ = response(&mut stream).await;
    Ok(path)
}

/// TODO: Oauth code in uri not validated i.e. add error handling.
async fn response<O: AsyncWrite + Unpin>(stream: &mut O) -> anyhow::Result<()> {
    let status = StatusCode::OK;
    let response = Response::builder()
        .status(status)
        .body("You may now close this webpage.".to_string())
        .unwrap();
    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
        .collect::<Vec<_>>()
        .join("\r\n");

    stream
        .write_all(format!("HTTP/1.1 {}\r\n{headers}\r\n\r\n", status).as_bytes())
        .await
        .unwrap();
    tokio::io::copy(&mut response.body().as_bytes(), stream).await?;

    Ok(())
}
