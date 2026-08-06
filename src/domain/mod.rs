//! Canonical character domain model and SRD-derived values.

#[path = "ids.rs"]
mod domain_ids;
#[path = "model.rs"]
mod domain_model;
#[path = "sheet.rs"]
mod domain_sheet;

pub use domain_ids::{BackgroundId, ClassId, Size, SpeciesId};
pub use domain_model::*;
pub use domain_sheet::{CharacterSheet, SavingThrow};
