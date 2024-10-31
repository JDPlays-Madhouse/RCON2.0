use std::{
    sync::{mpsc::channel, Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{logging::LogLevel, Logger};
use anyhow::Context;
use cached::{stores::DiskCacheBuilder, DiskCache, IOCached};
use futures::executor;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use twitch_api::types::{UserId, UserName};
use twitch_oauth2::{
    tokens::UserTokenBuilder, AccessToken, ClientId, ClientSecret, RefreshToken, Scope, UserToken,
};
use url::Url;

const LOGLOCATION: &str = "Twitch OAuth";

#[derive(Error, Debug, PartialEq, Clone)]
enum OAuthError {
    #[error("error with disk cache `{0}`")]
    DiskError(String),
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
    logger: Arc<Mutex<Logger>>,
) -> anyhow::Result<UserToken> {
    let cache = token_cache("TWITCH_OAUTH".to_string());
    let cache_key = client_id.clone() + &oauth_scope_to_string(&scopes);
    let cached_token_option = match cache.cache_get(&cache_key) {
        Ok(value) => value,
        Err(e) => {
            dbg!(e);
            None
        }
    };
    // Setup the http client to use with the library.
    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let client_secret = twitch_oauth2::ClientSecret::new(client_secret.to_string());

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
                dbg!(e);
            }
        }
    }

    let client_id = twitch_oauth2::ClientId::new(client_id.to_string());
    let redirect_url = twitch_oauth2::url::Url::parse(&redirect_url).unwrap();
    let response_port = redirect_url.port().unwrap_or(27934);

    // Create the builder!
    let mut builder = UserTokenBuilder::new(client_id, client_secret, redirect_url)
        .force_verify(false)
        .set_scopes(scopes);

    // Generate the URL, this is the url that the user should visit to authenticate.
    let (url, _csrf_code) = builder.generate_url();
    let _ = webbrowser::open(url.as_str());

    let input = response_uri(response_port);

    let u = twitch_oauth2::url::Url::parse(&input)
        .context("when parsing the input as a URL")
        .unwrap();

    // Grab the query parameters "state" and "code" from the url the user was redirected to.
    let map: std::collections::HashMap<_, _> = u.query_pairs().collect();

    let token = match (map.get("state"), map.get("code")) {
        (Some(state), Some(code)) => {
            // Finish the builder with `get_user_token`
            executor::block_on(builder.get_user_token(&reqwest, state, code)).unwrap()
        }
        _ => match (map.get("error"), map.get("error_description")) {
            (std::option::Option::Some(error), std::option::Option::Some(error_description)) => {
                anyhow::bail!(
                    "twitch errored with error: {} - {}",
                    error,
                    error_description
                );
            }
            _ => anyhow::bail!("invalid url passed"),
        },
    };
    let serializable_token = SerializableUserToken::from(token.clone());
    match cache.cache_set(cache_key, serializable_token) {
        Ok(_) => {
            logger
                .lock()
                .unwrap()
                .log(LogLevel::Debug, "Twitch OAuth", "Stored key in cache.");
        }
        Err(e) => {
            dbg!(&e);
            logger.lock().unwrap().log(
                LogLevel::Error,
                LOGLOCATION,
                format!("Disk Cache failure: {}", e).as_str(),
            )
        }
    }
    dbg!(Ok(token))
}

fn response_uri(port: u16) -> String {
    env_logger::init();
    use simple_server::Server;
    let (tx, rx) = channel::<String>();
    let host = "localhost";

    let _ = thread::spawn(move || {
        let tx_thread = tx.clone();
        let server = Server::with_timeout(Duration::from_secs(60), move |request, mut response| {
            dbg!(request.uri());
            let url = Url::parse("http://localhost")
                .unwrap()
                .join(&request.uri().to_string())
                .unwrap()
                .to_string();

            let _ = tx_thread.clone().send(url);
            Ok(response.body("You may now close this tab.".as_bytes().to_vec())?)
        });
        server.listen(host, &port.to_string());
    });
    rx.recv().context("Twitch OAuth URL").unwrap()
}
