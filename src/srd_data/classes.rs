//! Class and background rule catalogs.

use crate::srd_data::{BackgroundRule, ClassRule};

use super::core::{
    BARBARIAN_SKILLS, BARD_SKILLS, CLERIC_SKILLS, DRUID_SKILLS, FIGHTER_SKILLS, MONK_SKILLS,
    PALADIN_SKILLS, RANGER_SKILLS, ROGUE_SKILLS, SORCERER_SKILLS, WARLOCK_SKILLS, WIZARD_SKILLS,
};

#[must_use]
pub fn class_rule(name: &str) -> Option<ClassRule> {
    let rule = match name {
        "Barbarian" => ClassRule {
            hit_die: 12,
            saves: &["strength", "constitution"],
            skill_count: 2,
            skills: BARBARIAN_SKILLS,
            armor: "Light, Medium, Shields",
            weapons: "Simple and Martial",
        },
        "Bard" => ClassRule {
            hit_die: 8,
            saves: &["dexterity", "charisma"],
            skill_count: 3,
            skills: BARD_SKILLS,
            armor: "Light",
            weapons: "Simple",
        },
        "Cleric" => ClassRule {
            hit_die: 8,
            saves: &["wisdom", "charisma"],
            skill_count: 2,
            skills: CLERIC_SKILLS,
            armor: "Light, Medium, Shields",
            weapons: "Simple",
        },
        "Druid" => ClassRule {
            hit_die: 8,
            saves: &["intelligence", "wisdom"],
            skill_count: 2,
            skills: DRUID_SKILLS,
            armor: "Light, Shields",
            weapons: "Simple",
        },
        "Fighter" => ClassRule {
            hit_die: 10,
            saves: &["strength", "constitution"],
            skill_count: 2,
            skills: FIGHTER_SKILLS,
            armor: "Light, Medium, Heavy, Shields",
            weapons: "Simple and Martial",
        },
        "Monk" => ClassRule {
            hit_die: 8,
            saves: &["strength", "dexterity"],
            skill_count: 2,
            skills: MONK_SKILLS,
            armor: "None",
            weapons: "Simple and Martial weapons with Light property",
        },
        "Paladin" => ClassRule {
            hit_die: 10,
            saves: &["wisdom", "charisma"],
            skill_count: 2,
            skills: PALADIN_SKILLS,
            armor: "Light, Medium, Heavy, Shields",
            weapons: "Simple and Martial",
        },
        "Ranger" => ClassRule {
            hit_die: 10,
            saves: &["strength", "dexterity"],
            skill_count: 3,
            skills: RANGER_SKILLS,
            armor: "Light, Medium, Shields",
            weapons: "Simple and Martial",
        },
        "Rogue" => ClassRule {
            hit_die: 8,
            saves: &["dexterity", "intelligence"],
            skill_count: 4,
            skills: ROGUE_SKILLS,
            armor: "Light",
            weapons: "Simple and Martial weapons with Finesse or Light property",
        },
        "Sorcerer" => ClassRule {
            hit_die: 6,
            saves: &["constitution", "charisma"],
            skill_count: 2,
            skills: SORCERER_SKILLS,
            armor: "None",
            weapons: "Simple",
        },
        "Warlock" => ClassRule {
            hit_die: 8,
            saves: &["wisdom", "charisma"],
            skill_count: 2,
            skills: WARLOCK_SKILLS,
            armor: "Light",
            weapons: "Simple",
        },
        "Wizard" => ClassRule {
            hit_die: 6,
            saves: &["intelligence", "wisdom"],
            skill_count: 2,
            skills: WIZARD_SKILLS,
            armor: "None",
            weapons: "Simple",
        },
        _ => return None,
    };
    Some(rule)
}

#[must_use]
pub fn suggested_array(class: &str) -> Option<[u8; 6]> {
    match class {
        "Barbarian" => Some([15, 13, 14, 10, 12, 8]),
        "Bard" => Some([8, 14, 12, 13, 10, 15]),
        "Cleric" => Some([14, 8, 13, 10, 15, 12]),
        "Druid" => Some([8, 12, 14, 13, 15, 10]),
        "Fighter" => Some([15, 14, 13, 8, 10, 12]),
        "Monk" => Some([12, 15, 13, 10, 14, 8]),
        "Paladin" => Some([15, 10, 13, 8, 12, 14]),
        "Ranger" => Some([12, 15, 13, 8, 14, 10]),
        "Rogue" => Some([12, 15, 13, 14, 10, 8]),
        "Sorcerer" => Some([10, 13, 14, 8, 12, 15]),
        "Warlock" => Some([8, 14, 13, 12, 10, 15]),
        "Wizard" => Some([8, 12, 13, 15, 14, 10]),
        _ => None,
    }
}

#[must_use]
pub const fn point_buy_cost(score: u8) -> Option<u8> {
    match score {
        8 => Some(0),
        9 => Some(1),
        10 => Some(2),
        11 => Some(3),
        12 => Some(4),
        13 => Some(5),
        14 => Some(7),
        15 => Some(9),
        _ => None,
    }
}

#[must_use]
pub fn background_rule(name: &str) -> Option<BackgroundRule> {
    let rule = match name {
        "Acolyte" => BackgroundRule {
            abilities: &["intelligence", "wisdom", "charisma"],
            feat: "Magic Initiate",
            skills: &["Insight", "Religion"],
            tool: "Calligrapher's Supplies",
            magic_initiate_list: Some("Cleric"),
        },
        "Criminal" => BackgroundRule {
            abilities: &["dexterity", "constitution", "intelligence"],
            feat: "Alert",
            skills: &["Sleight of Hand", "Stealth"],
            tool: "Thieves' Tools",
            magic_initiate_list: None,
        },
        "Sage" => BackgroundRule {
            abilities: &["constitution", "intelligence", "wisdom"],
            feat: "Magic Initiate",
            skills: &["Arcana", "History"],
            tool: "Calligrapher's Supplies",
            magic_initiate_list: Some("Wizard"),
        },
        "Soldier" => BackgroundRule {
            abilities: &["strength", "dexterity", "constitution"],
            feat: "Savage Attacker",
            skills: &["Athletics", "Intimidation"],
            tool: "Gaming Set",
            magic_initiate_list: None,
        },
        _ => return None,
    };
    Some(rule)
}

#[must_use]
pub fn weapon_mastery_count(class: &str) -> usize {
    match class {
        "Barbarian" | "Paladin" | "Ranger" | "Rogue" => 2,
        "Fighter" => 3,
        _ => 0,
    }
}

#[must_use]
pub fn class_features(class: &str) -> &'static [&'static str] {
    match class {
        "Barbarian" => &["Rage", "Unarmored Defense", "Weapon Mastery"],
        "Bard" => &["Bardic Inspiration", "Spellcasting"],
        "Cleric" => &["Divine Order", "Spellcasting"],
        "Druid" => &["Druidic", "Primal Order", "Spellcasting"],
        "Fighter" => &["Fighting Style", "Second Wind", "Weapon Mastery"],
        "Monk" => &["Martial Arts", "Unarmored Defense"],
        "Paladin" => &["Lay on Hands", "Spellcasting", "Weapon Mastery"],
        "Ranger" => &["Favored Enemy", "Spellcasting", "Weapon Mastery"],
        "Rogue" => &[
            "Expertise",
            "Sneak Attack",
            "Thieves' Cant",
            "Weapon Mastery",
        ],
        "Sorcerer" => &["Innate Sorcery", "Spellcasting"],
        "Warlock" => &["Eldritch Invocations", "Pact Magic"],
        "Wizard" => &["Arcane Recovery", "Ritual Adept", "Spellcasting"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::{background_rule, class_rule};

    #[test]
    fn class_and_background_catalogs_return_typed_rules() {
        assert_eq!(class_rule("Fighter").map(|rule| rule.hit_die), Some(10));
        assert_eq!(
            background_rule("Criminal").map(|rule| rule.feat),
            Some("Alert")
        );
    }
}
