#![recursion_limit = "128"]

extern crate proc_macro;

use std::{cmp, fmt};

use proc_macro::TokenStream;
use proc_macro2::Span;

use quote::ToTokens;
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

impl ToTokens for StaticStGroup {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = quote! { ::string_template::StGroup };
        let brace_span: Span = self.brace_token.span;
        let templates = &self.templates;
        let init = quote_spanned! { brace_span =>
                                    {
                                        let mut templates = ::std::collections::HashMap::new();
                                        #( #templates )*
                                        ::string_template::StGroup::new(templates)
                                    }
        };
        let init_ptr = quote_spanned! { self.brace_token.span =>
                                   Box::into_raw(Box::new(#init))
        };
        let visibility = &self.visibility;
        let group_name = &self.group_name;
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
        tokens.extend(expanded);
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
        let name = input.parse()?;
        let content;
        let paren_token = parenthesized!(content in input);
        let formal_args = content.parse_terminated(Ident::parse)?;

        input.parse::<Token![::]>()?;
        input.parse::<Token![=]>()?;

        input.parse::<Token![<<]>()?;
        let template_body = input.parse()?;
        input.parse::<Token![>>]>()?;

        Ok(StaticSt {
            name,
            paren_token,
            formal_args,
            template_body,
        })
    }
}

impl ToTokens for StaticSt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format!(r#""{}""#, &self.name);
        let expanded = quote! {
            templates.insert(#name, ::string_template::St {
                imp: ::string_template::CompiledSt::new("#template_body", vec![]),
                attributes: ::string_template::Attributes::new(),
            });
        };
        tokens.extend(expanded);
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

#[proc_macro]
pub fn st_group(input: TokenStream) -> TokenStream {
    let group: StaticStGroup = parse_macro_input!(input as StaticStGroup);
    group.into_token_stream().into()
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
