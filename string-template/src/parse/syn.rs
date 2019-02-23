use std::collections::HashMap;
use std::str::FromStr;
use std::{cmp, fmt};

use quote::ToTokens;
use quote::{quote, quote_spanned};

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Ident, Token, Visibility};

use crate::parse::pest::StParser;
use crate::{CompiledTemplate, Error, Expr, Group};

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
    group: GroupBody,
}

impl StaticGroup {
    pub fn new(visibility: Visibility, group_name: Ident) -> StaticGroup {
        StaticGroup {
            visibility,
            group_name,
            brace_token: Default::default(),
            group: Default::default(),
        }
    }

    pub fn parse_str(template: impl AsRef<str>) -> Result<StaticGroup, Error> {
        syn::parse_str(template.as_ref()).map_err(|e| Error::syn(template, e))
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
        let visibility: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        input.parse::<Token![ref]>()?;
        let group_name = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let group = GroupBody::new(visibility.clone(), &content)?;
        Ok(StaticGroup {
            visibility,
            group_name,
            brace_token,
            group,
        })
    }
}

impl ToTokens for StaticGroup {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = quote! { ::string_template::Group };
        let templates = &self.group;
        let visibility = &self.visibility;
        let template_access_fns = self.group.template_access_fns();
        let group_name = &self.group_name;
        let expanded = quote_spanned! {
            self.brace_token.span =>
                #[allow(non_camel_case_types)]
                #visibility struct #group_name;

                impl #group_name {
                    #template_access_fns
                }

                impl ::std::ops::Deref for #group_name {
                    type Target = #ty;

                    fn deref(&self) -> &#ty {
                        static ONCE: ::std::sync::Once = ::std::sync::ONCE_INIT;
                        static mut VALUE: *mut #ty = 0 as *mut #ty;

                        fn init() -> #ty {
                            let mut templates = ::std::collections::HashMap::new();
                            #templates
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

#[derive(Clone, Debug, PartialEq)]
pub struct GroupBody {
    visibility: Visibility,
    templates: Punctuated<StaticSt, NoneDelimiter>,
}

impl GroupBody {
    pub fn new(visibility: Visibility, input: ParseStream) -> syn::Result<GroupBody> {
        let templates = input.parse_terminated(StaticSt::parse)?;
        Ok(GroupBody {
            visibility,
            templates,
        })
    }

    pub fn templates(self) -> HashMap<String, CompiledTemplate> {
        self.templates
            .into_iter()
            .map(|st| (st.name.to_string(), st.template_body.into()))
            .collect()
    }

    pub fn template_access_fns(&self) -> proc_macro2::TokenStream {
        let template_access_fns = self
            .templates
            .iter()
            .map(|st| st.access_fn(&self.visibility));
        quote! { #( #template_access_fns )* }
    }
}

fn public_visibility() -> Visibility {
    Visibility::Public(syn::VisPublic {
        pub_token: Default::default(),
    })
}

impl Default for GroupBody {
    fn default() -> GroupBody {
        GroupBody {
            visibility: public_visibility(),
            templates: Default::default(),
        }
    }
}

impl From<GroupBody> for Group {
    fn from(static_group: GroupBody) -> Group {
        let templates = static_group.templates();
        Group::from(templates)
    }
}

impl FromStr for GroupBody {
    type Err = Error;

    fn from_str(template: &str) -> Result<GroupBody, Self::Err> {
        syn::parse_str(template).map_err(|e| Error::syn(template, e))
    }
}

impl Parse for GroupBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let visibility = public_visibility();
        GroupBody::new(visibility, input)
    }
}

impl ToTokens for GroupBody {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let templates = &self.templates;
        let expanded = quote! { #( #templates )* };
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
            templates.insert(#name.to_string(), ::string_template::Template {
                imp: ::string_template::CompiledTemplate::new(#template_body, #compiled_template),
                attributes: ::string_template::Attributes::new(),
            });
        };
        tokens.extend(expanded);
    }
}

#[derive(Clone)]
struct TemplateBody {
    literal: syn::LitStr,
    expressions: Vec<Expr>,
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

impl From<TemplateBody> for CompiledTemplate {
    fn from(body: TemplateBody) -> CompiledTemplate {
        let TemplateBody {
            literal,
            expressions,
        } = body;
        CompiledTemplate::new(literal.value(), expressions)
    }
}

impl Parse for TemplateBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literal: syn::LitStr = input.parse()?;
        let expressions = StParser::expressions_of(&literal.value())?;
        Ok(TemplateBody {
            literal,
            expressions,
        })
    }
}

impl ToTokens for TemplateBody {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expressions = &self.expressions;
        tokens.extend(quote! {
            vec![
                #( #expressions ),*
            ]
        });
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expanded = match self {
            Expr::Literal(content) => {
                quote! { ::string_template::Expr::Literal(#content.to_string()) }
            }
            Expr::Attribute(name) => {
                quote! { ::string_template::Expr::Attribute(#name.to_string()) }
            }
            Expr::AttributePath(name, path) => {
                quote! {
                    ::string_template::Expr::AttributePath(
                        #name.to_string(),
                        vec![ #( #path.to_string() ),* ]
                    )
                }
            }
            Expr::Include(name, arg_names) => {
                quote! { ::string_template::Expr::Include(#name, vec![ #( #arg_names ),* ]) }
            }
        };
        tokens.extend(expanded);
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
