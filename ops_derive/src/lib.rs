use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

struct HandlerInfo {
    path: syn::Path,
    is_async: bool,
}

fn extract_handler_attr(attrs: &[&syn::Attribute]) -> Option<HandlerInfo> {
    for attr in attrs {
        if attr.path().is_ident("handler") {
            if let Ok(path) = attr.parse_args::<syn::Path>() {
                return Some(HandlerInfo {
                    path,
                    is_async: false,
                });
            }
        } else if attr.path().is_ident("async_handler") {
            if let Ok(path) = attr.parse_args::<syn::Path>() {
                return Some(HandlerInfo {
                    path,
                    is_async: true,
                });
            }
        }
    }
    None
}

// Helper to generate parameter reading code for handler dispatch
fn generate_param_reads_and_args(
    fields: &syn::FieldsNamed,
) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
    let field_idents: Vec<_> = fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let field_tys: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

    let args: Vec<_> = field_idents.iter().map(|f| quote! { #f }).collect();

    if field_idents.len() == 1 {
        let f = field_idents[0];
        let reading_code = quote! {
            let #f = vm.read_pc()?;
        };
        (reading_code, args)
    } else {
        let reading_code = quote! {
            #[repr(C, packed)]
            #[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
            struct __Tmp {
                #(
                    #field_idents: #field_tys
                ),*
            }

            let tmp: __Tmp = vm.read_pc()?;
        };

        let args: Vec<_> = field_idents
            .iter()
            .map(|f| quote! { tmp.#f })
            .collect();

        (reading_code, args)
    }
}

#[proc_macro_derive(DecodeOps, attributes(handler, async_handler))]
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
        let filtered_attrs: Vec<_> = variant.attrs
            .iter()
            .filter(|a| {
                !a.path().is_ident("handler")
                    && !a.path().is_ident("async_handler")
            })
            .collect();

        discr_variants.push(quote! {
            #(#filtered_attrs)*
            #v_ident
        });

        let arm = match &variant.fields {
            Fields::Unit => {
                quote! {
                    #(#filtered_attrs)*
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
                        #(#filtered_attrs)*
                        #discr_name::#v_ident => {
                            let #f = vm.read_pc()?;
                            Ok(#enum_name::#v_ident { #f })
                        }
                    }
                } else {
                    quote! {
                        #(#filtered_attrs)*
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

    // Generate run_op() match arms
    let mut handler_match_arms = Vec::new();

    for variant in &data_enum.variants {
        let v_ident = &variant.ident;
        let filtered_attrs: Vec<_> = variant.attrs
            .iter()
            .filter(|a| {
                !a.path().is_ident("handler")
                    && !a.path().is_ident("async_handler")
            })
            .collect();

        // Extract the discriminant value
        let Some((_, discriminant_expr)) = &variant.discriminant else {
            return syn::Error::new_spanned(
                v_ident,
                "All variants must have explicit discriminants for run_op generation",
            )
            .to_compile_error()
            .into();
        };

        // Check if this variant has a handler attribute
        let handler_info = extract_handler_attr(filtered_attrs.as_slice());

        if let Some(handler) = handler_info {
            let handler_path = &handler.path;
            let await_token = if handler.is_async {
                quote! { .await }
            } else {
                quote! {}
            };

            let handler_arm = match &variant.fields {
                Fields::Unit => {
                    quote! {
                        #(#filtered_attrs)*
                        #discriminant_expr => {
                            #handler_path(vm)#await_token?;
                        }
                    }
                }
                Fields::Named(fields) => {
                    let (param_reading, args) = generate_param_reads_and_args(fields);

                    quote! {
                        #(#filtered_attrs)*
                        #discriminant_expr => {
                            #param_reading
                            #handler_path(vm, #(#args),*)#await_token?;
                        }
                    }
                }
                Fields::Unnamed(_) => {
                    // Already handled in the first loop, this shouldn't happen
                    continue;
                }
            };

            handler_match_arms.push(handler_arm);
        }
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

        impl #enum_name {
            pub async fn run_op<const N: usize, S: Sync, D: VmDebug>(
                opcode: u8,
                vm: &mut VM<N, S, D>,
            ) -> Result<()> {
                match opcode {
                    #(#handler_match_arms),*
                    _ => return Err(crate::vm::VMError::InvalidOpcode(opcode, vm.pc)),
                }
                Ok(())
            }
        }
    };

    expanded.into()
}
