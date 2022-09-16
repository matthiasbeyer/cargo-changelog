use std::path::Path;

use assert_cmd::Command;

mod common;

#[test]
fn generate_command_creates_new_directory() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(temp_dir.path(), "cargo-changelog-testpkg-generatecommand");
    self::common::init_cargo_changelog(temp_dir.path());

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");
    if !unreleased_dir.exists() {
        panic!("Unreleased directory does not exist");
    }

    // call `cargo-changelog generate`
    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["generate", "minor"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let release_dir = temp_dir.path().join(".changelogs").join("0.1.0");
    if !release_dir.exists() {
        panic!("Release dir '0.1.0' does not exist");
    }
}

#[test]
fn generate_command_moves_from_unreleased_dir() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(temp_dir.path(), "cargo-changelog-testpkg-generatecommand");
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_new(temp_dir.path())
        .args(&[
            "--format=yaml",
            "--set",
            "issue=123",
            "--set",
            "subject='This is some text'",
        ])
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");
    let files_in_dir = |path: &Path| -> Vec<_> {
        std::fs::read_dir(path)
            .expect(&format!("Should exist: {}", path.display()))
            .into_iter()
            .collect::<Vec<_>>()
    };

    {
        let files = files_in_dir(&unreleased_dir);
        assert_eq!(
            files.len(),
            2,
            "Expected 2 entries in unreleased directory, found {}: {:?}",
            files.len(),
            files
        );
    }

    let released_dir = temp_dir.path().join(".changelogs").join("0.1.0");
    if released_dir.exists() {
        panic!("Release directory should not exist yet");
    }

    // call `cargo-changelog generate`
    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["generate", "minor"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    {
        let files = files_in_dir(&unreleased_dir);
        assert_eq!(
            files.len(),
            1,
            "Expected 1 entries (gitkeep) in unreleased directory, found {}: {:?}",
            files.len(),
            files
        );
    }
    {
        let files = files_in_dir(&released_dir);
        assert_eq!(
            files.len(),
            1,
            "Expected 1 entries in released directory, found {}: {:?}",
            files.len(),
            files
        );
    }
}
