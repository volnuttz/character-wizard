# Contributing to character-wizard

Read [AGENTS.md](AGENTS.md) before making changes. Use the pinned Rust 1.88.0
toolchain from the repository root; the official character-sheet PDF is a
development fixture only and must never be packaged or modified.

Keep SRD-derived data in `src/srd_data`, validation and calculations in
`src/domain`, prompts in `src/creation`, PDF projection in
`src/pdf_renderer`, and command coordination in `src/main.rs`. JSON remains the
canonical record and the supplied SRD remains the rule authority.

Add tests for every behavior change. PDF changes require render/read-back coverage
against the supported development fixture. Explain new dependencies when the
standard library and current dependencies are insufficient, and update the
roadmap and changelog for visible or milestone changes.

Run before opening a pull request:

```console
cargo +1.88.0 fmt --check
cargo +1.88.0 clippy --all-targets -- -D warnings
cargo +1.88.0 test --locked
cargo +1.88.0 audit
cargo +1.88.0 deny check
```

For CLI or packaging changes, also build and exercise the release binary. Parse
edited GitHub Actions workflows as YAML. Do not include private character data,
downloaded templates, build artifacts, or generated coverage files.

By submitting a contribution, you agree that it may be distributed under the
project's [MIT License](LICENSE).
