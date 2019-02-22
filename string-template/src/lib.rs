#![recursion_limit = "128"]

use std::collections::HashMap;
use std::str::FromStr;

mod error;
pub use crate::error::Error;

mod parse;
pub use crate::parse::pest::StParser;
pub use crate::parse::syn::{GroupBody, StaticGroup};

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
pub struct CompiledTemplate {
    template: String,
    // These should really be a vec of `&'a str`, where 'a is the
    // lifetime of _this struct_. But I don't know how to correctly
    // name that lifetime, or if it's even possible. It might not even
    // have meaning to try and say that, since if this vec or an item
    // in it was moved outside of this struct then the lifetimes do
    // matter.
    expressions: Vec<Expr>,
}

impl CompiledTemplate {
    pub fn new(template: impl Into<String>, expressions: Vec<Expr>) -> CompiledTemplate {
        CompiledTemplate {
            template: template.into(),
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

impl FromStr for CompiledTemplate {
    type Err = Error;

    fn from_str(template: &str) -> Result<CompiledTemplate, Self::Err> {
        let expressions = StParser::expressions_of(template)?;
        Ok(CompiledTemplate::new(template, expressions))
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
pub struct Group(HashMap<String, CompiledTemplate>);

impl Group {
    pub fn new(templates: HashMap<String, CompiledTemplate>) -> Group {
        Group(templates)
    }

    pub fn get(&self, template_name: impl AsRef<str>) -> Option<Template> {
        self.0
            .get(template_name.as_ref())
            .cloned()
            .map(Template::from)
    }
}

impl FromStr for Group {
    type Err = Error;

    fn from_str(template: &str) -> Result<Group, Self::Err> {
        let group = template.parse::<GroupBody>()?;
        Ok(group.into())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Template {
    pub imp: CompiledTemplate,
    pub attributes: Attributes,
}

impl Template {
    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(name, value);
    }

    pub fn render(&self) -> String {
        self.imp.render(&self.attributes)
    }
}

impl From<CompiledTemplate> for Template {
    fn from(compiled: CompiledTemplate) -> Template {
        Template {
            imp: compiled,
            attributes: Attributes::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_template(template: &'static str) -> Template {
        template
            .parse::<CompiledTemplate>()
            .expect("unexpectedly failed to parse template")
            .into()
    }

    #[test]
    fn renders_hello_world() {
        let mut hello = parse_template("Hello, <name>!");
        hello.add("name", "World");
        assert_eq!("Hello, World!", format!("{}", hello.render()));
    }

    #[test]
    fn renders_multiple_attributes() {
        let mut hello = parse_template("Hello, <title><name>!");
        hello.add("name", "World");
        hello.add("title", "Old ");
        assert_eq!("Hello, Old World!", format!("{}", hello.render()));
    }

    #[test]
    fn renders_missing_attributes_as_empty_string() {
        let mut hello = parse_template("Hello, <title><name>!");
        hello.add("name", "World");
        assert_eq!("Hello, World!", format!("{}", hello.render()));
    }

    fn parse_group(group: &'static str) -> Group {
        match group.parse() {
            Ok(group) => group,
            Err(error) => panic!("unexpectedly failed to parse Group: {}", error),
        }
    }

    fn get_template(group: &Group, name: &'static str) -> Template {
        group
            .get(name)
            .expect(&format!("unexpectedly failed to get template {}", name))
    }

    #[test]
    fn renders_template_from_group() {
        let group = parse_group(
            r#"
a() ::= "FOO"
"#,
        );
        let a = get_template(&group, "a");
        assert_eq!("FOO", a.render());
    }
}
