use std::{error::Error, fmt};

#[derive(Debug, PartialEq, Eq)]
pub enum WizardError {
    Back,
    SaveAndExit,
    Message(String),
}

impl fmt::Display for WizardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Back => formatter.write_str("wizard navigation cancelled"),
            Self::SaveAndExit => formatter.write_str("creation saved for later review"),
            Self::Message(message) => formatter.write_str(message),
        }
    }
}

impl Error for WizardError {}

impl From<String> for WizardError {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}
