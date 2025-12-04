use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemImpl, ReturnType, parse_macro_input};

#[proc_macro_derive(MortarFunctions)]
pub fn derive_mortar_functions(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let impl_block = quote! {
        impl #name {
            pub fn register(registry: &mut bevy_mortar_bond::MortarFunctionRegistry) {
                Self::bind_functions(registry);
            }
        }
    };

    impl_block.into()
}

#[proc_macro_attribute]
pub fn mortar_functions(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let self_ty = &input.self_ty;

    let mut function_registrations = Vec::new();

    for item in &input.items {
        if let syn::ImplItem::Fn(method) = item {
            let fn_name = &method.sig.ident;
            let fn_name_str = fn_name.to_string();

            // Check if return type is unit ().
            //
            // 检查返回类型是否为 unit ()。
            let returns_void = matches!(method.sig.output, ReturnType::Default);

            // Extract argument types (skip self parameter).
            //
            // 提取参数类型（跳过 self 参数）。
            let args: Vec<_> = method
                .sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg {
                        Some(pat_type.ty.as_ref())
                    } else {
                        None
                    }
                })
                .collect();

            let arg_count = args.len();

            let registration = if arg_count == 0 {
                if returns_void {
                    quote! {
                        registry.register(#fn_name_str, |_args| {
                            Self::#fn_name();
                            bevy_mortar_bond::MortarValue::Void
                        });
                    }
                } else {
                    quote! {
                        registry.register(#fn_name_str, |_args| {
                            Self::#fn_name().into()
                        });
                    }
                }
            } else {
                let arg_indices: Vec<_> = (0..arg_count).collect();
                let arg_names: Vec<syn::Ident> = (0..arg_count)
                    .map(|i| syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site()))
                    .collect();

                // Generate type conversions based on the argument types.
                //
                // 根据参数类型生成类型转换代码。
                let arg_conversions: Vec<_> = args
                    .iter()
                    .zip(arg_indices.iter())
                    .zip(arg_names.iter())
                    .map(|((ty, idx), name)| {
                        let type_str = quote!(#ty).to_string();
                        let type_str = type_str.replace(" ", "");

                        if type_str.contains("MortarString") {
                            quote! {
                                let #name = args.get(#idx)
                                    .and_then(|v| v.as_string())
                                    .cloned()
                                    .unwrap_or_else(|| bevy_mortar_bond::MortarString::from(""));
                            }
                        } else if type_str.contains("MortarNumber") {
                            quote! {
                                let #name = args.get(#idx)
                                    .and_then(|v| v.as_number())
                                    .unwrap_or_else(|| bevy_mortar_bond::MortarNumber::from(0.0));
                            }
                        } else if type_str.contains("MortarBoolean") {
                            quote! {
                                let #name = args.get(#idx)
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or_else(|| bevy_mortar_bond::MortarBoolean::from(false));
                            }
                        } else {
                            // Fallback to MortarValue for unknown types.
                            //
                            // 对未知类型回退到 MortarValue。
                            quote! {
                                let #name = args.get(#idx)
                                    .cloned()
                                    .unwrap_or(bevy_mortar_bond::MortarValue::Void);
                            }
                        }
                    })
                    .collect();

                if returns_void {
                    quote! {
                        registry.register(#fn_name_str, |args| {
                            #(#arg_conversions)*
                            Self::#fn_name(#(#arg_names),*);
                            bevy_mortar_bond::MortarValue::Void
                        });
                    }
                } else {
                    quote! {
                        registry.register(#fn_name_str, |args| {
                            #(#arg_conversions)*
                            Self::#fn_name(#(#arg_names),*).into()
                        });
                    }
                }
            };

            function_registrations.push(registration);
        }
    }

    let expanded = quote! {
        #input

        impl #self_ty {
            pub fn bind_functions(registry: &mut bevy_mortar_bond::MortarFunctionRegistry) {
                #(#function_registrations)*
            }
        }
    };

    expanded.into()
}
