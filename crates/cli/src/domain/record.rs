//! Canonical character record and its current implementation.

use std::collections::BTreeSet;

use crate::character_wizard_srd_data as srd;
use serde::{Deserialize, Serialize};

use crate::domain::{BackgroundId, ClassId, Size, SpeciesId};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AbilityScores {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}

impl AbilityScores {
    #[must_use]
    pub fn modifier(&self, ability: &str) -> i16 {
        let score = match ability {
            "strength" => self.strength,
            "dexterity" => self.dexterity,
            "constitution" => self.constitution,
            "intelligence" => self.intelligence,
            "wisdom" => self.wisdom,
            "charisma" => self.charisma,
            _ => return 0,
        };
        (i16::from(score) - 10).div_euclid(2)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if [
            self.strength,
            self.dexterity,
            self.constitution,
            self.intelligence,
            self.wisdom,
            self.charisma,
        ]
        .into_iter()
        .any(|score| !(3..=20).contains(&score))
        {
            return Err("ability scores must be between 3 and 20".to_owned());
        }
        Ok(())
    }

    #[must_use]
    pub const fn ordered_values(&self) -> [u8; 6] {
        [
            self.strength,
            self.dexterity,
            self.constitution,
            self.intelligence,
            self.wisdom,
            self.charisma,
        ]
    }

    #[must_use]
    pub fn score(&self, ability: &str) -> u8 {
        match ability {
            "strength" => self.strength,
            "dexterity" => self.dexterity,
            "constitution" => self.constitution,
            "intelligence" => self.intelligence,
            "wisdom" => self.wisdom,
            "charisma" => self.charisma,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AbilityGenerationMethod {
    SuggestedArray,
    StandardArray,
    Random,
    PointBuy,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AbilityScoreGeneration {
    pub method: AbilityGenerationMethod,
    pub scores: AbilityScores,
    #[serde(default)]
    pub character_class: Option<String>,
}

impl AbilityScoreGeneration {
    /// Validate an SRD ability-score generation result.
    ///
    /// # Errors
    ///
    /// Returns an error when the scores do not satisfy the selected method.
    pub fn validate(&self) -> Result<(), String> {
        self.scores.validate()?;
        let values = self.scores.ordered_values();
        match self.method {
            AbilityGenerationMethod::SuggestedArray => {
                let expected = self
                    .character_class
                    .as_deref()
                    .and_then(srd::suggested_array)
                    .ok_or_else(|| "suggested array requires a known SRD class".to_owned())?;
                if values != expected {
                    return Err("scores do not match the class suggested array".to_owned());
                }
            }
            AbilityGenerationMethod::StandardArray => {
                let mut actual = values;
                let mut expected = srd::STANDARD_ARRAY;
                actual.sort_unstable();
                expected.sort_unstable();
                if actual != expected {
                    return Err(
                        "scores must assign every standard-array value exactly once".to_owned()
                    );
                }
            }
            AbilityGenerationMethod::Random => {
                if values.into_iter().any(|score| !(3..=18).contains(&score)) {
                    return Err("randomly generated scores must be between 3 and 18".to_owned());
                }
            }
            AbilityGenerationMethod::PointBuy => {
                let cost: Option<u8> = values.into_iter().map(srd::point_buy_cost).sum();
                if cost != Some(srd::POINT_BUY_BUDGET) {
                    return Err("point-buy scores must cost exactly 27 points".to_owned());
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BackgroundAbilityAdjustment {
    pub background: String,
    pub base_scores: AbilityScores,
    pub increases: std::collections::BTreeMap<String, u8>,
}

impl BackgroundAbilityAdjustment {
    /// Validate and apply an SRD background ability adjustment.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown background or invalid increases.
    pub fn adjusted_scores(&self) -> Result<AbilityScores, String> {
        let rule = srd::background_rule(&self.background)
            .ok_or_else(|| format!("unknown SRD background: {}", self.background))?;
        if self
            .increases
            .keys()
            .any(|ability| !rule.abilities.contains(&ability.as_str()))
        {
            return Err(
                "background increases contain an ability not granted by the background".to_owned(),
            );
        }
        let mut amounts: Vec<u8> = self.increases.values().copied().collect();
        amounts.sort_unstable();
        if amounts != [1, 2] && amounts != [1, 1, 1] {
            return Err("background increases must be +2/+1 or +1/+1/+1".to_owned());
        }
        let adjusted = AbilityScores {
            strength: increased(&self.base_scores, &self.increases, "strength")?,
            dexterity: increased(&self.base_scores, &self.increases, "dexterity")?,
            constitution: increased(&self.base_scores, &self.increases, "constitution")?,
            intelligence: increased(&self.base_scores, &self.increases, "intelligence")?,
            wisdom: increased(&self.base_scores, &self.increases, "wisdom")?,
            charisma: increased(&self.base_scores, &self.increases, "charisma")?,
        };
        adjusted.validate()?;
        Ok(adjusted)
    }
}

fn increased(
    scores: &AbilityScores,
    increases: &std::collections::BTreeMap<String, u8>,
    ability: &str,
) -> Result<u8, String> {
    scores
        .score(ability)
        .checked_add(increases.get(ability).copied().unwrap_or(0))
        .filter(|score| *score <= 20)
        .ok_or_else(|| format!("background increase would raise {ability} above 20"))
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ClassChoices {
    pub weapon_masteries: BTreeSet<String>,
    pub tools: BTreeSet<String>,
    pub expertise: BTreeSet<String>,
    pub cantrips: BTreeSet<String>,
    pub prepared_spells: BTreeSet<String>,
    pub spellbook_spells: BTreeSet<String>,
    pub divine_order: Option<String>,
    pub primal_order: Option<String>,
    pub fighting_style: Option<String>,
    pub eldritch_invocation: Option<String>,
    pub additional_language: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MagicInitiateChoice {
    pub spell_list: String,
    pub spellcasting_ability: String,
    pub cantrips: [String; 2],
    pub level_one_spell: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EquipmentItem {
    pub name: String,
    pub quantity: u16,
    pub category: String,
    pub weapon: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct CoinPurse {
    pub copper: u16,
    pub silver: u16,
    pub electrum: u16,
    pub gold: u16,
    pub platinum: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WeaponAttack {
    pub name: String,
    pub attack_bonus: i16,
    pub damage: String,
    pub damage_type: String,
    pub range: String,
    pub properties: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpellSlotPool {
    pub level: u8,
    pub total: u8,
    pub recovery: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpellcastingProfile {
    pub source: String,
    pub ability: String,
    pub modifier: i16,
    pub save_dc: i16,
    pub attack_bonus: i16,
    pub granted_spell_slots: Vec<SpellSlotPool>,
    pub free_casts: Vec<String>,
}

impl SpellcastingProfile {
    #[must_use]
    pub fn summary(&self) -> String {
        let mut ability = self.ability.clone();
        if let Some(first) = ability.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        let mut value = format!(
            "{}: {ability} (mod {:+}, save DC {}, attack {:+})",
            self.source, self.modifier, self.save_dc, self.attack_bonus
        );
        if self.granted_spell_slots.is_empty() {
            value.push_str("; grants no spell slots");
        }
        if !self.free_casts.is_empty() {
            value.push_str("; ");
            value.push_str(&self.free_casts.join("; "));
        }
        value
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpellTableEntry {
    pub level: u8,
    pub name: String,
    pub casting_time: String,
    pub range: String,
    pub concentration: bool,
    pub ritual: bool,
    pub required_material: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClassResource {
    pub name: String,
    pub maximum: i16,
    pub unit: String,
    pub recovery: String,
    pub detail: Option<String>,
}

impl ClassResource {
    #[must_use]
    pub fn summary(&self) -> String {
        let mut parts = vec![format!("{}: {} {}", self.name, self.maximum, self.unit)];
        parts.extend(self.detail.iter().cloned());
        parts.push(self.recovery.clone());
        parts.join("; ")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Character {
    pub name: String,
    pub character_class: ClassId,
    pub background: BackgroundId,
    pub species: SpeciesId,
    pub size: Size,
    #[serde(default)]
    pub dragonborn_ancestry: Option<String>,
    #[serde(default)]
    pub elf_lineage: Option<String>,
    #[serde(default)]
    pub elf_spellcasting_ability: Option<String>,
    #[serde(default)]
    pub elf_keen_senses_skill: Option<String>,
    #[serde(default)]
    pub gnome_lineage: Option<String>,
    #[serde(default)]
    pub gnome_spellcasting_ability: Option<String>,
    #[serde(default)]
    pub goliath_ancestry: Option<String>,
    #[serde(default)]
    pub human_skill: Option<String>,
    #[serde(default)]
    pub human_origin_feat: Option<String>,
    #[serde(default)]
    pub tiefling_legacy: Option<String>,
    #[serde(default)]
    pub tiefling_spellcasting_ability: Option<String>,
    pub alignment: String,
    pub abilities: AbilityScores,
    pub class_skills: BTreeSet<String>,
    #[serde(default)]
    pub class_choices: ClassChoices,
    #[serde(default = "default_equipment_option")]
    pub class_equipment_option: String,
    #[serde(default = "default_equipment_option")]
    pub background_equipment_option: String,
    #[serde(default)]
    pub bard_starting_instrument: Option<String>,
    #[serde(default)]
    pub tool_proficiencies: BTreeSet<String>,
    #[serde(default)]
    pub magic_initiate_choices: Vec<MagicInitiateChoice>,
    #[serde(default)]
    pub skilled_proficiencies: BTreeSet<String>,
    pub selected_languages: [String; 2],
    #[serde(default)]
    pub backstory: Option<String>,
    #[serde(default)]
    pub appearance: Option<String>,
    #[serde(default)]
    pub personality: Option<String>,
    #[serde(default = "default_level")]
    pub level: u8,
    #[serde(default)]
    pub xp: u32,
}

fn default_equipment_option() -> String {
    "A".to_owned()
}
const fn default_level() -> u8 {
    1
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        AbilityGenerationMethod, AbilityScoreGeneration, AbilityScores,
        BackgroundAbilityAdjustment, Character,
    };

    const COMPLETE_ROGUE: &str = include_str!("../../../../fixtures/complete-character.json");

    #[test]
    fn canonical_fixture_round_trips_and_derives_golden_scalars() {
        let character = Character::from_json(COMPLETE_ROGUE).expect("complete fixture is valid");
        let sheet = character.sheet();
        assert_eq!(character.hit_points(), 9);
        assert_eq!(character.armor_class(), 14);
        assert_eq!(character.initiative_modifier(), 5);
        assert_eq!(character.passive_perception(), 12);
        assert_eq!(character.speed(), 30);
        assert_eq!(character.coins().gold, 24);
        let attacks = character.weapon_attacks();
        assert_eq!(
            attacks
                .iter()
                .map(|attack| attack.name.as_str())
                .collect::<Vec<_>>(),
            ["Dagger", "Shortsword", "Shortbow"]
        );
        assert_eq!(attacks[0].attack_bonus, 5);
        assert_eq!(attacks[0].damage, "1d4+3");
        assert_eq!(attacks[0].notes, ["Quantity 4", "Mastery: Nick"]);
        assert_eq!(sheet.hit_die(), 8);
        assert_eq!(
            sheet.saving_throw("dexterity"),
            crate::domain::SavingThrow {
                proficient: true,
                modifier: 5
            }
        );
        assert_eq!(
            sheet.saving_throw("wisdom"),
            crate::domain::SavingThrow {
                proficient: false,
                modifier: 0
            }
        );
        let serialized = character.to_json().expect("character serializes");
        assert_eq!(Character::from_json(&serialized), Ok(character));
    }

    #[test]
    fn rejects_unknown_top_level_fields() {
        let error = Character::from_json(
            r#"{"name":"Nix","character_class":"Rogue","background":"Criminal","species":"Tiefling","size":"Medium","alignment":"Neutral","abilities":{"strength":12,"dexterity":17,"constitution":13,"intelligence":15,"wisdom":10,"charisma":8},"class_skills":[],"selected_languages":["Elvish","Halfling"],"migration_probe":true}"#,
        )
        .expect_err("unknown fields are rejected");

        assert!(error.contains("unknown field `migration_probe`"));
    }

    #[test]
    fn rejects_a_nonrepeatable_human_origin_feat_already_granted_by_background() {
        let mut value: serde_json::Value =
            serde_json::from_str(COMPLETE_ROGUE).expect("fixture JSON");
        let object = value.as_object_mut().expect("character object");
        object.insert("species".to_owned(), serde_json::json!("Human"));
        object.insert("human_skill".to_owned(), serde_json::json!("Arcana"));
        object.insert("human_origin_feat".to_owned(), serde_json::json!("Alert"));
        object.insert("tiefling_legacy".to_owned(), serde_json::Value::Null);
        object.insert(
            "tiefling_spellcasting_ability".to_owned(),
            serde_json::Value::Null,
        );
        let error = Character::from_json(&value.to_string()).expect_err("duplicate feat");
        assert!(error.contains("Alert Origin feat can be taken only once"));
    }

    #[test]
    fn validates_all_ability_generation_methods() {
        let standard = AbilityScores {
            strength: 15,
            dexterity: 14,
            constitution: 13,
            intelligence: 12,
            wisdom: 10,
            charisma: 8,
        };
        assert_eq!(
            AbilityScoreGeneration {
                method: AbilityGenerationMethod::StandardArray,
                scores: standard.clone(),
                character_class: None
            }
            .validate(),
            Ok(())
        );
        assert_eq!(
            AbilityScoreGeneration {
                method: AbilityGenerationMethod::SuggestedArray,
                scores: AbilityScores {
                    strength: 12,
                    dexterity: 15,
                    constitution: 13,
                    intelligence: 14,
                    wisdom: 10,
                    charisma: 8
                },
                character_class: Some("Rogue".to_owned())
            }
            .validate(),
            Ok(())
        );
        assert_eq!(
            AbilityScoreGeneration {
                method: AbilityGenerationMethod::PointBuy,
                scores: standard,
                character_class: None
            }
            .validate(),
            Ok(())
        );
    }

    #[test]
    fn applies_background_ability_increases() {
        let adjustment = BackgroundAbilityAdjustment {
            background: "Criminal".to_owned(),
            base_scores: AbilityScores {
                strength: 12,
                dexterity: 15,
                constitution: 13,
                intelligence: 14,
                wisdom: 10,
                charisma: 8,
            },
            increases: BTreeMap::from([
                ("dexterity".to_owned(), 2),
                ("constitution".to_owned(), 1),
            ]),
        };
        let adjusted = adjustment.adjusted_scores().expect("valid adjustment");
        assert_eq!(adjusted.dexterity, 17);
        assert_eq!(adjusted.constitution, 14);
    }

    #[test]
    fn validates_spellcasting_and_wizard_equipment_route() {
        let character = Character::from_json(r#"{
          "name":"Ada","character_class":"Wizard","background":"Sage","species":"Dwarf","size":"Medium","alignment":"Neutral Good",
          "abilities":{"strength":8,"dexterity":12,"constitution":14,"intelligence":17,"wisdom":15,"charisma":10},
          "class_skills":["Investigation","Nature"],
          "class_choices":{"cantrips":["Fire Bolt","Mage Hand","Prestidigitation"],"prepared_spells":["Detect Magic","Mage Armor","Magic Missile","Shield"],"spellbook_spells":["Detect Magic","Find Familiar","Mage Armor","Magic Missile","Shield","Sleep"]},
          "magic_initiate_choices":[{"spell_list":"Wizard","spellcasting_ability":"intelligence","cantrips":["Mage Hand","Prestidigitation"],"level_one_spell":"Mage Armor"}],
          "selected_languages":["Dwarvish","Elvish"]
        }"#).expect("wizard route is valid");
        assert_eq!(character.hit_points(), 9);
        assert_eq!(character.armor_class(), 11);
        assert_eq!(character.coins().gold, 13);
        assert!(
            character
                .inventory()
                .iter()
                .any(|item| item.name == "Spellbook")
        );
        assert!(
            character
                .weapon_attacks()
                .iter()
                .any(|attack| attack.name == "Arcane Focus (Quarterstaff)")
        );
    }
}
