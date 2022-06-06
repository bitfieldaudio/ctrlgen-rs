use proc_macro2::TokenTree;
use syn::{parse_quote, Attribute};

use crate::{Argument, Method, Params};

use super::{InputData, ReceiverStyle};
impl InputData {
    pub fn parse_inherent_impl(item: &mut syn::ItemImpl, params: Params) -> syn::Result<InputData> {
        let returnval_mode = params.returnval.is_some();

        if let Some(x) = item.defaultness {
            return Err(syn::Error::new_spanned(x, "Default impls not supported"));
        }
        if let Some(x) = item.unsafety {
            return Err(syn::Error::new_spanned(
                x,
                "Handling `unsafe` is not implemented",
            ));
        }
        if let Some((_, path, _)) = &item.trait_ {
            return Err(syn::Error::new_spanned(
                path,
                "Trait impls are not supported, only inherent impls",
            ));
        }
        let generics = item.generics.clone();
        let (name, struct_args) = match &*item.self_ty {
            syn::Type::Path(p) => {
                if p.qself.is_some() {
                    return Err(syn::Error::new_spanned(
                        p,
                        "Impl has some tricky type. This is not supported",
                    ));
                }
                if p.path.segments.len() != 1 {
                    return Err(syn::Error::new_spanned(
                        p,
                        "Impl type must be a single ident with optional arguments",
                    ));
                }
                let segment = p.path.segments[0].clone();
                (segment.ident, segment.arguments)
            }
            _ => return Err(syn::Error::new_spanned(
                &*item.self_ty,
                "Type for `impl` should be a simple identifier without any paths or other tricks.",
            )),
        };

        let mut methods = Vec::with_capacity(item.items.len());

        for item in &mut item.items {
            match item {
                syn::ImplItem::Method(method) => {
                    if method.defaultness.is_some() {
                        panic!("`default` not supported");
                    }

                    methods.push(parse_method(
                        &mut method.sig,
                        &mut method.attrs,
                        returnval_mode,
                    )?);
                }
                _ => (),
            }
        }

        Ok(InputData {
            name,
            generics,
            struct_args,
            methods,
            params,
        })
    }
}

fn parse_method(
    method_signature: &mut syn::Signature,
    attrs: &mut Vec<syn::Attribute>,
    returnval_mode: bool,
) -> syn::Result<Method> {
    let mut enum_attr = vec![];
    let mut return_attr = vec![];
    let mut doc_attr = vec![];
    let r#async = method_signature.asyncness.is_some();
    if let Some(x) = method_signature.constness {
        return Err(syn::Error::new_spanned(x, "ctrlgen does not support const"));
    }
    if let Some(x) = method_signature.unsafety {
        return Err(syn::Error::new_spanned(
            x,
            "ctrlgen does not support unsafe",
        ));
    }
    if let Some(x) = &method_signature.abi {
        return Err(syn::Error::new_spanned(
            x,
            "ctrlgen does not support custom ABI in trait methods",
        ));
    }
    if !method_signature.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &method_signature.generics,
            "ctrlgen does not support generics or lifetimes in trait methods",
        ));
    }
    if let Some(x) = &method_signature.variadic {
        return Err(syn::Error::new_spanned(
            x,
            "ctrlgen does not support variadics",
        ));
    }
    if !returnval_mode && !matches!(method_signature.output, syn::ReturnType::Default) {
        return Err(syn::Error::new_spanned(
            &method_signature.output,
            "Specify `returnval` parameter to handle methods with return types.",
        ));
    }
    for a in attrs.iter() {
        match a.path.get_ident() {
            Some(x) if x == "ctrlgen_enum_attr" || x == "ctrlgen_return_attr" => {
                let g = match a.tokens.clone().into_iter().next() {
                    Some(TokenTree::Group(g)) => g,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            a,
                            "Input of `ctrlgen_{{enum|return}}_attr` should be single [...] group",
                        ));
                    }
                };
                let attr: Attribute = parse_quote! { # #g };
                match x {
                    x if x == "ctrlgen_enum_attr" => enum_attr.push(attr),
                    x if x == "ctrlgen_return_attr" => return_attr.push(attr),
                    _ => unreachable!(),
                }
            }
            Some(x) if x == "doc" => {
                doc_attr.push(a.clone());
            }
            _ => (),
        }
    }
    attrs.retain(|a| match a.path.get_ident() {
        Some(x) if x == "ctrlgen_enum_attr" || x == "ctrlgen_return_attr" => false,
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
                    if let Some(x) = &rr.1 {
                        return Err(syn::Error::new_spanned(
                            x,
                            "ctrlgen does not support explicit lifetimes",
                        ));
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
                for a in attrs.iter() {
                    match a.path.get_ident() {
                        Some(x) if x == "ctrlgen_enum_attr" => {
                            match a.tokens.clone().into_iter().next() {
                                Some(TokenTree::Group(g)) => {
                                    enum_attr.push(g);
                                }
                                _ => return Err(syn::Error::new_spanned(
                                    a,
                                    "Input of `ctrlgen_enum_attr` should be a single [...] group",
                                )),
                            }
                        }
                        Some(x) if x == "ctrlgen_to_owned" => {
                            if !a.tokens.is_empty() {
                                return Err(syn::Error::new_spanned(
                                    a,
                                    "`ctrlgen_to_owned` does not accept any additional arguments",
                                ));
                            }
                            to_owned = true;
                        }
                        _ => (),
                    }
                }
                arg.attrs.retain(|a| match a.path.get_ident() {
                    Some(x) if x == "ctrlgen_enum_attr" => false,
                    Some(x) if x == "ctrlgen_to_owned" => false,
                    _ => true,
                });
                match &*arg.pat {
                    syn::Pat::Ident(pi) => {
                        if pi.by_ref.is_some() {
                            return Err(syn::Error::new_spanned(pi, "ctrlgen does not support `ref` in argument names"));
                        }
                        if returnval_mode {
                            if pi.ident.to_string() == "ret" {
                                return Err(syn::Error::new_spanned(&pi.ident, format!("In `returnval` mode, method's arguments cannot be named literally `ret`. Rename it away in `{}`.", method_signature.ident)));
                            }
                        }
                        args.push(Argument { name: pi.ident.clone(), ty: *arg.ty.clone(), enum_attr, to_owned });
                    }
                    _ => return Err(syn::Error::new_spanned(arg, "ctrlgen does not support method arguments that are patterns, not just simple identifiers")),
                }
            }
        }
    }
    if receiver_style.is_none() {
        return Err(syn::Error::new_spanned(
            method_signature,
            "ctrlgen does not support methods that do not accept `self`",
        ));
    }
    Ok(Method {
        args,
        name: method_signature.ident.clone(),
        receiver_style: receiver_style.unwrap(),
        ret,
        enum_attr,
        return_attr,
        doc_attr,
        r#async,
    })
}
