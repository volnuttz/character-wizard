use std::collections::BTreeSet;

use inquire::{MultiSelect, Select, Text};

use crate::creation::{Result, WizardError, creation_workflow::choice_description};

/// Terminal-independent prompt operations required by the creation workflow.
///
/// The current terminal adapter is [`TerminalPromptPort`]. Keeping this contract
/// separate lets a scripted or graphical adapter be introduced without changing
/// the workflow's choice semantics.
pub trait PromptPort {
    /// Collect required text input.
    ///
    /// # Errors
    ///
    /// Returns an error when input is cancelled.
    fn prompt(&self, label: &str) -> Result<String>;

    /// Collect optional text input.
    ///
    /// # Errors
    ///
    /// Returns an error when input is cancelled.
    fn optional_prompt(&self, label: &str) -> Result<Option<String>>;

    /// Select one value from a closed list.
    ///
    /// # Errors
    ///
    /// Returns an error when selection is cancelled.
    fn choose(&self, label: &str, choices: &[&str]) -> Result<String>;

    /// Select an exact number of values from a closed list.
    ///
    /// # Errors
    ///
    /// Returns an error when selection is cancelled or incomplete.
    fn choose_set(&self, label: &str, choices: &[&str], count: usize) -> Result<BTreeSet<String>>;

    /// Select an exact number of values, optionally with SRD descriptions.
    ///
    /// # Errors
    ///
    /// Returns an error when selection is cancelled or incomplete.
    fn choose_set_with_descriptions(
        &self,
        label: &str,
        choices: &[&str],
        count: usize,
        descriptions: bool,
    ) -> Result<BTreeSet<String>>;
}

/// `inquire`-backed terminal implementation of [`PromptPort`].
#[derive(Debug, Default, Clone, Copy)]
pub struct TerminalPromptPort;

impl PromptPort for TerminalPromptPort {
    fn prompt(&self, label: &str) -> Result<String> {
        prompt(label)
    }

    fn optional_prompt(&self, label: &str) -> Result<Option<String>> {
        optional_prompt(label)
    }

    fn choose(&self, label: &str, choices: &[&str]) -> Result<String> {
        choose(label, choices)
    }

    fn choose_set(&self, label: &str, choices: &[&str], count: usize) -> Result<BTreeSet<String>> {
        choose_set(label, choices, count)
    }

    fn choose_set_with_descriptions(
        &self,
        label: &str,
        choices: &[&str],
        count: usize,
        descriptions: bool,
    ) -> Result<BTreeSet<String>> {
        choose_set_with_descriptions(label, choices, count, descriptions)
    }
}

pub(crate) fn prompt(label: &str) -> Result<String> {
    Text::new(label)
        .with_help_message("Press Esc to return to the previous step.")
        .prompt()
        .map_err(|_| WizardError::Back)
}

pub(crate) fn optional_prompt(label: &str) -> Result<Option<String>> {
    Text::new(&format!("{label} (optional)"))
        .with_help_message("Leave blank to skip; press Esc to return to the previous step.")
        .prompt()
        .map(|value| (!value.is_empty()).then_some(value))
        .map_err(|_| WizardError::Back)
}

pub(crate) fn choose(label: &str, choices: &[&str]) -> Result<String> {
    Select::new(label, options(choices, true))
        .with_help_message("Type to filter; Enter selects; Esc goes back.")
        .prompt()
        .map(|choice| choice.value)
        .map_err(|_| WizardError::Back)
}

pub(crate) fn choose_set(label: &str, choices: &[&str], count: usize) -> Result<BTreeSet<String>> {
    choose_set_with_descriptions(label, choices, count, true)
}

pub(crate) fn choose_set_with_descriptions(
    label: &str,
    choices: &[&str],
    count: usize,
    descriptions: bool,
) -> Result<BTreeSet<String>> {
    let prompt_label = exact_selection_label(label, count);
    let help_message = format!(
        "Select exactly {count}; Space toggles, Enter confirms, type filters, Esc goes back."
    );
    loop {
        let selected = MultiSelect::new(&prompt_label, options(choices, descriptions))
            .with_help_message(&help_message)
            .prompt()
            .map_err(|_| WizardError::Back)?;
        if selected.len() == count {
            return Ok(selected.into_iter().map(|choice| choice.value).collect());
        }
        println!(
            "Select exactly {count} option(s); you selected {}.",
            selected.len()
        );
    }
}

fn exact_selection_label(label: &str, count: usize) -> String {
    let noun = if count == 1 { "option" } else { "options" };
    format!("{label} (select exactly {count} {noun})")
}

#[derive(Clone)]
struct OptionLabel {
    value: String,
    label: String,
}
impl std::fmt::Display for OptionLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.label.fmt(f)
    }
}
fn options(choices: &[&str], descriptions: bool) -> Vec<OptionLabel> {
    choices
        .iter()
        .map(|choice| OptionLabel {
            value: (*choice).to_owned(),
            label: if descriptions {
                choice_description(choice).map_or_else(
                    || (*choice).to_owned(),
                    |description| format!("{choice} — {description}"),
                )
            } else {
                (*choice).to_owned()
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::exact_selection_label;

    #[test]
    fn exact_count_is_visible_in_multi_select_labels() {
        assert_eq!(
            exact_selection_label("Class skills", 2),
            "Class skills (select exactly 2 options)"
        );
        assert_eq!(
            exact_selection_label("Tool proficiency", 1),
            "Tool proficiency (select exactly 1 option)"
        );
    }
}
