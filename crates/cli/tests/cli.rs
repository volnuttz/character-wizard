use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn executable() -> &'static str {
    env!("CARGO_BIN_EXE_character-wizard")
}

fn temporary_directory(label: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "character-wizard-cli-{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).expect("create temporary directory");
    path
}

fn assert_file(path: &Path) {
    assert!(path.is_file(), "expected file: {}", path.display());
}

#[test]
fn validates_and_shows_the_smoke_character() {
    let fixture = repository_root().join("fixtures/complete-character.json");
    for (command, expected) in [("validate", "is valid"), ("show", "HP 9 · AC 14")] {
        let output = Command::new(executable())
            .arg(command)
            .arg(&fixture)
            .output()
            .expect("run CLI");
        assert!(output.status.success(), "{command}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "{command}: {output:?}"
        );
    }
}

#[test]
fn creates_with_a_discovered_current_directory_template() {
    let root = repository_root();
    let temporary = temporary_directory("local-template");
    fs::copy(
        root.join("assets/character-sheet.pdf"),
        temporary.join("character-sheet.pdf"),
    )
    .expect("copy character-sheet fixture");
    let json = temporary.join("character.json");
    let pdf = temporary.join("character.pdf");
    let output = Command::new(executable())
        .current_dir(&temporary)
        .arg("create")
        .arg("--from-json")
        .arg(root.join("fixtures/complete-character.json"))
        .arg("--json")
        .arg(&json)
        .arg("--output")
        .arg(&pdf)
        .arg("--force")
        .output()
        .expect("run CLI");
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Using local character sheet"),
        "{output:?}"
    );
    assert_file(&json);
    assert_file(&pdf);
    fs::remove_dir_all(temporary).expect("remove temporary directory");
}
