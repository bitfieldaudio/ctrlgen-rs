#![feature(generic_associated_types, type_alias_impl_trait)]

struct Service;

#[ctrlgen::ctrlgen(pub ServiceMsg,
    enum_attr[derive(Debug)],
    #[derive(Clone)]
)]
impl Service {
    pub fn foo(&mut self) {}
}

#[test]
fn enum_attr_word_syntax() {
    let msg = ServiceMsg::Foo {};
    format!("{msg:?}");
}

#[test]
fn enum_attr_hashtag_syntax() {
    let msg = ServiceMsg::Foo {};
    let _ = msg.clone();
}
