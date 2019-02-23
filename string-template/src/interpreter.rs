use crate::{Attributes, CompiledTemplate, Expr, Group, Template};

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
                    let attribute = attributes.get(name);
                    out.push_str(&attribute.to_string());
                }
                Expr::AttributePath(attribute_name, path) => {
                    let rendered = attributes.get(attribute_name).navigate(path);
                    out.push_str(&rendered.to_string());
                }
                Expr::Include(name, _arg_names) => {
                    let template = self.group.get(name).unwrap_or_else(Template::default);
                    out.push_str(&template.render());
                }
            }
        }
        out
    }
}
