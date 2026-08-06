//! Official character-sheet rendering boundary.

#[path = "appearance.rs"]
mod pdf_renderer_appearance;
#[path = "field_writer.rs"]
mod pdf_renderer_field_writer;
#[path = "projection.rs"]
mod pdf_renderer_projection;
#[path = "renderer.rs"]
mod pdf_renderer_renderer;
#[path = "template_inventory.rs"]
mod pdf_renderer_template_inventory;

pub use pdf_renderer_field_writer::{read_field_value, read_field_values, render_fields};
pub use pdf_renderer_renderer::*;
pub use pdf_renderer_template_inventory::{TemplateInventory, acroform_inventory, template_inventory};
