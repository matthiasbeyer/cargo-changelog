use std::io::Write;

use assert_cmd::Command;

mod common;

#[test]
fn release_command_works() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["init"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&[
            "new",
            "--interactive=false",
            "--edit=false",
            "--format=yaml",
        ])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["generate", "minor"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // call `cargo-changelog generate`
    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["release"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let changelog_file_path = temp_dir.path().join("CHANGELOG.md");
    if !changelog_file_path.exists() {
        panic!("Changelog does not exist");
    }

    if !changelog_file_path.is_file() {
        panic!("Changelog is not a file");
    }
}

#[test]
fn release_command_works_for_alpha_release() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["init"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&[
            "new",
            "--interactive=false",
            "--edit=false",
            "--format=yaml",
        ])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");

    let new_fragment_file_path = std::fs::read_dir(&unreleased_dir)
        .unwrap()
        .into_iter()
        .find(|rde| match rde {
            Ok(de) => !de.path().ends_with(".gitkeep"),
            Err(_) => true,
        })
        .unwrap()
        .unwrap();

    let mut new_fragment_file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(false)
        .open(new_fragment_file_path.path())
        .unwrap();

    writeln!(new_fragment_file, "").unwrap();
    writeln!(new_fragment_file, "test text").unwrap();
    new_fragment_file.sync_all().unwrap();
    drop(new_fragment_file);

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["generate", "custom", "0.1.0-alpha.1"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // call `cargo-changelog generate`
    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["release"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let changelog_file_path = temp_dir.path().join("CHANGELOG.md");
    if !changelog_file_path.exists() {
        panic!("Changelog does not exist");
    }

    if !changelog_file_path.is_file() {
        panic!("Changelog is not a file");
    }

    let changelog = std::fs::read_to_string(&changelog_file_path).unwrap();
    assert!(changelog.contains("0.1.0-alpha.1"));
    assert!(changelog.contains("test text"));
}
