#![recursion_limit = "128"]

use std::collections::HashMap;
use std::{cmp, fmt};

use quote::ToTokens;
use quote::{quote, quote_spanned};

use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Ident, Token, Visibility};

mod parser;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Literal(String),
    Attribute(String),
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Literal("".into())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct CompiledSt {
    template: String,
    // These should really be a vec of `&'a str`, where 'a is the
    // lifetime of _this struct_. But I don't know how to correctly
    // name that lifetime, or if it's even possible. It might not even
    // have meaning to try and say that, since if this vec or an item
    // in it was moved outside of this struct then the lifetimes do
    // matter.
    expressions: Vec<Expr>,
}

impl CompiledSt {
    pub fn new(template: impl Into<String>, expressions: Vec<Expr>) -> CompiledSt {
        CompiledSt {
            template: template.into(),
            expressions,
        }
    }

    pub fn compile(template: impl Into<String>) -> CompiledSt {
        enum State {
            Literal,
            Expression,
        };

        let template = template.into();
        let mut expressions = vec![];

        let mut state = State::Literal;
        let mut start = 0;
        let mut i = 0;
        for c in template.bytes() {
            match c {
                b'<' => {
                    expressions.push(Expr::Literal(template[start..i].into()));
                    state = State::Expression;
                    i += 1;
                    start = i;
                }
                b'>' => {
                    expressions.push(Expr::Attribute(template[start..i].into()));
                    state = State::Literal;
                    i += 1;
                    start = i;
                }
                _ => i += 1,
            }
        }
        match state {
            State::Literal => {
                expressions.push(Expr::Literal(template[start..i].into()));
            }
            State::Expression => panic!("encountered unfinished template expression"),
        }

        println!("{:?}", expressions);
        CompiledSt {
            template,
            expressions,
        }
    }

    pub fn render(&self, attributes: &Attributes) -> String {
        let mut out = String::new();
        for expr in &self.expressions {
            match expr {
                Expr::Literal(s) => out.push_str(s),
                Expr::Attribute(name) => {
                    out.push_str(attributes.get(name).unwrap_or(&String::new()))
                }
            }
        }
        out
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Attributes(HashMap<String, String>);

impl Attributes {
    pub fn new() -> Attributes {
        Attributes(HashMap::new())
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.0.insert(name.into(), value.into());
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&String> {
        self.0.get(name.as_ref())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct St {
    pub imp: CompiledSt,
    pub attributes: Attributes,
}

impl St {
    pub fn new(template: impl Into<String>) -> St {
        St {
            imp: CompiledSt::compile(template),
            attributes: Attributes::new(),
        }
    }

    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(name, value);
    }

    pub fn render(&self) -> String {
        self.imp.render(&self.attributes)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StGroup(HashMap<&'static str, St>);

impl StGroup {
    pub fn new(templates: HashMap<&'static str, St>) -> StGroup {
        StGroup(templates)
    }

    pub fn get(&self, template_name: impl AsRef<str>) -> Option<&St> {
        self.0.get(template_name.as_ref())
    }
}

#[derive(Clone)]
pub struct StaticStGroup {
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
        let templates = &self.templates;
        let group_name = &self.group_name;
        let visibility = &self.visibility;
        let expanded = quote_spanned! {
            self.brace_token.span =>
                #[allow(non_camel_case_types)]
                #visibility struct #group_name;

                impl ::std::ops::Deref for #group_name {
                    type Target = #ty;

                    fn deref(&self) -> &#ty {
                        static ONCE: ::std::sync::Once = ::std::sync::ONCE_INIT;
                        static mut VALUE: *mut #ty = 0 as *mut #ty;

                        fn init() -> #ty {
                            let mut templates = ::std::collections::HashMap::new();
                            #( #templates )*
                            ::string_template::StGroup::new(templates)
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
        let name = self.name.to_string();
        let template_body = &self.template_body.to_string();
        let compiled_template = &self.template_body;
        let expanded = quote! {
            templates.insert(#name, ::string_template::St {
                imp: ::string_template::CompiledSt::new(#template_body, #compiled_template),
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

impl fmt::Display for TemplateBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.foo)
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
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TemplateBody {
            foo: input.parse()?,
        })
    }
}

impl ToTokens for TemplateBody {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let content = self.foo.to_string();
        tokens.extend(quote! { vec![::string_template::Expr::Literal(#content.to_string())] });
    }
}

pub fn parse_group(template: &'static str) -> StaticStGroup {
    syn::parse_str(template).expect("unexpected parsing failure")
}

#[cfg(test)]
mod tests {
    use super::*;

    use proc_macro2::Span;

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
