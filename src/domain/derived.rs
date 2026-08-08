//! Derived character values and presentation-ready rule projections.

use std::collections::BTreeSet;

use crate::character_wizard_srd_data as srd;

use super::{
    content::{PackClass, PackEquipment, PackEquipmentKind, PackSpell},
    record::{
        Ability, Character, ClassResource, CoinPurse, EquipmentItem, SpellSlotPool,
        SpellTableEntry, SpellcastingProfile, WeaponAttack,
    },
};

#[derive(Clone, Copy)]
struct ArmorRuleView {
    base_ac: i16,
    dexterity_cap: Option<i16>,
    strength_requirement: Option<u8>,
}

struct WeaponRuleView<'a> {
    category: &'a str,
    kind: &'a str,
    properties: Vec<&'a str>,
    mastery: &'a str,
    damage: &'a str,
    damage_type: &'a str,
    normal_range: u16,
    long_range: Option<u16>,
    versatile_damage: Option<&'a str>,
}

type InventoryKey = (String, Option<String>, Option<String>);

impl Character {
    fn pack_class(&self) -> Option<&PackClass> {
        self.resolved_pack_class.as_ref()
    }

    #[must_use]
    pub fn class_name(&self) -> &str {
        self.pack_class()
            .map_or_else(|| self.character_class.as_str(), |rule| rule.name.as_str())
    }

    #[must_use]
    pub(crate) fn class_hit_die(&self) -> u8 {
        self.pack_class().map_or_else(
            || srd::class_rule(&self.character_class).map_or(0, |rule| rule.hit_die),
            |rule| rule.hit_die,
        )
    }

    #[must_use]
    pub(crate) fn class_saving_throws(&self) -> Vec<&str> {
        self.pack_class().map_or_else(
            || {
                srd::class_rule(&self.character_class)
                    .map_or(&[][..], |rule| rule.saves)
                    .to_vec()
            },
            |rule| rule.saving_throws.iter().map(String::as_str).collect(),
        )
    }

    fn pack_spell(&self, reference: &str) -> Option<&PackSpell> {
        self.resolved_pack_spells
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .find(|spell| spell.id == reference)
    }

    #[must_use]
    pub fn spell_name<'a>(&'a self, reference: &'a str) -> &'a str {
        self.pack_spell(reference)
            .map_or(reference, |spell| spell.name.as_str())
    }

    #[must_use]
    pub fn spell_is_on_list(&self, reference: &str, list: &str, level: u8) -> bool {
        if let Some(spell) = self.pack_spell(reference) {
            return spell.level == level && spell.lists.iter().any(|value| value == list);
        }
        let Some(spells) = (if ["Cleric", "Druid", "Wizard"].contains(&list) {
            srd::magic_initiate_spell_list(list)
        } else {
            srd::class_spell_list(list)
        }) else {
            return false;
        };
        if level == 0 {
            spells.cantrips.contains(&reference)
        } else {
            level == 1 && spells.level_one_spells.contains(&reference)
        }
    }

    fn pack_equipment_by_id(&self, id: &str) -> Option<&PackEquipment> {
        self.resolved_pack_equipment
            .iter()
            .find(|item| item.id == id)
    }

    fn armor_rule(&self, name: &str, equipment_id: Option<&str>) -> Option<ArmorRuleView> {
        if let Some(PackEquipment {
            kind:
                PackEquipmentKind::Armor {
                    base_ac,
                    dexterity_cap,
                    strength_requirement,
                    ..
                },
            ..
        }) = equipment_id.and_then(|id| self.pack_equipment_by_id(id))
        {
            return Some(ArmorRuleView {
                base_ac: *base_ac,
                dexterity_cap: *dexterity_cap,
                strength_requirement: *strength_requirement,
            });
        }
        srd::armor_rule(name).map(|rule| ArmorRuleView {
            base_ac: rule.base_ac,
            dexterity_cap: rule.dexterity_cap,
            strength_requirement: rule.strength_requirement,
        })
    }

    fn weapon_rule(&self, name: &str, equipment_id: Option<&str>) -> Option<WeaponRuleView<'_>> {
        if let Some(PackEquipment {
            kind:
                PackEquipmentKind::Weapon {
                    category,
                    kind,
                    properties,
                    mastery,
                    damage,
                    damage_type,
                    normal_range,
                    long_range,
                    versatile_damage,
                },
            ..
        }) = equipment_id.and_then(|id| self.pack_equipment_by_id(id))
        {
            return Some(WeaponRuleView {
                category,
                kind,
                properties: properties.iter().map(String::as_str).collect(),
                mastery,
                damage,
                damage_type,
                normal_range: *normal_range,
                long_range: *long_range,
                versatile_damage: versatile_damage.as_deref(),
            });
        }
        srd::weapon_rule(name).map(|rule| WeaponRuleView {
            category: rule.category,
            kind: rule.kind,
            properties: rule.properties.to_vec(),
            mastery: rule.mastery,
            damage: rule.damage,
            damage_type: rule.damage_type,
            normal_range: rule.normal_range,
            long_range: rule.long_range,
            versatile_damage: rule.versatile_damage,
        })
    }

    #[must_use]
    pub fn background_name(&self) -> &str {
        self.resolved_pack_background
            .as_ref()
            .map_or_else(|| self.background.as_str(), |rule| rule.name.as_str())
    }

    #[must_use]
    pub fn background_skills(&self) -> Vec<&str> {
        self.resolved_pack_background.as_ref().map_or_else(
            || {
                srd::background_rule(&self.background)
                    .map_or(&[][..], |rule| rule.skills)
                    .to_vec()
            },
            |rule| rule.skills.iter().map(String::as_str).collect(),
        )
    }

    #[must_use]
    pub fn background_feat(&self) -> Option<&str> {
        self.resolved_pack_background.as_ref().map_or_else(
            || srd::background_rule(&self.background).map(|rule| rule.feat),
            |rule| Some(rule.feat.as_str()),
        )
    }

    #[must_use]
    pub fn background_tool(&self) -> Option<&str> {
        self.resolved_pack_background.as_ref().map_or_else(
            || srd::background_rule(&self.background).map(|rule| rule.tool),
            |rule| Some(rule.tool.as_str()),
        )
    }

    #[must_use]
    pub fn background_magic_initiate_list(&self) -> Option<&str> {
        self.resolved_pack_background.as_ref().map_or_else(
            || srd::background_rule(&self.background).and_then(|rule| rule.magic_initiate_list),
            |rule| rule.magic_initiate_list.as_deref(),
        )
    }

    #[must_use]
    pub fn skills(&self) -> BTreeSet<String> {
        let mut values: BTreeSet<String> = self
            .background_skills()
            .into_iter()
            .map(str::to_owned)
            .collect();
        values.extend(self.class_skills.iter().cloned());
        values.extend(
            self.skilled_proficiencies
                .iter()
                .filter(|value| srd::skill_ability(value).is_some())
                .cloned(),
        );
        values.extend(self.human_skill.iter().cloned());
        values.extend(self.elf_keen_senses_skill.iter().cloned());
        values
    }

    #[must_use]
    pub fn skill_modifier(&self, skill: &str) -> i16 {
        let Some(ability) =
            srd::skill_ability(skill).and_then(|ability| Ability::try_from(ability).ok())
        else {
            return 0;
        };
        let mut value = self.abilities.modifier(ability);
        if self.class_choices.expertise.contains(skill) {
            value += i16::from(self.proficiency_bonus()) * 2;
        } else if self.skills().contains(skill) {
            value += i16::from(self.proficiency_bonus());
        }
        if self.class_choices.divine_order.as_deref() == Some("Thaumaturge")
            && ["Arcana", "Religion"].contains(&skill)
        {
            value += self.abilities.modifier(Ability::Wisdom).max(1);
        }
        if self.class_choices.primal_order.as_deref() == Some("Magician")
            && ["Arcana", "Nature"].contains(&skill)
        {
            value += self.abilities.modifier(Ability::Wisdom).max(1);
        }
        value
    }

    #[must_use]
    pub fn hit_points(&self) -> i16 {
        i16::from(self.class_hit_die())
            + self.abilities.modifier(Ability::Constitution)
            + i16::from(self.species == "Dwarf")
    }

    #[must_use]
    pub fn initiative_modifier(&self) -> i16 {
        self.abilities.modifier(Ability::Dexterity)
            + if self.background_feat() == Some("Alert")
                || self.human_origin_feat.as_deref() == Some("Alert")
            {
                i16::from(self.proficiency_bonus())
            } else {
                0
            }
    }

    #[must_use]
    pub fn speed(&self) -> u8 {
        let mut speed = if self.elf_lineage.as_deref() == Some("Wood Elf") {
            35
        } else {
            self.resolved_pack_species.as_ref().map_or_else(
                || srd::species_rule(&self.species).map_or(0, |rule| rule.speed),
                |rule| rule.speed,
            )
        };
        if self
            .equipped_armor_item()
            .as_ref()
            .and_then(|armor| self.armor_rule(&armor.name, armor.equipment_id.as_deref()))
            .and_then(|armor| armor.strength_requirement)
            .is_some_and(|required| self.abilities.strength < required)
        {
            speed = speed.saturating_sub(10);
        }
        speed
    }

    #[must_use]
    pub fn armor_class(&self) -> i16 {
        let dexterity = self.abilities.modifier(Ability::Dexterity);
        let armor = self.equipped_armor_item();
        let mut value = if let Some(rule) = armor
            .as_ref()
            .and_then(|item| self.armor_rule(&item.name, item.equipment_id.as_deref()))
        {
            rule.base_ac
                + match rule.dexterity_cap {
                    Some(cap) => dexterity.min(cap),
                    None => dexterity,
                }
        } else if self.character_class == "Barbarian" {
            10 + dexterity + self.abilities.modifier(Ability::Constitution)
        } else if self.character_class == "Monk" {
            10 + dexterity + self.abilities.modifier(Ability::Wisdom)
        } else {
            10 + dexterity
        };
        if armor.is_some() && self.class_choices.fighting_style.as_deref() == Some("Defense") {
            value += 1;
        }
        if self.shield_equipped() {
            value += self.shield_bonus();
        }
        value
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn inventory(&self) -> Vec<EquipmentItem> {
        let mut grants = Vec::new();
        if self.class_equipment_option != "Gold"
            && let Some((items, _)) =
                srd::class_equipment(&self.character_class, &self.class_equipment_option)
        {
            grants.extend_from_slice(items);
        }
        if self.background_equipment_option == "A"
            && self.resolved_pack_background.is_none()
            && let Some((items, _)) = srd::background_equipment(&self.background)
        {
            grants.extend_from_slice(items);
        }
        let mut merged: Vec<(InventoryKey, u16)> = Vec::new();
        for grant in grants {
            let expanded = srd::pack_contents(grant.name).unwrap_or(std::slice::from_ref(&grant));
            for item in expanded {
                let name = match item.name {
                    "Chosen Musical Instrument" => self
                        .bard_starting_instrument
                        .as_deref()
                        .unwrap_or(item.name),
                    "Chosen Monk Tool" => self
                        .class_choices
                        .tools
                        .first()
                        .map_or(item.name, String::as_str),
                    _ => item.name,
                };
                let weapon = item.weapon.or_else(|| srd::weapon_rule(name).map(|_| name));
                let key = (name.to_owned(), weapon.map(str::to_owned), None);
                if let Some((_, quantity)) =
                    merged.iter_mut().find(|(existing, _)| *existing == key)
                {
                    *quantity += item.quantity;
                } else {
                    merged.push((key, item.quantity));
                }
            }
        }
        if self.class_equipment_option != "Gold"
            && let Some(class) = self.pack_class()
        {
            for item in &class.equipment {
                let Some(name) = item.name.as_deref() else {
                    continue;
                };
                let weapon = srd::weapon_rule(name).map(|_| name.to_owned());
                let key = (name.to_owned(), weapon, None);
                if let Some((_, quantity)) =
                    merged.iter_mut().find(|(existing, _)| *existing == key)
                {
                    *quantity += item.quantity;
                } else {
                    merged.push((key, item.quantity));
                }
            }
        }
        if self.background_equipment_option == "A"
            && let Some(background) = &self.resolved_pack_background
        {
            for item in &background.equipment {
                let (name, weapon, equipment_id) = if let Some(id) = &item.equipment_id {
                    let Some(rule) = self
                        .resolved_pack_equipment
                        .iter()
                        .find(|rule| rule.id == *id)
                    else {
                        continue;
                    };
                    let weapon = matches!(&rule.kind, PackEquipmentKind::Weapon { .. })
                        .then(|| rule.name.clone());
                    (rule.name.clone(), weapon, Some(rule.id.clone()))
                } else {
                    let Some(name) = item.name.clone() else {
                        continue;
                    };
                    let weapon = srd::weapon_rule(&name).map(|_| name.clone());
                    (name, weapon, None)
                };
                let key = (name, weapon, equipment_id);
                if let Some((_, quantity)) =
                    merged.iter_mut().find(|(existing, _)| *existing == key)
                {
                    *quantity += item.quantity;
                } else {
                    merged.push((key, item.quantity));
                }
            }
        }
        merged
            .into_iter()
            .map(|((name, weapon, equipment_id), quantity)| {
                let category = if let Some(rule) = equipment_id
                    .as_deref()
                    .and_then(|id| self.pack_equipment_by_id(id))
                {
                    match &rule.kind {
                        PackEquipmentKind::Weapon { .. } => "Weapon",
                        PackEquipmentKind::Armor { .. } => "Armor",
                        PackEquipmentKind::Shield { .. } => "Shield",
                        PackEquipmentKind::Ammunition => "Ammunition",
                        PackEquipmentKind::Gear => "Gear",
                    }
                    .to_owned()
                } else if weapon.is_some() {
                    "Weapon".to_owned()
                } else if srd::armor_rule(&name).is_some() {
                    "Armor".to_owned()
                } else if name == "Shield" {
                    "Shield".to_owned()
                } else if ["Arrow", "Bolt", "Firearm Bullet", "Sling Bullet", "Needle"]
                    .contains(&name.as_str())
                {
                    "Ammunition".to_owned()
                } else {
                    "Gear".to_owned()
                };
                EquipmentItem {
                    name,
                    quantity,
                    category,
                    weapon,
                    equipment_id,
                }
            })
            .collect()
    }

    #[must_use]
    pub fn coins(&self) -> CoinPurse {
        let class_gold = if self.class_equipment_option == "Gold" {
            self.pack_class().map_or_else(
                || srd::class_starting_gold(&self.character_class).unwrap_or(0),
                |rule| rule.starting_gold,
            )
        } else {
            self.pack_class().map_or_else(
                || {
                    srd::class_equipment(&self.character_class, &self.class_equipment_option)
                        .map_or(0, |(_, gold)| gold)
                },
                |rule| rule.equipment_gold,
            )
        };
        let background_gold = if self.background_equipment_option == "Gold" {
            self.resolved_pack_background
                .as_ref()
                .map_or(50, |rule| rule.gold_alternative)
        } else {
            self.resolved_pack_background.as_ref().map_or_else(
                || srd::background_equipment(&self.background).map_or(0, |(_, gold)| gold),
                |rule| rule.equipment_gold,
            )
        };
        CoinPurse {
            gold: class_gold + background_gold,
            ..CoinPurse::default()
        }
    }

    fn equipped_armor_item(&self) -> Option<EquipmentItem> {
        self.inventory()
            .into_iter()
            .find(|item| item.category == "Armor")
    }

    #[must_use]
    pub fn shield_equipped(&self) -> bool {
        self.inventory()
            .iter()
            .any(|item| item.category == "Shield")
    }

    fn shield_bonus(&self) -> i16 {
        self.inventory()
            .iter()
            .filter(|item| item.category == "Shield")
            .map(|item| {
                item.equipment_id
                    .as_deref()
                    .and_then(|id| self.pack_equipment_by_id(id))
                    .map_or(2, |rule| {
                        if let PackEquipmentKind::Shield { armor_class_bonus } = &rule.kind {
                            *armor_class_bonus
                        } else {
                            2
                        }
                    })
            })
            .max()
            .unwrap_or(0)
    }

    fn is_weapon_proficient(&self, rule: &WeaponRuleView<'_>) -> bool {
        let training = srd::class_rule(&self.character_class).map_or("", |class| class.weapons);
        training == "Simple and Martial"
            || rule.category == "Simple"
            || (self.character_class == "Monk"
                && rule.kind == "Melee"
                && rule.properties.contains(&"Light"))
            || (self.character_class == "Rogue"
                && rule
                    .properties
                    .iter()
                    .any(|property| ["Finesse", "Light"].contains(property)))
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn weapon_attacks(&self) -> Vec<WeaponAttack> {
        self.inventory()
            .into_iter()
            .filter_map(|item| {
                let weapon = item.weapon.as_deref()?;
                let rule = self.weapon_rule(weapon, item.equipment_id.as_deref())?;
                let abilities: &[Ability] = if rule.kind == "Ranged" {
                    &[Ability::Dexterity]
                } else if rule.properties.contains(&"Finesse") || self.character_class == "Monk" {
                    &[Ability::Strength, Ability::Dexterity]
                } else {
                    &[Ability::Strength]
                };
                let modifier = abilities
                    .iter()
                    .map(|ability| self.abilities.modifier(*ability))
                    .max()
                    .unwrap_or(0);
                let mut attack_bonus = modifier
                    + if self.is_weapon_proficient(&rule) {
                        i16::from(self.proficiency_bonus())
                    } else {
                        0
                    };
                if self.class_choices.fighting_style.as_deref() == Some("Archery")
                    && rule.kind == "Ranged"
                {
                    attack_bonus += 2;
                }
                let suffix = if modifier == 0 {
                    String::new()
                } else {
                    format!("{modifier:+}")
                };
                let mut notes = Vec::new();
                if item.quantity > 1 {
                    notes.push(format!("Quantity {}", item.quantity));
                }
                if let Some(damage) = rule.versatile_damage {
                    notes.push(format!("Versatile {damage}{suffix}"));
                }
                if self.class_choices.weapon_masteries.contains(weapon) {
                    notes.push(format!("Mastery: {}", rule.mastery));
                }
                if rule.properties.contains(&"Heavy") {
                    let ability = if rule.kind == "Ranged" {
                        self.abilities.dexterity
                    } else {
                        self.abilities.strength
                    };
                    if ability < 13 {
                        notes.push("Heavy: attack rolls have Disadvantage".to_owned());
                    }
                }
                Some(WeaponAttack {
                    name: item.name,
                    attack_bonus,
                    damage: format!("{}{suffix}", rule.damage),
                    damage_type: rule.damage_type.to_owned(),
                    range: rule.long_range.map_or_else(
                        || format!("{} ft.", rule.normal_range),
                        |long| format!("{}/{long} ft.", rule.normal_range),
                    ),
                    properties: rule
                        .properties
                        .iter()
                        .map(|value| (*value).to_owned())
                        .collect(),
                    notes,
                })
            })
            .collect()
    }

    #[must_use]
    pub fn darkvision_range(&self) -> Option<u8> {
        match self.elf_lineage.as_deref() {
            Some("Drow") => Some(120),
            Some("High Elf" | "Wood Elf") => Some(60),
            _ => self.resolved_pack_species.as_ref().map_or_else(
                || srd::species_rule(&self.species).and_then(|rule| rule.darkvision_range),
                |rule| rule.darkvision_range,
            ),
        }
    }

    #[must_use]
    pub fn damage_resistances(&self) -> Vec<String> {
        let mut values = Vec::new();
        if self.species == "Dwarf" {
            values.push("Poison".to_owned());
        }
        if let Some(value) = self
            .dragonborn_ancestry
            .as_deref()
            .and_then(srd::dragonborn_damage_type)
        {
            values.push(value.to_owned());
        }
        if let Some(value) = self
            .tiefling_legacy
            .as_deref()
            .and_then(srd::tiefling_resistance)
        {
            values.push(value.to_owned());
        }
        values.dedup();
        values
    }

    #[must_use]
    pub fn spell_slots(&self) -> Vec<SpellSlotPool> {
        if let Some(spellcasting) = self
            .pack_class()
            .and_then(|class| class.spellcasting.as_ref())
        {
            return (spellcasting.spell_slots > 0)
                .then(|| SpellSlotPool {
                    level: 1,
                    total: spellcasting.spell_slots,
                    recovery: spellcasting.slot_recovery.clone(),
                })
                .into_iter()
                .collect();
        }
        srd::level_one_spell_slots(&self.character_class).map_or_else(
            Vec::new,
            |(level, total, recovery)| {
                vec![SpellSlotPool {
                    level,
                    total,
                    recovery: recovery.to_owned(),
                }]
            },
        )
    }

    fn spellcasting_profile(
        &self,
        source: String,
        ability: &str,
        slots: Vec<SpellSlotPool>,
        free_casts: Vec<String>,
    ) -> SpellcastingProfile {
        let modifier = self
            .abilities
            .modifier(Ability::try_from(ability).expect("validated spellcasting ability"));
        SpellcastingProfile {
            source,
            ability: ability.to_owned(),
            modifier,
            save_dc: 8 + modifier + i16::from(self.proficiency_bonus()),
            attack_bonus: modifier + i16::from(self.proficiency_bonus()),
            granted_spell_slots: slots,
            free_casts,
        }
    }

    #[must_use]
    pub fn spellcasting_profiles(&self) -> Vec<SpellcastingProfile> {
        let mut profiles = Vec::new();
        if let Some(spellcasting) = self
            .pack_class()
            .and_then(|class| class.spellcasting.as_ref())
        {
            profiles.push(self.spellcasting_profile(
                format!("{} Spellcasting", self.class_name()),
                &spellcasting.ability,
                self.spell_slots(),
                Vec::new(),
            ));
        } else if let Some(ability) = srd::class_spellcasting_ability(&self.character_class) {
            profiles.push(self.spellcasting_profile(
                format!("{} Spellcasting", self.character_class),
                ability,
                self.spell_slots(),
                Vec::new(),
            ));
        }
        profiles.extend(self.magic_initiate_choices.iter().map(|choice| {
            self.spellcasting_profile(
                format!("Magic Initiate ({})", choice.spell_list),
                &choice.spellcasting_ability,
                Vec::new(),
                vec![format!(
                    "{}: 1/Long Rest without a spell slot",
                    self.spell_name(&choice.level_one_spell)
                )],
            )
        }));
        let species_ability = self
            .elf_spellcasting_ability
            .as_deref()
            .or(self.gnome_spellcasting_ability.as_deref())
            .or(self.tiefling_spellcasting_ability.as_deref());
        if let Some(ability) = species_ability {
            let source = if let Some(lineage) = &self.elf_lineage {
                format!("Elven Lineage ({lineage})")
            } else if let Some(lineage) = &self.gnome_lineage {
                format!("Gnomish Lineage ({lineage})")
            } else if let Some(legacy) = &self.tiefling_legacy {
                format!("Fiendish Legacy ({legacy})")
            } else {
                "Species Spellcasting".to_owned()
            };
            let free_casts = if self.gnome_lineage.as_deref() == Some("Forest Gnome") {
                vec![format!(
                    "Speak with Animals: {}/Long Rest without a spell slot",
                    self.proficiency_bonus()
                )]
            } else {
                Vec::new()
            };
            profiles.push(self.spellcasting_profile(source, ability, Vec::new(), free_casts));
        }
        profiles
    }

    #[must_use]
    pub fn all_cantrips(&self) -> Vec<String> {
        let mut spells = Vec::new();
        let species: Vec<&str> = if let Some(lineage) = self.elf_lineage.as_deref() {
            vec![match lineage {
                "Drow" => "Dancing Lights",
                "High Elf" => "Prestidigitation",
                "Wood Elf" => "Druidcraft",
                _ => "",
            }]
        } else if let Some(lineage) = self.gnome_lineage.as_deref() {
            if lineage == "Forest Gnome" {
                vec!["Minor Illusion"]
            } else {
                vec!["Mending", "Prestidigitation"]
            }
        } else if let Some(legacy) = self.tiefling_legacy.as_deref() {
            vec![srd::tiefling_cantrip(legacy).unwrap_or(""), "Thaumaturgy"]
        } else {
            Vec::new()
        };
        for spell in species
            .into_iter()
            .map(str::to_owned)
            .chain(
                self.magic_initiate_choices
                    .iter()
                    .flat_map(|choice| choice.cantrips.iter().cloned()),
            )
            .chain(self.class_choices.cantrips.iter().cloned())
        {
            if !spell.is_empty() && !spells.contains(&spell) {
                spells.push(spell);
            }
        }
        spells
    }

    #[must_use]
    pub fn all_prepared_spells(&self) -> Vec<String> {
        let mut spells = Vec::new();
        if self.gnome_lineage.as_deref() == Some("Forest Gnome") {
            spells.push("Speak with Animals".to_owned());
        }
        for spell in self
            .magic_initiate_choices
            .iter()
            .map(|choice| choice.level_one_spell.clone())
            .chain(self.class_choices.prepared_spells.iter().cloned())
            .chain(
                srd::class_always_prepared(&self.character_class)
                    .iter()
                    .map(|value| (*value).to_owned()),
            )
        {
            if !spells.contains(&spell) {
                spells.push(spell);
            }
        }
        spells
    }

    #[must_use]
    pub fn spell_table_entries(&self) -> Vec<SpellTableEntry> {
        self.all_cantrips()
            .into_iter()
            .map(|spell| (0, spell))
            .chain(
                self.all_prepared_spells()
                    .into_iter()
                    .map(|spell| (1, spell)),
            )
            .filter_map(|(level, name)| {
                if let Some(rule) = self.pack_spell(&name) {
                    Some(SpellTableEntry {
                        level: rule.level,
                        name: rule.name.clone(),
                        casting_time: rule.casting_time.clone(),
                        range: rule.range.clone(),
                        concentration: rule.concentration,
                        ritual: rule.ritual,
                        required_material: rule.material.is_some(),
                        notes: rule.notes.clone(),
                    })
                } else {
                    let rule = srd::spell_rule(&name)?;
                    Some(SpellTableEntry {
                        level,
                        name,
                        casting_time: rule.casting_time.clone(),
                        range: rule.range.clone(),
                        concentration: rule.concentration,
                        ritual: rule.ritual,
                        required_material: rule.required_material.is_some(),
                        notes: rule.notes.clone(),
                    })
                }
            })
            .collect()
    }

    #[must_use]
    pub fn armor_training(&self) -> String {
        if let Some(rule) = self.pack_class() {
            rule.armor_training.join(", ")
        } else if self.class_choices.divine_order.as_deref() == Some("Protector") {
            "Light, Medium, Heavy, Shields".to_owned()
        } else if self.class_choices.primal_order.as_deref() == Some("Warden") {
            "Light, Medium, Shields".to_owned()
        } else {
            srd::class_rule(&self.character_class)
                .map_or("", |rule| rule.armor)
                .to_owned()
        }
    }

    #[must_use]
    pub fn weapon_proficiencies(&self) -> String {
        if let Some(rule) = self.pack_class() {
            rule.weapon_training.join(" and ")
        } else if self.class_choices.divine_order.as_deref() == Some("Protector")
            || self.class_choices.primal_order.as_deref() == Some("Warden")
        {
            "Simple and Martial".to_owned()
        } else {
            srd::class_rule(&self.character_class)
                .map_or("", |rule| rule.weapons)
                .to_owned()
        }
    }

    #[must_use]
    pub fn all_tool_proficiencies(&self) -> Vec<String> {
        let mut values: Vec<String> = self
            .background_tool()
            .into_iter()
            .map(str::to_owned)
            .collect();
        for tool in self
            .class_choices
            .tools
            .iter()
            .chain(&self.tool_proficiencies)
        {
            if !values.contains(tool) {
                values.push(tool.clone());
            }
        }
        values
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn class_traits(&self) -> Vec<String> {
        let mut traits: Vec<String> = self.pack_class().map_or_else(
            || {
                srd::class_features(&self.character_class)
                    .iter()
                    .map(|value| (*value).to_owned())
                    .collect()
            },
            |rule| rule.features.clone(),
        );
        if !self.class_choices.weapon_masteries.is_empty() {
            let selected = self
                .class_choices
                .weapon_masteries
                .iter()
                .map(|weapon| {
                    format!(
                        "{weapon} ({})",
                        srd::weapon_rule(weapon).map_or("", |rule| rule.mastery)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            traits.push(format!("Weapon Mastery: {selected}"));
        }
        if let Some(class) = self.pack_class() {
            for choice in &class.choices {
                let selected = self
                    .class_choices
                    .pack_choices
                    .get(&choice.id)
                    .into_iter()
                    .flatten()
                    .filter_map(|id| choice.options.iter().find(|option| option.id == *id))
                    .map(|option| {
                        option.description.as_ref().map_or_else(
                            || option.name.clone(),
                            |description| format!("{} — {description}", option.name),
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if !selected.is_empty() {
                    traits.push(format!("{}: {selected}", choice.label));
                }
            }
        }
        if !self.class_choices.tools.is_empty() {
            traits.push(format!(
                "Class Tools: {}",
                self.class_choices
                    .tools
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.class_choices.expertise.is_empty() {
            traits.push(format!(
                "Expertise: {}",
                self.class_choices
                    .expertise
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if let Some(value) = &self.class_choices.fighting_style {
            traits.push(format!("Fighting Style: {value}"));
        }
        if let Some(value) = &self.class_choices.eldritch_invocation {
            traits.push(format!("Eldritch Invocation: {value}"));
        }
        if let Some(value) = &self.class_choices.additional_language {
            traits.push(format!("Thieves' Cant: additional language ({value})"));
        }
        if !self.class_choices.cantrips.is_empty() {
            traits.push(format!(
                "Cantrips: {}",
                self.class_choices
                    .cantrips
                    .iter()
                    .map(|spell| self.spell_name(spell).to_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.class_choices.spellbook_spells.is_empty() {
            traits.push(format!(
                "Spellbook: {}",
                self.class_choices
                    .spellbook_spells
                    .iter()
                    .map(|spell| self.spell_name(spell).to_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.class_choices.prepared_spells.is_empty() {
            traits.push(format!(
                "Prepared Spells: {}",
                self.class_choices
                    .prepared_spells
                    .iter()
                    .map(|spell| self.spell_name(spell).to_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        traits
    }

    #[must_use]
    pub fn species_name(&self) -> &str {
        self.resolved_pack_species
            .as_ref()
            .map_or_else(|| self.species.as_str(), |rule| rule.name.as_str())
    }

    #[must_use]
    pub fn species_traits(&self) -> Vec<String> {
        let mut traits: Vec<String> = self.resolved_pack_species.as_ref().map_or_else(
            || {
                srd::species_traits(&self.species)
                    .iter()
                    .map(|value| (*value).to_owned())
                    .collect()
            },
            |rule| rule.traits.clone(),
        );
        if let Some(value) = &self.dragonborn_ancestry {
            traits.push(format!(
                "Draconic Ancestry: {value} ({})",
                srd::dragonborn_damage_type(value).unwrap_or("")
            ));
        }
        if let Some(value) = &self.elf_lineage {
            traits.push(format!("Elven Lineage: {value}"));
            traits.push(format!(
                "Keen Senses: {}",
                self.elf_keen_senses_skill.as_deref().unwrap_or("")
            ));
        }
        if let Some(value) = &self.gnome_lineage {
            traits.push(format!("Gnomish Lineage: {value}"));
        }
        if let Some(value) = &self.goliath_ancestry {
            traits.push(format!("Giant Ancestry: {value}"));
        }
        if let Some(value) = &self.human_skill {
            traits.push(format!("Skillful: {value}"));
        }
        if let Some(value) = &self.human_origin_feat {
            traits.push(format!("Versatile: {value}"));
        }
        if let Some(value) = &self.tiefling_legacy {
            traits.push(format!("Fiendish Legacy: {value}"));
        }
        if let Some(value) = self.darkvision_range() {
            traits.push(format!("Darkvision: {value} ft."));
        }
        if !self.damage_resistances().is_empty() {
            traits.push(format!(
                "Damage Resistance: {}",
                self.damage_resistances().join(", ")
            ));
        }
        let species_source = ["Elven Lineage", "Gnomish Lineage", "Fiendish Legacy"];
        if let Some(profile) = self.spellcasting_profiles().into_iter().find(|profile| {
            species_source
                .iter()
                .any(|source| profile.source.starts_with(source))
        }) {
            traits.push(profile.summary());
        }
        let species_cantrips: Vec<String> = if self.elf_lineage.is_some()
            || self.gnome_lineage.is_some()
            || self.tiefling_legacy.is_some()
        {
            let class_and_feat: BTreeSet<String> = self
                .class_choices
                .cantrips
                .iter()
                .cloned()
                .chain(
                    self.magic_initiate_choices
                        .iter()
                        .flat_map(|choice| choice.cantrips.iter().cloned()),
                )
                .collect();
            self.all_cantrips()
                .into_iter()
                .filter(|spell| !class_and_feat.contains(spell))
                .collect()
        } else {
            Vec::new()
        };
        if !species_cantrips.is_empty() {
            traits.push(format!("Cantrips: {}", species_cantrips.join(", ")));
        }
        traits
    }

    #[must_use]
    pub fn origin_feat_traits(&self) -> Vec<String> {
        let mut traits = Vec::new();
        if self.background_feat() == Some("Alert")
            || self.human_origin_feat.as_deref() == Some("Alert")
        {
            traits.push("Alert: Initiative Proficiency; Initiative Swap".to_owned());
        }
        if self.background_feat() == Some("Savage Attacker")
            || self.human_origin_feat.as_deref() == Some("Savage Attacker")
        {
            traits.push("Savage Attacker: roll weapon damage dice twice once per turn".to_owned());
        }
        traits.extend(self.magic_initiate_choices.iter().map(|choice| {
            format!(
                "Magic Initiate ({}): {}; {}",
                choice.spell_list,
                choice
                    .cantrips
                    .iter()
                    .map(|spell| self.spell_name(spell))
                    .collect::<Vec<_>>()
                    .join(", "),
                self.spell_name(&choice.level_one_spell)
            )
        }));
        if !self.skilled_proficiencies.is_empty() {
            traits.push(format!(
                "Skilled: {}",
                self.skilled_proficiencies
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        traits
    }

    #[must_use]
    pub fn class_resources(&self) -> Vec<ClassResource> {
        if let Some(class) = self.pack_class() {
            return class
                .resources
                .iter()
                .map(|resource| ClassResource {
                    name: resource.name.clone(),
                    maximum: resource.maximum,
                    unit: resource.unit.clone(),
                    recovery: resource.recovery.clone(),
                    detail: resource.detail.clone(),
                })
                .collect();
        }
        let resource = match self.character_class.as_str() {
            "Barbarian" => ClassResource {
                name: "Rage".to_owned(),
                maximum: 2,
                unit: "uses".to_owned(),
                detail: Some("+2 Rage damage".to_owned()),
                recovery: "regain 1 on Short Rest; all on Long Rest".to_owned(),
            },
            "Bard" => ClassResource {
                name: "Bardic Inspiration".to_owned(),
                maximum: self.abilities.modifier(Ability::Charisma).max(1),
                unit: "d6 uses".to_owned(),
                detail: None,
                recovery: "regain all on Long Rest".to_owned(),
            },
            "Fighter" => ClassResource {
                name: "Second Wind".to_owned(),
                maximum: 2,
                unit: "uses".to_owned(),
                detail: Some(format!("heal 1d10+{} HP", self.level)),
                recovery: "regain 1 on Short Rest; all on Long Rest".to_owned(),
            },
            "Paladin" => ClassResource {
                name: "Lay on Hands".to_owned(),
                maximum: i16::from(5 * self.level),
                unit: "HP".to_owned(),
                detail: None,
                recovery: "replenishes on Long Rest".to_owned(),
            },
            "Ranger" => ClassResource {
                name: "Favored Enemy".to_owned(),
                maximum: 2,
                unit: "free Hunter's Mark casts".to_owned(),
                detail: None,
                recovery: "regain all on Long Rest".to_owned(),
            },
            "Sorcerer" => ClassResource {
                name: "Innate Sorcery".to_owned(),
                maximum: 2,
                unit: "uses".to_owned(),
                detail: None,
                recovery: "regain all on Long Rest".to_owned(),
            },
            "Warlock" => ClassResource {
                name: "Pact Magic".to_owned(),
                maximum: 1,
                unit: "level-1 slot".to_owned(),
                detail: None,
                recovery: "regain on Short or Long Rest".to_owned(),
            },
            "Wizard" => ClassResource {
                name: "Arcane Recovery".to_owned(),
                maximum: 1,
                unit: "use".to_owned(),
                detail: Some(format!(
                    "recover {} level(s) of spell slots",
                    self.level.div_ceil(2)
                )),
                recovery: "regain on Long Rest".to_owned(),
            },
            _ => return Vec::new(),
        };
        vec![resource]
    }

    #[must_use]
    pub fn passive_perception(&self) -> i16 {
        10 + self.skill_modifier("Perception")
    }

    /// Serialize the canonical record with stable pretty formatting.
    ///
    /// # Errors
    ///
    /// Returns an error if the record cannot be represented as JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map(|mut value| {
                value.push('\n');
                value
            })
            .map_err(|error| error.to_string())
    }

    #[must_use]
    pub fn proficiency_bonus(&self) -> u8 {
        2 + (self.level - 1) / 4
    }
}

#[cfg(test)]
mod tests {
    use super::Character;

    #[test]
    fn derives_combat_values_from_the_canonical_fixture() {
        let character =
            Character::from_json(include_str!("../../fixtures/complete-character.json"))
                .expect("valid fixture");
        assert_eq!(character.hit_points(), 9);
        assert_eq!(character.armor_class(), 14);
        assert_eq!(character.passive_perception(), 12);
    }
}
