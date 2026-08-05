//! Closed SRD identifiers used by the canonical character record.

use std::{fmt, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

macro_rules! srd_id {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
        pub enum $name {
            $(#[serde(rename = $value)] $variant),+
        }

        impl $name {
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $value),+ }
            }
        }

        impl Deref for $name {
            type Target = str;
            fn deref(&self) -> &Self::Target { self.as_str() }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool { self.as_str() == *other }
        }

        impl FromStr for $name {
            type Err = String;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value { $($value => Ok(Self::$variant),)+ _ => Err(format!("unknown SRD {}: {value}", stringify!($name))) }
            }
        }
    };
}

srd_id!(ClassId {
    Barbarian => "Barbarian", Bard => "Bard", Cleric => "Cleric", Druid => "Druid",
    Fighter => "Fighter", Monk => "Monk", Paladin => "Paladin", Ranger => "Ranger",
    Rogue => "Rogue", Sorcerer => "Sorcerer", Warlock => "Warlock", Wizard => "Wizard",
});

srd_id!(BackgroundId {
    Acolyte => "Acolyte", Criminal => "Criminal", Sage => "Sage", Soldier => "Soldier",
});

srd_id!(SpeciesId {
    Dragonborn => "Dragonborn", Dwarf => "Dwarf", Elf => "Elf", Gnome => "Gnome",
    Goliath => "Goliath", Halfling => "Halfling", Human => "Human", Orc => "Orc",
    Tiefling => "Tiefling",
});

srd_id!(Size {
    Small => "Small", Medium => "Medium",
});
