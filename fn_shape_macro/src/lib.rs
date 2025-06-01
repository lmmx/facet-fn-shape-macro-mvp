extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use unsynn::*;

/// `#[facet_fn] fn foo(...) -> R { ... }`
#[proc_macro_attribute]
pub fn facet_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Convert to proc_macro2 for parsing
    let item2: TokenStream2 = item.into();
    let mut it = item2.to_token_iter();

    // Parse `fn` and the function name
    let _ = Ident::parse(&mut it).expect("expected `fn` keyword");
    let fn_name = Ident::parse(&mut it).expect("expected function name");

    // Simple approach: collect all remaining tokens and parse manually
    let all_remaining: Vec<TokenTree> = it.collect();
    let mut pos = 0;

    // Check for generics
    let generics = if pos < all_remaining.len() {
        if let TokenTree::Punct(p) = &all_remaining[pos] {
            if p.as_char() == '<' {
                // Found generics, collect them
                let mut generic_tokens = TokenStream2::new();
                let mut depth = 0;

                while pos < all_remaining.len() {
                    let token = &all_remaining[pos];
                    if let TokenTree::Punct(punct) = token {
                        match punct.as_char() {
                            '<' => depth += 1,
                            '>' => {
                                depth -= 1;
                                token.to_tokens(&mut generic_tokens);
                                pos += 1;
                                if depth == 0 {
                                    break;
                                }
                                continue;
                            }
                            _ => {}
                        }
                    }
                    token.to_tokens(&mut generic_tokens);
                    pos += 1;
                }
                Some(generic_tokens)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Find and parse the parameter list
    let paren_pos = {
        let mut found_pos = None;
        for (i, token) in all_remaining[pos..].iter().enumerate() {
            if let TokenTree::Group(group) = token {
                if group.delimiter() == proc_macro2::Delimiter::Parenthesis {
                    found_pos = Some(pos + i);
                    break;
                }
            }
        }
        found_pos.expect("expected parameter list")
    };

    let paren_group = if let TokenTree::Group(group) = &all_remaining[paren_pos] {
        group.clone()
    } else {
        panic!("expected parenthesis group");
    };
    pos = paren_pos + 1;

    // Parse parameters
    let params = {
        let params_ts: TokenStream2 = paren_group.stream();
        extract_parameters_from_stream(params_ts)
    };

    // Check for return type
    let ret = if pos < all_remaining.len() - 1 {
        // -1 because last should be brace
        if let (TokenTree::Punct(p1), TokenTree::Punct(p2)) =
            (&all_remaining[pos], &all_remaining[pos + 1])
        {
            if p1.as_char() == '-' && p2.as_char() == '>' {
                // Found return type, collect until brace
                pos += 2; // skip '->'
                let mut ret_tokens = TokenStream2::new();
                while pos < all_remaining.len() {
                    let token = &all_remaining[pos];
                    if let TokenTree::Group(group) = token {
                        if group.delimiter() == proc_macro2::Delimiter::Brace {
                            break;
                        }
                    }
                    token.to_tokens(&mut ret_tokens);
                    pos += 1;
                }
                ret_tokens
            } else {
                quote! { () }
            }
        } else {
            quote! { () }
        }
    } else {
        quote! { () }
    };

    // Parse the function body - should be the last brace group
    let body_ts = {
        let mut body_tokens = TokenStream2::new();
        if let Some(TokenTree::Group(group)) = all_remaining.last() {
            if group.delimiter() == proc_macro2::Delimiter::Brace {
                TokenTree::Group(group.clone()).to_tokens(&mut body_tokens);
            }
        }
        body_tokens
    };

    generate_function_shape(fn_name, params, generics, ret, body_ts)
}

fn extract_parameters_from_stream(params_ts: TokenStream2) -> Vec<Parameter> {
    let mut pit = params_ts.to_token_iter();
    let mut parameters = Vec::new();

    loop {
        // Try parsing an identifier
        let name = match Ident::parse(&mut pit) {
            Ok(id) => id,
            Err(_) => break,
        };
        // Expect and consume the colon
        if Operator::<':'>::parse(&mut pit).is_err() {
            break;
        }
        // Collect type tokens until comma or end
        let mut ty = TokenStream2::new();
        loop {
            match TokenTree::parse(&mut pit) {
                Ok(tt) => {
                    if let TokenTree::Punct(p) = &tt {
                        if p.as_char() == ',' {
                            break;
                        }
                    }
                    tt.to_tokens(&mut ty);
                }
                Err(_) => break,
            }
        }
        parameters.push(Parameter {
            name,
            param_type: ty,
        });
        // Consume optional comma
        let _ = Operator::<','>::parse(&mut pit);
    }

    parameters
}

fn generate_function_shape(
    fn_name: Ident,
    params: Vec<Parameter>,
    generics: Option<TokenStream2>,
    return_type: TokenStream2,
    body: TokenStream2,
) -> TokenStream {
    let hidden_mod = Ident::new(&format!("__fn_shape_{}", fn_name), Span::call_site());
    let shape_name = Ident::new(
        &format!("{}_SHAPE", fn_name.to_string().to_uppercase()),
        Span::call_site(),
    );
    let defs: Vec<_> = params
        .iter()
        .map(|p| {
            let name = &p.name;
            let ty = &p.param_type;
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
            let ty = &p.param_type;
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

struct Parameter {
    name: Ident,
    param_type: TokenStream2,
}
