//! Arch Browser - early experimental skeleton
//!
//! This binary follows the CEF multi-process model.
//! The same executable is re-launched by CEF for renderer / GPU / utility processes.
//!
//! Current state: minimal entry point only. Real browser window, tabs, UI chrome,
//! bookmarks, etc. are not implemented yet.

use std::process;

fn main() {
    // Basic logging so we can see CEF startup messages
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Arch Browser v0.1.0 - experimental CEF skeleton");

    // TODO: proper CEF initialization following tauri-apps/cef-rs examples
    // (Args, execute_process, initialize, create browser window, run message loop)
    //
    // For now we only print a clear message so the project compiles and the
    // multi-process entry point is ready to be filled.

    eprintln!("Arch Browser skeleton");
    eprintln!("====================");
    eprintln!("This is an early experimental build.");
    eprintln!("Full CEF browser window is not yet implemented.");
    eprintln!("See README.md for architecture notes and next steps.");
    eprintln!("");
    eprintln!("Repository: https://github.com/1-RageNova-1/arch-browser");

    // Exit cleanly. When real CEF code is added, this will become the
    // message loop + proper shutdown.
    process::exit(0);
}
