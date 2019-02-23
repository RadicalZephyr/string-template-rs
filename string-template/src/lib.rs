#![recursion_limit = "128"]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

mod error;
pub use crate::error::Error;

mod parse;
pub use crate::parse::pest::StParser;
pub use crate::parse::syn::{GroupBody, StaticGroup};

#[cfg(test)]
mod tests;

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

type AttributeMap = HashMap<String, String>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Attributes(AttributeMap);

impl Attributes {
    pub fn new() -> Attributes {
        Attributes(AttributeMap::new())
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.0.insert(name.into(), value.into());
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&String> {
        self.0.get(name.as_ref())
    }
}

type TemplateMap = HashMap<String, CompiledTemplate>;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Group(Rc<RefCell<TemplateMap>>);

impl Group {
    pub fn new() -> Group {
        Group::default()
    }

    pub fn get(&self, template_name: impl AsRef<str>) -> Option<Template> {
        let group = self.clone();
        self.0
            .borrow()
            .get(template_name.as_ref())
            .cloned()
            .map(move |imp| Template::new(group, imp))
    }
}

impl Clone for Group {
    fn clone(&self) -> Group {
        Group(Rc::clone(&self.0))
    }
}

impl From<TemplateMap> for Group {
    fn from(templates: TemplateMap) -> Group {
        Group(Rc::new(RefCell::new(templates)))
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
    pub group: Group,
    pub imp: CompiledTemplate,
    pub attributes: Attributes,
}

impl Template {
    pub fn new(group: Group, imp: CompiledTemplate) -> Template {
        Template {
            group,
            imp,
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

impl From<CompiledTemplate> for Template {
    fn from(compiled: CompiledTemplate) -> Template {
        Template {
            group: Group::default(),
            imp: compiled,
            attributes: Attributes::new(),
        }
    }
}
