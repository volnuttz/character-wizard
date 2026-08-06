# Releasing character-wizard

character-wizard follows Semantic Versioning. The crate version, lockfile,
Git tag, executable version, changelog heading, and release title must agree.

## Release checklist

1. Confirm the Quality, Dependency Audit, and Native binaries workflows pass on
   the exact release commit.
2. Update the package version in `Cargo.toml`, refresh `Cargo.lock` if
   needed, and update `docs/roadmap.md` plus `CHANGELOG.md`.
3. Run the complete local gate:

   ```console
   cargo +1.88.0 fmt --check
   cargo +1.88.0 clippy --all-targets -- -D warnings
   cargo +1.88.0 test --locked
   cargo +1.88.0 audit
   cargo +1.88.0 deny check
   cargo +1.88.0 build --release --locked
   target/release/character-wizard --version
   ```

4. Smoke-test `validate`, `show`, and `create --from-json` with the supported
   development template. Confirm the optimized executable contains no PDF fixture.
5. Commit and push the release changes. After all commit workflows pass, create
   and push an annotated tag:

   ```console
   git tag -a vX.Y.Z -m "character-wizard X.Y.Z"
   git push origin vX.Y.Z
   ```

The tag-triggered Native binaries workflow rebuilds and smoke-tests Linux x86-64,
Windows x86-64, macOS ARM64, and macOS x86-64 binaries, creates archives and
SHA-256 files, and publishes the existing tag through a GitHub Release.

If publication fails after a successful tag build, recover without moving the tag:

```console
gh workflow run "Native binaries" --ref main -f release_tag=vX.Y.Z
```

Audit every release for four archives and four matching checksum files, correct
version output, and absence of the SRD and character-sheet PDFs.

## Signing and notarization

Native binaries remain unsigned. Do not describe them as trusted merely because
they came from GitHub; users should verify SHA-256 first. Authenticode and Apple
Developer ID signing/notarization remain deferred until project credentials are
available.
