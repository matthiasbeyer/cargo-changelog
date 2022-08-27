use assert_cmd::Command;

#[test]
fn no_repo_errors() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .current_dir(&temp_dir)
        .assert()
        .failure();
}
