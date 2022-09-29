#![feature(try_trait_v2, type_alias_impl_trait)]
#![doc = include_str!("../../README.md")]

#[cfg(feature = "support")]
pub mod support;

#[cfg(feature = "alloc")]
extern crate alloc;

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

