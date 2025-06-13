use assert_cmd::Command;

mod common;

const TEMPLATE: &str = r#"
{{#each (reverse (sort_versions this.versions))}}
{{#each (group_by_header this.entries "type" default="misc")}}
{{#each this ~}}
{{indent this.text " "}}
{{indent this.text spaces=5}}
{{/each ~}}
{{~ /each ~}}
{{~ /each ~}}
"#;

#[test]
fn generate_changelog_with_body_indented() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo(temp_dir.path(), "generate_changelog_with_body_indented");
    self::common::init_cargo_changelog(temp_dir.path());

    std::fs::write(temp_dir.path().join(".changelogs/template.md"), TEMPLATE).unwrap();

    self::common::cargo_changelog_add(temp_dir.path())
        .args([
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='Test subject'",
            "--read",
            "-",
        ])
        .write_stdin("test123\ntest456")
        .assert()
        .success();

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(["create-release", "minor"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    // call `cargo-changelog create-release`
    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(["generate-changelog"])
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

    let file_content = std::fs::read_to_string(changelog_file_path).unwrap();
    let expected_content = "\n test123\n test456\n     test123\n     test456\n\n";
    assert_eq!(file_content, expected_content);
}
