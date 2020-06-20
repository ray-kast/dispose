#![warn(missing_docs)]

use proc_macro::TokenStream;

#[proc_macro_derive(Dispose)]
pub fn derive_dispose(_item: TokenStream) -> TokenStream {
    "derive ::dispose::Dispose for () { }".parse().unwrap()
}
