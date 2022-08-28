use std::io::Write;

use assert_cmd::Command;

mod common;

#[test]
fn new_command_creates_default_header() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["init"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    {
        // Write some header field to the config file
        let config_file_path = temp_dir.path().join(".changelog.toml");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .write(true)
            .open(config_file_path)
            .unwrap();

        write!(
            file,
            r#"field = {{ type = "bool", default_value = true, required = true }}"#
        )
        .unwrap();
        file.sync_all().unwrap()
    }

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
    let fragment = std::fs::read_dir(&unreleased_dir)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .unwrap();

    let new_fragment_file_contents = std::fs::read_to_string(fragment.path()).unwrap();
    let yaml_header = new_fragment_file_contents
        .lines()
        .skip(1)
        .take_while(|line| *line != "---")
        .collect::<String>();

    let yaml = serde_yaml::from_str::<serde_yaml::Value>(&yaml_header);
    assert!(
        yaml.is_ok(),
        "Failed to parse fragment file: {:?}",
        yaml.unwrap_err()
    );
    let yaml = yaml.unwrap();

    let field = yaml.get("field").unwrap();
    assert!(field.is_bool());
    assert!(std::matches!(field, serde_yaml::Value::Bool(true)));
}