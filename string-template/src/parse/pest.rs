use pest_derive::Parser;

#[derive(Copy, Clone, Debug, Parser)]
#[grammar = "st.pest"]
pub struct StParser;

#[cfg(test)]
mod tests {
    use pest::iterators::Pairs;
    use pest::Parser;

    use super::*;

    use pest::{consumes_to, parses_to};

    fn parse_file(template: &'static str) -> Pairs<'_, Rule> {
        StParser::parse(Rule::file, template).expect("unexpectedly failed to parse template")
    }

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
                        single_line_body(8, 15, [
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
                        single_line_body(9, 16, [
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
                        single_line_body(15, 22, [
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
                        multi_line_body(15, 20, [
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
                        multi_line_body(15, 22, [
                            multi_line_literal(16, 21)
                        ])
                    ])
                ])
            ]
        };
    }
}
