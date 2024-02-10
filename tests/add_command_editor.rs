use std::io::Write;

use assert_cmd::Command;

mod common;

// In this test implementation we use a trick:
//
// We create a shell script that "edits" a file by creating a file next to it with the same name +
// ".edited".
//
// We set this script as EDITOR and VISUAL and then execute the "add" command. If the test sees the
// "*.edited" file, it knows that the editor was called
//

const EDITOR_COMMAND_SCRIPT: &str = r#"#!/bin/sh
touch "${1}.edited"
"#;

#[test]
fn add_command_opens_editor() {
    let temp_dir = tempfile::Builder::new()
        .prefix("cargo-changelog")
        .tempdir()
        .unwrap();
    self::common::init_git(temp_dir.path());
    self::common::init_cargo_changelog(temp_dir.path());

    let (script_temp_dir, editor_script_path) = {
        let temp = tempfile::Builder::new()
            .prefix("cargo-changelog-add-editor-script-helper")
            .tempdir()
            .unwrap();
        let script_path = temp.path().join("editor");
        let mut script = std::fs::OpenOptions::new()
            .create(true)
            .append(false)
            .write(true)
            .open(&script_path)
            .unwrap();
        write!(script, "{EDITOR_COMMAND_SCRIPT}").unwrap();
        script.sync_all().unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            let mut p = script.metadata().unwrap().permissions();
            p.set_mode(0o744);
            script.set_permissions(p).unwrap();
        }

        assert!(
            script_path.exists(),
            "Does not exist: {}",
            script_path.display()
        );
        assert!(
            script_path.is_file(),
            "Not a file: {}",
            script_path.display()
        );

        (temp, script_path)
    };

    Command::cargo_bin("cargo-changelog")
        .unwrap()
        .envs([
            ("EDITOR", editor_script_path.display().to_string()),
            ("VISUAL", editor_script_path.display().to_string()),
        ])
        .args([
            "add",
            "--interactive=false",
            "--format=toml",
            "--set",
            "issue=123",
            "--set",
            "subject='This is some text'",
            "--set",
            "type=Misc",
        ])
        .current_dir(&temp_dir)
        .assert()
        .success();

    drop(editor_script_path);
    drop(script_temp_dir);

    let unreleased_dir = temp_dir.path().join(".changelogs").join("unreleased");
    let files = std::fs::read_dir(unreleased_dir)
        .unwrap()
        .filter_map(|direntry| match direntry {
            Ok(direntry) => {
                if direntry.path().display().to_string().ends_with("edited") {
                    Some(direntry)
                } else {
                    None
                }
            }
            Err(e) => panic!("Error while iterating over directory: {e:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        files.len(),
        1,
        "Expected 1 file to be found, found: {}: {files:?}",
        files.len()
    );
}
