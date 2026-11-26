//! CEF application bootstrap and process lifecycle.
use cef::*;
use std::{
    env,
    fs,
    path::PathBuf,
};

const CHROME_POLICY_ROOT: &str = "/etc/opt/arch-browser/policies";

struct BrowserDirectories {
    root_cache: PathBuf,
    profile: PathBuf,
}

fn prepare_browser_directories() -> std::io::Result<BrowserDirectories> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);

    let data_home = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/share"));

    let root_cache = data_home.join("arch-browser");
    let profile = root_cache.join("profile");

    fs::create_dir_all(&profile)?;

    let root_cache = fs::canonicalize(root_cache)?;
    let profile = fs::canonicalize(profile)?;

    Ok(BrowserDirectories {
        root_cache,
        profile,
    })
}

pub mod simple_app;
pub mod simple_handler;

pub struct Library;

pub fn load_cef() -> Library {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    Library
}

pub fn run_main(main_args: &MainArgs, cmd_line: &CommandLine, sandbox_info: *mut u8) {
    let process_type = CefString::from("type");
    let is_browser_process = cmd_line.has_switch(Some(&process_type)) != 1;

    let exit_code = execute_process(Some(main_args), None, sandbox_info);
    if !is_browser_process {
        assert!(exit_code >= 0, "CEF child process failed");
        return;
    }

    assert_eq!(exit_code, -1, "CEF browser process unexpectedly exited");

    let directories = prepare_browser_directories()
        .expect("could not prepare Arch Browser profile directories");

    arch_browser::privacy::start();

    let mut app = simple_app::SimpleApp::new();
    let settings = Settings {
        no_sandbox: 0,
        windowless_rendering_enabled: 0,
        chrome_policy_id: CefString::from(CHROME_POLICY_ROOT),
        root_cache_path: CefString::from(
            directories.root_cache.to_string_lossy().as_ref(),
        ),
        cache_path: CefString::from(
            directories.profile.to_string_lossy().as_ref(),
        ),
        persist_session_cookies: 1,
        ..Default::default()
    };

    log::info!("Starting CEF Chrome runtime with managed extension policies");
    log::info!("uBlock Origin Lite is force-installed and pinned by the Debian package policy");

    assert_eq!(
        initialize(Some(main_args), Some(&settings), Some(&mut app), sandbox_info),
        1,
        "CEF initialization failed"
    );

    run_message_loop();
    shutdown();
}
