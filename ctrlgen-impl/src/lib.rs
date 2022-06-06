use convert_case::Casing;
use proc_macro2::TokenStream;
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

struct InputData {
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

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum AccessMode {
    Priv,
    Pub,
    PubCrate,
}

impl AccessMode {
    pub(crate) fn code(self) -> TokenStream {
        match self {
            AccessMode::Priv => q! {},
            AccessMode::Pub => q! {pub},
            AccessMode::PubCrate => q! {pub(crate)},
        }
    }
}

struct Params {
    access_mode: AccessMode,
    returnval: Option<syn::Type>,
    proxy: Option<syn::Ident>,
    enum_attr: Vec<syn::Attribute>,
    enum_name: Ident,
}

mod generate;
mod parse_args;
mod parse_input;

pub fn ctrlgen_impl(
    attrs: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let input: TokenStream = item.into();
    let attrs: TokenStream = attrs.into();

    let params = parse_args::parse_args(attrs);

    let mut ret = TokenStream::new();
    let mut imp: syn::ItemImpl = syn::parse2(input).unwrap();
    let input_data = InputData::parse_inherent_impl(&mut imp, params);

    let params = &input_data.params;

    //dbg!(thetrait);
    input_data.generate_enum(&mut ret);

    input_data.generate_call_impl(&mut ret);

    if let Some(proxy) = &params.proxy {
        input_data.generate_proxy(&mut ret, proxy);
    }
    ret.extend(quote::quote! {#imp});

    ret.into()
}
