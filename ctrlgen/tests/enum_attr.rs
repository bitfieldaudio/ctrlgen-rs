#![feature(generic_associated_types, type_alias_impl_trait)]
use ctrlgen::returnval::LocalRetval;
use ctrlgen::CallMut;

#[derive(Default)]
struct Service;

#[ctrlgen::ctrlgen(pub ServiceMsg,
    enum_attr[derive(Debug)]
)]
impl Service {
    pub fn foo(&mut self) {}
}

#[test]
fn proxy() {
    let msg = ServiceMsg::Foo {};
    format!("{msg:?}");
}
