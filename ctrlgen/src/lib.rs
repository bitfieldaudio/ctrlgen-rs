#![feature(try_trait_v2, generic_associated_types, type_alias_impl_trait)]
#![doc = include_str!("../../README.md")]

#[cfg(feature = "tokio")]
pub mod promise;
pub mod returnval;

#[cfg(feature = "alloc")]
extern crate alloc;

use std::marker::PhantomData;

pub use ctrlgen_derive::ctrlgen;

pub trait Proxy<Msg> {
    fn send(&self, msg: Msg);
}

pub trait Returnval {
    type Sender<T>;
    type Receiver<T>;
    type SendError;
    type RecvResult<T>;

    fn create<T>() -> (Self::Sender<T>, Self::Receiver<T>);
    fn send<T>(tx: Self::Sender<T>, msg: T) -> core::result::Result<(), Self::SendError>;
    fn recv<T>(rx: Self::Receiver<T>) -> Self::RecvResult<T>;
}

pub trait AsyncReturnval {
    type Sender<T>;
    type Receiver<T>;
    type SendError;
    type RecvResult<T>;

    type SendFuture<T>: core::future::Future<Output = core::result::Result<(), Self::SendError>>;
    type RecvFuture<T>: core::future::Future<Output = Self::RecvResult<T>>;

    fn create<T>() -> (Self::Sender<T>, Self::Receiver<T>);

    fn async_send<T>(tx: Self::Sender<T>, msg: T) -> Self::SendFuture<T>;
    fn async_recv<T>(rx: Self::Receiver<T>) -> Self::RecvFuture<T>;
}

pub trait CallMut<Service> {
    type Error;
    fn call_mut(self, service: &mut Service) -> core::result::Result<(), Self::Error>;
}

pub trait CallMutAsync<Service> {
    type Error;
    type Future<'a>: core::future::Future<Output = core::result::Result<(), Self::Error>> + 'a
    where
        Service: 'a;
    fn call_mut_async(self, service: &mut Service) -> Self::Future<'_>;
}

impl<T, U: 'static> CallMutAsync<T> for U
where
    U: CallMut<T>,
{
    type Error = U::Error;
    type Future<'a> = impl core::future::Future<Output = core::result::Result<(), Self::Error>> + 'a
        where T: 'a;

    fn call_mut_async(self, service: &mut T) -> Self::Future<'_> {
        async { self.call_mut(service) }
    }
}

#[cfg(feature = "flume")]
pub struct FlumeProxy<Msg> {
    sender: flume::Sender<Msg>,
}

#[cfg(feature = "flume")]
impl<Msg> FlumeProxy<Msg> {
    pub fn new(sender: flume::Sender<Msg>) -> Self {
        Self { sender }
    }
}

#[cfg(feature = "flume")]
impl<Msg: core::fmt::Debug> Proxy<Msg> for FlumeProxy<Msg> {
    fn send(&self, msg: Msg) {
        self.sender.send(msg).unwrap()
    }
}

#[cfg(feature = "tokio")]
pub struct TokioProxy<Msg> {
    sender: tokio::sync::mpsc::UnboundedSender<Msg>,
}

#[cfg(feature = "tokio")]
impl<Msg> TokioProxy<Msg> {
    pub fn new(sender: tokio::sync::mpsc::UnboundedSender<Msg>) -> Self {
        Self { sender }
    }
}

#[cfg(feature = "tokio")]
impl<Msg: core::fmt::Debug> Proxy<Msg> for TokioProxy<Msg> {
    fn send(&self, msg: Msg) {
        self.sender.send(msg).unwrap()
    }
}

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
