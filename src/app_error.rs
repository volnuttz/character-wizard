//! Typed application errors and CLI exit-code mapping.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorKind {
    Input,
    Persistence,
    Rules,
    Rendering,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppError {
    kind: ErrorKind,
    message: String,
}

impl AppError {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub(crate) const fn exit_code(&self) -> u8 {
        match self.kind {
            ErrorKind::Input | ErrorKind::Persistence | ErrorKind::Rules | ErrorKind::Rendering => {
                1
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

impl From<(u8, String)> for AppError {
    fn from((_code, message): (u8, String)) -> Self {
        Self::new(ErrorKind::Input, message)
    }
}
