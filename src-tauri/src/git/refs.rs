// Lists the refs (branches, remote branches, tags) of a repository, for
// choosing which ones the history shows. Branch actions come in step 19.

use std::path::Path;

use crate::error::AppError;
use crate::git::repo::has_commits;
use crate::git::runner::run_git;
use crate::models::{RefInfo, RefKind};

/// Local branches, remote-tracking branches and tags, sorted by name within
/// each kind. A detached HEAD is listed too, as "HEAD".
pub fn list_refs(repo: &Path) -> Result<Vec<RefInfo>, AppError> {
    // %(HEAD) is "*" for the checked-out branch; %(upstream) the full name of
    // the branch it tracks (empty if none). %1f = the 0x1f separator
    // (for-each-ref writes hex escapes as %xx, unlike log's %x1f).
    let output = run_git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname)%1f%(HEAD)%1f%(upstream)",
            "refs/heads",
            "refs/remotes",
            "refs/tags",
        ],
    )?;

    let mut refs = Vec::new();
    if is_detached(repo) {
        refs.push(RefInfo {
            full_name: "HEAD".to_string(),
            name: "HEAD".to_string(),
            kind: RefKind::Head,
            current: true,
            upstream: None,
        });
    }
    // Ref names cannot contain newlines, so lines are safe here.
    for line in output.lines() {
        let mut fields = line.split('\x1f');
        let full_name = fields.next().unwrap_or("");
        let current = fields.next() == Some("*");
        let upstream = fields.next().filter(|u| !u.is_empty());
        if let Some(mut info) = ref_info(full_name, current) {
            info.upstream = upstream.map(|u| u.to_string());
            refs.push(info);
        }
    }
    Ok(refs)
}

/// Kind and short name of a full ref name; None for refs we don't show,
/// like "refs/remotes/origin/HEAD" (only says which branch is the default).
pub fn ref_info(full_name: &str, current: bool) -> Option<RefInfo> {
    const PREFIXES: [(&str, RefKind); 3] = [
        ("refs/heads/", RefKind::Branch),
        ("refs/remotes/", RefKind::Remote),
        ("refs/tags/", RefKind::Tag),
    ];
    for (prefix, kind) in PREFIXES {
        let Some(name) = full_name.strip_prefix(prefix) else {
            continue;
        };
        if kind == RefKind::Remote && name.ends_with("/HEAD") {
            return None;
        }
        return Some(RefInfo {
            full_name: full_name.to_string(),
            name: name.to_string(),
            kind,
            current,
            upstream: None,
        });
    }
    None
}

// HEAD points at a commit instead of a branch. (symbolic-ref fails then.)
fn is_detached(repo: &Path) -> bool {
    has_commits(repo) && run_git(repo, &["symbolic-ref", "--quiet", "HEAD"]).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{commit_file, git, server_and_clone};

    fn names(refs: &[RefInfo]) -> Vec<(&str, RefKind, bool)> {
        refs.iter()
            .map(|r| (r.full_name.as_str(), r.kind, r.current))
            .collect()
    }

    #[test]
    fn lists_branches_remotes_and_tags() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        git(root, &["branch", "feature/login"]);
        git(root, &["tag", "v1.0"]);
        // An annotated tag is a ref like any other.
        git(root, &["tag", "-a", "-m", "release", "v2.0"]);
        // origin/HEAD exists after a normal clone; make sure it does here too.
        git(root, &["remote", "set-head", "origin", "main"]);

        let refs = list_refs(root).unwrap();
        assert_eq!(
            names(&refs),
            vec![
                ("refs/heads/feature/login", RefKind::Branch, false),
                ("refs/heads/main", RefKind::Branch, true),
                ("refs/remotes/origin/main", RefKind::Remote, false),
                ("refs/tags/v1.0", RefKind::Tag, false),
                ("refs/tags/v2.0", RefKind::Tag, false),
            ]
        );
        assert_eq!(refs[0].name, "feature/login");
        assert_eq!(refs[2].name, "origin/main");
        // main tracks origin/main; the new branch tracks nothing.
        assert_eq!(
            refs[1].upstream.as_deref(),
            Some("refs/remotes/origin/main")
        );
        assert_eq!(refs[0].upstream, None);
    }

    #[test]
    fn lists_detached_head_first() {
        let (_server, work) = server_and_clone();
        commit_file(work.path(), "second");
        git(work.path(), &["checkout", "--quiet", "--detach", "HEAD~1"]);

        let refs = list_refs(work.path()).unwrap();
        assert_eq!(refs[0].full_name, "HEAD");
        assert_eq!(refs[0].kind, RefKind::Head);
        // No branch is current while HEAD is detached.
        assert!(refs[1..].iter().all(|r| !r.current));
    }

    #[test]
    fn empty_repository_has_no_refs() {
        let dir = tempfile::TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet"]);
        assert!(list_refs(dir.path()).unwrap().is_empty());
    }
}
