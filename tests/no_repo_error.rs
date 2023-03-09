use assert_cmd::Command;

#[test]
fn no_repo_errors() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(["release"]) // we need some subcommand, otherwise nothing happens
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn no_repo_errors_with_no_repo_error_message() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(["release"]) // we need some subcommand, otherwise nothing happens
        .current_dir(&temp_dir)
        .assert()
        .stderr(predicates::str::contains("could not find repository"));
}
