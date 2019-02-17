#![recursion_limit = "128"]

use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::{cmp, fmt};

use failure::Fail;

use proc_macro2::LineColumn;

use quote::ToTokens;
use quote::{quote, quote_spanned};

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Ident, Token, Visibility};

mod parser;

#[derive(Clone, Debug, Fail, PartialEq, Eq)]
#[fail(display = "{}", _0)]
pub struct Error(String);

fn make_multi_line_error(
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    template: &str,
    error: syn::Error,
) -> String {
    let width = if end_col > start_col {
        end_col - start_col
    } else {
        1
    };
    let mut out: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let mut lines = template.lines();
    let mut idx = 0;
    while let Some(line) = lines.next() {
        idx += 1;
        write!(out, "{}\n", line).ok();
        if idx >= start_line {
            break;
        }
    }

    let line_column = start_col + 1;

    let mut num_vertical_lines = end_line - start_line - 1;
    while let Some(arrow_line) = lines.next() {
        if arrow_line.len() > line_column {
            num_vertical_lines -= 1;
            write!(out, "{}\n", arrow_line).ok();
            continue;
        }
        write!(out, "{:length$}^\n", arrow_line, length = start_col).ok();
        break;
    }

    let vertical_lines: Vec<_> = lines.take(num_vertical_lines).collect();
    for line in vertical_lines {
        let bar = if line.len() > line_column { "" } else { "|" };
        write!(out, "{:length$}{}\n", line, bar, length = start_col).ok();
    }

    write!(
        out,
        "{ep:before$}{ep:^^width$}{ep:after$}|\n",
        ep = "",
        before = end_col - 1,
        width = width,
        after = start_col - end_col,
    )
    .ok();

    write!(
        out,
        "{ep:before$}|{ep:width$}{ep:-^after$}| {}",
        error,
        ep = "",
        before = end_col - 1,
        width = width - 1,
        after = start_col - end_col,
    )
    .ok();

    String::from_utf8_lossy(out.get_ref()).to_string()
}

impl Error {
    pub fn new(template: impl AsRef<str>, error: syn::Error) -> Error {
        let template = template.as_ref();
        let span = error.span();
        let LineColumn {
            line: start_line,
            column: start_col,
        } = span.start();
        let LineColumn {
            line: end_line,
            column: end_col,
        } = span.end();

        if start_line == end_line && end_col > start_col {
            let template_lines = template.lines().take(start_line).collect::<Vec<&'_ str>>();
            let msg = format!(
                "{}\n{ep:spacing$}{ep:^^width$} {}",
                template_lines.join("\n"),
                error,
                ep = "",
                spacing = start_col,
                width = end_col - start_col
            );
            Error(msg)
        } else {
            let msg =
                make_multi_line_error(start_line, start_col, end_line, end_col, template, error);
            Error(msg)
        }
    }
}

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

    pub fn get(&self, template_name: impl AsRef<str>) -> Option<St> {
        self.0.get(template_name.as_ref()).cloned()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct NoneDelimiter;

impl Parse for NoneDelimiter {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(NoneDelimiter)
    }
}

#[derive(Clone)]
pub struct StaticStGroup {
    visibility: Visibility,
    group_name: Ident,
    brace_token: token::Brace,
    templates: Punctuated<StaticSt, NoneDelimiter>,
}

impl StaticStGroup {
    pub fn new(visibility: Visibility, group_name: Ident) -> StaticStGroup {
        StaticStGroup {
            visibility,
            group_name,
            brace_token: Default::default(),
            templates: Default::default(),
        }
    }

    pub fn parse_str(template: impl AsRef<str>) -> Result<StaticStGroup, Error> {
        syn::parse_str(template.as_ref()).map_err(|e| Error::new(template, e))
    }
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
    fn parse(input: ParseStream) -> syn::Result<Self> {
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

impl StaticSt {
    pub fn access_fn(&self, vis: &Visibility) -> proc_macro2::TokenStream {
        let name = &self.name;
        let name_str = name.to_string();
        quote! {
            #vis fn #name(&self) -> St {
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

    fn parse_group(template: &'static str) -> StaticStGroup {
        match StaticStGroup::parse_str(template) {
            Ok(actual) => actual,
            Err(error) => {
                panic!("unexpectedly failed to parse template:\n{}\n", error);
            }
        }
    }

    fn error_message(template: &'static str) -> String {
        match StaticStGroup::parse_str(template) {
            Ok(actual) => panic!("unexpectedly parsed invalid template: {}", template),
            Err(error) => error.to_string(),
        }
    }

    #[test]
    fn parse_no_arg_template() {
        assert_eq!(
            parse_group(r#"static ref group_a { a() ::= "foo" }"#),
            StaticStGroup::new(
                Visibility::Public(syn::VisPublic {
                    pub_token: token::Pub::default(),
                }),
                Ident::new("group_a", Span::call_site())
            ),
        );
    }

    #[test]
    fn show_error_in_single_line_template() {
        assert_eq!(
            r#"
static bunny ref group_a { a() ::= "foo" }
       ^^^^^ expected `ref`"#,
            error_message(
                r#"
static bunny ref group_a { a() ::= "foo" }"#
            )
        );
    }

    #[test]
    fn show_single_line_error_in_multi_line_template() {
        assert_eq!(
            r#"
static ref group_a {
 a() lemons ::= "foo"
     ^^^^^^ expected `::`"#,
            error_message(
                r#"
static ref group_a {
 a() lemons ::= "foo"
}"#,
            )
        );
    }

    #[test]
    fn show_multi_line_error_in_multi_line_template() {
        assert_eq!(
            r#"
static ref group_a { (
 a() ) ::= "foo"     ^
     ^               |
     |---------------| expected identifier"#,
            error_message(
                r#"
static ref group_a { (
 a() ) ::= "foo"
}"#,
            )
        );
    }

    #[test]
    fn show_multi_line_error_in_multi_line_template_with_longer_vertical_lines() {
        assert_eq!(
            r#"
static ref group_a { (
                     ^
++++++++++++++++++++++++++++++
 a() ) ::= "foo"     |
     ^               |
     |---------------| expected identifier"#,
            error_message(
                r#"
static ref group_a { (

++++++++++++++++++++++++++++++
 a() ) ::= "foo"
}"#,
            )
        );
    }

    #[test]
    fn show_multi_line_error_in_multi_line_template_with_long_arrow_line() {
        assert_eq!(
            r#"
static ref group_a { (
++++++++++++++++++++++++++++++
 a() ) ::= "foo"     ^
     ^               |
     |---------------| expected identifier"#,
            error_message(
                r#"
static ref group_a { (
++++++++++++++++++++++++++++++
 a() ) ::= "foo"
}"#,
            )
        );
    }
}
