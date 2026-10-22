mod commands;
mod error;
mod git;
mod models;
mod storage;

use tauri::{Manager, WindowEvent};

use git::operations::Operations;
use storage::repos::RepoStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Operations::new())
        .setup(|app| {
            // Per-user app data folder, e.g. on macOS:
            // ~/Library/Application Support/com.github.halab4dev.moongit
            let data_dir = app.path().app_data_dir()?;
            app.manage(RepoStore::open(data_dir.join("repos.json")));
            Ok(())
        })
        // Closing the main window quits the app, diff windows included.
        // (By default the app keeps running while any window is open.)
        .on_window_event(|window, event| {
            if window.label() == "main" && matches!(event, WindowEvent::Destroyed) {
                window.app_handle().exit(0);
            }
        })
        // Every command callable from the frontend must be listed here.
        .invoke_handler(tauri::generate_handler![
            commands::repo::list_repos,
            commands::repo::add_repo,
            commands::repo::remove_repo,
            commands::repo::move_repo,
            commands::group::create_group,
            commands::group::rename_group,
            commands::group::delete_group,
            commands::clone::clone_repo,
            commands::clone::cancel_operation,
            commands::clone::suggest_folder_name,
            commands::status::get_status,
            commands::stage::stage_files,
            commands::stage::unstage_files,
            commands::diff::get_file_diff,
            commands::commit::commit_changes,
            commands::commit::get_head_commit,
            commands::fetch::fetch_repo,
            commands::pull::pull_repo,
            commands::pull::get_pull_default,
            commands::push::push_repo,
            commands::push::list_remotes,
            commands::log::get_log,
            commands::log::list_refs,
            commands::log::find_commits,
            commands::log::get_commit_details,
            commands::log::get_commit_file_diff,
            commands::branch::checkout_branch,
            commands::branch::create_branch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
