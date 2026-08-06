# SRD content map

## Code responsibilities

- `src/srd_data`: immutable source data and rule metadata.
- `src/domain`: schema validation, calculated values, and JSON round trips.
- `src/creation`: ordering and collection of interactive choices.
- `src/pdf_renderer`: projection of a valid character onto the sheet.
- `src/main.rs`: command boundaries, errors, and terminal presentation.

## Character-creation sequence

Follow SRD 5.2.1 pages 19–23: choose class; determine background, species, and
languages; determine ability scores; choose alignment; fill derived details.

## Current invariants

- Package: `character-wizard-cli`; command: `character-wizard`.
- Rust: `1.88.0`, edition 2024.
- Characters start at level 1 and 0 XP.
- Background skill proficiencies combine with non-duplicating class choices.
- Background increases affect only its three listed abilities and cannot raise a
  score above 20.
- Proficiency bonus is derived from level.
- Dwarf toughness contributes to maximum HP.
- Human and Tiefling explicitly choose Small or Medium; every other current SRD
  species has a fixed size.
- JSON is canonical; PDFs are generated artifacts.
- Character JSON uses only the current schema. Do not add schema versions,
  migrations, compatibility aliases, or legacy-shape fallbacks.
- The official character-sheet template is external and never a crate or release
  asset. Runtime resolution checks `--template`, the current directory, and the
  validated user cache before downloading and caching a supported copy.

## Test routing

- Rule calculations and validation: focused tests under `src/domain/`.
- Score generation and prompt helpers: focused tests under `src/creation/`.
- SRD inventories and lookups: focused tests under `src/srd_data/`.
- Field projection and PDF write/read-back: focused tests under
  `src/pdf_renderer/`, using the development fixture when required.
