#[cfg(procmacro2_semver_exempt)]
use std::io::{Cursor, Write};

use failure::Fail;

use pest::error::Error as PestError;

use proc_macro2::Span;

#[cfg(procmacro2_semver_exempt)]
use proc_macro2::LineColumn;

use crate::parse::pest::Rule;

#[cfg(procmacro2_semver_exempt)]
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

#[cfg(procmacro2_semver_exempt)]
fn make_error(template: impl AsRef<str>, error: syn::Error) -> Error {
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
        Error::Formatted(msg)
    } else {
        let msg = make_multi_line_error(start_line, start_col, end_line, end_col, template, error);
        Error::Formatted(msg)
    }
}

#[cfg(not(procmacro2_semver_exempt))]
fn make_error(_template: impl AsRef<str>, error: syn::Error) -> Error {
    Error::Syn(error)
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    Formatted(String),

    #[fail(display = "{}", _0)]
    Pest(PestError<Rule>),

    #[fail(display = "{}", _0)]
    Syn(syn::Error),
}

impl Error {
    pub fn syn(template: impl AsRef<str>, error: syn::Error) -> Error {
        make_error(template, error)
    }
}

impl From<PestError<Rule>> for Error {
    fn from(error: PestError<Rule>) -> Error {
        Error::Pest(error)
    }
}

impl From<syn::Error> for Error {
    fn from(error: syn::Error) -> Error {
        Error::Syn(error)
    }
}

impl From<Error> for syn::Error {
    fn from(error: Error) -> syn::Error {
        match error {
            Error::Syn(error) => error,
            Error::Pest(error) => syn::Error::new(Span::call_site(), error),
            Error::Formatted(error) => panic!("{}", error),
        }
    }
}
