use pest::iterators::{Pair, Pairs};
use pest::Parser;

use pest_derive::Parser;

use crate::parse::Error;
use crate::Expr;

fn parse_field_reference(mut exprs: Pairs<Rule>) -> Result<Expr, Error> {
    let name = exprs.next().unwrap().as_str().to_string();
    let path: Vec<String> = exprs
        .map(|expr| match expr.as_rule() {
            Rule::identifier => expr.as_str().to_string(),
            rule => unreachable!("unexpected rule: {:?}", rule),
        })
        .collect();

    if path.is_empty() {
        Ok(Expr::Attribute(name))
    } else {
        Ok(Expr::AttributePath(name, path))
    }
}

fn parse_expr(expr: Pair<Rule>) -> Result<Expr, Error> {
    match expr.as_rule() {
        Rule::field_reference => parse_field_reference(expr.into_inner()),
        Rule::template_include => {
            let mut content = expr.into_inner();
            let literal = content.next().unwrap().as_str();
            Ok(Expr::Include(literal.to_string(), vec![]))
        }
        rule => unimplemented!("{:?}", rule),
    }
}

fn parse_expression(expression: Pair<Rule>) -> Result<Expr, Error> {
    match expression.as_rule() {
        Rule::literal => {
            let literal = expression.as_str();
            Ok(Expr::Literal(literal.to_string()))
        }
        Rule::expression => parse_expr(expression.into_inner().next().unwrap()),
        rule => unimplemented!("{:?}", rule),
    }
}

#[derive(Copy, Clone, Debug, Parser)]
#[grammar = "template.pest"]
pub struct TemplateParser;

impl TemplateParser {
    pub fn expressions_of(template: &str) -> Result<Vec<Expr>, Error> {
        let mut pairs = TemplateParser::parse(Rule::template_body, template)?;
        pairs
            .next()
            .unwrap()
            .into_inner()
            .map(parse_expression)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pest::{consumes_to, parses_to};

    #[test]
    fn parse_structure() {
        parses_to! {
            parser: TemplateParser,
            input: r#"<greeting> <person.name>! <message()>"#,
            rule: Rule::template_body,
            tokens: [
                template_body(0, 37, [
                    literal(0, 0),
                    expression(0, 10, [
                        field_reference(1, 9, [
                            identifier(1, 9)
                        ])
                    ]),
                    literal(10, 11),
                    expression(11, 24, [
                        field_reference(12, 23, [
                            identifier(12, 18),
                            identifier(19, 23)
                        ])
                    ]),
                    literal(24, 26),
                    expression(26, 37, [
                        template_include(27, 36, [
                            identifier(27, 34)
                        ])
                    ]),
                    literal(37, 37)
                ])
            ]
        }
    }
}
