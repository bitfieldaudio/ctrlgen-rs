use syn::bracketed;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::Attribute;
use syn::Token;

use crate::Params;
use crate::Proxy;

impl Parse for Proxy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // if input.peek(Token![struct]) {
        //     let _kwd: Token![struct] = input.parse()?;
        //     Ok(Self::Struct(input.parse()?))
        // } else
        if input.peek(Token![trait]) {
            let kwd: Token![trait] = input.parse()?;
            Ok(Self::Trait(kwd, input.parse()?))
        // } else if input.peek(Token![impl]) {
        //     let _kwd: Token![impl] = input.parse()?;
        //     let generics = input.parse()?;
        //     let path = input.parse()?;
        //     Ok(Self::Impl(ProxyImpl { generics, path }))
        } else {
            Err(syn::Error::new(
                input.span(),
                "Expected `struct`, `trait` or `impl`",
            ))
        }
    }
}

impl Parse for Params {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut enum_attr = Attribute::parse_outer(input)?;

        let visibility: syn::Visibility = input
            .fork()
            .parse::<syn::Visibility>()
            .ok()
            .and_then(|_| input.parse().ok())
            .unwrap_or(syn::Visibility::Inherited);

        let _: Token![enum] = input.parse()?;

        let enum_name: syn::Ident = input.parse()?;
        let mut returnval = None;
        let mut proxies = Vec::new();
        let mut context = None;

        while input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
            if input.is_empty() {
                // Allow trailing comma
                break;
            }
            if input.peek(Token![trait]) {
                proxies.extend(input.parse());
                continue;
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
                    let contents;
                    if input.peek(syn::token::Paren) {
                        let _paren = syn::parenthesized!(contents in input);
                    } else {
                        let _paren = syn::braced!(contents in input);
                    }
                    let punct: Punctuated<Proxy, Token![;]> =
                        Punctuated::parse_terminated(&contents)?;
                    proxies.extend(punct)
                }
                "context" => {
                    if context.is_some() {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "Argument `context` specified twice",
                        ));
                    }
                    let contents;
                    let _paren = syn::parenthesized!(contents in input);

                    let ident = contents.parse()?;
                    let _colon: Token![:] = contents.parse()?;
                    let ty = contents.parse()?;

                    context = Some((ident, ty))
                }
                _ => {
                    return Err(syn::Error::new(
                        arg.span(),
                        format!("Unknown argument `{arg}` to ctrlgen"),
                    ))
                }
            };
        }

        Ok(Self {
            visibility,
            enum_name,
            returnval,
            proxies,
            enum_attr,
            context,
        })
    }
}
