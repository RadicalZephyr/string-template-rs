#![recursion_limit = "128"]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

use indexmap::IndexSet;

use lazy_static::lazy_static;

use serde::ser::{Serialize, Serializer};

mod context;
pub use crate::context::Context;

mod error;
pub use crate::error::Error;

mod interpreter;
pub use crate::interpreter::Interpreter;

mod parse;
pub use crate::parse::pest::TemplateParser;
pub use crate::parse::syn::{AsDynamicTemplate, Group as StaticGroup, GroupBody};

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
    formal_arguments: Option<IndexSet<String>>,
    expressions: Vec<Expr>,
}

impl CompiledTemplate {
    pub fn new(template: impl Into<String>, expressions: Vec<Expr>) -> CompiledTemplate {
        CompiledTemplate {
            template: template.into(),
            formal_arguments: None,
            expressions,
        }
    }

    pub fn with_args(
        template: impl Into<String>,
        formal_arguments: impl IntoIterator<Item = String>,
        expressions: Vec<Expr>,
    ) -> CompiledTemplate {
        CompiledTemplate {
            template: template.into(),
            formal_arguments: Some(formal_arguments.into_iter().collect()),
            expressions,
        }
    }

    pub fn assert_is_argument(&self, arg_name: impl AsRef<str>) -> Result<(), Error> {
        let arg_name = arg_name.as_ref();
        match &self.formal_arguments {
            Some(formal_arguments) if formal_arguments.contains(arg_name) => Ok(()),
            Some(_) => Err(Error::NoSuchAttribute(arg_name.to_string())),
            None => Ok(()),
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

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type Item = (&'a String, &'a Context);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Context>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl IntoIterator for Attributes {
    type Item = (String, Context);
    type IntoIter = std::collections::hash_map::IntoIter<String, Context>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
        let name = name.into();
        self.imp.assert_is_argument(&name)?;
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

impl Serialize for Template {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.attributes.len()))?;
        for (k, v) in &self.attributes {
            map.serialize_entry(k, v.borrow())?;
        }
        map.end()
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
