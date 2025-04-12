use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, parse_macro_input};

#[proc_macro]
pub fn default_name_generator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);

    let name = input.value();

    let function_name = Ident::new(&name.to_lowercase(), input.span());
    let counter_name = Ident::new(&format!("{}_COUNTER", name.to_uppercase()), input.span());

    let expanded = quote! {

        static #counter_name: AtomicU64 = AtomicU64::new(0);

        pub fn #function_name() -> String {
            let count = #counter_name.fetch_add(1, Ordering::SeqCst);
            format!(concat!(#name, " {}"), count)
        }
    };

    TokenStream::from(expanded)
}
