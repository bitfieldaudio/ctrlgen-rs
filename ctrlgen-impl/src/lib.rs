#![feature(try_blocks)]
use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::quote as q;
use syn::Ident;

struct Argument {
    name: Ident,
    ty: syn::Type,
    enum_attr: Vec<proc_macro2::Group>,
    to_owned: bool,
}

impl std::fmt::Debug for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = &self.ty;
        f.debug_struct("Argument")
            .field("name", &self.name.to_string())
            .field("ty", &format!("{}", q! {#t}))
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReceiverStyle {
    Move,
    Mut,
    Ref,
}

struct Method {
    name: Ident,
    receiver_style: ReceiverStyle,
    args: Vec<Argument>,
    ret: Option<syn::Type>,
    enum_attr: Vec<syn::Attribute>,
    return_attr: Vec<syn::Attribute>,
    doc_attr: Vec<syn::Attribute>,
    r#async: bool,
}

impl Method {
    fn variant_name(&self) -> proc_macro2::Ident {
        let mut ident = quote::format_ident!(
            "{}",
            self.name
                .to_string()
                .to_case(convert_case::Case::UpperCamel)
        );
        ident.set_span(self.name.span());
        ident
    }
}

impl std::fmt::Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Method")
            .field("name", &self.name.to_string())
            .field("receiver_style", &self.receiver_style)
            .field("args", &self.args)
            .finish()
    }
}

pub struct InputData {
    /// Inherent impl name.
    name: Ident,
    generics: syn::Generics,
    struct_args: syn::PathArguments,
    methods: Vec<Method>,
    params: Params,
}

impl InputData {
    fn has_async_functions(&self) -> bool {
        self.methods.iter().any(|x| x.r#async)
    }
}

pub struct ProxyImpl {
    path: syn::TypePath,
    generics: syn::Generics,
}

pub enum Proxy {
    Struct(syn::Ident),
    Trait(syn::Ident),
    Impl(ProxyImpl),
}

pub struct Params {
    visibility: syn::Visibility,
    returnval: Option<syn::Type>,
    proxies: Vec<Proxy>,
    enum_attr: Vec<syn::Attribute>,
    enum_name: Ident,
}

pub mod generate;
pub mod parse_args;
pub mod parse_input;

pub fn ctrlgen_impl(attrs: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let params = syn::parse2(attrs)?;

    let mut ret = TokenStream::new();
    let mut imp: syn::ItemImpl = syn::parse2(input)?;
    let input_data = InputData::parse_inherent_impl(&mut imp, params)?;

    ret.extend(input_data.generate_enum());
    ret.extend(input_data.generate_call_impl());
    ret.extend(input_data.generate_proxies());
    ret.extend(quote::quote! {#imp});

    syn::Result::<TokenStream>::Ok(ret)
}
