use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub title: String,
    pub url: String,
    pub visited_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct BrowserSettings {
    pub search_engine: String,
    pub restore_tabs: bool,
    pub block_popups: bool,
    pub block_trackers: bool,
    pub animations: bool,
    pub homepage: String,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            search_engine: "https://duckduckgo.com/?q={query}".into(),
            restore_tabs: true,
            block_popups: true,
            block_trackers: true,
            animations: true,
            homepage: "arch://newtab".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct BrowserState {
    pub version: u32,
    pub settings: BrowserSettings,
    pub bookmarks: Vec<Bookmark>,
    pub history: Vec<HistoryEntry>,
    pub restored_tabs: Vec<String>,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            version: STATE_VERSION,
            settings: BrowserSettings::default(),
            bookmarks: Vec::new(),
            history: Vec::new(),
            restored_tabs: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum StateError {
    Io(io::Error),
    Json(serde_json::Error),
    UnsupportedVersion(u32),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "state I/O error: {error}"),
            Self::Json(error) => write!(f, "state file is invalid JSON: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported state version: {version}")
            }
        }
    }
}

impl std::error::Error for StateError {}

impl From<io::Error> for StateError {
    fn from(value: io::Error) -> Self { Self::Io(value) }
}

impl From<serde_json::Error> for StateError {
    fn from(value: serde_json::Error) -> Self { Self::Json(value) }
}

pub struct StateStore {
    path: PathBuf,
}

impl StateStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> Result<BrowserState, StateError> {
        if !self.path.exists() {
            return Ok(BrowserState::default());
        }
        let bytes = fs::read(&self.path)?;
        let state: BrowserState = serde_json::from_slice(&bytes)?;
        if state.version != STATE_VERSION {
            return Err(StateError::UnsupportedVersion(state.version));
        }
        Ok(state)
    }

    pub fn save(&self, state: &BrowserState) -> Result<(), StateError> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let mut state = state.clone();
        state.version = STATE_VERSION;
        let bytes = serde_json::to_vec_pretty(&state)?;
        let temporary = self.path.with_extension("json.tmp");
        let mut file = File::create(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, &self.path)?;
        Ok(())
    }

    pub fn path(&self) -> &Path { &self.path }
}

impl BrowserState {
    pub fn add_bookmark(&mut self, title: impl Into<String>, url: impl Into<String>) {
        let url = url.into();
        if self.bookmarks.iter().any(|item| item.url == url) { return; }
        self.bookmarks.push(Bookmark { title: title.into(), url });
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        self.bookmarks.retain(|item| item.url != url);
    }

    pub fn record_visit(&mut self, title: impl Into<String>, url: impl Into<String>) {
        let url = url.into();
        self.history.retain(|entry| entry.url != url);
        self.history.insert(0, HistoryEntry {
            title: title.into(),
            url,
            visited_at: SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
        });
        self.history.truncate(10_000);
    }

    pub fn export_bookmarks(&self) -> Result<String, StateError> {
        Ok(serde_json::to_string_pretty(&self.bookmarks)?)
    }

    pub fn import_bookmarks(&mut self, json: &str) -> Result<usize, StateError> {
        let imported: Vec<Bookmark> = serde_json::from_str(json)?;
        let mut added = 0;
        for bookmark in imported {
            if !bookmark.url.trim().is_empty()
                && !self.bookmarks.iter().any(|existing| existing.url == bookmark.url)
            {
                self.bookmarks.push(bookmark);
                added += 1;
            }
        }
        Ok(added)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_stable_and_safe() {
        let settings = BrowserSettings::default();
        assert_eq!(settings.homepage, "arch://newtab");
        assert!(settings.block_popups);
        assert!(settings.block_trackers);
    }

    #[test]
    fn bookmarks_deduplicate_and_remove() {
        let mut state = BrowserState::default();
        state.add_bookmark("Example", "https://example.com");
        state.add_bookmark("Duplicate", "https://example.com");
        assert_eq!(state.bookmarks.len(), 1);
        state.remove_bookmark("https://example.com");
        assert!(state.bookmarks.is_empty());
    }

    #[test]
    fn bookmark_import_export_round_trip() {
        let mut state = BrowserState::default();
        state.add_bookmark("Example", "https://example.com");
        let exported = state.export_bookmarks().unwrap();
        let mut restored = BrowserState::default();
        assert_eq!(restored.import_bookmarks(&exported).unwrap(), 1);
        assert_eq!(restored.bookmarks, state.bookmarks);
        assert_eq!(restored.import_bookmarks(&exported).unwrap(), 0);
    }

    #[test]
    fn history_moves_revisited_url_to_front() {
        let mut state = BrowserState::default();
        state.record_visit("First", "https://one.example");
        state.record_visit("Second", "https://two.example");
        state.record_visit("First updated", "https://one.example");
        assert_eq!(state.history.len(), 2);
        assert_eq!(state.history[0].url, "https://one.example");
        assert_eq!(state.history[0].title, "First updated");
    }

    #[test]
    fn store_round_trips_and_rejects_corruption() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state.json");
        let store = StateStore::new(&path);
        let mut state = BrowserState::default();
        state.add_bookmark("Example", "https://example.com");
        store.save(&state).unwrap();
        assert_eq!(store.load().unwrap(), state);

        fs::write(&path, b"{broken").unwrap();
        assert!(matches!(store.load(), Err(StateError::Json(_))));
    }
}
