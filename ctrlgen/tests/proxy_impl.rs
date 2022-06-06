#![feature(generic_associated_types, type_alias_impl_trait)]
use ctrlgen::returnval::LocalRetval;
use ctrlgen::Proxy;

#[derive(Default)]
struct Service {
    counter: i32,
    flag: bool,
}

#[ctrlgen::ctrlgen(pub enum ServiceMsg,
    returnval = LocalRetval,
    proxy(impl ServiceProxy),
)]
impl Service {
    pub fn increment_by(&mut self, arg: i32) -> i32 {
        self.counter += arg;
        self.counter
    }

    pub fn set_flag(&mut self, flag: bool) {
        self.flag = flag;
    }
}

struct ServiceProxy {}

impl Proxy<ServiceMsg> for ServiceProxy {
    fn send(&self, _msg: ServiceMsg) {}
}
