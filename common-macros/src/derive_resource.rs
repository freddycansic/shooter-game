use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn derive_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let gene = quote! {
        impl common::ecs::resource::Resource for #name {
            const ID: common::ecs::stable_id::StableId = common::ecs::stable_id::StableId::from_str(stringify!(#name));
        }
    };

    gene.into()
}
