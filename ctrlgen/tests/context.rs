use std::cell::RefCell;

use ctrlgen::support::LocalRetval;
use ctrlgen::CallMut;

#[derive(Default)]
struct Service {
    counter: i32,
    last_ctx: i32,
}

#[ctrlgen::ctrlgen(
    pub enum ServiceMsg,
    context(ctx: i32)
)]
impl Service {
    pub fn increment_by(&mut self, ctx: i32, arg: i32) {
        self.counter += arg;
        self.last_ctx = ctx;
    }
}

#[test]
fn call_mut_works() {
    let mut service = Service {
        counter: 0,
        last_ctx: 0,
    };
    let msg = ServiceMsg::IncrementBy { arg: 2 };
    msg.call_mut_with_ctx(&mut service, 3).unwrap();

    assert_eq!(service.last_ctx, 3)
}
