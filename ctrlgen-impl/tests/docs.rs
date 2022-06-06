use std::fmt::Display;

use ctrlgen_impl::ctrlgen_impl;
use proc_macro2::TokenStream;
use quote::quote as q;
use quote::ToTokens;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_quote;

#[derive(PartialEq, Eq)]
struct Items(Vec<syn::Item>);

impl Parse for Items {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self(items))
    }
}

impl std::fmt::Debug for Items {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tokens = TokenStream::new();
        for item in &self.0 {
            item.to_tokens(&mut tokens)
        }
        Display::fmt(&tokens, f)
    }
}

#[test]
fn preserve_documentation_enum() {
    let params = q! {
        Msg
    };
    let block = q! {
      impl Struct {
        /// Foo function
        fn foo(&mut self) {}
      }
    };

    let generated: Items = syn::parse2(ctrlgen_impl(params, block)).unwrap();
    let expected: Items = parse_quote! {
        enum Msg {
            /// Foo function
            Foo {},
        }

        impl ::ctrlgen::CallMut<Struct> for Msg {
            type Output = ();
            fn call_mut (self, this: &mut Struct) -> Self::Output {
                match self {
                    Self::Foo {} => { this.foo (); () }
                }
            }
        }
        impl Struct {
          /// Foo function
          fn foo(&mut self) {}
        }
    };

    assert_eq!(generated, expected)
}

#[test]
fn preserve_documentation_proxy() {
    let params = q! {
        Msg, proxy = Proxy
    };
    let block = q! {
      impl Struct {
        /// Foo function
        fn foo(&mut self) {}
      }
    };

    let generated: Items = syn::parse2(ctrlgen_impl(params, block)).unwrap();
    let expected: Items = parse_quote! {
        enum Msg {
            /// Foo function
            Foo {},
        }

        impl ::ctrlgen::CallMut<Struct> for Msg {
            type Output = ();
            fn call_mut (self, this: &mut Struct) -> Self::Output {
                match self {
                    Self::Foo {} => { this.foo (); () }
                }
            }
        }

        struct Proxy<Sender: ::ctrlgen::MessageSender<Msg>> {
            sender: Sender
        }
        impl<Sender: ::ctrlgen::MessageSender<Msg>> Proxy<Sender> {
            fn new(sender: Sender) -> Self {
                Self { sender }
            }
            /// Foo function
            fn foo(&self, ) {
                let msg = Msg::Foo {};
                self.sender.send(msg);
            }
        }   
        
        impl Struct {
          /// Foo function
          fn foo(&mut self) {}
        }

    };

    assert_eq!(generated, expected)
}

