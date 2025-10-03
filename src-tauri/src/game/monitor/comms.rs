use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct Comms<T: Clone> {
    tx: Sender<T>,
    rx: Receiver<T>,
}

impl<T: Clone> Clone for Comms<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: self.tx.subscribe(),
        }
    }
}

impl<T: Clone> Default for Comms<T> {
    fn default() -> Self {
        let (tx, rx) = channel(10);
        Self { tx, rx }
    }
}

impl<T: Clone> Comms<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_tx(&self) -> Sender<T> {
        self.tx.clone()
    }

    pub fn new_rx(&self) -> Receiver<T> {
        self.tx.subscribe()
    }

    pub fn tx(&self) -> &Sender<T> {
        &self.tx
    }

    pub fn rx(&self) -> &Receiver<T> {
        &self.rx
    }

    pub fn tx_mut(&mut self) -> &mut Sender<T> {
        &mut self.tx
    }

    pub fn rx_mut(&mut self) -> &mut Receiver<T> {
        &mut self.rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comms() {
        let mut comms = Comms::<u64>::new();
        let mut rx1 = comms.new_rx();

        let tx1 = comms.new_tx();
        let _ = tx1.send(5);
        assert_eq!(rx1.try_recv().unwrap(), 5);
        assert_eq!(comms.rx_mut().try_recv().unwrap(), 5);

        let _ = tx1.send(6);
        let _ = comms.tx_mut().send(7);

        assert_eq!(rx1.try_recv().unwrap(), 6);
        assert_eq!(rx1.try_recv().unwrap(), 7);
        assert_eq!(comms.rx_mut().try_recv().unwrap(), 6);
        assert_eq!(comms.rx_mut().try_recv().unwrap(), 7);
    }
}
