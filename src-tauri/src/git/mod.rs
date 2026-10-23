// Everything that runs git lives under this module.
// One file per area: runner (process handling), repo, status, stage, diff, later log, branch...

pub mod branch;
pub mod clone;
pub mod commit;
pub mod diff;
pub mod fetch;
pub mod log;
pub mod merge;
pub mod operations;
pub mod pull;
pub mod push;
pub mod rebase;
pub mod refs;
pub mod repo;
pub mod runner;
pub mod search;
pub mod show;
pub mod stage;
pub mod stash;
pub mod status;
#[cfg(test)]
pub mod test_support;
