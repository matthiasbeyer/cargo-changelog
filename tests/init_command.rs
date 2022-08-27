use assert_cmd::Command;

#[test]
fn init_command_creates_config_file() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();

    if !std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&temp_dir)
        .status()
        .unwrap()
        .success()
    {
        panic!("Failed to git-init");
    }

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

    if !std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&temp_dir)
        .status()
        .unwrap()
        .success()
    {
        panic!("Failed to git-init");
    }

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

