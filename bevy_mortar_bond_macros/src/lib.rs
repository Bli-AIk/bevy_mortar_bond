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

            let arg_count = method.sig.inputs.len();

            // Check if return type is unit ()
            let returns_void = matches!(method.sig.output, ReturnType::Default);

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

                if returns_void {
                    quote! {
                        registry.register(#fn_name_str, |args| {
                            #(
                                let #arg_names = args.get(#arg_indices)
                                    .cloned()
                                    .unwrap_or(bevy_mortar_bond::MortarValue::Void);
                            )*
                            Self::#fn_name(#(#arg_names),*);
                            bevy_mortar_bond::MortarValue::Void
                        });
                    }
                } else {
                    quote! {
                        registry.register(#fn_name_str, |args| {
                            #(
                                let #arg_names = args.get(#arg_indices)
                                    .cloned()
                                    .unwrap_or(bevy_mortar_bond::MortarValue::Void);
                            )*
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
