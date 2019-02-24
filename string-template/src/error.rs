use failure::Fail;

use serde_json::error::Error as SerdeError;

use crate::parse::Error as ParseError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Parse(ParseError),

    #[fail(display = "{:?}", _0)]
    Serde(SerdeError),

    #[fail(display = "no such attribute: {}", _0)]
    NoSuchAttribute(String),
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        use self::Error::*;

        match (self, other) {
            (Parse(_), Parse(_)) | (Serde(_), Serde(_)) => true,
            (NoSuchAttribute(name1), NoSuchAttribute(name2)) if name1 == name2 => true,
            _ => false,
        }
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Error {
        Error::Parse(error)
    }
}

impl From<SerdeError> for Error {
    fn from(error: SerdeError) -> Error {
        Error::Serde(error)
    }
}

#[cfg(all(test, procmacro2_semver_exempt))]
mod tests {
    use crate::StaticGroup;

    fn error_message(template: &'static str) -> String {
        match StaticGroup::parse_str(template) {
            Ok(_) => panic!("unexpectedly parsed invalid template: {}", template),
            Err(error) => error.to_string(),
        }
    }

    #[test]
    fn show_error_in_single_line_template() {
        assert_eq!(
            r#"
static bunny ref group_a { a() ::= "foo" }
       ^^^^^ expected `ref`"#,
            error_message(
                r#"
static bunny ref group_a { a() ::= "foo" }"#
            )
        );
    }

    #[test]
    fn show_single_line_error_in_multi_line_template() {
        assert_eq!(
            r#"
static ref group_a {
 a() lemons ::= "foo"
     ^^^^^^ expected `::`"#,
            error_message(
                r#"
static ref group_a {
 a() lemons ::= "foo"
}"#,
            )
        );
    }

    #[test]
    fn show_multi_line_error_in_multi_line_template() {
        assert_eq!(
            r#"
static ref group_a { (
 a() ) ::= "foo"     ^
     ^               |
     |---------------| expected identifier"#,
            error_message(
                r#"
static ref group_a { (
 a() ) ::= "foo"
}"#,
            )
        );
    }

    #[test]
    fn show_multi_line_error_in_multi_line_template_with_longer_vertical_lines() {
        assert_eq!(
            r#"
static ref group_a { (
                     ^
++++++++++++++++++++++++++++++
 a() ) ::= "foo"     |
     ^               |
     |---------------| expected identifier"#,
            error_message(
                r#"
static ref group_a { (

++++++++++++++++++++++++++++++
 a() ) ::= "foo"
}"#,
            )
        );
    }

    #[test]
    fn show_multi_line_error_in_multi_line_template_with_long_arrow_line() {
        assert_eq!(
            r#"
static ref group_a { (
++++++++++++++++++++++++++++++
 a() ) ::= "foo"     ^
     ^               |
     |---------------| expected identifier"#,
            error_message(
                r#"
static ref group_a { (
++++++++++++++++++++++++++++++
 a() ) ::= "foo"
}"#,
            )
        );
    }
}
