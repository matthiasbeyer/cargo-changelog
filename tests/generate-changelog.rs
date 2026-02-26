use std::io::Write;

mod common;

#[test]
fn generate_changelog_command_works() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(temp_dir.path(), "generate_changelog_command_works");
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_add(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='Test subject'",
            "--set",
            "type=Misc",
        ])
        .assert()
        .success();

    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["create-release", "minor"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // call `cargo-changelog create-release`
    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["generate"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let changelog_file_path = temp_dir.path().join("CHANGELOG.md");
    if !changelog_file_path.exists() {
        panic!("Changelog does not exist");
    }

    if !changelog_file_path.is_file() {
        panic!("Changelog is not a file");
    }
}

#[test]
fn generate_changelog_command_works_for_alpha_release() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(
        temp_dir.path(),
        "generate_changelog_command_works_for_alpha_release",
    );
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_add(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='Test subject'",
            "--set",
            "type=Misc",
        ])
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");

    let new_fragment_file_path = std::fs::read_dir(unreleased_dir)
        .unwrap()
        .find(|rde| match rde {
            Ok(de) => !de.path().ends_with(".gitkeep"),
            Err(_) => true,
        })
        .unwrap()
        .unwrap();

    let mut new_fragment_file = std::fs::OpenOptions::new()
        .append(true)
        .create(false)
        .open(new_fragment_file_path.path())
        .unwrap();

    writeln!(new_fragment_file).unwrap();
    writeln!(new_fragment_file, "test text").unwrap();
    new_fragment_file.sync_all().unwrap();
    drop(new_fragment_file);

    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["create-release", "custom", "0.1.0-alpha.1"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // call `cargo-changelog create-release`
    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["generate"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let changelog_file_path = temp_dir.path().join("CHANGELOG.md");
    if !changelog_file_path.exists() {
        panic!("Changelog does not exist");
    }

    if !changelog_file_path.is_file() {
        panic!("Changelog is not a file");
    }

    let changelog = std::fs::read_to_string(changelog_file_path).unwrap();
    assert!(changelog.contains("0.1.0-alpha.1"));
    assert!(changelog.contains("test text"));
}

#[test]
fn generate_changelog_command_works_with_suffix() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(
        temp_dir.path(),
        "generate_changelog_command_works_with_suffix",
    );
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_add(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='Test subject'",
            "--set",
            "type=Misc",
        ])
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");

    let new_fragment_file_path = std::fs::read_dir(unreleased_dir)
        .unwrap()
        .find(|rde| match rde {
            Ok(de) => !de.path().ends_with(".gitkeep"),
            Err(_) => true,
        })
        .unwrap()
        .unwrap();

    let mut new_fragment_file = std::fs::OpenOptions::new()
        .append(true)
        .create(false)
        .open(new_fragment_file_path.path())
        .unwrap();

    writeln!(new_fragment_file).unwrap();
    writeln!(new_fragment_file, "test text").unwrap();
    new_fragment_file.sync_all().unwrap();
    drop(new_fragment_file);

    {
        let suffix_path = temp_dir.path().join(".changelogs").join("suffix.md");

        let mut suffix_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(suffix_path)
            .unwrap();

        writeln!(suffix_file, "this is the suffix part").unwrap();
        suffix_file.sync_all().unwrap();
    }

    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["create-release", "custom", "0.1.0-alpha.1"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["generate"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let changelog_file_path = temp_dir.path().join("CHANGELOG.md");
    if !changelog_file_path.exists() {
        panic!("Changelog does not exist");
    }

    if !changelog_file_path.is_file() {
        panic!("Changelog is not a file");
    }

    let changelog = std::fs::read_to_string(changelog_file_path).unwrap();
    assert!(changelog.contains("0.1.0-alpha.1"));
    assert!(changelog.contains("test text"));
    assert!(changelog.contains("this is the suffix part"));
}

#[test]
fn generate_changelog_command_works_with_suffix_with_all_flag() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(
        temp_dir.path(),
        "generate_changelog_command_works_with_suffix",
    );
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_add(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='Test subject'",
            "--set",
            "type=Misc",
        ])
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");

    let new_fragment_file_path = std::fs::read_dir(unreleased_dir)
        .unwrap()
        .find(|rde| match rde {
            Ok(de) => !de.path().ends_with(".gitkeep"),
            Err(_) => true,
        })
        .unwrap()
        .unwrap();

    let mut new_fragment_file = std::fs::OpenOptions::new()
        .append(true)
        .create(false)
        .open(new_fragment_file_path.path())
        .unwrap();

    writeln!(new_fragment_file).unwrap();
    writeln!(new_fragment_file, "test text").unwrap();
    new_fragment_file.sync_all().unwrap();
    drop(new_fragment_file);

    {
        let suffix_path = temp_dir.path().join(".changelogs").join("suffix.md");

        let mut suffix_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(suffix_path)
            .unwrap();

        writeln!(suffix_file, "this is the suffix part").unwrap();
        suffix_file.sync_all().unwrap();
    }

    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["create-release", "custom", "0.1.0-alpha.1"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["generate", "--all"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let changelog_file_path = temp_dir.path().join("CHANGELOG.md");
    if !changelog_file_path.exists() {
        panic!("Changelog does not exist");
    }

    if !changelog_file_path.is_file() {
        panic!("Changelog is not a file");
    }

    let changelog = std::fs::read_to_string(changelog_file_path).unwrap();
    assert!(changelog.contains("0.1.0-alpha.1"));
    assert!(changelog.contains("test text"));
    assert!(changelog.contains("this is the suffix part"));
}

#[test]
fn generate_changelog_works_with_default_headers() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(temp_dir.path(), "generate_changelog_command_works");
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_add(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='Test subject'",
        ])
        .assert()
        .success();

    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["create-release", "minor"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // call `cargo-changelog create-release`
    assert_cmd::cargo::cargo_bin_cmd!("cargo-changelog")
        .args(["generate"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    let changelog_file_path = temp_dir.path().join("CHANGELOG.md");
    if !changelog_file_path.exists() {
        panic!("Changelog does not exist");
    }

    if !changelog_file_path.is_file() {
        panic!("Changelog is not a file");
    }
}
