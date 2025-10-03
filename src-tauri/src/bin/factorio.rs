use std::time::Duration;

use miette::IntoDiagnostic;
use rcon2_lib::game::factorio::*;
use rcon2_lib::Result;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    miette::set_panic_hook();
    let username = dotenvy::var("FACTORIO_USERNAME").into_diagnostic()?;
    let token = dotenvy::var("FACTORIO_TOKEN").into_diagnostic()?;
    let server_name = dotenvy::var("FACTORIO_SERVER").into_diagnostic()?;

    let server_description_short =
        ServerDescriptionShort::find_server_by_name(&username, &token, &server_name)
            .await?
            .ok_or(MatchMakingApiError::ServerUnreachable)?;

    println!("{:#?}", &server_description_short);
    let mut server_description =
        ServerDescription::new(server_description_short.host_address(), &token).await?;
    loop {
        let _ = server_description.update(&token).await;
        println!("{:?}", server_description.duration_since_last_heartbeat()?);
        sleep(Duration::from_secs(5)).await;
    }
}
