use crate::integration::IntegrationEvent;
use anyhow::Result;
use tokio::{
    spawn,
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};
use tracing::error;

/// Responsible for receiving [IntegrationEvent] and handling them.
#[derive(Debug)]
pub struct Runner {
    rx: Receiver<IntegrationEvent>,
    tx: Sender<IntegrationEvent>,
}

impl Runner {
    fn new() -> Self {
        let (tx, rx) = channel::<IntegrationEvent>(100);
        Self { rx, tx }
    }

    /// Used for getting the transmittor, returns a clone.
    fn tx(&self) -> Sender<IntegrationEvent> {
        self.tx.clone()
    }

    fn run(mut self) -> Result<JoinHandle<Result<()>>> {
        use IntegrationEvent::*;
        Ok(spawn(async move {
            loop {
                match self.rx.recv().await {
                    Some(Chat { msg, author }) => {
                        todo!()
                    }
                    Some(Connected) => todo!(),
                    Some(ChannelPoint { id, redeemer }) => todo!(),
                    Some(e) => {
                        error!("Unexpected integration event: {:?}", e)
                    }
                    None => return Ok(()),
                }
            }
        }))
    }
}
