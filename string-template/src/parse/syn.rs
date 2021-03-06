use std::{cmp, fmt, str};

use proc_macro2::TokenStream;

use quote::ToTokens;
use quote::{quote, quote_spanned};

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Ident, Token, Visibility};

use crate::parse::pest::TemplateParser;
use crate::parse::Error;
use crate::{CompiledTemplate, Expr, Group as RuntimeGroup, TemplateMap};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct NoneDelimiter;

impl Parse for NoneDelimiter {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(NoneDelimiter)
    }
}

#[derive(Clone)]
pub struct Group {
    visibility: Visibility,
    group_name: Ident,
    brace_token: token::Brace,
    group: GroupBody,
}

impl Group {
    pub fn new(visibility: Visibility, group_name: Ident) -> Group {
        Group {
            visibility,
            group_name,
            brace_token: Default::default(),
            group: Default::default(),
        }
    }

    pub fn with_group(visibility: Visibility, group_name: Ident, group: GroupBody) -> Group {
        Group {
            visibility,
            group_name,
            brace_token: Default::default(),
            group,
        }
    }

    pub fn parse_str(template: impl AsRef<str>) -> Result<Group, Error> {
        syn::parse_str(template.as_ref()).map_err(|e| Error::syn(template, e))
    }
}

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Group")
            .field("group_name", &self.group_name)
            .finish()
    }
}

impl cmp::PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.group_name == other.group_name
    }
}

impl Parse for Group {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let visibility: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        input.parse::<Token![ref]>()?;
        let group_name = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let group = GroupBody::new(visibility.clone(), &content)?;
        Ok(Group {
            visibility,
            group_name,
            brace_token,
            group,
        })
    }
}

impl ToTokens for Group {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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
                            ::string_template::Group::from(templates)
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
    templates: Punctuated<Template, NoneDelimiter>,
}

impl GroupBody {
    pub fn new(visibility: Visibility, input: ParseStream) -> syn::Result<GroupBody> {
        let templates = input.parse_terminated(Template::parse)?;
        Ok(GroupBody {
            visibility,
            templates,
        })
    }

    pub fn templates(self) -> TemplateMap {
        self.templates
            .into_iter()
            .map(|st| (st.name.to_string(), st.into()))
            .collect()
    }

    pub fn template_access_fns(&self) -> TokenStream {
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

impl From<GroupBody> for RuntimeGroup {
    fn from(static_group: GroupBody) -> RuntimeGroup {
        let templates = static_group.templates();
        RuntimeGroup::from(templates)
    }
}

impl str::FromStr for GroupBody {
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
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let templates = &self.templates;
        let expanded = quote! { #( #templates )* };
        tokens.extend(expanded);
    }
}

#[derive(Clone)]
pub struct Template {
    name: Ident,
    paren_token: token::Paren,
    formal_args: Punctuated<Ident, Token![,]>,
    template_body: TemplateBody,
}

impl Template {
    pub fn access_fn(&self, vis: &Visibility) -> TokenStream {
        let name = &self.name;
        let name_str = name.to_string();
        quote! {
            #vis fn #name(&self) -> ::string_template::Template {
                self.get(#name_str).unwrap()
            }
        }
    }
}

impl fmt::Debug for Template {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Template")
            .field("name", &self.name)
            .finish()
    }
}

impl cmp::PartialEq for Template {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl From<Template> for CompiledTemplate {
    fn from(body: Template) -> CompiledTemplate {
        let Template {
            formal_args,
            template_body,
            ..
        } = body;
        let TemplateBody {
            literal,
            expressions,
        } = template_body;
        CompiledTemplate::with_args(
            literal.value(),
            formal_args.iter().map(Ident::to_string),
            expressions,
        )
    }
}

impl Parse for Template {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let content;
        let paren_token = parenthesized!(content in input);
        let formal_args = content.parse_terminated(Ident::parse)?;

        input.parse::<Token![::]>()?;
        input.parse::<Token![=]>()?;

        let template_body = input.parse()?;

        Ok(Template {
            name,
            paren_token,
            formal_args,
            template_body,
        })
    }
}

impl ToTokens for Template {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name.to_string();
        let template_body = &self.template_body.to_string();
        let formal_args = self.formal_args.iter().map(Ident::to_string);
        let compiled_template = &self.template_body;
        let expanded = quote! {
            templates.insert(
                #name.to_string(),
                ::string_template::CompiledTemplate::with_args(
                    #template_body,
                    vec![ #( #formal_args.to_string() ),* ],
                    #compiled_template
                )
            );
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

impl Parse for TemplateBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literal: syn::LitStr = input.parse()?;
        let expressions = TemplateParser::expressions_of(&literal.value())?;
        Ok(TemplateBody {
            literal,
            expressions,
        })
    }
}

impl ToTokens for TemplateBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expressions = &self.expressions;
        tokens.extend(quote! {
            vec![
                #( #expressions ),*
            ]
        });
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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
                quote! { ::string_template::Expr::Include(#name.to_string(), vec![ #( #arg_names ),* ]) }
            }
        };
        tokens.extend(expanded);
    }
}

pub trait AsDynamicTemplate {
    fn as_dynamic_template(&self) -> TokenStream;
}

impl AsDynamicTemplate for Group {
    fn as_dynamic_template(&self) -> TokenStream {
        let template_body = self.group.as_dynamic_template();
        quote! { ::string_template_test::parse_group( #template_body ) }
    }
}

impl AsDynamicTemplate for GroupBody {
    fn as_dynamic_template(&self) -> TokenStream {
        let template_str: String = self
            .templates
            .iter()
            .map(Template::as_dynamic_template)
            .map(|tokens| format!("{}\n", tokens))
            .collect();
        quote! { #template_str }
    }
}

impl AsDynamicTemplate for Template {
    fn as_dynamic_template(&self) -> TokenStream {
        let name = &self.name;
        let formal_args = &self.formal_args;
        let template_body = self.template_body.as_dynamic_template();
        quote! { #name ( #( #formal_args ),* ) ::= #template_body }
    }
}

impl AsDynamicTemplate for TemplateBody {
    fn as_dynamic_template(&self) -> TokenStream {
        let literal = &self.literal;
        quote! { #literal }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proc_macro2::Span;

    fn parse_static_group(template: &'static str) -> Group {
        match Group::parse_str(template) {
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
            Group::new(
                Visibility::Public(syn::VisPublic {
                    pub_token: token::Pub::default(),
                }),
                Ident::new("group_a", Span::call_site())
            ),
        );
    }
}
