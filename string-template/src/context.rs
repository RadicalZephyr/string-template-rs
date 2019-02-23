use std::fmt;

use serde::Serialize;
use serde_json::value::{to_value, Value as Json};

use crate::Error;

/// The context wraps the attribute values attached to a template.
///
#[derive(Debug, Clone)]
pub struct Context {
    data: Json,
}

impl Context {
    /// Create a context with null data
    pub fn null() -> Context {
        Context { data: Json::Null }
    }

    /// Create a context with given data
    pub fn wraps<T: Serialize>(e: T) -> Result<Context, Error> {
        to_value(e)
            .map_err(Error::from)
            .map(|d| Context { data: d })
    }
}

impl Context {
    pub fn navigate(&self, path: &[String]) -> Context {
        let mut node = &self.data;
        for segment in path {
            match node {
                Json::Object(map) => node = map.get(segment).unwrap_or(&Json::Null),
                _ => break,
            }
        }

        Context::wraps(node).unwrap()
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data.render())
    }
}

/// Render Json data with default format
trait JsonRender {
    fn render(&self) -> String;
}

impl JsonRender for Json {
    fn render(&self) -> String {
        match self {
            Json::String(s) => s.to_string(),
            Json::Bool(i) => i.to_string(),
            Json::Number(n) => n.to_string(),
            Json::Null => "".to_owned(),
            Json::Array(a) => {
                let mut buf = String::new();
                for i in a.iter() {
                    buf.push_str(i.render().as_ref());
                }
                buf
            }
            Json::Object(_) => "[object]".to_owned(),
        }
    }
}
