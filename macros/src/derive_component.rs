use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let gene = quote! {
        impl Component for #name {
            const ID: StableComponentId = StableComponentId(const_sha1::sha1(stringify!(#name)));
        }
    };

    gene.into()
}
