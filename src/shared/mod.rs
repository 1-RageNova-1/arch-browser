//! CEF application bootstrap and process lifecycle.
use cef::*;
use std::{
    env,
    fs,
    path::PathBuf,
};

const UBLOCK_ORIGIN_LITE_ID: &str = "ddkjiahejlhfcafbddmgiahcphecmpfh";
const CHROME_WEB_STORE_UPDATE_URL: &str = "https://clients2.google.com/service/update2/crx";

struct BrowserDirectories {
    policy_root: PathBuf,
    root_cache: PathBuf,
    profile: PathBuf,
}

fn prepare_browser_directories() -> std::io::Result<BrowserDirectories> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);

    let config_home = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));

    let data_home = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/share"));

    let app_config = config_home.join("arch-browser");
    let policy_root = app_config.join("policies");
    let managed_policy_dir = policy_root.join("managed");
    let root_cache = data_home.join("arch-browser");
    let profile = root_cache.join("profile");

    fs::create_dir_all(&managed_policy_dir)?;
    fs::create_dir_all(&profile)?;

    let policy_root = fs::canonicalize(policy_root)?;
    let root_cache = fs::canonicalize(root_cache)?;
    let profile = fs::canonicalize(profile)?;

    let policy = format!(
        r#"{{
  "ExtensionSettings": {{
    "*": {{
      "installation_mode": "allowed"
    }},
    "{UBLOCK_ORIGIN_LITE_ID}": {{
      "installation_mode": "force_installed",
      "update_url": "{CHROME_WEB_STORE_UPDATE_URL}"
    }}
  }}
}}
"#
    );

    fs::write(managed_policy_dir.join("arch-browser-extensions.json"), policy)?;

    Ok(BrowserDirectories {
        policy_root,
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
        .expect("could not prepare Arch Browser profile and extension policies");

    arch_browser::privacy::start();

    let mut app = simple_app::SimpleApp::new();
    let settings = Settings {
        no_sandbox: 0,
        windowless_rendering_enabled: 0,
        chrome_policy_id: CefString::from(
            directories.policy_root.to_string_lossy().as_ref(),
        ),
        root_cache_path: CefString::from(
            directories.root_cache.to_string_lossy().as_ref(),
        ),
        cache_path: CefString::from(
            directories.profile.to_string_lossy().as_ref(),
        ),
        persist_session_cookies: 1,
        ..Default::default()
    };

    log::info!("Starting CEF Chrome runtime with persistent profile");
    log::info!("uBlock Origin Lite is configured for automatic installation");

    assert_eq!(
        initialize(Some(main_args), Some(&settings), Some(&mut app), sandbox_info),
        1,
        "CEF initialization failed"
    );

    run_message_loop();
    shutdown();
}
