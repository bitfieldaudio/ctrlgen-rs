#![feature(try_trait_v2, type_alias_impl_trait, impl_trait_in_assoc_type)]
#![doc = core::include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

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

pub trait IsUnit {
    fn new() -> Self;
}
impl IsUnit for () {
    fn new() -> Self {
        ()
    }
}

pub trait CallMut<Service>: Sized {
    type Error;
    type Context;
    fn call_mut_with_ctx(
        self,
        service: &mut Service,
        context: Self::Context,
    ) -> core::result::Result<(), Self::Error>;

    fn call_mut(self, service: &mut Service) -> core::result::Result<(), Self::Error>
    where
        Self::Context: IsUnit,
    {
        self.call_mut_with_ctx(service, Self::Context::new())
    }
}

pub trait CallMutAsync<Service>: Sized {
    type Error;
    type Context;
    type Future<'a>: core::future::Future<Output = core::result::Result<(), Self::Error>> + 'a
    where
        Service: 'a;
    fn call_mut_async_with_ctx(
        self,
        service: &mut Service,
        context: Self::Context,
    ) -> Self::Future<'_>;

    fn call_mut_async(self, service: &mut Service) -> Self::Future<'_>
    where
        Self::Context: IsUnit,
    {
        self.call_mut_async_with_ctx(service, Self::Context::new())
    }
}

impl<T, U: 'static> CallMutAsync<T> for U
where
    U: CallMut<T>,
{
    type Error = U::Error;
    type Context = U::Context;
    type Future<'a> = impl core::future::Future<Output = core::result::Result<(), Self::Error>> + 'a
        where T: 'a;

    fn call_mut_async_with_ctx(self, service: &mut T, context: Self::Context) -> Self::Future<'_> {
        async { self.call_mut_with_ctx(service, context) }
    }
}
