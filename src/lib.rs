#[macro_use]
extern crate syn;

use proc_macro::{TokenStream};
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Result, parse_macro_input, ItemEnum, ItemImpl, ImplItem, Variant, ImplItemMethod, FnArg, Pat};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use std::borrow::Borrow;

struct UnionType {
    enum_type: ItemEnum,
    impl_type: ItemImpl,
}

impl Parse for UnionType {
    fn parse(input: ParseStream) -> Result<Self> {
        let enum_type: ItemEnum = input.parse()?;
        let impl_type: ItemImpl = input.parse()?;
        Ok(UnionType { enum_type, impl_type })
    }
}

fn union_method(impl_item: &ImplItemMethod, ident: &Ident, variants: &Punctuated<Variant, Token![,]>) -> TokenStream2 {
    let vis = &impl_item.vis;
    let name = &impl_item.sig.ident;
    let inputs = &impl_item.sig.inputs;
    let output = &impl_item.sig.output;
    let mut tokens = quote! {
        #vis fn #name(#inputs) #output
    };
    let input_names: Vec<_> = inputs.iter()
        .filter_map(|it| if let FnArg::Typed(t) = it { Some(t) } else { None })
        .filter_map(|it| if let Pat::Ident(i) = it.pat.borrow() { Some(i) } else { None })
        .map(|it| &it.ident)
        .collect();

    let mut match_stmts = TokenStream2::new();
    for variant in variants.iter() {
        let variant_name = &variant.ident;
        let arm = quote! {
            #ident::#variant_name(x) => x.#name(#(#input_names),*),
        };
        match_stmts.extend(arm.into_iter());
    }
    let match_statement = quote! {{
        match &self {
            #match_stmts
        }
    }};
    tokens.extend(match_statement.into_iter());
    tokens
}

#[proc_macro]
pub fn union_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as UnionType);
    let ident = &input.enum_type.ident;
    let vis = &input.enum_type.vis;
    let mut item_token_stream = TokenStream2::new();
    let mut item_ts: Vec<TokenStream2> = Vec::new();
    for item in input.enum_type.variants.iter() {
        let t: TokenStream2 = quote! {
                #item(#item),
        }.into();
        item_ts.push(t);
    }
    item_token_stream.extend(item_ts.into_iter());

    let mut impl_token_stream = TokenStream2::new();
    let impl_functions = input.impl_type.items
        .iter().filter_map(|it| if let ImplItem::Method(method) = it { Some(method) } else { None })
        .map(|it| {
            union_method(it, ident, &input.enum_type.variants)
        });
    for impl_function in impl_functions {
        impl_token_stream.extend(impl_function.into_iter());
    }
    let tokens = quote! {
            #vis enum #ident {#item_token_stream}
            impl #ident {
                #impl_token_stream
            }
        };
    tokens.into()
}
