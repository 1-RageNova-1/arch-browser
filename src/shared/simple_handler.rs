use cef::*;
use std::os::raw::c_int;
use std::sync::{Arc, Mutex};

pub struct SimpleHandler;

impl SimpleHandler {
    pub fn new() -> Client {
        SimpleHandlerClient::new(Arc::new(Mutex::new(Vec::<Browser>::new())))
    }
}

wrap_client! {
    struct SimpleHandlerClient {
        browsers: Arc<Mutex<Vec<Browser>>>,
    }

    impl Client {
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(SimpleLifeSpanHandler::new(self.browsers.clone()))
        }

        fn request_handler(&self) -> Option<RequestHandler> {
            Some(ArchRequestHandler::new())
        }
    }
}

wrap_request_handler! {
    struct ArchRequestHandler;

    impl RequestHandler {
        fn resource_request_handler(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            _request: Option<&mut Request>,
            is_navigation: c_int,
            _is_download: c_int,
            request_initiator: Option<&CefString>,
            _disable_default_handling: Option<&mut c_int>,
        ) -> Option<ResourceRequestHandler> {
            if is_navigation != 0 {
                return None;
            }

            let source_url = request_initiator
                .map(|value| value.to_string())
                .filter(|value| !value.is_empty())
                .or_else(|| {
                    frame.map(|frame| {
                        cef::string::CefStringUtf16::from(&frame.url()).to_string()
                    })
                })
                .unwrap_or_default();

            Some(TrackerResourceRequestHandler::new(source_url))
        }
    }
}

wrap_resource_request_handler! {
    struct TrackerResourceRequestHandler {
        source_url: String,
    }

    impl ResourceRequestHandler {
        fn on_before_resource_load(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _callback: Option<&mut Callback>,
        ) -> ReturnValue {
            let Some(request) = request else {
                return ReturnValue::CONTINUE;
            };

            let url = cef::string::CefStringUtf16::from(&request.url()).to_string();
            let method = cef::string::CefStringUtf16::from(&request.method()).to_string();
            let resource_type = request.resource_type();

            let request_type = if resource_type == ResourceType::STYLESHEET {
                "stylesheet"
            } else if resource_type == ResourceType::SCRIPT
                || resource_type == ResourceType::WORKER
                || resource_type == ResourceType::SHARED_WORKER
                || resource_type == ResourceType::SERVICE_WORKER
            {
                "script"
            } else if resource_type == ResourceType::IMAGE
                || resource_type == ResourceType::FAVICON
            {
                "image"
            } else if resource_type == ResourceType::FONT_RESOURCE {
                "font"
            } else if resource_type == ResourceType::MEDIA {
                "media"
            } else if resource_type == ResourceType::OBJECT
                || resource_type == ResourceType::PLUGIN_RESOURCE
            {
                "object"
            } else if resource_type == ResourceType::PING {
                "ping"
            } else if resource_type == ResourceType::XHR {
                "xmlhttprequest"
            } else if resource_type == ResourceType::SUB_FRAME {
                "subdocument"
            } else {
                "other"
            };

            let source_url = if self.source_url.is_empty() {
                frame.map(|frame| {
                    cef::string::CefStringUtf16::from(&frame.url()).to_string()
                }).unwrap_or_default()
            } else {
                self.source_url.clone()
            };

            if arch_browser::privacy::should_block(
                &url,
                &source_url,
                request_type,
                if method.is_empty() { "GET" } else { &method },
            ) {
                log::debug!("Blocked tracking request: {url}");
                ReturnValue::CANCEL
            } else {
                ReturnValue::CONTINUE
            }
        }
    }
}

wrap_life_span_handler! {
    struct SimpleLifeSpanHandler {
        browsers: Arc<Mutex<Vec<Browser>>>,
    }

    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            if let Some(browser) = browser {
                self.browsers
                    .lock()
                    .expect("browser list poisoned")
                    .push(browser.clone());
            }
        }

        fn on_before_close(&self, browser: Option<&mut Browser>) {
            let Some(browser) = browser else {
                return;
            };

            let mut browsers = self.browsers.lock().expect("browser list poisoned");
            if let Some(index) = browsers
                .iter()
                .position(|item| item.is_same(Some(&mut browser.clone())) != 0)
            {
                browsers.remove(index);
            }

            if browsers.is_empty() {
                quit_message_loop();
            }
        }
    }
}
