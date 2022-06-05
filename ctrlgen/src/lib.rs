#![feature(try_trait_v2, generic_associated_types, type_alias_impl_trait)]
pub mod promise;
pub mod returnval;

pub use ctrlgen_derive::ctrlgen;

pub trait MessageSender<Msg> {
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
    type Output;
    fn call_mut(self, service: &mut Service) -> Self::Output;
}

pub trait CallMutAsync<Service> {
    type Future<'a>: core::future::Future + 'a
    where
        Service: 'a;
    fn call_mut_async(self, service: &mut Service) -> Self::Future<'_>;
}

impl<T, U: 'static> CallMutAsync<T> for U
where
    U: CallMut<T>,
{
    type Future<'a> = impl core::future::Future<Output = U::Output> + 'a
        where T: 'a;

    fn call_mut_async(self, service: &mut T) -> Self::Future<'_> {
        async { self.call_mut(service) }
    }
}
