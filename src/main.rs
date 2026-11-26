//! Arch Browser: Chromium Embedded Framework process entry point.
pub mod shared;
fn main() -> Result<(), &'static str> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let _cef_library = shared::load_cef();
    let args = cef::args::Args::new();
    let Some(command_line) = args.as_cmd_line() else {
        return Err("CEF could not parse the process command line");
    };
    shared::run_main(args.as_main_args(), &command_line, std::ptr::null_mut());
    Ok(())
}
