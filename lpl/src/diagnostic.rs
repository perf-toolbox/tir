use core::fmt;

use crate::Span;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InternalError {
    #[error("Expected '{0}' not found")]
    ExpectedNotFound(&'static str, Span),
    #[error("Expected '{0}' not found")]
    OwnedExpectedNotFound(String, Span),
    #[error("Predicate not satisfied")]
    PredNotSatisfied(Span),
    #[error("No items could be parsed")]
    EmptyList(Span),
    #[error("Expected string-like input")]
    NotStringLike(Span),
    #[error("Expected string to start with '{0}'")]
    UnexpectedPrefix(&'static str, Span),
    #[error("Unexpected end of input")]
    UnexpectedEof(Span),
    #[error("Expected end of input")]
    ExpectedEof(Span),
    #[error("Pair parsers not matched")]
    UnmatchedPair(Span),
    #[error("Expected an alphabetic character")]
    NotAlpha(Span),
}

pub enum DiagnosticKind {
    Error,
    Warning,
}

pub trait DiagnosticLike: fmt::Debug {
    fn span(&self) -> Span;
    fn err_code(&self) -> Option<u32>;
    fn kind(&self) -> DiagnosticKind;
    fn message(&self) -> String;
}

pub enum Diagnostic {
    Internal(InternalError),
    External(Box<dyn DiagnosticLike>),
}

impl InternalError {
    pub fn span(&self) -> &Span {
        match self {
            InternalError::ExpectedNotFound(_, span) => span,
            InternalError::OwnedExpectedNotFound(_, span) => span,
            InternalError::PredNotSatisfied(span) => span,
            InternalError::EmptyList(span) => span,
            InternalError::NotStringLike(span) => span,
            InternalError::UnexpectedPrefix(_, span) => span,
            InternalError::UnexpectedEof(span) => span,
            InternalError::ExpectedEof(span) => span,
            InternalError::UnmatchedPair(span) => span,
            InternalError::NotAlpha(span) => span,
        }
    }
}

impl Diagnostic {
    pub fn span(&self) -> Span {
        match self {
            Diagnostic::Internal(internal) => internal.span().clone(),
            Diagnostic::External(external) => external.span(),
        }
    }

    pub fn err_code(&self) -> Option<u32> {
        match self {
            Diagnostic::Internal(_) => None,
            Diagnostic::External(d) => d.err_code(),
        }
    }

    pub fn kind(&self) -> DiagnosticKind {
        match self {
            Diagnostic::Internal(_) => DiagnosticKind::Error,
            Diagnostic::External(d) => d.kind(),
        }
    }

    pub fn message(&self) -> String {
        match self {
            Diagnostic::Internal(d) => d.to_string(),
            Diagnostic::External(d) => d.message(),
        }
    }
}

impl fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Diagnostic::Internal(d) => d.fmt(f),
            Diagnostic::External(d) => fmt::Debug::fmt(d, f),
        }
    }
}

impl From<InternalError> for Diagnostic {
    fn from(value: InternalError) -> Self {
        Diagnostic::Internal(value)
    }
}
