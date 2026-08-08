//! Validated runtime rule records supplied by external data packs.

use serde::{Deserialize, Serialize};

use crate::domain::Size;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackSpecies {
    pub id: String,
    pub name: String,
    pub sizes: Vec<Size>,
    pub speed: u8,
    #[serde(default)]
    pub darkvision_range: Option<u8>,
    #[serde(default)]
    pub traits: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackBackground {
    pub id: String,
    pub name: String,
    pub abilities: Vec<String>,
    pub skills: Vec<String>,
    pub feat: String,
    pub tool: String,
    #[serde(default)]
    pub magic_initiate_list: Option<String>,
    pub equipment: Vec<PackEquipmentGrant>,
    #[serde(default)]
    pub equipment_gold: u16,
    #[serde(default = "default_background_gold_alternative")]
    pub gold_alternative: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackClass {
    pub id: String,
    pub name: String,
    pub hit_die: u8,
    pub saving_throws: Vec<String>,
    pub skill_count: usize,
    pub skills: Vec<String>,
    #[serde(default)]
    pub armor_training: Vec<String>,
    #[serde(default)]
    pub weapon_training: Vec<String>,
    pub equipment: Vec<PackEquipmentGrant>,
    #[serde(default)]
    pub equipment_gold: u16,
    pub starting_gold: u16,
    pub features: Vec<String>,
    #[serde(default)]
    pub weapon_mastery_count: usize,
    #[serde(default)]
    pub choices: Vec<PackClassChoice>,
    #[serde(default)]
    pub resources: Vec<PackClassResource>,
    #[serde(default)]
    pub spellcasting: Option<PackClassSpellcasting>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackClassChoice {
    pub id: String,
    pub label: String,
    pub count: usize,
    pub options: Vec<PackClassChoiceOption>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackClassChoiceOption {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackClassResource {
    pub name: String,
    pub maximum: i16,
    pub unit: String,
    pub recovery: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackClassSpellcasting {
    pub ability: String,
    pub spell_list: String,
    #[serde(default)]
    pub cantrip_count: usize,
    #[serde(default)]
    pub prepared_spell_count: usize,
    #[serde(default)]
    pub spell_slots: u8,
    pub slot_recovery: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackEquipmentGrant {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub equipment_id: Option<String>,
    #[serde(default = "default_equipment_quantity")]
    pub quantity: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackEquipment {
    pub id: String,
    pub name: String,
    pub kind: PackEquipmentKind,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum PackEquipmentKind {
    Gear,
    Ammunition,
    Shield {
        armor_class_bonus: i16,
    },
    Armor {
        category: String,
        base_ac: i16,
        #[serde(default)]
        dexterity_cap: Option<i16>,
        #[serde(default)]
        strength_requirement: Option<u8>,
    },
    Weapon {
        category: String,
        kind: String,
        #[serde(default)]
        properties: Vec<String>,
        mastery: String,
        damage: String,
        damage_type: String,
        normal_range: u16,
        #[serde(default)]
        long_range: Option<u16>,
        #[serde(default)]
        versatile_damage: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackSpell {
    pub id: String,
    pub name: String,
    pub level: u8,
    pub school: String,
    pub lists: Vec<String>,
    pub casting_time: String,
    pub range: String,
    pub components: Vec<String>,
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub concentration: bool,
    #[serde(default)]
    pub ritual: bool,
    pub notes: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

const fn default_equipment_quantity() -> u16 {
    1
}

const fn default_background_gold_alternative() -> u16 {
    50
}
