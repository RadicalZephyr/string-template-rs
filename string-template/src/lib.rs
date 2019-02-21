#![recursion_limit = "128"]

use std::collections::HashMap;
use std::str::FromStr;

mod error;
pub use crate::error::Error;

mod parse;
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
pub struct Group(HashMap<String, Template>);

impl Group {
    pub fn new(templates: HashMap<String, Template>) -> Group {
        Group(templates)
    }

    pub fn get(&self, template_name: impl AsRef<str>) -> Option<Template> {
        self.0.get(template_name.as_ref()).cloned()
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
    pub imp: CompiledSt,
    pub attributes: Attributes,
}

impl Template {
    pub fn new(template: impl Into<String>) -> Template {
        Template {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_hello_world() {
        let mut hello = Template::new("Hello, <name>!");
        hello.add("name", "World");
        assert_eq!("Hello, World!", format!("{}", hello.render()));
    }

    #[test]
    fn renders_multiple_attributes() {
        let mut hello = Template::new("Hello, <title><name>!");
        hello.add("name", "World");
        hello.add("title", "Old ");
        assert_eq!("Hello, Old World!", format!("{}", hello.render()));
    }

    #[test]
    fn renders_missing_attributes_as_empty_string() {
        let mut hello = Template::new("Hello, <title><name>!");
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
        let group: Group = parse_group(
            r#"
a() ::= "FOO"
"#,
        );
        let a = get_template(&group, "a");
        assert_eq!("FOO", a.render());
    }
}
