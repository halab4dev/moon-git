mod commands;
mod error;
mod git;
mod models;
mod storage;

use tauri::Manager;

use storage::repos::RepoStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Per-user app data folder, e.g. on macOS:
            // ~/Library/Application Support/com.github.halab4dev.moongit
            let data_dir = app.path().app_data_dir()?;
            app.manage(RepoStore::open(data_dir.join("repos.json")));
            Ok(())
        })
        // Every command callable from the frontend must be listed here.
        .invoke_handler(tauri::generate_handler![
            commands::repo::list_repos,
            commands::repo::add_repo,
            commands::repo::remove_repo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
