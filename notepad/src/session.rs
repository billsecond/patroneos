//! The session: every window's tabs, written while Notepad runs (a moment
//! after each change) and when it closes, so they come back the next time
//! it starts, unsaved changes included, while the setting is on. Nothing is
//! written to the files themselves.
//!
//! It's kept in ~/.local/state/patrone-notepad/session/: session.ini lists
//! the windows and their tabs; the text of a tab with unsaved changes, or
//! one that was never saved, is in a file of its own beside it.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use gtk::glib;

use crate::document::{Encoding, LineEnding};

#[derive(Clone, Debug, PartialEq)]
pub struct TabState {
    /// The tab's file, as a URI (GIO's: local files and network drives).
    pub uri: Option<String>,
    pub modified: bool,
    /// The name of the file that keeps the tab's text, when the text can't
    /// just be read from `uri` again.
    pub key: Option<String>,
    /// The text to keep in `key`'s file: written when it's `Some`, left as
    /// it is when it's `None`. Loading fills it in.
    pub text: Option<String>,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    /// The cursor, in characters from the start.
    pub cursor: i32,
    pub zoom: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowState {
    pub tabs: Vec<TabState>,
    pub current: usize,
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
}

pub fn dir() -> PathBuf {
    glib::user_state_dir()
        .join("patrone-notepad")
        .join("session")
}

const INDEX: &str = "session.ini";

fn text_file(dir: &Path, key: &str) -> PathBuf {
    dir.join(format!("{key}.txt"))
}

/// Writes the session: the tabs' new text first, then the list, then it
/// removes the text no tab needs any more.
pub fn save(dir: &Path, windows: &[WindowState]) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let file = glib::KeyFile::new();
    file.set_integer("session", "version", 1);
    file.set_integer("session", "windows", windows.len() as i32);
    let mut keep = HashSet::new();
    for (w, window) in windows.iter().enumerate() {
        let group = format!("window{w}");
        file.set_integer(&group, "tabs", window.tabs.len() as i32);
        file.set_integer(&group, "current", window.current as i32);
        file.set_integer(&group, "width", window.width);
        file.set_integer(&group, "height", window.height);
        file.set_boolean(&group, "maximized", window.maximized);
        for (t, tab) in window.tabs.iter().enumerate() {
            let group = format!("window{w}-tab{t}");
            if let Some(uri) = &tab.uri {
                file.set_string(&group, "uri", uri);
            }
            file.set_boolean(&group, "modified", tab.modified);
            if let Some(key) = &tab.key {
                if let Some(text) = &tab.text {
                    glib::file_set_contents(text_file(dir, key), text.as_bytes())
                        .map_err(|error| io::Error::other(error.to_string()))?;
                }
                file.set_string(&group, "text", key);
                keep.insert(format!("{key}.txt"));
            }
            file.set_string(&group, "encoding", tab.encoding.key());
            file.set_string(&group, "line-ending", tab.line_ending.key());
            file.set_integer(&group, "cursor", tab.cursor);
            file.set_integer(&group, "zoom", tab.zoom as i32);
        }
    }
    glib::file_set_contents(dir.join(INDEX), file.to_data().as_bytes())
        .map_err(|error| io::Error::other(error.to_string()))?;
    for entry in fs::read_dir(dir)?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".txt") && !keep.contains(&name) {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}

/// The session last written, with each tab's kept text. Empty when there's
/// none.
pub fn load(dir: &Path) -> Vec<WindowState> {
    let file = glib::KeyFile::new();
    if file
        .load_from_file(dir.join(INDEX), glib::KeyFileFlags::NONE)
        .is_err()
    {
        return Vec::new();
    }
    let int = |group: &str, key: &str, default: i32| file.integer(group, key).unwrap_or(default);
    let count = int("session", "windows", 0).max(0);
    let mut windows = Vec::new();
    for w in 0..count {
        let group = format!("window{w}");
        let mut tabs = Vec::new();
        for t in 0..int(&group, "tabs", 0).max(0) {
            let group = format!("window{w}-tab{t}");
            if !file.has_group(&group) {
                continue;
            }
            let uri = file.string(&group, "uri").ok().map(|uri| uri.to_string());
            let key = file.string(&group, "text").ok().map(|key| key.to_string());
            let text = key
                .as_deref()
                .and_then(|key| fs::read(text_file(dir, key)).ok())
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
            tabs.push(TabState {
                uri,
                modified: file.boolean(&group, "modified").unwrap_or(false),
                // A tab whose text went missing reads its file again
                key: text.as_ref().and(key),
                text,
                encoding: file
                    .string(&group, "encoding")
                    .ok()
                    .and_then(|key| Encoding::from_key(&key))
                    .unwrap_or(Encoding::Utf8),
                line_ending: file
                    .string(&group, "line-ending")
                    .ok()
                    .and_then(|key| LineEnding::from_key(&key))
                    .unwrap_or(LineEnding::Lf),
                cursor: int(&group, "cursor", 0).max(0),
                zoom: int(&group, "zoom", 100).clamp(10, 500) as u32,
            });
        }
        if tabs.is_empty() {
            continue;
        }
        windows.push(WindowState {
            current: (int(&group, "current", 0).max(0) as usize).min(tabs.len() - 1),
            tabs,
            width: int(&group, "width", 0),
            height: int(&group, "height", 0),
            maximized: file.boolean(&group, "maximized").unwrap_or(false),
        });
    }
    windows
}

/// Forgets the session, for when the setting's turned off.
pub fn clear(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "patrone-notepad-test-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn tab(uri: Option<&str>, key: Option<&str>, text: Option<&str>) -> TabState {
        TabState {
            uri: uri.map(str::to_owned),
            modified: text.is_some(),
            key: key.map(str::to_owned),
            text: text.map(str::to_owned),
            encoding: Encoding::Utf16Le,
            line_ending: LineEnding::CrLf,
            cursor: 3,
            zoom: 130,
        }
    }

    #[test]
    fn brings_back_what_it_saved() {
        let dir = scratch("round-trip");
        let windows = vec![
            WindowState {
                tabs: vec![
                    tab(Some("file:///home/me/notes.txt"), None, None),
                    tab(None, Some("a"), Some("never saved\nsecond line")),
                    tab(
                        Some("file:///home/me/todo.txt"),
                        Some("b"),
                        Some("changed, not saved"),
                    ),
                ],
                current: 1,
                width: 900,
                height: 600,
                maximized: false,
            },
            WindowState {
                tabs: vec![tab(None, Some("c"), Some(""))],
                current: 0,
                width: 700,
                height: 500,
                maximized: true,
            },
        ];
        save(&dir, &windows).unwrap();
        assert_eq!(load(&dir), windows);
        clear(&dir);
        assert!(load(&dir).is_empty());
    }

    #[test]
    fn keeps_text_it_wasnt_given_again_and_drops_the_rest() {
        let dir = scratch("kept");
        let first = vec![WindowState {
            tabs: vec![
                tab(None, Some("a"), Some("first")),
                tab(None, Some("b"), Some("second")),
            ],
            current: 0,
            width: 1,
            height: 1,
            maximized: false,
        }];
        save(&dir, &first).unwrap();
        // The next save names "a" without its text (unchanged), and "b"'s tab
        // has closed
        let mut next = first.clone();
        next[0].tabs.truncate(1);
        next[0].tabs[0].text = None;
        save(&dir, &next).unwrap();
        let loaded = load(&dir);
        assert_eq!(loaded[0].tabs.len(), 1);
        assert_eq!(loaded[0].tabs[0].text.as_deref(), Some("first"));
        assert!(!dir.join("b.txt").exists());
        clear(&dir);
    }

    #[test]
    fn a_tab_whose_text_is_gone_reads_its_file() {
        let dir = scratch("missing");
        let windows = vec![WindowState {
            tabs: vec![tab(Some("file:///tmp/x.txt"), Some("gone"), Some("text"))],
            current: 0,
            width: 1,
            height: 1,
            maximized: false,
        }];
        save(&dir, &windows).unwrap();
        fs::remove_file(dir.join("gone.txt")).unwrap();
        let loaded = load(&dir);
        assert_eq!(loaded[0].tabs[0].key, None);
        assert_eq!(loaded[0].tabs[0].text, None);
        clear(&dir);
    }
}
