//! Supported-template inventory and field-tree traversal.

use std::{collections::BTreeMap, fmt::Write as _, path::Path};

use lopdf::{Document, Object, ObjectId};
use sha2::{Digest, Sha256};

type Result<T> = std::result::Result<T, String>;

/// Compact supported-template identity used by the Rust proof tests.
#[derive(Debug, PartialEq, Eq)]
pub struct TemplateInventory {
    pub page_count: usize,
    pub field_count: usize,
    pub text_field_count: usize,
    pub button_field_count: usize,
    pub untyped_field_count: usize,
    pub sha256: String,
}

/// Enumerate every `AcroForm` field exposed by the page widgets.
///
/// # Errors
///
/// Returns an error when the PDF cannot be read or a page/widget is malformed.
pub fn template_inventory(template: impl AsRef<Path>) -> Result<TemplateInventory> {
    let document = Document::load(template).map_err(|error| error.to_string())?;
    let entries = field_entries(&document)?;
    let mut hasher = Sha256::new();
    let mut text_field_count = 0;
    let mut button_field_count = 0;
    let mut untyped_field_count = 0;
    for (name, kind) in &entries {
        hasher.update(name.as_bytes());
        hasher.update(b"\t");
        hasher.update(kind.as_bytes());
        hasher.update(b"\n");
        match kind.as_str() {
            "/Tx" => text_field_count += 1,
            "/Btn" => button_field_count += 1,
            _ => untyped_field_count += 1,
        }
    }
    let mut sha256 = String::new();
    for byte in hasher.finalize() {
        write!(&mut sha256, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Ok(TemplateInventory {
        page_count: document.get_pages().len(),
        field_count: entries.len(),
        text_field_count,
        button_field_count,
        untyped_field_count,
        sha256,
    })
}

/// Enumerate the root `AcroForm` field tree, including non-widget hierarchy entries.
///
/// # Errors
///
/// Returns an error when the document lacks a readable `AcroForm` field tree.
pub fn acroform_inventory(template: impl AsRef<Path>) -> Result<TemplateInventory> {
    let document = Document::load(template).map_err(|error| error.to_string())?;
    let (_, catalog) = document
        .dereference(
            document
                .trailer
                .get(b"Root")
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let catalog = catalog.as_dict().map_err(|error| error.to_string())?;
    let (_, form) = document
        .dereference(
            catalog
                .get(b"AcroForm")
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let form = form.as_dict().map_err(|error| error.to_string())?;
    let (_, roots) = document
        .dereference(form.get(b"Fields").map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    let mut entries = Vec::new();
    for field in roots.as_array().map_err(|error| error.to_string())? {
        collect_field_entries(&document, field, "", &mut entries)?;
    }
    Ok(inventory_from_entries(document.get_pages().len(), entries))
}
pub(super) fn field_index(document: &Document) -> Result<BTreeMap<String, (ObjectId, ObjectId)>> {
    let paths = acroform_path_index(document)?;
    let mut index = BTreeMap::new();
    for (widget, target, name, _) in field_entries_with_ids(document)? {
        let path = paths.get(&target).cloned().unwrap_or(name.clone());
        let candidate = if name == path
            || name.is_empty()
            || path
                .rsplit('.')
                .next()
                .is_some_and(|segment| segment == name)
        {
            path
        } else {
            format!("{path}.{name}")
        };
        index.insert(candidate, (widget, target));
        index.insert(name, (widget, target));
    }
    Ok(index)
}

fn acroform_path_index(document: &Document) -> Result<BTreeMap<ObjectId, String>> {
    let (_, catalog) = document
        .dereference(
            document
                .trailer
                .get(b"Root")
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let catalog = catalog.as_dict().map_err(|error| error.to_string())?;
    let (_, form) = document
        .dereference(
            catalog
                .get(b"AcroForm")
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let form = form.as_dict().map_err(|error| error.to_string())?;
    let (_, roots) = document
        .dereference(form.get(b"Fields").map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    let mut index = BTreeMap::new();
    for field in roots.as_array().map_err(|error| error.to_string())? {
        collect_field_paths(document, field, "", &mut index)?;
    }
    Ok(index)
}

fn collect_field_paths(
    document: &Document,
    field: &Object,
    parent_name: &str,
    index: &mut BTreeMap<ObjectId, String>,
) -> Result<()> {
    let (id, field) = document
        .dereference(field)
        .map_err(|error| error.to_string())?;
    let field = field.as_dict().map_err(|error| error.to_string())?;
    let full_name = if let Ok(name) = field.get(b"T").and_then(Object::as_str) {
        let name = String::from_utf8_lossy(name);
        if parent_name.is_empty() {
            name.into_owned()
        } else {
            format!("{parent_name}.{name}")
        }
    } else {
        parent_name.to_owned()
    };
    if let Some(id) = id {
        index.insert(id, full_name.clone());
    }
    if let Ok(kids) = field.get(b"Kids") {
        let (_, kids) = document
            .dereference(kids)
            .map_err(|error| error.to_string())?;
        for child in kids.as_array().map_err(|error| error.to_string())? {
            collect_field_paths(document, child, &full_name, index)?;
        }
    }
    Ok(())
}

fn field_entries(document: &Document) -> Result<Vec<(String, String)>> {
    let mut entries = field_entries_with_ids(document)?
        .into_iter()
        .map(|(_, target, name, kind)| {
            let target_name = document
                .get_object(target)
                .and_then(Object::as_dict)
                .ok()
                .and_then(|field| field.get(b"T").ok())
                .and_then(|value| value.as_str().ok())
                .map_or(name, |value| String::from_utf8_lossy(value).into_owned());
            (target_name, kind)
        })
        .collect::<Vec<_>>();
    entries.sort_unstable();
    entries.dedup();
    Ok(entries)
}

fn inventory_from_entries(
    page_count: usize,
    mut entries: Vec<(String, String)>,
) -> TemplateInventory {
    entries.sort_unstable();
    entries.dedup();
    let mut hasher = Sha256::new();
    let mut text_field_count = 0;
    let mut button_field_count = 0;
    let mut untyped_field_count = 0;
    for (name, kind) in &entries {
        hasher.update(name.as_bytes());
        hasher.update(b"\t");
        hasher.update(kind.as_bytes());
        hasher.update(b"\n");
        match kind.as_str() {
            "/Tx" => text_field_count += 1,
            "/Btn" => button_field_count += 1,
            _ => untyped_field_count += 1,
        }
    }
    let mut sha256 = String::new();
    for byte in hasher.finalize() {
        write!(&mut sha256, "{byte:02x}").expect("writing to a String cannot fail");
    }
    TemplateInventory {
        page_count,
        field_count: entries.len(),
        text_field_count,
        button_field_count,
        untyped_field_count,
        sha256,
    }
}

fn collect_field_entries(
    document: &Document,
    field: &Object,
    parent_name: &str,
    entries: &mut Vec<(String, String)>,
) -> Result<()> {
    let (_, field) = document
        .dereference(field)
        .map_err(|error| error.to_string())?;
    let field = field.as_dict().map_err(|error| error.to_string())?;
    let full_name = if let Ok(name) = field.get(b"T").and_then(Object::as_str) {
        let name = String::from_utf8_lossy(name);
        let full_name = if parent_name.is_empty() {
            name.into_owned()
        } else {
            format!("{parent_name}.{name}")
        };
        let kind = field
            .get(b"FT")
            .ok()
            .and_then(|value| value.as_name().ok())
            .map_or_else(
                || "None".to_owned(),
                |value| format!("/{}", String::from_utf8_lossy(value)),
            );
        entries.push((full_name.clone(), kind));
        full_name
    } else {
        parent_name.to_owned()
    };
    if let Ok(kids) = field.get(b"Kids") {
        let (_, kids) = document
            .dereference(kids)
            .map_err(|error| error.to_string())?;
        for child in kids.as_array().map_err(|error| error.to_string())? {
            collect_field_entries(document, child, &full_name, entries)?;
        }
    }
    Ok(())
}

fn field_entries_with_ids(
    document: &Document,
) -> Result<Vec<(ObjectId, ObjectId, String, String)>> {
    let mut entries = Vec::new();
    for page_id in document.get_pages().into_values() {
        let page = document
            .get_object(page_id)
            .and_then(Object::as_dict)
            .map_err(|error| error.to_string())?;
        let (_, annotations) = document
            .dereference(page.get(b"Annots").map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
        let annotations = annotations.as_array().map_err(|error| error.to_string())?;
        for annotation in annotations {
            let widget_id = annotation
                .as_reference()
                .map_err(|error| error.to_string())?;
            let widget = document
                .get_object(widget_id)
                .and_then(Object::as_dict)
                .map_err(|error| error.to_string())?;
            let target = if widget.get(b"T").is_ok() {
                widget_id
            } else {
                widget
                    .get(b"Parent")
                    .ok()
                    .and_then(|parent| parent.as_reference().ok())
                    .unwrap_or(widget_id)
            };
            let field = document
                .get_object(target)
                .and_then(Object::as_dict)
                .map_err(|error| error.to_string())?;
            let name = widget
                .get(b"T")
                .or_else(|_| field.get(b"T"))
                .and_then(Object::as_str)
                .map_err(|error| error.to_string())?;
            let kind = field
                .get(b"FT")
                .or_else(|_| widget.get(b"FT"))
                .ok()
                .and_then(|value| value.as_name().ok())
                .map_or_else(
                    || "None".to_owned(),
                    |value| format!("/{}", String::from_utf8_lossy(value)),
                );
            entries.push((
                widget_id,
                target,
                String::from_utf8_lossy(name).into_owned(),
                kind,
            ));
        }
    }
    Ok(entries)
}
