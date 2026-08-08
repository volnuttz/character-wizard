//! Presentation-neutral character review projection.

use std::fmt::Write as _;

use crate::character_wizard_domain::{Ability, Character};

use super::creation_prompts::PromptPort;

pub(super) fn display_character_review(prompts: &dyn PromptPort, character: &Character) {
    prompts.display("\nCharacter review");
    for (label, value) in character_review_rows(character) {
        prompts.display(&format!("{label:<18} {value}"));
    }
    prompts.display("\nAccept the character or choose a section to revise.");
}

#[allow(clippy::too_many_lines)]
pub(super) fn character_review_rows(character: &Character) -> Vec<(&'static str, String)> {
    let abilities = [
        ("STR", Ability::Strength, character.abilities.strength),
        ("DEX", Ability::Dexterity, character.abilities.dexterity),
        (
            "CON",
            Ability::Constitution,
            character.abilities.constitution,
        ),
        (
            "INT",
            Ability::Intelligence,
            character.abilities.intelligence,
        ),
        ("WIS", Ability::Wisdom, character.abilities.wisdom),
        ("CHA", Ability::Charisma, character.abilities.charisma),
    ]
    .into_iter()
    .map(|(short_name, ability, score)| {
        format!(
            "{short_name} {score} ({:+})",
            character.abilities.modifier(ability)
        )
    })
    .collect::<Vec<_>>()
    .join("  ");
    let languages = std::iter::once("Common".to_owned())
        .chain(character.selected_languages.iter().cloned())
        .chain(character.class_choices.additional_language.iter().cloned())
        .collect::<Vec<_>>()
        .join(", ");
    let mut equipment = character
        .inventory()
        .into_iter()
        .map(|item| {
            if item.quantity > 1 {
                format!("{} x {}", item.quantity, item.name)
            } else {
                item.name
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    if equipment.is_empty() {
        equipment.push_str("No package items");
    }
    let gold = character.coins().gold;
    write!(equipment, "; {gold} GP").expect("writing to String cannot fail");
    let mut rows = vec![
        ("Name", character.name.clone()),
        (
            "Character",
            format!(
                "Level {} {} {}",
                character.level,
                character.species_name(),
                character.class_name()
            ),
        ),
        ("Background", character.background_name().to_owned()),
        ("Alignment", character.alignment.clone()),
        ("Size", character.size.to_string()),
        ("Abilities", abilities),
        (
            "Combat",
            format!(
                "HP {}  AC {}  Initiative {:+}  Speed {} ft.",
                character.hit_points(),
                character.armor_class(),
                character.initiative_modifier(),
                character.speed()
            ),
        ),
        (
            "Skills",
            character
                .skills()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", "),
        ),
        ("Languages", languages),
        ("Equipment", equipment),
    ];
    if !character.class_choices.cantrips.is_empty() {
        rows.push((
            "Cantrips",
            character
                .class_choices
                .cantrips
                .iter()
                .map(|spell| character.spell_name(spell).to_owned())
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    if !character.class_choices.prepared_spells.is_empty() {
        rows.push((
            "Prepared spells",
            character
                .class_choices
                .prepared_spells
                .iter()
                .map(|spell| character.spell_name(spell).to_owned())
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    if character.resolved_pack_class.is_some() {
        rows.push(("Class features", character.class_traits().join("; ")));
        if !character.class_resources().is_empty() {
            rows.push((
                "Class resources",
                character
                    .class_resources()
                    .iter()
                    .map(crate::character_wizard_domain::ClassResource::summary)
                    .collect::<Vec<_>>()
                    .join("; "),
            ));
        }
    }
    rows
}
