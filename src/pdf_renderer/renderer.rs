//! Supported-template validation and `AcroForm` rendering implementation.

use crate::character_wizard_domain::Character;
use std::{collections::BTreeMap, path::Path};

use crate::pdf_renderer::pdf_renderer_projection::project_spell_rows;
use crate::pdf_renderer::{acroform_inventory, render_fields};

type Result<T> = std::result::Result<T, String>;

const SUPPORTED_CATALOG_SHA256: &str =
    "211ab47d8428008778f7688b48cb69be264810d147685040e0d774ddaaa30b49";
const TEMPLATE_PAGE_URL: &str = "https://www.dndbeyond.com/resources/1779-d-d-character-sheets";
const TEMPLATE_DOWNLOAD_URL: &str =
    "https://media.dndbeyond.com/compendium-images/free-rules/ph/character-sheet.pdf";

/// Validate the exact supported two-page official character sheet.
///
/// # Errors
///
/// Returns an error when the template is unreadable or its field catalog differs
/// from the development fixture.
pub fn validate_template(template: impl AsRef<Path>) -> Result<()> {
    let template = template.as_ref();
    let inventory = acroform_inventory(template).map_err(|error| {
        format!("Unable to read PDF template {}. Download the official sheet from {TEMPLATE_PAGE_URL}. ({error})", template.display())
    })?;
    if inventory.page_count != 2
        || inventory.field_count != 425
        || inventory.text_field_count != 260
        || inventory.button_field_count != 151
        || inventory.untyped_field_count != 14
        || inventory.sha256 != SUPPORTED_CATALOG_SHA256
    {
        return Err(format!(
            "Incompatible PDF template {}. Download the supported official sheet from {TEMPLATE_DOWNLOAD_URL} (this direct URL may change; see {TEMPLATE_PAGE_URL}).",
            template.display()
        ));
    }
    Ok(())
}

/// Render the canonical identity, ability, save, skill, and combat summary fields.
///
/// # Errors
///
/// Returns an error when the supported template cannot be read or written.
#[allow(clippy::too_many_lines)]
pub fn render_character(
    character: &Character,
    template: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<()> {
    validate_template(&template)?;
    let sheet = character.sheet();
    let mut values = BTreeMap::from([
        ("Text1".to_owned(), character.name.clone()),
        ("Text6".to_owned(), character.background_name().to_owned()),
        ("Text7".to_owned(), character.character_class.to_string()),
        ("Text8".to_owned(), character.species_name().to_owned()),
        ("Text9".to_owned(), String::new()),
        ("Text11".to_owned(), character.level.to_string()),
        ("Text12".to_owned(), character.xp.to_string()),
        ("Text13".to_owned(), character.armor_class().to_string()),
        ("Text14".to_owned(), character.hit_points().to_string()),
        ("Text15".to_owned(), String::new()),
        ("Text16".to_owned(), character.hit_points().to_string()),
        ("Text17".to_owned(), format!("1d{}", sheet.hit_die())),
        (
            "Text19".to_owned(),
            signed(i16::from(character.proficiency_bonus())),
        ),
        ("Text18".to_owned(), String::new()),
        ("Text26".to_owned(), signed(character.initiative_modifier())),
        ("Text27".to_owned(), character.speed().to_string()),
        (
            "Text28".to_owned(),
            character.size.chars().next().unwrap_or(' ').to_string(),
        ),
        (
            "Text29".to_owned(),
            character.passive_perception().to_string(),
        ),
        ("Text54".to_owned(), character.class_traits().join("\n")),
        (
            "Text55".to_owned(),
            character
                .class_resources()
                .iter()
                .map(crate::character_wizard_domain::ClassResource::summary)
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        ("Text57".to_owned(), character.species_traits().join("\n")),
        (
            "Text58".to_owned(),
            character.origin_feat_traits().join("\n"),
        ),
        ("Text59".to_owned(), character.weapon_proficiencies()),
        (
            "Text60".to_owned(),
            character.all_tool_proficiencies().join("\n"),
        ),
        (
            "Text96".to_owned(),
            character.appearance.clone().unwrap_or_default(),
        ),
        ("Text97".to_owned(), character_details(character)),
        ("Text93".to_owned(), String::new()),
        ("Text94".to_owned(), String::new()),
        ("Text95".to_owned(), String::new()),
        ("Text111".to_owned(), String::new()),
        (
            "Text98".to_owned(),
            character_languages(character).join("\n"),
        ),
        ("Text100".to_owned(), character.alignment.clone()),
    ]);
    for (ability, score_field, modifier_field, save_field, checkbox) in [
        ("strength", "Text64", "Text21", "Text91", "Check Box37"),
        ("dexterity", "Text66", "Text22", "Text87", "Check Box33"),
        ("constitution", "Text67", "Text24", "Text86", "Check Box32"),
        ("intelligence", "Text63", "Text20", "Text69", "Check Box4"),
        ("wisdom", "Text65", "Text23", "Text75", "Check Box21"),
        ("charisma", "Text68", "Text25", "Text81", "Check Box26"),
    ] {
        values.insert(
            score_field.to_owned(),
            character.abilities.score(ability).to_string(),
        );
        values.insert(
            modifier_field.to_owned(),
            signed(character.abilities.modifier(ability)),
        );
        let save = sheet.saving_throw(ability);
        values.insert(save_field.to_owned(), signed(save.modifier));
        values.insert(
            checkbox.to_owned(),
            if save.proficient { "/Yes" } else { "/Off" }.to_owned(),
        );
    }
    for (skill, field, checkbox) in [
        ("Animal Handling", "Text76", "Check Box22"),
        ("Insight", "Text77", "Check Box23"),
        ("Medicine", "Text78", "Check Box25"),
        ("Perception", "Text79", "Check Box31"),
        ("Survival", "Text80", "Check Box24"),
        ("Deception", "Text82", "Check Box27"),
        ("Intimidation", "Text83", "Check Box28"),
        ("Performance", "Text84", "Check Box30"),
        ("Persuasion", "Text85", "Check Box29"),
        ("Acrobatics", "Text88", "Check Box34"),
        ("Sleight of Hand", "Text89", "Check Box35"),
        ("Stealth", "Text90", "Check Box36"),
        ("Athletics", "Text92", "Check Box38"),
        ("Arcana", "Text70", "Check Box16"),
        ("History", "Text71", "Check Box17"),
        ("Investigation", "Text72", "Check Box19"),
        ("Nature", "Text73", "Check Box20"),
        ("Religion", "Text74", "Check Box18"),
    ] {
        values.insert(field.to_owned(), signed(character.skill_modifier(skill)));
        values.insert(
            checkbox.to_owned(),
            if character.skills().contains(skill) {
                "/Yes"
            } else {
                "/Off"
            }
            .to_owned(),
        );
    }
    for (training, checkbox) in [
        ("Light", "Check Box13"),
        ("Medium", "Check Box14"),
        ("Heavy", "Check Box15"),
        ("Shields", "Check Box12"),
    ] {
        values.insert(
            checkbox.to_owned(),
            if character.armor_training().contains(training) {
                "/Yes"
            } else {
                "/Off"
            }
            .to_owned(),
        );
    }
    let attacks = character.weapon_attacks();
    for (index, fields) in [
        ["Text30", "Text31", "Text32", "Text33"],
        ["Text34", "Text35", "Text36", "Text37"],
        ["Text38", "Text39", "Text40", "Text41"],
        ["Text42", "Text43", "Text44", "Text45"],
        ["Text46", "Text47", "Text48", "Text49"],
        ["Text50", "Text51", "Text52", "Text53"],
    ]
    .iter()
    .enumerate()
    {
        if let Some(attack) = attacks.get(index) {
            values.insert(fields[0].to_owned(), attack.name.clone());
            values.insert(fields[1].to_owned(), signed(attack.attack_bonus));
            values.insert(
                fields[2].to_owned(),
                format!("{} {}", attack.damage, attack.damage_type),
            );
            values.insert(
                fields[3].to_owned(),
                std::iter::once(format!("Range {}", attack.range))
                    .chain(attack.properties.iter().cloned())
                    .chain(attack.notes.iter().cloned())
                    .collect::<Vec<_>>()
                    .join("; "),
            );
        } else {
            for field in fields {
                values.insert((*field).to_owned(), String::new());
            }
        }
    }
    let equipment = character
        .inventory()
        .iter()
        .map(|item| {
            if item.quantity > 1 {
                format!("{} x {}", item.quantity, item.name)
            } else {
                item.name.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    values.insert("Text99".to_owned(), equipment);
    let coins = character.coins();
    for (field, amount) in [
        ("Text226", coins.copper),
        ("Text267", coins.silver),
        ("Text268", coins.electrum),
        ("Text269", coins.gold),
        ("Text270", coins.platinum),
    ] {
        values.insert(field.to_owned(), amount.to_string());
    }
    if let Some(profile) = character.spellcasting_profiles().first() {
        values.insert("Text93".to_owned(), signed(profile.modifier));
        values.insert("Text94".to_owned(), profile.save_dc.to_string());
        values.insert("Text95".to_owned(), signed(profile.attack_bonus));
        values.insert("Text111".to_owned(), title_case(&profile.ability));
    }
    for field in [
        "Text112", "Text113", "Text114", "Text117", "Text116", "Text115", "Text118", "Text119",
        "Text120",
    ] {
        values.insert(field.to_owned(), String::new());
    }
    if let Some(slot) = character.spell_slots().first() {
        values.insert("Text112".to_owned(), slot.total.to_string());
    }
    project_spell_rows(character, &mut values);
    render_fields(template, output, &values)
}

fn character_languages(character: &Character) -> Vec<String> {
    let mut languages = vec!["Common".to_owned()];
    languages.extend(character.selected_languages.iter().cloned());
    languages.extend(character.class_choices.additional_language.iter().cloned());
    languages
}

fn character_details(character: &Character) -> String {
    [
        character
            .backstory
            .as_ref()
            .map(|value| format!("Backstory\n{value}")),
        character
            .personality
            .as_ref()
            .map(|value| format!("Personality\n{value}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("\n\n")
}

fn signed(value: i16) -> String {
    format!("{value:+}")
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::render_character;
    use crate::{
        character_wizard_domain::{
            Character, PackBackground, PackEquipment, PackSpecies, PackSpell,
        },
        pdf_renderer::pdf_renderer_field_writer::read_field_value,
    };

    #[test]
    fn pack_species_name_is_stored_in_the_existing_species_field() {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let mut character =
            Character::from_json(include_str!("../../fixtures/complete-character.json"))
                .expect("character fixture");
        character.species = "moonfolk".parse().expect("species id");
        character.size = "Small".parse().expect("size");
        character.resolved_pack_species = Some(
            serde_json::from_str::<PackSpecies>(
                r#"{"id":"moonfolk","name":"Moonfolk","sizes":["Small"],"speed":35,"traits":["Moonlit Step"]}"#,
            )
            .expect("pack species"),
        );
        let output = std::env::temp_dir().join(format!(
            "character-wizard-pack-species-render-{}-{}.pdf",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        render_character(
            &character,
            std::path::Path::new("assets/character-sheet.pdf"),
            &output,
        )
        .expect("render character");
        let value = read_field_value(&output, "Text8").expect("read species field");
        assert_eq!(value.as_str().expect("text value"), b"Moonfolk");
        let document = lopdf::Document::load(&output).expect("reopen output");
        assert_eq!(document.get_pages().len(), 2);
        assert!(document.catalog().expect("catalog").has(b"AcroForm"));
        std::fs::remove_file(output).expect("remove proof PDF");
    }

    #[test]
    fn pack_background_summary_is_stored_in_existing_fields() {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let mut character =
            Character::from_json(include_str!("../../fixtures/complete-character.json"))
                .expect("character fixture");
        character.background = "lunar-scout".parse().expect("background id");
        character.class_equipment_option = "Gold".to_owned();
        character.background_equipment_option = "A".to_owned();
        character.resolved_pack_background = Some(
            serde_json::from_str::<PackBackground>(
                r#"{"id":"lunar-scout","name":"Lunar Scout","abilities":["dexterity","wisdom","charisma"],"skills":["Perception","Survival"],"feat":"Alert","tool":"Navigator's Tools","equipment":[{"equipment_id":"moonblade"},{"equipment_id":"moonweave"},{"equipment_id":"moonward"}],"equipment_gold":12,"gold_alternative":50}"#,
            )
            .expect("pack background"),
        );
        character.resolved_pack_equipment = serde_json::from_str::<Vec<PackEquipment>>(
            r#"[{"id":"moonblade","name":"Moonblade","kind":{"type":"weapon","category":"Simple","kind":"Melee","properties":["Finesse","Light"],"mastery":"Vex","damage":"1d8","damage_type":"Radiant","normal_range":5}},{"id":"moonweave","name":"Moonweave Armor","kind":{"type":"armor","category":"Light","base_ac":13}},{"id":"moonward","name":"Moonward","kind":{"type":"shield","armor_class_bonus":3}}]"#,
        )
        .expect("pack equipment");
        let output = std::env::temp_dir().join(format!(
            "character-wizard-pack-background-render-{}-{}.pdf",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        render_character(
            &character,
            std::path::Path::new("assets/character-sheet.pdf"),
            &output,
        )
        .expect("render character");
        for (field, expected) in [
            ("Text6", "Lunar Scout"),
            ("Text58", "Alert: Initiative Proficiency; Initiative Swap"),
            ("Text60", "Navigator's Tools"),
            ("Text13", "19"),
            ("Text30", "Moonblade"),
            ("Text31", "+5"),
            ("Text32", "1d8+3 Radiant"),
        ] {
            let value = read_field_value(&output, field).expect("read background field");
            assert_eq!(value.as_str().expect("text value"), expected.as_bytes());
        }
        let inventory = read_field_value(&output, "Text99").expect("read inventory field");
        let inventory =
            std::str::from_utf8(inventory.as_str().expect("text value")).expect("UTF-8 inventory");
        assert!(inventory.contains("Moonblade"));
        assert!(inventory.contains("Moonweave Armor"));
        assert!(inventory.contains("Moonward"));
        let document = lopdf::Document::load(&output).expect("reopen output");
        assert_eq!(document.get_pages().len(), 2);
        assert!(document.catalog().expect("catalog").has(b"AcroForm"));
        std::fs::remove_file(output).expect("remove proof PDF");
    }

    #[test]
    fn pack_spell_metadata_is_stored_in_existing_spell_fields() {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let mut character =
            Character::from_json(include_str!("../../fixtures/complete-character.json"))
                .expect("character fixture");
        character
            .class_choices
            .cantrips
            .insert("moon-spark".to_owned());
        character.resolved_pack_spells = Some(
            serde_json::from_str::<Vec<PackSpell>>(
                r#"[{"id":"moon-spark","name":"Moon Spark","level":0,"school":"Evocation","lists":["Wizard"],"casting_time":"Action","range":"60 feet","components":["V","S","M"],"material":"a moonstone shard","concentration":true,"notes":"Duration: up to 1 minute","tags":["Damage"]}]"#,
            )
            .expect("pack spells"),
        );
        let row = character
            .spell_table_entries()
            .iter()
            .position(|spell| spell.name == "Moon Spark")
            .expect("custom spell row");
        let output = std::env::temp_dir().join(format!(
            "character-wizard-pack-spell-render-{}-{}.pdf",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        render_character(
            &character,
            std::path::Path::new("assets/character-sheet.pdf"),
            &output,
        )
        .expect("render character");
        for (field, expected) in [
            (format!("Text105.{row}"), "0"),
            (format!("Text106.{row}"), "Moon Spark"),
            (format!("Text107.{row}"), "Action"),
            (format!("Text109.{row}"), "60 feet"),
        ] {
            let value = read_field_value(&output, &field).expect("read custom spell field");
            assert_eq!(value.as_str().expect("text value"), expected.as_bytes());
        }
        let local = row;
        for field in [
            format!("Check Box252.{local}"),
            format!("Check Box254.0.{local}"),
        ] {
            let value = read_field_value(&output, &field).expect("read custom spell checkbox");
            assert_ne!(value.as_name().expect("checkbox value"), b"Off");
        }
        std::fs::remove_file(output).expect("remove proof PDF");
    }
}
