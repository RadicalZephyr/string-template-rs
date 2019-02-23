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
        Rule::single_line_literal | Rule::multi_line_literal => {
            let literal = expression.as_str();
            Ok(Expr::Literal(literal.to_string()))
        }
        Rule::expr => parse_expr(expression.into_inner().next().unwrap()),
        rule => unimplemented!("{:?}", rule),
    }
}

#[derive(Copy, Clone, Debug, Parser)]
#[grammar = "st.pest"]
pub struct StParser;

impl StParser {
    pub fn expressions_of(template: &str) -> Result<Vec<Expr>, Error> {
        let mut pairs = StParser::parse(Rule::multi_line_body, template)?;
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

    use crate::{CompiledTemplate, Template};

    #[test]
    fn parse_no_arg_template() {
        parses_to! {
            parser: StParser,
            input: "a() ::= <<foo>>",
            rule: Rule::template,
            tokens: [
                template(0, 15, [
                    identifier(0, 1),
                    template_body(8, 15, [
                        single_line_body(10, 13, [
                            single_line_literal(10, 13)
                        ])
                    ])
                ])
            ]
        };
    }

    #[test]
    fn parse_one_arg_template() {
        parses_to! {
            parser: StParser,
            input: "a(x) ::= <<foo>>",
            rule: Rule::template,
            tokens: [
                template(0, 16, [
                    identifier(0, 1),
                    formal_args(2, 3, [
                        identifier(2, 3),
                    ]),
                    template_body(9, 16, [
                        single_line_body(11, 14, [
                            single_line_literal(11, 14)
                        ])
                    ])
                ])
            ]
        };
    }

    #[test]
    fn parse_multiple_arg_template() {
        parses_to! {
            parser: StParser,
            input: "a(x, y, z) ::= <<foo>>",
            rule: Rule::template,
            tokens: [
                template(0, 22, [
                    identifier(0, 1),
                    formal_args(2, 9, [
                        identifier(2, 3),
                        identifier(5, 6),
                        identifier(8, 9),
                    ]),
                    template_body(15, 22, [
                        single_line_body(17, 20, [
                            single_line_literal(17, 20)
                        ])
                    ])
                ])
            ]
        };
    }

    #[test]
    fn parse_multiple_line_style_template() {
        parses_to! {
            parser: StParser,
            input: r#"a(x, y, z) ::= "foo""#,
            rule: Rule::template,
            tokens: [
                template(0, 20, [
                    identifier(0, 1),
                    formal_args(2, 9, [
                        identifier(2, 3),
                        identifier(5, 6),
                        identifier(8, 9),
                    ]),
                    template_body(15, 20, [
                        multi_line_body(16, 19, [
                            multi_line_literal(16, 19)
                        ])
                    ])
                ])
            ]
        };
    }

    #[test]
    fn parse_multiple_line_template() {
        parses_to! {
            parser: StParser,
            input: r#"a(x, y, z) ::= "
foo
""#,
            rule: Rule::template,
            tokens: [
                template(0, 22, [
                    identifier(0, 1),
                    formal_args(2, 9, [
                        identifier(2, 3),
                        identifier(5, 6),
                        identifier(8, 9),
                    ]),
                    template_body(15, 22, [
                        multi_line_body(16, 21, [
                            multi_line_literal(16, 21)
                        ])
                    ])
                ])
            ]
        };
    }

    fn parse_template(template: &'static str) -> Template {
        template.parse::<CompiledTemplate>().unwrap().into()
    }

    #[test]
    fn parse_into_template() {
        let template: Template = parse_template("foo");
        assert_eq!("foo", template.render());
    }

    #[test]
    fn parse_into_template_with_expression() {
        let mut hello: Template = parse_template("Hello <name>!");
        hello.add("name", "World");
        assert_eq!("Hello World!", hello.render());
    }
}
