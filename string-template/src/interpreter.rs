use crate::{Attributes, CompiledTemplate, Expr, Group};

pub struct Interpreter {
    group: Group,
}

impl Interpreter {
    pub fn new(group: Group) -> Interpreter {
        Interpreter { group }
    }

    pub fn render(&self, template: &CompiledTemplate, attributes: &Attributes) -> String {
        let mut out = String::new();
        for expr in &template.expressions {
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
