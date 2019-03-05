extern crate proc_macro;

use proc_macro::TokenStream;

use quote::ToTokens;

use syn::parse_macro_input;

use string_template::StaticGroup;

mod test;
use crate::test::Test;

#[proc_macro]
pub fn st_group(input: TokenStream) -> TokenStream {
    let group: StaticGroup = parse_macro_input!(input as StaticGroup);
    group.into_token_stream().into()
}

#[proc_macro]
pub fn st_test(input: TokenStream) -> TokenStream {
    let test: Test = parse_macro_input!(input as Test);
    test.into_token_stream().into()
}
