use cef::*;
use std::sync::{Arc, Mutex};
pub struct SimpleHandler;
impl SimpleHandler {
    pub fn new() -> Client {
        SimpleHandlerClient::new(Arc::new(Mutex::new(Vec::<Browser>::new())))
    }
}
wrap_client! {
    struct SimpleHandlerClient { browsers: Arc<Mutex<Vec<Browser>>>, }
    impl Client {
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(SimpleLifeSpanHandler::new(self.browsers.clone()))
        }
    }
}
wrap_life_span_handler! {
    struct SimpleLifeSpanHandler { browsers: Arc<Mutex<Vec<Browser>>>, }
    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            if let Some(browser) = browser {
                self.browsers.lock().expect("browser list poisoned").push(browser.clone());
            }
        }
        fn on_before_close(&self, browser: Option<&mut Browser>) {
            let Some(browser) = browser else { return; };
            let mut browsers = self.browsers.lock().expect("browser list poisoned");
            if let Some(index) = browsers.iter().position(|item| item.is_same(Some(&mut browser.clone())) != 0) {
                browsers.remove(index);
            }
            if browsers.is_empty() { quit_message_loop(); }
        }
    }
}
