use tauri::{AppHandle, Manager};

use super::run_blocking;
use crate::error::AppError;
use crate::models::Account;
use crate::storage::accounts::AccountStore;

// Saved logins (step 13a). The token goes from the "Sign in" dialog straight
// into the keyring; no command ever returns it.
//
// The keyring can be slow, or wait for the user (macOS may ask "allow access
// to the keychain?"), so every command runs off the main thread. The store
// is fetched inside the background task through the AppHandle, which (unlike
// State) can be moved there.

#[tauri::command]
pub async fn list_accounts(app: AppHandle) -> Result<Vec<Account>, AppError> {
    run_blocking(move || app.state::<AccountStore>().list()).await
}

/// The account git will use for `host`, to prefill the user name.
#[tauri::command]
pub async fn find_account(app: AppHandle, host: String) -> Result<Option<Account>, AppError> {
    run_blocking(move || app.state::<AccountStore>().find(&host)).await
}

/// Saves a login before it is tried. Retry the git command, then call
/// `finish_sign_in` with whether the server accepted it. Returns the account id.
#[tauri::command]
pub async fn begin_sign_in(
    app: AppHandle,
    host: String,
    username: String,
    token: String,
) -> Result<String, AppError> {
    run_blocking(move || {
        app.state::<AccountStore>()
            .begin_sign_in(&host, &username, &token)
    })
    .await
}

/// `accepted`: false = the server refused it; the previous state comes back.
#[tauri::command]
pub async fn finish_sign_in(app: AppHandle, id: String, accepted: bool) -> Result<(), AppError> {
    run_blocking(move || app.state::<AccountStore>().finish_sign_in(&id, accepted)).await
}
