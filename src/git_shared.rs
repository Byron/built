use crate::{fmt_option_str, write_variable};
use std::{fs, io};

#[derive(Debug, Default, PartialEq)]
pub(crate) struct RepoInfo {
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub dirty: Option<bool>,
    pub commit_id: Option<String>,
    pub commit_id_short: Option<String>,
}

pub(crate) fn write_variables(mut w: &fs::File, info: RepoInfo) -> io::Result<()> {
    use io::Write;

    write_variable!(
        w,
        "GIT_VERSION",
        "Option<&str>",
        fmt_option_str(info.tag),
        "If the crate was compiled from within a git-repository, \
        `GIT_VERSION` contains HEAD's tag. The short commit id is used if HEAD is not tagged."
    );
    write_variable!(
        w,
        "GIT_DIRTY",
        "Option<bool>",
        match info.dirty {
            Some(true) => "Some(true)",
            Some(false) => "Some(false)",
            None => "None",
        },
        "If the repository had dirty/staged files."
    );

    let doc = "If the crate was compiled from within a git-repository, `GIT_HEAD_REF` \
        contains full name to the reference pointed to by HEAD \
        (e.g.: `refs/heads/master`). If HEAD is detached or the branch name is not \
        valid UTF-8 `None` will be stored.\n";
    write_variable!(
        w,
        "GIT_HEAD_REF",
        "Option<&str>",
        fmt_option_str(info.branch),
        doc
    );

    write_variable!(
        w,
        "GIT_COMMIT_HASH",
        "Option<&str>",
        fmt_option_str(info.commit_id),
        "If the crate was compiled from within a git-repository, `GIT_COMMIT_HASH` \
    contains HEAD's full commit SHA-1 hash."
    );

    write_variable!(
        w,
        "GIT_COMMIT_HASH_SHORT",
        "Option<&str>",
        fmt_option_str(info.commit_id_short),
        "If the crate was compiled from within a git-repository, `GIT_COMMIT_HASH_SHORT` \
    contains HEAD's short commit SHA-1 hash."
    );

    Ok(())
}
