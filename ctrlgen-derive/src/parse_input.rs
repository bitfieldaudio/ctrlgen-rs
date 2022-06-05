use proc_macro2::TokenTree;

use crate::{Argument, Method, Params};

use super::{InputData, ReceiverStyle};
impl InputData {
    pub(crate) fn parse_inherent_impl(item: &mut syn::ItemImpl, params: Params) -> InputData {
        let returnval_mode = params.returnval.is_some();

        if item.defaultness.is_some() {
            panic!("Default impls not supported");
        }
        if item.unsafety.is_some() {
            panic!("Handling `unsafe` is not implemented");
        }
        if item.trait_.is_some() {
            panic!("Trait impls are not supported, only inherent impls");
        }
        let generics = item.generics.clone();
        let (name, struct_args) = match &*item.self_ty {
            syn::Type::Path(p) => {
                if p.qself.is_some() {
                    panic!("Impl has some tricky type. This is not supported");
                }
                if p.path.segments.len() != 1 {
                    panic!("Impl type must be a single ident with optional arguments")
                }
                let segment = p.path.segments[0].clone();
                (segment.ident, segment.arguments)
            }
            _ => panic!(
                "Type for `impl` should be a simple identifier without any paths or other tricks."
            ),
        };

        let mut methods = Vec::with_capacity(item.items.len());

        for item in &mut item.items {
            match item {
                syn::ImplItem::Method(method) => {
                    if method.defaultness.is_some() {
                        panic!("`default` not supported");
                    }

                    parse_method(
                        &mut method.sig,
                        &mut method.attrs,
                        returnval_mode,
                        &mut methods,
                    );
                }
                _ => (),
            }
        }

        InputData {
            name,
            generics,
            struct_args,
            methods,
            params,
        }
    }
}

fn parse_method(
    method_signature: &mut syn::Signature,
    attrs: &mut Vec<syn::Attribute>,
    returnval_mode: bool,
    methods: &mut Vec<Method>,
) {
    let mut enum_attr = vec![];
    let mut return_attr = vec![];
    if method_signature.constness.is_some() {
        panic!("ctrlgen does not support const");
    }
    let r#async = method_signature.asyncness.is_some();
    if method_signature.unsafety.is_some() {
        panic!("ctrlgen does not support unsafe");
    }
    if method_signature.abi.is_some() {
        panic!("ctrlgen does not support custom ABI in trait methods")
    }
    if !method_signature.generics.params.is_empty() {
        panic!("ctrlgen does not support generics or lifetimes in trait methods")
    }
    if method_signature.variadic.is_some() {
        panic!("ctrlgen does not support variadics")
    }
    if !returnval_mode && !matches!(method_signature.output, syn::ReturnType::Default) {
        panic!("Specify `returnval` parameter to handle methods with return types.")
    }
    attrs.retain(|a| match a.path.get_ident() {
        Some(x) if x == "ctrlgen_enum_attr" || x == "ctrlgen_return_attr" => {
            let g = match a.tokens.clone().into_iter().next() {
                Some(TokenTree::Group(g)) => g,
                _ => {
                    panic!("Input of `ctrlgen_{{enum|return}}_attr` should be single [...] group")
                }
            };
            match x {
                x if x == "ctrlgen_enum_attr" => enum_attr.push(g),
                x if x == "ctrlgen_return_attr" => return_attr.push(g),
                _ => unreachable!(),
            }
            false
        }
        _ => true,
    });
    let mut args = Vec::with_capacity(method_signature.inputs.len());
    let mut receiver_style = None;
    let ret = match &method_signature.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, t) => Some(*t.clone()),
    };
    for input_args in &mut method_signature.inputs {
        match input_args {
            syn::FnArg::Receiver(r) => {
                receiver_style = if let Some(rr) = &r.reference {
                    if rr.1.is_some() {
                        panic!("ctrlgen does not support explicit lifetimes");
                    }
                    if r.mutability.is_some() {
                        Some(ReceiverStyle::Mut)
                    } else {
                        Some(ReceiverStyle::Ref)
                    }
                } else {
                    Some(ReceiverStyle::Move)
                }
            }
            syn::FnArg::Typed(arg) => {
                let mut enum_attr = vec![];
                let mut to_owned = false;
                arg.attrs.retain(|a| match a.path.get_ident() {
                    Some(x) if x == "ctrlgen_enum_attr" => {
                        match a.tokens.clone().into_iter().next() {
                            Some(TokenTree::Group(g)) => {
                                enum_attr.push(g);
                            }
                            _ => panic!(
                                "Input of `ctrlgen_enum_attr` should be a single [...] group"
                            ),
                        }
                        false
                    }
                    Some(x) if x == "ctrlgen_to_owned" => {
                        if !a.tokens.is_empty() {
                            panic!("`ctrlgen_to_owned` does not accept any additional arguments");
                        }
                        to_owned = true;
                        false
                    }
                    _ => true,
                });
                match &*arg.pat {
                    syn::Pat::Ident(pi) => {
                        if pi.by_ref.is_some() {
                            panic!("ctrlgen does not support `ref` in argument names");
                        }
                        if returnval_mode {
                            if pi.ident.to_string() == "ret" {
                                panic!("In `returnval` mode, method's arguments cannot be named literally `ret`. Rename it away in `{}`.", method_signature.ident);
                            }
                        }
                        args.push(Argument { name: pi.ident.clone(), ty: *arg.ty.clone(), enum_attr, to_owned });
                    }
                    _ => panic!("ctrlgen does not support method arguments that are patterns, not just simple identifiers"),
                }
            }
        }
    }
    if receiver_style.is_none() {
        panic!("ctrlgen does not support methods that do not accept `self`")
    }
    let method = Method {
        args,
        name: method_signature.ident.clone(),
        receiver_style: receiver_style.unwrap(),
        ret,
        enum_attr,
        return_attr,
        r#async,
    };
    methods.push(method);
}
