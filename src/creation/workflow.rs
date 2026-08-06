//! Interactive creation workflow and completion.

use std::{collections::BTreeSet, fs, path::Path};

use crate::character_wizard_domain::{
    AbilityGenerationMethod, AbilityScoreGeneration, AbilityScores, BackgroundAbilityAdjustment,
    Character, ClassChoices, MagicInitiateChoice,
};
use rand::RngExt as _;
use serde::{Deserialize, Serialize};

use crate::{
    character_wizard_srd_data,
    creation::{
        Result, WizardError,
        creation_prompts::{PromptPort, TerminalPromptPort},
    },
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OriginDraft {
    pub name: String,
    pub character_class: String,
    pub background: String,
    pub species: String,
    pub size: String,
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
    #[serde(default)]
    pub magic_initiate_choices: Vec<MagicInitiateChoice>,
    #[serde(default)]
    pub skilled_proficiencies: BTreeSet<String>,
    pub selected_languages: [String; 2],
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildDraft {
    pub class_skills: BTreeSet<String>,
    #[serde(default)]
    pub class_choices: ClassChoices,
    pub class_equipment_option: String,
    pub background_equipment_option: String,
    #[serde(default)]
    pub bard_starting_instrument: Option<String>,
    pub alignment: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct DetailsDraft {
    pub backstory: Option<String>,
    pub appearance: Option<String>,
    pub personality: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct CharacterDraft {
    pub origin: Option<OriginDraft>,
    pub abilities: Option<AbilityScores>,
    pub build: Option<BuildDraft>,
    pub details: Option<DetailsDraft>,
}

impl CharacterDraft {
    /// Load a current-format checkpoint.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read or contains an invalid draft.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .map_err(|error| format!("unable to read draft {}: {error}", path.display()))?;
        Ok(serde_json::from_str(&source)
            .map_err(|error| format!("invalid draft {}: {error}", path.display()))?)
    }

    /// Save a current-format checkpoint atomically enough for an interrupted CLI session.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization, directory creation, or writing fails.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| format!("unable to create {}: {error}", parent.display()))?;
        }
        let mut source = serde_json::to_string_pretty(self).map_err(|error| error.to_string())?;
        source.push('\n');
        Ok(fs::write(path, source)
            .map_err(|error| format!("unable to write draft {}: {error}", path.display()))?)
    }

    /// Convert a complete checkpoint into the canonical validated record.
    ///
    /// # Errors
    ///
    /// Returns an error identifying the first incomplete stage or domain validation failure.
    pub fn into_character(self) -> Result<Character> {
        let origin = self
            .origin
            .ok_or_else(|| "draft is missing origin choices".to_owned())?;
        let abilities = self
            .abilities
            .ok_or_else(|| "draft is missing ability scores".to_owned())?;
        let build = self
            .build
            .ok_or_else(|| "draft is missing build choices".to_owned())?;
        let details = self.details.unwrap_or_default();
        let character = Character {
            name: origin.name,
            character_class: origin.character_class.parse()?,
            background: origin.background.parse()?,
            species: origin.species.parse()?,
            size: origin.size.parse()?,
            dragonborn_ancestry: origin.dragonborn_ancestry,
            elf_lineage: origin.elf_lineage,
            elf_spellcasting_ability: origin.elf_spellcasting_ability,
            elf_keen_senses_skill: origin.elf_keen_senses_skill,
            gnome_lineage: origin.gnome_lineage,
            gnome_spellcasting_ability: origin.gnome_spellcasting_ability,
            goliath_ancestry: origin.goliath_ancestry,
            human_skill: origin.human_skill,
            human_origin_feat: origin.human_origin_feat,
            tiefling_legacy: origin.tiefling_legacy,
            tiefling_spellcasting_ability: origin.tiefling_spellcasting_ability,
            alignment: build.alignment,
            abilities,
            class_skills: build.class_skills,
            class_choices: build.class_choices,
            class_equipment_option: build.class_equipment_option,
            background_equipment_option: build.background_equipment_option,
            bard_starting_instrument: build.bard_starting_instrument,
            tool_proficiencies: origin
                .skilled_proficiencies
                .iter()
                .filter(|value| character_wizard_srd_data::is_tool(value))
                .cloned()
                .collect(),
            magic_initiate_choices: origin.magic_initiate_choices,
            skilled_proficiencies: origin.skilled_proficiencies,
            selected_languages: origin.selected_languages,
            backstory: details.backstory,
            appearance: details.appearance,
            personality: details.personality,
            level: 1,
            xp: 0,
        };
        let json = character.to_json()?;
        Ok(Character::from_json(&json)?)
    }
}

/// Run the native staged terminal wizard, resuming any existing checkpoint.
///
/// # Errors
///
/// Returns an error for input cancellation, checkpoint I/O, or invalid choices.
#[allow(clippy::too_many_lines)]
pub fn run_interactive(draft_path: impl AsRef<Path>) -> Result<Character> {
    run_interactive_with(draft_path, &TerminalPromptPort)
}

/// Run the staged wizard with a caller-supplied prompt adapter.
///
/// # Errors
///
/// Returns an error for input cancellation, checkpoint I/O, or invalid choices.
#[allow(clippy::too_many_lines)]
pub fn run_interactive_with(
    draft_path: impl AsRef<Path>,
    prompts: &dyn PromptPort,
) -> Result<Character> {
    let draft_path = draft_path.as_ref();
    let mut draft = if draft_path.is_file() {
        CharacterDraft::load(draft_path)?
    } else {
        CharacterDraft::default()
    };
    loop {
        if draft.origin.is_none() {
            print_progress(1, "Origin");
            match collect_origin(prompts) {
                Ok(origin) => {
                    draft.origin = Some(origin);
                    draft.save(draft_path)?;
                }
                Err(WizardError::Back) => {
                    println!("Origin is the first stage; there is nothing to go back to.");
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        if draft.abilities.is_none() {
            let origin = draft
                .origin
                .as_ref()
                .ok_or_else(|| "origin checkpoint missing".to_owned())?;
            print_progress(2, "Abilities");
            match collect_abilities(origin, prompts) {
                Ok(abilities) => {
                    draft.abilities = Some(abilities);
                    draft.save(draft_path)?;
                }
                Err(WizardError::Back) => {
                    draft.origin = None;
                    draft.save(draft_path)?;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        if draft.build.is_none() {
            let origin = draft
                .origin
                .as_ref()
                .ok_or_else(|| "origin checkpoint missing".to_owned())?;
            print_progress(3, "Build");
            match collect_build(origin, prompts) {
                Ok(build) => {
                    draft.build = Some(build);
                    draft.save(draft_path)?;
                }
                Err(WizardError::Back) => {
                    draft.abilities = None;
                    draft.save(draft_path)?;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        if draft.details.is_none() {
            print_progress(4, "Details");
            match collect_details(prompts) {
                Ok(details) => {
                    draft.details = Some(details);
                    draft.save(draft_path)?;
                }
                Err(WizardError::Back) => {
                    draft.build = None;
                    draft.save(draft_path)?;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        let character = draft.clone().into_character()?;
        print_progress(5, "Review");
        print_character_review(&character);
        let action = match prompts.choose(
            "Review action",
            &[
                "Accept",
                "Edit origin",
                "Edit abilities",
                "Edit build",
                "Edit details",
                "Save and exit",
            ],
        ) {
            Ok(action) => action,
            Err(WizardError::Back) => {
                draft.details = None;
                draft.save(draft_path)?;
                continue;
            }
            Err(error) => return Err(error),
        };
        match action.as_str() {
            "Accept" => return Ok(character),
            "Edit origin" => {
                draft.origin = None;
                draft.abilities = None;
                draft.build = None;
                draft.details = None;
            }
            "Edit abilities" => {
                draft.abilities = None;
                draft.build = None;
            }
            "Edit build" => draft.build = None,
            "Edit details" => draft.details = None,
            _ => return Err(WizardError::SaveAndExit),
        }
        draft.save(draft_path)?;
    }
}

fn print_progress(stage: usize, label: &str) {
    println!("\n== Step {stage}/5: {label} ==");
}

#[allow(clippy::too_many_lines)]
fn collect_origin(prompts: &dyn PromptPort) -> Result<OriginDraft> {
    let prompt = |label: &str| prompts.prompt(label);
    let choose = |label: &str, choices: &[&str]| prompts.choose(label, choices);
    let choose_set =
        |label: &str, choices: &[&str], count| prompts.choose_set(label, choices, count);
    let name = prompt("Character name")?;
    let character_class = choose("Class", &character_wizard_srd_data::CLASS_NAMES)?;
    let background = choose("Background", &character_wizard_srd_data::BACKGROUND_NAMES)?;
    let species = choose("Species", &character_wizard_srd_data::SPECIES_NAMES)?;
    let sizes =
        character_wizard_srd_data::species_rule(&species).map_or(&[][..], |rule| rule.sizes);
    let size = if sizes.len() == 1 {
        sizes[0].to_owned()
    } else {
        choose("Size", sizes)?
    };
    let mut origin = OriginDraft {
        name,
        character_class,
        background: background.clone(),
        species: species.clone(),
        size,
        dragonborn_ancestry: None,
        elf_lineage: None,
        elf_spellcasting_ability: None,
        elf_keen_senses_skill: None,
        gnome_lineage: None,
        gnome_spellcasting_ability: None,
        goliath_ancestry: None,
        human_skill: None,
        human_origin_feat: None,
        tiefling_legacy: None,
        tiefling_spellcasting_ability: None,
        magic_initiate_choices: Vec::new(),
        skilled_proficiencies: BTreeSet::new(),
        selected_languages: choose_plain_pair(
            prompts,
            "Choose two standard languages",
            &character_wizard_srd_data::STANDARD_LANGUAGES,
        )?,
    };
    match species.as_str() {
        "Dragonborn" => {
            origin.dragonborn_ancestry = Some(choose(
                "Draconic ancestry",
                &[
                    "Black", "Blue", "Brass", "Bronze", "Copper", "Gold", "Green", "Red", "Silver",
                    "White",
                ],
            )?);
        }
        "Elf" => {
            origin.elf_lineage = Some(choose("Elven lineage", &["Drow", "High Elf", "Wood Elf"])?);
            origin.elf_spellcasting_ability = Some(choose(
                "Spellcasting ability",
                &character_wizard_srd_data::SPELLCASTING_ABILITIES,
            )?);
            let background_skills = character_wizard_srd_data::background_rule(&background)
                .map_or(&[][..], |rule| rule.skills);
            let available: Vec<&str> = ["Insight", "Perception", "Survival"]
                .into_iter()
                .filter(|skill| !background_skills.contains(skill))
                .collect();
            origin.elf_keen_senses_skill = Some(choose("Keen Senses skill", &available)?);
        }
        "Gnome" => {
            origin.gnome_lineage =
                Some(choose("Gnomish lineage", &["Forest Gnome", "Rock Gnome"])?);
            origin.gnome_spellcasting_ability = Some(choose(
                "Spellcasting ability",
                &character_wizard_srd_data::SPELLCASTING_ABILITIES,
            )?);
        }
        "Goliath" => {
            origin.goliath_ancestry = Some(choose(
                "Giant ancestry",
                &[
                    "Cloud Giant",
                    "Fire Giant",
                    "Frost Giant",
                    "Hill Giant",
                    "Stone Giant",
                    "Storm Giant",
                ],
            )?);
        }
        "Human" => {
            let background_skills = character_wizard_srd_data::background_rule(&background)
                .map_or(&[][..], |rule| rule.skills);
            let available: Vec<&str> = character_wizard_srd_data::SKILLS
                .iter()
                .copied()
                .filter(|skill| !background_skills.contains(skill))
                .collect();
            origin.human_skill = Some(choose("Additional skill", &available)?);
            let background_feat = character_wizard_srd_data::background_rule(&background)
                .map_or("", |rule| rule.feat);
            let feats: Vec<&str> = character_wizard_srd_data::ORIGIN_FEATS
                .iter()
                .copied()
                .filter(|feat| {
                    *feat != background_feat || ["Magic Initiate", "Skilled"].contains(feat)
                })
                .collect();
            origin.human_origin_feat = Some(choose("Origin feat", &feats)?);
        }
        "Tiefling" => {
            origin.tiefling_legacy = Some(choose(
                "Fiendish legacy",
                &["Abyssal", "Chthonic", "Infernal"],
            )?);
            origin.tiefling_spellcasting_ability = Some(choose(
                "Spellcasting ability",
                &character_wizard_srd_data::SPELLCASTING_ABILITIES,
            )?);
        }
        _ => {}
    }
    let mut magic_lists = Vec::new();
    if let Some(list) = character_wizard_srd_data::background_rule(&background)
        .and_then(|rule| rule.magic_initiate_list)
    {
        magic_lists.push(list.to_owned());
    }
    if origin.human_origin_feat.as_deref() == Some("Magic Initiate") {
        let available: Vec<&str> = ["Cleric", "Druid", "Wizard"]
            .into_iter()
            .filter(|candidate| !magic_lists.iter().any(|list| list == candidate))
            .collect();
        magic_lists.push(choose("Human Magic Initiate list", &available)?);
    }
    for list in magic_lists {
        origin
            .magic_initiate_choices
            .push(collect_magic_initiate(&list, prompts)?);
    }
    if origin.human_origin_feat.as_deref() == Some("Skilled") {
        let background_rule = character_wizard_srd_data::background_rule(&background)
            .ok_or_else(|| "unknown background".to_owned())?;
        let unavailable = [
            origin.human_skill.as_deref(),
            origin.elf_keen_senses_skill.as_deref(),
        ];
        let choices: Vec<&str> = character_wizard_srd_data::SKILLS
            .iter()
            .copied()
            .filter(|skill| !background_rule.skills.contains(skill))
            .filter(|skill| !unavailable.contains(&Some(*skill)))
            .chain(
                character_wizard_srd_data::ARTISAN_TOOLS
                    .iter()
                    .chain(character_wizard_srd_data::MUSICAL_INSTRUMENTS.iter())
                    .copied()
                    .filter(|tool| *tool != background_rule.tool),
            )
            .collect();
        origin.skilled_proficiencies = choose_set("Skilled proficiencies", &choices, 3)?;
    }
    Ok(origin)
}

fn collect_magic_initiate(list: &str, prompts: &dyn PromptPort) -> Result<MagicInitiateChoice> {
    let choose = |label: &str, choices: &[&str]| prompts.choose(label, choices);
    let rules = character_wizard_srd_data::magic_initiate_spell_list(list)
        .ok_or_else(|| "invalid spell list".to_owned())?;
    Ok(MagicInitiateChoice {
        spell_list: list.to_owned(),
        spellcasting_ability: choose(
            "Magic Initiate spellcasting ability",
            &character_wizard_srd_data::SPELLCASTING_ABILITIES,
        )?,
        cantrips: choose_pair(prompts, "Magic Initiate cantrips", rules.cantrips)?,
        level_one_spell: choose("Magic Initiate level 1 spell", rules.level_one_spells)?,
    })
}

fn collect_abilities(origin: &OriginDraft, prompts: &dyn PromptPort) -> Result<AbilityScores> {
    let choose = |label: &str, choices: &[&str]| prompts.choose(label, choices);
    let method_label = choose(
        "Generate ability scores",
        &[
            "Use the class suggested array",
            "Assign the standard array",
            "Roll 4d6 and drop the lowest",
            "Use 27-point point buy",
        ],
    )?;
    let (method, scores) = match method_label.as_str() {
        "Use the class suggested array" => {
            let values = character_wizard_srd_data::suggested_array(&origin.character_class)
                .ok_or_else(|| "unknown class suggested array".to_owned())?;
            (AbilityGenerationMethod::SuggestedArray, scores_from(values))
        }
        "Assign the standard array" => (
            AbilityGenerationMethod::StandardArray,
            assign_score_pool(character_wizard_srd_data::STANDARD_ARRAY, prompts)?,
        ),
        "Roll 4d6 and drop the lowest" => {
            let mut rng = rand::rng();
            let mut values = [0; 6];
            for value in &mut values {
                let mut dice = [0_u8; 4];
                for die in &mut dice {
                    *die = rng.random_range(1..=6);
                }
                dice.sort_unstable();
                *value = dice[1..].iter().sum();
            }
            (
                AbilityGenerationMethod::Random,
                assign_score_pool(values, prompts)?,
            )
        }
        _ => (
            AbilityGenerationMethod::PointBuy,
            collect_point_buy_scores(prompts)?,
        ),
    };
    AbilityScoreGeneration {
        method,
        scores: scores.clone(),
        character_class: Some(origin.character_class.clone()),
    }
    .validate()?;
    apply_background_increases(&scores, &origin.background, prompts)
}

const ABILITIES: [&str; 6] = [
    "strength",
    "dexterity",
    "constitution",
    "intelligence",
    "wisdom",
    "charisma",
];

fn scores_from(values: [u8; 6]) -> AbilityScores {
    AbilityScores {
        strength: values[0],
        dexterity: values[1],
        constitution: values[2],
        intelligence: values[3],
        wisdom: values[4],
        charisma: values[5],
    }
}

fn assign_score_pool(mut pool: [u8; 6], prompts: &dyn PromptPort) -> Result<AbilityScores> {
    let choose = |label: &str, choices: &[&str]| prompts.choose(label, choices);
    let mut values = [0; 6];
    for (index, ability) in ABILITIES.iter().enumerate() {
        let labels: Vec<String> = pool[index..].iter().map(u8::to_string).collect();
        let choices: Vec<&str> = labels.iter().map(String::as_str).collect();
        let selected = choose(
            &format!(
                "Assign {} (remaining: {})",
                title(ability),
                choices.join(", ")
            ),
            &choices,
        )?
        .parse::<u8>()
        .map_err(|error| error.to_string())?;
        let relative = pool[index..]
            .iter()
            .position(|value| *value == selected)
            .ok_or_else(|| "selected score is unavailable".to_owned())?;
        pool.swap(index, index + relative);
        values[index] = selected;
    }
    Ok(scores_from(values))
}

fn collect_point_buy_scores(prompts: &dyn PromptPort) -> Result<AbilityScores> {
    let choose = |label: &str, choices: &[&str]| prompts.choose(label, choices);
    let mut values = [8_u8; 6];
    loop {
        let spent: u8 = values
            .iter()
            .map(|value| character_wizard_srd_data::point_buy_cost(*value).unwrap_or(0))
            .sum();
        let remaining = character_wizard_srd_data::POINT_BUY_BUDGET - spent;
        let summary = ABILITIES
            .iter()
            .zip(values)
            .map(|(ability, value)| format!("{} {value}", title(ability)))
            .collect::<Vec<_>>()
            .join(", ");
        let choice = choose(
            &format!("Point cost — {remaining} points remaining ({summary})"),
            &[
                "strength",
                "dexterity",
                "constitution",
                "intelligence",
                "wisdom",
                "charisma",
                "Finish",
            ],
        )?;
        if choice == "Finish" {
            if remaining == 0 {
                return Ok(scores_from(values));
            }
            println!("Spend the remaining {remaining} points before finishing.");
            continue;
        }
        let index = ABILITIES
            .iter()
            .position(|ability| *ability == choice)
            .ok_or_else(|| "unknown ability".to_owned())?;
        let refunded = character_wizard_srd_data::point_buy_cost(values[index]).unwrap_or(0);
        let available = remaining + refunded;
        let labels: Vec<String> = (8..=15)
            .filter(|score| {
                character_wizard_srd_data::point_buy_cost(*score)
                    .is_some_and(|cost| cost <= available)
            })
            .map(|score| score.to_string())
            .collect();
        let choices: Vec<&str> = labels.iter().map(String::as_str).collect();
        values[index] = choose(
            &format!("Set {} ({remaining} points remaining)", title(&choice)),
            &choices,
        )?
        .parse::<u8>()
        .map_err(|error| error.to_string())?;
    }
}

fn apply_background_increases(
    scores: &AbilityScores,
    background: &str,
    prompts: &dyn PromptPort,
) -> Result<AbilityScores> {
    let choose = |label: &str, choices: &[&str]| prompts.choose(label, choices);
    let rule = character_wizard_srd_data::background_rule(background)
        .ok_or_else(|| "unknown background".to_owned())?;
    let plus_one: Vec<&str> = rule
        .abilities
        .iter()
        .copied()
        .filter(|ability| scores.score(ability) <= 19)
        .collect();
    let plus_two: Vec<&str> = rule
        .abilities
        .iter()
        .copied()
        .filter(|ability| {
            scores.score(ability) <= 18 && plus_one.iter().any(|other| other != ability)
        })
        .collect();
    let mut methods = Vec::new();
    if !plus_two.is_empty() {
        methods.push("+2 to one and +1 to another");
    }
    if plus_one.len() == rule.abilities.len() {
        methods.push("+1 to all three");
    }
    if methods.is_empty() {
        return Err(WizardError::Message(
            "no legal background ability increases remain".to_owned(),
        ));
    }
    let method = choose("Apply background ability increases", &methods)?;
    let increases = if method.starts_with("+2") {
        let plus_two_choice = choose("Ability to increase by 2", &plus_two)?;
        let candidates: Vec<&str> = plus_one
            .into_iter()
            .filter(|ability| *ability != plus_two_choice)
            .collect();
        let plus_one_choice = choose("Different ability to increase by 1", &candidates)?;
        [(plus_two_choice, 2), (plus_one_choice, 1)]
            .into_iter()
            .collect()
    } else {
        rule.abilities
            .iter()
            .map(|ability| ((*ability).to_owned(), 1))
            .collect()
    };
    BackgroundAbilityAdjustment {
        background: background.to_owned(),
        base_scores: scores.clone(),
        increases,
    }
    .adjusted_scores()
    .map_err(Into::into)
}

fn title(value: &str) -> String {
    let mut value = value.to_owned();
    if let Some(first) = value.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    value
}

pub(crate) fn choice_description(choice: &str) -> Option<String> {
    if let Some(rule) = character_wizard_srd_data::class_rule(choice) {
        return Some(format!(
            "d{} Hit Die; saves {}; choose {} skills; armor {}; weapons {}; features {}",
            rule.hit_die,
            rule.saves.join(" and "),
            rule.skill_count,
            rule.armor,
            rule.weapons,
            character_wizard_srd_data::class_features(choice).join(", ")
        ));
    }
    if let Some(rule) = character_wizard_srd_data::background_rule(choice) {
        return Some(format!(
            "boost {}; feat {}; skills {}; tool {}",
            rule.abilities.join(", "),
            rule.feat,
            rule.skills.join(" and "),
            rule.tool
        ));
    }
    if let Some(rule) = character_wizard_srd_data::species_rule(choice) {
        let vision = rule.darkvision_range.map_or_else(
            || "no Darkvision".to_owned(),
            |range| format!("Darkvision {range} ft."),
        );
        return Some(format!(
            "{}; Speed {} ft.; {vision}; {}",
            rule.sizes.join(" or "),
            rule.speed,
            character_wizard_srd_data::species_traits(choice).join(", ")
        ));
    }
    if let Some(rule) = character_wizard_srd_data::spell_rule(choice) {
        let mut tags = Vec::new();
        if rule.concentration {
            tags.push("Concentration");
        }
        if rule.ritual {
            tags.push("Ritual");
        }
        if rule.required_material.is_some() {
            tags.push("Material");
        }
        return Some(format!(
            "{}; {}; {}{}",
            rule.casting_time,
            rule.range,
            rule.notes,
            if tags.is_empty() {
                String::new()
            } else {
                format!("; {}", tags.join(", "))
            }
        ));
    }
    if let Some(rule) = character_wizard_srd_data::weapon_rule(choice) {
        return Some(format!(
            "{} {} {}; mastery {}; range {}{}; {}",
            rule.damage,
            rule.damage_type,
            rule.kind,
            rule.mastery,
            rule.normal_range,
            rule.long_range
                .map_or_else(String::new, |range| format!("/{range}")),
            rule.properties.join(", ")
        ));
    }
    if let Some(ability) = character_wizard_srd_data::skill_ability(choice) {
        return Some(format!("uses {}", title(ability)));
    }
    match choice {
        "Alert" => Some("add Proficiency Bonus to Initiative".to_owned()),
        "Magic Initiate" => Some("learn two cantrips and one level 1 spell".to_owned()),
        "Savage Attacker" => Some("reroll weapon damage dice once per turn".to_owned()),
        "Skilled" => Some("gain three additional skill or tool proficiencies".to_owned()),
        "Archery" => Some("+2 to attack rolls with ranged weapons".to_owned()),
        "Defense" => Some("+1 AC while wearing armor".to_owned()),
        "Great Weapon Fighting" => Some("minimum 3 on two-handed weapon damage dice".to_owned()),
        "Two-Weapon Fighting" => Some("add ability modifier to the extra Light attack".to_owned()),
        "Armor of Shadows" => Some("cast Mage Armor on yourself without a spell slot".to_owned()),
        "Eldritch Mind" => Some("advantage on Concentration saves".to_owned()),
        "Pact of the Blade" => Some("conjure or bond a pact weapon".to_owned()),
        "Pact of the Chain" => {
            Some("learn Find Familiar and gain special familiar forms".to_owned())
        }
        "Pact of the Tome" => {
            Some("gain a Book of Shadows with three cantrips and two rituals".to_owned())
        }
        _ => None,
    }
}

#[allow(clippy::too_many_lines)]
fn collect_build(origin: &OriginDraft, prompts: &dyn PromptPort) -> Result<BuildDraft> {
    let choose = |label: &str, choices: &[&str]| prompts.choose(label, choices);
    let choose_set =
        |label: &str, choices: &[&str], count| prompts.choose_set(label, choices, count);
    let rule = character_wizard_srd_data::class_rule(&origin.character_class)
        .ok_or_else(|| "unknown class".to_owned())?;
    let background = character_wizard_srd_data::background_rule(&origin.background)
        .ok_or_else(|| "unknown background".to_owned())?;
    let unavailable: BTreeSet<&str> = background
        .skills
        .iter()
        .copied()
        .chain(origin.human_skill.as_deref())
        .chain(origin.elf_keen_senses_skill.as_deref())
        .chain(
            origin
                .skilled_proficiencies
                .iter()
                .filter(|value| character_wizard_srd_data::skill_ability(value).is_some())
                .map(String::as_str),
        )
        .collect();
    let available_skills: Vec<&str> = rule
        .skills
        .iter()
        .copied()
        .filter(|skill| !unavailable.contains(skill))
        .collect();
    let class_skills = choose_set("Class skills", &available_skills, rule.skill_count)?;
    let mut choices = ClassChoices::default();
    let mastery_count = character_wizard_srd_data::weapon_mastery_count(&origin.character_class);
    if mastery_count > 0 {
        let mastery_options: Vec<&str> = character_wizard_srd_data::WEAPON_NAMES
            .iter()
            .copied()
            .filter(|weapon| {
                let weapon = character_wizard_srd_data::weapon_rule(weapon).expect("known weapon");
                match origin.character_class.as_str() {
                    "Barbarian" => weapon.kind == "Melee",
                    "Rogue" => {
                        weapon.category == "Simple"
                            || weapon
                                .properties
                                .iter()
                                .any(|property| ["Finesse", "Light"].contains(property))
                    }
                    _ => true,
                }
            })
            .collect();
        choices.weapon_masteries = choose_set("Weapon masteries", &mastery_options, mastery_count)?;
    }
    if origin.character_class == "Bard" {
        choices.tools = choose_set(
            "Musical instrument proficiencies",
            &character_wizard_srd_data::MUSICAL_INSTRUMENTS,
            3,
        )?;
    }
    if origin.character_class == "Monk" {
        let tools: Vec<&str> = character_wizard_srd_data::ARTISAN_TOOLS
            .iter()
            .chain(character_wizard_srd_data::MUSICAL_INSTRUMENTS.iter())
            .copied()
            .collect();
        choices
            .tools
            .insert(choose("One artisan tool or musical instrument", &tools)?);
    }
    if origin.character_class == "Rogue" {
        let expertise: Vec<&str> = unavailable
            .iter()
            .copied()
            .chain(class_skills.iter().map(String::as_str))
            .collect();
        choices.expertise = choose_set("Two existing skills for Expertise", &expertise, 2)?;
        let languages: Vec<&str> = character_wizard_srd_data::STANDARD_LANGUAGES
            .iter()
            .copied()
            .filter(|language| {
                !origin
                    .selected_languages
                    .iter()
                    .any(|value| value == language)
            })
            .collect();
        choices.additional_language = Some(choose("Additional Rogue language", &languages)?);
    }
    if origin.character_class == "Cleric" {
        choices.divine_order = Some(choose("Divine Order", &["Protector", "Thaumaturge"])?);
    }
    if origin.character_class == "Druid" {
        choices.primal_order = Some(choose("Primal Order", &["Magician", "Warden"])?);
    }
    if origin.character_class == "Fighter" {
        choices.fighting_style = Some(choose(
            "Fighting Style",
            &character_wizard_srd_data::FIGHTING_STYLES,
        )?);
    }
    if origin.character_class == "Warlock" {
        choices.eldritch_invocation = Some(choose(
            "Eldritch Invocation",
            &character_wizard_srd_data::WARLOCK_INVOCATIONS,
        )?);
    }
    if let Some(spells) = character_wizard_srd_data::class_spell_list(&origin.character_class) {
        let mut cantrip_count = match origin.character_class.as_str() {
            "Bard" | "Druid" | "Warlock" => 2,
            "Cleric" | "Wizard" => 3,
            "Sorcerer" => 4,
            _ => 0,
        };
        if choices.divine_order.as_deref() == Some("Thaumaturge")
            || choices.primal_order.as_deref() == Some("Magician")
        {
            cantrip_count += 1;
        }
        if cantrip_count > 0 {
            choices.cantrips = choose_set("Class cantrips", spells.cantrips, cantrip_count)?;
        }
        if origin.character_class == "Wizard" {
            choices.spellbook_spells =
                choose_set("Wizard spellbook spells", spells.level_one_spells, 6)?;
            let options: Vec<&str> = choices
                .spellbook_spells
                .iter()
                .map(String::as_str)
                .collect();
            choices.prepared_spells = choose_set("Prepared Wizard spells", &options, 4)?;
        } else {
            let prepared = match origin.character_class.as_str() {
                "Bard" | "Cleric" | "Druid" => 4,
                "Paladin" | "Ranger" | "Sorcerer" | "Warlock" => 2,
                _ => 0,
            };
            if prepared > 0 {
                let options: Vec<&str> = spells
                    .level_one_spells
                    .iter()
                    .copied()
                    .filter(|spell| {
                        !character_wizard_srd_data::class_always_prepared(&origin.character_class)
                            .contains(spell)
                    })
                    .collect();
                choices.prepared_spells = choose_set("Prepared class spells", &options, prepared)?;
            }
        }
    }
    let class_options: &[&str] = if origin.character_class == "Fighter" {
        &["A", "B", "Gold"]
    } else {
        &["A", "Gold"]
    };
    let class_equipment_option =
        choose_class_equipment(prompts, &origin.character_class, class_options)?;
    let bard_starting_instrument =
        if origin.character_class == "Bard" && class_equipment_option != "Gold" {
            let options: Vec<&str> = choices.tools.iter().map(String::as_str).collect();
            Some(choose("Starting instrument", &options)?)
        } else {
            None
        };
    Ok(BuildDraft {
        class_skills,
        class_choices: choices,
        class_equipment_option,
        background_equipment_option: choose_background_equipment(prompts, &origin.background)?,
        bard_starting_instrument,
        alignment: choose("Alignment", &character_wizard_srd_data::ALIGNMENTS)?,
    })
}

fn choose_class_equipment(
    prompts: &dyn PromptPort,
    character_class: &str,
    options: &[&str],
) -> Result<String> {
    let labels = class_equipment_labels(character_class, options);
    choose_equipment_option(prompts, "Class equipment", options, &labels)
}

fn choose_background_equipment(prompts: &dyn PromptPort, background: &str) -> Result<String> {
    let options = ["A", "Gold"];
    let labels = background_equipment_labels(background);
    choose_equipment_option(prompts, "Background equipment", &options, &labels)
}

fn choose_equipment_option(
    prompts: &dyn PromptPort,
    label: &str,
    options: &[&str],
    display_labels: &[String],
) -> Result<String> {
    let choices: Vec<&str> = display_labels.iter().map(String::as_str).collect();
    let selected = prompts.choose(label, &choices)?;
    options
        .iter()
        .find(|option| {
            selected == **option
                || selected
                    .strip_prefix(**option)
                    .is_some_and(|suffix| suffix.starts_with(" —"))
        })
        .map(|option| (*option).to_owned())
        .ok_or_else(|| format!("invalid {label} choice: {selected}").into())
}

fn class_equipment_labels(character_class: &str, options: &[&str]) -> Vec<String> {
    options
        .iter()
        .filter_map(|option| {
            if *option == "Gold" {
                character_wizard_srd_data::class_starting_gold(character_class)
                    .map(|gold| format!("Gold — start with {gold} GP and no class package"))
            } else {
                character_wizard_srd_data::class_equipment(character_class, option).map(
                    |(items, gold)| {
                        format!("{option} — {}; plus {gold} GP", equipment_summary(items))
                    },
                )
            }
        })
        .collect()
}

fn background_equipment_labels(background: &str) -> Vec<String> {
    let mut labels = Vec::new();
    if let Some((items, gold)) = character_wizard_srd_data::background_equipment(background) {
        labels.push(format!("A — {}; plus {gold} GP", equipment_summary(items)));
    }
    labels.push("Gold — start with 50 GP and no background package".to_owned());
    labels
}

fn equipment_summary(items: &[character_wizard_srd_data::EquipmentGrant]) -> String {
    items
        .iter()
        .map(|item| {
            if item.quantity > 1 {
                format!("{} x {}", item.quantity, item.name)
            } else {
                item.name.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn print_character_review(character: &Character) {
    println!("\nCharacter review");
    for (label, value) in character_review_rows(character) {
        println!("{label:<18} {value}");
    }
    println!("\nAccept the character or choose a section to revise.");
}

fn character_review_rows(character: &Character) -> Vec<(&'static str, String)> {
    let abilities = [
        ("STR", "strength", character.abilities.strength),
        ("DEX", "dexterity", character.abilities.dexterity),
        ("CON", "constitution", character.abilities.constitution),
        ("INT", "intelligence", character.abilities.intelligence),
        ("WIS", "wisdom", character.abilities.wisdom),
        ("CHA", "charisma", character.abilities.charisma),
    ]
    .into_iter()
    .map(|(short_name, ability, score)| {
        format!(
            "{short_name} {score} ({:+})",
            character.abilities.modifier(ability)
        )
    })
    .collect::<Vec<_>>()
    .join("  ");
    let languages = std::iter::once("Common".to_owned())
        .chain(character.selected_languages.iter().cloned())
        .chain(character.class_choices.additional_language.iter().cloned())
        .collect::<Vec<_>>()
        .join(", ");
    let equipment = format!(
        "class {}, background {}",
        character.class_equipment_option, character.background_equipment_option
    );
    let mut rows = vec![
        ("Name", character.name.clone()),
        (
            "Character",
            format!(
                "Level {} {} {}",
                character.level, character.species, character.character_class
            ),
        ),
        ("Background", character.background.to_string()),
        ("Alignment", character.alignment.clone()),
        ("Size", character.size.to_string()),
        ("Abilities", abilities),
        (
            "Combat",
            format!(
                "HP {}  AC {}  Initiative {:+}  Speed {} ft.",
                character.hit_points(),
                character.armor_class(),
                character.initiative_modifier(),
                character.speed()
            ),
        ),
        (
            "Skills",
            character
                .skills()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", "),
        ),
        ("Languages", languages),
        ("Equipment", equipment),
    ];
    if !character.class_choices.cantrips.is_empty() {
        rows.push((
            "Cantrips",
            character
                .class_choices
                .cantrips
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    if !character.class_choices.prepared_spells.is_empty() {
        rows.push((
            "Prepared spells",
            character
                .class_choices
                .prepared_spells
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    rows
}

fn collect_details(prompts: &dyn PromptPort) -> Result<DetailsDraft> {
    Ok(DetailsDraft {
        backstory: prompts.optional_prompt("Backstory")?,
        appearance: prompts.optional_prompt("Appearance")?,
        personality: prompts.optional_prompt("Personality")?,
    })
}

fn choose_pair(prompts: &dyn PromptPort, label: &str, choices: &[&str]) -> Result<[String; 2]> {
    let values = prompts
        .choose_set(label, choices, 2)?
        .into_iter()
        .collect::<Vec<_>>();
    Ok([values[0].clone(), values[1].clone()])
}

fn choose_plain_pair(
    prompts: &dyn PromptPort,
    label: &str,
    choices: &[&str],
) -> Result<[String; 2]> {
    let values = prompts
        .choose_set_with_descriptions(label, choices, 2, false)?
        .into_iter()
        .collect::<Vec<_>>();
    Ok([values[0].clone(), values[1].clone()])
}

#[cfg(test)]
mod tests {
    use std::{
        cell::Cell,
        collections::BTreeSet,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::{
        BuildDraft, CharacterDraft, DetailsDraft, OriginDraft, background_equipment_labels,
        character_review_rows, class_equipment_labels, collect_details, run_interactive_with,
    };
    use crate::creation::{PromptPort, Result, WizardError};
    use crate::character_wizard_domain::Character;

    struct ScriptedPrompts;

    struct AcceptingPrompts;

    struct FullRoguePrompts {
        back_from_abilities_once: Cell<bool>,
        name_prompts: Cell<usize>,
    }

    impl FullRoguePrompts {
        const fn new(back_from_abilities_once: bool) -> Self {
            Self {
                back_from_abilities_once: Cell::new(back_from_abilities_once),
                name_prompts: Cell::new(0),
            }
        }
    }

    fn temporary_draft(label: &str) -> std::path::PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "character-wizard-{label}-{}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    impl PromptPort for ScriptedPrompts {
        fn prompt(&self, _label: &str) -> Result<String> {
            Err(WizardError::Message(
                "unexpected required text prompt".to_owned(),
            ))
        }

        fn optional_prompt(&self, label: &str) -> Result<Option<String>> {
            Ok(Some(format!("scripted {label}")))
        }

        fn choose(&self, _label: &str, _choices: &[&str]) -> Result<String> {
            Err(WizardError::Message("unexpected choice prompt".to_owned()))
        }

        fn choose_set(
            &self,
            _label: &str,
            _choices: &[&str],
            _count: usize,
        ) -> Result<BTreeSet<String>> {
            Err(WizardError::Message(
                "unexpected multi-select prompt".to_owned(),
            ))
        }

        fn choose_set_with_descriptions(
            &self,
            _label: &str,
            _choices: &[&str],
            _count: usize,
            _descriptions: bool,
        ) -> Result<BTreeSet<String>> {
            Err(WizardError::Message(
                "unexpected described multi-select prompt".to_owned(),
            ))
        }
    }

    impl PromptPort for AcceptingPrompts {
        fn prompt(&self, _label: &str) -> Result<String> {
            Err(WizardError::Message(
                "unexpected required text prompt".to_owned(),
            ))
        }

        fn optional_prompt(&self, _label: &str) -> Result<Option<String>> {
            Err(WizardError::Message(
                "unexpected optional text prompt".to_owned(),
            ))
        }

        fn choose(&self, label: &str, _choices: &[&str]) -> Result<String> {
            if label == "Review action" {
                Ok("Accept".to_owned())
            } else {
                Err(WizardError::Message(format!(
                    "unexpected choice prompt: {label}"
                )))
            }
        }

        fn choose_set(
            &self,
            _label: &str,
            _choices: &[&str],
            _count: usize,
        ) -> Result<BTreeSet<String>> {
            Err(WizardError::Message(
                "unexpected multi-select prompt".to_owned(),
            ))
        }

        fn choose_set_with_descriptions(
            &self,
            _label: &str,
            _choices: &[&str],
            _count: usize,
            _descriptions: bool,
        ) -> Result<BTreeSet<String>> {
            Err(WizardError::Message(
                "unexpected described multi-select prompt".to_owned(),
            ))
        }
    }

    impl PromptPort for FullRoguePrompts {
        fn prompt(&self, label: &str) -> Result<String> {
            if label == "Character name" {
                self.name_prompts.set(self.name_prompts.get() + 1);
                Ok("Scripted Rogue".to_owned())
            } else {
                Err(WizardError::Message(format!(
                    "unexpected required text prompt: {label}"
                )))
            }
        }

        fn optional_prompt(&self, _label: &str) -> Result<Option<String>> {
            Ok(None)
        }

        fn choose(&self, label: &str, _choices: &[&str]) -> Result<String> {
            let value = match label {
                "Class" => "Rogue",
                "Background" => "Criminal",
                "Species" => "Dwarf",
                "Generate ability scores" => {
                    if self.back_from_abilities_once.replace(false) {
                        return Err(WizardError::Back);
                    }
                    "Use the class suggested array"
                }
                "Apply background ability increases" => "+2 to one and +1 to another",
                "Ability to increase by 2" => "dexterity",
                "Different ability to increase by 1" => "constitution",
                "Additional Rogue language" => "Dwarvish",
                "Class equipment" | "Background equipment" => "A",
                "Alignment" => "Neutral",
                "Review action" => "Accept",
                _ => {
                    return Err(WizardError::Message(format!(
                        "unexpected choice prompt: {label}"
                    )));
                }
            };
            Ok(value.to_owned())
        }

        fn choose_set(
            &self,
            label: &str,
            _choices: &[&str],
            _count: usize,
        ) -> Result<BTreeSet<String>> {
            let values = match label {
                "Class skills" => ["Athletics", "Deception", "Insight", "Investigation"].as_slice(),
                "Weapon masteries" => ["Dagger", "Shortsword"].as_slice(),
                "Two existing skills for Expertise" => ["Investigation", "Stealth"].as_slice(),
                _ => {
                    return Err(WizardError::Message(format!(
                        "unexpected multi-select prompt: {label}"
                    )));
                }
            };
            Ok(values.iter().map(|value| (*value).to_owned()).collect())
        }

        fn choose_set_with_descriptions(
            &self,
            label: &str,
            _choices: &[&str],
            _count: usize,
            descriptions: bool,
        ) -> Result<BTreeSet<String>> {
            if label == "Choose two standard languages" && !descriptions {
                Ok(["Elvish", "Halfling"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect())
            } else {
                Err(WizardError::Message(format!(
                    "unexpected described multi-select prompt: {label}"
                )))
            }
        }
    }

    #[test]
    fn workflow_uses_the_supplied_prompt_port() {
        let details = collect_details(&ScriptedPrompts).expect("scripted details");
        assert_eq!(details.backstory.as_deref(), Some("scripted Backstory"));
        assert_eq!(details.appearance.as_deref(), Some("scripted Appearance"));
        assert_eq!(details.personality.as_deref(), Some("scripted Personality"));
    }

    #[test]
    fn equipment_details_are_embedded_in_selectable_rows() {
        assert_eq!(
            class_equipment_labels("Wizard", &["A", "Gold"]),
            [
                "A — 2 x Dagger, Arcane Focus (Quarterstaff), Robe, Spellbook, Scholar's Pack; plus 5 GP",
                "Gold — start with 55 GP and no class package"
            ]
        );
        let background = background_equipment_labels("Criminal");
        assert!(background[0].starts_with("A — 2 x Dagger, Thieves' Tools"));
        assert_eq!(
            background[1],
            "Gold — start with 50 GP and no background package"
        );
    }

    #[test]
    fn review_rows_are_human_readable_instead_of_json() {
        let character =
            Character::from_json(include_str!("../../fixtures/complete-character.json"))
                .expect("valid fixture");
        let rows = character_review_rows(&character);
        assert_eq!(rows[0], ("Name", "Binary Smoke Test".to_owned()));
        assert!(rows.iter().any(|(label, value)| {
            *label == "Combat" && value.contains("HP 9") && value.contains("AC 14")
        }));
        assert!(rows.iter().all(|(_, value)| !value.contains('{')));
    }

    #[test]
    fn scripted_port_completes_every_wizard_stage() {
        let path = temporary_draft("full-workflow");
        let prompts = FullRoguePrompts::new(false);
        let character = run_interactive_with(&path, &prompts).expect("scripted workflow completes");
        std::fs::remove_file(path).expect("remove scripted draft");

        assert_eq!(character.name, "Scripted Rogue");
        assert_eq!(character.character_class, "Rogue");
        assert_eq!(character.abilities.dexterity, 17);
        assert_eq!(character.abilities.constitution, 14);
        assert_eq!(character.class_skills.len(), 4);
        assert_eq!(character.class_choices.weapon_masteries.len(), 2);
        assert_eq!(character.class_choices.expertise.len(), 2);
        assert_eq!(prompts.name_prompts.get(), 1);
    }

    #[test]
    fn scripted_back_navigation_replays_the_origin_stage() {
        let path = temporary_draft("back-navigation");
        let prompts = FullRoguePrompts::new(true);
        let character =
            run_interactive_with(&path, &prompts).expect("workflow recovers after Back");
        std::fs::remove_file(path).expect("remove scripted draft");

        assert_eq!(character.name, "Scripted Rogue");
        assert_eq!(prompts.name_prompts.get(), 2);
    }

    #[test]
    fn reads_an_origin_checkpoint() {
        let draft: CharacterDraft = serde_json::from_str(
            r#"{
              "origin": {
                "name": "Checkpoint",
                "character_class": "Rogue",
                "background": "Criminal",
                "species": "Tiefling",
                "size": "Medium",
                "tiefling_legacy": "Infernal",
                "tiefling_spellcasting_ability": "charisma",
                "magic_initiate_choices": [],
                "skilled_proficiencies": [],
                "selected_languages": ["Elvish", "Halfling"]
              },
              "abilities": null,
              "build": null,
              "details": null
            }"#,
        )
        .expect("draft is valid");
        assert_eq!(draft.origin.expect("origin").name, "Checkpoint");
        assert!(draft.abilities.is_none());
    }

    #[test]
    fn complete_checkpoint_builds_the_canonical_character() {
        let source = include_str!("../../fixtures/complete-character.json");
        let character = Character::from_json(source).expect("character fixture");
        let draft = CharacterDraft {
            origin: Some(OriginDraft {
                name: character.name.clone(),
                character_class: character.character_class.to_string(),
                background: character.background.to_string(),
                species: character.species.to_string(),
                size: character.size.to_string(),
                dragonborn_ancestry: character.dragonborn_ancestry.clone(),
                elf_lineage: character.elf_lineage.clone(),
                elf_spellcasting_ability: character.elf_spellcasting_ability.clone(),
                elf_keen_senses_skill: character.elf_keen_senses_skill.clone(),
                gnome_lineage: character.gnome_lineage.clone(),
                gnome_spellcasting_ability: character.gnome_spellcasting_ability.clone(),
                goliath_ancestry: character.goliath_ancestry.clone(),
                human_skill: character.human_skill.clone(),
                human_origin_feat: character.human_origin_feat.clone(),
                tiefling_legacy: character.tiefling_legacy.clone(),
                tiefling_spellcasting_ability: character.tiefling_spellcasting_ability.clone(),
                magic_initiate_choices: character.magic_initiate_choices.clone(),
                skilled_proficiencies: character.skilled_proficiencies.clone(),
                selected_languages: character.selected_languages.clone(),
            }),
            abilities: Some(character.abilities.clone()),
            build: Some(BuildDraft {
                class_skills: character.class_skills.clone(),
                class_choices: character.class_choices.clone(),
                class_equipment_option: character.class_equipment_option.clone(),
                background_equipment_option: character.background_equipment_option.clone(),
                bard_starting_instrument: character.bard_starting_instrument.clone(),
                alignment: character.alignment.clone(),
            }),
            details: Some(DetailsDraft {
                backstory: character.backstory.clone(),
                appearance: character.appearance.clone(),
                personality: character.personality.clone(),
            }),
        };
        assert_eq!(draft.clone().into_character(), Ok(character.clone()));

        let path = temporary_draft("complete-draft");
        draft.save(&path).expect("save completed draft");
        let completed = run_interactive_with(&path, &AcceptingPrompts)
            .expect("scripted review accepts completed draft");
        std::fs::remove_file(path).expect("remove completed draft");
        assert_eq!(completed, character);
    }
}
