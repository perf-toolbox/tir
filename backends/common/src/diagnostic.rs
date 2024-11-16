use lpl::{Diagnostic, DiagnosticLike, Span};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiagKind {
    #[error("Expected an assembly directive")]
    ExpectedDirective(Span),
    #[error("Expected an assembly identifier")]
    ExpectedIdent(Span),
    #[error("Expected an assembly label")]
    ExpectedLabel(Span),
    #[error("Expected a '{0}' directive")]
    ExpectedSpecificDirective(&'static str, Span),
    #[error("Unexpected token")]
    UnexpectedToken(Span),
    #[error("End of stream")]
    EndOfStream,
}

impl DiagnosticLike for DiagKind {
    fn kind(&self) -> lpl::DiagnosticKind {
        lpl::DiagnosticKind::Error
    }

    fn span(&self) -> Span {
        match self {
            DiagKind::ExpectedDirective(span) => span.clone(),
            DiagKind::ExpectedIdent(span) => span.clone(),
            DiagKind::ExpectedLabel(span) => span.clone(),
            DiagKind::ExpectedSpecificDirective(_, span) => span.clone(),
            DiagKind::UnexpectedToken(span) => span.clone(),
            DiagKind::EndOfStream => Span::empty(),
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
