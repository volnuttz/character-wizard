# Using Codex in character-wizard

This repository includes durable project guidance and reusable skills for Codex.
Start Codex from the repository root so it discovers both `AGENTS.md` and the
skills under `.agents/skills/`.

## What Codex loads

`AGENTS.md` contains rules that apply to all work in this repository: architecture,
source-of-truth boundaries, coding conventions, and verification commands. Codex
loads it automatically at the start of a session.

Skills contain task-specific workflows. Codex can select a skill when a request
matches its description, or you can explicitly invoke one by typing `$` or using
`/skills` in the CLI or IDE.

## Included skills

| Skill | Use it for |
| --- | --- |
| `$add-srd-content` | Adding or correcting SRD-derived rules and character options |
| `$maintain-pdf-mapping` | Inspecting or changing AcroForm field mappings and PDF output |
| `$verify-character-wizard` | Running the appropriate tests and full quality gate |
| `$release-character-wizard` | Selecting, preparing, publishing, and auditing a release |

Example prompts:

```text
Use $add-srd-content to add spell choices for level-1 Wizards from the supplied SRD.

Use $maintain-pdf-mapping to fill the saving-throw proficiency checkboxes.

Use $verify-character-wizard to validate my current branch and fix any failures.

Use $verify-character-wizard to rebuild the optimized native executable and run its smoke tests.

Use $release-character-wizard to suggest the next version and publish the release.
```

## Good task prompts

State the outcome and constraints; Codex can inspect the repository for details.
Useful examples include:

```text
Add point-buy ability generation. Keep the existing standard-array and random modes.

Diagnose why Passive Perception appears in the wrong PDF field. Do not fix it yet.

Add tests for Human size selection and implement the prompt.
```

Mention whether you want diagnosis only, implementation, or a review. Name the
relevant skill when you want Codex to follow that workflow explicitly.

## Verification and troubleshooting

Ask Codex to summarize its active instructions if repository guidance appears not
to load. Restart the Codex session after pulling newly added skills if they do not
appear in `/skills`.

The project quality gate is:

```console
cargo +1.88.0 fmt --check
cargo +1.88.0 clippy --workspace --all-targets -- -D warnings
cargo +1.88.0 test --workspace --locked
cargo +1.88.0 audit
cargo +1.88.0 deny check
```

Release tooling lives in `Cargo.toml` and `.github/workflows/`. The official
character-sheet PDF is a development fixture only: distributions and releases
must not embed it. At runtime the CLI accepts `--template`, discovers a local or
cached copy, or downloads and validates the supported official sheet. See
[`releasing.md`](releasing.md) for version tags, release recovery, checksums, and
the unsigned-binary policy.

Codex's official documentation explains the discovery and precedence rules for
[`AGENTS.md`](https://developers.openai.com/codex/guides/agents-md) and the
repository locations and invocation behavior for
[skills](https://developers.openai.com/codex/skills).
