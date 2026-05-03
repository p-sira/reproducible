use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, LitInt, parse_macro_input};

/// Shorthand for creating a `Row` from a function, using its name as the row label.
#[proc_macro]
pub fn row(input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as Expr);
    let f_str = quote!(#f).to_string();

    let expanded = quote! {
        reproducible::rows::Row::new(#f_str, #f)
    };

    TokenStream::from(expanded)
}

/// Wrap a function that takes positional arguments into one that takes a slice.
#[proc_macro]
pub fn wrap_args(input: TokenStream) -> TokenStream {
    // Expected: wrap_args!(func, 2)
    let parsed = syn::parse::Parser::parse2(
        |input: syn::parse::ParseStream| {
            let f: Expr = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            let n: LitInt = input.parse()?;
            Ok((f, n))
        },
        proc_macro2::TokenStream::from(input),
    );

    let (f, n) = match parsed {
        Ok(res) => res,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let n_val: usize = match n.base10_parse() {
        Ok(val) => val,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let indices = (0..n_val).map(|i| {
        let idx = syn::Index::from(i);
        quote! { i[#idx] }
    });

    let expanded = quote! {
        |i: &[_]| vec![#f(#(#indices),*)]
    };

    TokenStream::from(expanded)
}

/// Wrap a function that takes a fixed array into one that takes a slice.
#[proc_macro]
pub fn as_array(input: TokenStream) -> TokenStream {
    // Expected: as_array!(func, 3)
    let parsed = syn::parse::Parser::parse2(
        |input: syn::parse::ParseStream| {
            let f: Expr = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            let n: Expr = input.parse()?;
            Ok((f, n))
        },
        proc_macro2::TokenStream::from(input),
    );

    let (f, n) = match parsed {
        Ok(res) => res,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let expanded = quote! {
        |i: &[f64]| vec![#f(i[..#n].try_into().unwrap())]
    };

    TokenStream::from(expanded)
}
