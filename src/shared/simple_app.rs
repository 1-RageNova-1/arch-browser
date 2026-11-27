//! Arch Browser's initial page and CEF browser window.
use cef::*;
use std::{cell::RefCell, env, fs, path::PathBuf};

use super::simple_handler::SimpleHandler;

wrap_window_delegate! {
    struct ArchWindowDelegate {
        browser_view: RefCell<Option<BrowserView>>,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1180, height: 780 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            let browser_view = self.browser_view.borrow();
            let (Some(window), Some(browser_view)) = (window, browser_view.as_ref()) else {
                return;
            };

            let mut layout_settings = BoxLayoutSettings::default();
            layout_settings.horizontal = 0;
            layout_settings.default_flex = 0;
            let layout = window.set_to_box_layout(Some(&layout_settings));

            let mut view = View::from(browser_view);
            window.add_child_view(Some(&mut view));

            if let Some(mut toolbar) = browser_view.chrome_toolbar() {
                window.add_child_view_at(Some(&mut toolbar), 0);
                if let Some(layout) = layout {
                    layout.set_flex_for_view(Some(&mut view), 1);
                }
            }

            window.layout();
            window.show();
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            *self.browser_view.borrow_mut() = None;
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            let browser_view = self.browser_view.borrow();
            let Some(browser_view) = browser_view.as_ref() else {
                return 1;
            };

            match browser_view.browser() {
                Some(browser) => browser.host().map(|host| host.try_close_browser()).unwrap_or(1),
                None => 1,
            }
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::CHROME
        }
    }
}

wrap_browser_view_delegate! {
    struct ArchBrowserViewDelegate;

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn chrome_toolbar_type(
            &self,
            _browser_view: Option<&mut BrowserView>,
        ) -> ChromeToolbarType {
            ChromeToolbarType::NORMAL
        }

        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::CHROME
        }
    }
}

wrap_app! {
    pub struct SimpleApp;

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(SimpleBrowserProcessHandler::new(RefCell::new(None)))
        }
    }
}

fn homepage_data_url() -> String {
    let html = include_str!("../homepage.html");
    let data_home = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    let homepage_path = data_home.join("arch-browser").join("homepage.html");

    if let Some(parent) = homepage_path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            log::warn!("Could not create persistent homepage directory: {error}");
        } else if let Err(error) = fs::write(&homepage_path, html) {
            log::warn!("Could not write persistent homepage: {error}");
        } else {
            let path = homepage_path.to_string_lossy();
            let mut encoded = String::with_capacity(path.len());
            for byte in path.bytes() {
                if byte.is_ascii_alphanumeric() || b"/-._~".contains(&byte) {
                    encoded.push(byte as char);
                } else {
                    encoded.push_str(&format!("%{byte:02X}"));
                }
            }
            return format!("file://{encoded}");
        }
    }

    let encoded = base64_encode(Some(html.as_bytes()));
    let encoded = CefString::from(&encoded);
    format!("data:text/html;base64,{}", encoded.to_string())
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

            let homepage = homepage_data_url();
            let initial_url = if requested_url.is_empty() {
                homepage.as_str()
            } else {
                requested_url.as_str()
            };

            log::info!("Opening initial page: {initial_url}");
            let url = CefString::from(initial_url);
            let settings = BrowserSettings::default();

            let mut client = SimpleHandler::new();
            *self.client.borrow_mut() = Some(client.clone());
            let mut browser_view_delegate = ArchBrowserViewDelegate::new();

            let browser_view = browser_view_create(
                Some(&mut client),
                Some(&url),
                Some(&settings),
                None,
                None,
                Some(&mut browser_view_delegate),
            );

            let mut delegate = ArchWindowDelegate::new(RefCell::new(browser_view));
            window_create_top_level(Some(&mut delegate));
        }

        fn default_client(&self) -> Option<Client> {
            self.client.borrow().clone()
        }
    }
}
