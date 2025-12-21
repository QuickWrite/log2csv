use codespan_reporting::diagnostic::{Diagnostic, Label};

#[derive(Debug)]
pub enum ErrorType {
    UnknownKey,
    InvalidSyntax,
}

#[derive(Debug)]
pub struct ParseError {
    pub file: usize,
    pub span: std::ops::Range<usize>,
    pub error_type: ErrorType,
}

impl ParseError {
    pub fn to_diagnostic(&self) -> Diagnostic<usize> {
        let message = match self.error_type {
            ErrorType::UnknownKey => "unknown key",
            ErrorType::InvalidSyntax => "invalid syntax",
        };

        Diagnostic::error().with_message(message).with_labels(vec![
            Label::primary(self.file, self.span.clone()).with_message(message),
        ])
    }
}
