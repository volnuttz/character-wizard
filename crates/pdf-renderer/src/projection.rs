//! Projection of domain values onto supported `AcroForm` field names.

use std::collections::BTreeMap;

use character_wizard_domain::Character;

pub(super) fn project_spell_rows(character: &Character, values: &mut BTreeMap<String, String>) {
    const NOTES: [&str; 30] = [
        "Text108", "Text208", "Text209", "Text210", "Text211", "Text212", "Text213", "Text214",
        "Text215", "Text216", "Text217", "Text218", "Text219", "Text220", "Text221", "Text222",
        "Text223", "Text224", "Text225", "Text227", "Text228", "Text229", "Text230", "Text244",
        "Text231", "Text232", "Text233", "Text234", "Text235", "Text236",
    ];
    let spells = character.spell_table_entries();
    for (row, notes_field) in NOTES.iter().enumerate() {
        let spell = spells.get(row);
        values.insert(
            format!("Text105.{row}"),
            spell.map_or_else(String::new, |value| value.level.to_string()),
        );
        values.insert(
            format!("Text106.{row}"),
            spell.map_or_else(String::new, |value| value.name.clone()),
        );
        values.insert(
            format!("Text107.{row}"),
            spell.map_or_else(String::new, |value| value.casting_time.clone()),
        );
        values.insert(
            format!("Text109.{row}"),
            spell.map_or_else(String::new, |value| value.range.clone()),
        );
        values.insert(
            (*notes_field).to_owned(),
            spell.map_or_else(String::new, |value| value.notes.clone()),
        );
        let (concentration, ritual, material) = spell.map_or((false, false, false), |value| {
            (value.concentration, value.ritual, value.required_material)
        });
        let (section, local) = if row < 7 {
            (0, row)
        } else if row < 20 {
            (1, row - 7)
        } else {
            (2, row - 20)
        };
        let concentration_field = match section {
            0 => format!("Check Box252.{local}"),
            1 => format!("Check Box255.{local}"),
            _ => format!("Check Box258.{local}"),
        };
        let ritual_field = match section {
            0 => format!("Check Box253.{local}"),
            1 => format!("Check Box256.{local}"),
            _ => format!("Check Box259.{local}"),
        };
        let material_field = match section {
            0 => format!("Check Box254.0.{local}"),
            1 => format!("Check Box257.{local}"),
            _ => format!("Check Box260.{local}"),
        };
        for (field, checked) in [
            (concentration_field, concentration),
            (ritual_field, ritual),
            (material_field, material),
        ] {
            values.insert(field, if checked { "/Yes" } else { "/Off" }.to_owned());
        }
    }
}
