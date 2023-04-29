#![allow(dead_code)]

pub fn init_git(temp_dir: &std::path::Path) {
    if !std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir)
        .status()
        .unwrap()
        .success()
    {
        panic!("Failed to git-init");
    }
}

pub fn init_cargo(temp_dir: &std::path::Path, name: &str) {
    if !std::process::Command::new("cargo")
        .args(["init", "--bin", "--name", name])
        .current_dir(temp_dir)
        .status()
        .unwrap()
        .success()
    {
        panic!("Failed to cargo-init");
    }
}

pub fn init_cargo_changelog(temp_dir: &std::path::Path) {
    assert_cmd::Command::cargo_bin("cargo-changelog")
        .unwrap()
        .args(["init"])
        .current_dir(temp_dir)
        .assert()
        .success();
}

pub fn cargo_changelog_cmd(dir: &std::path::Path) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("cargo-changelog").unwrap();
    cmd.current_dir(dir);
    cmd
}

pub fn cargo_changelog_add(dir: &std::path::Path) -> assert_cmd::Command {
    let mut cmd = cargo_changelog_cmd(dir);
    cmd.arg("add");
    cmd.arg("--interactive");
    cmd.arg("false");
    cmd.arg("--edit");
    cmd.arg("false");
    cmd
}
