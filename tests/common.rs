pub fn init_git(temp_dir: &std::path::Path) {
    if !std::process::Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir)
        .status()
        .unwrap()
        .success()
    {
        panic!("Failed to git-init");
    }
}
