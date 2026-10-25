// The branch list of the sidebar (step 25), like SmartGit's Branches view:
// local branches (with ahead/behind against their upstream), and the
// remote-tracking branches grouped by remote ("origin", "upstream"…).

use std::path::Path;

use crate::error::AppError;
use crate::git::branch::split_remote_name;
use crate::git::push::list_remotes;
use crate::git::refs::ref_info;
use crate::git::remote;
use crate::git::runner::run_git;
use crate::models::{BranchList, LocalBranch, RefKind, RemoteBranch, RemoteGroup};

pub fn list_branches(repo: &Path) -> Result<BranchList, AppError> {
    // %(upstream:short): "origin/main". %(upstream:track,nobracket):
    // "ahead 1, behind 2", "ahead 1", "behind 2", "gone" or "" (up to date / none).
    let output = run_git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname)%1f%(HEAD)%1f%(upstream:short)%1f%(upstream:track,nobracket)",
            "refs/heads",
            "refs/remotes",
        ],
    )?;
    let remotes = list_remotes(repo)?;
    let mut urls = remote::urls(repo)?;
    let mut list = BranchList {
        local: Vec::new(),
        // Every remote gets a group, even one without branches (not fetched yet).
        remotes: remotes
            .iter()
            .map(|name| RemoteGroup {
                name: name.clone(),
                url: urls.remove(name),
                branches: Vec::new(),
            })
            .collect(),
    };

    for line in output.lines() {
        let fields: Vec<&str> = line.split('\x1f').collect();
        let [full_name, head, upstream, track] = fields[..] else {
            continue;
        };
        // ref_info skips "refs/remotes/origin/HEAD" (it only names the default branch).
        let Some(info) = ref_info(full_name, head == "*") else {
            continue;
        };
        match info.kind {
            RefKind::Branch => {
                let (ahead, behind, gone) = parse_track(track);
                list.local.push(LocalBranch {
                    name: info.name,
                    full_name: info.full_name,
                    current: info.current,
                    upstream: Some(upstream.to_string()).filter(|u| !u.is_empty()),
                    ahead,
                    behind,
                    upstream_gone: gone,
                });
            }
            RefKind::Remote => {
                // "origin/feature/x" → group "origin", branch "feature/x". A
                // ref of a remote that was removed: group by its first part.
                let (remote, branch) = split_remote_name(&remotes, &info.name)
                    .or_else(|| {
                        info.name
                            .split_once('/')
                            .map(|(r, b)| (r.to_string(), b.to_string()))
                    })
                    .unwrap_or_else(|| (info.name.clone(), info.name.clone()));
                let branch = RemoteBranch {
                    name: branch,
                    full_name: info.full_name,
                };
                match list.remotes.iter_mut().find(|g| g.name == remote) {
                    Some(group) => group.branches.push(branch),
                    None => list.remotes.push(RemoteGroup {
                        name: remote,
                        url: None,
                        branches: vec![branch],
                    }),
                }
            }
            _ => {}
        }
    }
    Ok(list)
}

/// "ahead 1, behind 2" → (1, 2, false); "gone" → (0, 0, true); "" → (0, 0, false).
fn parse_track(track: &str) -> (u32, u32, bool) {
    if track == "gone" {
        return (0, 0, true);
    }
    let mut ahead = 0;
    let mut behind = 0;
    for part in track.split(", ") {
        if let Some(n) = part.strip_prefix("ahead ") {
            ahead = n.parse().unwrap_or(0);
        } else if let Some(n) = part.strip_prefix("behind ") {
            behind = n.parse().unwrap_or(0);
        }
    }
    (ahead, behind, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{clone_of, commit_file, git, server_and_clone};

    #[test]
    fn reads_the_track_text() {
        assert_eq!(parse_track("ahead 1, behind 2"), (1, 2, false));
        assert_eq!(parse_track("ahead 3"), (3, 0, false));
        assert_eq!(parse_track("behind 4"), (0, 4, false));
        assert_eq!(parse_track("gone"), (0, 0, true));
        assert_eq!(parse_track(""), (0, 0, false));
    }

    #[test]
    fn lists_local_branches_and_remotes_by_name() {
        let (server, work) = server_and_clone();
        let root = work.path();
        // A second remote with its own branches, and one with none yet.
        let other = clone_of(server.path());
        git(
            other.path(),
            &["switch", "--quiet", "--create", "feature/x"],
        );
        commit_file(other.path(), "x.txt");
        let other_url = other.path().to_string_lossy().to_string();
        git(root, &["remote", "add", "upstream", &other_url]);
        git(root, &["fetch", "--quiet", "upstream"]);
        git(root, &["remote", "add", "empty", "/nowhere"]);
        // main: one commit ahead of origin/main. topic: no upstream.
        commit_file(root, "local.txt");
        git(root, &["branch", "topic"]);

        let list = list_branches(root).unwrap();

        let local: Vec<(&str, bool, Option<&str>, u32)> = list
            .local
            .iter()
            .map(|b| (b.name.as_str(), b.current, b.upstream.as_deref(), b.ahead))
            .collect();
        assert_eq!(
            local,
            [
                ("main", true, Some("origin/main"), 1),
                ("topic", false, None, 0)
            ]
        );

        let groups: Vec<(&str, Vec<&str>)> = list
            .remotes
            .iter()
            .map(|g| {
                (
                    g.name.as_str(),
                    g.branches.iter().map(|b| b.name.as_str()).collect(),
                )
            })
            .collect();
        // `git remote` lists remotes by name; origin/HEAD is left out.
        assert_eq!(
            groups,
            [
                ("empty", vec![]),
                ("origin", vec!["main"]),
                ("upstream", vec!["feature/x", "main"]),
            ]
        );
        assert_eq!(list.remotes[0].url.as_deref(), Some("/nowhere"));
        assert_eq!(list.remotes[2].url.as_deref(), Some(other_url.as_str()));
        assert_eq!(
            list.remotes[2].branches[0].full_name,
            "refs/remotes/upstream/feature/x"
        );
    }

    #[test]
    fn marks_a_gone_upstream() {
        let (_server, work) = server_and_clone();
        let root = work.path();
        git(root, &["switch", "--quiet", "--create", "temp"]);
        git(root, &["push", "--quiet", "-u", "origin", "temp"]);
        git(root, &["push", "--quiet", "origin", "--delete", "temp"]);
        let list = list_branches(root).unwrap();
        let temp = list.local.iter().find(|b| b.name == "temp").unwrap();
        assert!(temp.upstream_gone);
    }
}
