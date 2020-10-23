#[macro_use]
extern crate syn;

use proc_macro::{TokenStream};
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Result, parse_macro_input, ItemEnum, ItemImpl, ImplItem, ImplItemMethod, FnArg, Pat};
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

fn union_method_arm<'a>(union_type_name: &'a Ident, subtype_name: &'a Ident,
                        method_name: &'a Ident, argument_names: impl Iterator<Item=&'a Ident>) -> TokenStream2 {
    quote! {
        #union_type_name::#subtype_name(x) => x.#method_name(#(#argument_names),*),
    }
}

fn argument_names(parameters: &Punctuated<FnArg, Token![,]>) -> Vec<&Ident> {
    parameters.iter()
        .filter_map(|it| if let FnArg::Typed(x) = it { Some(&x.pat) } else { None })
        .filter_map(|it| if let Pat::Ident(x) = it.borrow() { Some(&x.ident) } else { None }) // todo: mut and ref here?
        .collect()
}

fn union_method<'a>(impl_item_method: &'a ImplItemMethod, union_type_name: &'a Ident, subtypes: impl Iterator<Item=&'a Ident>) -> TokenStream2 {
    let attrs = &impl_item_method.attrs.iter()
        .map(|it| quote! {#it})
        .fold(TokenStream2::new(), |mut acc, x| {
            acc.extend(x.into_iter());
            acc
        });
    let vis = &impl_item_method.vis;
    let sig = &impl_item_method.sig;
    let method_name = &sig.ident;
    let argument_names = argument_names(&sig.inputs).into_iter();
    let arms = subtypes
        .map(|it|
            union_method_arm(union_type_name, &it, method_name, argument_names.clone()))
        .fold(TokenStream2::new(), |mut acc, x| {
            acc.extend(x.into_iter());
            acc
        });
    quote! {
        #attrs
        #vis#sig {
            match self {
                #arms
            }
        }
    }
}

#[proc_macro]
pub fn union_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as UnionType);
    let attrs = &input.enum_type.attrs.iter()
        .map(|it| quote! {#it})
        .fold(TokenStream2::new(), |mut acc, x| {
            acc.extend(x.into_iter());
            acc
        });
    let typename = &input.enum_type.ident;
    let visibility = &input.enum_type.vis;
    let subtypes = &input.enum_type.variants.iter()
        .map(|it| &it.ident);
    let subtype_token_stream = subtypes.clone()
        .map(|it| quote! {
            #it(#it),
        })
        .fold(TokenStream2::new(), |mut acc, x| {
            acc.extend(x.into_iter());
            acc
        });
    let impl_functions = input.impl_type.items.iter()
        .filter_map(|it| if let ImplItem::Method(method) = it { Some(method) } else { None });
    let impl_functions_token_stream = impl_functions
        .map(|it| union_method(it, typename, subtypes.clone()))
        .fold(TokenStream2::new(), |mut acc, x| {
            acc.extend(x.into_iter());
            acc
        });
    let tokens = quote! {
        #attrs
        #visibility enum #typename {
            #subtype_token_stream
        }

        impl #typename {
            #impl_functions_token_stream
        }
    };
    tokens.into()
}
