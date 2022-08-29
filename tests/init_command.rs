use assert_cmd::Command;

mod common;

#[test]
fn init_command_creates_config_file() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["init"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let config_file_path = temp_dir.path().join(".changelog.toml");
    if !config_file_path.exists() {
        panic!("Config file does not exist after `cargo-changelog init`");
    }
}

#[test]
fn init_command_creates_fragment_dir() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["init"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let config_file_path = temp_dir.path().join(".changelogs");
    if !config_file_path.exists() {
        panic!("Fragments directory '.changelogs' does not exist after `cargo-changelog init`");
    }
}

#[test]
fn init_command_creates_fragment_dir_unreleased() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["init"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let config_file_path = temp_dir.path().join(".changelogs").join("unreleased");
    if !config_file_path.exists() {
        panic!("Fragments directory '.changelogs/unreleased' does not exist after `cargo-changelog init`");
    }
}

#[test]
fn init_command_creates_default_template() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["init"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let template_path = temp_dir.path().join(".changelogs").join("template.md");
    if !template_path.exists() {
        panic!(
            "Template file '.changelogs/template.md' does not exist after `cargo-changelog init`"
        );
    }
}
