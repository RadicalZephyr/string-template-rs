use pest::error::Error as PestError;

use proc_macro2::Span;

use crate::parse::pest::Rule;

pub enum Error {
    Pest(PestError<Rule>),
    Syn(syn::Error),
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
        }
    }
}
