mod derive_stable_id;

use crate::derive_stable_id::derive_stable_id;
use proc_macro::TokenStream;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    derive_stable_id(input, "common::ecs::component::Component")
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    derive_stable_id(input, "common::ecs::resource::Resource")
}

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    derive_stable_id(input, "common::ecs::event::Event")
}
