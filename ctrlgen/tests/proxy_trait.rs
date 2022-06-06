#![feature(generic_associated_types, type_alias_impl_trait)]
use std::cell::RefCell;

use ctrlgen::returnval::LocalRetval;
use ctrlgen::CallMut;
use ctrlgen::Proxy;

#[derive(Default)]
struct Service {
    counter: i32,
}

#[ctrlgen::ctrlgen(pub enum ServiceMsg,
    proxy(trait ServiceProxyTrait),
    returnval = LocalRetval,
)]
impl Service {
    pub fn increment_by(&mut self, arg: i32) -> i32 {
        self.counter += arg;
        self.counter
    }
}

#[test]
fn proxy_trait_impl_directly() {
    struct ServiceProxy {
        service: RefCell<Service>,
    }

    impl ServiceProxyTrait for ServiceProxy {
        fn send(&self, msg: ServiceMsg) {
            msg.call_mut(&mut *self.service.borrow_mut());
        }
    }

    let proxy = ServiceProxy {
        service: RefCell::new(Service {
            counter: 0
        })
    };

    proxy.increment_by(2);
    assert!(proxy.service.borrow().counter == 2);
}

#[test]
fn proxy_trait_impl_proxy() {
    struct ServiceProxy1 {
        service: RefCell<Service>,
    }

    impl Proxy<ServiceMsg> for ServiceProxy1 {
        fn send(&self, msg: ServiceMsg) {
            msg.call_mut(&mut *self.service.borrow_mut());
        }
    }

    let proxy = ServiceProxy1 {
        service: RefCell::new(Service {
            counter: 0
        })
    };

    proxy.increment_by(2);
    assert!(proxy.service.borrow().counter == 2);
}
