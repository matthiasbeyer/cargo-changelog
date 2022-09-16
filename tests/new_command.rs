use std::io::Write;

use assert_cmd::Command;

mod common;

#[test]
fn new_command_creates_yaml_file() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_new(temp_dir.path())
        .args(&[
            "--format=yaml",
            "--set",
            "issue=123",
            "--set",
            "subject=This is some text",
        ])
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");
    if !unreleased_dir.exists() {
        panic!("Unreleased directory does not exist");
    }

    let files = std::fs::read_dir(&unreleased_dir)
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    assert_eq!(
        files.len(),
        2,
        "Expected 2 entries in unreleased directory, found {}: {:?}",
        files.len(),
        files
    );

    let new_fragment_file = files
        .into_iter()
        .find(|rde| match rde {
            Ok(de) => !de.path().ends_with(".gitkeep"),
            Err(_) => true,
        })
        .unwrap()
        .unwrap();
    {
        let ft = new_fragment_file.file_type().unwrap();
        assert!(
            ft.is_file(),
            "Expected {} to be a file, is {:?}",
            new_fragment_file.path().display(),
            ft
        );
    }

    let new_fragment_file_contents = std::fs::read_to_string(new_fragment_file.path()).unwrap();
    let yaml_header = new_fragment_file_contents
        .lines()
        .skip(1)
        .take_while(|line| *line != "---")
        .collect::<Vec<_>>()
        .join("\n");

    let yaml = serde_yaml::from_str::<serde_yaml::Value>(&yaml_header);
    assert!(
        yaml.is_ok(),
        "Failed to parse fragment file: {:?}",
        yaml.unwrap_err()
    );
}

#[test]
fn new_command_creates_unreleased_gitkeep() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let unreleased_gitkeep_path = temp_dir
        .path()
        .join(".changelogs")
        .join("unreleased")
        .join(".gitkeep");
    if !unreleased_gitkeep_path.exists() {
        panic!("unreleased gitkeep file does not exist");
    }
    if !unreleased_gitkeep_path.is_file() {
        panic!("unreleased gitkeep file is not a file");
    }
}

#[test]
fn new_command_with_text_creates_yaml_with_text_from_stdin() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let test_text = "This is a test text";
    {
        let text_temp_dir = tempdir::TempDir::new("cargo-changelog-new-test-text").unwrap();
        let path = text_temp_dir.path().join("text_file.txt");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&path)
            .unwrap();

        write!(file, "{}", test_text).unwrap();
        file.sync_all().unwrap();
        drop(file); // make sure we close the handle

        self::common::cargo_changelog_new(temp_dir.path())
            .args(&[
                "--format=yaml",
                "--set",
                "issue=123",
                "--set",
                "subject='This is some text'",
                "--read=-", // read text from STDIN
            ])
            .pipe_stdin(path)
            .unwrap()
            .assert()
            .success();
    }

    let fragment_file = std::fs::read_dir(&temp_dir.path().join(".changelogs").join("unreleased"))
        .unwrap()
        .into_iter()
        .find(|rde| match rde {
            Ok(de) => !de.path().ends_with(".gitkeep"),
            Err(_) => true,
        })
        .unwrap()
        .unwrap();

    let new_fragment_file_contents = std::fs::read_to_string(fragment_file.path()).unwrap();
    let contents = new_fragment_file_contents
        .lines()
        .skip(1)
        .skip_while(|line| *line != "---")
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(contents, test_text);
}

#[test]
fn new_command_with_text_creates_yaml_with_text_from_file() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let test_text = "This is a test text";
    {
        let text_temp_dir = tempdir::TempDir::new("cargo-changelog-new-test-text").unwrap();
        let path = text_temp_dir.path().join("text_file.txt");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&path)
            .unwrap();

        write!(file, "{}", test_text).unwrap();
        file.sync_all().unwrap();
        drop(file); // make sure we close the handle

        self::common::cargo_changelog_new(temp_dir.path())
            .args(&[
                "--format=yaml",
                "--set",
                "issue=123",
                "--set",
                "subject='This is some text'",
                // read text from PATH
                "--read",
                &path.display().to_string(),
            ])
            .pipe_stdin(path)
            .unwrap()
            .assert()
            .success();
    }

    let fragment_file = std::fs::read_dir(&temp_dir.path().join(".changelogs").join("unreleased"))
        .unwrap()
        .into_iter()
        .find(|rde| match rde {
            Ok(de) => !de.path().ends_with(".gitkeep"),
            Err(_) => true,
        })
        .unwrap()
        .unwrap();

    let new_fragment_file_contents = std::fs::read_to_string(fragment_file.path()).unwrap();
    let contents = new_fragment_file_contents
        .lines()
        .skip(1)
        .skip_while(|line| *line != "---")
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(contents, test_text);
}

#[test]
fn new_command_creates_toml_header() {
    let temp_dir = tempdir::TempDir::new("cargo-changelog").unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_new(temp_dir.path())
        .args(&[
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='This is some text'",
        ])
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");

    let new_fragment_file = std::fs::read_dir(&unreleased_dir)
        .unwrap()
        .into_iter()
        .find(|rde| match rde {
            Ok(de) => !de.path().ends_with(".gitkeep"),
            Err(_) => true,
        })
        .unwrap()
        .unwrap();

    let new_fragment_file_contents = std::fs::read_to_string(new_fragment_file.path()).unwrap();
    let toml_header = new_fragment_file_contents
        .lines()
        .skip(1)
        .take_while(|line| *line != "+++")
        .collect::<Vec<_>>()
        .join("\n");

    let toml = toml::from_str::<serde_yaml::Value>(&toml_header);
    assert!(
        toml.is_ok(),
        "Failed to parse fragment file: {:?}",
        toml.unwrap_err()
    );
}
