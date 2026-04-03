mod derive_component;
mod derive_resource;

use proc_macro::TokenStream;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    derive_component::derive_component(input)
}
