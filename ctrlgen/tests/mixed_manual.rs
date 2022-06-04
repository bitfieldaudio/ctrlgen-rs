#![feature(generic_associated_types, type_alias_impl_trait)]
use core::future::Future;

use ctrlgen::promise;
use ctrlgen::CallMut;
use ctrlgen::Returnval;

struct FailedToSendRetVal;
struct TokioRetval;
impl Returnval for TokioRetval {
    type Sender<T> = promise::Sender<T>;
    type Receiver<T> = promise::Promise<T>;
    type SendError = FailedToSendRetVal;

    type RecvResult<T> = promise::Promise<T>;

    fn create<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        promise::Promise::channel()
    }

    fn recv<T>(rx: Self::Receiver<T>) -> Self::RecvResult<T> {
        rx
    }

    fn send<T>(tx: Self::Sender<T>, msg: T) -> core::result::Result<(), Self::SendError> {
        tx.send(msg).map_err(|_| FailedToSendRetVal)
    }
}

struct XServiceX;

// Generated

enum XMessageX
where
    TokioRetval: Returnval,
{
    Foo {
        arg: i32,
        ret: <TokioRetval as ::ctrlgen::Returnval>::Sender<u32>,
    },
    Bar {
        flag: bool,
    },
}

impl CallMut<XServiceX> for XMessageX
where
    TokioRetval: Returnval,
{
    type Output = core::result::Result<(), <TokioRetval as Returnval>::SendError>;
    fn call_mut(self, service: &mut XServiceX) -> Self::Output {
        match self {
            XMessageX::Foo { arg, ret } => <TokioRetval as Returnval>::send(ret, todo!("call foo")),
            XMessageX::Bar { flag } => {
                todo!("Call bar");
                Ok(())
            }
        }
    }
}

// impl CallMutAsync<XServiceX> for XMessageX
// where
//     TokioRetval: AsyncReturnval,
// {
//     type Future = impl Future<Output = Result<(), <TokioRetval as Returnval>::SendError>>;
//     fn call_mut_async(self, service: &mut XServiceX) -> Self::Future {
//         match self {
//             XMessageX::Foo { arg, ret } => {
//                 <TokioRetval as Returnval>::send(ret, todo!("call foo"))
//             }
//             XMessageX::Bar { flag } => {
//                 todo!("Call bar");
//                 Ok(())
//             }
//         }
//     }
// }
