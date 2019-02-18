use std::io::{Cursor, Write};

use failure::Fail;

use proc_macro2::LineColumn;

#[derive(Clone, Debug, Fail, PartialEq, Eq)]
#[fail(display = "{}", _0)]
pub struct Error(String);

fn make_multi_line_error(
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    template: &str,
    error: syn::Error,
) -> String {
    let width = if end_col > start_col {
        end_col - start_col
    } else {
        1
    };
    let mut out: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let mut lines = template.lines();
    let mut idx = 0;
    while let Some(line) = lines.next() {
        idx += 1;
        writeln!(out, "{}", line).ok();
        if idx >= start_line {
            break;
        }
    }

    let line_column = start_col + 1;

    let mut num_vertical_lines = end_line - start_line - 1;
    while let Some(arrow_line) = lines.next() {
        if arrow_line.len() > line_column {
            num_vertical_lines -= 1;
            writeln!(out, "{}", arrow_line).ok();
            continue;
        }
        writeln!(out, "{:length$}^", arrow_line, length = start_col).ok();
        break;
    }

    let vertical_lines: Vec<_> = lines.take(num_vertical_lines).collect();
    for line in vertical_lines {
        let maybe_bar = if line.len() > line_column { "" } else { "|" };
        writeln!(out, "{:length$}{}", line, maybe_bar, length = start_col).ok();
    }

    writeln!(
        out,
        "{ep:before$}{ep:^^width$}{ep:after$}|",
        ep = "",
        before = end_col - 1,
        width = width,
        after = start_col - end_col,
    )
    .ok();

    write!(
        out,
        "{ep:before$}|{ep:width$}{ep:-^after$}| {}",
        error,
        ep = "",
        before = end_col - 1,
        width = width - 1,
        after = start_col - end_col,
    )
    .ok();

    String::from_utf8_lossy(out.get_ref()).to_string()
}

impl Error {
    pub fn new(template: impl AsRef<str>, error: syn::Error) -> Error {
        let template = template.as_ref();
        let span = error.span();
        let LineColumn {
            line: start_line,
            column: start_col,
        } = span.start();
        let LineColumn {
            line: end_line,
            column: end_col,
        } = span.end();

        if start_line == end_line && end_col > start_col {
            let template_lines = template.lines().take(start_line).collect::<Vec<&'_ str>>();
            let msg = format!(
                "{}\n{ep:spacing$}{ep:^^width$} {}",
                template_lines.join("\n"),
                error,
                ep = "",
                spacing = start_col,
                width = end_col - start_col
            );
            Error(msg)
        } else {
            let msg =
                make_multi_line_error(start_line, start_col, end_line, end_col, template, error);
            Error(msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::StaticStGroup;

    fn error_message(template: &'static str) -> String {
        match StaticStGroup::parse_str(template) {
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
