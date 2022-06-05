use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::TokenStreamExt;

use crate::AccessMode;

use super::Params;
enum ParserState<I, G> {
    ExpectingName,
    ExpectingNewParam,
    ExpectingIdent(I),
    ExpectingEqsign(I),
    ExpectingGroup(G),
}

#[derive(Debug, Clone, Copy)]
enum RootLevelIdentAssignmentTargets {
    Returnval,
    Proxy,
}
#[derive(Debug, Clone, Copy)]
enum RootLevelGroupAssignmentTargets {
    CustomAttr,
}

pub(crate) fn parse_args(input: TokenStream) -> Params {
    let mut proxy = None;
    let mut access_mode = AccessMode::Priv;
    let mut returnval = TokenStream::new();
    let mut enum_attr = vec![];
    let mut enum_name = None;

    let mut state = ParserState::<RootLevelIdentAssignmentTargets,RootLevelGroupAssignmentTargets>::ExpectingName;

    use ParserState::*;
    use RootLevelGroupAssignmentTargets::*;
    use RootLevelIdentAssignmentTargets::*;

    for x in input {
        match state {
            ExpectingName => match x {
                TokenTree::Ident(y) => match y.to_string().as_str() {
                    "pub" => access_mode = AccessMode::Pub,
                    "pub_crate" => access_mode = AccessMode::PubCrate,
                    _ => {
                        enum_name = Some(y);
                        state = ExpectingNewParam;
                    }
                },
                _ => panic!("Expected enum name or visibility as first parameter"),
            },
            ExpectingNewParam => match x {
                TokenTree::Ident(y) => match y.to_string().as_str() {
                    "returnval" => state = ExpectingEqsign(Returnval),
                    "proxy" => state = ExpectingEqsign(Proxy),
                    "enum_attr" => state = ExpectingGroup(CustomAttr),
                    z => panic!("Unknown parameter {}", z),
                },
                TokenTree::Group(_) => panic!("No group is expected here"),
                TokenTree::Punct(y) if y.as_char() == ',' => (),
                TokenTree::Punct(_) => panic!("No punctuation is expected here"),
                TokenTree::Literal(_) => panic!("No literal is expected here"),
            },
            ExpectingIdent(Returnval) => match x {
                TokenTree::Punct(y) if y.as_char() == ',' => state = ExpectingNewParam,
                x => returnval.append(x),
            },
            ExpectingIdent(t) => {
                match x {
                    TokenTree::Ident(y) => match t {
                        Returnval => unreachable!(),
                        Proxy => proxy = Some(y),
                    },
                    _ => panic!(
                        "Single identifier is expected in {:?} state after `=` sign",
                        t
                    ),
                }
                state = ExpectingNewParam;
            }
            ExpectingEqsign(t) => match x {
                TokenTree::Punct(y) if y.as_char() == '=' => state = ExpectingIdent(t),
                _ => panic!("Expected `=` character after parameter for {:?}", t),
            },
            ExpectingGroup(t) => {
                match x {
                    TokenTree::Group(y) => match t {
                        CustomAttr => enum_attr.push(y),
                    },
                    _ => panic!("Expected a group after parameter for {:?}", t),
                }
                state = ExpectingNewParam
            }
        }
    }

    let enum_name = enum_name.expect("`name` parameter is required.");

    let returnval = if returnval.is_empty() {
        None
    } else {
        Some(syn::parse(returnval.into()).unwrap())
    };

    Params {
        proxy,
        // call_fns,
        access_mode,
        returnval,
        enum_attr,
        enum_name,
    }
}
