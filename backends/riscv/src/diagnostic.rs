use lpl::{Diagnostic, DiagnosticLike, Span};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiagKind {
    #[error("Unknown register '{0}'")]
    UnknownRegister(String, Span),
    #[error("Unknown opcode")]
    UnknownOpcode(Span),
}

impl DiagnosticLike for DiagKind {
    fn kind(&self) -> lpl::DiagnosticKind {
        lpl::DiagnosticKind::Error
    }

    fn span(&self) -> Span {
        match self {
            DiagKind::UnknownRegister(_, span) => span.clone(),
            DiagKind::UnknownOpcode(span) => span.clone(),
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
