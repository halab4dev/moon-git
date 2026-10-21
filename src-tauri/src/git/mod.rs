// Everything that runs git lives under this module.
// One file per area: runner (process handling), repo, status, stage, later log, branch...

pub mod clone;
pub mod operations;
pub mod repo;
pub mod runner;
pub mod stage;
pub mod status;
