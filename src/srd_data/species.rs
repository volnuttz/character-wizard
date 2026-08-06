//! Species rule catalog.

use crate::srd_data::SpeciesRule;

#[must_use]
pub fn species_rule(name: &str) -> Option<SpeciesRule> {
    let rule = match name {
        "Dragonborn" | "Elf" => SpeciesRule {
            sizes: &["Medium"],
            speed: 30,
            darkvision_range: Some(60),
        },
        "Dwarf" | "Orc" => SpeciesRule {
            sizes: &["Medium"],
            speed: 30,
            darkvision_range: Some(120),
        },
        "Gnome" => SpeciesRule {
            sizes: &["Small"],
            speed: 30,
            darkvision_range: Some(60),
        },
        "Goliath" => SpeciesRule {
            sizes: &["Medium"],
            speed: 35,
            darkvision_range: None,
        },
        "Halfling" => SpeciesRule {
            sizes: &["Small"],
            speed: 30,
            darkvision_range: None,
        },
        "Human" => SpeciesRule {
            sizes: &["Medium", "Small"],
            speed: 30,
            darkvision_range: None,
        },
        "Tiefling" => SpeciesRule {
            sizes: &["Medium", "Small"],
            speed: 30,
            darkvision_range: Some(60),
        },
        _ => return None,
    };
    Some(rule)
}

#[must_use]
pub fn skill_ability(skill: &str) -> Option<&'static str> {
    match skill {
        "Acrobatics" | "Sleight of Hand" | "Stealth" => Some("dexterity"),
        "Animal Handling" | "Insight" | "Medicine" | "Perception" | "Survival" => Some("wisdom"),
        "Arcana" | "History" | "Investigation" | "Nature" | "Religion" => Some("intelligence"),
        "Athletics" => Some("strength"),
        "Deception" | "Intimidation" | "Performance" | "Persuasion" => Some("charisma"),
        _ => None,
    }
}

#[must_use]
pub fn species_traits(species: &str) -> &'static [&'static str] {
    match species {
        "Dragonborn" => &["Breath Weapon"],
        "Dwarf" => &["Dwarven Resilience", "Dwarven Toughness", "Stonecunning"],
        "Elf" => &["Fey Ancestry", "Trance"],
        "Gnome" => &["Gnomish Cunning"],
        "Goliath" => &["Giant Ancestry", "Powerful Build"],
        "Halfling" => &["Brave", "Halfling Nimbleness", "Luck", "Naturally Stealthy"],
        "Human" => &["Resourceful", "Skillful", "Versatile"],
        "Orc" => &["Adrenaline Rush", "Relentless Endurance"],
        "Tiefling" => &["Otherworldly Presence"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::{skill_ability, species_rule};

    #[test]
    fn species_and_skill_catalogs_are_queryable() {
        assert_eq!(
            species_rule("Dwarf").and_then(|rule| rule.darkvision_range),
            Some(120)
        );
        assert_eq!(skill_ability("Perception"), Some("wisdom"));
    }
}
