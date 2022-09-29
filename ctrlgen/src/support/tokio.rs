use super::promise;
use crate::Proxy;
use crate::Returnval;

use super::FailedToSendRetval;

pub struct TokioProxy<Msg> {
    sender: tokio::sync::mpsc::UnboundedSender<Msg>,
}

impl<Msg> TokioProxy<Msg> {
    pub fn new(sender: tokio::sync::mpsc::UnboundedSender<Msg>) -> Self {
        Self { sender }
    }
}

impl<Msg: core::fmt::Debug> Proxy<Msg> for TokioProxy<Msg> {
    fn send(&self, msg: Msg) {
        self.sender.send(msg).unwrap()
    }
}

pub struct TokioRetval;

impl Returnval for TokioRetval {
    type Sender<T> = promise::Sender<T>;
    type Receiver<T> = promise::Promise<T>;
    type SendError = FailedToSendRetval;

    type RecvResult<T> = promise::Promise<T>;

    fn create<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        promise::Promise::channel()
    }

    fn recv<T>(rx: Self::Receiver<T>) -> Self::RecvResult<T> {
        rx
    }

    fn send<T>(tx: Self::Sender<T>, msg: T) -> core::result::Result<(), Self::SendError> {
        tx.send(msg).map_err(|_| FailedToSendRetval)
    }
}
