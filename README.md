# character-wizard

Create a complete level-1 D&D character in your terminal. `character-wizard`
walks you through the choices in the SRD 5.2.1, saves the character as JSON, and
fills the official two-page character sheet for you.

```console
character-wizard create
```

At the end, a character named Legolas produces:

```text
legolas.json
legolas.pdf
```

The JSON file is the source of truth; the PDF is a convenient, filled-in view.

## Get it

Download the archive for your computer and its matching `.sha256` file from the
[latest release](https://github.com/volnuttz/character-wizard/releases/latest).

| Platform | Archive |
| --- | --- |
| Linux (x86_64) | `character-wizard-linux-x86_64.tar.gz` |
| macOS (Apple Silicon) | `character-wizard-macos-arm64.tar.gz` |
| macOS (Intel) | `character-wizard-macos-x86_64.tar.gz` |
| Windows (x86_64) | `character-wizard-windows-x86_64.zip` |

Verify the archive, extract it, then run `character-wizard` (or
`character-wizard.exe` on Windows). You can place the executable on your `PATH`
to run it from any folder. No Python or package manager is required.

```console
# Linux
sha256sum --check character-wizard-linux-x86_64.tar.gz.sha256

# macOS
shasum --algorithm 256 --check character-wizard-macos-arm64.tar.gz.sha256
```

In Windows PowerShell, run `Get-FileHash .\character-wizard-windows-x86_64.zip
-Algorithm SHA256` and compare it with the downloaded `.sha256` file. The
binaries are unsigned, so Windows SmartScreen or macOS Gatekeeper may ask for
confirmation the first time you open one.

## Create a character

Run `character-wizard create` and answer the prompts. Your progress is saved
after each completed stage, so closing the terminal does not lose your work;
rerun the command to continue from the checkpoint. The final review lets you go
back and change a section before writing the files.

The official fillable character-sheet PDF is found in this order:

1. The file passed with `--template`.
2. `character-sheet.pdf` in the folder where you run the command.
3. A validated cached copy.

If none is available, the program downloads and validates the supported official
sheet, then caches it. For offline use, download the sheet yourself from the
[official character-sheet downloads](https://www.dndbeyond.com/resources/1779-d-d-character-sheets)
and pass its path explicitly:

```console
character-wizard create --template /path/to/character-sheet.pdf
```

Set `CHARACTER_WIZARD_CACHE_DIR` to use a different cache location.

## Choose file names and paths

By default, output names come from the character name, lowercased and made safe
for a filename: `Legolas` becomes `legolas.json` and `legolas.pdf`. Use `--json`
and `--output` whenever you want different names or folders:

```console
character-wizard create --json saves/legolas.json --output sheets/legolas.pdf
```

Existing outputs require confirmation. Add `--force` for scripts and other
non-interactive use.

## Work with saved characters

Validate a JSON record or print a quick character summary:

```console
character-wizard validate legolas.json
character-wizard show legolas.json
```

You can render a known JSON record without completing the wizard again:

```console
character-wizard create --from-json legolas.json --force
```

That command still uses the name stored in the JSON for the default output
names. Pass `--json` or `--output` to override either one.

The optional `./characters` collection supports bare character names:

```console
character-wizard list
character-wizard show legolas
character-wizard edit legolas
character-wizard render legolas
```

Export a collection character as a compact copy-and-paste code and import it
back into canonical JSON:

```console
character-wizard export legolas
character-wizard import <code>
```

Explicit JSON paths remain supported. See [portable character
sharing](docs/sharing.md) for destinations, data-pack handling, format limits,
and security guidance.

## Build from source

This project uses Rust 1.88.0.

```console
rustup toolchain install 1.88.0 --profile minimal --component rustfmt --component clippy
cargo +1.88.0 build --release --locked
target/release/character-wizard --help
```

For contributors, the complete check is:

```console
cargo +1.88.0 fmt --check
cargo +1.88.0 clippy --all-targets -- -D warnings
cargo +1.88.0 test --locked
cargo +1.88.0 audit
cargo +1.88.0 deny check
```

The current scope includes all 12 SRD classes, four backgrounds, and nine
species at level 1. See [CONTRIBUTING.md](CONTRIBUTING.md), the
[changelog](CHANGELOG.md), and the [roadmap](docs/roadmap.md) for project
details. SRD attribution and character-sheet template terms are in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## License

The original character-wizard source code is available under the [MIT
License](LICENSE).
