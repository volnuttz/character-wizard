# character-wizard

A native interactive command-line wizard for creating level-1 D&D characters
from SRD 5.2.1. It saves a validated canonical JSON record and fills the official
`character-sheet.pdf` AcroForm.

## Install and run

Download the archive and matching `.sha256` file for your platform from the
[latest GitHub Release](https://github.com/volnuttz/character-wizard/releases/latest):

- `character-wizard-linux-x86_64.tar.gz`
- `character-wizard-windows-x86_64.zip`
- `character-wizard-macos-arm64.tar.gz` for Apple Silicon
- `character-wizard-macos-x86_64.tar.gz` for Intel Macs

Verify the archive before extracting it:

```console
# Linux
sha256sum --check character-wizard-linux-x86_64.tar.gz.sha256

# macOS
shasum --algorithm 256 --check character-wizard-macos-arm64.tar.gz.sha256
```

On Windows PowerShell, compare `Get-FileHash` with the hash in the downloaded
`.sha256` file:

```powershell
Get-FileHash .\character-wizard-windows-x86_64.zip -Algorithm SHA256
```

Extract the archive and place `character-wizard` (or `character-wizard.exe`) on `PATH`. No
Python runtime or package manager is required. The binaries are unsigned, so
Windows SmartScreen or macOS Gatekeeper may warn on first launch.

character-wizard downloads and validates the supported official fillable character
sheet when needed. For manual or offline use, it is available from:

- [Official character-sheet downloads](https://www.dndbeyond.com/resources/1779-d-d-character-sheets)
- [Direct PDF download](https://media.dndbeyond.com/compendium-images/free-rules/ph/character-sheet.pdf)

The direct URL may change. The native renderer validates the exact supported
two-page AcroForm before prompting or writing outputs.

```console
character-wizard create
character-wizard create --template character-sheet.pdf
character-wizard validate character.json
character-wizard show character.json
character-wizard create --template character-sheet.pdf --from-json character.json --force
```

Creation writes `character.json` and `character-sheet-filled.pdf` by default.
Use `--json`, `--output`, and `--draft` to choose other paths. Existing outputs
require confirmation unless `--force` is supplied. Interactive creation saves a
checkpoint after every completed stage, supports final review and editing, and
resumes from the same draft path.

The template is resolved in this order: an explicit `--template`, a
`character-sheet.pdf` in the current directory, or the application cache. When
none is available, character-wizard visibly downloads the supported official sheet,
validates it, and saves it to the user cache. Set `CHARACTER_WIZARD_CACHE_DIR` to choose
a different cache location. Use `--template` for offline or reproducible runs.

Character JSON is the canonical current-schema record. The PDF is a rendered
view; older or unknown JSON shapes are rejected rather than silently migrated.

## Build from source

The workspace pins Rust 1.88.0:

```console
rustup toolchain install 1.88.0 --profile minimal --component rustfmt --component clippy
cargo +1.88.0 build --release --locked -p character-wizard-cli
target/release/character-wizard --version
```

The complete development gate is:

```console
cargo +1.88.0 fmt --check
cargo +1.88.0 clippy --workspace --all-targets -- -D warnings
cargo +1.88.0 test --workspace --locked
cargo +1.88.0 audit
cargo +1.88.0 deny check
```

The current scope covers all 12 SRD classes, 4 backgrounds, and 9 species at
level 1, including suggested/standard arrays, rolled scores, 27-point point buy,
background increases, class and origin choices, equipment, combat values,
spellcasting, checkpoint/resume, and the full supported character-sheet mapping.

The completed Python-to-Rust parity and performance results are summarized in
[`docs/rust-migration.md`](docs/rust-migration.md). The legacy Python
implementation and migration-only generated artifacts were retired after the
verified `v0.3.0` native release.

See [CONTRIBUTING.md](CONTRIBUTING.md), [CHANGELOG.md](CHANGELOG.md), and the
[roadmap](docs/roadmap.md). SRD attribution and template terms are in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## License

character-wizard's original source code is available under the [MIT License](LICENSE).
