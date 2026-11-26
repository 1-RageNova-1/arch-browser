//! Network-level tracker filtering using Brave's adblock-rust engine.
//! The engine is !Send/!Sync, so it lives on one dedicated worker thread.
use adblock::{
    lists::{FilterSet, ParseOptions},
    request::Request,
    Engine,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, SyncSender, TrySendError},
        OnceLock,
    },
    thread,
    time::Duration,
};

const FILTER_DIRECTORY: &str = "/usr/share/arch-browser/filters";
const REQUEST_QUEUE_CAPACITY: usize = 512;
const FILTER_TIMEOUT: Duration = Duration::from_millis(250);

struct FilterCheck {
    url: String,
    source_url: String,
    request_type: String,
    method: String,
    response: SyncSender<bool>,
}

static FILTER_WORKER: OnceLock<SyncSender<FilterCheck>> = OnceLock::new();
static FILTER_STARTED: OnceLock<()> = OnceLock::new();

pub fn start() {
    FILTER_STARTED.get_or_init(|| {
        let (sender, receiver) = mpsc::sync_channel::<FilterCheck>(REQUEST_QUEUE_CAPACITY);
        let (ready_sender, ready_receiver) = mpsc::sync_channel::<()>(1);

        let _ = FILTER_WORKER.set(sender);

        thread::Builder::new()
            .name("arch-browser-filter-engine".to_owned())
            .spawn(move || {
                let engine = load_engine();
                log::info!("Privacy filter engine ready");
                let _ = ready_sender.send(());

                while let Ok(job) = receiver.recv() {
                    let blocked = Request::new(
                        &job.url,
                        &job.source_url,
                        &job.request_type,
                        &job.method,
                    )
                    .map(|request| engine.check_network_request(&request).should_block())
                    .unwrap_or(false);

                    let _ = job.response.send(blocked);
                }
            })
            .expect("could not start Arch Browser privacy filter worker");

        if ready_receiver.recv().is_err() {
            log::error!("Privacy filter worker stopped during initialization");
        }
    });
}

pub fn should_block(url: &str, source_url: &str, request_type: &str, method: &str) -> bool {
    if url.is_empty() || source_url.is_empty() {
        return false;
    }

    start();

    let Some(sender) = FILTER_WORKER.get() else {
        return false;
    };

    let (response_sender, response_receiver) = mpsc::sync_channel(1);
    let job = FilterCheck {
        url: url.to_owned(),
        source_url: source_url.to_owned(),
        request_type: request_type.to_owned(),
        method: method.to_owned(),
        response: response_sender,
    };

    match sender.try_send(job) {
        Ok(()) => response_receiver.recv_timeout(FILTER_TIMEOUT).unwrap_or(false),
        Err(TrySendError::Full(_)) => {
            log::debug!("Privacy filter queue is busy; allowing request");
            false
        }
        Err(TrySendError::Disconnected(_)) => {
            log::warn!("Privacy filter worker is unavailable; allowing request");
            false
        }
    }
}

fn load_engine() -> Engine {
    let mut filter_set = FilterSet::new(false);
    let directory = std::env::var_os("ARCH_BROWSER_FILTERS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(FILTER_DIRECTORY));

    let easylist_loaded = add_filter_file(&mut filter_set, &directory, "easylist.txt");
    let easyprivacy_loaded = add_filter_file(&mut filter_set, &directory, "easyprivacy.txt");

    if !easylist_loaded || !easyprivacy_loaded {
        log::warn!(
            "EasyList/EasyPrivacy files were not both found in {}; using bundled Google tracker rules as a fallback",
            directory.display()
        );
    }

    filter_set.add_filter_list(
        include_str!("privacy/google-trackers.txt").to_owned(),
        ParseOptions::default(),
    );

    Engine::new_with_filter_set(filter_set)
}

fn add_filter_file(filter_set: &mut FilterSet, directory: &Path, file_name: &str) -> bool {
    let path = directory.join(file_name);
    match fs::read_to_string(&path) {
        Ok(contents) if !contents.trim().is_empty() => {
            let rule_count = contents.lines().count();
            filter_set.add_filter_list(contents, ParseOptions::default());
            log::info!("Loaded {file_name} ({rule_count} lines)");
            true
        }
        Ok(_) => {
            log::warn!("Filter list {} is empty", path.display());
            false
        }
        Err(error) => {
            log::warn!("Could not read filter list {}: {error}", path.display());
            false
        }
    }
}

#[cfg(test)]
fn engine_from_text(list_text: &str) -> Engine {
    let mut set = FilterSet::new(false);
    set.add_filter_list(list_text.to_owned(), ParseOptions::default());
    Engine::new_with_filter_set(set)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_google_analytics_as_a_third_party_request() {
        let engine = engine_from_text(include_str!("privacy/google-trackers.txt"));
        let request = Request::new(
            "https://www.google-analytics.com/g/collect?v=2",
            "https://www.google.com/search?q=privacy",
            "xmlhttprequest",
            "POST",
        )
        .expect("valid analytics request");

        assert!(engine.check_network_request(&request).should_block());
    }

    #[test]
    fn blocks_doubleclick_ads_as_a_third_party_request() {
        let engine = engine_from_text(include_str!("privacy/google-trackers.txt"));
        let request = Request::new(
            "https://stats.g.doubleclick.net/g/collect",
            "https://www.google.com/search?q=privacy",
            "ping",
            "GET",
        )
        .expect("valid tracking request");

        assert!(engine.check_network_request(&request).should_block());
    }

    #[test]
    fn keeps_google_search_available() {
        let engine = engine_from_text(include_str!("privacy/google-trackers.txt"));
        let request = Request::new(
            "https://www.google.com/search?q=privacy",
            "https://www.google.com/",
            "document",
            "GET",
        )
        .expect("valid search request");

        assert!(!engine.check_network_request(&request).should_block());
    }
}
