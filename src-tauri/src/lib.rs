mod commands;
mod error;
mod git;
mod models;
mod storage;

use tauri::{Manager, WindowEvent};

use git::operations::Operations;
use storage::accounts::{AccountStore, Keyring};
use storage::repos::RepoStore;

/// The argument git uses to start MoonGit as its credential helper (step 12).
pub const CREDENTIAL_HELPER_ARG: &str = git::credential::HELPER_ARG;

/// Helper mode: answers git on stdin/stdout, opens no window. Returns the exit code.
pub fn credential_helper(args: &[String]) -> i32 {
    git::credential::helper_main(args)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Edit → Copy Repository Path (step 31a).
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Operations::new())
        .setup(|app| {
            // Per-user app data folder, e.g. on macOS:
            // ~/Library/Application Support/com.github.halab4dev.moongit
            let data_dir = app.path().app_data_dir()?;
            app.manage(RepoStore::open(data_dir.join("repos.json")));
            // Saved logins (step 13a): accounts.json here, tokens in the OS keyring.
            git::credential::set_data_dir(data_dir.clone());
            app.manage(AccountStore::new(data_dir, Box::new(Keyring)));
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
            commands::account::list_accounts,
            commands::account::find_account,
            commands::account::begin_sign_in,
            commands::account::finish_sign_in,
            commands::account::remove_account,
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
            commands::stage::discard_changes,
            commands::stage::resolve_conflicts,
            commands::stage::get_conflict_versions,
            commands::stage::save_conflict_result,
            commands::diff::get_file_diff,
            commands::commit::commit_changes,
            commands::commit::get_head_commit,
            commands::fetch::fetch_repo,
            commands::pull::pull_repo,
            commands::pull::get_pull_default,
            commands::push::push_repo,
            commands::push::force_push_preview,
            commands::push::force_push_repo,
            commands::push::list_remotes,
            commands::log::get_log,
            commands::log::list_refs,
            commands::log::find_commits,
            commands::log::get_commit_details,
            commands::log::get_commit_file_diff,
            commands::branch::checkout_branch,
            commands::branch::create_branch,
            commands::branch::rename_branch,
            commands::branch::delete_branch,
            commands::branch::delete_remote_branch,
            commands::branch::set_branch_upstream,
            commands::branch::list_branches,
            commands::branch::add_remote,
            commands::branch::set_remote_url,
            commands::stash::create_stash,
            commands::stash::list_stashes,
            commands::stash::run_stash_action,
            commands::merge::get_merge_preview,
            commands::merge::merge_into_current,
            commands::merge::abort_merge,
            commands::rebase::get_rebase_preview,
            commands::rebase::rebase_current,
            commands::rebase::run_rebase_action,
            commands::reset::get_reset_preview,
            commands::reset::reset_current,
            commands::cherry_pick::cherry_pick_commit,
            commands::cherry_pick::run_cherry_pick_action,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
