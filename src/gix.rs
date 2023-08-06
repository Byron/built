use crate::git_shared::RepoInfo;
use crate::{fmt_option_str, write_variable};
use std::{fs, io, io::Write, path};

fn get_repo_info(manifest_location: &path::Path) -> Option<RepoInfo> {
    let repo = gix::discover(manifest_location).ok()?;

    let branch = repo.head_name().ok()?.map(|n| n.to_string());

    let repo_info = if let Ok(commit) = repo.head_commit() {
        RepoInfo {
            branch,
            tag: commit.describe().format().ok().map(|f| f.to_string()),
            dirty: is_dirty(manifest_location),
            commit_id: Some(commit.id().to_string()),
            commit_id_short: commit.id().shorten().ok().map(|p| p.to_string()),
        }
    } else {
        RepoInfo {
            branch,
            ..Default::default()
        }
    };

    Some(repo_info)
}

// TODO: replace git2 with gitoxide once this functionality becomes available in git-repository.
fn is_dirty(manifest_location: &path::Path) -> Option<bool> {
    let mut options = git2::StatusOptions::new();
    options.include_ignored(false);
    options.include_untracked(false);

    let dirty = git2::Repository::discover(manifest_location)
        .ok()?
        .statuses(Some(&mut options))
        .ok()?
        .iter()
        .any(|status| !matches!(status.status(), git2::Status::CURRENT));

    Some(dirty)
}

pub(crate) fn write_git_version(
    manifest_location: &path::Path,
    w: &mut fs::File,
) -> io::Result<()> {
    let info = get_repo_info(manifest_location).unwrap_or_default();
    crate::git_shared::write_variables(w, info)
}

// NOTE: Copy-pasted test from `git2` with adaptation to `gix`

#[cfg(test)]
mod tests {
    #[test]
    fn parse_git_repo() {
        use std::fs;
        use std::path;

        let repo_root = tempfile::tempdir().unwrap();
        assert_eq!(super::get_repo_info(repo_root.as_ref()), None);

        let repo = git2::Repository::init_opts(
            &repo_root,
            git2::RepositoryInitOptions::new()
                .external_template(false)
                .mkdir(false)
                .no_reinit(true)
                .mkpath(false),
        )
        .unwrap();

        let cruft_file = repo_root.path().join("cruftfile");
        std::fs::write(&cruft_file, "Who? Me?").unwrap();

        let project_root = repo_root.path().join("project_root");
        fs::create_dir(&project_root).unwrap();

        let sig = git2::Signature::now("foo", "bar").unwrap();
        let mut idx = repo.index().unwrap();
        idx.add_path(path::Path::new("cruftfile")).unwrap();
        idx.write().unwrap();
        let commit_oid = repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Testing testing 1 2 3",
                &repo.find_tree(idx.write_tree().unwrap()).unwrap(),
                &[],
            )
            .unwrap();

        let binding = repo
            .find_commit(commit_oid)
            .unwrap()
            .into_object()
            .short_id()
            .unwrap();

        let commit_oid_short = binding.as_str().unwrap();

        let commit_hash = format!("{}", commit_oid);
        let commit_hash_short = commit_oid_short.to_string();

        assert!(commit_hash.starts_with(&commit_hash_short));

        // The commit, the commit-id is something and the repo is not dirty
        let repo_info = super::get_repo_info(&project_root).unwrap();
        assert!(!repo_info.tag.unwrap().is_empty());
        assert_eq!(repo_info.dirty, Some(false));

        // Tag the commit, it should be retrieved
        repo.tag(
            "foobar",
            &repo
                .find_object(commit_oid, Some(git2::ObjectType::Commit))
                .unwrap(),
            &sig,
            "Tagged foobar",
            false,
        )
        .unwrap();

        let repo_info = super::get_repo_info(&project_root).unwrap();
        assert_eq!(repo_info.tag, Some(String::from("foobar")));
        assert_eq!(repo_info.dirty, Some(false));

        // Make some dirt
        std::fs::write(cruft_file, "now dirty").unwrap();
        let repo_info = super::get_repo_info(&project_root).unwrap();
        assert_eq!(repo_info.tag, Some(String::from("foobar")));
        assert_eq!(repo_info.dirty, Some(true));

        let branch_short_name = "baz";
        let branch_name = "refs/heads/baz";
        let commit = repo.find_commit(commit_oid).unwrap();
        repo.branch(branch_short_name, &commit, true).unwrap();
        repo.set_head(branch_name).unwrap();

        let repo_info = super::get_repo_info(&project_root).unwrap();
        assert_eq!(repo_info.branch, Some(branch_name.to_owned()));
        assert_eq!(repo_info.commit_id, Some(commit_hash));
        assert_eq!(repo_info.commit_id_short, Some(commit_hash_short));
    }
}
