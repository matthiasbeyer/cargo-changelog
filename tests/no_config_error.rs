use assert_cmd::Command;

#[test]
fn no_configuration_file_errors() {
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
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn no_configuration_file_errors_with_error_message() {
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
        .current_dir(&temp_dir)
        .assert()
        .stderr(predicates::str::contains(
            "Configuration file does not exist",
        ));
}
