//! Unified access to built-in SRD rules and optional validated campaign content.

use crate::{
    character_wizard_domain::{
        Character, DataPackReference, PackBackground, PackClass, PackEquipment, PackSpecies,
        PackSpell, ResolvedCharacter,
    },
    character_wizard_srd_data as srd,
};

/// Immutable rule catalog used by creation, validation, and rendering orchestration.
#[derive(Debug, Clone, Copy)]
pub struct RulesContext<'a> {
    reference: Option<&'a DataPackReference>,
    name: Option<&'a str>,
    classes: &'a [PackClass],
    species: &'a [PackSpecies],
    backgrounds: &'a [PackBackground],
    equipment: &'a [PackEquipment],
    spells: &'a [PackSpell],
}

impl RulesContext<'static> {
    #[must_use]
    pub const fn srd() -> Self {
        Self {
            reference: None,
            name: None,
            classes: &[],
            species: &[],
            backgrounds: &[],
            equipment: &[],
            spells: &[],
        }
    }
}

impl<'a> RulesContext<'a> {
    #[must_use]
    pub const fn with_pack(
        reference: &'a DataPackReference,
        name: &'a str,
        classes: &'a [PackClass],
        species: &'a [PackSpecies],
        backgrounds: &'a [PackBackground],
        equipment: &'a [PackEquipment],
        spells: &'a [PackSpell],
    ) -> Self {
        Self {
            reference: Some(reference),
            name: Some(name),
            classes,
            species,
            backgrounds,
            equipment,
            spells,
        }
    }

    #[must_use]
    pub const fn reference(self) -> Option<&'a DataPackReference> {
        self.reference
    }

    #[must_use]
    pub const fn name(self) -> Option<&'a str> {
        self.name
    }

    #[must_use]
    pub const fn classes(self) -> &'a [PackClass] {
        self.classes
    }

    #[must_use]
    pub const fn species(self) -> &'a [PackSpecies] {
        self.species
    }

    #[must_use]
    pub const fn backgrounds(self) -> &'a [PackBackground] {
        self.backgrounds
    }

    #[must_use]
    pub const fn equipment(self) -> &'a [PackEquipment] {
        self.equipment
    }

    #[must_use]
    pub const fn spells(self) -> &'a [PackSpell] {
        self.spells
    }

    #[must_use]
    pub fn custom_class(self, value: &str) -> Option<&'a PackClass> {
        self.classes.iter().find(|rule| {
            rule.id.eq_ignore_ascii_case(value) || rule.name.eq_ignore_ascii_case(value)
        })
    }

    #[must_use]
    pub fn custom_background(self, value: &str) -> Option<&'a PackBackground> {
        self.backgrounds.iter().find(|rule| {
            rule.id.eq_ignore_ascii_case(value) || rule.name.eq_ignore_ascii_case(value)
        })
    }

    #[must_use]
    pub fn custom_species(self, value: &str) -> Option<&'a PackSpecies> {
        self.species.iter().find(|rule| {
            rule.id.eq_ignore_ascii_case(value) || rule.name.eq_ignore_ascii_case(value)
        })
    }

    /// Attach the exact custom mechanics required by a canonical character.
    ///
    /// # Errors
    ///
    /// Returns an error when provenance differs or referenced content is unavailable.
    pub fn resolve(self, mut character: Character) -> Result<ResolvedCharacter, String> {
        match (&character.data_pack, self.reference) {
            (None, _) => {}
            (Some(required), Some(actual)) if required == actual => {}
            (Some(required), Some(_)) => {
                return Err(format!(
                    "character requires data pack {} version {} (format version {})",
                    required.id, required.version, required.format_version
                ));
            }
            (Some(required), None) => {
                return Err(format!(
                    "character requires data pack {}; pass --data <directory>",
                    required.id
                ));
            }
        }
        character.resolve_pack_class(self.classes)?;
        character.resolve_pack_background(self.backgrounds)?;
        character.resolve_pack_equipment(self.equipment)?;
        character.resolve_pack_spells(self.spells)?;
        character.resolve_pack_species(if srd::species_rule(&character.species).is_some() {
            &[]
        } else {
            self.species
        })?;
        character.validate()?;
        Ok(ResolvedCharacter(character))
    }
}
