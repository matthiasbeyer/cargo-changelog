use std::io::Write;

mod common;

#[test]
fn new_command_creates_default_header() {
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
        writeln!(file, r#"type = "bool""#).unwrap();
        writeln!(file, "default_value = true").unwrap();
        writeln!(file, "required = true").unwrap();

        writeln!(file, "[header_fields.number]").unwrap();
        writeln!(file, r#"type = "int""#).unwrap();
        writeln!(file, "required = true").unwrap();
        file.sync_all().unwrap()
    }

    self::common::cargo_changelog_new(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "number=345",
            "--set",
            "subject='This is some text'",
            "--set",
            "type=Misc",
        ])
        .assert()
        .success();

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");
    let fragment = std::fs::read_dir(unreleased_dir)
        .unwrap()
        .into_iter()
        .find(|rde| match rde {
            Ok(de) => !de.path().ends_with(".gitkeep"),
            Err(_) => true,
        })
        .unwrap()
        .unwrap();

    let new_fragment_file_contents = std::fs::read_to_string(fragment.path()).unwrap();
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
    let toml = toml.unwrap();

    let field = toml.get("field").unwrap();
    assert!(field.is_bool());
    assert!(std::matches!(field, toml::Value::Boolean(true)));

    let number = toml.get("number").unwrap();
    assert!(number.is_integer());
    assert_eq!(number.as_integer().unwrap(), 345);

    let number = toml.get("issue").unwrap();
    assert!(number.is_integer());
    assert_eq!(number.as_integer().unwrap(), 123);
}
