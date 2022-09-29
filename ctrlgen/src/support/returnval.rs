use crate::Returnval;

use core::cell::RefCell;
use core::convert::Infallible;

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

