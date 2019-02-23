use crate::{Attributes, CompiledTemplate, Context, Expr, Group, Template};

pub struct Interpreter {
    group: Group,
}

impl Interpreter {
    pub fn new(group: Group) -> Interpreter {
        Interpreter { group }
    }

    pub fn render(&self, template: &CompiledTemplate, attributes: &Attributes) -> String {
        let e = Context::null();

        let mut out = String::new();
        for expr in &template.expressions {
            match expr {
                Expr::Literal(s) => out.push_str(s),
                Expr::Attribute(name) => {
                    let rendered = attributes.get(name).unwrap_or(&e).to_string();
                    out.push_str(&rendered);
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
