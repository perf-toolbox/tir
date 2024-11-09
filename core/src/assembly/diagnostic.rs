use lpl::{Diagnostic, DiagnosticLike, Span};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiagKind {
    #[error("Unknown dialect '{0}'")]
    UnknownDialect(String, Span),
    #[error("Unknown operation '{0}' in dialect '{1}'")]
    UnknownOperation(String, String, Span),
}

impl DiagnosticLike for DiagKind {
    fn kind(&self) -> lpl::DiagnosticKind {
        lpl::DiagnosticKind::Error
    }

    fn span(&self) -> Span {
        match self {
            DiagKind::UnknownDialect(_, span) => span.clone(),
            DiagKind::UnknownOperation(_, _, span) => span.clone(),
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
