use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let gene = quote! {
        impl Component for #name {
            const ID: StableComponentId = StableComponentId(hash_type_name(stringify!(#name)));
        }
    };

    gene.into()
}
