use crate::ProxyImpl;
use syn::bracketed;
use syn::parse::Parse;
use syn::Attribute;
use syn::Token;

use super::Params;

impl Parse for Params {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let visibility: Option<syn::Visibility> = if input.fork().parse::<syn::Visibility>().is_ok()
        {
            input.parse().ok()
        } else {
            None
        };
        let visibility = visibility.unwrap_or(syn::Visibility::Inherited);

        let enum_name: syn::Ident = input.parse()?;
        let mut returnval = None;
        let mut proxy = None;
        let mut enum_attr = Vec::new();
        let mut proxy_impl = None;

        while input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
            if input.is_empty() {
                // Allow trailing comma
                break;
            }
            let arg: syn::Ident = input.parse()?;
            match arg.to_string().as_str() {
                "enum_attr" => {
                    let content;
                    enum_attr.push(Attribute {
                        pound_token: Token![#](input.span()),
                        style: syn::AttrStyle::Outer,
                        bracket_token: bracketed!(content in input),
                        path: content.call(syn::Path::parse_mod_style)?,
                        tokens: content.parse()?,
                    })
                }
                "returnval" => {
                    if returnval.is_some() {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "Argument `returnval` specified twice",
                        ));
                    }
                    let _eq: Token![=] = input.parse()?;
                    returnval = Some(input.parse()?)
                }
                "proxy" => {
                    if proxy.is_some() {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "Argument `proxy` specified twice",
                        ));
                    }
                    let _eq: Token![=] = input.parse()?;
                    proxy = Some(input.parse()?);
                }
                "proxy_impl" => {
                    if proxy_impl.is_some() {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "Argument `proxy_impl` specified twice",
                        ));
                    }
                    let generics = input.parse()?;
                    let _eq: Token![=] = input.parse()?;
                    let path = input.parse()?;
                    proxy_impl = Some(ProxyImpl { path, generics });
                }
                _ => {
                    return Err(syn::Error::new(
                        arg.span(),
                        format!("Unknown argument to ctrlgen {arg}"),
                    ))
                }
            };
        }

        Ok(Self {
            visibility,
            enum_name,
            returnval,
            proxy,
            proxy_impl,
            enum_attr,
        })
    }
}
