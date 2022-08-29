use assert_cmd::Command;

mod common;

#[test]
fn generate_command_creates_new_directory() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());

    // create a new cargo project in the temp dir
    Command::new("cargo")
        .args(&[
            "init",
            "--bin",
            "--name",
            "cargo-changelog-testpkg-generatecommand",
        ])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // initialize changelog
    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["init"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");
    if !unreleased_dir.exists() {
        panic!("Unreleased directory does not exist");
    }

    // call `cargo-changelog generate`
    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(&["generate", "minor"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let release_dir = temp_dir.path().join(".changelogs").join("0.1.0");
    if !release_dir.exists() {
        panic!("Release dir '0.1.0' does not exist");
    }
}
