// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // git starts this same program as its credential helper
    // ("moongit --git-credential get"): answer it, and don't open a window.
    let args: Vec<String> = std::env::args().collect();
    if args
        .iter()
        .any(|arg| arg == moongit_lib::CREDENTIAL_HELPER_ARG)
    {
        std::process::exit(moongit_lib::credential_helper(&args));
    }
    moongit_lib::run()
}
