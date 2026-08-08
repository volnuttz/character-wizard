//! Versioned external campaign-data pack discovery and validation.

use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;

use crate::character_wizard_domain::{
    PackBackground, PackClass, PackEquipment, PackEquipmentKind, PackSpecies, PackSpell,
};

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
    #[serde(skip)]
    pub backgrounds: Vec<PackBackground>,
    #[serde(skip)]
    pub equipment: Vec<PackEquipment>,
    #[serde(skip)]
    pub spells: Vec<PackSpell>,
    #[serde(skip)]
    pub classes: Vec<PackClass>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ContentFamily {
    Species,
    Backgrounds,
    Equipment,
    Spells,
    Classes,
}

impl ContentFamily {
    const fn label(self) -> &'static str {
        match self {
            Self::Species => "species",
            Self::Backgrounds => "backgrounds",
            Self::Equipment => "equipment",
            Self::Spells => "spells",
            Self::Classes => "classes",
        }
    }
}

/// Load and validate a version-1 external data pack.
///
/// Every supported content family uses a typed, mechanically active schema.
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
    if let Some(relative) = manifest.files.get(&ContentFamily::Equipment) {
        let path = directory.join(relative);
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("unable to read equipment data: {error}"))?;
        manifest.equipment = serde_json::from_str(&source)
            .map_err(|error| format!("invalid equipment data {}: {error}", path.display()))?;
        validate_equipment(&manifest.equipment)?;
    }
    if let Some(relative) = manifest.files.get(&ContentFamily::Spells) {
        let path = directory.join(relative);
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("unable to read spells data: {error}"))?;
        manifest.spells = serde_json::from_str(&source)
            .map_err(|error| format!("invalid spells data {}: {error}", path.display()))?;
        validate_spells(&manifest.spells)?;
    }
    if let Some(relative) = manifest.files.get(&ContentFamily::Backgrounds) {
        let path = directory.join(relative);
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("unable to read backgrounds data: {error}"))?;
        manifest.backgrounds = serde_json::from_str(&source)
            .map_err(|error| format!("invalid backgrounds data {}: {error}", path.display()))?;
        validate_backgrounds(&manifest.backgrounds, &manifest.equipment)?;
    }
    if let Some(relative) = manifest.files.get(&ContentFamily::Classes) {
        let path = directory.join(relative);
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("unable to read classes data: {error}"))?;
        manifest.classes = serde_json::from_str(&source)
            .map_err(|error| format!("invalid classes data {}: {error}", path.display()))?;
        validate_classes(&manifest.classes, &manifest.spells)?;
    }
    Ok(manifest)
}

#[allow(clippy::too_many_lines)]
fn validate_classes(classes: &[PackClass], spells: &[PackSpell]) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    let mut names = std::collections::BTreeSet::new();
    for rule in classes {
        if !is_identifier(&rule.id) || !ids.insert(rule.id.as_str()) {
            return Err(format!("invalid or duplicate pack class id: {}", rule.id));
        }
        let normalized_name = rule.name.trim().to_ascii_lowercase();
        if normalized_name.is_empty() || !names.insert(normalized_name) {
            return Err(format!(
                "invalid or duplicate pack class name: {}",
                rule.name
            ));
        }
        if crate::character_wizard_srd_data::CLASS_NAMES
            .iter()
            .any(|name| {
                name.eq_ignore_ascii_case(&rule.id) || name.eq_ignore_ascii_case(&rule.name)
            })
        {
            return Err(format!("pack class conflicts with SRD class: {}", rule.id));
        }
        let saves: std::collections::BTreeSet<&str> =
            rule.saving_throws.iter().map(String::as_str).collect();
        let skills: std::collections::BTreeSet<&str> =
            rule.skills.iter().map(String::as_str).collect();
        let armor: std::collections::BTreeSet<&str> =
            rule.armor_training.iter().map(String::as_str).collect();
        let weapons: std::collections::BTreeSet<&str> =
            rule.weapon_training.iter().map(String::as_str).collect();
        let features: std::collections::BTreeSet<&str> =
            rule.features.iter().map(String::as_str).collect();
        let choice_ids: std::collections::BTreeSet<&str> = rule
            .choices
            .iter()
            .map(|choice| choice.id.as_str())
            .collect();
        let resource_names: std::collections::BTreeSet<String> = rule
            .resources
            .iter()
            .map(|resource| resource.name.trim().to_ascii_lowercase())
            .collect();
        let valid_choices = rule.choices.len() == choice_ids.len()
            && rule.choices.iter().all(|choice| {
                let option_ids: std::collections::BTreeSet<&str> = choice
                    .options
                    .iter()
                    .map(|option| option.id.as_str())
                    .collect();
                let option_names: std::collections::BTreeSet<String> = choice
                    .options
                    .iter()
                    .map(|option| option.name.trim().to_ascii_lowercase())
                    .collect();
                is_identifier(&choice.id)
                    && !choice.label.trim().is_empty()
                    && choice.count > 0
                    && choice.count <= choice.options.len()
                    && choice.options.len() == option_ids.len()
                    && choice.options.len() == option_names.len()
                    && choice.options.iter().all(|option| {
                        is_identifier(&option.id)
                            && !option.name.trim().is_empty()
                            && option
                                .description
                                .as_deref()
                                .is_none_or(|description| !description.trim().is_empty())
                    })
            });
        let valid_resources = rule.resources.len() == resource_names.len()
            && rule.resources.iter().all(|resource| {
                !resource.name.trim().is_empty()
                    && resource.maximum > 0
                    && !resource.unit.trim().is_empty()
                    && !resource.recovery.trim().is_empty()
                    && resource
                        .detail
                        .as_deref()
                        .is_none_or(|detail| !detail.trim().is_empty())
            });
        let valid_spellcasting = rule.spellcasting.as_ref().is_none_or(|spellcasting| {
            let Some(list) =
                crate::character_wizard_srd_data::class_spell_list(&spellcasting.spell_list)
            else {
                return false;
            };
            let pack_cantrips = spells
                .iter()
                .filter(|spell| spell.level == 0 && spell.lists.contains(&spellcasting.spell_list))
                .count();
            let pack_level_one = spells
                .iter()
                .filter(|spell| spell.level == 1 && spell.lists.contains(&spellcasting.spell_list))
                .count();
            crate::character_wizard_srd_data::SPELLCASTING_ABILITIES
                .contains(&spellcasting.ability.as_str())
                && spellcasting.cantrip_count <= list.cantrips.len() + pack_cantrips
                && spellcasting.prepared_spell_count <= list.level_one_spells.len() + pack_level_one
                && spellcasting.spell_slots <= 4
                && !spellcasting.slot_recovery.trim().is_empty()
                && (spellcasting.cantrip_count > 0 || spellcasting.prepared_spell_count > 0)
                && ((spellcasting.prepared_spell_count > 0) == (spellcasting.spell_slots > 0))
        });
        let valid = [6, 8, 10, 12].contains(&rule.hit_die)
            && rule.saving_throws.len() == 2
            && saves.len() == 2
            && rule.saving_throws.iter().all(|ability| {
                crate::character_wizard_srd_data::ABILITIES.contains(&ability.as_str())
            })
            && rule.skill_count > 0
            && rule.skill_count <= rule.skills.len()
            && rule.skills.len() == skills.len()
            && rule
                .skills
                .iter()
                .all(|skill| crate::character_wizard_srd_data::skill_ability(skill).is_some())
            && rule.armor_training.len() == armor.len()
            && rule.armor_training.iter().all(|training| {
                ["Light", "Medium", "Heavy", "Shields"].contains(&training.as_str())
            })
            && rule.weapon_training.len() == weapons.len()
            && rule
                .weapon_training
                .iter()
                .all(|training| ["Simple", "Martial"].contains(&training.as_str()))
            && !rule.equipment.is_empty()
            && rule.equipment.iter().all(|grant| {
                grant.quantity > 0
                    && grant.equipment_id.is_none()
                    && grant
                        .name
                        .as_deref()
                        .is_some_and(|name| !name.trim().is_empty())
            })
            && rule.starting_gold > 0
            && !rule.features.is_empty()
            && rule.features.len() == features.len()
            && rule
                .features
                .iter()
                .all(|feature| !feature.trim().is_empty())
            && rule.weapon_mastery_count <= 3
            && (rule.weapon_mastery_count == 0 || !rule.weapon_training.is_empty())
            && valid_choices
            && valid_resources
            && valid_spellcasting;
        if !valid {
            return Err(format!("invalid pack class mechanics: {}", rule.id));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn validate_backgrounds(
    backgrounds: &[PackBackground],
    equipment: &[PackEquipment],
) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    let mut names = std::collections::BTreeSet::new();
    for rule in backgrounds {
        if !is_identifier(&rule.id) {
            return Err(format!("invalid pack background id: {}", rule.id));
        }
        if !ids.insert(rule.id.as_str()) {
            return Err(format!("duplicate pack background id: {}", rule.id));
        }
        let normalized_name = rule.name.trim().to_ascii_lowercase();
        if normalized_name.is_empty() || !names.insert(normalized_name) {
            return Err(format!(
                "invalid or duplicate pack background name: {}",
                rule.name
            ));
        }
        if crate::character_wizard_srd_data::BACKGROUND_NAMES
            .iter()
            .any(|name| {
                name.eq_ignore_ascii_case(&rule.id) || name.eq_ignore_ascii_case(&rule.name)
            })
        {
            return Err(format!(
                "pack background conflicts with SRD background: {}",
                rule.id
            ));
        }
        let abilities: std::collections::BTreeSet<&str> =
            rule.abilities.iter().map(String::as_str).collect();
        if rule.abilities.len() != 3
            || abilities.len() != 3
            || abilities.iter().any(|ability| {
                ![
                    "strength",
                    "dexterity",
                    "constitution",
                    "intelligence",
                    "wisdom",
                    "charisma",
                ]
                .contains(ability)
            })
        {
            return Err(format!(
                "pack background {} must grant three different abilities",
                rule.id
            ));
        }
        let skills: std::collections::BTreeSet<&str> =
            rule.skills.iter().map(String::as_str).collect();
        if rule.skills.len() != 2
            || skills.len() != 2
            || skills
                .iter()
                .any(|skill| crate::character_wizard_srd_data::skill_ability(skill).is_none())
        {
            return Err(format!(
                "pack background {} must grant two different SRD skills",
                rule.id
            ));
        }
        if !crate::character_wizard_srd_data::ORIGIN_FEATS.contains(&rule.feat.as_str()) {
            return Err(format!(
                "invalid pack background Origin feat: {}",
                rule.feat
            ));
        }
        if !crate::character_wizard_srd_data::is_tool(&rule.tool) {
            return Err(format!("invalid pack background tool: {}", rule.tool));
        }
        let requires_magic_list = rule.feat == "Magic Initiate";
        if requires_magic_list != rule.magic_initiate_list.is_some()
            || rule.magic_initiate_list.as_deref().is_some_and(|list| {
                crate::character_wizard_srd_data::magic_initiate_spell_list(list).is_none()
            })
        {
            return Err(format!(
                "pack background {} must define a valid magic_initiate_list exactly when its feat is Magic Initiate",
                rule.id
            ));
        }
        if rule.equipment.is_empty() {
            return Err(format!(
                "pack background {} must grant non-empty starting equipment",
                rule.id
            ));
        }
        for grant in &rule.equipment {
            if grant.quantity == 0
                || (grant.name.is_some() == grant.equipment_id.is_some())
                || grant
                    .name
                    .as_deref()
                    .is_some_and(|name| name.trim().is_empty())
            {
                return Err(format!(
                    "pack background {} equipment grants must define exactly one non-empty name or equipment_id",
                    rule.id
                ));
            }
            if let Some(id) = &grant.equipment_id
                && !equipment.iter().any(|item| item.id == *id)
            {
                return Err(format!(
                    "pack background {} references unknown pack equipment: {id}",
                    rule.id
                ));
            }
        }
    }
    Ok(())
}

fn validate_equipment(equipment: &[PackEquipment]) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    let mut names = std::collections::BTreeSet::new();
    for item in equipment {
        if !is_identifier(&item.id) || !ids.insert(item.id.as_str()) {
            return Err(format!(
                "invalid or duplicate pack equipment id: {}",
                item.id
            ));
        }
        let normalized_name = item.name.trim().to_ascii_lowercase();
        if normalized_name.is_empty() || !names.insert(normalized_name) {
            return Err(format!(
                "invalid or duplicate pack equipment name: {}",
                item.name
            ));
        }
        if crate::character_wizard_srd_data::weapon_rule(&item.name).is_some()
            || crate::character_wizard_srd_data::armor_rule(&item.name).is_some()
            || item.name.eq_ignore_ascii_case("Shield")
        {
            return Err(format!(
                "pack equipment conflicts with SRD equipment: {}",
                item.id
            ));
        }
        match &item.kind {
            PackEquipmentKind::Gear | PackEquipmentKind::Ammunition => {}
            PackEquipmentKind::Shield { armor_class_bonus } => {
                if !(1..=10).contains(armor_class_bonus) {
                    return Err(format!("invalid pack shield AC bonus: {}", item.id));
                }
            }
            PackEquipmentKind::Armor {
                category,
                base_ac,
                dexterity_cap,
                strength_requirement,
            } => {
                if !["Light", "Medium", "Heavy"].contains(&category.as_str())
                    || !(10..=25).contains(base_ac)
                    || dexterity_cap.is_some_and(|cap| !(0..=10).contains(&cap))
                    || strength_requirement.is_some_and(|score| !(3..=20).contains(&score))
                {
                    return Err(format!("invalid pack armor mechanics: {}", item.id));
                }
            }
            PackEquipmentKind::Weapon {
                category,
                kind,
                properties,
                mastery,
                damage,
                damage_type,
                normal_range,
                long_range,
                versatile_damage,
            } => validate_pack_weapon(
                item,
                category,
                kind,
                properties,
                mastery,
                damage,
                damage_type,
                *normal_range,
                *long_range,
                versatile_damage.as_deref(),
            )?,
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_pack_weapon(
    item: &PackEquipment,
    category: &str,
    kind: &str,
    properties: &[String],
    mastery: &str,
    damage: &str,
    damage_type: &str,
    normal_range: u16,
    long_range: Option<u16>,
    versatile_damage: Option<&str>,
) -> Result<(), String> {
    const PROPERTIES: [&str; 11] = [
        "Ammunition",
        "Finesse",
        "Heavy",
        "Light",
        "Loading",
        "Reach",
        "Thrown",
        "Two-Handed",
        "Versatile",
        "Improvised",
        "Special",
    ];
    const MASTERIES: [&str; 8] = [
        "Cleave", "Graze", "Nick", "Push", "Sap", "Slow", "Topple", "Vex",
    ];
    const DAMAGE_TYPES: [&str; 13] = [
        "Acid",
        "Bludgeoning",
        "Cold",
        "Fire",
        "Force",
        "Lightning",
        "Necrotic",
        "Piercing",
        "Poison",
        "Psychic",
        "Radiant",
        "Slashing",
        "Thunder",
    ];
    let unique_properties: std::collections::BTreeSet<&str> =
        properties.iter().map(String::as_str).collect();
    let valid = ["Simple", "Martial"].contains(&category)
        && ["Melee", "Ranged"].contains(&kind)
        && properties.len() == unique_properties.len()
        && properties
            .iter()
            .all(|property| PROPERTIES.contains(&property.as_str()))
        && MASTERIES.contains(&mastery)
        && valid_damage_dice(damage)
        && DAMAGE_TYPES.contains(&damage_type)
        && normal_range > 0
        && long_range.is_none_or(|long| long >= normal_range)
        && (properties.iter().any(|property| property == "Versatile")
            == versatile_damage.is_some())
        && versatile_damage.is_none_or(valid_damage_dice);
    if valid {
        Ok(())
    } else {
        Err(format!("invalid pack weapon mechanics: {}", item.id))
    }
}

fn valid_damage_dice(value: &str) -> bool {
    let Some((count, sides)) = value.split_once('d') else {
        return false;
    };
    count
        .parse::<u8>()
        .is_ok_and(|count| (1..=10).contains(&count))
        && sides
            .parse::<u16>()
            .is_ok_and(|sides| (2..=100).contains(&sides))
}

fn validate_spells(spells: &[PackSpell]) -> Result<(), String> {
    const SCHOOLS: [&str; 8] = [
        "Abjuration",
        "Conjuration",
        "Divination",
        "Enchantment",
        "Evocation",
        "Illusion",
        "Necromancy",
        "Transmutation",
    ];
    const LISTS: [&str; 8] = [
        "Bard", "Cleric", "Druid", "Paladin", "Ranger", "Sorcerer", "Warlock", "Wizard",
    ];
    let mut ids = std::collections::BTreeSet::new();
    let mut names = std::collections::BTreeSet::new();
    for spell in spells {
        if !is_identifier(&spell.id) || !ids.insert(spell.id.as_str()) {
            return Err(format!("invalid or duplicate pack spell id: {}", spell.id));
        }
        let normalized_name = spell.name.trim().to_ascii_lowercase();
        if normalized_name.is_empty() || !names.insert(normalized_name) {
            return Err(format!(
                "invalid or duplicate pack spell name: {}",
                spell.name
            ));
        }
        let conflicts_with_srd = LISTS
            .iter()
            .filter_map(|list| crate::character_wizard_srd_data::class_spell_list(list))
            .flat_map(|list| list.cantrips.iter().chain(list.level_one_spells.iter()))
            .any(|name| name.eq_ignore_ascii_case(&spell.name));
        if conflicts_with_srd {
            return Err(format!("pack spell conflicts with SRD spell: {}", spell.id));
        }
        let lists: std::collections::BTreeSet<&str> =
            spell.lists.iter().map(String::as_str).collect();
        let components: std::collections::BTreeSet<&str> =
            spell.components.iter().map(String::as_str).collect();
        let tags: std::collections::BTreeSet<&str> =
            spell.tags.iter().map(String::as_str).collect();
        let has_material = components.contains("M");
        let valid = spell.level <= 1
            && SCHOOLS.contains(&spell.school.as_str())
            && !spell.lists.is_empty()
            && spell.lists.len() == lists.len()
            && spell
                .lists
                .iter()
                .all(|list| LISTS.contains(&list.as_str()))
            && !spell.casting_time.trim().is_empty()
            && !spell.range.trim().is_empty()
            && !spell.notes.trim().is_empty()
            && !spell.components.is_empty()
            && spell.components.len() == components.len()
            && spell
                .components
                .iter()
                .all(|component| ["V", "S", "M"].contains(&component.as_str()))
            && (has_material == spell.material.is_some())
            && spell
                .material
                .as_deref()
                .is_none_or(|material| !material.trim().is_empty())
            && (!spell.ritual || spell.level == 1)
            && spell.tags.len() == tags.len()
            && spell.tags.iter().all(|tag| !tag.trim().is_empty());
        if !valid {
            return Err(format!("invalid pack spell mechanics: {}", spell.id));
        }
    }
    Ok(())
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

    use super::{
        load, validate_backgrounds, validate_classes, validate_equipment, validate_spells,
    };
    use crate::character_wizard_domain::{PackBackground, PackClass, PackEquipment, PackSpell};

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
            r#"{"format_version":1,"id":"my-campaign","version":1,"name":"My Campaign","files":{"species":"species.json","backgrounds":"backgrounds.json","equipment":"equipment.json","spells":"spells.json","classes":"classes.json"}}"#,
        )
        .expect("write manifest");
        std::fs::write(
            directory.join("species.json"),
            r#"[{"id":"moonfolk","name":"Moonfolk","sizes":["Small"],"speed":35,"traits":["Moonlit Step"]}]"#,
        )
        .expect("write content");
        std::fs::write(
            directory.join("backgrounds.json"),
            r#"[{"id":"lunar-scout","name":"Lunar Scout","abilities":["dexterity","wisdom","charisma"],"skills":["Perception","Survival"],"feat":"Alert","tool":"Navigator's Tools","equipment":[{"equipment_id":"moonblade"}],"equipment_gold":12,"gold_alternative":50}]"#,
        )
        .expect("write backgrounds");
        std::fs::write(
            directory.join("equipment.json"),
            r#"[{"id":"moonblade","name":"Moonblade","kind":{"type":"weapon","category":"Simple","kind":"Melee","properties":["Finesse","Light"],"mastery":"Vex","damage":"1d8","damage_type":"Radiant","normal_range":5}}]"#,
        )
        .expect("write equipment");
        std::fs::write(
            directory.join("spells.json"),
            r#"[{"id":"moon-spark","name":"Moon Spark","level":0,"school":"Evocation","lists":["Wizard"],"casting_time":"Action","range":"60 feet","components":["V","S"],"notes":"Duration: Instantaneous","tags":["Damage"]}]"#,
        )
        .expect("write spells");
        std::fs::write(
            directory.join("classes.json"),
            r#"[{"id":"moon-warden","name":"Moon Warden","hit_die":10,"saving_throws":["strength","wisdom"],"skill_count":2,"skills":["Athletics","Insight","Nature","Perception","Survival"],"armor_training":["Light","Medium","Shields"],"weapon_training":["Simple","Martial"],"equipment":[{"name":"Longsword"},{"name":"Scale Mail"},{"name":"Shield"}],"equipment_gold":10,"starting_gold":150,"features":["Moonlit Vigil"],"weapon_mastery_count":2}]"#,
        )
        .expect("write classes");

        let manifest = load(&directory).expect("load pack");
        std::fs::remove_dir_all(&directory).expect("remove pack");
        assert_eq!(manifest.id, "my-campaign");
        assert_eq!(manifest.files.len(), 5);
        assert_eq!(manifest.species[0].id, "moonfolk");
        assert_eq!(manifest.backgrounds[0].id, "lunar-scout");
        assert_eq!(manifest.equipment[0].id, "moonblade");
        assert_eq!(manifest.spells[0].id, "moon-spark");
        assert_eq!(manifest.classes[0].id, "moon-warden");
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

    #[test]
    fn rejects_invalid_or_conflicting_background_rules() {
        let conflicting: PackBackground = serde_json::from_str(
            r#"{"id":"acolyte","name":"Custom Acolyte","abilities":["intelligence","wisdom","charisma"],"skills":["Insight","Religion"],"feat":"Alert","tool":"Calligrapher's Supplies","equipment":[{"name":"Robe"}]}"#,
        )
        .expect("background record");
        assert!(
            validate_backgrounds(&[conflicting], &[])
                .expect_err("SRD collision")
                .contains("conflicts with SRD background")
        );

        let invalid_magic: PackBackground = serde_json::from_str(
            r#"{"id":"hedge-mage","name":"Hedge Mage","abilities":["intelligence","wisdom","charisma"],"skills":["Arcana","Nature"],"feat":"Magic Initiate","tool":"Herbalism Kit","equipment":[{"name":"Robe"}]}"#,
        )
        .expect("background record");
        assert!(
            validate_backgrounds(&[invalid_magic], &[])
                .expect_err("missing spell list")
                .contains("magic_initiate_list")
        );
    }

    #[test]
    fn rejects_invalid_equipment_and_unknown_background_references() {
        let invalid_weapon: PackEquipment = serde_json::from_str(
            r#"{"id":"moonblade","name":"Moonblade","kind":{"type":"weapon","category":"Simple","kind":"Melee","properties":["Finesse"],"mastery":"Vex","damage":"bright","damage_type":"Radiant","normal_range":5}}"#,
        )
        .expect("equipment record");
        assert!(
            validate_equipment(&[invalid_weapon])
                .expect_err("invalid damage dice")
                .contains("invalid pack weapon mechanics")
        );

        let background: PackBackground = serde_json::from_str(
            r#"{"id":"lunar-scout","name":"Lunar Scout","abilities":["dexterity","wisdom","charisma"],"skills":["Perception","Survival"],"feat":"Alert","tool":"Navigator's Tools","equipment":[{"equipment_id":"missing-item"}]}"#,
        )
        .expect("background record");
        assert!(
            validate_backgrounds(&[background], &[])
                .expect_err("unknown item")
                .contains("references unknown pack equipment")
        );
    }

    #[test]
    fn rejects_invalid_or_conflicting_spell_rules() {
        let conflicting: PackSpell = serde_json::from_str(
            r#"{"id":"acid-splash","name":"acid splash","level":0,"school":"Evocation","lists":["Wizard"],"casting_time":"Action","range":"60 feet","components":["V","S"],"notes":"Duration: Instantaneous"}"#,
        )
        .expect("spell record");
        assert!(
            validate_spells(&[conflicting])
                .expect_err("SRD collision")
                .contains("conflicts with SRD spell")
        );

        let invalid_material: PackSpell = serde_json::from_str(
            r#"{"id":"moon-spark","name":"Moon Spark","level":0,"school":"Evocation","lists":["Wizard"],"casting_time":"Action","range":"60 feet","components":["V","M"],"notes":"Duration: Instantaneous"}"#,
        )
        .expect("spell record");
        assert!(
            validate_spells(&[invalid_material])
                .expect_err("material mismatch")
                .contains("invalid pack spell mechanics")
        );

        let invalid_list: PackSpell = serde_json::from_str(
            r#"{"id":"moon-spark","name":"Moon Spark","level":0,"school":"Evocation","lists":["Artificer"],"casting_time":"Action","range":"60 feet","components":["V"],"notes":"Duration: Instantaneous"}"#,
        )
        .expect("spell record");
        assert!(
            validate_spells(&[invalid_list])
                .expect_err("unsupported list")
                .contains("invalid pack spell mechanics")
        );
    }

    #[test]
    fn rejects_invalid_or_conflicting_class_rules() {
        let conflicting: PackClass = serde_json::from_str(
            r#"{"id":"fighter","name":"Custom Fighter","hit_die":10,"saving_throws":["strength","constitution"],"skill_count":2,"skills":["Athletics","Perception"],"armor_training":["Light"],"weapon_training":["Simple"],"equipment":[{"name":"Longsword"}],"starting_gold":100,"features":["Watchful"]}"#,
        )
        .expect("class record");
        assert!(
            validate_classes(&[conflicting], &[])
                .expect_err("SRD collision")
                .contains("conflicts with SRD class")
        );

        let invalid_spellcasting: PackClass = serde_json::from_str(
            r#"{"id":"moon-mage","name":"Moon Mage","hit_die":7,"saving_throws":["intelligence","intelligence"],"skill_count":3,"skills":["Arcana"],"armor_training":["Mystic"],"weapon_training":["Arcane"],"equipment":[{"equipment_id":"moonblade"}],"starting_gold":0,"features":[],"weapon_mastery_count":4}"#,
        )
        .expect("class record");
        assert!(
            validate_classes(&[invalid_spellcasting], &[])
                .expect_err("invalid class mechanics")
                .contains("invalid pack class mechanics")
        );

        let invalid_declarative_extensions: PackClass = serde_json::from_str(
            r#"{"id":"moon-mystic","name":"Moon Mystic","hit_die":8,"saving_throws":["intelligence","wisdom"],"skill_count":2,"skills":["Arcana","Insight","Nature"],"weapon_training":["Simple"],"equipment":[{"name":"Quarterstaff"}],"starting_gold":100,"features":["Moon Lore"],"choices":[{"id":"calling","label":"Calling","count":2,"options":[{"id":"seer","name":"Seer"}]}],"resources":[{"name":"Focus","maximum":0,"unit":"uses","recovery":"Long Rest"}],"spellcasting":{"ability":"wisdom","spell_list":"Artificer","cantrip_count":2,"prepared_spell_count":2,"spell_slots":0,"slot_recovery":""}}"#,
        )
        .expect("extended class record");
        assert!(
            validate_classes(&[invalid_declarative_extensions], &[])
                .expect_err("invalid extensions")
                .contains("invalid pack class mechanics")
        );
    }
}
