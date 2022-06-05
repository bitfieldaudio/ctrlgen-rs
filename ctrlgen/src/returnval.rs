use crate::promise;
use crate::Returnval;

use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;

#[derive(Debug)]
pub struct FailedToSendRetval;
impl std::error::Error for FailedToSendRetval {}
impl std::fmt::Display for FailedToSendRetval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to send return value")
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

pub struct LocalRetval;
impl Returnval for LocalRetval {
    type Sender<T> = Rc<RefCell<Option<T>>>;
    type Receiver<T> = Rc<RefCell<Option<T>>>;
    type SendError = Infallible;

    type RecvResult<T> = Self::Receiver<T>;

    fn create<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        let x = Rc::new(RefCell::new(None));
        (x.clone(), x)
    }

    fn recv<T>(rx: Self::Receiver<T>) -> Self::RecvResult<T> {
        rx
    }

    fn send<T>(tx: Self::Sender<T>, msg: T) -> core::result::Result<(), Self::SendError> {
        tx.replace(Some(msg));
        Ok(())
    }
}
