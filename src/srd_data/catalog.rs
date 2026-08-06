//! Stable façade over the SRD rule-family catalogs.

#[path = "classes.rs"]
mod classes;
#[path = "core.rs"]
mod core;
#[path = "equipment.rs"]
mod equipment;
#[path = "species.rs"]
mod species;
#[path = "spells.rs"]
mod spells;

pub use classes::*;
pub use core::*;
pub use equipment::*;
pub use species::*;
pub use spells::*;

#[cfg(test)]
mod tests {
    use super::{background_rule, class_rule, skill_ability, species_rule};

    #[test]
    fn inventories_are_complete() {
        assert_eq!(
            12,
            [
                "Barbarian",
                "Bard",
                "Cleric",
                "Druid",
                "Fighter",
                "Monk",
                "Paladin",
                "Ranger",
                "Rogue",
                "Sorcerer",
                "Warlock",
                "Wizard"
            ]
            .into_iter()
            .filter_map(class_rule)
            .count()
        );
        assert_eq!(
            4,
            ["Acolyte", "Criminal", "Sage", "Soldier"]
                .into_iter()
                .filter_map(background_rule)
                .count()
        );
        assert_eq!(
            9,
            [
                "Dragonborn",
                "Dwarf",
                "Elf",
                "Gnome",
                "Goliath",
                "Halfling",
                "Human",
                "Orc",
                "Tiefling"
            ]
            .into_iter()
            .filter_map(species_rule)
            .count()
        );
        assert_eq!(Some("wisdom"), skill_ability("Perception"));
    }
}
