# Campaign data packs

Campaign data packs are opt-in directories passed with `--data`. Built-in rules
remain strictly sourced from the supplied SRD; no pack is loaded unless the user
explicitly selects one.

Declared content files must be JSON arrays. Basic custom species records are
mechanically active for interactive creation and constrained random generation;
backgrounds, equipment, and spells are still validated as files but are not
active yet. Quick creation also includes pack species in its random catalog.

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
