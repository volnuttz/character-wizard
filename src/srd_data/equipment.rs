//! Armor, weapons, packs, and starting-equipment catalogs.

use crate::srd_data::{ArmorRule, EquipmentGrant, WeaponRule};

#[must_use]
pub fn armor_rule(name: &str) -> Option<ArmorRule> {
    let rule = match name {
        "Padded Armor" | "Leather Armor" => ArmorRule {
            category: "Light",
            base_ac: 11,
            dexterity_cap: None,
            strength_requirement: None,
        },
        "Studded Leather Armor" => ArmorRule {
            category: "Light",
            base_ac: 12,
            dexterity_cap: None,
            strength_requirement: None,
        },
        "Hide Armor" => ArmorRule {
            category: "Medium",
            base_ac: 12,
            dexterity_cap: Some(2),
            strength_requirement: None,
        },
        "Chain Shirt" => ArmorRule {
            category: "Medium",
            base_ac: 13,
            dexterity_cap: Some(2),
            strength_requirement: None,
        },
        "Scale Mail" | "Breastplate" => ArmorRule {
            category: "Medium",
            base_ac: 14,
            dexterity_cap: Some(2),
            strength_requirement: None,
        },
        "Half Plate Armor" => ArmorRule {
            category: "Medium",
            base_ac: 15,
            dexterity_cap: Some(2),
            strength_requirement: None,
        },
        "Ring Mail" => ArmorRule {
            category: "Heavy",
            base_ac: 14,
            dexterity_cap: Some(0),
            strength_requirement: None,
        },
        "Chain Mail" => ArmorRule {
            category: "Heavy",
            base_ac: 16,
            dexterity_cap: Some(0),
            strength_requirement: Some(13),
        },
        "Splint Armor" => ArmorRule {
            category: "Heavy",
            base_ac: 17,
            dexterity_cap: Some(0),
            strength_requirement: Some(15),
        },
        "Plate Armor" => ArmorRule {
            category: "Heavy",
            base_ac: 18,
            dexterity_cap: Some(0),
            strength_requirement: Some(15),
        },
        _ => return None,
    };
    Some(rule)
}

macro_rules! weapon {
    ($category:expr, $kind:expr, [$($property:expr),*], $mastery:expr, $damage:expr, $damage_type:expr, $range:expr, $long:expr, $versatile:expr) => {
        WeaponRule { category: $category, kind: $kind, properties: &[$($property),*], mastery: $mastery, damage: $damage, damage_type: $damage_type, normal_range: $range, long_range: $long, versatile_damage: $versatile }
    };
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn weapon_rule(name: &str) -> Option<WeaponRule> {
    let rule = match name {
        "Club" => weapon!(
            "Simple",
            "Melee",
            ["Light"],
            "Slow",
            "1d4",
            "Bludgeoning",
            5,
            None,
            None
        ),
        "Dagger" => weapon!(
            "Simple",
            "Melee",
            ["Finesse", "Light", "Thrown"],
            "Nick",
            "1d4",
            "Piercing",
            20,
            Some(60),
            None
        ),
        "Greatclub" => weapon!(
            "Simple",
            "Melee",
            ["Two-Handed"],
            "Push",
            "1d8",
            "Bludgeoning",
            5,
            None,
            None
        ),
        "Handaxe" => weapon!(
            "Simple",
            "Melee",
            ["Light", "Thrown"],
            "Vex",
            "1d6",
            "Slashing",
            20,
            Some(60),
            None
        ),
        "Javelin" => weapon!(
            "Simple",
            "Melee",
            ["Thrown"],
            "Slow",
            "1d6",
            "Piercing",
            30,
            Some(120),
            None
        ),
        "Light Hammer" => weapon!(
            "Simple",
            "Melee",
            ["Light", "Thrown"],
            "Nick",
            "1d4",
            "Bludgeoning",
            20,
            Some(60),
            None
        ),
        "Mace" => weapon!(
            "Simple",
            "Melee",
            [],
            "Sap",
            "1d6",
            "Bludgeoning",
            5,
            None,
            None
        ),
        "Quarterstaff" => weapon!(
            "Simple",
            "Melee",
            ["Versatile"],
            "Topple",
            "1d6",
            "Bludgeoning",
            5,
            None,
            Some("1d8")
        ),
        "Sickle" => weapon!(
            "Simple",
            "Melee",
            ["Light"],
            "Nick",
            "1d4",
            "Slashing",
            5,
            None,
            None
        ),
        "Spear" => weapon!(
            "Simple",
            "Melee",
            ["Thrown", "Versatile"],
            "Sap",
            "1d6",
            "Piercing",
            20,
            Some(60),
            Some("1d8")
        ),
        "Dart" => weapon!(
            "Simple",
            "Ranged",
            ["Finesse", "Thrown"],
            "Vex",
            "1d4",
            "Piercing",
            20,
            Some(60),
            None
        ),
        "Light Crossbow" => weapon!(
            "Simple",
            "Ranged",
            ["Ammunition", "Loading", "Two-Handed"],
            "Slow",
            "1d8",
            "Piercing",
            80,
            Some(320),
            None
        ),
        "Shortbow" => weapon!(
            "Simple",
            "Ranged",
            ["Ammunition", "Two-Handed"],
            "Vex",
            "1d6",
            "Piercing",
            80,
            Some(320),
            None
        ),
        "Sling" => weapon!(
            "Simple",
            "Ranged",
            ["Ammunition"],
            "Slow",
            "1d4",
            "Bludgeoning",
            30,
            Some(120),
            None
        ),
        "Battleaxe" => weapon!(
            "Martial",
            "Melee",
            ["Versatile"],
            "Topple",
            "1d8",
            "Slashing",
            5,
            None,
            Some("1d10")
        ),
        "Flail" => weapon!(
            "Martial",
            "Melee",
            [],
            "Sap",
            "1d8",
            "Bludgeoning",
            5,
            None,
            None
        ),
        "Glaive" => weapon!(
            "Martial",
            "Melee",
            ["Heavy", "Reach", "Two-Handed"],
            "Graze",
            "1d10",
            "Slashing",
            10,
            None,
            None
        ),
        "Greataxe" => weapon!(
            "Martial",
            "Melee",
            ["Heavy", "Two-Handed"],
            "Cleave",
            "1d12",
            "Slashing",
            5,
            None,
            None
        ),
        "Greatsword" => weapon!(
            "Martial",
            "Melee",
            ["Heavy", "Two-Handed"],
            "Graze",
            "2d6",
            "Slashing",
            5,
            None,
            None
        ),
        "Halberd" => weapon!(
            "Martial",
            "Melee",
            ["Heavy", "Reach", "Two-Handed"],
            "Cleave",
            "1d10",
            "Slashing",
            10,
            None,
            None
        ),
        "Lance" => weapon!(
            "Martial",
            "Melee",
            ["Heavy", "Reach", "Two-Handed"],
            "Topple",
            "1d10",
            "Piercing",
            10,
            None,
            None
        ),
        "Longsword" => weapon!(
            "Martial",
            "Melee",
            ["Versatile"],
            "Sap",
            "1d8",
            "Slashing",
            5,
            None,
            Some("1d10")
        ),
        "Maul" => weapon!(
            "Martial",
            "Melee",
            ["Heavy", "Two-Handed"],
            "Topple",
            "2d6",
            "Bludgeoning",
            5,
            None,
            None
        ),
        "Morningstar" => weapon!(
            "Martial",
            "Melee",
            [],
            "Sap",
            "1d8",
            "Piercing",
            5,
            None,
            None
        ),
        "Pike" => weapon!(
            "Martial",
            "Melee",
            ["Heavy", "Reach", "Two-Handed"],
            "Push",
            "1d10",
            "Piercing",
            10,
            None,
            None
        ),
        "Rapier" => weapon!(
            "Martial",
            "Melee",
            ["Finesse"],
            "Vex",
            "1d8",
            "Piercing",
            5,
            None,
            None
        ),
        "Scimitar" => weapon!(
            "Martial",
            "Melee",
            ["Finesse", "Light"],
            "Nick",
            "1d6",
            "Slashing",
            5,
            None,
            None
        ),
        "Shortsword" => weapon!(
            "Martial",
            "Melee",
            ["Finesse", "Light"],
            "Vex",
            "1d6",
            "Piercing",
            5,
            None,
            None
        ),
        "Trident" => weapon!(
            "Martial",
            "Melee",
            ["Thrown", "Versatile"],
            "Topple",
            "1d8",
            "Piercing",
            20,
            Some(60),
            Some("1d10")
        ),
        "Warhammer" => weapon!(
            "Martial",
            "Melee",
            ["Versatile"],
            "Push",
            "1d8",
            "Bludgeoning",
            5,
            None,
            Some("1d10")
        ),
        "War Pick" => weapon!(
            "Martial",
            "Melee",
            ["Versatile"],
            "Sap",
            "1d8",
            "Piercing",
            5,
            None,
            Some("1d10")
        ),
        "Whip" => weapon!(
            "Martial",
            "Melee",
            ["Finesse", "Reach"],
            "Slow",
            "1d4",
            "Slashing",
            10,
            None,
            None
        ),
        "Blowgun" => weapon!(
            "Martial",
            "Ranged",
            ["Ammunition", "Loading"],
            "Vex",
            "1",
            "Piercing",
            25,
            Some(100),
            None
        ),
        "Hand Crossbow" => weapon!(
            "Martial",
            "Ranged",
            ["Ammunition", "Light", "Loading"],
            "Vex",
            "1d6",
            "Piercing",
            30,
            Some(120),
            None
        ),
        "Heavy Crossbow" => weapon!(
            "Martial",
            "Ranged",
            ["Ammunition", "Heavy", "Loading", "Two-Handed"],
            "Push",
            "1d10",
            "Piercing",
            100,
            Some(400),
            None
        ),
        "Longbow" => weapon!(
            "Martial",
            "Ranged",
            ["Ammunition", "Heavy", "Two-Handed"],
            "Slow",
            "1d8",
            "Piercing",
            150,
            Some(600),
            None
        ),
        "Musket" => weapon!(
            "Martial",
            "Ranged",
            ["Ammunition", "Loading", "Two-Handed"],
            "Slow",
            "1d12",
            "Piercing",
            40,
            Some(120),
            None
        ),
        "Pistol" => weapon!(
            "Martial",
            "Ranged",
            ["Ammunition", "Loading"],
            "Vex",
            "1d10",
            "Piercing",
            30,
            Some(90),
            None
        ),
        _ => return None,
    };
    Some(rule)
}

const EXPLORER_PACK: &[EquipmentGrant] = &[
    EquipmentGrant::one("Backpack"),
    EquipmentGrant::one("Bedroll"),
    EquipmentGrant::many("Oil", 2),
    EquipmentGrant::many("Rations", 10),
    EquipmentGrant::one("Rope"),
    EquipmentGrant::one("Tinderbox"),
    EquipmentGrant::many("Torch", 10),
    EquipmentGrant::one("Waterskin"),
];
const DUNGEONEER_PACK: &[EquipmentGrant] = &[
    EquipmentGrant::one("Backpack"),
    EquipmentGrant::one("Caltrops"),
    EquipmentGrant::one("Crowbar"),
    EquipmentGrant::many("Oil", 2),
    EquipmentGrant::many("Rations", 10),
    EquipmentGrant::one("Rope"),
    EquipmentGrant::one("Tinderbox"),
    EquipmentGrant::many("Torch", 10),
    EquipmentGrant::one("Waterskin"),
];
const ENTERTAINER_PACK: &[EquipmentGrant] = &[
    EquipmentGrant::one("Backpack"),
    EquipmentGrant::one("Bedroll"),
    EquipmentGrant::one("Bell"),
    EquipmentGrant::one("Bullseye Lantern"),
    EquipmentGrant::many("Costume", 3),
    EquipmentGrant::one("Mirror"),
    EquipmentGrant::many("Oil", 8),
    EquipmentGrant::many("Rations", 9),
    EquipmentGrant::one("Tinderbox"),
    EquipmentGrant::one("Waterskin"),
];
const PRIEST_PACK: &[EquipmentGrant] = &[
    EquipmentGrant::one("Backpack"),
    EquipmentGrant::one("Blanket"),
    EquipmentGrant::one("Holy Water"),
    EquipmentGrant::one("Lamp"),
    EquipmentGrant::many("Rations", 7),
    EquipmentGrant::one("Robe"),
    EquipmentGrant::one("Tinderbox"),
];
const SCHOLAR_PACK: &[EquipmentGrant] = &[
    EquipmentGrant::one("Backpack"),
    EquipmentGrant::one("Book"),
    EquipmentGrant::one("Ink"),
    EquipmentGrant::one("Ink Pen"),
    EquipmentGrant::one("Lamp"),
    EquipmentGrant::many("Oil", 10),
    EquipmentGrant::many("Parchment", 10),
    EquipmentGrant::one("Tinderbox"),
];

#[must_use]
pub fn pack_contents(name: &str) -> Option<&'static [EquipmentGrant]> {
    match name {
        "Explorer's Pack" => Some(EXPLORER_PACK),
        "Dungeoneer's Pack" => Some(DUNGEONEER_PACK),
        "Entertainer's Pack" => Some(ENTERTAINER_PACK),
        "Priest's Pack" => Some(PRIEST_PACK),
        "Scholar's Pack" => Some(SCHOLAR_PACK),
        _ => None,
    }
}

const BARBARIAN_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Greataxe"),
    EquipmentGrant::many("Handaxe", 4),
    EquipmentGrant::one("Explorer's Pack"),
];
const BARD_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Leather Armor"),
    EquipmentGrant::many("Dagger", 2),
    EquipmentGrant::one("Chosen Musical Instrument"),
    EquipmentGrant::one("Entertainer's Pack"),
];
const CLERIC_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Chain Shirt"),
    EquipmentGrant::one("Shield"),
    EquipmentGrant::one("Mace"),
    EquipmentGrant::one("Holy Symbol"),
    EquipmentGrant::one("Priest's Pack"),
];
const DRUID_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Leather Armor"),
    EquipmentGrant::one("Shield"),
    EquipmentGrant::one("Sickle"),
    EquipmentGrant::weapon("Druidic Focus (Quarterstaff)", "Quarterstaff"),
    EquipmentGrant::one("Explorer's Pack"),
    EquipmentGrant::one("Herbalism Kit"),
];
const FIGHTER_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Chain Mail"),
    EquipmentGrant::one("Greatsword"),
    EquipmentGrant::one("Flail"),
    EquipmentGrant::many("Javelin", 8),
    EquipmentGrant::one("Dungeoneer's Pack"),
];
const FIGHTER_B: &[EquipmentGrant] = &[
    EquipmentGrant::one("Studded Leather Armor"),
    EquipmentGrant::one("Scimitar"),
    EquipmentGrant::one("Shortsword"),
    EquipmentGrant::one("Longbow"),
    EquipmentGrant::many("Arrow", 20),
    EquipmentGrant::one("Quiver"),
    EquipmentGrant::one("Dungeoneer's Pack"),
];
const MONK_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Spear"),
    EquipmentGrant::many("Dagger", 5),
    EquipmentGrant::one("Chosen Monk Tool"),
    EquipmentGrant::one("Explorer's Pack"),
];
const PALADIN_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Chain Mail"),
    EquipmentGrant::one("Shield"),
    EquipmentGrant::one("Longsword"),
    EquipmentGrant::many("Javelin", 6),
    EquipmentGrant::one("Holy Symbol"),
    EquipmentGrant::one("Priest's Pack"),
];
const RANGER_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Studded Leather Armor"),
    EquipmentGrant::one("Scimitar"),
    EquipmentGrant::one("Shortsword"),
    EquipmentGrant::one("Longbow"),
    EquipmentGrant::many("Arrow", 20),
    EquipmentGrant::one("Quiver"),
    EquipmentGrant::one("Druidic Focus (Sprig of Mistletoe)"),
    EquipmentGrant::one("Explorer's Pack"),
];
const ROGUE_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Leather Armor"),
    EquipmentGrant::many("Dagger", 2),
    EquipmentGrant::one("Shortsword"),
    EquipmentGrant::one("Shortbow"),
    EquipmentGrant::many("Arrow", 20),
    EquipmentGrant::one("Quiver"),
    EquipmentGrant::one("Thieves' Tools"),
    EquipmentGrant::one("Burglar's Pack"),
];
const SORCERER_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Spear"),
    EquipmentGrant::many("Dagger", 2),
    EquipmentGrant::one("Arcane Focus (Crystal)"),
    EquipmentGrant::one("Dungeoneer's Pack"),
];
const WARLOCK_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Leather Armor"),
    EquipmentGrant::one("Sickle"),
    EquipmentGrant::many("Dagger", 2),
    EquipmentGrant::one("Arcane Focus (Orb)"),
    EquipmentGrant::one("Book (Occult Lore)"),
    EquipmentGrant::one("Scholar's Pack"),
];
const WIZARD_A: &[EquipmentGrant] = &[
    EquipmentGrant::many("Dagger", 2),
    EquipmentGrant::weapon("Arcane Focus (Quarterstaff)", "Quarterstaff"),
    EquipmentGrant::one("Robe"),
    EquipmentGrant::one("Spellbook"),
    EquipmentGrant::one("Scholar's Pack"),
];

#[must_use]
pub fn class_equipment(class: &str, option: &str) -> Option<(&'static [EquipmentGrant], u16)> {
    match (class, option) {
        ("Barbarian", "A") => Some((BARBARIAN_A, 15)),
        ("Bard", "A") => Some((BARD_A, 19)),
        ("Cleric", "A") => Some((CLERIC_A, 7)),
        ("Druid", "A") => Some((DRUID_A, 9)),
        ("Fighter", "A") => Some((FIGHTER_A, 4)),
        ("Fighter", "B") => Some((FIGHTER_B, 11)),
        ("Monk", "A") => Some((MONK_A, 11)),
        ("Paladin", "A") => Some((PALADIN_A, 9)),
        ("Ranger", "A") => Some((RANGER_A, 7)),
        ("Rogue", "A") => Some((ROGUE_A, 8)),
        ("Sorcerer", "A") => Some((SORCERER_A, 28)),
        ("Warlock", "A") => Some((WARLOCK_A, 15)),
        ("Wizard", "A") => Some((WIZARD_A, 5)),
        _ => None,
    }
}

#[must_use]
pub fn class_starting_gold(class: &str) -> Option<u16> {
    match class {
        "Barbarian" => Some(75),
        "Bard" => Some(90),
        "Cleric" => Some(110),
        "Druid" | "Monk" | "Sorcerer" => Some(50),
        "Fighter" => Some(155),
        "Paladin" | "Ranger" => Some(150),
        "Rogue" | "Warlock" => Some(100),
        "Wizard" => Some(55),
        _ => None,
    }
}

const ACOLYTE_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Calligrapher's Supplies"),
    EquipmentGrant::one("Book (Prayers)"),
    EquipmentGrant::one("Holy Symbol"),
    EquipmentGrant::many("Parchment", 10),
    EquipmentGrant::one("Robe"),
];
const CRIMINAL_A: &[EquipmentGrant] = &[
    EquipmentGrant::many("Dagger", 2),
    EquipmentGrant::one("Thieves' Tools"),
    EquipmentGrant::one("Crowbar"),
    EquipmentGrant::many("Pouch", 2),
    EquipmentGrant::one("Traveler's Clothes"),
];
const SAGE_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Quarterstaff"),
    EquipmentGrant::one("Calligrapher's Supplies"),
    EquipmentGrant::one("Book (History)"),
    EquipmentGrant::many("Parchment", 8),
    EquipmentGrant::one("Robe"),
];
const SOLDIER_A: &[EquipmentGrant] = &[
    EquipmentGrant::one("Spear"),
    EquipmentGrant::one("Shortbow"),
    EquipmentGrant::many("Arrow", 20),
    EquipmentGrant::one("Gaming Set"),
    EquipmentGrant::one("Healer's Kit"),
    EquipmentGrant::one("Quiver"),
    EquipmentGrant::one("Traveler's Clothes"),
];

#[must_use]
pub fn background_equipment(background: &str) -> Option<(&'static [EquipmentGrant], u16)> {
    match background {
        "Acolyte" => Some((ACOLYTE_A, 8)),
        "Criminal" => Some((CRIMINAL_A, 16)),
        "Sage" => Some((SAGE_A, 8)),
        "Soldier" => Some((SOLDIER_A, 14)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{armor_rule, class_equipment, weapon_rule};

    #[test]
    fn equipment_catalogs_return_structured_rules() {
        assert_eq!(
            armor_rule("Leather Armor").map(|rule| rule.base_ac),
            Some(11)
        );
        assert_eq!(weapon_rule("Dagger").map(|rule| rule.damage), Some("1d4"));
        assert!(class_equipment("Rogue", "A").is_some());
    }
}
