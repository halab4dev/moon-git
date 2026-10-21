// Everything that runs git lives under this module.
// One file per area: runner (process handling), repo, later status, log, branch...

pub mod clone;
pub mod operations;
pub mod repo;
pub mod runner;
