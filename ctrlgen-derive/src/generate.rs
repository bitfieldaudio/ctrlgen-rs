use proc_macro2::{Ident, TokenStream};
use quote::quote as q;

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

use super::InputData;
impl InputData {
    pub(crate) fn generate_enum(&self, out: &mut TokenStream) {
        let returnval_handler = self.params.returnval.as_ref();
        let custom_attrs = &self.params.enum_attr[..];
        let pub_or_priv = self.params.access_mode.code();
        let enum_name = &self.params.enum_name;
        let mut variants = TokenStream::new();
        for method in &self.methods {
            let variant_name = method.variant_name();
            let mut variant_params = TokenStream::new();
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
            let mut custom_attributes = TokenStream::new();
            for aa in &method.enum_attr {
                custom_attributes.extend(q! {# #aa});
            }

            variants.extend(q! {
                #custom_attributes #variant_name { #variant_params },
            });
        }
        let mut customattrs = TokenStream::new();
        for ca in custom_attrs {
            customattrs.extend(q! {# #ca});
        }
        let mut maybe_where = TokenStream::new();
        if let Some(returnval_trait) = returnval_handler {
            maybe_where = q! {
                where #returnval_trait : ::ctrlgen::Returnval
            }
        }
        out.extend(q! {
            #customattrs
            #pub_or_priv enum #enum_name
            #maybe_where
            {
                #variants
            }
        });
    }

    pub(crate) fn generate_call_impl(&self, out: &mut TokenStream) {
        let returnval_handler = self.params.returnval.as_ref();
        let struct_name = &self.name;
        let enum_name = &self.params.enum_name;
        let is_async = self.has_async_functions();
        
        let output_type = if let Some(returnval_trait) = returnval_handler {
            q! {
                core::result::Result<(), <#returnval_trait as ::ctrlgen::Returnval>::SendError>
            }
        } else {
            q! { () }
        };
        let output_ok = if returnval_handler.is_some() {
            q! {
                Ok(())
            }
        } else {
            q! { () }
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
                    #output_ok
                });
            }

            cases.extend(q! {
                Self::#variant_name { #args } => {
                    #body
                }
            })
        }

        let maybe_where = if let Some(returnval_trait) = returnval_handler {
            q! {
                where #returnval_trait : ::ctrlgen::Returnval
            }
        } else {
            Default::default()
        };

        if !is_async {
            out.extend(q! {
                impl ::ctrlgen::CallMut<#struct_name> for #enum_name
                #maybe_where
                {
                    type Output = #output_type;
                    fn call_mut(self, this: &mut #struct_name) -> Self::Output {
                        match self {
                            #cases
                        }
                    }
                }
            });
        } else {
            out.extend(q! {
                impl ::ctrlgen::CallMutAsync<#struct_name> for #enum_name
                #maybe_where
                {
                    type Future<'a> = impl core::future::Future<Output = #output_type> + 'a
                        where #struct_name: 'a;
                    fn call_mut_async(self, this: &mut #struct_name) -> Self::Future<'_> {
                        async {
                            match self {
                                #cases
                            }
                        }
                    }
                }
            });
        }
    }

    pub(crate) fn generate_proxy(&self, out: &mut TokenStream, proxy_name: &Ident) {
        let returnval_handler = self.params.returnval.as_ref();
        let enum_name = &self.params.enum_name;
        let visibility = self.params.access_mode.code();

        let mut methods = TokenStream::new();

        for method in &self.methods {
            let method_name = &method.name;
            let variant_name = method.variant_name();
            let mut args = TokenStream::new();
            let mut arg_names = TokenStream::new();
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
            if let (Some(ret), Some(returnval_trait)) = (&method.ret, returnval_handler) {
                methods.extend(q! {
                    #visibility fn #method_name(&self, #args) -> <#returnval_trait as ::ctrlgen::Returnval>::RecvResult<#ret> {
                        let ret = <#returnval_trait as ::ctrlgen::Returnval>::create();
                        let msg = #enum_name::#variant_name { #arg_names ret: ret.0 };
                        self.sender.send(msg);
                        <#returnval_trait as ::ctrlgen::Returnval>::recv(ret.1)                        
                    }
                })
            } else {
                methods.extend(q! {
                    #visibility fn #method_name(&self, #args) {
                        let msg = #enum_name::#variant_name { #arg_names };
                        self.sender.send(msg);
                    }
                })
            }
        }
        let maybe_where = if let Some(returnval_trait) = returnval_handler {
            q! {
                where #returnval_trait : ::ctrlgen::Returnval
            }
        } else {
            Default::default()
        };

        out.extend(q! {
            #visibility struct #proxy_name<Sender: ::ctrlgen::MessageSender<#enum_name>> {
                sender: Sender
            }

            impl<Sender: ::ctrlgen::MessageSender<#enum_name>> #proxy_name<Sender>
            #maybe_where
            {
                #visibility fn new(sender: Sender) -> Self {
                    Self { sender }
                }

                #methods
            }
        });
    }
}
