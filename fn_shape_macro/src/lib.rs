extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use unsynn::*;

mod func_params;

mod ret_type;

mod generics;

mod func_body;

mod func_sig;
use func_sig::parse_function_signature;

/// `#[facet_fn] fn foo(...) -> R { ... }`
#[proc_macro_attribute]
pub fn facet_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Convert to proc_macro2 for parsing
    let item2: TokenStream2 = item.into();
    let parsed = parse_function_signature(item2);
    generate_function_shape(parsed)
}

fn generate_function_shape(parsed: func_sig::ParsedFunctionSignature) -> TokenStream {
    let fn_name = parsed.name;
    let generics = parsed.generics;
    let params = parsed.parameters;
    let return_type = parsed.return_type;
    let body = parsed.body;

    let hidden_mod = Ident::new(&format!("__fn_shape_{}", fn_name), Span::call_site());
    let shape_name = Ident::new(
        &format!("{}_SHAPE", fn_name.to_string().to_uppercase()),
        Span::call_site(),
    );
    let defs: Vec<_> = params
        .iter()
        .map(|p| {
            let name = &p.name;
            let ty = &p.param_type_tokens();
            quote! { #name: #ty }
        })
        .collect();
    let idents: Vec<_> = params
        .iter()
        .map(|p| {
            let name = &p.name;
            quote! { #name }
        })
        .collect();
    let types: Vec<_> = params
        .iter()
        .map(|p| {
            let ty = &p.param_type_tokens();
            quote! { #ty }
        })
        .collect();
    let names: Vec<_> = params
        .iter()
        .map(|p| p.name.to_string())
        .collect::<Vec<_>>();
    let arity = params.len();
    let fn_name_str = fn_name.to_string();

    // Always use a function approach - it works for both generic and non-generic
    let generics_type = if generics.is_some() {
        // For generic functions, try to extract just the type parameters for PhantomData
        // This is a simplified approach - we'll use the first type parameter
        quote! { T } // This assumes single generic - could be improved
    } else {
        quote! { () }
    };

    let shape_definition = quote! {
        pub fn shape #generics () -> FunctionShape<( #( #types ),* ), #return_type, #generics_type> {
            FunctionShape::new(
                #fn_name_str,
                #arity,
                &[ #( #names ),* ]
            )
        }
    };

    let out = quote! {
        // 1) Move the real implementation into a private module
        #[allow(non_snake_case)]
        mod #hidden_mod {
            use super::*;
            pub(super) fn inner #generics ( #( #defs ),* ) -> #return_type #body

            #[derive(Debug, Clone)]
            pub struct FunctionShape<Args, Ret, Generics = ()> {
                pub name: &'static str,
                pub param_count: usize,
                pub param_names: &'static [&'static str],
                _args: core::marker::PhantomData<Args>,
                _ret: core::marker::PhantomData<Ret>,
                _generics: core::marker::PhantomData<Generics>,
            }

            impl<Args, Ret, Generics> FunctionShape<Args, Ret, Generics> {
                pub const fn new(
                    name: &'static str,
                    param_count: usize,
                    param_names: &'static [&'static str],
                ) -> Self {
                    Self {
                        name,
                        param_count,
                        param_names,
                        _args: core::marker::PhantomData,
                        _ret: core::marker::PhantomData,
                        _generics: core::marker::PhantomData,
                    }
                }
            }

            #shape_definition
        }

        // 2) Public wrapper retains the exact original signature
        pub fn #fn_name #generics ( #( #defs ),* ) -> #return_type {
            #hidden_mod::inner( #( #idents ),* )
        }

        // 3) Re-export the shape function with function name
        pub use #hidden_mod::shape as #shape_name;
    };

    out.into()
}

/// `fn_shape!(function_name)` or `fn_shape!(function_name<T>)` - Access the shape metadata for a function
#[proc_macro]
pub fn fn_shape(input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();
    let mut tokens = input2.to_token_iter();

    // Parse the function name
    let fn_name = Ident::parse(&mut tokens).expect("expected function name");

    // Check if there are generic arguments
    let generic_args = if let Ok(TokenTree::Punct(p)) = TokenTree::parse(&mut tokens) {
        if p.as_char() == '<' {
            // Collect generic arguments
            let mut generic_tokens = TokenStream2::new();
            p.to_tokens(&mut generic_tokens);

            let mut depth = 1;
            while depth > 0 {
                if let Ok(token) = TokenTree::parse(&mut tokens) {
                    if let TokenTree::Punct(punct) = &token {
                        match punct.as_char() {
                            '<' => depth += 1,
                            '>' => depth -= 1,
                            _ => {}
                        }
                    }
                    token.to_tokens(&mut generic_tokens);
                } else {
                    break;
                }
            }
            Some(generic_tokens)
        } else {
            None
        }
    } else {
        None
    };

    // Generate the shape function name
    let shape_name = Ident::new(
        &format!("{}_SHAPE", fn_name.to_string().to_uppercase()),
        Span::call_site(),
    );

    let out = if let Some(generics) = generic_args {
        quote! { #shape_name::#generics() }
    } else {
        quote! { #shape_name() }
    };
    out.into()
}
