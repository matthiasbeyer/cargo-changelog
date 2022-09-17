use itertools::Itertools;

pub fn repo_is_dirty(repo: &git2::Repository) -> bool {
    if repo.state() != git2::RepositoryState::Clean {
        log::trace!("Repository status is unclean: {:?}", repo.state());
        return true;
    }

    let status = repo
        .statuses(Some(git2::StatusOptions::new().include_ignored(false)))
        .unwrap();
    if status.is_empty() {
        false
    } else {
        log::trace!(
            "Repository is dirty: {}",
            status
                .iter()
                .flat_map(|s| s.path().map(|s| s.to_owned()))
                .join(", ")
        );
        true
    }
}
