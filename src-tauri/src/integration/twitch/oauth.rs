use std::{sync::mpsc::channel, thread, time::Duration};

use anyhow::Context;
use futures::executor;
use twitch_oauth2::{tokens::UserTokenBuilder, Scope, UserToken};
use url::Url;

pub fn oauth(
    scopes: Vec<Scope>,
    client_id: &'static str,
    client_secret: &'static str,
    redirect_url: &'static str,
) -> anyhow::Result<UserToken> {
    // Setup the http client to use with the library.
    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let client_id = twitch_oauth2::ClientId::new(client_id.to_string());
    let client_secret = twitch_oauth2::ClientSecret::new(client_secret.to_string());
    let redirect_url = twitch_oauth2::url::Url::parse(redirect_url).unwrap();

    // Create the builder!
    let mut builder = UserTokenBuilder::new(client_id, client_secret, redirect_url)
        .force_verify(true)
        .set_scopes(scopes);

    // Generate the URL, this is the url that the user should visit to authenticate.
    let (url, _csrf_code) = builder.generate_url();
    dbg!(&url);
    let _ = webbrowser::open(url.as_str());

    // TODO: Check which port is available then pass it in.
    let input = response_uri(27934);

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
    Ok(token)
}

fn get_env_or_arg(env: &str, args: &mut impl Iterator<Item = String>) -> Option<String> {
    std::env::var(env).ok().or_else(|| args.next())
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
