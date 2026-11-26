use cef::*;
use std::cell::RefCell;
use super::simple_handler::SimpleHandler;
wrap_window_delegate! {
    struct ArchWindowDelegate { browser_view: RefCell<Option<BrowserView>>, }
    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1180, height: 780 }
        }
    }
    impl PanelDelegate {}
    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            let browser_view = self.browser_view.borrow();
            let (Some(window), Some(browser_view)) = (window, browser_view.as_ref()) else { return; };
            let mut view = View::from(browser_view);
            window.add_child_view(Some(&mut view));
            window.show();
        }
        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            *self.browser_view.borrow_mut() = None;
        }
        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            let browser_view = self.browser_view.borrow();
            let Some(browser_view) = browser_view.as_ref() else { return 1; };
            match browser_view.browser() {
                Some(mut browser) => browser.host().map(|host| host.try_close_browser()).unwrap_or(1),
                None => 1,
            }
        }
        fn window_runtime_style(&self) -> RuntimeStyle { RuntimeStyle::DEFAULT }
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
wrap_browser_process_handler! {
    struct SimpleBrowserProcessHandler { client: RefCell<Option<Client>>, }
    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let command_line = command_line_get_global().expect("CEF command line unavailable");
            let requested_url = command_line.switch_value(Some(&CefString::from("url"))).to_string();
            let initial_url = if requested_url.is_empty() { "https://www.google.com/" } else { requested_url.as_str() };
            let url = CefString::from(initial_url);
            let settings = BrowserSettings::default();
            let mut client = SimpleHandler::new();
            *self.client.borrow_mut() = Some(client.clone());
            let browser_view = browser_view_create(Some(&mut client), Some(&url), Some(&settings), None, None, None);
            let mut delegate = ArchWindowDelegate::new(RefCell::new(browser_view));
            window_create_top_level(Some(&mut delegate));
        }
        fn default_client(&self) -> Option<Client> { self.client.borrow().clone() }
    }
}
