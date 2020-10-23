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

fn impl_from(union_type_name: &Ident, subtype_name: &Ident) -> TokenStream2 {
    quote! {
        impl std::convert::From<#subtype_name> for #union_type_name {
            fn from(x: #subtype_name) -> Self {
                #union_type_name::#subtype_name(x)
            }
        }
    }
}

fn impl_reverse_try_from(union_type_name: &Ident, subtype_name: &Ident) -> TokenStream2 {
    quote! {
        impl std::convert::TryFrom<#union_type_name> for #subtype_name {
            type Error = ();
            fn try_from(x: #union_type_name) -> Result<Self, Self::Error> {
                if let #union_type_name::#subtype_name(x) = x {
                    Ok(x)
                } else {
                    Err(())
                }
            }
        }
    }
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

/// To make a enum a "union type"
/// Assume there exists an A type and a B type,
/// both of them has implemented `fn` `f` and `g`
/// ```rust, no_run
/// union_type! {
///     #[derive(Debug, Clone)]
///     enum C {
///         A,
///         B
///     }
///     impl C {
///         fn f(&self) -> i32;
///         fn g<T: Display>(&self, t: T) -> String;
///     }
/// }
/// ```
/// Then type C becomes an union type, you can cast from and into C with A and B:
/// ```rust, no_run
/// let a = A::new();
/// let mut c: C = a.into();
/// let b = c.try_into();               // cause an Err
/// let a: A = c.try_into().unwrap();   // successful
/// ```
/// And will call its child type's function when:
/// ```rust, no_run
/// let a = A::new();
/// let mut c: C = a.into();
/// c.f(); // equivalent with call a.f()
/// let b = B::new();
/// c = b.into();
/// c.f(); // equivalent with call b.f()
/// ```
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
    let impl_froms = subtypes.clone()
        .map(|it| impl_from(typename, it))
        .fold(TokenStream2::new(), |mut acc, x| {
            acc.extend(x.into_iter());
            acc
        });
    let impl_reverse_try_froms = subtypes.clone()
        .map(|it| impl_reverse_try_from(typename, it))
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

        #impl_froms

        #impl_reverse_try_froms
    };
    tokens.into()
}
