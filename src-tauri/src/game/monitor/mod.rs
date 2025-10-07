mod comms;
pub use comms::Comms;
use config::Config;
use tokio::task::JoinHandle;

use crate::{game::GameServerStatus, servers::GameServer};

type Result<T, E = MonitorError> = core::result::Result<T, E>;

#[derive(Debug, Default)]
pub struct Monitor {
    current_server_comms: Comms<GameServer>,
    status_comms: Comms<GameServerStatus>,
    current_config: Config,
    join_handle: Option<JoinHandle<()>>,
}

impl Monitor {
    pub fn new() -> Self {
        Self::default()
    }

    /// TODO: [Result] might be unnessesary
    pub async fn run(&mut self) -> Result<()> {
        let mut status_comms = self.status_comms.clone();
        let current_server_comms = self.current_server_comms.clone();
        let current_config = self.current_config.clone();
        self.join_handle = Some(tokio::spawn(async move {
            let mut error_count: u8 = 0;
            loop {
                match status_comms.rx_mut().recv().await {
                    Ok(status) => {
                        error_count = error_count.saturating_sub(1);
                    }
                    Err(e) => {
                        tracing::error!("{e}");
                        error_count = error_count.saturating_add(1);
                        if error_count > 10 {
                            return;
                        }
                    }
                }
                // tokio::select! {
                //     Ok(status) = self.status_comms_mut().rx_mut().recv().await => {
                //         println!("{:?}", status)
                //     },
                //     else => break
            }
        }));
        Ok(())
    }

    pub fn current_server_comms(&self) -> &Comms<GameServer> {
        &self.current_server_comms
    }

    pub fn status_comms(&self) -> &Comms<GameServerStatus> {
        &self.status_comms
    }

    pub fn current_config(&self) -> &Config {
        &self.current_config
    }

    pub fn join_handle(&self) -> Option<&JoinHandle<()>> {
        self.join_handle.as_ref()
    }

    pub fn current_server_comms_mut(&mut self) -> &mut Comms<GameServer> {
        &mut self.current_server_comms
    }

    pub fn status_comms_mut(&mut self) -> &mut Comms<GameServerStatus> {
        &mut self.status_comms
    }

    pub fn current_config_mut(&mut self) -> &mut Config {
        &mut self.current_config
    }

    pub fn join_handle_mut(&mut self) -> &mut Option<JoinHandle<()>> {
        &mut self.join_handle
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic, PartialEq)]
pub enum MonitorError {}
