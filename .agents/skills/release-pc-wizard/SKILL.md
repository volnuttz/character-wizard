---
name: release-pc-wizard
description: Prepare and publish a pc-wizard Semantic Versioning release through version selection, release metadata updates, local quality and distribution checks, staged GitHub Actions verification, annotated tagging, GitHub Release publication, and final asset auditing. Use when asked to suggest a release version, cut a release, publish a version tag, or diagnose/recover this repository's release process.
---

# Release pc-wizard

Follow `docs/releasing.md` as the repository policy and use `$verify-pc-wizard`
for its quality gate. Preserve unrelated work and never move an existing release
tag.

## Assess the release

1. Inspect the worktree, existing tags, commits since the latest tag,
   `CHANGELOG.md`, `docs/roadmap.md`, version declarations, and release workflows.
2. Require a clean or fully understood worktree. Do not include unrelated user
   changes in release metadata or commits.
3. Suggest a Semantic Versioning increment from user-visible impact:
   - patch for compatible fixes only;
   - minor for backward-compatible features or substantial pre-1.0 expansion;
   - major for stable-API breaking changes.
4. Explain the recommendation briefly and proceed when the request authorizes a
   release. Ask only when competing versions would materially change intent.

## Prepare the release commit

1. Update every version surface to the same `X.Y.Z`:
   - package version in `Cargo.toml`;
   - `Cargo.lock` via `cargo +1.88.0 update` when required;
   - `docs/roadmap.md` current baseline;
   - `CHANGELOG.md` dated release heading and comparison links.
2. Keep an empty `Unreleased` heading above the new changelog release.
3. Update other user-facing release documentation when its claims would become
   stale, especially signing and packaging statements.
4. Review the complete diff and run `git diff --check`.

## Verify locally

Run the complete release gate from the repository root:

```console
cargo +1.88.0 fmt --check
cargo +1.88.0 clippy --all-targets -- -D warnings
cargo +1.88.0 test --locked
cargo +1.88.0 audit
cargo +1.88.0 deny check
cargo +1.88.0 build --release --locked
target/release/character-wizard --version
```

Inspect every native archive. Confirm that the SRD PDF and official
character-sheet template are absent. If release, packaging, CLI, PDF, or
workflow code changed, also run the additional checks required by `AGENTS.md`,
`docs/releasing.md`, and `$verify-pc-wizard`.

Do not commit or publish while a relevant check is failing. Fix only failures in
scope, then rerun the failed check and the full gate.

## Stage publication safely

1. Commit only the audited release files with a message such as
   `release: pc-wizard X.Y.Z` and push the release commit to the default branch.
2. Monitor both `Quality` and `Native binaries` workflows for that exact commit.
   Verify the commit SHA; do not rely only on workflow names or the latest run.
3. Create and push the annotated `vX.Y.Z` tag only after both commit workflows
   succeed:

   ```console
   git tag -a vX.Y.Z -m "pc-wizard X.Y.Z"
   git push origin vX.Y.Z
   ```

4. Monitor both tag-triggered workflows through completion. The tag's Native
   binaries run must include the successful GitHub Release publication job.
5. Prefer authenticated `gh` inspection. If `gh` authentication is unavailable
   and the repository is public, use the GitHub REST API read-only endpoints for
   workflow and release status. Never expose credentials in output.

## Audit and recover

Confirm that the published release:

- is neither a draft nor a prerelease unless explicitly requested;
- has tag `vX.Y.Z` and title `pc-wizard vX.Y.Z`;
- contains Linux x86-64, Windows x86-64, macOS ARM64, and macOS x86-64 archives;
- contains one matching `.sha256` file for every archive;
- contains no source PDF fixture;
- leaves the local branch clean and synchronized with its upstream.

If the tag build succeeds but publication fails, follow the manual recovery flow
in `docs/releasing.md` without deleting, recreating, or moving the tag. Report any
authentication or permissions blocker precisely and leave the repository in a
recoverable state.

## Handoff

Lead with whether the release was published. Include the chosen version and why,
release URL, release commit, local gate results, GitHub workflow results, asset
audit, and any remaining blocker. Do not claim publication until the release and
assets have been inspected.
