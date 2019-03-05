#![recursion_limit = "128"]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

use lazy_static::lazy_static;

use serde::Serialize;

mod context;
pub use crate::context::Context;

mod error;
pub use crate::error::Error;

mod interpreter;
pub use crate::interpreter::Interpreter;

mod parse;
pub use crate::parse::pest::TemplateParser;
pub use crate::parse::syn::{Group as StaticGroup, GroupBody};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Literal(String),
    Attribute(String),
    AttributePath(String, Vec<String>),
    Include(String, Vec<String>),
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
}

impl FromStr for CompiledTemplate {
    type Err = Error;

    fn from_str(template: &str) -> Result<CompiledTemplate, Self::Err> {
        let expressions = TemplateParser::expressions_of(template)?;
        Ok(CompiledTemplate::new(template, expressions))
    }
}

type AttributeMap = HashMap<String, Context>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attributes(AttributeMap);

impl Attributes {
    pub fn new() -> Attributes {
        Attributes(AttributeMap::new())
    }

    pub fn insert(&mut self, name: impl Into<String>, value: Context) {
        self.0.entry(name.into()).or_default().concat(value);
    }

    pub fn get(&self, name: impl AsRef<str>) -> &Context {
        lazy_static! {
            static ref NULL_CONTEXT: Context = Context::null();
        }
        self.0.get(name.as_ref()).unwrap_or(&NULL_CONTEXT)
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

#[derive(Clone, Debug, Default, PartialEq)]
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

    pub fn add(
        &mut self,
        name: impl Into<String>,
        value: impl Serialize,
    ) -> Result<&mut Self, Error> {
        self.attributes.insert(name, Context::wraps(value)?);
        Ok(self)
    }

    pub fn render(&self) -> String {
        let template = self.imp.clone();
        let group = self.group.clone();
        let interpreter = Interpreter::new(group);
        interpreter.render(&template, &self.attributes)
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
