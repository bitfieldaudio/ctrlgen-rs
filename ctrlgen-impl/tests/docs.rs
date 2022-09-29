use std::fmt::Display;

use ctrlgen_impl::InputData;
use ctrlgen_impl::Params;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_quote;
use syn::ItemImpl;

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
    let params: Params = parse_quote! {
        enum Msg
    };
    let mut block: ItemImpl = parse_quote! {
      impl Struct {
        /// Foo function
        fn foo(&mut self) {}
      }
    };
    let input = InputData::parse_inherent_impl(&mut block, params).unwrap();
    let generated: syn::ItemEnum = syn::parse2(input.generate_enum()).unwrap();

    let expected: syn::ItemEnum = parse_quote! {
        enum Msg {
            /// Foo function
            Foo {},
        }
    };

    assert_eq!(generated, expected)
}

#[test]
fn preserve_documentation_proxy() {
    let params: Params = parse_quote! {
        enum Msg, trait Proxy
    };
    let mut block: ItemImpl = parse_quote! {
      impl Struct {
        /// Foo function
        fn foo(&mut self) {}
      }
    };

    let _input = InputData::parse_inherent_impl(&mut block, params).unwrap();
    // syn::Item
    // let generated: syn::Item= syn::parse2(input.generate_proxies()).unwrap();

    // match &generated.items[0] {
    //     syn::TraitItem::Method(x) => {
    //         let doc = x.attrs[0].clone();
    //         let expected: syn::Attribute = parse_quote! {
    //             #[doc = r" Foo function"]
    //         };
    //         assert_eq!(doc, expected)
    //     }
    //     _ => panic!("Expected only a method"),
    // }
}
