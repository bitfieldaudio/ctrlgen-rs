#[proc_macro_attribute]
pub fn ctrlgen(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    ctrlgen_impl::ctrlgen_impl(attrs.into(), item.into()).into()
}
