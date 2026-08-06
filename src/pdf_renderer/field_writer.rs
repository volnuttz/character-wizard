//! `AcroForm` field mutation and proof read-back.

use std::{collections::BTreeMap, path::Path};

use lopdf::{Document, Object, ObjectId, dictionary};

use crate::pdf_renderer::{
    pdf_renderer_appearance::text_appearance, pdf_renderer_template_inventory::field_index,
};

type Result<T> = std::result::Result<T, String>;

/// Render a precomputed mapping of supported template field names to values.
///
/// Button values use `"/Off"` for unchecked and any other value for checked;
/// the renderer obtains the actual on-state from the widget appearance stream.
///
/// # Errors
///
/// Returns an error when the template is unreadable, a named field is absent, a
/// field has an unsupported type, or the output cannot be written.
pub fn render_fields(
    template: impl AsRef<Path>,
    output: impl AsRef<Path>,
    values: &BTreeMap<String, String>,
) -> Result<()> {
    let mut document = Document::load(template).map_err(|error| error.to_string())?;
    let index = field_index(&document)?;
    for (name, value) in values {
        let (widget, target) = index
            .get(name)
            .copied()
            .ok_or_else(|| format!("field not found: {name}"))?;
        let kind = field_kind(&document, widget, target)?;
        match kind.as_str() {
            "/Tx" => set_text(&mut document, widget, target, value)?,
            "/Btn" => set_checkbox(&mut document, widget, target, value != "/Off")?,
            _ => return Err(format!("unsupported field type for {name}: {kind}")),
        }
    }
    document.save(output).map_err(|error| error.to_string())?;
    Ok(())
}

/// Read the stored field value after reopening a proof output.
///
/// # Errors
///
/// Returns an error when the PDF cannot be read, the field is absent, or it has
/// no stored value.
pub fn read_field_value(path: impl AsRef<Path>, field_name: &str) -> Result<Object> {
    let document = Document::load(path).map_err(|error| error.to_string())?;
    let (_, target) = field_index(&document)?
        .get(field_name)
        .copied()
        .ok_or_else(|| format!("field not found: {field_name}"))?;
    document
        .get_object(target)
        .and_then(Object::as_dict)
        .map_err(|error| error.to_string())?
        .get(b"V")
        .cloned()
        .map_err(|error| error.to_string())
}

/// Read a set of stored field values with one template traversal.
///
/// # Errors
///
/// Returns an error when the PDF cannot be read or any requested field is absent.
pub fn read_field_values(
    path: impl AsRef<Path>,
    field_names: impl IntoIterator<Item = String>,
) -> Result<BTreeMap<String, Object>> {
    let document = Document::load(path).map_err(|error| error.to_string())?;
    let index = field_index(&document)?;
    field_names
        .into_iter()
        .map(|name| {
            let (_, target) = index
                .get(&name)
                .copied()
                .ok_or_else(|| format!("field not found: {name}"))?;
            let value = document
                .get_object(target)
                .and_then(Object::as_dict)
                .map_err(|error| error.to_string())?
                .get(b"V")
                .cloned()
                .map_err(|error| format!("field {name}: {error}"))?;
            Ok((name, value))
        })
        .collect()
}

fn set_text(
    document: &mut Document,
    widget: ObjectId,
    target: ObjectId,
    value: &str,
) -> Result<()> {
    let appearance = text_appearance(document, widget, target, value)?;
    let field = document
        .get_object_mut(target)
        .and_then(Object::as_dict_mut)
        .map_err(|error| error.to_string())?;
    field.set("V", Object::string_literal(value));
    field.set("DA", Object::string_literal("/Helv 0 Tf 0 g"));
    let widget = document
        .get_object_mut(widget)
        .and_then(Object::as_dict_mut)
        .map_err(|error| error.to_string())?;
    widget.set("DA", Object::string_literal("/Helv 0 Tf 0 g"));
    widget.set(
        "AP",
        Object::Dictionary(dictionary! { "N" => Object::Reference(appearance) }),
    );
    Ok(())
}
fn set_checkbox(
    document: &mut Document,
    widget: ObjectId,
    target: ObjectId,
    checked: bool,
) -> Result<()> {
    let state = if checked {
        checkbox_on_state(document, widget)?
    } else {
        b"Off".to_vec()
    };
    document
        .get_object_mut(target)
        .and_then(Object::as_dict_mut)
        .map_err(|error| error.to_string())?
        .set("V", Object::Name(state.clone()));
    document
        .get_object_mut(widget)
        .and_then(Object::as_dict_mut)
        .map_err(|error| error.to_string())?
        .set("AS", Object::Name(state));
    Ok(())
}

fn field_kind(document: &Document, widget: ObjectId, target: ObjectId) -> Result<String> {
    let target = document
        .get_object(target)
        .and_then(Object::as_dict)
        .map_err(|error| error.to_string())?;
    let widget = document
        .get_object(widget)
        .and_then(Object::as_dict)
        .map_err(|error| error.to_string())?;
    target
        .get(b"FT")
        .or_else(|_| widget.get(b"FT"))
        .and_then(Object::as_name)
        .map(|value| format!("/{}", String::from_utf8_lossy(value)))
        .map_err(|error| error.to_string())
}

fn checkbox_on_state(document: &Document, widget: ObjectId) -> Result<Vec<u8>> {
    let widget = document
        .get_object(widget)
        .and_then(Object::as_dict)
        .map_err(|error| error.to_string())?;
    let appearance = widget
        .get(b"AP")
        .and_then(Object::as_dict)
        .map_err(|error| error.to_string())?;
    let normal = appearance
        .get(b"N")
        .and_then(Object::as_dict)
        .map_err(|error| error.to_string())?;
    normal
        .iter()
        .find_map(|(name, _)| (name.as_slice() != b"Off").then(|| name.clone()))
        .ok_or_else(|| "checkbox has no on-state appearance".to_owned())
}
