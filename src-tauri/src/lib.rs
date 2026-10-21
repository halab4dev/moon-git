mod commands;
mod error;
mod git;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Every command callable from the frontend must be listed here.
        .invoke_handler(tauri::generate_handler![commands::repo::check_repo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
