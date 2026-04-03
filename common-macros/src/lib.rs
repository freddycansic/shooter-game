mod derive_component;
mod derive_resource;

use proc_macro::TokenStream;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    derive_component::derive_component(input)
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    derive_resource::derive_resource(input)
}
