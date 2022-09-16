use assert_cmd::Command;

mod common;

#[test]
fn verify_metadata_command_succeeds_with_no_changelogs() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["verify-metadata"])
        .current_dir(&temp_dir)
        .assert()
        .success();
}

#[test]
fn verify_metadata_command_succeeds_with_empty_changelog() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_new(temp_dir.path())
        .args(&[
            "--format=yaml",
            "--set",
            "issue=123",
        ])
        .assert()
        .success();

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["verify-metadata"])
        .current_dir(&temp_dir)
        .assert()
        .success();
}
