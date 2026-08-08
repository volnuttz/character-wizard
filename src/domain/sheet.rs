//! Read-only character-sheet projection.
//!
//! Renderers use this boundary instead of querying SRD catalog data themselves.

use crate::Character;

/// A calculated saving throw suitable for presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavingThrow {
    /// Whether the character is proficient with this saving throw.
    pub proficient: bool,
    /// The total modifier, including proficiency when applicable.
    pub modifier: i16,
}

/// Read-only, SRD-derived values required by character-sheet adapters.
#[derive(Debug, Clone, Copy)]
pub struct CharacterSheet<'a> {
    character: &'a Character,
}

impl<'a> CharacterSheet<'a> {
    pub(crate) const fn new(character: &'a Character) -> Self {
        Self { character }
    }

    /// Return the class hit die.
    #[must_use]
    pub fn hit_die(self) -> u8 {
        self.character.class_hit_die()
    }

    /// Calculate one ability saving throw.
    #[must_use]
    pub fn saving_throw(self, ability: &str) -> SavingThrow {
        let proficient = self.character.class_saving_throws().contains(&ability);
        let modifier = self.character.abilities.modifier(ability)
            + if proficient {
                i16::from(self.character.proficiency_bonus())
            } else {
                0
            };
        SavingThrow {
            proficient,
            modifier,
        }
    }
}
