extern crate proc_macro;

use proc_macro::TokenStream;

use quote::ToTokens;

use syn::parse_macro_input;

use string_template::StaticGroup;

#[proc_macro]
pub fn st_group(input: TokenStream) -> TokenStream {
    let group: StaticGroup = parse_macro_input!(input as StaticGroup);
    group.into_token_stream().into()
}
