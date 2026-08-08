//! Canonical character model boundary.

#[path = "content.rs"]
mod content;
#[path = "derived.rs"]
mod derived;
#[path = "record.rs"]
mod record;
#[path = "validation.rs"]
mod validation;

pub use content::*;
pub use record::*;
