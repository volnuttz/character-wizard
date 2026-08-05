//! Official character-sheet rendering boundary.

mod appearance;
mod field_writer;
mod projection;
mod renderer;
mod template_inventory;

pub use field_writer::{read_field_value, read_field_values, render_fields};
pub use renderer::*;
pub use template_inventory::{TemplateInventory, acroform_inventory, template_inventory};
