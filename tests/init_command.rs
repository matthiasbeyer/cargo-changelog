mod common;

#[test]
fn init_command_creates_config_file() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let config_file_path = temp_dir.path().join("changelog.toml");
    if !config_file_path.exists() {
        panic!("Config file does not exist after `cargo-changelog init`");
    }
}

#[test]
fn init_command_creates_fragment_dir() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let config_file_path = temp_dir.path().join(".changelogs");
    if !config_file_path.exists() {
        panic!("Fragments directory '.changelogs' does not exist after `cargo-changelog init`");
    }
}

#[test]
fn init_command_creates_fragment_dir_unreleased() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let config_file_path = temp_dir.path().join(".changelogs").join("unreleased");
    if !config_file_path.exists() {
        panic!("Fragments directory '.changelogs/unreleased' does not exist after `cargo-changelog init`");
    }
}

#[test]
fn init_command_creates_default_template() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let template_path = temp_dir.path().join(".changelogs").join("template.md");
    if !template_path.exists() {
        panic!(
            "Template file '.changelogs/template.md' does not exist after `cargo-changelog init`"
        );
    }
}
