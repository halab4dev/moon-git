// Everything that runs git lives under this module.
// One file per area: runner (process handling), repo, status, stage, diff, later log, branch...

pub mod access;
pub mod branch;
pub mod branches;
pub mod cherry_pick;
pub mod clone;
pub mod commit;
pub mod conflict;
pub mod credential;
pub mod diff;
pub mod discard;
pub mod fetch;
pub mod identity;
pub mod log;
pub mod merge;
pub mod operations;
pub mod pull;
pub mod push;
pub mod rebase;
pub mod refs;
pub mod remote;
pub mod repo;
pub mod reset;
pub mod runner;
pub mod search;
pub mod show;
pub mod stage;
pub mod stash;
pub mod status;
#[cfg(test)]
pub mod test_support;
