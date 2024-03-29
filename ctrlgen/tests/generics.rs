#![feature(type_alias_impl_trait)]
use std::cell::RefCell;

use ctrlgen::support::FnProxy;
use ctrlgen::support::LocalRetval;
use ctrlgen::CallMut;

#[derive(Default)]
struct Service<T: From<i32>> {
    counter: T,
}

#[ctrlgen::ctrlgen(pub enum ServiceMsg,
    returnval = LocalRetval,
    proxy(trait ServiceProxyTrait)
)]
impl<T: From<i32> + Into<i32> + Copy> Service<T> {
    pub fn increment_by(&mut self, arg: i32) -> i32 {
        self.counter = arg.into();
        self.counter.into()
    }
}

#[test]
fn proxy() {
    let service = RefCell::new(Service { counter: 0 });

    // With proxy:
    let proxy = FnProxy::new(|msg: ServiceMsg| {
        msg.call_mut(&mut *service.borrow_mut()).unwrap();
    });

    let ret = proxy.increment_by(2);
    assert_eq!(*ret.borrow(), Some(2));
    assert_eq!(service.borrow().counter, 2);
}
