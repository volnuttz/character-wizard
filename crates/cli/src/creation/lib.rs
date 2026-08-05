//! Interactive creation state, checkpoint persistence, and terminal workflow.

mod error;
mod prompts;
mod workflow;

pub use error::WizardError;
pub use prompts::{PromptPort, TerminalPromptPort};
pub use workflow::{
    BuildDraft, CharacterDraft, DetailsDraft, OriginDraft, run_interactive, run_interactive_with,
};

pub(crate) type Result<T> = std::result::Result<T, WizardError>;
