# character-wizard contributor guidance

## Project scope

- Build a Rust 1.88 CLI that creates D&D characters from
  `assets/SRD_CC_v5.2.1.pdf` and renders them with the separately downloaded
  official `assets/character-sheet.pdf` development fixture.
- Treat the supplied SRD PDF as the authority for game rules. Do not silently mix
  in rules from the Player's Handbook, older SRDs, third-party sources, or memory.
- Keep JSON as the canonical character record. The filled PDF is a rendered view.
- The current product scope is level-1 character creation unless a task explicitly
  expands it.

## Repository map

- `crates/srd-data/`: SRD-derived data and rule metadata.
- `crates/domain/`: Serde models, validation, and derived character values.
- `crates/creation/`: interactive workflow, drafts, resume, and review.
- `crates/pdf-renderer/`: character-sheet AcroForm mapping and rendering.
- `crates/cli/`: native command-line entry point and presentation.
- `crates/integration-tests/`: native PDF integration tests.
- `fixtures/`: Rust-native CLI and rendering smoke inputs.
- `CHANGELOG.md`: versioned user-visible release history.
- `docs/codex.md`: Codex usage and repository skills.
- `docs/releasing.md`: versioning, release recovery, and signing policy.
- `docs/roadmap.md`: current baseline, known gaps, phases, and pending tasks.
- `.github/workflows/`: cross-platform quality, binary, checksum, and release jobs.
- `.agents/skills/`: reusable Codex workflows for this repository.

## Working rules

- Use Cargo and the pinned Rust 1.88 toolchain. Keep formatting and Clippy's
  workspace lint policy clean.
- Put validation and derived game logic in models or rule helpers, not in CLI
  presentation code.
- Keep prompts thin: gather choices in `creation`, validate in domain models,
  and render from a completed `Character`.
- Preserve user changes and avoid modifying the source PDFs.
- Keep the official character-sheet template out of crates, executables, archives,
  and release assets. Resolve it from `--template`, the current directory, or the
  validated user cache; download and cache it only when none of those are usable.
- Keep `docs/roadmap.md` current when completing, adding, blocking, or
  reprioritizing roadmap work. Mark tasks complete only after verification.
- Add or update tests for every behavior change. PDF mapping changes require a
  write/read-back test against the supported official template used as the local
  development fixture.
- When adding dependencies, explain why the standard library and current
  dependencies are insufficient.

## Required verification

Run the smallest relevant tests while iterating. Before handing off code changes,
run the full gate:

```console
cargo +1.88.0 fmt --check
cargo +1.88.0 clippy --workspace --all-targets -- -D warnings
cargo +1.88.0 test --workspace --locked
cargo +1.88.0 audit
cargo +1.88.0 deny check
```

Also build the release binary and run `target/release/character-wizard --help` after
changing commands or options. For an
interactive-flow change, exercise the affected path or test the prompt adapters.
For packaging or release changes, run the native create smoke scenario; validate
workflow YAML when Actions files change.

## Repository skills

- Use `$add-srd-content` for SRD-derived classes, backgrounds, species, spells,
  equipment, feats, or character-creation rules.
- Use `$maintain-pdf-mapping` for `character-sheet.pdf`, AcroForm fields, or
  `crates/pdf-renderer` changes.
- Use `$verify-pc-wizard` for pre-handoff validation or diagnosing quality-gate
  failures.
- Use `$release-pc-wizard` to select, prepare, verify, publish, or recover a
  versioned release.
