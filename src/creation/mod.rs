//! Interactive creation state, checkpoint persistence, and terminal workflow.

#[path = "error.rs"]
mod creation_error;
#[path = "prompts.rs"]
mod creation_prompts;
#[path = "workflow.rs"]
mod creation_workflow;

pub use creation_error::WizardError;
pub use creation_prompts::{PromptPort, TerminalPromptPort};
pub use creation_workflow::{
    BuildDraft, CharacterDraft, DetailsDraft, OriginDraft, run_interactive, run_interactive_with,
};

pub(crate) type Result<T> = std::result::Result<T, WizardError>;
