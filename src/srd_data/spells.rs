//! SRD 5.2.1-derived level-1 catalog and lookup helpers.
//!
//! The checked-in spell catalog is the normative source for level-1 rules.

use std::{collections::BTreeMap, sync::OnceLock};

use crate::srd_data::{SpellList, SpellRule};

#[must_use]
/// Return structured sheet metadata for an SRD creation spell.
///
/// # Panics
///
/// Panics only if the checked-in SRD spell data is malformed.
pub fn spell_rule(name: &str) -> Option<&'static SpellRule> {
    static RULES: OnceLock<BTreeMap<String, SpellRule>> = OnceLock::new();
    RULES
        .get_or_init(|| {
            serde_json::from_str(include_str!("../../data/spells.json"))
                .expect("checked SRD spell data is valid")
        })
        .get(name)
}

const CLERIC_CANTRIPS: &[&str] = &[
    "Guidance",
    "Light",
    "Mending",
    "Resistance",
    "Sacred Flame",
    "Spare the Dying",
    "Thaumaturgy",
];
const CLERIC_SPELLS: &[&str] = &[
    "Bane",
    "Bless",
    "Command",
    "Create or Destroy Water",
    "Cure Wounds",
    "Detect Evil and Good",
    "Detect Magic",
    "Detect Poison and Disease",
    "Guiding Bolt",
    "Healing Word",
    "Inflict Wounds",
    "Protection from Evil and Good",
    "Purify Food and Drink",
    "Sanctuary",
    "Shield of Faith",
];
const DRUID_CANTRIPS: &[&str] = &[
    "Druidcraft",
    "Elementalism",
    "Guidance",
    "Mending",
    "Message",
    "Poison Spray",
    "Produce Flame",
    "Resistance",
    "Shillelagh",
    "Spare the Dying",
    "Starry Wisp",
];
const DRUID_SPELLS: &[&str] = &[
    "Animal Friendship",
    "Charm Person",
    "Create or Destroy Water",
    "Cure Wounds",
    "Detect Magic",
    "Detect Poison and Disease",
    "Entangle",
    "Faerie Fire",
    "Fog Cloud",
    "Goodberry",
    "Healing Word",
    "Ice Knife",
    "Jump",
    "Longstrider",
    "Protection from Evil and Good",
    "Purify Food and Drink",
    "Speak with Animals",
    "Thunderwave",
];
const WIZARD_CANTRIPS: &[&str] = &[
    "Acid Splash",
    "Chill Touch",
    "Dancing Lights",
    "Elementalism",
    "Fire Bolt",
    "Light",
    "Mage Hand",
    "Mending",
    "Message",
    "Minor Illusion",
    "Poison Spray",
    "Prestidigitation",
    "Ray of Frost",
    "Shocking Grasp",
    "True Strike",
];
const WIZARD_SPELLS: &[&str] = &[
    "Alarm",
    "Burning Hands",
    "Charm Person",
    "Chromatic Orb",
    "Color Spray",
    "Comprehend Languages",
    "Detect Magic",
    "Disguise Self",
    "Expeditious Retreat",
    "False Life",
    "Feather Fall",
    "Find Familiar",
    "Floating Disk",
    "Fog Cloud",
    "Grease",
    "Hideous Laughter",
    "Ice Knife",
    "Identify",
    "Illusory Script",
    "Jump",
    "Longstrider",
    "Mage Armor",
    "Magic Missile",
    "Protection from Evil and Good",
    "Ray of Sickness",
    "Shield",
    "Silent Image",
    "Sleep",
    "Thunderwave",
    "Unseen Servant",
];
const BARD_CANTRIPS: &[&str] = &[
    "Dancing Lights",
    "Light",
    "Mage Hand",
    "Mending",
    "Message",
    "Minor Illusion",
    "Prestidigitation",
    "Starry Wisp",
    "True Strike",
    "Vicious Mockery",
];
const BARD_SPELLS: &[&str] = &[
    "Animal Friendship",
    "Bane",
    "Charm Person",
    "Color Spray",
    "Command",
    "Comprehend Languages",
    "Cure Wounds",
    "Detect Magic",
    "Disguise Self",
    "Dissonant Whispers",
    "Faerie Fire",
    "Feather Fall",
    "Healing Word",
    "Heroism",
    "Hideous Laughter",
    "Identify",
    "Illusory Script",
    "Longstrider",
    "Silent Image",
    "Sleep",
    "Speak with Animals",
    "Thunderwave",
    "Unseen Servant",
];
const PALADIN_SPELLS: &[&str] = &[
    "Bless",
    "Command",
    "Cure Wounds",
    "Detect Evil and Good",
    "Detect Magic",
    "Detect Poison and Disease",
    "Divine Favor",
    "Divine Smite",
    "Heroism",
    "Protection from Evil and Good",
    "Purify Food and Drink",
    "Searing Smite",
    "Shield of Faith",
];
const RANGER_SPELLS: &[&str] = &[
    "Alarm",
    "Animal Friendship",
    "Cure Wounds",
    "Detect Magic",
    "Detect Poison and Disease",
    "Ensnaring Strike",
    "Entangle",
    "Fog Cloud",
    "Goodberry",
    "Hunter's Mark",
    "Jump",
    "Longstrider",
    "Speak with Animals",
];
const SORCERER_CANTRIPS: &[&str] = &[
    "Acid Splash",
    "Chill Touch",
    "Dancing Lights",
    "Elementalism",
    "Fire Bolt",
    "Light",
    "Mage Hand",
    "Mending",
    "Message",
    "Minor Illusion",
    "Poison Spray",
    "Prestidigitation",
    "Ray of Frost",
    "Shocking Grasp",
    "Sorcerous Burst",
    "True Strike",
];
const SORCERER_SPELLS: &[&str] = &[
    "Burning Hands",
    "Charm Person",
    "Chromatic Orb",
    "Color Spray",
    "Comprehend Languages",
    "Detect Magic",
    "Disguise Self",
    "Expeditious Retreat",
    "False Life",
    "Feather Fall",
    "Fog Cloud",
    "Grease",
    "Ice Knife",
    "Jump",
    "Mage Armor",
    "Magic Missile",
    "Ray of Sickness",
    "Shield",
    "Silent Image",
    "Sleep",
    "Thunderwave",
];
const WARLOCK_CANTRIPS: &[&str] = &[
    "Chill Touch",
    "Eldritch Blast",
    "Mage Hand",
    "Minor Illusion",
    "Poison Spray",
    "Prestidigitation",
    "True Strike",
];
const WARLOCK_SPELLS: &[&str] = &[
    "Bane",
    "Charm Person",
    "Comprehend Languages",
    "Detect Magic",
    "Expeditious Retreat",
    "Hellish Rebuke",
    "Hex",
    "Hideous Laughter",
    "Illusory Script",
    "Protection from Evil and Good",
    "Speak with Animals",
    "Unseen Servant",
];

#[must_use]
pub fn magic_initiate_spell_list(name: &str) -> Option<SpellList> {
    match name {
        "Cleric" => Some(SpellList {
            cantrips: CLERIC_CANTRIPS,
            level_one_spells: CLERIC_SPELLS,
        }),
        "Druid" => Some(SpellList {
            cantrips: DRUID_CANTRIPS,
            level_one_spells: DRUID_SPELLS,
        }),
        "Wizard" => Some(SpellList {
            cantrips: WIZARD_CANTRIPS,
            level_one_spells: WIZARD_SPELLS,
        }),
        _ => None,
    }
}

#[must_use]
pub fn class_spell_list(name: &str) -> Option<SpellList> {
    match name {
        "Bard" => Some(SpellList {
            cantrips: BARD_CANTRIPS,
            level_one_spells: BARD_SPELLS,
        }),
        "Cleric" => magic_initiate_spell_list("Cleric"),
        "Druid" => magic_initiate_spell_list("Druid"),
        "Paladin" => Some(SpellList {
            cantrips: &[],
            level_one_spells: PALADIN_SPELLS,
        }),
        "Ranger" => Some(SpellList {
            cantrips: &[],
            level_one_spells: RANGER_SPELLS,
        }),
        "Sorcerer" => Some(SpellList {
            cantrips: SORCERER_CANTRIPS,
            level_one_spells: SORCERER_SPELLS,
        }),
        "Warlock" => Some(SpellList {
            cantrips: WARLOCK_CANTRIPS,
            level_one_spells: WARLOCK_SPELLS,
        }),
        "Wizard" => magic_initiate_spell_list("Wizard"),
        _ => None,
    }
}

#[must_use]
pub fn class_always_prepared(name: &str) -> &'static [&'static str] {
    match name {
        "Druid" => &["Speak with Animals"],
        "Ranger" => &["Hunter's Mark"],
        _ => &[],
    }
}

#[must_use]
pub fn class_spellcasting_ability(name: &str) -> Option<&'static str> {
    match name {
        "Bard" | "Sorcerer" | "Warlock" | "Paladin" => Some("charisma"),
        "Cleric" | "Druid" | "Ranger" => Some("wisdom"),
        "Wizard" => Some("intelligence"),
        _ => None,
    }
}

#[must_use]
pub fn level_one_spell_slots(name: &str) -> Option<(u8, u8, &'static str)> {
    match name {
        "Warlock" => Some((1, 1, "Short or Long Rest")),
        "Bard" | "Cleric" | "Druid" | "Paladin" | "Ranger" | "Sorcerer" | "Wizard" => {
            Some((1, 2, "Long Rest"))
        }
        _ => None,
    }
}

#[must_use]
pub fn dragonborn_damage_type(ancestry: &str) -> Option<&'static str> {
    match ancestry {
        "Black" | "Copper" => Some("Acid"),
        "Blue" | "Bronze" => Some("Lightning"),
        "Brass" | "Gold" | "Red" => Some("Fire"),
        "Green" => Some("Poison"),
        "Silver" | "White" => Some("Cold"),
        _ => None,
    }
}

#[must_use]
pub fn tiefling_resistance(legacy: &str) -> Option<&'static str> {
    match legacy {
        "Abyssal" => Some("Poison"),
        "Chthonic" => Some("Necrotic"),
        "Infernal" => Some("Fire"),
        _ => None,
    }
}

#[must_use]
pub fn tiefling_cantrip(legacy: &str) -> Option<&'static str> {
    match legacy {
        "Abyssal" => Some("Poison Spray"),
        "Chthonic" => Some("Chill Touch"),
        "Infernal" => Some("Fire Bolt"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{class_spell_list, spell_rule};

    #[test]
    fn spell_catalog_exposes_lists_and_metadata() {
        assert!(class_spell_list("Wizard").is_some());
        assert_eq!(
            spell_rule("Magic Missile").map(|rule| rule.casting_time.as_str()),
            Some("Action")
        );
    }
}
