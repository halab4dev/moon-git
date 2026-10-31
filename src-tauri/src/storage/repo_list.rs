// Rules for changing the repository list: groups, moves, duplicates.
// Pure in-memory logic, no file access, so it is easy to test.
// RepoStore (repos.rs) wraps these with locking and saving.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AppError, ErrorKind};
use crate::git::identity::clean_identity;
use crate::models::{GroupIdentity, RepoEntry, RepoGroup, RepoInfo, RepoList};

const MAX_NAME_LEN: usize = 100;

impl RepoList {
    pub fn add_repo(
        &mut self,
        info: RepoInfo,
        group_id: Option<String>,
    ) -> Result<RepoEntry, AppError> {
        self.check_group_exists(&group_id)?;

        if let Some(existing) = self.repos.iter().find(|r| r.path == info.root) {
            return Err(AppError::new(
                ErrorKind::RepoAlreadyAdded,
                format!("{} is already in the list.", existing.name),
            ));
        }

        let entry = RepoEntry {
            id: new_id(),
            name: info.name,
            path: info.root,
            group_id,
        };
        self.repos.push(entry.clone());
        Ok(entry)
    }

    /// Removes the entry from the list only; the folder on disk is not touched.
    pub fn remove_repo(&mut self, id: &str) {
        self.repos.retain(|r| r.id != id);
    }

    pub fn move_repo(&mut self, repo_id: &str, group_id: Option<String>) -> Result<(), AppError> {
        self.check_group_exists(&group_id)?;
        let repo = self
            .repos
            .iter_mut()
            .find(|r| r.id == repo_id)
            .ok_or_else(|| not_found("Repository"))?;
        repo.group_id = group_id;
        Ok(())
    }

    pub fn create_group(
        &mut self,
        name: &str,
        parent_id: Option<String>,
    ) -> Result<RepoGroup, AppError> {
        self.check_group_exists(&parent_id)?;
        let name = clean_name(name)?;
        self.check_unique_name(&name, &parent_id, None)?;

        let group = RepoGroup {
            id: new_id(),
            name,
            parent_id,
            identity: None,
        };
        self.groups.push(group.clone());
        Ok(group)
    }

    pub fn rename_group(&mut self, id: &str, name: &str) -> Result<(), AppError> {
        let name = clean_name(name)?;
        let parent_id = self.find_group(id)?.parent_id.clone();
        self.check_unique_name(&name, &parent_id, Some(id))?;

        // find_group above proved the group exists, so this always finds it.
        if let Some(group) = self.groups.iter_mut().find(|g| g.id == id) {
            group.name = name;
        }
        Ok(())
    }

    /// Deletes the group but keeps its contents: subgroups and repos move up
    /// one level, into the deleted group's parent.
    pub fn delete_group(&mut self, id: &str) -> Result<(), AppError> {
        let parent_id = self.find_group(id)?.parent_id.clone();

        // as_deref turns Option<String> into Option<&str> so it can be compared with `id`.
        for child in self
            .groups
            .iter_mut()
            .filter(|g| g.parent_id.as_deref() == Some(id))
        {
            child.parent_id = parent_id.clone();
        }
        for repo in self
            .repos
            .iter_mut()
            .filter(|r| r.group_id.as_deref() == Some(id))
        {
            repo.group_id = parent_id.clone();
        }
        self.groups.retain(|g| g.id != id);
        Ok(())
    }

    /// `identity`: (name, email) for the repositories in this group (and its
    /// subgroups, unless they have their own); None = remove it.
    pub fn set_group_identity(
        &mut self,
        id: &str,
        identity: Option<(&str, &str)>,
    ) -> Result<(), AppError> {
        let identity = match identity {
            Some((name, email)) => {
                let (name, email) = clean_identity(name, email)?;
                Some(GroupIdentity { name, email })
            }
            None => None,
        };
        let group = self
            .groups
            .iter_mut()
            .find(|g| g.id == id)
            .ok_or_else(|| not_found("Group"))?;
        group.identity = identity;
        Ok(())
    }

    /// The repositories of a group, including those of its subgroups.
    pub fn repos_in_group(&self, group_id: &str) -> Vec<&RepoEntry> {
        let mut groups = vec![group_id.to_string()];
        // Collect the subgroups level by level (at most one pass per group).
        for _ in 0..self.groups.len() {
            let more: Vec<String> = self
                .groups
                .iter()
                .filter(|g| {
                    g.parent_id.as_ref().is_some_and(|p| groups.contains(p))
                        && !groups.contains(&g.id)
                })
                .map(|g| g.id.clone())
                .collect();
            if more.is_empty() {
                break;
            }
            groups.extend(more);
        }
        self.repos
            .iter()
            .filter(|r| r.group_id.as_ref().is_some_and(|g| groups.contains(g)))
            .collect()
    }

    fn find_group(&self, id: &str) -> Result<&RepoGroup, AppError> {
        self.groups
            .iter()
            .find(|g| g.id == id)
            .ok_or_else(|| not_found("Group"))
    }

    fn check_group_exists(&self, group_id: &Option<String>) -> Result<(), AppError> {
        match group_id {
            Some(id) => self.find_group(id).map(|_| ()),
            None => Ok(()),
        }
    }

    // Two groups with the same parent may not share a name (ignoring case).
    // `skip_id` lets a group keep its own name when renaming.
    fn check_unique_name(
        &self,
        name: &str,
        parent_id: &Option<String>,
        skip_id: Option<&str>,
    ) -> Result<(), AppError> {
        let taken = self.groups.iter().any(|g| {
            g.parent_id == *parent_id
                && Some(g.id.as_str()) != skip_id
                && g.name.to_lowercase() == name.to_lowercase()
        });
        if taken {
            return Err(AppError::new(
                ErrorKind::InvalidInput,
                format!("A group named \"{name}\" already exists here."),
            ));
        }
        Ok(())
    }
}

fn clean_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new(
            ErrorKind::InvalidInput,
            "The name cannot be empty.".to_string(),
        ));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(AppError::new(
            ErrorKind::InvalidInput,
            format!("The name is too long (max {MAX_NAME_LEN} characters)."),
        ));
    }
    Ok(name.to_string())
}

fn not_found(what: &str) -> AppError {
    AppError::new(
        ErrorKind::NotFound,
        format!("{what} not found. The list may be out of date; try again."),
    )
}

// Nanoseconds since 1970, in hex. Ids are created one at a time under the store's
// lock by user actions, so this is unique enough without pulling in a UUID crate.
fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(root: &str) -> RepoInfo {
        RepoInfo {
            root: root.to_string(),
            name: root.rsplit('/').next().unwrap().to_string(),
        }
    }

    #[test]
    fn group_identity_and_repos_of_a_group() {
        let mut list = RepoList::default();
        let work = list.create_group("Work", None).unwrap();
        let team = list.create_group("Team", Some(work.id.clone())).unwrap();
        let other = list.create_group("Other", None).unwrap();
        list.add_repo(info("/w/a"), Some(work.id.clone())).unwrap();
        list.add_repo(info("/w/t/b"), Some(team.id.clone()))
            .unwrap();
        list.add_repo(info("/o/c"), Some(other.id.clone())).unwrap();
        list.add_repo(info("/d"), None).unwrap();

        // A group's repositories include its subgroups' ones.
        let paths = |id: &str| {
            list.repos_in_group(id)
                .iter()
                .map(|r| r.path.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(paths(&work.id), ["/w/a", "/w/t/b"]);
        assert_eq!(paths(&team.id), ["/w/t/b"]);

        list.set_group_identity(&work.id, Some((" Me ", "me@work.example")))
            .unwrap();
        let identity = list
            .groups
            .iter()
            .find(|g| g.id == work.id)
            .unwrap()
            .identity
            .clone()
            .unwrap();
        assert_eq!(
            (identity.name.as_str(), identity.email.as_str()),
            ("Me", "me@work.example")
        );
        list.set_group_identity(&work.id, None).unwrap();
        assert!(list.groups.iter().all(|g| g.identity.is_none()));

        let err = list
            .set_group_identity(&work.id, Some(("Me", "no-at-sign")))
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn creates_nested_groups() {
        let mut list = RepoList::default();
        let work = list.create_group("Work", None).unwrap();
        let client = list
            .create_group("  Client A  ", Some(work.id.clone()))
            .unwrap();

        assert_eq!(client.name, "Client A");
        assert_eq!(client.parent_id, Some(work.id));
        assert_eq!(list.groups.len(), 2);
    }

    #[test]
    fn rejects_empty_and_too_long_names() {
        let mut list = RepoList::default();
        assert_eq!(
            list.create_group("   ", None).unwrap_err().kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(
            list.create_group(&"x".repeat(101), None).unwrap_err().kind,
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn sibling_names_must_differ_ignoring_case() {
        let mut list = RepoList::default();
        let work = list.create_group("Work", None).unwrap();
        list.create_group("Tools", Some(work.id.clone())).unwrap();

        let err = list
            .create_group("tools", Some(work.id.clone()))
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
        // Same name under a different parent is fine.
        list.create_group("Tools", None).unwrap();
    }

    #[test]
    fn rejects_unknown_parent() {
        let mut list = RepoList::default();
        let err = list
            .create_group("X", Some("missing".to_string()))
            .unwrap_err();
        assert_eq!(err.kind, ErrorKind::NotFound);
    }

    #[test]
    fn renames_group_and_allows_changing_case_of_own_name() {
        let mut list = RepoList::default();
        let g = list.create_group("work", None).unwrap();
        list.rename_group(&g.id, "Work").unwrap();
        assert_eq!(list.groups[0].name, "Work");
    }

    #[test]
    fn rename_rejects_name_of_sibling() {
        let mut list = RepoList::default();
        list.create_group("A", None).unwrap();
        let b = list.create_group("B", None).unwrap();
        assert_eq!(
            list.rename_group(&b.id, "a").unwrap_err().kind,
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn delete_moves_contents_up_one_level() {
        let mut list = RepoList::default();
        let work = list.create_group("Work", None).unwrap();
        let client = list.create_group("Client", Some(work.id.clone())).unwrap();
        let inner = list.create_group("Inner", Some(client.id.clone())).unwrap();
        let repo = list
            .add_repo(info("/w/app"), Some(client.id.clone()))
            .unwrap();

        list.delete_group(&client.id).unwrap();

        assert!(list.groups.iter().all(|g| g.id != client.id));
        let inner_after = list.groups.iter().find(|g| g.id == inner.id).unwrap();
        assert_eq!(inner_after.parent_id, Some(work.id.clone()));
        let repo_after = list.repos.iter().find(|r| r.id == repo.id).unwrap();
        assert_eq!(repo_after.group_id, Some(work.id));
    }

    #[test]
    fn adds_repo_into_group_and_moves_it() {
        let mut list = RepoList::default();
        let g = list.create_group("G", None).unwrap();
        let repo = list.add_repo(info("/w/app"), Some(g.id.clone())).unwrap();
        assert_eq!(repo.group_id, Some(g.id));

        list.move_repo(&repo.id, None).unwrap();
        assert_eq!(list.repos[0].group_id, None);
    }

    #[test]
    fn move_rejects_unknown_repo_or_group() {
        let mut list = RepoList::default();
        let repo = list.add_repo(info("/w/app"), None).unwrap();
        assert_eq!(
            list.move_repo("nope", None).unwrap_err().kind,
            ErrorKind::NotFound
        );
        assert_eq!(
            list.move_repo(&repo.id, Some("nope".to_string()))
                .unwrap_err()
                .kind,
            ErrorKind::NotFound
        );
    }

    #[test]
    fn rejects_duplicate_repo_path() {
        let mut list = RepoList::default();
        list.add_repo(info("/w/app"), None).unwrap();
        assert_eq!(
            list.add_repo(info("/w/app"), None).unwrap_err().kind,
            ErrorKind::RepoAlreadyAdded
        );
    }
}
