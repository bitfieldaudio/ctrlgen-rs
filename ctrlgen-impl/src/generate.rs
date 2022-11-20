use proc_macro2::TokenStream;
use quote::{quote as q, quote_spanned};
use syn::{parse_quote, punctuated::Punctuated, Token, WhereClause};

#[cfg(feature = "std")]
fn borrow_toowned() -> TokenStream {
    q! {::std::borrow::ToOwned}
}
#[cfg(all(feature = "alloc", not(feature = "std")))]
fn borrow_toowned() -> TokenStream {
    q! {::alloc::borrow::ToOwned}
}
#[cfg(all(not(feature = "alloc"), not(feature = "std")))]
fn borrow_toowned() -> TokenStream {
    panic!("Cannot use borrow::ToOwned without either `std` or `alloc` features of ctrlgen")
}

use crate::{Proxy};

use super::InputData;
impl InputData {
    pub fn make_where_clause(&self) -> WhereClause {
        let mut where_clause = self
            .generics
            .where_clause
            .clone()
            .unwrap_or_else(|| WhereClause {
                where_token: <Token![where]>::default(),
                predicates: Punctuated::new(),
            });
        if let Some(returnval_trait) = self.params.returnval.as_ref() {
            where_clause.predicates.push(parse_quote! {
                #returnval_trait : ::ctrlgen::Returnval
            })
        }
        where_clause
    }

    pub fn generate_enum(&self) -> TokenStream {
        let returnval_handler = self.params.returnval.as_ref();
        let custom_attrs = &self.params.enum_attr[..];
        let visibility = &self.params.visibility;
        let enum_name = &self.params.enum_name;
        let mut variants = TokenStream::new();
        for method in &self.methods {
            let variant_name = method.variant_name();
            let mut variant_params = TokenStream::new();
            let doc_attr = &method.doc_attr;
            for arg in &method.args {
                let argument_name = &arg.name;
                let argument_type = if !arg.to_owned {
                    let ty = &arg.ty;
                    q! {#ty}
                } else {
                    match &arg.ty {
                        syn::Type::Reference(r) => {
                            let ty = &*r.elem;
                            let toowned = borrow_toowned();
                            q! {<#ty as #toowned>::Owned}
                        }
                        _ => panic!(
                            "Argument marked with `#[ctrlgen_to_owned]` must be a &reference"
                        ),
                    }
                };
                let mut custom_attributes = TokenStream::new();
                for aa in &arg.enum_attr {
                    custom_attributes.extend(q! {# #aa});
                }
                variant_params.extend(q! {
                    #custom_attributes #argument_name : #argument_type,
                });
            }
            if let Some(return_type) = &method.ret {
                let mut custom_attributes = TokenStream::new();
                for aa in &method.return_attr {
                    custom_attributes.extend(q! {# #aa});
                }
                if let Some(returnval_trait) = returnval_handler {
                    variant_params.extend(q! {
                        #custom_attributes ret : <#returnval_trait as ::ctrlgen::Returnval>::Sender<#return_type>,
                    });
                }
            } else {
                if !method.return_attr.is_empty() {
                    panic!("`ctrlgen_return_attr[]` used in method without a return type. Add `-> ()` to force using the return channel.");
                }
            }
            let custom_attributes = &method.enum_attr;

            variants.extend(q! {
                #(#doc_attr)*
                #(#custom_attributes)*
                #variant_name { #variant_params },
            });
        }
        let maybe_where = if let Some(returnval_trait) = returnval_handler {
            q! {
                where #returnval_trait : ::ctrlgen::Returnval
            }
        } else {
            Default::default()
        };
        q! {
            #(#custom_attrs)*
            #visibility enum #enum_name
            #maybe_where
            {
                #variants
            }
        }
    }

    pub fn generate_call_impl(&self) -> TokenStream {
        let returnval_handler = self.params.returnval.as_ref();
        let struct_name = &self.name;
        let enum_name = &self.params.enum_name;
        let is_async = self.has_async_functions();

        let error_type = if let Some(returnval_trait) = returnval_handler {
            q! {
                <#returnval_trait as ::ctrlgen::Returnval>::SendError
            }
        } else {
            q! { ::core::convert::Infallible }
        };

        let mut cases = TokenStream::new();

        for method in &self.methods {
            let method_name = &method.name;
            let variant_name = method.variant_name();
            let mut args = TokenStream::new();
            for arg in &method.args {
                let arg_name = &arg.name;
                args.extend(q! {
                    #arg_name,
                })
            }
            let call_args = args.clone();

            let func_call = if method.r#async {
                q! { this.#method_name(#call_args).await }
            } else {
                q! { this.#method_name(#call_args) }
            };

            let mut body = TokenStream::new();
            if let (Some(_), Some(returnval_trait)) = (&method.ret, returnval_handler) {
                args.extend(q! { ret, });
                body.extend(q! {
                    <#returnval_trait as ::ctrlgen::Returnval>::send(ret, #func_call)
                });
            } else {
                body.extend(q! {
                    #func_call;
                    Ok(())
                });
            }

            cases.extend(q! {
                Self::#variant_name { #args } => {
                    #body
                }
            })
        }

        let (impl_generics, _, _) = &self.generics.split_for_impl();
        let struct_args = &self.struct_args;
        let where_clause = self.make_where_clause();

        if !is_async {
            q! {
                impl #impl_generics ::ctrlgen::CallMut < #struct_name #struct_args > for #enum_name
                #where_clause
                {
                    type Error = #error_type;
                    fn call_mut(self, this: &mut #struct_name #struct_args) -> ::core::result::Result<(), Self::Error> {
                        match self {
                            #cases
                        }
                    }
                }
            }
        } else {
            q! {
                impl #impl_generics ::ctrlgen::CallMutAsync < #struct_name #struct_args > for #enum_name
                #where_clause
                {
                    type Error = #error_type;
                    type Future<'__ctrlgen__lifetime> = impl core::future::Future<Output = ::core::result::Result<(), Self::Error>> + '__ctrlgen__lifetime
                        where #struct_name #struct_args: '__ctrlgen__lifetime;
                    fn call_mut_async<'__ctrlgen__lifetime>(self, this: &'__ctrlgen__lifetime mut #struct_name #struct_args) -> Self::Future<'__ctrlgen__lifetime> {
                        async move {
                            match self {
                                #cases
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn generate_proxies(&self) -> TokenStream {
        let mut res = TokenStream::new();
        for proxy in self.params.proxies.iter() {
            res.extend(self.generate_proxy(proxy));
        }
        res
    }

    pub fn generate_proxy(&self, proxy: &Proxy) -> TokenStream {
        match proxy {
            crate::Proxy::Trait(kwd, x) => {
                self.generate_proxy_trait(kwd, x)
            }
        }
    }

    pub fn generate_proxy_trait(&self, kwd: &Token![trait], trait_: &syn::Ident) -> TokenStream {
        let returnval_handler = self.params.returnval.as_ref();
        let proxy_name = trait_;
        let enum_name = &self.params.enum_name;
        let visibility = &self.params.visibility;

        let mut methods = TokenStream::new();

        for method in &self.methods {
            let method_name = &method.name;
            let variant_name = method.variant_name();
            let mut args = TokenStream::new();
            let mut arg_names = TokenStream::new();
            let doc_attr = &method.doc_attr;
            for arg in &method.args {
                let arg_name = &arg.name;
                let arg_type = &arg.ty;
                args.extend(q! {
                    #arg_name: #arg_type,
                });
                arg_names.extend(q! {
                    #arg_name,
                });
            }
            let span = method.name.span();
            if let (Some(ret), Some(returnval_trait)) = (&method.ret, returnval_handler) {
                methods.extend(quote_spanned! { span=>
                    #(#doc_attr)*
                    fn #method_name(&self, #args) -> <#returnval_trait as ::ctrlgen::Returnval>::RecvResult<#ret> {
                        let ret = <#returnval_trait as ::ctrlgen::Returnval>::create();
                        let msg = #enum_name::#variant_name { #arg_names ret: ret.0 };
                        <Self as ::ctrlgen::Proxy<#enum_name>>::send(self, msg);
                        <#returnval_trait as ::ctrlgen::Returnval>::recv(ret.1)                        
                    }
                })
            } else {
                methods.extend(quote_spanned! { span=>
                    #(#doc_attr)*
                    fn #method_name(&self, #args) {
                        let msg = #enum_name::#variant_name { #arg_names };
                        <Self as ::ctrlgen::Proxy<#enum_name>>::send(self, msg);
                    }
                })
            }
        }

        q! {
            #visibility #kwd #proxy_name: ::ctrlgen::Proxy<#enum_name> {
                #methods
            }

            impl< T : ::ctrlgen::Proxy<#enum_name>> #proxy_name for T {}
        }
    }
}
