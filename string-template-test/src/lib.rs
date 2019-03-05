use serde::Serialize;
use string_template::{Group, Template};

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

pub fn get_template(group: &Group, name: &'static str) -> Template {
    group
        .get(name)
        .unwrap_or_else(|| panic!("unexpectedly failed to find template {}", name))
}

pub fn parse_group(group: &'static str) -> Group {
    match group.parse() {
        Ok(group) => group,
        Err(error) => panic!("unexpectedly failed to parse Group: {}", error),
    }
}
