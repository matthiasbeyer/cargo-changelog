use std::io::Write;

mod common;

#[test]
fn add_command_creates_toml_file() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_add(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject=This is some text",
            "--set",
            "type=Bugfix",
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
    let toml_header = new_fragment_file_contents
        .lines()
        .skip(1)
        .take_while(|line| *line != "+++")
        .collect::<Vec<_>>()
        .join("\n");

    let toml = toml::from_str::<toml::Value>(&toml_header);
    assert!(
        toml.is_ok(),
        "Failed to parse fragment file: {:?}",
        toml.unwrap_err()
    );
}

#[test]
fn add_command_creates_unreleased_gitkeep() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
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
fn add_command_with_text_creates_toml_with_text_from_stdin() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let test_text = "This is a test text";
    {
        let text_temp_dir = tempfile::Builder::new()
            .prefix("cargo-changelog-new-test-text")
            .tempdir()
            .unwrap();
        let path = text_temp_dir.path().join("text_file.txt");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&path)
            .unwrap();

        write!(file, "{test_text}").unwrap();
        file.sync_all().unwrap();
        drop(file); // make sure we close the handle

        self::common::cargo_changelog_add(temp_dir.path())
            .args([
                "--format=toml",
                "--set",
                "issue=123",
                "--set",
                "subject='This is some text'",
                "--set",
                "type=Bugfix",
                "--read=-", // read text from STDIN
            ])
            .pipe_stdin(path)
            .unwrap()
            .assert()
            .success();
    }

    let fragment_file = std::fs::read_dir(temp_dir.path().join(".changelogs").join("unreleased"))
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
        .skip_while(|line| *line != "+++")
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(contents, test_text);
}

#[test]
fn add_command_with_text_creates_toml_with_text_from_file() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let test_text = "This is a test text";
    {
        let text_temp_dir = tempfile::Builder::new()
            .prefix("cargo-changelog-new-test-text")
            .tempdir()
            .unwrap();
        let path = text_temp_dir.path().join("text_file.txt");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&path)
            .unwrap();

        write!(file, "{test_text}").unwrap();
        file.sync_all().unwrap();
        drop(file); // make sure we close the handle

        self::common::cargo_changelog_add(temp_dir.path())
            .args([
                "--format=toml",
                "--set",
                "issue=123",
                "--set",
                "subject='This is some text'",
                "--set",
                "type=Bugfix",
                // read text from PATH
                "--read",
                &path.display().to_string(),
            ])
            .pipe_stdin(path)
            .unwrap()
            .assert()
            .success();
    }

    let fragment_file = std::fs::read_dir(temp_dir.path().join(".changelogs").join("unreleased"))
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
        .skip_while(|line| *line != "+++")
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(contents, test_text);
}

#[test]
fn add_command_creates_toml_header() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    self::common::cargo_changelog_add(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='This is some text'",
            "--set",
            "type=Bugfix",
        ])
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");

    let new_fragment_file = std::fs::read_dir(unreleased_dir)
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

    let toml = toml::from_str::<toml::Value>(&toml_header);
    assert!(
        toml.is_ok(),
        "Failed to parse fragment file: {:?}",
        toml.unwrap_err()
    );
}

#[test]
fn add_command_cannot_create_nonexistent_oneof() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    {
        // Write some header field to the config file
        let config_file_path = temp_dir.path().join("changelog.toml");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .write(true)
            .open(config_file_path)
            .unwrap();

        writeln!(file, "[header_fields.field]").unwrap();
        writeln!(file, r#"type = ["foo", "bar"]"#).unwrap();
        writeln!(file, "default_value = true").unwrap();
        writeln!(file, "required = true").unwrap();
        file.sync_all().unwrap()
    }

    self::common::cargo_changelog_add(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='This is some text'",
            "--set",
            "field=baz",
            "--set",
            "type=Bugfix",
        ])
        .assert()
        .failure();
}
