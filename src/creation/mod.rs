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
    BuildDraft, CharacterDraft, DetailsDraft, OriginDraft, generate_random_character,
    run_edit_interactive, run_edit_interactive_with, run_interactive, run_interactive_with,
    run_quick_interactive,
};

pub(crate) type Result<T> = std::result::Result<T, WizardError>;
