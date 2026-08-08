//! Interactive creation state, checkpoint persistence, and terminal workflow.

#[path = "error.rs"]
mod creation_error;
#[path = "prompts.rs"]
mod creation_prompts;
#[path = "review.rs"]
mod creation_review;
#[path = "workflow.rs"]
mod creation_workflow;

pub use creation_error::WizardError;
#[cfg(test)]
pub(crate) use creation_prompts::PromptPort;
pub use creation_workflow::{
    generate_random_character_with_rules, run_edit_interactive_with_rules,
    run_interactive_with_rules, run_quick_interactive_with_rules,
};

pub(crate) type Result<T> = std::result::Result<T, WizardError>;
