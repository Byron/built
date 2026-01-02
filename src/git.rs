/// Retrieves the git-tag or hash describing the exact version and a boolean
/// that indicates if the repository currently has dirty/staged files.
///
/// If a valid git-repo can't be discovered at or above the given path,
/// or if any operation on the repository fails, `None` is returned.
#[cfg(feature = "git2")]
pub fn get_repo_description(root: &std::path::Path) -> Option<(String, bool)> {
    let repo = git2::Repository::discover(root).ok()?;
    let mut desc_opt = git2::DescribeOptions::new();
    desc_opt.describe_tags().show_commit_oid_as_fallback(true);
    let tag = repo
        .describe(&desc_opt)
        .and_then(|desc| desc.format(None))
        .ok()?;
    let mut st_opt = git2::StatusOptions::new();
    st_opt.include_ignored(false);
    st_opt.include_untracked(false);
    let dirty = repo
        .statuses(Some(&mut st_opt))
        .ok()?
        .iter()
        .any(|status| !matches!(status.status(), git2::Status::CURRENT));
    Some((tag, dirty))
}

/// Retrieves the branch name and hash of HEAD.
///
/// The returned value is a tuple of head's reference-name, long-hash and short-hash. The
/// branch name will be `None` if the head is detached, or it's not valid UTF-8.
///
/// If a valid git-repo can't be discovered at or above the given path,
/// or if any operation on the repository fails, `None` is returned.
#[cfg(feature = "git2")]
pub fn get_repo_head(root: &std::path::Path) -> Option<(Option<String>, String, String)> {
    let repo = git2::Repository::discover(root).ok()?;
    // Supposed to be the reference pointed to by HEAD, but it's HEAD
    // itself, if detached
    let head_ref = repo.head().ok()?;
    let branch = {
        // Check whether `head` is really the pointed to reference and
        // not HEAD itself.
        if repo.head_detached().ok()? {
            None
        } else {
            head_ref.name()
        }
    };
    let head = head_ref.peel_to_commit().ok()?;
    let commit_id = head.id();
    let commit_id_short = head.into_object().short_id().ok()?;
    Some((
        branch.map(ToString::to_string),
        commit_id.to_string(),
        commit_id_short.as_str().unwrap_or_default().to_string(),
    ))
}
