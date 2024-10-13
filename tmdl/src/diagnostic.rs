use lpl::{Diagnostic, DiagnosticLike, Span};
use thiserror::Error;

use crate::SyntaxKind;

#[derive(Error, Debug)]
pub enum DiagKind {
    #[error("Unexpected end of input")]
    UnexpectedEof(Span),
    #[error("Expected token '{0}' not found")]
    TokenNotFound(SyntaxKind, Span),
    #[error("Expected tokens not found")]
    MultipleTokensNotFound(Span),
}

impl DiagnosticLike for DiagKind {
    fn kind(&self) -> lpl::DiagnosticKind {
        lpl::DiagnosticKind::Error
    }

    fn span(&self) -> Span {
        match self {
            DiagKind::UnexpectedEof(span) => span.clone(),
            DiagKind::TokenNotFound(_, span) => span.clone(),
            DiagKind::MultipleTokensNotFound(span) => span.clone(),
        }
    }

    fn err_code(&self) -> Option<u32> {
        None
    }

    fn message(&self) -> String {
        self.to_string()
    }
}

impl From<DiagKind> for Diagnostic {
    fn from(value: DiagKind) -> Self {
        Diagnostic::External(Box::new(value))
    }
}
