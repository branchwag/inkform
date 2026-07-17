use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InkformErrorKind {
    InvalidInput,
    LowQualitySample,
    UnsupportedCharacter,
    GenerationFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InkformError {
    kind: InkformErrorKind,
    message: String,
}

impl InkformError {
    #[must_use]
    pub fn new(kind: InkformErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> InkformErrorKind {
        self.kind
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for InkformError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for InkformError {}
