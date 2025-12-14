use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

struct FieldInfo {
    name: syn::Ident,
    ty: syn::Type,
}

#[proc_macro_derive(DecodeOps)]
pub fn derive_decode_ops(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = input.ident;

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new_spanned(enum_name, "DecodeOps only supports enums")
            .to_compile_error()
            .into();
    };

    let discr_name = format_ident!("{}Discriminants", enum_name);

    let mut discr_variants = Vec::new();
    let mut match_arms = Vec::new();

    for variant in &data_enum.variants {
        let v_ident = &variant.ident;
        let attrs = &variant.attrs;

        discr_variants.push(quote! {
            #(#attrs)*
            #v_ident
        });

        let arm = match &variant.fields {
            Fields::Unit => {
                quote! {
                    #(#attrs)*
                    #discr_name::#v_ident => Ok(#enum_name::#v_ident)
                }
            }
            Fields::Named(fields) => {
                let field_idents: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();

                let field_tys: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

                if field_idents.len() == 1 {
                    let f = field_idents[0];
                    quote! {
                        #(#attrs)*
                        #discr_name::#v_ident => {
                            let #f = vm.read_pc()?;
                            Ok(#enum_name::#v_ident { #f })
                        }
                    }
                } else {
                    quote! {
                        #(#attrs)*
                        #discr_name::#v_ident => {
                            #[repr(C, packed)]
                            #[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
                            struct __Tmp {
                                #(
                                    #field_idents: #field_tys
                                ),*
                            }

                            let tmp: __Tmp = vm.read_pc()?;

                            Ok(#enum_name::#v_ident {
                                #(
                                    #field_idents: tmp.#field_idents
                                ),*
                            })
                        }
                    }
                }
            }
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(
                    v_ident,
                    "DecodeOps does not support tuple variants",
                )
                .to_compile_error()
                .into();
            }
        };

        match_arms.push(arm);
    }

    let expanded = quote! {
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        pub enum #discr_name {
            #(#discr_variants),*
        }

        impl #discr_name {
            pub fn to_op<const N: usize, S: Sync, D: VmDebug>(
                &self,
                vm: &mut VM<N, S, D>,
            ) -> Result<#enum_name> {
                match self {
                    #(#match_arms),*
                }
            }
        }
    };

    expanded.into()
}
