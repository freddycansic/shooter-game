mod derive_component;

use proc_macro::TokenStream;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    derive_component::derive_component(input)
}
