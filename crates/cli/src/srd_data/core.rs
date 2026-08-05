//! Shared identifiers, skills, tools, and creation constants.

pub const ABILITIES: [&str; 6] = [
    "strength",
    "dexterity",
    "constitution",
    "intelligence",
    "wisdom",
    "charisma",
];
pub const CLASS_NAMES: [&str; 12] = [
    "Barbarian",
    "Bard",
    "Cleric",
    "Druid",
    "Fighter",
    "Monk",
    "Paladin",
    "Ranger",
    "Rogue",
    "Sorcerer",
    "Warlock",
    "Wizard",
];
pub const BACKGROUND_NAMES: [&str; 4] = ["Acolyte", "Criminal", "Sage", "Soldier"];
pub const SPECIES_NAMES: [&str; 9] = [
    "Dragonborn",
    "Dwarf",
    "Elf",
    "Gnome",
    "Goliath",
    "Halfling",
    "Human",
    "Orc",
    "Tiefling",
];
pub const WEAPON_NAMES: [&str; 38] = [
    "Club",
    "Dagger",
    "Greatclub",
    "Handaxe",
    "Javelin",
    "Light Hammer",
    "Mace",
    "Quarterstaff",
    "Sickle",
    "Spear",
    "Dart",
    "Light Crossbow",
    "Shortbow",
    "Sling",
    "Battleaxe",
    "Flail",
    "Glaive",
    "Greataxe",
    "Greatsword",
    "Halberd",
    "Lance",
    "Longsword",
    "Maul",
    "Morningstar",
    "Pike",
    "Rapier",
    "Scimitar",
    "Shortsword",
    "Trident",
    "Warhammer",
    "War Pick",
    "Whip",
    "Blowgun",
    "Hand Crossbow",
    "Heavy Crossbow",
    "Longbow",
    "Musket",
    "Pistol",
];
pub const ALIGNMENTS: [&str; 9] = [
    "Lawful Good",
    "Neutral Good",
    "Chaotic Good",
    "Lawful Neutral",
    "Neutral",
    "Chaotic Neutral",
    "Lawful Evil",
    "Neutral Evil",
    "Chaotic Evil",
];
pub const STANDARD_LANGUAGES: [&str; 9] = [
    "Common Sign Language",
    "Draconic",
    "Dwarvish",
    "Elvish",
    "Giant",
    "Gnomish",
    "Goblin",
    "Halfling",
    "Orc",
];
pub const SPELLCASTING_ABILITIES: [&str; 3] = ["intelligence", "wisdom", "charisma"];
pub const STANDARD_ARRAY: [u8; 6] = [15, 14, 13, 12, 10, 8];
pub const POINT_BUY_BUDGET: u8 = 27;
pub const ORIGIN_FEATS: [&str; 4] = ["Alert", "Magic Initiate", "Savage Attacker", "Skilled"];
pub const FIGHTING_STYLES: [&str; 4] = [
    "Archery",
    "Defense",
    "Great Weapon Fighting",
    "Two-Weapon Fighting",
];
pub const WARLOCK_INVOCATIONS: [&str; 5] = [
    "Armor of Shadows",
    "Eldritch Mind",
    "Pact of the Blade",
    "Pact of the Chain",
    "Pact of the Tome",
];
pub const ARTISAN_TOOLS: [&str; 17] = [
    "Alchemist's Supplies",
    "Brewer's Supplies",
    "Calligrapher's Supplies",
    "Carpenter's Tools",
    "Cartographer's Tools",
    "Cobbler's Tools",
    "Cook's Utensils",
    "Glassblower's Tools",
    "Jeweler's Tools",
    "Leatherworker's Tools",
    "Mason's Tools",
    "Painter's Supplies",
    "Potter's Tools",
    "Smith's Tools",
    "Tinker's Tools",
    "Weaver's Tools",
    "Woodcarver's Tools",
];
pub const MUSICAL_INSTRUMENTS: [&str; 10] = [
    "Musical Instrument (Bagpipes)",
    "Musical Instrument (Drum)",
    "Musical Instrument (Dulcimer)",
    "Musical Instrument (Flute)",
    "Musical Instrument (Horn)",
    "Musical Instrument (Lute)",
    "Musical Instrument (Lyre)",
    "Musical Instrument (Pan Flute)",
    "Musical Instrument (Shawm)",
    "Musical Instrument (Viol)",
];

#[must_use]
pub fn is_tool(name: &str) -> bool {
    ARTISAN_TOOLS.contains(&name)
        || MUSICAL_INSTRUMENTS.contains(&name)
        || [
            "Disguise Kit",
            "Forgery Kit",
            "Gaming Set (Dice)",
            "Gaming Set (Dragonchess)",
            "Gaming Set (Playing Cards)",
            "Gaming Set (Three-Dragon Ante)",
            "Herbalism Kit",
            "Navigator's Tools",
            "Poisoner's Kit",
            "Thieves' Tools",
        ]
        .contains(&name)
}
pub const SKILLS: [&str; 18] = [
    "Acrobatics",
    "Animal Handling",
    "Arcana",
    "Athletics",
    "Deception",
    "History",
    "Insight",
    "Intimidation",
    "Investigation",
    "Medicine",
    "Nature",
    "Perception",
    "Performance",
    "Persuasion",
    "Religion",
    "Sleight of Hand",
    "Stealth",
    "Survival",
];

pub(super) const BARBARIAN_SKILLS: &[&str] = &[
    "Animal Handling",
    "Athletics",
    "Intimidation",
    "Nature",
    "Perception",
    "Survival",
];
pub(super) const BARD_SKILLS: &[&str] = &SKILLS;
pub(super) const CLERIC_SKILLS: &[&str] =
    &["History", "Insight", "Medicine", "Persuasion", "Religion"];
pub(super) const DRUID_SKILLS: &[&str] = &[
    "Animal Handling",
    "Arcana",
    "Insight",
    "Medicine",
    "Nature",
    "Perception",
    "Religion",
    "Survival",
];
pub(super) const FIGHTER_SKILLS: &[&str] = &[
    "Acrobatics",
    "Animal Handling",
    "Athletics",
    "History",
    "Insight",
    "Intimidation",
    "Perception",
    "Persuasion",
    "Survival",
];
pub(super) const MONK_SKILLS: &[&str] = &[
    "Acrobatics",
    "Athletics",
    "History",
    "Insight",
    "Religion",
    "Stealth",
];
pub(super) const PALADIN_SKILLS: &[&str] = &[
    "Athletics",
    "Insight",
    "Intimidation",
    "Medicine",
    "Persuasion",
    "Religion",
];
pub(super) const RANGER_SKILLS: &[&str] = &[
    "Animal Handling",
    "Athletics",
    "Insight",
    "Investigation",
    "Nature",
    "Perception",
    "Stealth",
    "Survival",
];
pub(super) const ROGUE_SKILLS: &[&str] = &[
    "Acrobatics",
    "Athletics",
    "Deception",
    "Insight",
    "Intimidation",
    "Investigation",
    "Perception",
    "Persuasion",
    "Sleight of Hand",
    "Stealth",
];
pub(super) const SORCERER_SKILLS: &[&str] = &[
    "Arcana",
    "Deception",
    "Insight",
    "Intimidation",
    "Persuasion",
    "Religion",
];
pub(super) const WARLOCK_SKILLS: &[&str] = &[
    "Arcana",
    "Deception",
    "History",
    "Intimidation",
    "Investigation",
    "Nature",
    "Religion",
];
pub(super) const WIZARD_SKILLS: &[&str] = &[
    "Arcana",
    "History",
    "Insight",
    "Investigation",
    "Medicine",
    "Nature",
    "Religion",
];
