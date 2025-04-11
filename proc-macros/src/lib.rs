use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Selectable)]
pub fn derive_selectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ty = &input.ident;

    let expanded = quote! {
        impl Selectable for #ty {
            fn selected(&self) -> bool {
                self.selected
            }

            fn select(&mut self) {
                self.selected = true;
            }

            fn deselect(&mut self) {
                self.selected = false;
            }

            fn toggle_selected(&mut self) {
                self.selected = !self.selected;
            }
        }
    };

    TokenStream::from(expanded)
}
