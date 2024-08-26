extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

#[proc_macro_attribute]
pub fn params_handle(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let struct_name = Ident::new(&fn_name.to_string(), proc_macro2::Span::call_site());
    let fn_body = &input.block;

    let expanded = quote! {
        pub struct #struct_name;

        impl ParamsProcessor for #struct_name {
            fn process(&self, path: &str, params: &HashMap<String, String>) -> serde_json::Map<String, Value> {
                #fn_body
            }
        }
    };

    TokenStream::from(expanded)
}
