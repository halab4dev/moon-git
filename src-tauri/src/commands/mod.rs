// Tauri commands, grouped by feature. Commands stay thin: validate input,
// call into git/ or storage/, and return a model.

pub mod clone;
pub mod group;
pub mod repo;
