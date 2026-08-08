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

#[cfg(test)]
pub(crate) use pdf_renderer_field_writer::read_field_value;
pub(crate) use pdf_renderer_field_writer::render_fields;
pub use pdf_renderer_renderer::*;
pub(crate) use pdf_renderer_template_inventory::acroform_inventory;
