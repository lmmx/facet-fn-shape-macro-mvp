#[cfg(test)]
use proc_macro2::TokenStream;
use unsynn::*;

unsynn! {
    /// Parses either a `TokenTree` or `<...>` grouping (which is not a [`Group`] as far as proc-macros
    /// are concerned).
    #[derive(Clone)]
    pub struct AngleTokenTree(
        pub Either<Cons<Lt, Vec<Cons<Except<Gt>, AngleTokenTree>>, Gt>, TokenTree>,
    );

    /// Generic parameters with angle brackets
    pub struct GenericParams {
        /// Opening angle bracket
        pub _lt: Lt,
        /// Generic parameters content
        pub params: Vec<Cons<Except<Gt>, AngleTokenTree>>,
        /// Closing angle bracket
        pub _gt: Gt,
    }
}

/// Parse generics from token slice starting at given position
/// Returns (Option<TokenStream>, tokens_consumed)
#[cfg(test)]
pub fn parse_generics(tokens: &[TokenTree]) -> (Option<TokenStream>, usize) {
    if tokens.is_empty() {
        return (None, 0);
    }

    // Check if first token is '<'
    if let TokenTree::Punct(p) = &tokens[0] {
        if p.as_char() == '<' {
            // Convert tokens to TokenStream for parsing
            let mut token_stream = TokenStream::new();
            for token in tokens {
                token_stream.extend(core::iter::once(token.clone()));
            }

            let mut it = token_stream.to_token_iter();

            match it.parse::<GenericParams>() {
                Ok(generics) => {
                    // Calculate how many tokens were consumed
                    let consumed = 1 + generics.params.len() + 1; // < + params + >
                    let generics_tokens = generics.to_token_stream();
                    (Some(generics_tokens), consumed)
                }
                Err(_) => (None, 0),
            }
        } else {
            (None, 0)
        }
    } else {
        (None, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_no_generics() {
        let input: Vec<TokenTree> = quote! { fn_name() }.into_iter().collect();
        let (generics, consumed) = parse_generics(&input);
        assert!(generics.is_none());
        assert_eq!(consumed, 0);
    }

    #[test]
    fn test_simple_generics() {
        let input: Vec<TokenTree> = quote! { <T> }.into_iter().collect();
        let (generics, consumed) = parse_generics(&input);
        assert!(generics.is_some());
        assert_eq!(consumed, 3); // <, T, >
        assert_eq!(generics.unwrap().to_string().trim(), "< T >");
    }

    #[test]
    fn test_multiple_generics() {
        let input: Vec<TokenTree> = quote! { <T, U: Clone> }.into_iter().collect();
        let (generics, consumed) = parse_generics(&input);
        assert!(generics.is_some());
        assert_eq!(consumed, 7); // <, T, ,, U, :, Clone, >
        assert_eq!(generics.unwrap().to_string().trim(), "< T , U : Clone >");
    }

    #[test]
    fn test_complex_generics() {
        let input: Vec<TokenTree> = quote! { <T: Add<Output = T>> }.into_iter().collect();
        let (generics, _consumed) = parse_generics(&input);
        assert!(generics.is_some());
        assert_eq!(
            generics.unwrap().to_string().trim(),
            "< T : Add < Output = T > >"
        );
    }

    #[test]
    fn test_generics_with_following_tokens() {
        let input: Vec<TokenTree> = quote! { <T> (x: T) }.into_iter().collect();
        let (generics, consumed) = parse_generics(&input);
        assert!(generics.is_some());
        assert_eq!(consumed, 3); // Only consumes <T>
        assert_eq!(generics.unwrap().to_string().trim(), "< T >");
    }
}
