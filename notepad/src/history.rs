//! Version history: snapshots of each file's text, and each unsaved tab's,
//! kept on this computer while Restore open tabs (auto-save) is on: one
//! every few minutes while it changes, one each time it's saved, and one
//! of the changes a tab loses when it's closed without saving. Turning
//! auto-save off stops them and deletes what was kept.
//!
//! They're in ~/.local/state/patrone-notepad/history/, a folder for each
//! document (a file's named after its address, an unsaved tab's after its
//! session key) with an `info.ini` (its name and address) and a text file
//! for each snapshot, named after the time it was taken. Each document
//! keeps its newest 50, none older than 30 days, and all of them together
//! stay under 100 MB.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use gtk::glib;

pub const MAX_PER_DOCUMENT: usize = 50;
pub const MAX_AGE: i64 = 30 * 24 * 60 * 60 * 1_000_000;
pub const MAX_TOTAL: u64 = 100 * 1024 * 1024;
/// Texts bigger than this aren't kept.
pub const MAX_SNAPSHOT: usize = 4 * 1024 * 1024;
/// While a document changes, a snapshot at most this often.
pub const EVERY: i64 = 5 * 60 * 1_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    /// When it was taken, in microseconds since 1970.
    pub time: i64,
    pub path: PathBuf,
    pub size: u64,
}

pub fn dir() -> PathBuf {
    glib::user_state_dir()
        .join("patrone-notepad")
        .join("history")
}

/// A document's folder name: its file's address, hashed, or the unsaved
/// tab's session key.
pub fn document_id(uri: Option<&str>, session_key: Option<&str>) -> Option<String> {
    match (uri, session_key) {
        (Some(uri), _) => {
            let hash = glib::compute_checksum_for_string(glib::ChecksumType::Sha256, uri)?;
            Some(format!("file-{}", &hash[..32]))
        }
        (None, Some(key)) => Some(key.to_string()),
        (None, None) => None,
    }
}

/// A document's snapshots, oldest first.
pub fn list(root: &Path, id: &str) -> Vec<Snapshot> {
    let Ok(entries) = fs::read_dir(root.join(id)) else {
        return Vec::new();
    };
    let mut snapshots: Vec<Snapshot> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let time = name.strip_suffix(".txt")?.parse::<i64>().ok()?;
            Some(Snapshot {
                time,
                size: entry.metadata().ok()?.len(),
                path: entry.path(),
            })
        })
        .collect();
    snapshots.sort_by_key(|snapshot| snapshot.time);
    snapshots
}

pub fn read(snapshot: &Snapshot) -> Option<String> {
    fs::read(&snapshot.path)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// Keeps a snapshot of `text`, unless it's the same as the last one, too
/// big, or (without `force`) the last one is younger than `EVERY`. True
/// when it kept one.
pub fn record(
    root: &Path,
    id: &str,
    name: &str,
    uri: Option<&str>,
    text: &str,
    now: i64,
    force: bool,
) -> io::Result<bool> {
    if text.len() > MAX_SNAPSHOT || text.is_empty() {
        return Ok(false);
    }
    let folder = root.join(id);
    let latest = list(root, id).pop();
    if let Some(latest) = &latest {
        if !force && now - latest.time < EVERY {
            return Ok(false);
        }
        if read(latest).as_deref() == Some(text) {
            return Ok(false);
        }
    }
    fs::create_dir_all(&folder)?;
    let info = glib::KeyFile::new();
    info.set_string("document", "name", name);
    if let Some(uri) = uri {
        info.set_string("document", "uri", uri);
    }
    let error = |error: glib::Error| io::Error::other(error.to_string());
    glib::file_set_contents(folder.join("info.ini"), info.to_data().as_bytes()).map_err(error)?;
    // Never two at the same microsecond
    let time = latest.map_or(now, |latest| now.max(latest.time + 1));
    glib::file_set_contents(folder.join(format!("{time}.txt")), text.as_bytes()).map_err(error)?;
    prune(root, now);
    Ok(true)
}

/// Deletes what's past the limits: older than `MAX_AGE`, past the newest
/// `MAX_PER_DOCUMENT` of a document, and the oldest of all while they're
/// over `MAX_TOTAL`.
pub fn prune(root: &Path, now: i64) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut all = Vec::new();
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        let snapshots = list(root, &id);
        let count = snapshots.len();
        for (index, snapshot) in snapshots.into_iter().enumerate() {
            if now - snapshot.time > MAX_AGE || index + MAX_PER_DOCUMENT < count {
                let _ = fs::remove_file(&snapshot.path);
            } else {
                all.push(snapshot);
            }
        }
        if list(root, &id).is_empty() {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
    all.sort_by_key(|snapshot| snapshot.time);
    let mut total: u64 = all.iter().map(|snapshot| snapshot.size).sum();
    for snapshot in all {
        if total <= MAX_TOTAL {
            break;
        }
        let _ = fs::remove_file(&snapshot.path);
        total -= snapshot.size;
    }
}

/// An unsaved tab was saved as a file: its snapshots go with it.
pub fn rename(root: &Path, from: &str, to: &str) {
    if from == to {
        return;
    }
    let source = root.join(from);
    let target = root.join(to);
    if !source.is_dir() {
        return;
    }
    if !target.exists() {
        let _ = fs::rename(&source, &target);
        return;
    }
    for snapshot in list(root, from) {
        if let Some(name) = snapshot.path.file_name() {
            let _ = fs::rename(&snapshot.path, target.join(name));
        }
    }
    let _ = fs::remove_dir_all(source);
}

/// Forgets every snapshot, for when auto-save's turned off.
pub fn clear(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

/// "Today, 14:05", "Yesterday, 09:12", "3 Nov 2026, 17:40".
fn when(time: i64) -> String {
    let Ok(moment) = glib::DateTime::from_unix_local(time / 1_000_000) else {
        return String::new();
    };
    let day = |moment: &glib::DateTime| (moment.year(), moment.day_of_year());
    let today = glib::DateTime::now_local().ok();
    let yesterday = today.as_ref().and_then(|today| today.add_days(-1).ok());
    let date = if today.as_ref().map(day) == Some(day(&moment)) {
        "Today".to_string()
    } else if yesterday.as_ref().map(day) == Some(day(&moment)) {
        "Yesterday".to_string()
    } else {
        moment
            .format("%-d %b %Y")
            .map(|text| text.to_string())
            .unwrap_or_default()
    };
    let clock = moment
        .format("%H:%M")
        .map(|text| text.to_string())
        .unwrap_or_default();
    format!("{date}, {clock}")
}

/// Version history for a tab: its snapshots, newest first, each one's text
/// beside them, and Restore, which puts that text in the tab as an edit
/// (Undo takes it back; the file changes when it's saved).
pub fn show(parent: &gtk::ApplicationWindow, tab: &std::rc::Rc<crate::tab::Tab>) {
    use gtk::prelude::*;

    let dialog = gtk::Window::builder()
        .title(format!("Version history - {}", tab.title()))
        .modal(true)
        .transient_for(parent)
        .default_width(820)
        .default_height(540)
        .build();
    dialog.add_css_class("history-window");
    let snapshots: Vec<Snapshot> = tab
        .history_id()
        .map(|id| list(&dir(), &id))
        .unwrap_or_default()
        .into_iter()
        .rev()
        .collect();
    let kept = crate::app::app().settings().restore_session;

    let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
    content.add_css_class("history-content");
    if snapshots.is_empty() {
        content.append(
            &gtk::Label::builder()
                .label(if kept {
                    "No versions yet. Notepad keeps one every few minutes while the text changes, \
                     and one each time it's saved."
                } else {
                    "Version history is kept while Restore open tabs is on (Settings, Sessions)."
                })
                .wrap(true)
                .xalign(0.0)
                .vexpand(true)
                .valign(gtk::Align::Start)
                .build(),
        );
    }
    let list_box = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Browse)
        .css_classes(["history-list"])
        .build();
    for snapshot in &snapshots {
        let line = gtk::Box::new(gtk::Orientation::Vertical, 2);
        line.append(
            &gtk::Label::builder()
                .label(when(snapshot.time))
                .xalign(0.0)
                .build(),
        );
        line.append(
            &gtk::Label::builder()
                .label(format!("{} KB", snapshot.size.div_ceil(1024)))
                .xalign(0.0)
                .css_classes(["side-detail"])
                .build(),
        );
        list_box.append(&gtk::ListBoxRow::builder().child(&line).build());
    }
    let text = gtk::TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .wrap_mode(gtk::WrapMode::WordChar)
        .left_margin(12)
        .top_margin(8)
        .css_classes(["editor", "history-text", &tab.view_class()])
        .build();
    let restore = gtk::Button::builder()
        .label("Restore this version")
        .css_classes(["suggested-action"])
        .sensitive(false)
        .build();
    let close = gtk::Button::with_label("Close");
    if !snapshots.is_empty() {
        let paned = gtk::Paned::builder()
            .orientation(gtk::Orientation::Horizontal)
            .start_child(
                &gtk::ScrolledWindow::builder()
                    .child(&list_box)
                    .hscrollbar_policy(gtk::PolicyType::Never)
                    .width_request(220)
                    .build(),
            )
            .end_child(
                &gtk::ScrolledWindow::builder()
                    .child(&text)
                    .hexpand(true)
                    .build(),
            )
            .shrink_start_child(false)
            .resize_start_child(false)
            .position(230)
            .vexpand(true)
            .build();
        content.append(&paned);
    }
    let buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .build();
    buttons.append(&restore);
    buttons.append(&close);
    content.append(&buttons);
    dialog.set_child(Some(&content));

    let shown = std::rc::Rc::new(std::cell::RefCell::new(None::<String>));
    let (view, chosen, button) = (text.clone(), shown.clone(), restore.clone());
    let all = snapshots.clone();
    list_box.connect_row_selected(move |_, row| {
        let found = row
            .and_then(|row| all.get(row.index() as usize))
            .and_then(read);
        view.buffer().set_text(found.as_deref().unwrap_or(""));
        button.set_sensitive(found.is_some());
        chosen.replace(found);
    });
    if let Some(first) = list_box.row_at_index(0) {
        list_box.select_row(Some(&first));
    }
    let weak_dialog = dialog.downgrade();
    let weak_tab = std::rc::Rc::downgrade(tab);
    restore.connect_clicked(move |_| {
        if let (Some(text), Some(tab)) = (shown.borrow().clone(), weak_tab.upgrade()) {
            tab.replace_text(&text);
            tab.view.grab_focus();
        }
        if let Some(dialog) = weak_dialog.upgrade() {
            dialog.close();
        }
    });
    let weak_dialog = dialog.downgrade();
    close.connect_clicked(move |_| {
        if let Some(dialog) = weak_dialog.upgrade() {
            dialog.close();
        }
    });
    let keys = gtk::EventControllerKey::new();
    let weak_dialog = dialog.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape
            && let Some(dialog) = weak_dialog.upgrade()
        {
            dialog.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    dialog.add_controller(keys);
    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "patrone-notepad-history-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    const MINUTE: i64 = 60 * 1_000_000;

    #[test]
    fn keeps_a_snapshot_every_few_minutes_and_on_save() {
        let root = scratch("every");
        let start = 1_000 * MINUTE;
        assert!(record(&root, "doc", "a.txt", None, "one", start, false).unwrap());
        // The same text, or too soon after, isn't kept again
        assert!(
            !record(
                &root,
                "doc",
                "a.txt",
                None,
                "one",
                start + 10 * MINUTE,
                false
            )
            .unwrap()
        );
        assert!(!record(&root, "doc", "a.txt", None, "two", start + MINUTE, false).unwrap());
        // Saving keeps one at once
        assert!(record(&root, "doc", "a.txt", None, "two", start + MINUTE, true).unwrap());
        assert!(
            record(
                &root,
                "doc",
                "a.txt",
                None,
                "three",
                start + 7 * MINUTE,
                false
            )
            .unwrap()
        );
        let texts: Vec<String> = list(&root, "doc").iter().filter_map(read).collect();
        assert_eq!(texts, ["one", "two", "three"]);
        clear(&root);
    }

    #[test]
    fn stays_within_its_limits() {
        let root = scratch("limits");
        let start = 100_000 * MINUTE;
        for index in 0..(MAX_PER_DOCUMENT + 5) {
            record(
                &root,
                "doc",
                "a",
                None,
                &format!("v{index}"),
                start + index as i64 * 6 * MINUTE,
                false,
            )
            .unwrap();
        }
        let snapshots = list(&root, "doc");
        assert_eq!(snapshots.len(), MAX_PER_DOCUMENT);
        assert_eq!(read(&snapshots[0]).as_deref(), Some("v5"));
        // Old ones go, and a document with none left goes with them
        record(&root, "old", "b", None, "old text", start, true).unwrap();
        prune(&root, start + MAX_AGE + MINUTE);
        assert!(list(&root, "old").is_empty());
        assert!(!root.join("old").exists());
        assert!(!list(&root, "doc").is_empty());
        clear(&root);
    }

    #[test]
    fn follows_a_tab_to_its_file() {
        let root = scratch("rename");
        record(&root, "tab-1", "Untitled", None, "draft", MINUTE, true).unwrap();
        record(&root, "file-x", "a.txt", None, "older", 0, true).unwrap();
        rename(&root, "tab-1", "file-x");
        let texts: Vec<String> = list(&root, "file-x").iter().filter_map(read).collect();
        assert_eq!(texts, ["older", "draft"]);
        assert!(!root.join("tab-1").exists());
        clear(&root);
    }

    #[test]
    fn names_documents() {
        let file = document_id(Some("file:///home/me/a.txt"), Some("tab-1")).unwrap();
        assert!(file.starts_with("file-") && file.len() == 37);
        assert_eq!(document_id(None, Some("tab-1")).as_deref(), Some("tab-1"));
        assert_eq!(document_id(None, None), None);
    }
}
