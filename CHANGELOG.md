# Changelog

All notable changes to character-wizard are documented here. The project follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Campaign data packs can define custom backgrounds with stable IDs, ability
  increases, skills, Origin feats and subchoices, tool proficiencies, starting
  equipment, and gold alternatives.
- Custom backgrounds are available in interactive creation, editing, quick
  creation, and constrained random generation through `random --background`.
- Custom-background mechanics are validated on exact pack resolution and are
  included in character summaries, derived values, inventories, and PDFs.
- Custom backgrounds can grant typed custom weapons, armor, shields,
  ammunition, and gear by stable equipment ID; their supported mechanics flow
  into attacks, AC, speed, inventory summaries, and PDFs.
- Campaign data packs can define validated level 0 and level 1 spells with
  stable IDs, display metadata, components, tags, and supported class-list
  membership.
- Custom spells participate in eligible Magic Initiate and built-in class spell
  choices across interactive, edit, random, and quick creation. Character JSON
  retains stable spell IDs while reviews and PDF spell tables show pack names
  and casting metadata.
- Campaign data packs can define basic non-spellcasting level-1 classes with
  stable IDs, Hit Dice, saving throws, skill choices, armor and weapon training,
  fixed starting equipment or gold, feature descriptions, and optional Weapon
  Mastery choices.
- Custom classes participate in interactive, edit, constrained random, and
  quick creation; their HP, saves, proficiencies, inventory, gold, summaries,
  and existing PDF fields use the resolved pack mechanics and display name.
- Custom classes can additionally define reusable descriptive choices with
  stable option IDs, fixed level-1 resources, and optional spellcasting that
  borrows a supported SRD list and includes compatible pack spells. Choices,
  resources, spellcasting statistics, slots, and spell tables are validated and
  rendered through existing character and PDF projections.

## [0.5.1] - 2026-08-06

### Changed

- `create` now defaults its JSON and filled-PDF output paths from the character
  name (for example, `legolas.json` and `legolas.pdf`). The `--json` and
  `--output` options continue to override either path.

## [0.5.0] - 2026-08-06

### Changed

- Renamed the project, executable, native archives, cache environment variable,
  repository metadata, and user-facing documentation from `pc-wizard` to
  `character-wizard`.
- Consolidated the migration-era Cargo workspace into one
  `character-wizard-cli` binary crate with encapsulated SRD data, domain,
  creation, template-resolution, PDF-renderer, and CLI modules.
- Moved the embedded spell catalog into `data/spells.json` while preserving its
  compile-time inclusion in the native executable.
- Updated contributor guidance and repository skills for the single-crate Rust
  layout, current module paths, focused test commands, and release naming.
- Added manual-dispatch recovery for every required release workflow when an
  authenticated push is intentionally suppressed from triggering GitHub Actions.
- Scoped workflow concurrency by commit so delayed events for an older revision
  cannot cancel release checks for a newer revision.

This release does not intentionally change the canonical character JSON schema,
SRD rules, interactive creation behavior, or rendered character-sheet values.

## [0.4.0] - 2026-07-16

### Added

- `create` now resolves the official character sheet from `--template`, the
  current directory, or a validated user-cache copy, and visibly downloads,
  validates, and caches it when no supported copy exists.
- Scripted end-to-end wizard scenarios now cover complete creation and backward
  stage navigation in addition to prompt-port injection.

### Fixed

- Standard-language prompts no longer attach species descriptions to the
  `Halfling` and `Orc` language names.
- Exact-count selections now state the required number directly for skills,
  spells, cantrips, masteries, tools, and languages.
- Wizard navigation now consistently uses Escape for Back and no longer
  advertises an unsupported typed `back` command.

### Changed

- Equipment package contents now appear directly in selectable rows, and the
  final review uses a structured character summary instead of raw JSON.
- Domain validation and derived calculations, SRD catalogs, PDF rendering, and
  creation workflow code were split into focused modules without changing the
  canonical character schema.
- Retired the legacy Python implementation, Python packaging and test
  configuration, migration-only contracts, and stored benchmark artifacts after
  the verified `v0.3.0` native release.
- Native release smoke tests now use a Rust-owned canonical character fixture;
  historical parity and performance results remain documented.

## [0.3.0] - 2026-07-16

### Added

- A production Rust 1.88 native CLI with separate SRD data, domain, interactive
  creation, PDF rendering, CLI, and integration-test crates.
- Frozen Python-oracle parity matrices for every class, background, species,
  creation spell, derived value, and all 375 supported PDF projections.
- Native interactive suggested-array, standard-array, 4d6-drop-lowest, and
  27-point-buy workflows with background boosts, checkpoints, resume, editing,
  review, and cancellation-safe output behavior.
- Rust-native cross-platform quality, coverage, dependency/license auditing,
  binary smoke testing, archive/checksum packaging, and benchmark artifacts.

### Changed

- Production builds and releases now ship the optimized Rust executable directly;
  Python, uv, PyInstaller, wheels, and source distributions are no longer part of
  the shipping path.
- The native renderer validates the complete 425-entry AcroForm catalog and
  reproduces the frozen 375-value projection with dynamic checkbox on-states and
  auto-sized text appearances.
- Minimum supported Rust is 1.88.0. The official sheet remains external.

### Performance

- On the Linux x86-64 migration baseline, the 1.61 MiB optimized binary measured
  roughly 2.1–2.4 ms warm for help/version/show and 43.7 ms for JSON plus PDF
  creation, compared with roughly 437–608 ms and 1.34 s for the Python oracle.

## [0.2.1] - 2026-07-15

This release contains no user-facing character-creation or PDF-output changes.

### Added

- Broader parameterized and property-based validation coverage for classes,
  backgrounds, species, ability scores, and point-buy constraints.
- Automated weekly dependency updates and scheduled vulnerability auditing.
- Contributor guidance, a code of conduct, issue templates, and a pull-request
  checklist.
- CI coverage reporting with an enforced 85% line-coverage minimum.

### Changed

- Development distributions now include the contributor documentation while
  continuing to exclude repository-only PDF assets.

## [0.2.0] - 2026-07-13

### Fixed

- Standard-array, rolled-score, and point-buy menus now wrap numeric values in
  labeled Questionary choices, preventing integer options from crashing during
  ability-score assignment.

### Changed

- Repository-only SRD and character-sheet PDF fixtures now live under `assets/`;
  they remain excluded from wheels, source distributions, executables, and release
  assets.
- Character JSON is explicitly current-schema-only: files carry no schema version,
  and the project does not provide migrations or legacy-shape fallbacks. The binary
  smoke fixture now contains the complete current canonical shape, and character
  size must be present rather than being inferred when omitted.
- Character JSON now stores class skills and selected standard languages as user
  choices, while aggregate skills, languages, equipment, combat values, and
  spellcasting values are exposed as derived data.
- Alignment, language, skill-count, duplicate-proficiency, and other cross-field
  choices are validated at the model boundary.

### Added

- Interactive menus now show contextual SRD details for the highlighted class,
  background, species option, feat, spell, weapon mastery, skill, invocation, and
  equipment package without changing the selected value.
- Incomplete interactive sessions are checkpointed by stage and can be resumed;
  completed characters receive a final review with navigation back to each stage
  and confirmation before clearing dependent answers.
- `create --from-json` supports non-interactive creation from complete current-schema
  JSON.
- New `validate` and `show` commands check character files and display selected and
  derived values with actionable validation errors.
- Existing output files require confirmation unless `--force` is supplied.
- All 84 spells available during level-1 creation now carry SRD casting-time,
  range, component, duration, Concentration, Ritual, and material metadata; the
  character sheet fills every corresponding spell-table column and duration notes.
- Class, Magic Initiate, and species spellcasting now use separate derived profiles
  with ability, modifier, save DC, attack bonus, granted slots, and slotless-cast
  resources. Noncasting classes use an applicable side profile in the sheet's
  spellcasting summary; additional profiles remain visible in trait details.
- Interactive 27-point ability-score point-buy with live remaining-point feedback.
- Background ability-increase prompts omit choices that would raise a score above
  20.
- Ability-generation methods and background increases are validated at the model
  boundary before character construction.
- Optional backstory, appearance, and personality prompts with JSON persistence.
- Explicit Small/Medium selection for Human and Tiefling characters, including
  character-sheet rendering.
- Dragonborn characters choose a draconic ancestry, which determines their
  Breath Weapon and Damage Resistance damage type.
- Elf characters choose an Elven Lineage, lineage spellcasting ability, and Keen
  Senses skill proficiency.
- Gnome characters choose a Forest or Rock Gnomish Lineage and a lineage
  spellcasting ability.
- Goliath characters choose one of the six SRD Giant Ancestries and retain its
  supernatural boon details.
- Human characters choose an additional skill proficiency and one of the four SRD
  Origin feats.
- Tiefling characters choose an Abyssal, Chthonic, or Infernal legacy and its
  spellcasting ability.
- Species choices now determine speed, Darkvision, damage resistances, cantrips,
  level-gated prepared spells, displayed traits, and character-sheet values.
- Origin feats now apply their level-1 benefits and required choices: Alert adds
  proficiency to Initiative, Magic Initiate selects validated spells, Skilled
  grants three skill/tool proficiencies, and Savage Attacker is retained in the
  character's rendered feat traits.
- All level-1 class choices are now interactive and model-validated, including
  weapon masteries, tools and instruments, Divine and Primal Orders, Fighter
  Fighting Style, Rogue Expertise and language, class spells, the Wizard
  spellbook, and eligible level-1 Eldritch Invocations.
- Class selections and weapon mastery properties are included in the rendered
  class-features field.
- Class and background starting-equipment packages can be chosen independently
  from their starting-gold alternatives, including both Fighter packages and
  the Bard's package instrument choice.
- Starting inventories now model weapons, armor, shields, ammunition, gear, and
  coins as structured derived data.
- Armor Class now accounts for armor, Dexterity limits, shields, Barbarian and
  Monk Unarmored Defense, and the Defense Fighting Style; heavy armor can reduce
  Speed when its Strength requirement is unmet.
- Weapon attacks now include proficiency, attack and damage modifiers, range,
  properties, quantity, Fighting Style adjustments, and unlocked mastery notes.
- Level-1 class spellcasting ability, modifier, save DC, attack bonus, spell-slot
  count, and recovery cadence are calculated from SRD class rules.
- Rendered equipment summaries and Armor Class now reflect the selected starting
  equipment rather than static package descriptions.
- Saving-throw, skill-proficiency, and armor-training checkboxes now use the
  official template's verified `/Yes` and `/Off` appearance states.
- Corrected the existing ability, saving-throw, and skill modifier field mapping
  after correlating AcroForm widget coordinates with the visible sheet labels.
- Rogue Expertise is shown through checked skill proficiencies, doubled modifiers,
  and the visible Expertise entry in class features because the official sheet has
  no separate Expertise checkbox.
- Character-sheet attack rows now show each starting weapon's attack bonus, damage
  and type, range, properties, quantity, and applicable mastery details.
- Starting gear and all five coin denominations now populate the dedicated
  equipment and coin fields on the second page.
- Class spellcasting ability, modifier, save DC, attack bonus, level-1 slots,
  cantrips, and prepared spells now populate the second-page spellcasting fields.
- Corrected species traits, alignment, and languages to use the widgets matching
  their printed sheet labels.
- Optional appearance, backstory, and personality details now populate their
  dedicated second-page fields.
- Level-1 Rage, Bardic Inspiration, Second Wind, Lay on Hands, Favored Enemy,
  Innate Sorcery, Pact Magic, and Arcane Recovery resources now appear in the
  second Class Features column with their totals and recovery cadence.
- Text fields now use generated auto-sized appearances so long attack notes,
  equipment lists, traits, feats, narrative details, and class features remain
  inside their widgets.
- Corrected the remaining page-one header, combat summary, Hit Dice, proficiency,
  size, Passive Perception, and weapon/tool mappings through rendered-page review.
- Added PDFium-based rendered-page regression coverage for every populated region
  on representative martial and spellcasting sheets.

## [0.1.0] - 2026-07-13

### Added

- Interactive level-1 SRD 5.2.1 character creation with JSON persistence.
- Two-page AcroForm rendering using a required, separately downloaded official
  character-sheet template.
- Wheel, source distribution, and `uv tool` installation workflows.
- MIT licensing and SRD/character-sheet third-party notices.
- Linux x86-64, Windows x86-64, macOS Apple Silicon, and macOS Intel standalone
  executables with SHA-256 files.
- Cross-platform quality, native-binary, and tag-triggered GitHub Release jobs.

[Unreleased]: https://github.com/volnuttz/character-wizard/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/volnuttz/character-wizard/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/volnuttz/character-wizard/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/volnuttz/character-wizard/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/volnuttz/character-wizard/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/volnuttz/character-wizard/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/volnuttz/character-wizard/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/volnuttz/character-wizard/releases/tag/v0.1.0
