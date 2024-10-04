use crate::Span;

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub span: Span,
}

impl ParserError {
    pub fn new<S: AsRef<str>>(message: S, span: Span) -> Self {
        Self {
            message: message.as_ref().to_string(),
            span,
        }
    }

    pub fn take_message(self) -> String {
        self.message
    }
}
