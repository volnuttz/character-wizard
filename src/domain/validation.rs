//! Canonical character parsing and cross-field validation.

use std::collections::BTreeSet;

use crate::character_wizard_srd_data as srd;

use crate::domain::CharacterSheet;

use super::{
    content::{PackBackground, PackClass, PackEquipment, PackSpecies, PackSpell},
    record::Character,
};

impl Character {
    /// Resolve the exact custom class referenced by this character.
    ///
    /// # Errors
    ///
    /// Returns an error when the class ID is absent or its resolved choices are invalid.
    pub fn resolve_pack_class(&mut self, classes: &[PackClass]) -> Result<(), String> {
        self.resolved_pack_class = if srd::class_rule(&self.character_class).is_some() {
            None
        } else {
            Some(
                classes
                    .iter()
                    .find(|rule| rule.id == self.character_class.as_str())
                    .cloned()
                    .ok_or_else(|| {
                        format!("unknown class in data pack: {}", self.character_class)
                    })?,
            )
        };
        self.validate()
    }

    /// Resolve custom spells used by class and Origin-feat choices.
    ///
    /// # Errors
    ///
    /// Returns an error when a selected spell is absent from the exact pack or
    /// is not eligible for its selected list and level.
    pub fn resolve_pack_spells(&mut self, spells: &[PackSpell]) -> Result<(), String> {
        self.resolved_pack_spells = self.data_pack.as_ref().map(|_| spells.to_vec());
        self.validate()
    }

    fn unresolved_pack_spell(&self, reference: &str) -> bool {
        self.data_pack.is_some()
            && self.resolved_pack_spells.is_none()
            && srd::spell_rule(reference).is_none()
    }

    /// Resolve the exact custom-equipment catalog referenced by this character.
    ///
    /// # Errors
    ///
    /// Returns an error when a custom background grant references a missing item
    /// or the resolved character is invalid.
    pub fn resolve_pack_equipment(&mut self, equipment: &[PackEquipment]) -> Result<(), String> {
        if let Some(background) = &self.resolved_pack_background {
            for grant in &background.equipment {
                if let Some(id) = &grant.equipment_id
                    && !equipment.iter().any(|item| item.id == *id)
                {
                    return Err(format!("unknown equipment in data pack: {id}"));
                }
            }
        }
        self.resolved_pack_equipment = equipment.to_vec();
        self.validate()
    }

    /// Resolve external background mechanics for this character.
    ///
    /// # Errors
    ///
    /// Returns an error when a referenced background is absent or its choices
    /// make the completed character invalid.
    pub fn resolve_pack_background(
        &mut self,
        backgrounds: &[PackBackground],
    ) -> Result<(), String> {
        if srd::background_rule(&self.background).is_some() {
            self.resolved_pack_background = None;
        } else {
            let rule = backgrounds
                .iter()
                .find(|rule| rule.id == self.background.as_str())
                .ok_or_else(|| format!("unknown background in data pack: {}", self.background))?;
            self.resolved_pack_background = Some(rule.clone());
        }
        self.validate()
    }

    /// Resolve and validate external species mechanics for this character.
    ///
    /// # Errors
    ///
    /// Returns an error when a pack species ID is missing or its size is invalid.
    pub fn resolve_pack_species(&mut self, species: &[PackSpecies]) -> Result<(), String> {
        if srd::species_rule(&self.species).is_some() {
            self.resolved_pack_species = None;
            return Ok(());
        }
        let rule = species
            .iter()
            .find(|rule| rule.id == self.species.as_str())
            .ok_or_else(|| format!("unknown species in data pack: {}", self.species))?;
        if !rule.sizes.contains(&self.size) {
            return Err(format!(
                "invalid size for pack species {}: {}",
                rule.id, self.size
            ));
        }
        self.resolved_pack_species = Some(rule.clone());
        Ok(())
    }

    /// Return calculated values intended for character-sheet adapters.
    #[must_use]
    pub const fn sheet(&self) -> CharacterSheet<'_> {
        CharacterSheet::new(self)
    }

    /// Parse and structurally validate the complete canonical v1 character record.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON, unknown fields, missing required data,
    /// or representative scalar constraints outside their allowed bounds.
    pub fn from_json(source: &str) -> Result<Self, String> {
        let mut character: Self =
            serde_json::from_str(source).map_err(|error| error.to_string())?;
        for detail in [
            &mut character.backstory,
            &mut character.appearance,
            &mut character.personality,
        ] {
            if detail
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                *detail = None;
            } else if let Some(value) = detail {
                *value = value.trim().to_owned();
            }
        }
        character.validate()?;
        Ok(character)
    }

    /// Validate the canonical record using any runtime-resolved pack mechanics.
    ///
    /// # Errors
    ///
    /// Returns the first structural or cross-field validation error.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("name must not be empty".to_owned());
        }
        if !(1..=20).contains(&self.level) {
            return Err("level must be between 1 and 20".to_owned());
        }
        self.abilities.validate()?;
        if self.selected_languages[0] == self.selected_languages[1] {
            return Err("choose two different standard languages".to_owned());
        }
        self.validate_closed_values()?;
        self.validate_species_choices()?;
        if srd::background_rule(&self.background).is_none()
            && self.resolved_pack_background.is_none()
        {
            // Pack-dependent cross-field checks run after the CLI resolves the
            // exact referenced pack revision.
            return Ok(());
        }
        self.validate_origin_choices()?;
        if srd::class_rule(&self.character_class).is_none() && self.resolved_pack_class.is_none() {
            return Ok(());
        }
        self.validate_class_choices()?;
        self.validate_equipment_choices()
    }

    fn validate_closed_values(&self) -> Result<(), String> {
        if !["Small", "Medium"].contains(&self.size.as_str()) {
            return Err(format!("invalid size: {}", self.size));
        }
        if !srd::ALIGNMENTS.contains(&self.alignment.as_str()) {
            return Err(format!("invalid alignment: {}", self.alignment));
        }
        if srd::class_rule(&self.character_class).is_none()
            && self.resolved_pack_class.is_none()
            && self.data_pack.is_none()
        {
            return Err(format!("unknown SRD class: {}", self.character_class));
        }
        if srd::background_rule(&self.background).is_none()
            && self.resolved_pack_background.is_none()
            && self.data_pack.is_none()
        {
            return Err(format!("unknown SRD background: {}", self.background));
        }
        if srd::species_rule(&self.species).is_none() && self.data_pack.is_none() {
            return Err(format!("unknown SRD species: {}", self.species));
        }
        if self
            .selected_languages
            .iter()
            .any(|value| !srd::STANDARD_LANGUAGES.contains(&value.as_str()))
        {
            return Err("invalid standard language".to_owned());
        }
        if self.class_equipment_option != "Gold"
            && !self
                .class_equipment_option
                .bytes()
                .all(|value| value.is_ascii_uppercase())
        {
            return Err("invalid class starting-equipment option".to_owned());
        }
        if !["A", "Gold"].contains(&self.background_equipment_option.as_str()) {
            return Err("invalid background starting-equipment option".to_owned());
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn validate_species_choices(&self) -> Result<(), String> {
        let Some(species) = srd::species_rule(&self.species) else {
            if self.dragonborn_ancestry.is_some()
                || self.elf_lineage.is_some()
                || self.elf_spellcasting_ability.is_some()
                || self.elf_keen_senses_skill.is_some()
                || self.gnome_lineage.is_some()
                || self.gnome_spellcasting_ability.is_some()
                || self.goliath_ancestry.is_some()
                || self.human_skill.is_some()
                || self.human_origin_feat.is_some()
                || self.tiefling_legacy.is_some()
                || self.tiefling_spellcasting_ability.is_some()
            {
                return Err("SRD species subchoices are invalid for pack species".to_owned());
            }
            return Ok(());
        };
        if !species.sizes.contains(&self.size.as_str()) {
            return Err(format!("invalid size for {}: {}", self.species, self.size));
        }
        validate_required_only(
            self.species == "Dragonborn",
            self.dragonborn_ancestry.is_some(),
            "Dragonborn characters must choose a draconic ancestry",
            "draconic ancestry is only valid for Dragonborn characters",
        )?;
        if let Some(value) = &self.dragonborn_ancestry
            && ![
                "Black", "Blue", "Brass", "Bronze", "Copper", "Gold", "Green", "Red", "Silver",
                "White",
            ]
            .contains(&value.as_str())
        {
            return Err("invalid draconic ancestry".to_owned());
        }
        let elf_choices = self.elf_lineage.is_some()
            && self.elf_spellcasting_ability.is_some()
            && self.elf_keen_senses_skill.is_some();
        let any_elf = self.elf_lineage.is_some()
            || self.elf_spellcasting_ability.is_some()
            || self.elf_keen_senses_skill.is_some();
        if self.species == "Elf" && !elf_choices {
            return Err(
                "Elf characters must choose a lineage, spellcasting ability, and Keen Senses skill"
                    .to_owned(),
            );
        }
        if self.species != "Elf" && any_elf {
            return Err("Elf lineage choices are only valid for Elf characters".to_owned());
        }
        if let Some(value) = &self.elf_lineage
            && !["Drow", "High Elf", "Wood Elf"].contains(&value.as_str())
        {
            return Err("invalid Elven lineage".to_owned());
        }
        if let Some(value) = &self.elf_keen_senses_skill
            && !["Insight", "Perception", "Survival"].contains(&value.as_str())
        {
            return Err("invalid Keen Senses skill".to_owned());
        }
        let all_gnome = self.gnome_lineage.is_some() && self.gnome_spellcasting_ability.is_some();
        let any_gnome = self.gnome_lineage.is_some() || self.gnome_spellcasting_ability.is_some();
        if self.species == "Gnome" && !all_gnome {
            return Err(
                "Gnome characters must choose a Gnomish Lineage and spellcasting ability"
                    .to_owned(),
            );
        }
        if self.species != "Gnome" && any_gnome {
            return Err("Gnomish Lineage choices are only valid for Gnome characters".to_owned());
        }
        if let Some(value) = &self.gnome_lineage
            && !["Forest Gnome", "Rock Gnome"].contains(&value.as_str())
        {
            return Err("invalid Gnomish Lineage".to_owned());
        }
        validate_required_only(
            self.species == "Goliath",
            self.goliath_ancestry.is_some(),
            "Goliath characters must choose a Giant Ancestry",
            "Giant Ancestry is only valid for Goliath characters",
        )?;
        if let Some(value) = &self.goliath_ancestry
            && ![
                "Cloud Giant",
                "Fire Giant",
                "Frost Giant",
                "Hill Giant",
                "Stone Giant",
                "Storm Giant",
            ]
            .contains(&value.as_str())
        {
            return Err("invalid Giant Ancestry".to_owned());
        }
        let all_human = self.human_skill.is_some() && self.human_origin_feat.is_some();
        let any_human = self.human_skill.is_some() || self.human_origin_feat.is_some();
        if self.species == "Human" && !all_human {
            return Err(
                "Human characters must choose an additional skill and Origin feat".to_owned(),
            );
        }
        if self.species != "Human" && any_human {
            return Err("Human species choices are only valid for Human characters".to_owned());
        }
        if let Some(skill) = &self.human_skill
            && srd::skill_ability(skill).is_none()
        {
            return Err(format!("unknown Human Skillful proficiency: {skill}"));
        }
        if let Some(feat) = &self.human_origin_feat
            && !srd::ORIGIN_FEATS.contains(&feat.as_str())
        {
            return Err("invalid Human Origin feat".to_owned());
        }
        let all_tiefling =
            self.tiefling_legacy.is_some() && self.tiefling_spellcasting_ability.is_some();
        let any_tiefling =
            self.tiefling_legacy.is_some() || self.tiefling_spellcasting_ability.is_some();
        if self.species == "Tiefling" && !all_tiefling {
            return Err(
                "Tiefling characters must choose a Fiendish Legacy and spellcasting ability"
                    .to_owned(),
            );
        }
        if self.species != "Tiefling" && any_tiefling {
            return Err(
                "Fiendish Legacy choices are only valid for Tiefling characters".to_owned(),
            );
        }
        if let Some(value) = &self.tiefling_legacy
            && !["Abyssal", "Chthonic", "Infernal"].contains(&value.as_str())
        {
            return Err("invalid Fiendish Legacy".to_owned());
        }
        for ability in [
            &self.elf_spellcasting_ability,
            &self.gnome_spellcasting_ability,
            &self.tiefling_spellcasting_ability,
        ] {
            if let Some(value) = ability
                && !srd::SPELLCASTING_ABILITIES.contains(&value.as_str())
            {
                return Err("invalid species spellcasting ability".to_owned());
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn validate_origin_choices(&self) -> Result<(), String> {
        let Some(background_feat) = self.background_feat() else {
            return Ok(());
        };
        let background_skills = self.background_skills();
        if self.human_origin_feat.as_deref() == Some(background_feat)
            && !["Magic Initiate", "Skilled"].contains(&background_feat)
        {
            return Err(format!(
                "the {background_feat} Origin feat can be taken only once"
            ));
        }
        if self
            .human_skill
            .as_deref()
            .is_some_and(|skill| background_skills.contains(&skill))
        {
            return Err(
                "the Human Skillful proficiency must be additional to background skills".to_owned(),
            );
        }
        let expected_magic = usize::from(self.background_magic_initiate_list().is_some())
            + usize::from(self.human_origin_feat.as_deref() == Some("Magic Initiate"));
        if self.magic_initiate_choices.len() != expected_magic {
            return Err(format!(
                "character requires exactly {expected_magic} Magic Initiate choice(s)"
            ));
        }
        let mut lists = BTreeSet::new();
        for choice in &self.magic_initiate_choices {
            if srd::magic_initiate_spell_list(&choice.spell_list).is_none() {
                return Err("invalid Magic Initiate spell list".to_owned());
            }
            if !srd::SPELLCASTING_ABILITIES.contains(&choice.spellcasting_ability.as_str()) {
                return Err("invalid Magic Initiate spellcasting ability".to_owned());
            }
            if choice.cantrips[0] == choice.cantrips[1] {
                return Err("Magic Initiate requires two different cantrips".to_owned());
            }
            if choice.cantrips.iter().any(|cantrip| {
                !self.spell_is_on_list(cantrip, &choice.spell_list, 0)
                    && !self.unresolved_pack_spell(cantrip)
            }) {
                return Err(format!(
                    "Magic Initiate cantrips must come from the {} list",
                    choice.spell_list
                ));
            }
            if !self.spell_is_on_list(&choice.level_one_spell, &choice.spell_list, 1)
                && !self.unresolved_pack_spell(&choice.level_one_spell)
            {
                return Err(format!(
                    "Magic Initiate level 1 spell must come from the {} list",
                    choice.spell_list
                ));
            }
            if !lists.insert(choice.spell_list.as_str()) {
                return Err(
                    "repeatable Magic Initiate choices must use different spell lists".to_owned(),
                );
            }
        }
        if self
            .background_magic_initiate_list()
            .is_some_and(|required| !lists.contains(required))
        {
            return Err(format!(
                "the {} background requires Magic Initiate ({})",
                self.background,
                self.background_magic_initiate_list().expect("present")
            ));
        }
        let skilled_count = usize::from(background_feat == "Skilled")
            + usize::from(self.human_origin_feat.as_deref() == Some("Skilled"));
        let expected_proficiencies = skilled_count * 3;
        if self.skilled_proficiencies.len() != expected_proficiencies {
            return Err(format!(
                "Skilled requires exactly {expected_proficiencies} distinct skill or tool proficiencies"
            ));
        }
        if self
            .skilled_proficiencies
            .iter()
            .any(|value| srd::skill_ability(value).is_none() && !srd::is_tool(value))
        {
            return Err("unknown Skilled proficiencies".to_owned());
        }
        let existing: BTreeSet<&str> = background_skills
            .into_iter()
            .chain(self.human_skill.as_deref())
            .chain(self.elf_keen_senses_skill.as_deref())
            .collect();
        if self
            .skilled_proficiencies
            .iter()
            .any(|value| existing.contains(value.as_str()))
        {
            return Err(
                "Skilled must grant proficiencies the character does not already have".to_owned(),
            );
        }
        if self
            .skilled_proficiencies
            .iter()
            .filter(|value| srd::is_tool(value))
            .any(|tool| !self.tool_proficiencies.contains(tool))
        {
            return Err("Skilled tool choices must be included in tool proficiencies".to_owned());
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn validate_class_choices(&self) -> Result<(), String> {
        let srd_rule = srd::class_rule(&self.character_class);
        let pack_rule = self.resolved_pack_class.as_ref();
        let skill_count = pack_rule.map_or_else(
            || srd_rule.expect("validated class").skill_count,
            |rule| rule.skill_count,
        );
        let skills: Vec<&str> = pack_rule.map_or_else(
            || srd_rule.expect("validated class").skills.to_vec(),
            |rule| rule.skills.iter().map(String::as_str).collect(),
        );
        if self.class_skills.len() != skill_count {
            return Err(format!(
                "{} requires exactly {} class skills",
                self.class_name(),
                skill_count
            ));
        }
        if self
            .class_skills
            .iter()
            .any(|skill| !skills.contains(&skill.as_str()))
        {
            return Err(format!("invalid {} class skill choice", self.class_name()));
        }
        let mut granted: BTreeSet<&str> = self.background_skills().into_iter().collect();
        granted.extend(
            self.skilled_proficiencies
                .iter()
                .filter(|value| srd::skill_ability(value).is_some())
                .map(String::as_str),
        );
        granted.extend(self.human_skill.as_deref());
        granted.extend(self.elf_keen_senses_skill.as_deref());
        if self
            .class_skills
            .iter()
            .any(|skill| granted.contains(skill.as_str()))
        {
            return Err("class skills must not duplicate another granted proficiency".to_owned());
        }
        let mastery_count = pack_rule.map_or_else(
            || srd::weapon_mastery_count(&self.character_class),
            |rule| rule.weapon_mastery_count,
        );
        if self.class_choices.weapon_masteries.len() != mastery_count {
            return Err(format!(
                "{} requires exactly {mastery_count} weapon masteries",
                self.character_class
            ));
        }
        if self.class_choices.weapon_masteries.iter().any(|weapon| {
            srd::weapon_rule(weapon).is_none_or(|weapon_rule| {
                pack_rule.map_or_else(
                    || {
                        (self.character_class == "Barbarian" && weapon_rule.kind != "Melee")
                            || (self.character_class == "Rogue"
                                && weapon_rule.category != "Simple"
                                && !weapon_rule
                                    .properties
                                    .iter()
                                    .any(|property| ["Finesse", "Light"].contains(property)))
                    },
                    |class| {
                        !class
                            .weapon_training
                            .iter()
                            .any(|training| training == weapon_rule.category)
                    },
                )
            })
        }) {
            return Err(format!(
                "invalid {} weapon mastery choice",
                self.character_class
            ));
        }
        if let Some(class) = pack_rule {
            if self.class_choices.pack_choices.len() != class.choices.len() {
                return Err(format!(
                    "{} requires exactly one selection set for each class choice",
                    self.class_name()
                ));
            }
            for choice in &class.choices {
                let Some(selected) = self.class_choices.pack_choices.get(&choice.id) else {
                    return Err(format!("missing {} choice", choice.label));
                };
                if selected.len() != choice.count
                    || selected
                        .iter()
                        .any(|option| !choice.options.iter().any(|rule| rule.id == *option))
                {
                    return Err(format!("invalid {} choice", choice.label));
                }
            }
            if self
                .class_choices
                .pack_choices
                .keys()
                .any(|id| !class.choices.iter().any(|choice| choice.id == *id))
            {
                return Err("unknown pack class choice".to_owned());
            }
        } else if !self.class_choices.pack_choices.is_empty() {
            return Err("pack class choices require a resolved pack class".to_owned());
        }
        let expected_tools = if self.character_class == "Bard" {
            3
        } else {
            usize::from(self.character_class == "Monk")
        };
        if self.class_choices.tools.len() != expected_tools {
            return Err(format!(
                "{} requires exactly {expected_tools} class tool choices",
                self.character_class
            ));
        }
        if self.character_class == "Bard"
            && self
                .class_choices
                .tools
                .iter()
                .any(|tool| !srd::MUSICAL_INSTRUMENTS.contains(&tool.as_str()))
        {
            return Err("invalid Bard class tool choice".to_owned());
        }
        if self.character_class == "Monk"
            && self.class_choices.tools.iter().any(|tool| {
                !srd::MUSICAL_INSTRUMENTS.contains(&tool.as_str())
                    && !srd::ARTISAN_TOOLS.contains(&tool.as_str())
            })
        {
            return Err("invalid Monk class tool choice".to_owned());
        }
        let expected_expertise = if self.character_class == "Rogue" {
            2
        } else {
            0
        };
        if self.class_choices.expertise.len() != expected_expertise {
            return Err(format!(
                "{} requires exactly {expected_expertise} Expertise choices",
                self.character_class
            ));
        }
        if self
            .class_choices
            .expertise
            .iter()
            .any(|skill| !self.skills().contains(skill))
        {
            return Err("Expertise choices must be existing skill proficiencies".to_owned());
        }
        validate_required_only(
            self.character_class == "Cleric",
            self.class_choices.divine_order.is_some(),
            "Divine Order is required only for Clerics",
            "Divine Order is required only for Clerics",
        )?;
        validate_required_only(
            self.character_class == "Druid",
            self.class_choices.primal_order.is_some(),
            "Primal Order is required only for Druids",
            "Primal Order is required only for Druids",
        )?;
        validate_required_only(
            self.character_class == "Fighter",
            self.class_choices.fighting_style.is_some(),
            "Fighting Style is required only for Fighters",
            "Fighting Style is required only for Fighters",
        )?;
        validate_required_only(
            self.character_class == "Warlock",
            self.class_choices.eldritch_invocation.is_some(),
            "an Eldritch Invocation is required only for Warlocks",
            "an Eldritch Invocation is required only for Warlocks",
        )?;
        validate_required_only(
            self.character_class == "Rogue",
            self.class_choices.additional_language.is_some(),
            "an additional language is required only for Rogues",
            "an additional language is required only for Rogues",
        )?;
        if let Some(value) = &self.class_choices.divine_order
            && !["Protector", "Thaumaturge"].contains(&value.as_str())
        {
            return Err("invalid Divine Order".to_owned());
        }
        if let Some(value) = &self.class_choices.primal_order
            && !["Magician", "Warden"].contains(&value.as_str())
        {
            return Err("invalid Primal Order".to_owned());
        }
        if let Some(value) = &self.class_choices.fighting_style
            && !srd::FIGHTING_STYLES.contains(&value.as_str())
        {
            return Err("invalid Fighter Fighting Style".to_owned());
        }
        if let Some(value) = &self.class_choices.eldritch_invocation
            && !srd::WARLOCK_INVOCATIONS.contains(&value.as_str())
        {
            return Err("invalid level-1 Warlock invocation".to_owned());
        }
        let mut expected_cantrips = pack_rule
            .and_then(|class| class.spellcasting.as_ref())
            .map_or_else(
                || match self.character_class.as_str() {
                    "Bard" | "Druid" | "Warlock" => 2,
                    "Cleric" | "Wizard" => 3,
                    "Sorcerer" => 4,
                    _ => 0,
                },
                |spellcasting| spellcasting.cantrip_count,
            );
        if self.class_choices.divine_order.as_deref() == Some("Thaumaturge")
            || self.class_choices.primal_order.as_deref() == Some("Magician")
        {
            expected_cantrips += 1;
        }
        if self.class_choices.cantrips.len() != expected_cantrips {
            return Err(format!(
                "invalid number or selection of {} cantrips",
                self.character_class
            ));
        }
        let spell_list = pack_rule
            .and_then(|class| class.spellcasting.as_ref())
            .map_or_else(
                || self.character_class.as_str(),
                |spellcasting| spellcasting.spell_list.as_str(),
            );
        if self.class_choices.cantrips.iter().any(|spell| {
            !self.spell_is_on_list(spell, spell_list, 0) && !self.unresolved_pack_spell(spell)
        }) {
            return Err(format!(
                "invalid number or selection of {} cantrips",
                self.character_class
            ));
        }
        let expected_prepared = pack_rule
            .and_then(|class| class.spellcasting.as_ref())
            .map_or_else(
                || match self.character_class.as_str() {
                    "Bard" | "Cleric" | "Druid" | "Wizard" => 4,
                    "Paladin" | "Ranger" | "Sorcerer" | "Warlock" => 2,
                    _ => 0,
                },
                |spellcasting| spellcasting.prepared_spell_count,
            );
        if self.class_choices.prepared_spells.len() != expected_prepared {
            return Err(format!(
                "invalid number or selection of {} prepared spells",
                self.character_class
            ));
        }
        if self.class_choices.prepared_spells.iter().any(|spell| {
            (!self.spell_is_on_list(spell, spell_list, 1) && !self.unresolved_pack_spell(spell))
                || (pack_rule.is_none()
                    && srd::class_always_prepared(&self.character_class).contains(&spell.as_str()))
        }) {
            return Err(format!(
                "invalid number or selection of {} prepared spells",
                self.character_class
            ));
        }
        let expected_spellbook = usize::from(self.character_class == "Wizard") * 6;
        if self.class_choices.spellbook_spells.len() != expected_spellbook {
            return Err(format!(
                "{} requires exactly {expected_spellbook} spellbook spells",
                self.character_class
            ));
        }
        if self.class_choices.spellbook_spells.iter().any(|spell| {
            !self.spell_is_on_list(spell, "Wizard", 1) && !self.unresolved_pack_spell(spell)
        }) {
            return Err("Wizard spellbook spells must be level 1 Wizard spells".to_owned());
        }
        if self.character_class == "Wizard"
            && !self
                .class_choices
                .prepared_spells
                .is_subset(&self.class_choices.spellbook_spells)
        {
            return Err("Wizard prepared spells must be in the character's spellbook".to_owned());
        }
        if let Some(language) = &self.class_choices.additional_language {
            if !srd::STANDARD_LANGUAGES.contains(&language.as_str()) {
                return Err("invalid Rogue additional language".to_owned());
            }
            if self.selected_languages.contains(language) {
                return Err("the Rogue additional language must be a new language".to_owned());
            }
        }
        Ok(())
    }

    fn validate_equipment_choices(&self) -> Result<(), String> {
        let valid_class_equipment = self.resolved_pack_class.as_ref().map_or_else(
            || {
                self.class_equipment_option == "Gold"
                    || srd::class_equipment(&self.character_class, &self.class_equipment_option)
                        .is_some()
            },
            |_| ["A", "Gold"].contains(&self.class_equipment_option.as_str()),
        );
        if !valid_class_equipment {
            return Err(format!(
                "invalid {} starting-equipment option: {}",
                self.character_class, self.class_equipment_option
            ));
        }
        if self.character_class == "Bard" && self.class_equipment_option != "Gold" {
            if self
                .bard_starting_instrument
                .as_ref()
                .is_none_or(|value| !self.class_choices.tools.contains(value))
            {
                return Err(
                    "the Bard starting instrument must be one of the chosen proficiencies"
                        .to_owned(),
                );
            }
        } else if self.bard_starting_instrument.is_some() {
            return Err(
                "a Bard starting instrument requires the Bard equipment package".to_owned(),
            );
        }
        Ok(())
    }
}
fn validate_required_only(
    required: bool,
    present: bool,
    missing: &str,
    extraneous: &str,
) -> Result<(), String> {
    match (required, present) {
        (true, false) => Err(missing.to_owned()),
        (false, true) => Err(extraneous.to_owned()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::Character;
    use crate::character_wizard_domain::{PackBackground, PackClass, PackSpell};

    #[test]
    fn rejects_out_of_scope_levels_at_the_validation_boundary() {
        let source = include_str!("../../fixtures/complete-character.json");
        let mut value: serde_json::Value = serde_json::from_str(source).expect("fixture JSON");
        value["level"] = serde_json::json!(0);
        let error = Character::from_json(&value.to_string()).expect_err("level zero is invalid");
        assert_eq!(error, "level must be between 1 and 20");
    }

    #[test]
    fn pack_background_must_resolve_before_full_validation() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../fixtures/complete-character.json"))
                .expect("fixture JSON");
        value["background"] = serde_json::json!("shadow-agent");
        value["data_pack"] = serde_json::json!({
            "id": "test-pack",
            "format_version": 1,
            "version": 1
        });
        let mut character = Character::from_json(&value.to_string()).expect("structural record");
        assert_eq!(
            character.resolve_pack_background(&[]),
            Err("unknown background in data pack: shadow-agent".to_owned())
        );
        let rule: PackBackground = serde_json::from_str(
            r#"{"id":"shadow-agent","name":"Shadow Agent","abilities":["dexterity","constitution","intelligence"],"skills":["Sleight of Hand","Stealth"],"feat":"Alert","tool":"Thieves' Tools","equipment":[{"name":"Dagger"}],"equipment_gold":10,"gold_alternative":50}"#,
        )
        .expect("pack background");
        character
            .resolve_pack_background(std::slice::from_ref(&rule))
            .expect("resolved character");
        assert_eq!(character.background_name(), "Shadow Agent");
    }

    #[test]
    fn pack_spell_references_require_exact_catalog_resolution() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../fixtures/complete-character.json"))
                .expect("fixture JSON");
        value["background"] = serde_json::json!("moon-scholar");
        value["data_pack"] = serde_json::json!({
            "id": "test-pack",
            "format_version": 1,
            "version": 1
        });
        value["magic_initiate_choices"] = serde_json::json!([{
            "spell_list": "Wizard",
            "spellcasting_ability": "intelligence",
            "cantrips": ["moon-spark", "Light"],
            "level_one_spell": "Magic Missile"
        }]);
        let mut character = Character::from_json(&value.to_string()).expect("structural record");
        let background: PackBackground = serde_json::from_str(
            r#"{"id":"moon-scholar","name":"Moon Scholar","abilities":["dexterity","constitution","intelligence"],"skills":["Sleight of Hand","Stealth"],"feat":"Magic Initiate","magic_initiate_list":"Wizard","tool":"Thieves' Tools","equipment":[{"name":"Dagger"}],"equipment_gold":10,"gold_alternative":50}"#,
        )
        .expect("background record");
        character
            .resolve_pack_background(std::slice::from_ref(&background))
            .expect("resolve background before spells");
        assert_eq!(
            character.resolve_pack_spells(&[]),
            Err("Magic Initiate cantrips must come from the Wizard list".to_owned())
        );

        let spell: PackSpell = serde_json::from_str(
            r#"{"id":"moon-spark","name":"Moon Spark","level":0,"school":"Evocation","lists":["Wizard"],"casting_time":"Action","range":"60 feet","components":["V","S"],"notes":"Duration: Instantaneous","tags":["Damage"]}"#,
        )
        .expect("spell record");
        character
            .resolve_pack_spells(std::slice::from_ref(&spell))
            .expect("resolve exact spell catalog");
        assert_eq!(character.spell_name("moon-spark"), "Moon Spark");
    }

    #[test]
    fn pack_class_requires_exact_catalog_resolution() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../fixtures/complete-character.json"))
                .expect("fixture JSON");
        value["character_class"] = serde_json::json!("moon-warden");
        value["data_pack"] = serde_json::json!({
            "id": "test-pack",
            "format_version": 1,
            "version": 1
        });
        value["class_skills"] = serde_json::json!(["Athletics", "Perception"]);
        value["class_choices"] = serde_json::json!({
            "weapon_masteries": ["Longsword", "Shortbow"]
        });
        value["class_equipment_option"] = serde_json::json!("A");
        let mut character = Character::from_json(&value.to_string()).expect("structural record");
        assert_eq!(
            character.resolve_pack_class(&[]),
            Err("unknown class in data pack: moon-warden".to_owned())
        );
        let rule: PackClass = serde_json::from_str(
            r#"{"id":"moon-warden","name":"Moon Warden","hit_die":10,"saving_throws":["strength","wisdom"],"skill_count":2,"skills":["Athletics","Perception","Survival"],"armor_training":["Light","Medium","Shields"],"weapon_training":["Simple","Martial"],"equipment":[{"name":"Longsword"}],"equipment_gold":10,"starting_gold":150,"features":["Moonlit Vigil"],"weapon_mastery_count":2}"#,
        )
        .expect("class record");
        character
            .resolve_pack_class(std::slice::from_ref(&rule))
            .expect("resolve exact class catalog");
        assert_eq!(character.class_name(), "Moon Warden");
        assert_eq!(character.hit_points(), 11);
    }
}
