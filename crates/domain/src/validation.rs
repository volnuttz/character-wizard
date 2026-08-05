//! Canonical character parsing and cross-field validation.

use std::collections::BTreeSet;

use character_wizard_srd_data as srd;

use crate::CharacterSheet;

use super::record::Character;

impl Character {
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
        if character.name.trim().is_empty() {
            return Err("name must not be empty".to_owned());
        }
        if !(1..=20).contains(&character.level) {
            return Err("level must be between 1 and 20".to_owned());
        }
        character.abilities.validate()?;
        if character.selected_languages[0] == character.selected_languages[1] {
            return Err("choose two different standard languages".to_owned());
        }
        character.validate_closed_values()?;
        character.validate_species_choices()?;
        character.validate_origin_choices()?;
        character.validate_class_choices()?;
        character.validate_equipment_choices()?;
        Ok(character)
    }

    fn validate_closed_values(&self) -> Result<(), String> {
        if !["Small", "Medium"].contains(&self.size.as_str()) {
            return Err(format!("invalid size: {}", self.size));
        }
        if !srd::ALIGNMENTS.contains(&self.alignment.as_str()) {
            return Err(format!("invalid alignment: {}", self.alignment));
        }
        if srd::class_rule(&self.character_class).is_none() {
            return Err(format!("unknown SRD class: {}", self.character_class));
        }
        if srd::background_rule(&self.background).is_none() {
            return Err(format!("unknown SRD background: {}", self.background));
        }
        if srd::species_rule(&self.species).is_none() {
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
        let species = srd::species_rule(&self.species).expect("validated species");
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
        let background = srd::background_rule(&self.background).expect("validated background");
        if self.human_origin_feat.as_deref() == Some(background.feat)
            && !["Magic Initiate", "Skilled"].contains(&background.feat)
        {
            return Err(format!(
                "the {} Origin feat can be taken only once",
                background.feat
            ));
        }
        if self
            .human_skill
            .as_deref()
            .is_some_and(|skill| background.skills.contains(&skill))
        {
            return Err(
                "the Human Skillful proficiency must be additional to background skills".to_owned(),
            );
        }
        let expected_magic = usize::from(background.magic_initiate_list.is_some())
            + usize::from(self.human_origin_feat.as_deref() == Some("Magic Initiate"));
        if self.magic_initiate_choices.len() != expected_magic {
            return Err(format!(
                "character requires exactly {expected_magic} Magic Initiate choice(s)"
            ));
        }
        let mut lists = BTreeSet::new();
        for choice in &self.magic_initiate_choices {
            let spell_list = srd::magic_initiate_spell_list(&choice.spell_list)
                .ok_or_else(|| "invalid Magic Initiate spell list".to_owned())?;
            if !srd::SPELLCASTING_ABILITIES.contains(&choice.spellcasting_ability.as_str()) {
                return Err("invalid Magic Initiate spellcasting ability".to_owned());
            }
            if choice.cantrips[0] == choice.cantrips[1] {
                return Err("Magic Initiate requires two different cantrips".to_owned());
            }
            if choice
                .cantrips
                .iter()
                .any(|cantrip| !spell_list.cantrips.contains(&cantrip.as_str()))
            {
                return Err(format!(
                    "Magic Initiate cantrips must come from the {} list",
                    choice.spell_list
                ));
            }
            if !spell_list
                .level_one_spells
                .contains(&choice.level_one_spell.as_str())
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
        if background
            .magic_initiate_list
            .is_some_and(|required| !lists.contains(required))
        {
            return Err(format!(
                "the {} background requires Magic Initiate ({})",
                self.background,
                background.magic_initiate_list.expect("present")
            ));
        }
        let has_skilled = self.human_origin_feat.as_deref() == Some("Skilled");
        if has_skilled != (self.skilled_proficiencies.len() == 3) {
            return Err(if has_skilled {
                "Skilled requires exactly three skill or tool proficiencies"
            } else {
                "Skilled proficiencies require the Skilled Origin feat"
            }
            .to_owned());
        }
        if self
            .skilled_proficiencies
            .iter()
            .any(|value| srd::skill_ability(value).is_none() && !srd::is_tool(value))
        {
            return Err("unknown Skilled proficiencies".to_owned());
        }
        let existing: BTreeSet<&str> = background
            .skills
            .iter()
            .copied()
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
        let rule = srd::class_rule(&self.character_class).expect("validated class");
        if self.class_skills.len() != rule.skill_count {
            return Err(format!(
                "{} requires exactly {} class skills",
                self.character_class, rule.skill_count
            ));
        }
        if self
            .class_skills
            .iter()
            .any(|skill| !rule.skills.contains(&skill.as_str()))
        {
            return Err(format!(
                "invalid {} class skill choice",
                self.character_class
            ));
        }
        let mut granted: BTreeSet<&str> = srd::background_rule(&self.background)
            .expect("validated background")
            .skills
            .iter()
            .copied()
            .collect();
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
        let mastery_count = srd::weapon_mastery_count(&self.character_class);
        if self.class_choices.weapon_masteries.len() != mastery_count {
            return Err(format!(
                "{} requires exactly {mastery_count} weapon masteries",
                self.character_class
            ));
        }
        if self.class_choices.weapon_masteries.iter().any(|weapon| {
            srd::weapon_rule(weapon).is_none_or(|rule| {
                (self.character_class == "Barbarian" && rule.kind != "Melee")
                    || (self.character_class == "Rogue"
                        && rule.category != "Simple"
                        && !rule
                            .properties
                            .iter()
                            .any(|property| ["Finesse", "Light"].contains(property)))
            })
        }) {
            return Err(format!(
                "invalid {} weapon mastery choice",
                self.character_class
            ));
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
        let mut expected_cantrips = match self.character_class.as_str() {
            "Bard" | "Druid" | "Warlock" => 2,
            "Cleric" | "Wizard" => 3,
            "Sorcerer" => 4,
            _ => 0,
        };
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
        let spell_list = srd::class_spell_list(&self.character_class);
        if self
            .class_choices
            .cantrips
            .iter()
            .any(|spell| spell_list.is_none_or(|list| !list.cantrips.contains(&spell.as_str())))
        {
            return Err(format!(
                "invalid number or selection of {} cantrips",
                self.character_class
            ));
        }
        let expected_prepared = match self.character_class.as_str() {
            "Bard" | "Cleric" | "Druid" | "Wizard" => 4,
            "Paladin" | "Ranger" | "Sorcerer" | "Warlock" => 2,
            _ => 0,
        };
        if self.class_choices.prepared_spells.len() != expected_prepared {
            return Err(format!(
                "invalid number or selection of {} prepared spells",
                self.character_class
            ));
        }
        if self.class_choices.prepared_spells.iter().any(|spell| {
            spell_list.is_none_or(|list| {
                !list.level_one_spells.contains(&spell.as_str())
                    || srd::class_always_prepared(&self.character_class).contains(&spell.as_str())
            })
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
            spell_list.is_none_or(|list| !list.level_one_spells.contains(&spell.as_str()))
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
        if self.class_equipment_option != "Gold"
            && srd::class_equipment(&self.character_class, &self.class_equipment_option).is_none()
        {
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

    #[test]
    fn rejects_out_of_scope_levels_at_the_validation_boundary() {
        let source = include_str!("../../../fixtures/complete-character.json");
        let mut value: serde_json::Value = serde_json::from_str(source).expect("fixture JSON");
        value["level"] = serde_json::json!(0);
        let error = Character::from_json(&value.to_string()).expect_err("level zero is invalid");
        assert_eq!(error, "level must be between 1 and 20");
    }
}
