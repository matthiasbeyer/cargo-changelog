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
