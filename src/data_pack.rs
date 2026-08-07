//! Versioned external campaign-data pack discovery and validation.

use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;

use crate::character_wizard_domain::PackSpecies;

pub const MANIFEST_FILE: &str = "data-pack.json";
const FORMAT_VERSION: u8 = 1;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DataPackManifest {
    pub format_version: u8,
    pub id: String,
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub files: BTreeMap<ContentFamily, PathBuf>,
    #[serde(skip)]
    pub species: Vec<PackSpecies>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ContentFamily {
    Species,
    Backgrounds,
    Equipment,
    Spells,
}

impl ContentFamily {
    const fn label(self) -> &'static str {
        match self {
            Self::Species => "species",
            Self::Backgrounds => "backgrounds",
            Self::Equipment => "equipment",
            Self::Spells => "spells",
        }
    }
}

/// Load and validate a version-1 external data pack.
///
/// Content files are currently validated as JSON arrays only. They are not yet
/// mechanically active; future content-family slices will add their schemas and
/// catalog integration.
///
/// # Errors
///
/// Returns an error when the directory, manifest, format, or declared content
/// file is invalid.
pub fn load(directory: &Path) -> Result<DataPackManifest, String> {
    if !directory.is_dir() {
        return Err(format!(
            "data pack is not a directory: {}",
            directory.display()
        ));
    }
    let manifest_path = directory.join(MANIFEST_FILE);
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("unable to read {}: {error}", manifest_path.display()))?;
    let mut manifest: DataPackManifest = serde_json::from_str(&source).map_err(|error| {
        format!(
            "invalid data pack manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    validate_manifest(&manifest, directory)?;
    if let Some(relative) = manifest.files.get(&ContentFamily::Species) {
        let path = directory.join(relative);
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("unable to read species data: {error}"))?;
        manifest.species = serde_json::from_str(&source)
            .map_err(|error| format!("invalid species data {}: {error}", path.display()))?;
        validate_species(&manifest.species)?;
    }
    Ok(manifest)
}

fn validate_species(species: &[PackSpecies]) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    for rule in species {
        if !is_identifier(&rule.id) {
            return Err(format!("invalid pack species id: {}", rule.id));
        }
        if !ids.insert(&rule.id) {
            return Err(format!("duplicate pack species id: {}", rule.id));
        }
        if crate::character_wizard_srd_data::SPECIES_NAMES
            .iter()
            .any(|name| {
                name.eq_ignore_ascii_case(&rule.id) || name.eq_ignore_ascii_case(&rule.name)
            })
        {
            return Err(format!(
                "pack species conflicts with SRD species: {}",
                rule.id
            ));
        }
        if rule.name.trim().is_empty() || rule.sizes.is_empty() || !(5..=120).contains(&rule.speed)
        {
            return Err(format!(
                "invalid basic mechanics for pack species: {}",
                rule.id
            ));
        }
        if rule
            .traits
            .iter()
            .any(|trait_name| trait_name.trim().is_empty())
        {
            return Err(format!(
                "pack species traits must not be empty: {}",
                rule.id
            ));
        }
    }
    Ok(())
}

fn validate_manifest(manifest: &DataPackManifest, directory: &Path) -> Result<(), String> {
    if manifest.format_version != FORMAT_VERSION {
        return Err(format!(
            "unsupported data pack format version {}; expected {FORMAT_VERSION}",
            manifest.format_version
        ));
    }
    if !is_identifier(&manifest.id) {
        return Err("data pack id must use lowercase letters, digits, and hyphens".to_owned());
    }
    if manifest.version == 0 {
        return Err("data pack version must be at least 1".to_owned());
    }
    if manifest.name.trim().is_empty() {
        return Err("data pack name must not be empty".to_owned());
    }
    for (family, relative) in &manifest.files {
        if !is_safe_relative_path(relative) {
            return Err(format!(
                "data pack {} file must be a relative path inside the pack: {}",
                family.label(),
                relative.display()
            ));
        }
        let path = directory.join(relative);
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("unable to read {} data: {error}", family.label()))?;
        let value: serde_json::Value = serde_json::from_str(&source).map_err(|error| {
            format!(
                "invalid {} data {}: {error}",
                family.label(),
                path.display()
            )
        })?;
        if !value.is_array() {
            return Err(format!(
                "{} data {} must be a JSON array",
                family.label(),
                path.display()
            ));
        }
    }
    Ok(())
}

fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_safe_relative_path(path: &Path) -> bool {
    path.components()
        .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::load;

    #[test]
    fn loads_a_versioned_pack_with_declared_json_content() {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "character-wizard-data-pack-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).expect("create pack");
        std::fs::write(
            directory.join("data-pack.json"),
            r#"{"format_version":1,"id":"my-campaign","version":1,"name":"My Campaign","files":{"species":"species.json"}}"#,
        )
        .expect("write manifest");
        std::fs::write(
            directory.join("species.json"),
            r#"[{"id":"moonfolk","name":"Moonfolk","sizes":["Small"],"speed":35,"traits":["Moonlit Step"]}]"#,
        )
        .expect("write content");

        let manifest = load(&directory).expect("load pack");
        std::fs::remove_dir_all(&directory).expect("remove pack");
        assert_eq!(manifest.id, "my-campaign");
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.species[0].id, "moonfolk");
    }

    #[test]
    fn rejects_a_manifest_that_escapes_the_pack_directory() {
        let manifest: super::DataPackManifest = serde_json::from_str(
            r#"{"format_version":1,"id":"my-campaign","version":1,"name":"My Campaign","files":{"species":"../species.json"}}"#,
        )
        .expect("parse manifest");
        assert!(
            super::validate_manifest(&manifest, std::path::Path::new("."))
                .expect_err("unsafe path")
                .contains("relative path")
        );
    }
}
