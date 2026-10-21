// Filtering the history by commit id prefix or message text (step 17).
//
// Both filter modes need to look at the whole history, not only the pages
// loaded so far, so each call walks every commit of the selected refs once
// (with full messages). About 0.5 s for 30,000 commits.
//   - Highlight mode asks for the ids of all matching commits.
//   - Hide mode asks for pages of matching commits only. Their parents are
//     rewritten to the nearest matching ancestors, so the graph stays
//     connected instead of showing loose lines.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{AppError, ErrorKind};
use crate::git::log::{MAX_PAGE, parse_log, run_log};
use crate::models::{CommitSummary, LogPage};

// Like log.rs, but with the full message (%B) as the last field instead of
// the subject: the filter searches the body too.
const FORMAT_WITH_MESSAGE: &str = "--format=%H%x1f%P%x1f%aN%x1f%aE%x1f%at%x1f%D%x1f%B";

/// A commit matches when its id starts with `text` (if `text` looks like a
/// hash), or its message contains `text`. Case-insensitive either way.
pub fn matches_filter(id: &str, message: &str, text: &str) -> bool {
    let needle = text.trim().to_lowercase();
    if needle.is_empty() {
        return true;
    }
    let looks_like_hash = needle.chars().all(|c| c.is_ascii_hexdigit());
    (looks_like_hash && id.starts_with(&needle)) || message.to_lowercase().contains(&needle)
}

/// Ids of all matching commits, newest first (the order of the history).
pub fn find_matches(
    repo: &Path,
    refs: Option<&[String]>,
    text: &str,
) -> Result<Vec<String>, AppError> {
    let commits = walk(repo, refs)?;
    Ok(commits
        .into_iter()
        .filter(|(commit, message)| matches_filter(&commit.id, message, text))
        .map(|(commit, _)| commit.id)
        .collect())
}

/// Like `read_log`, but only matching commits, with rewritten parents.
pub fn read_filtered_log(
    repo: &Path,
    skip: usize,
    limit: usize,
    refs: Option<&[String]>,
    text: &str,
) -> Result<LogPage, AppError> {
    if limit == 0 || limit > MAX_PAGE {
        return Err(AppError::new(
            ErrorKind::InvalidInput,
            format!("Page size must be between 1 and {MAX_PAGE}."),
        ));
    }
    let walked = walk(repo, refs)?;
    let matched: Vec<bool> = walked
        .iter()
        .map(|(commit, message)| matches_filter(&commit.id, message, text))
        .collect();
    let commits: Vec<CommitSummary> = walked.into_iter().map(|(commit, _)| commit).collect();
    let mut rewritten = rewrite_parents(&commits, &matched);

    let mut page = Vec::new();
    let mut total = 0;
    for (index, commit) in commits.into_iter().enumerate() {
        if !matched[index] {
            continue;
        }
        if total >= skip && total < skip + limit {
            page.push(CommitSummary {
                parents: std::mem::take(&mut rewritten[index]),
                ..commit
            });
        }
        total += 1;
    }
    Ok(LogPage {
        commits: page,
        has_more: total > skip + limit,
    })
}

/// Every commit of the selected refs, with its full message.
fn walk(repo: &Path, refs: Option<&[String]>) -> Result<Vec<(CommitSummary, String)>, AppError> {
    let Some(output) = run_log(repo, FORMAT_WITH_MESSAGE, refs, &[])? else {
        return Ok(Vec::new());
    };
    // parse_log reads the last field as the subject; here it is the full
    // message, so the real subject is taken from it afterwards.
    let commits = parse_log(&output)?;
    Ok(commits
        .into_iter()
        .map(|mut commit| {
            let message = std::mem::take(&mut commit.subject);
            commit.subject = subject_of(&message);
            (commit, message)
        })
        .collect())
}

/// The subject like git's %s: the first paragraph, its lines joined by spaces.
fn subject_of(message: &str) -> String {
    message
        .lines()
        .map(|line| line.trim())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// For each matching commit: its nearest matching ancestors, found by
/// walking through the commits that don't match. Entries of commits that
/// don't match stay empty.
///
/// Works from the oldest commit up (the list is newest first, every commit
/// before its parents), so a commit's parents are always done before it.
/// No recursion: a history of 50,000 commits would overflow the stack.
fn rewrite_parents(commits: &[CommitSummary], matched: &[bool]) -> Vec<Vec<String>> {
    let index: HashMap<&str, usize> = commits
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.as_str(), i))
        .collect();
    // reach[i]: the matching commits you meet first when walking down from i
    // (i itself if it matches). Stored as indexes into `commits`.
    let mut reach: Vec<Vec<usize>> = vec![Vec::new(); commits.len()];
    let mut rewritten: Vec<Vec<String>> = vec![Vec::new(); commits.len()];

    for i in (0..commits.len()).rev() {
        // The matching commits reached through each parent, without duplicates,
        // in parent order (first parent first, as the graph expects).
        let mut below: Vec<usize> = Vec::new();
        for parent in &commits[i].parents {
            let Some(&p) = index.get(parent.as_str()) else {
                continue; // outside the walked history
            };
            for &found in &reach[p] {
                if !below.contains(&found) {
                    below.push(found);
                }
            }
        }
        if matched[i] {
            rewritten[i] = below.iter().map(|&b| commits[b].id.clone()).collect();
            reach[i] = vec![i];
        } else {
            reach[i] = below;
        }
    }
    rewritten
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_support::{commit_file, git};
    use tempfile::TempDir;

    fn new_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "--quiet", "--initial-branch=main"]);
        git(dir.path(), &["config", "user.name", "Test"]);
        git(dir.path(), &["config", "user.email", "test@example.com"]);
        dir
    }

    fn commit_message(dir: &Path, message: &str) {
        git(dir, &["commit", "--quiet", "--allow-empty", "-m", message]);
    }

    fn subjects(page: &LogPage) -> Vec<&str> {
        page.commits.iter().map(|c| c.subject.as_str()).collect()
    }

    fn id_of(page: &LogPage, subject: &str) -> String {
        page.commits
            .iter()
            .find(|c| c.subject == subject)
            .unwrap()
            .id
            .clone()
    }

    #[test]
    fn matches_message_or_hash_prefix() {
        let id = "a1b2c3d4e5";
        assert!(matches_filter(id, "Fix Login", "login"));
        assert!(matches_filter(
            id,
            "subject\n\nbody mentions TICKET-42",
            "ticket-42"
        ));
        assert!(matches_filter(id, "unrelated", "A1B2"));
        // Hex text in the middle of an id does not count; only a prefix does.
        assert!(!matches_filter(id, "unrelated", "c3d4"));
        // Not hex: only the message is searched.
        assert!(!matches_filter(id, "unrelated", "a1b2x"));
        assert!(matches_filter(id, "anything", "   "));
    }

    #[test]
    fn subject_is_the_first_paragraph() {
        assert_eq!(subject_of("Fix bug\n\nLong body"), "Fix bug");
        assert_eq!(
            subject_of("Wrapped\nsubject line\n\nbody"),
            "Wrapped subject line"
        );
        assert_eq!(subject_of(""), "");
    }

    #[test]
    fn finds_matches_in_the_whole_history_newest_first() {
        let dir = new_repo();
        let root = dir.path();
        commit_message(root, "feat: login");
        commit_message(root, "chore: deps");
        commit_message(root, "fix: login timeout\n\nCloses #12");
        commit_message(root, "docs");

        let ids = find_matches(root, None, "LOGIN").unwrap();
        assert_eq!(ids.len(), 2);
        let all = crate::git::log::read_log(root, 0, 10, None).unwrap();
        assert_eq!(
            ids,
            vec![
                id_of(&all, "fix: login timeout"),
                id_of(&all, "feat: login")
            ]
        );

        // The body is searched too, and a hash prefix works.
        assert_eq!(find_matches(root, None, "#12").unwrap().len(), 1);
        let docs = id_of(&all, "docs");
        assert_eq!(find_matches(root, None, &docs[..7]).unwrap(), vec![docs]);
    }

    #[test]
    fn hide_mode_links_matches_to_their_nearest_matching_ancestor() {
        let dir = new_repo();
        let root = dir.path();
        commit_message(root, "one");
        commit_message(root, "match two");
        commit_message(root, "three");
        commit_message(root, "match four");

        let page = read_filtered_log(root, 0, 100, None, "match").unwrap();
        assert_eq!(subjects(&page), vec!["match four", "match two"]);
        // "three" is skipped: four's parent becomes two. "one" is skipped: two has none.
        assert_eq!(page.commits[0].parents, vec![page.commits[1].id.clone()]);
        assert!(page.commits[1].parents.is_empty());
        assert!(!page.has_more);
    }

    #[test]
    fn hide_mode_rewrites_merges_without_duplicates() {
        //   merge (match)
        //   |    \
        //   m1    s1 (match)
        //   |     |
        //   base (match)
        let dir = new_repo();
        let root = dir.path();
        commit_message(root, "match base");
        git(root, &["checkout", "--quiet", "-b", "side"]);
        commit_file(root, "match s1");
        git(root, &["checkout", "--quiet", "main"]);
        commit_file(root, "m1");
        git(
            root,
            &["merge", "--quiet", "--no-ff", "-m", "match merge", "side"],
        );

        let page = read_filtered_log(root, 0, 100, None, "match").unwrap();
        assert_eq!(
            subjects(&page),
            vec!["match merge", "match s1", "match base"]
        );
        let base = id_of(&page, "match base");
        let s1 = id_of(&page, "match s1");
        // Through m1 the merge reaches base; through its second parent, s1.
        assert_eq!(page.commits[0].parents, vec![base.clone(), s1]);
        assert_eq!(page.commits[1].parents, vec![base]);
    }

    #[test]
    fn hide_mode_pages_and_respects_the_ref_selection() {
        let dir = new_repo();
        let root = dir.path();
        for i in 0..5 {
            commit_message(root, &format!("match {i}"));
        }
        git(root, &["checkout", "--quiet", "-b", "side"]);
        commit_message(root, "match on side");
        git(root, &["checkout", "--quiet", "main"]);

        let first = read_filtered_log(root, 0, 2, None, "match").unwrap();
        assert_eq!(subjects(&first), vec!["match on side", "match 4"]);
        assert!(first.has_more);
        let last = read_filtered_log(root, 4, 2, None, "match").unwrap();
        assert_eq!(subjects(&last), vec!["match 1", "match 0"]);
        assert!(!last.has_more);

        let only_main = vec!["refs/heads/main".to_string()];
        let page = read_filtered_log(root, 0, 100, Some(&only_main), "match").unwrap();
        assert_eq!(page.commits.len(), 5);
        assert_eq!(
            find_matches(root, Some(&only_main), "side").unwrap().len(),
            0
        );
    }

    #[test]
    fn empty_repository_has_no_matches() {
        let dir = new_repo();
        assert!(find_matches(dir.path(), None, "x").unwrap().is_empty());
        assert!(
            read_filtered_log(dir.path(), 0, 10, None, "x")
                .unwrap()
                .commits
                .is_empty()
        );
    }
}
