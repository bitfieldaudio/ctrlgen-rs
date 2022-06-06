#[proc_macro_attribute]
pub fn ctrlgen(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match ctrlgen_impl::ctrlgen_impl(attrs.into(), item.into()) {
        Ok(x) => x.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
