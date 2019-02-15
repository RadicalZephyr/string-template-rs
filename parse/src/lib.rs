#![recursion_limit = "128"]

extern crate proc_macro;

use std::{cmp, fmt};

use proc_macro::TokenStream;

use quote::{quote, quote_spanned};

use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, parse_macro_input, token, Ident, Token, Visibility};

#[derive(Clone)]
struct StaticStGroup {
    visibility: Visibility,
    group_name: Ident,
    brace_token: token::Brace,
    templates: Punctuated<StaticSt, Token![;]>,
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
        let visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        input.parse::<Token![ref]>()?;
        let group_name = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let templates = content.parse_terminated(StaticSt::parse)?;
        Ok(StaticStGroup {
            visibility,
            group_name,
            templates,
            brace_token,
        })
    }
}

#[derive(Clone)]
struct StaticSt {
    name: Ident,
    paren_token: token::Paren,
    formal_args: Punctuated<Ident, Token![,]>,
    template_body: TemplateBody,
}

impl fmt::Debug for StaticSt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StaticSt")
            .field("name", &self.name)
            .finish()
    }
}

impl cmp::PartialEq for StaticSt {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Parse for StaticSt {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(StaticSt {
            name: input.parse()?,
            paren_token: parenthesized!(content in input),
            formal_args: content.parse_terminated(Ident::parse)?,
            template_body: {
                input.parse::<Token![::]>()?;
                input.parse::<Token![=]>()?;
                input.parse::<Token![<<]>()?;
                let template_body = input.parse()?;
                input.parse::<Token![>>]>()?;
                template_body
            },
        })
    }
}

#[derive(Clone)]
struct TemplateBody {
    foo: Ident,
}

impl fmt::Debug for TemplateBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TemplateBody").finish()
    }
}

impl cmp::PartialEq for TemplateBody {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Parse for TemplateBody {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TemplateBody {
            foo: input.parse()?,
        })
    }
}

fn init_for_group(
    brace_token: &token::Brace,
    group: Punctuated<StaticSt, Token![;]>,
) -> proc_macro2::TokenStream {
    quote_spanned! { brace_token.span => {
            let templates = ::std::collections::HashMap::new();
            ::string_template::StGroup::new(templates)
        }
    }
}

#[proc_macro]
pub fn st_group(input: TokenStream) -> TokenStream {
    let StaticStGroup {
        visibility,
        group_name,
        brace_token,
        templates,
    } = parse_macro_input!(input as StaticStGroup);

    let ty = quote! { ::string_template::StGroup };

    let init = init_for_group(&brace_token, templates);

    let init_ptr = quote_spanned! { brace_token.span =>
                               Box::into_raw(Box::new(#init))
    };

    let expanded = quote! {
        #visibility struct #group_name;

        impl ::std::ops::Deref for #group_name {
            type Target = #ty;

            fn deref(&self) -> &#ty {
                static ONCE: ::std::sync::Once = ::std::sync::ONCE_INIT;
                static mut VALUE: *mut #ty = 0 as *mut #ty;

                unsafe {
                    ONCE.call_once(|| VALUE = #init_ptr);
                    &*VALUE
                }
            }
        }
    };
    expanded.into()
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
                visibility: Visibility::Public(syn::VisPublic {
                    pub_token: token::Pub {
                        span: Span::call_site()
                    }
                }),
                group_name: Ident::new("group_a", Span::call_site()),
                brace_token: token::Brace {
                    span: Span::call_site()
                },
                templates: Punctuated::new(),
            },
            parse_group("static ref group_a {\n a() ::= <<foo>>\n }")
        );
    }
}
