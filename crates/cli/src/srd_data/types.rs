//! Typed records returned by the SRD catalog.

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassRule {
    pub hit_die: u8,
    pub saves: &'static [&'static str],
    pub skill_count: usize,
    pub skills: &'static [&'static str],
    pub armor: &'static str,
    pub weapons: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundRule {
    pub abilities: &'static [&'static str],
    pub feat: &'static str,
    pub skills: &'static [&'static str],
    pub tool: &'static str,
    pub magic_initiate_list: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeciesRule {
    pub sizes: &'static [&'static str],
    pub speed: u8,
    pub darkvision_range: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquipmentGrant {
    pub name: &'static str,
    pub quantity: u16,
    pub weapon: Option<&'static str>,
}

impl EquipmentGrant {
    pub(crate) const fn one(name: &'static str) -> Self {
        Self {
            name,
            quantity: 1,
            weapon: None,
        }
    }

    pub(crate) const fn many(name: &'static str, quantity: u16) -> Self {
        Self {
            name,
            quantity,
            weapon: None,
        }
    }

    pub(crate) const fn weapon(name: &'static str, weapon: &'static str) -> Self {
        Self {
            name,
            quantity: 1,
            weapon: Some(weapon),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmorRule {
    pub category: &'static str,
    pub base_ac: i16,
    pub dexterity_cap: Option<i16>,
    pub strength_requirement: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponRule {
    pub category: &'static str,
    pub kind: &'static str,
    pub properties: &'static [&'static str],
    pub mastery: &'static str,
    pub damage: &'static str,
    pub damage_type: &'static str,
    pub normal_range: u16,
    pub long_range: Option<u16>,
    pub versatile_damage: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellList {
    pub cantrips: &'static [&'static str],
    pub level_one_spells: &'static [&'static str],
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SpellRule {
    pub casting_time: String,
    pub range: String,
    pub concentration: bool,
    pub ritual: bool,
    pub required_material: Option<String>,
    pub notes: String,
}
