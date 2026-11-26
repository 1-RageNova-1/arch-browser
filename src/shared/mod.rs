//! CEF application bootstrap and process lifecycle.
use cef::*;

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

    arch_browser::privacy::start();

    let mut app = simple_app::SimpleApp::new();
    let settings = Settings {
        no_sandbox: 0,
        windowless_rendering_enabled: 0,
        ..Default::default()
    };

    assert_eq!(
        initialize(Some(main_args), Some(&settings), Some(&mut app), sandbox_info),
        1,
        "CEF initialization failed"
    );

    run_message_loop();
    shutdown();
}
