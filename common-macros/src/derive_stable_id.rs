use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Path};

pub(crate) fn derive_stable_id(input: TokenStream, trait_path: &str) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let trait_path_token = syn::parse_str::<Path>(trait_path).unwrap();
    
    let gene = quote! {
        impl #trait_path_token for #name {
            const ID: common::ecs::stable_id::StableId = common::ecs::stable_id::StableId::from_str(stringify!(#name));
        }
    };

    gene.into()
}
