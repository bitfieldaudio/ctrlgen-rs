use core::cell::RefCell;
use core::convert::Infallible;
use std::marker::PhantomData;

use crate::Proxy;
use crate::Returnval;

#[cfg(feature = "flume")]
pub mod flume;

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "tokio")]
pub mod promise;

#[derive(Debug)]
pub struct FailedToSendRetval;
impl std::error::Error for FailedToSendRetval {}
impl std::fmt::Display for FailedToSendRetval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to send return value")
    }
}

/// A Proxy that sends messages through a function
pub struct FnProxy<Msg, F: Fn(Msg)> {
    f: F,
    _phantom: PhantomData<Msg>,
}

impl<Msg, F: Fn(Msg)> FnProxy<Msg, F> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<Msg, F: Fn(Msg)> Proxy<Msg> for FnProxy<Msg, F> {
    fn send(&self, msg: Msg) {
        (self.f)(msg)
    }
}

#[cfg(feature = "alloc")]
use alloc::rc::Rc;

#[cfg(feature = "alloc")]
pub struct LocalRetval;

#[cfg(feature = "alloc")]
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
