use assert_cmd::Command;

mod common;

#[test]
fn no_configuration_file_errors() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(["release"]) // we need some subcommand, otherwise nothing happens
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn no_configuration_file_errors_with_error_message() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(["release"]) // we need some subcommand, otherwise nothing happens
        .current_dir(&temp_dir)
        .assert()
        .stderr(predicates::str::contains(
            "Configuration file does not exist",
        ));
}
