//! This crate contains the procedural macros that reduce boilerplate for
//! `bevy_mortar_bond` integrations. Its job is to turn annotated Rust impls into
//! function-registration glue so host code can expose Mortar-callable APIs without
//! hand-writing repetitive registry plumbing.
//!
//! 这个 crate 存放 `bevy_mortar_bond` 用来减少样板代码的过程宏。它的职责是把带注解的
//! Rust 实现转换成函数注册胶水代码，让宿主侧可以把 API 暴露给 Mortar 调用，而不必手写
//! 大量重复的注册表接线逻辑。

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

/// Generate type conversion code for a single function argument.
fn generate_arg_conversion(
    ty: &syn::Type,
    idx: usize,
    name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let type_str = quote!(#ty).to_string().replace(" ", "");

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
        quote! {
            let #name = args.get(#idx)
                .cloned()
                .unwrap_or(bevy_mortar_bond::MortarValue::Void);
        }
    }
}

/// Generate registration code for a single method.
fn generate_registration(method: &syn::ImplItemFn) -> proc_macro2::TokenStream {
    let fn_name = &method.sig.ident;
    let fn_name_str = fn_name.to_string();
    let returns_void = matches!(method.sig.output, ReturnType::Default);

    // Extract argument types (skip self parameter).
    let args: Vec<_> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(pat_type.ty.as_ref()),
            _ => None,
        })
        .collect();

    if args.is_empty() {
        return generate_no_arg_registration(&fn_name_str, fn_name, returns_void);
    }

    let arg_names: Vec<syn::Ident> = (0..args.len())
        .map(|i| syn::Ident::new(&format!("arg{i}"), proc_macro2::Span::call_site()))
        .collect();

    let arg_conversions: Vec<_> = args
        .iter()
        .enumerate()
        .zip(arg_names.iter())
        .map(|((idx, ty), name)| generate_arg_conversion(ty, idx, name))
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
}

fn generate_no_arg_registration(
    fn_name_str: &str,
    fn_name: &syn::Ident,
    returns_void: bool,
) -> proc_macro2::TokenStream {
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
}

#[proc_macro_attribute]
pub fn mortar_functions(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let self_ty = &input.self_ty;

    let function_registrations: Vec<_> = input
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(method) => Some(generate_registration(method)),
            _ => None,
        })
        .collect();

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
