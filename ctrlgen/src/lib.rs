#![feature(try_trait_v2, generic_associated_types, type_alias_impl_trait)]
#![doc = include_str!("../../README.md")]

#[cfg(feature = "tokio")]
pub mod promise;
pub mod returnval;

#[cfg(feature = "alloc")]
extern crate alloc;

use std::marker::PhantomData;

pub use ctrlgen_derive::ctrlgen;

pub trait MessageSender<Msg> {
    fn send(&self, msg: Msg);
}

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

impl<T, Msg> MessageSender<Msg> for T
where
    T: Fn(Msg),
{
    fn send(&self, msg: Msg) {
        self(msg)
    }
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

pub struct BasicProxy<Msg, Sender>
where
    Sender: MessageSender<Msg>,
{
    sender: Sender,
    _phantom: PhantomData<Msg>,
}

impl<Msg, Sender> Proxy<Msg> for BasicProxy<Msg, Sender>
where
    Sender: MessageSender<Msg>,
{
    fn send(&self, msg: Msg) {
        self.sender.send(msg)
    }
}

impl<Msg, Sender> BasicProxy<Msg, Sender>
where
    Sender: MessageSender<Msg>,
{
    pub fn new(sender: Sender) -> Self {
        Self {
            sender,
            _phantom: Default::default(),
        }
    }
}

#[cfg(feature = "tokio")]
impl<Msg: core::fmt::Debug> MessageSender<Msg> for tokio::sync::mpsc::UnboundedSender<Msg> {
    fn send(&self, msg: Msg) {
        self.send(msg).unwrap()
    }
}

#[cfg(feature = "tokio")]
pub type TokioProxy<Msg> = BasicProxy<Msg, tokio::sync::mpsc::UnboundedSender<Msg>>;

#[cfg(feature = "flume")]
impl<Msg: core::fmt::Debug> MessageSender<Msg> for flume::Sender<Msg> {
    fn send(&self, msg: Msg) {
        self.send(msg).unwrap()
    }
}

#[cfg(feature = "flume")]
pub type FlumeProxy<Msg> = BasicProxy<Msg, flume::Sender<Msg>>;
