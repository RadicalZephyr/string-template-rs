extern crate proc_macro;

use std::{cmp, fmt};

use proc_macro::TokenStream;

use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, parse_macro_input, token, Ident, Token};

#[derive(Clone)]
struct StaticStGroup {
    group_name: Ident,
    brace_token: token::Brace,
    group: Punctuated<St, Token![;]>,
}

impl fmt::Debug for StaticStGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StGroup")
            .field("group_name", &self.group_name)
            .finish()
    }
}

impl cmp::PartialEq for StaticStGroup {
    fn eq(&self, other: &Self) -> bool {
        self.group_name == other.group_name
    }
}

impl Parse for StaticStGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![static]>()?;
        input.parse::<Token![ref]>()?;
        let group_name = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let group = content.parse_terminated(St::parse)?;
        Ok(StaticStGroup {
            group_name,
            group,
            brace_token,
        })
    }
}

#[derive(Clone)]
struct St {
    name: Ident,
    paren_token: token::Paren,
    formal_args: Punctuated<Ident, Token![,]>,
    template: Template,
}

impl fmt::Debug for St {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("St").field("name", &self.name).finish()
    }
}

impl cmp::PartialEq for St {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Parse for St {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(St {
            name: input.parse()?,
            paren_token: parenthesized!(content in input),
            formal_args: content.parse_terminated(Ident::parse)?,
            template: {
                input.parse::<Token![::]>()?;
                input.parse::<Token![=]>()?;
                input.parse::<Token![<<]>()?;
                let template = input.parse()?;
                input.parse::<Token![>>]>()?;
                template
            },
        })
    }
}

#[derive(Clone)]
struct Template {
    foo: Ident,
}

impl fmt::Debug for Template {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Template").finish()
    }
}

impl cmp::PartialEq for Template {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Parse for Template {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Template {
            foo: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn st_group(input: TokenStream) -> TokenStream {
    let _group = parse_macro_input!(input as StaticStGroup);
    "static FOO: u8 = 0;".parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    use proc_macro2::Span;

    fn parse_group(template: &'static str) -> StaticStGroup {
        syn::parse_str(template).expect("unexpected parsing failure")
    }

    #[test]
    fn parse_no_arg_template() {
        assert_eq!(
            StaticStGroup {
                group_name: Ident::new("group_a", Span::call_site()),
                brace_token: token::Brace {
                    span: Span::call_site()
                },
                group: Punctuated::new(),
            },
            parse_group("static ref group_a {\n a() ::= <<foo>>\n }")
        );
    }
}
