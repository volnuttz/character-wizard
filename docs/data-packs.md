# Campaign data packs

Campaign data packs are opt-in directories passed with `--data`. Built-in rules
remain strictly sourced from the supplied SRD; no pack is loaded unless the user
explicitly selects one.

Declared content files must be JSON arrays. Basic custom species and custom
background records are mechanically active for interactive creation, editing,
constrained random generation, and quick creation. Typed custom equipment is
active only when granted by a custom background. Custom level 0 and level 1
spells are active for supported Magic Initiate and built-in class spell choices.
Basic declarative custom classes are active across the same creation paths.

## Version 1 manifest

Place `data-pack.json` at the pack directory root:

```json
{
  "format_version": 1,
  "id": "my-campaign",
  "version": 1,
  "name": "My Campaign",
  "files": {
    "species": "species.json",
    "backgrounds": "backgrounds.json",
    "equipment": "equipment.json",
    "spells": "spells.json",
    "classes": "classes.json"
  }
}
```

`id` uses lowercase letters, digits, and hyphens. `version` is a positive pack
revision and must change whenever published mechanics change. The supported file families
are `species`, `backgrounds`, `equipment`, `spells`, and `classes`. Paths must
stay inside the pack directory, and each declared file must contain a JSON array.

Use it with any command, for example:

```console
character-wizard create --data ./my-campaign
character-wizard random --data ./my-campaign
character-wizard random --data ./my-campaign --background lunar-scout
```

Characters created with `--data` record the pack ID, pack version, and format version in their
canonical JSON. Loading, editing, validating, or rendering one of those
characters requires passing the matching pack again with `--data`.

## Species records

Species IDs are add-only and cannot collide with built-in SRD names. A basic
species record supports size, speed, darkvision, and descriptive traits:

```json
[
  {
    "id": "moonfolk",
    "name": "Moonfolk",
    "sizes": ["Small"],
    "speed": 35,
    "darkvision_range": 60,
    "traits": ["Moonlit Step"]
  }
]
```

Generate one by stable ID or display name:

```console
character-wizard random --data ./my-campaign --species moonfolk
```

Custom species do not inherit SRD-only lineage, ancestry, Skillful, Versatile,
or legacy choices.

## Background records

Background IDs are add-only and cannot collide with built-in SRD names. A
background defines three eligible abilities, two SRD skills, one supported
Origin feat, one SRD tool proficiency, and its starting equipment and gold:

```json
[
  {
    "id": "lunar-scout",
    "name": "Lunar Scout",
    "abilities": ["dexterity", "wisdom", "charisma"],
    "skills": ["Perception", "Survival"],
    "feat": "Alert",
    "tool": "Navigator's Tools",
    "equipment": [
      { "equipment_id": "moonblade" },
      { "name": "Arrow", "quantity": 20 },
      { "name": "Navigator's Tools" }
    ],
    "equipment_gold": 12,
    "gold_alternative": 50
  }
]
```

`quantity` defaults to 1. Each grant defines exactly one `name` or
`equipment_id`. A name grants an existing built-in or display-only item; an
equipment ID must resolve to a typed record in the same pack. `equipment_gold`
accompanies the package, while `gold_alternative` replaces the package.

Supported feats are `Alert`, `Magic Initiate`, `Savage Attacker`, and `Skilled`.
A `Magic Initiate` background must also set `magic_initiate_list` to `Cleric`,
`Druid`, or `Wizard`; other feats must omit it. The normal feat choices and
duplicate-proficiency rules apply during creation and validation.

## Equipment records

Equipment IDs are add-only and cannot collide with built-in weapon, armor, or
shield names. Custom equipment is intentionally limited to custom-background
packages; it is not added to class packages, shopping, or independent equipment
selection.

Each record has an ID, display name, and nested `kind`. For example:

```json
[
  {
    "id": "moonblade",
    "name": "Moonblade",
    "kind": {
      "type": "weapon",
      "category": "Simple",
      "kind": "Melee",
      "properties": ["Finesse", "Light"],
      "mastery": "Vex",
      "damage": "1d8",
      "damage_type": "Radiant",
      "normal_range": 5
    }
  },
  {
    "id": "moonweave",
    "name": "Moonweave Armor",
    "kind": {
      "type": "armor",
      "category": "Light",
      "base_ac": 13
    }
  },
  {
    "id": "moonward",
    "name": "Moonward",
    "kind": {
      "type": "shield",
      "armor_class_bonus": 3
    }
  }
]
```

Supported types are `weapon`, `armor`, `shield`, `ammunition`, and `gear`.
Weapons support SRD-compatible category, melee/ranged kind, properties, mastery,
damage dice, damage type, range, and optional `long_range` and
`versatile_damage`. Armor supports category, base AC, optional Dexterity cap,
and optional Strength requirement. These mechanics feed the existing attacks,
AC, speed, inventory, character review, and PDF projections.

## Spell records

Spell IDs are add-only and cannot collide by display name with an SRD spell.
The ID is stored in canonical character JSON; prompts, reviews, and rendered
spell tables use the display name.

The current level-1 creation scope accepts cantrips and level 1 spells:

```json
[
  {
    "id": "moon-spark",
    "name": "Moon Spark",
    "level": 0,
    "school": "Evocation",
    "lists": ["Wizard"],
    "casting_time": "Action",
    "range": "60 feet",
    "components": ["V", "S"],
    "notes": "Duration: Instantaneous",
    "tags": ["Damage"]
  },
  {
    "id": "moon-shield",
    "name": "Moon Shield",
    "level": 1,
    "school": "Abjuration",
    "lists": ["Wizard"],
    "casting_time": "Reaction",
    "range": "Self",
    "components": ["V", "S", "M"],
    "material": "a moonstone shard",
    "concentration": true,
    "notes": "Duration: up to 1 minute",
    "tags": ["Defense"]
  }
]
```

Supported schools are Abjuration, Conjuration, Divination, Enchantment,
Evocation, Illusion, Necromancy, and Transmutation. Supported lists are Bard,
Cleric, Druid, Paladin, Ranger, Sorcerer, Warlock, and Wizard. Components are
`V`, `S`, and `M`; `material` is required exactly when `M` is present.
`concentration` and `ritual` default to false, while `tags` defaults to an empty
array. Ritual spells must be level 1 in this initial slice.

Pack spells join eligible built-in class catalogs and Magic Initiate catalogs
when their `lists` membership matches. Loading a character with a pack spell
requires the exact recorded pack version, just like custom species,
backgrounds, and equipment. School and tags are validated descriptive metadata;
packs cannot attach executable mechanics or new prompt behavior to a spell.

## Class records

Class IDs are add-only and cannot collide with built-in SRD class names. The ID
is stored in canonical character JSON; creation, review, collection, `show`, and
PDF output use the display name.

The initial class slice is level-1, declarative, and non-spellcasting:

```json
[
  {
    "id": "moon-warden",
    "name": "Moon Warden",
    "hit_die": 10,
    "saving_throws": ["strength", "wisdom"],
    "skill_count": 2,
    "skills": [
      "Animal Handling",
      "Athletics",
      "Insight",
      "Nature",
      "Perception",
      "Survival"
    ],
    "armor_training": ["Light", "Medium", "Shields"],
    "weapon_training": ["Simple", "Martial"],
    "equipment": [
      { "name": "Longsword" },
      { "name": "Scale Mail" },
      { "name": "Shield" }
    ],
    "equipment_gold": 10,
    "starting_gold": 150,
    "features": ["Moonlit Vigil", "Lunar Challenge"],
    "weapon_mastery_count": 2
  }
]
```

Supported Hit Dice are d6, d8, d10, and d12. A class grants exactly two distinct
ability saving throws and selects `skill_count` different proficiencies from its
SRD skill pool. Armor training accepts `Light`, `Medium`, `Heavy`, and `Shields`;
weapon training accepts `Simple` and `Martial`. Weapon Mastery may select up to
three built-in weapons from a trained category.

The fixed `A` equipment package uses named items and carries `equipment_gold`;
the `Gold` route skips the package and grants `starting_gold`. Class equipment
does not accept custom `equipment_id` references in this first slice, preserving
the background-owned custom-equipment boundary.

Features are fixed display descriptions. Optional declarative extensions add
stable descriptive choices, fixed level-1 resources, and basic spellcasting:

```json
{
  "choices": [
    {
      "id": "lunar-calling",
      "label": "Lunar Calling",
      "count": 1,
      "options": [
        {
          "id": "guardian",
          "name": "Guardian",
          "description": "Stand watch over allies beneath the moon."
        },
        {
          "id": "seer",
          "name": "Seer"
        }
      ]
    }
  ],
  "resources": [
    {
      "name": "Moonlight Resolve",
      "maximum": 2,
      "unit": "uses",
      "recovery": "regain all on Long Rest",
      "detail": "fuel Moon Warden features"
    }
  ],
  "spellcasting": {
    "ability": "wisdom",
    "spell_list": "Druid",
    "cantrip_count": 2,
    "prepared_spell_count": 2,
    "spell_slots": 2,
    "slot_recovery": "Long Rest"
  }
}
```

Choice and option IDs use the same stable lowercase identifier format as other
pack content. Selected option IDs are stored under `class_choices.pack_choices`
in canonical character JSON; labels, names, and descriptions are resolved from
the exact pack for reviews and PDFs. Choices are descriptive and do not grant
hidden calculations or execute behavior.

Resources have a fixed positive maximum, unit, recovery text, and optional
detail. They are level-1 display and tracking metadata; formulas based on ability
scores or character level are not supported.

Spellcasting selects one supported Bard, Cleric, Druid, Paladin, Ranger,
Sorcerer, Warlock, or Wizard list, one Intelligence, Wisdom, or Charisma ability,
creation-time cantrip and prepared-spell counts, and a fixed number of level-1
slots. Pack spells declaring membership in the borrowed list join its SRD spell
catalog. The custom class does not inherit the source class's features,
always-prepared spells, spellbook, resources, or bespoke prompts.

Custom classes still cannot define tools, Expertise, executable scripts,
arbitrary formulas or effects, class-specific prompt code, or progression beyond
the current level-1 creation scope.
