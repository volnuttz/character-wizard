# Campaign data packs

Campaign data packs are opt-in directories passed with `--data`. Built-in rules
remain strictly sourced from the supplied SRD; no pack is loaded unless the user
explicitly selects one.

Declared content files must be JSON arrays. Basic custom species and custom
background records are mechanically active for interactive creation, editing,
constrained random generation, and quick creation. Typed custom equipment is
active only when granted by a custom background. Spells are still validated as
files but are not active yet.

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
    "spells": "spells.json"
  }
}
```

`id` uses lowercase letters, digits, and hyphens. `version` is a positive pack
revision and must change whenever published mechanics change. The supported file families
are `species`, `backgrounds`, `equipment`, and `spells`. Paths must stay inside
the pack directory, and each declared file must contain a JSON array.

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
