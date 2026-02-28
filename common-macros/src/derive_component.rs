use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub(crate) fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let gene = quote! {
        impl common::ecs::component::Component for #name {
            const ID: common::ecs::component::StableComponentId = common::ecs::component::StableComponentId::from_str(stringify!(#name));
        }
    };

    gene.into()
}
