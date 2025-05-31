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

    // Parse the parameter list `( ... )`
    let paren = ParenthesisGroup::parse(&mut it).expect("expected `( ... )`");

    // Extract (ident, type) pairs from inside the parentheses
    let mut params = Vec::<Parameter>::new();
    {
        // Access the inner Group and get its stream
        let inner_group = &paren.0; // ParenthesisGroup(Group)
        let params_ts: TokenStream2 = inner_group.stream().into();
        let mut pit = params_ts.to_token_iter();

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
            params.push(Parameter {
                name,
                param_type: ty,
            });
            // Consume optional comma
            let _ = Operator::<','>::parse(&mut pit);
        }
    }

    // Parse `-> RetType` if present, and then parse the body
    let mut ret = quote! { () };
    let body_ts = if Operator::<'-', '>'>::parse(&mut it).is_ok() {
        // Collect return type tokens until we hit a brace group
        let mut return_tokens = TokenStream2::new();
        let mut body_group = None;
        
        loop {
            match TokenTree::parse(&mut it) {
                Ok(token) => {
                    if let TokenTree::Group(ref g) = token {
                        if g.delimiter() == proc_macro2::Delimiter::Brace {
                            // This is the function body
                            body_group = Some(g.clone());
                            break;
                        }
                    }
                    token.to_tokens(&mut return_tokens);
                }
                Err(_) => break,
            }
        }
        
        if !return_tokens.is_empty() {
            ret = return_tokens;
        }
        
        // Convert the Group to TokenStream2
        body_group.map(|g| {
            let mut ts = TokenStream2::new();
            TokenTree::Group(g).to_tokens(&mut ts);
            ts
        }).unwrap_or_else(|| {
            panic!("expected function body after return type");
        })
    } else {
        // No return type, parse body directly
        let body_grp = BraceGroup::parse(&mut it).expect("expected `{ ... }` body");
        let mut body_ts = TokenStream2::new();
        body_grp.to_tokens(&mut body_ts);
        body_ts
    };

    // Generate the wrapper + metadata
    let hidden_mod = Ident::new(&format!("__fn_shape_{}", fn_name), Span::call_site());
    let shape_name = Ident::new(&format!("{}_SHAPE", fn_name.to_string().to_uppercase()), Span::call_site());
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

    let out = quote! {
        // 1) Move the real implementation into a private module
        #[allow(non_snake_case)]
        mod #hidden_mod {
            use super::*;
            pub(super) fn inner( #( #defs ),* ) -> #ret #body_ts

            #[derive(Debug, Clone)]
            pub struct FunctionShape<Args, Ret> {
                pub name: &'static str,
                pub param_count: usize,
                pub param_names: &'static [&'static str],
                _args: core::marker::PhantomData<Args>,
                _ret:  core::marker::PhantomData<Ret>,
            }

            impl<Args, Ret> FunctionShape<Args, Ret> {
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
                    }
                }
            }

            pub const SHAPE: FunctionShape<( #( #types ),* ), #ret> = FunctionShape::new(
                #fn_name_str,
                #arity,
                &[ #( #names ),* ]
            );
        }

        // 2) Public wrapper retains the exact original signature
        pub fn #fn_name( #( #defs ),* ) -> #ret {
            #hidden_mod::inner( #( #idents ),* )
        }

        // 3) Re-export the constant with function name
        pub use #hidden_mod::SHAPE as #shape_name;
    };

    out.into()
}

/// `fn_shape!(function_name)` - Access the shape metadata for a function
#[proc_macro]
pub fn fn_shape(input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();
    let mut tokens = input2.to_token_iter();
    
    // Parse the function name
    let fn_name = Ident::parse(&mut tokens).expect("expected function name");
    
    // Generate the shape constant name
    let shape_name = Ident::new(&format!("{}_SHAPE", fn_name.to_string().to_uppercase()), Span::call_site());
    
    let out = quote! { #shape_name };
    out.into()
}

struct Parameter {
    name: Ident,
    param_type: TokenStream2,
}