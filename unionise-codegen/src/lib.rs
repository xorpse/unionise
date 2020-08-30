extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};

#[proc_macro_derive(Unionise, attributes(unionise))]
pub fn unionise_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemEnum);
    let name = &input.ident;
    let vis = &input.vis;

    let union_name = format_ident!("C{}", name);

    let simple_reprc = input.variants
        .iter()
        .all(|v| {
            use syn::Fields::*;
            match &v.fields {
                Unit => true,
                _ => false,
            }
        });

    if simple_reprc {
        let mut from_fields = Vec::new();
        let mut to_fields = Vec::new();
        let mut variants = Vec::new();

        for v in input.variants.iter() {
            let ident = &v.ident;
            from_fields.push(quote! { #name::#ident => #union_name::#ident });
            to_fields.push(quote! { #union_name::#ident => #name::#ident });
            variants.push(quote! { #ident });
        }

        let imple = quote! {
            #[derive(Clone, Copy)]
            #[repr(C)]
            #vis enum #union_name {
                #(#variants),*
            }

            impl From<#name> for #union_name {
                fn from(v: #name) -> Self {
                    match v {
                        #(#from_fields),*
                    }
                }
            }

            impl From<#union_name> for #name {
                fn from(v: #union_name) -> Self {
                    match v {
                        #(#to_fields),*
                    }
                }
            }
        };

        imple.into_token_stream().into()
    } else {
        let union_tag_name = format_ident!("C{}Tag", name);
        let union_val_name = format_ident!("C{}Val", name);

        let mut from_fields = Vec::new();
        let mut to_fields = Vec::new();
        let mut tag_names = Vec::new();
        let mut field_types = Vec::new();
        let mut field_names_types = Vec::new();

        for v in input.variants.iter() {
            use syn::Fields::*;

            let ident = &v.ident;
            //from_fields.push(quote! { #name::#ident => #union_name::#ident });
            //to_fields.push(quote! { #union_name::#ident => #name::#ident });
            tag_names.push(quote! { #ident });

            let field_ident = format_ident!("C{}Field_{}", name, ident);
            let field_name = format_ident!("field_{}", ident);

            // TODO: unionise the fields for non primitive types
            // based on annotated #[unionise(into = path)]

            match &v.fields {
                Unit => {
                    field_types.push(quote! {
                        #[derive(Clone, Copy)]
                        #[repr(C)]
                        #vis struct #field_ident { }
                    });

                    from_fields.push(quote! {
                        #name::#ident => #union_name {
                            tag: #union_tag_name::#ident,
                            val: #union_val_name { #field_name: #field_ident { } },
                        }
                    });

                    to_fields.push(quote! {
                        #union_name { tag: #union_tag_name::#ident, .. } => #name::#ident
                    });
                },
                vs@Unnamed(_) => {
                    let mut fields = Vec::new();
                    let mut from_to_inner_fields = Vec::new();
                    let mut to_from_inner_fields_to = Vec::new();
                    let mut to_from_inner_fields_from = Vec::new();

                    for (i, f) in vs.iter().enumerate() {
                        let field_id = format_ident!("field_{}", i);
                        let attrs = &f.attrs;
                        match attrs.iter().find(|attr| attr.path.is_ident("unionise")) {
                            Some(attr) => {
                                let f = attr.parse_args::<syn::Path>().unwrap();
                                fields.push(quote! { #field_id: #f });
                                from_to_inner_fields.push(quote! { #field_id });
                                to_from_inner_fields_from.push(quote! { #field_id.into() });
                                to_from_inner_fields_to.push(quote! { #field_id: #field_id.into() });
                            },
                            None => {
                                fields.push(quote! { #field_id: #f });
                                from_to_inner_fields.push(quote! { #field_id });
                                to_from_inner_fields_from.push(quote! { #field_id });
                                to_from_inner_fields_to.push(quote! { #field_id });
                            },
                        }
                    }

                    field_types.push(quote! {
                        #[derive(Clone, Copy)]
                        #[repr(C)]
                        #vis struct #field_ident {
                            #(#fields),*
                        }
                    });

                    from_fields.push(quote! {
                        #name::#ident(#(#from_to_inner_fields),*) => #union_name {
                            tag: #union_tag_name::#ident,
                            val: #union_val_name {
                                #field_name: #field_ident {
                                    #(#to_from_inner_fields_to),*
                                },
                            },
                        }
                    });

                    to_fields.push(quote! {
                        #union_name {
                            tag: #union_tag_name::#ident,
                            val: #union_val_name { #field_name: #field_ident { #(#from_to_inner_fields),* } }
                        } => #name::#ident(
                            #(#to_from_inner_fields_from),*
                        )
                    });
                },
                vs@Named(_) => {
                    let mut fields = Vec::new();
                    let mut from_to_inner_fields = Vec::new();
                    let mut to_from_inner_fields = Vec::new();

                    for v in vs.iter() {
                        let field_id = &v.ident;
                        let attrs = &v.attrs;
                        match attrs.iter().find(|attr| attr.path.is_ident("unionise")) {
                            Some(attr) => {
                                let f = attr.parse_args::<syn::Path>().unwrap();
                                fields.push(quote! { #field_id: #f });
                                from_to_inner_fields.push(quote! { #field_id });
                                to_from_inner_fields.push(quote! { #field_id: #field_id.into() });
                            },
                            None => {
                                let f = &v.ty;
                                fields.push(quote! { #field_id: #f });
                                from_to_inner_fields.push(quote! { #field_id });
                                to_from_inner_fields.push(quote! { #field_id });
                            },
                        }
                    }

                    field_types.push(quote! {
                        #[derive(Clone, Copy)]
                        #[repr(C)]
                        #vis struct #field_ident {
                            #(#fields),*
                        }
                    });

                    from_fields.push(quote! {
                        #name::#ident { #(#from_to_inner_fields),* } => #union_name {
                            tag: #union_tag_name::#ident,
                            val: #union_val_name { #field_name: #field_ident { #(#to_from_inner_fields),* } },
                        }
                    });

                    to_fields.push(quote! {
                        #union_name {
                            tag: #union_tag_name::#ident,
                            val: #union_val_name { #field_name: #field_ident { #(#from_to_inner_fields),* } }
                        } => #name::#ident {
                            #(#to_from_inner_fields),*
                        }
                    });
                },
            }
            field_names_types.push(quote!( #field_name: #field_ident ));
        }

        let imple = quote! {
            #[derive(Clone, Copy)]
            #[repr(C)]
            #vis enum #union_tag_name {
                #(#tag_names,)*
            }

            #(#field_types)*

            #[derive(Clone, Copy)]
            #[repr(C)]
            #[allow(non_snake_case)]
            #vis union #union_val_name {
                #(#field_names_types),*
            }

            #[derive(Clone, Copy)]
            #[repr(C)]
            #vis struct #union_name {
                tag: #union_tag_name,
                val: #union_val_name,
            }

            impl From<#name> for #union_name {
                fn from(v: #name) -> Self {
                    unsafe {
                        match v {
                            #(#from_fields),*
                        }
                    }
                }
            }

            impl From<#union_name> for #name {
                fn from(v: #union_name) -> Self {
                    unsafe {
                        match v {
                            #(#to_fields),*
                        }
                    }
                }
            }
        };
        imple.into_token_stream().into()
    }
}
