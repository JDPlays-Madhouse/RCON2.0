use std::{
    sync::mpsc::{self, RecvError},
    thread::JoinHandle,
};

use super::{
    IntegrationChannels, IntegrationCommand, IntegrationControl, IntegrationEvent,
    PlatformAuthenticate, PlatformConnection, Scopes, Transmittor,
};
use anyhow::Result;

fn default_scopes_twitch() -> Vec<&'static str> {
    vec!["user:read:chat"]
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct TwitchApiConnection {
    pub username: &'static str,
    client_id: &'static str,
    client_secret: &'static str,
    pub scope: Vec<&'static str>,
    pub event_tx: Option<mpsc::Sender<IntegrationEvent>>,
    pub command_channels: IntegrationChannels<IntegrationCommand>,
    pub command_joinhandle: Option<JoinHandle<Result<(), mpsc::RecvError>>>,
}

#[allow(dead_code)]
impl TwitchApiConnection {
    pub fn new(
        username: &'static str,
        client_id: &'static str,
        client_secret: &'static str,
    ) -> Self {
        Self {
            username,
            client_id,
            client_secret,
            ..Self::default()
        }
    }
}

impl Transmittor for TwitchApiConnection {
    /// Adds or changes the integration event transmitor.
    ///
    /// Returns the old transmittor in an option.
    fn add_transmitor(
        &mut self,
        tx: mpsc::Sender<IntegrationEvent>,
    ) -> Option<mpsc::Sender<IntegrationEvent>> {
        let old_tx = self.event_tx.take();
        self.event_tx = Some(tx);
        old_tx
    }

    fn remove_transmitor(&mut self) -> Option<mpsc::Sender<IntegrationEvent>> {
        self.event_tx.take()
    }

    fn transmit_event(
        &self,
        event: IntegrationEvent,
    ) -> Result<(), std::sync::mpsc::SendError<IntegrationEvent>> {
        if self.event_tx.is_some() {
            self.event_tx.clone().unwrap().send(event)
        } else {
            Err(mpsc::SendError(event))
        }
    }
}

impl Scopes for TwitchApiConnection {
    fn has_scope(&self, scope: &'static str) -> bool {
        self.scope.iter().any(|s| *s == scope)
    }

    fn add_scope(mut self, new_scope: &'static str) -> Self {
        if !self.has_scope(new_scope) {
            self.scope.push(new_scope)
        };
        self
    }

    fn default_scopes(mut self) -> Self {
        for scope in default_scopes_twitch() {
            self = self.add_scope(scope);
        }
        self
    }

    fn remove_scope(mut self, scope: &'static str) -> Self {
        self.scope.retain(|s| *s != scope);
        self
    }
}

impl PlatformConnection for TwitchApiConnection {
    fn connect(&self) -> Result<()> {
        todo!()
    }
}

impl PlatformAuthenticate for TwitchApiConnection {
    fn authenticate(&self) -> Result<()> {
        todo!()
    }
}

impl IntegrationControl for TwitchApiConnection {
    fn command_get_tx(&self) -> mpsc::Sender<IntegrationCommand> {
        self.command_channels.tx.clone()
    }

    fn start_thread(&mut self) -> Result<(), RecvError> {
        if self.command_channels.rx.is_none() {
            return Err(RecvError);
        }
        let command_rx = self.command_channels.rx.take().unwrap();
        self.command_joinhandle = Some(std::thread::spawn(move || {
            loop {
                match command_rx.recv() {
                    Ok(cmd) => match cmd {
                        IntegrationCommand::Stop => {
                            dbg!(cmd);
                            break;
                        }
                        _ => {
                            dbg!(cmd);
                        }
                    },
                    Err(e) => {
                        dbg!(e);
                        return Err(e);
                    }
                }
            }
            Ok(())
        }));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use crate::integration::IntegrationChannels;

    use super::*;

    #[fixture]
    fn twitch_connection() -> TwitchApiConnection {
        TwitchApiConnection::new("username", "client_id", "client_secret")
    }

    #[fixture]
    fn channels() -> IntegrationChannels<IntegrationEvent> {
        IntegrationChannels::default()
    }

    #[rstest]
    fn connection_initilization(twitch_connection: TwitchApiConnection) {
        let empty_vec: Vec<&str> = Vec::new();
        assert_eq!(twitch_connection.username, "username");
        assert_eq!(twitch_connection.client_id, "client_id");
        assert_eq!(twitch_connection.client_secret, "client_secret");
        assert_eq!(twitch_connection.scope, empty_vec);
    }

    #[rstest]
    fn add_one_scope(mut twitch_connection: TwitchApiConnection) {
        let scope: Vec<&str> = Vec::from(["test::scope"]);
        twitch_connection = twitch_connection.add_scope("test::scope");
        assert_eq!(twitch_connection.username, "username");
        assert_eq!(twitch_connection.client_id, "client_id");
        assert_eq!(twitch_connection.client_secret, "client_secret");
        assert_eq!(twitch_connection.scope, scope);
    }

    #[rstest]
    fn add_two_scope(mut twitch_connection: TwitchApiConnection) {
        let scope: Vec<&str> = Vec::from(["test::scope1", "test::scope2"]);
        twitch_connection = twitch_connection
            .add_scope("test::scope1")
            .add_scope("test::scope2");
        assert_eq!(twitch_connection.username, "username");
        assert_eq!(twitch_connection.client_id, "client_id");
        assert_eq!(twitch_connection.client_secret, "client_secret");
        assert_eq!(twitch_connection.scope, scope);
    }

    #[rstest]
    fn add_default_scope(mut twitch_connection: TwitchApiConnection) {
        twitch_connection = twitch_connection.default_scopes();
        assert_eq!(twitch_connection.username, "username");
        assert_eq!(twitch_connection.client_id, "client_id");
        assert_eq!(twitch_connection.client_secret, "client_secret");
        assert_eq!(twitch_connection.scope, default_scopes_twitch.clone());
    }

    #[rstest]
    fn has_scope(mut twitch_connection: TwitchApiConnection) {
        let scope = "test::scope";
        let empty_scopes: Vec<&str> = vec![];

        assert!(!twitch_connection.has_scope(scope));
        assert_eq!(twitch_connection.scope, empty_scopes);
        twitch_connection = twitch_connection.add_scope(scope);
        assert!(twitch_connection.has_scope(scope));
        assert_eq!(twitch_connection.scope, vec![scope]);
    }

    #[rstest]
    fn remove_scope(mut twitch_connection: TwitchApiConnection) {
        let scope = "test::scope";
        let empty_scopes: Vec<&str> = vec![];

        assert!(!twitch_connection.has_scope(scope));
        assert_eq!(twitch_connection.scope, empty_scopes);
        twitch_connection = twitch_connection.add_scope(scope);
        assert!(twitch_connection.has_scope(scope));
        assert_eq!(twitch_connection.scope, vec![scope]);
        twitch_connection = twitch_connection.remove_scope(scope);
        assert!(!twitch_connection.has_scope(scope));
        assert_eq!(twitch_connection.scope, empty_scopes);
    }

    #[rstest]
    fn channel_transmitting(
        mut channels: IntegrationChannels<IntegrationEvent>,
        mut twitch_connection: TwitchApiConnection,
    ) {
        twitch_connection.add_transmitor(channels.tx.clone());
        let msg = IntegrationEvent::Chat("Test 124");
        assert!(twitch_connection
            .transmit_event(IntegrationEvent::Connected)
            .is_ok());
        assert!(twitch_connection.transmit_event(msg).is_ok());

        let rx = channels.take_rx().unwrap();
        assert_eq!(rx.recv().unwrap(), IntegrationEvent::Connected);
        assert_eq!(rx.recv().unwrap(), msg);
    }
}
