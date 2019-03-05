use serde::Serialize;
use string_template::Template;

#[cfg(test)]
mod tests;

pub trait TemplateTestExt {
    fn add_expect(&mut self, name: impl Into<String>, value: impl Serialize) -> &mut Self;
}

impl TemplateTestExt for Template {
    fn add_expect(&mut self, name: impl Into<String>, value: impl Serialize) -> &mut Self {
        self.add(name, value)
            .expect("unexpectedly failed to add attribute to template")
    }
}
