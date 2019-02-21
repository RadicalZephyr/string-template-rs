use std::collections::HashMap;
use std::{cmp, fmt};

use quote::ToTokens;
use quote::{quote, quote_spanned};

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Ident, Token, Visibility};

use crate::{Error, Group, Template};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct NoneDelimiter;

impl Parse for NoneDelimiter {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(NoneDelimiter)
    }
}

#[derive(Clone)]
pub struct StaticGroup {
    visibility: Visibility,
    group_name: Ident,
    brace_token: token::Brace,
    templates: Punctuated<StaticSt, NoneDelimiter>,
}

impl StaticGroup {
    pub fn new(visibility: Visibility, group_name: Ident) -> StaticGroup {
        StaticGroup {
            visibility,
            group_name,
            brace_token: Default::default(),
            templates: Default::default(),
        }
    }

    pub fn parse_str(template: impl AsRef<str>) -> Result<StaticGroup, Error> {
        syn::parse_str(template.as_ref()).map_err(|e| Error::new(template, e))
    }

    pub fn templates(self) -> HashMap<String, Template> {
        self.templates
            .into_iter()
            .map(|st| (st.name.to_string(), st.template_body.into()))
            .collect()
    }
}

impl fmt::Debug for StaticGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Group")
            .field("group_name", &self.group_name)
            .finish()
    }
}

impl cmp::PartialEq for StaticGroup {
    fn eq(&self, other: &Self) -> bool {
        self.group_name == other.group_name
    }
}

impl Parse for StaticGroup {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        input.parse::<Token![ref]>()?;
        let group_name = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let templates = content.parse_terminated(StaticSt::parse)?;
        Ok(StaticGroup {
            visibility,
            group_name,
            templates,
            brace_token,
        })
    }
}

impl From<StaticGroup> for Group {
    fn from(static_group: StaticGroup) -> Group {
        let templates = static_group.templates();
        Group(templates)
    }
}

impl ToTokens for StaticGroup {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = quote! { ::string_template::Group };
        let templates = &self.templates;
        let visibility = &self.visibility;
        let template_access_fns = self.templates.iter().map(|st| st.access_fn(&visibility));
        let group_name = &self.group_name;
        let expanded = quote_spanned! {
            self.brace_token.span =>
                #[allow(non_camel_case_types)]
                #visibility struct #group_name;

                impl #group_name {
                    #( #template_access_fns )*
                }

                impl ::std::ops::Deref for #group_name {
                    type Target = #ty;

                    fn deref(&self) -> &#ty {
                        static ONCE: ::std::sync::Once = ::std::sync::ONCE_INIT;
                        static mut VALUE: *mut #ty = 0 as *mut #ty;

                        fn init() -> #ty {
                            let mut templates = ::std::collections::HashMap::new();
                            #( #templates )*
                            ::string_template::Group::new(templates)
                        }

                        unsafe {
                            ONCE.call_once(|| VALUE = Box::into_raw(Box::new(init())));
                            &*VALUE
                        }
                    }
                }
        };
        tokens.extend(expanded);
    }
}

#[derive(Clone)]
pub struct StaticSt {
    name: Ident,
    paren_token: token::Paren,
    formal_args: Punctuated<Ident, Token![,]>,
    template_body: TemplateBody,
}

impl StaticSt {
    pub fn access_fn(&self, vis: &Visibility) -> proc_macro2::TokenStream {
        let name = &self.name;
        let name_str = name.to_string();
        quote! {
            #vis fn #name(&self) -> Template {
                self.get(#name_str).unwrap()
            }
        }
    }
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
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let content;
        let paren_token = parenthesized!(content in input);
        let formal_args = content.parse_terminated(Ident::parse)?;

        input.parse::<Token![::]>()?;
        input.parse::<Token![=]>()?;

        let template_body = input.parse()?;

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
        let name = self.name.to_string();
        let template_body = &self.template_body.to_string();
        let compiled_template = &self.template_body;
        let expanded = quote! {
            templates.insert(#name, ::string_template::Template {
                imp: ::string_template::CompiledSt::new(#template_body, #compiled_template),
                attributes: ::string_template::Attributes::new(),
            });
        };
        tokens.extend(expanded);
    }
}

#[derive(Clone)]
struct TemplateBody {
    literal: syn::LitStr,
}

impl fmt::Display for TemplateBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.literal.value())
    }
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

// In the body of a template, whitespace is critically important, so
// the Syn parser may not be the best tool. It will probably be easier
// to write this using a pest parser since syn doesn't really have any
// tools for showing me the whitespace.
impl Parse for TemplateBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TemplateBody {
            literal: input.parse()?,
        })
    }
}

impl From<TemplateBody> for Template {
    fn from(template: TemplateBody) -> Template {
        Template::new(template.to_string())
    }
}

impl ToTokens for TemplateBody {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let content = self.literal.value();
        tokens.extend(quote! { vec![::string_template::Expr::Literal(#content.to_string())] });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proc_macro2::Span;

    fn parse_static_group(template: &'static str) -> StaticGroup {
        match StaticGroup::parse_str(template) {
            Ok(actual) => actual,
            Err(error) => {
                panic!("unexpectedly failed to parse template:\n{}\n", error);
            }
        }
    }

    #[test]
    fn parse_no_arg_literal_template() {
        assert_eq!(
            parse_static_group(r#"static ref group_a { a() ::= "foo" }"#),
            StaticGroup::new(
                Visibility::Public(syn::VisPublic {
                    pub_token: token::Pub::default(),
                }),
                Ident::new("group_a", Span::call_site())
            ),
        );
    }
}
