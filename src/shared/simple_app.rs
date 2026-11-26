use cef::*;
use std::cell::RefCell;
use super::simple_handler::SimpleHandler;

wrap_app! {
    pub struct SimpleApp;

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(SimpleBrowserProcessHandler::new(RefCell::new(None)))
        }
    }
}

wrap_browser_process_handler! {
    struct SimpleBrowserProcessHandler {
        client: RefCell<Option<Client>>,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let command_line = command_line_get_global().expect("CEF command line unavailable");
            let requested_url = cef::string::CefStringUtf16::from(
                &command_line.switch_value(Some(&CefString::from("url")))
            ).to_string();

            let initial_url = if requested_url.is_empty() {
                "https://example.com/"
            } else {
                requested_url.as_str()
            };
            let url = CefString::from(initial_url);
            let settings = BrowserSettings::default();

            let mut client = SimpleHandler::new();
            *self.client.borrow_mut() = Some(client.clone());

            // Use CEF's native Linux window path, matching the upstream cefsimple
            // example. The Views path is optional and is not required for startup.
            let window_info = WindowInfo {
                runtime_style: RuntimeStyle::DEFAULT,
                ..Default::default()
            };

            browser_host_create_browser(
                Some(&window_info),
                Some(&mut client),
                Some(&url),
                Some(&settings),
                None,
                None,
            );
        }

        fn default_client(&self) -> Option<Client> {
            self.client.borrow().clone()
        }
    }
}
