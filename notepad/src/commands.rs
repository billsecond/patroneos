//! Command search (Ctrl+K): a box at the top of the window where typing a
//! few words ("word wrap", "line endings crlf", "go to") lists the commands
//! they match, with their keys; Enter (or a click) runs the one chosen.

use std::rc::{Rc, Weak};

use gtk::prelude::*;
use gtk::{gdk, glib};

use crate::window::Window;

/// A command: its name, other words it's found by, and the action it runs
/// (with its target, for actions that take one).
struct Command {
    name: &'static str,
    words: &'static str,
    action: &'static str,
    target: Option<&'static str>,
}

const fn command(name: &'static str, words: &'static str, action: &'static str) -> Command {
    Command {
        name,
        words,
        action,
        target: None,
    }
}

const fn with(
    name: &'static str,
    words: &'static str,
    action: &'static str,
    target: &'static str,
) -> Command {
    Command {
        name,
        words,
        action,
        target: Some(target),
    }
}

const COMMANDS: &[Command] = &[
    command("New tab", "create document note", "win.new-tab"),
    command("New window", "create", "win.new-window"),
    command("Open…", "file load", "win.open"),
    command("Save", "file write", "win.save"),
    command("Save as…", "file write rename copy", "win.save-as"),
    command("Save all", "file write every", "win.save-all"),
    command(
        "Version history…",
        "snapshots versions restore older backup",
        "win.history",
    ),
    command("Print…", "printer paper pdf", "win.print"),
    command("Export as HTML…", "markdown web page", "win.export-html"),
    command("Close tab", "", "win.close-tab"),
    command("Close other tabs", "", "win.close-others"),
    command("Pin tab", "unpin keep", "win.pin-tab"),
    command("Close window", "", "win.close-window"),
    command("Exit", "quit", "win.exit"),
    command("Undo", "", "win.undo"),
    command("Redo", "", "win.redo"),
    command("Cut", "clipboard", "win.cut"),
    command("Copy", "clipboard", "win.copy"),
    command("Paste", "clipboard", "win.paste"),
    command("Delete", "remove", "win.delete"),
    command("Find", "search", "win.find"),
    command("Find next", "search", "win.find-next"),
    command("Find previous", "search", "win.find-previous"),
    command("Replace", "search", "win.replace"),
    command("Go to line…", "jump", "win.go-to"),
    command("Select all", "", "win.select-all"),
    command("Time and date", "insert now today", "win.time-date"),
    command("Font…", "typeface size", "win.font"),
    command("Zoom in", "bigger larger", "win.zoom-in"),
    command("Zoom out", "smaller", "win.zoom-out"),
    command("Reset zoom", "100", "win.zoom-reset"),
    command("Status bar", "show hide", "app.status-bar"),
    command("Word wrap", "lines long", "app.word-wrap"),
    command("Line numbers", "gutter show hide", "app.line-numbers"),
    command(
        "Highlight current line",
        "cursor show hide",
        "app.highlight-line",
    ),
    command(
        "Side panel",
        "open tabs recent outline sidebar",
        "app.side-panel",
    ),
    command("Formatted Markdown", "markdown raw as typed", "win.md-view"),
    command(
        "Markdown preview",
        "markdown side by side",
        "win.md-preview",
    ),
    with("Heading 1", "markdown title", "win.md-heading", "1"),
    with("Heading 2", "markdown", "win.md-heading", "2"),
    with("Heading 3", "markdown", "win.md-heading", "3"),
    with(
        "Text (no heading)",
        "markdown body paragraph",
        "win.md-heading",
        "0",
    ),
    command("Bold", "markdown strong", "win.md-bold"),
    command("Italic", "markdown emphasis", "win.md-italic"),
    command("Strikethrough", "markdown", "win.md-strike"),
    command("Bulleted list", "markdown", "win.md-bullets"),
    command("Numbered list", "markdown", "win.md-numbers"),
    command("Link…", "markdown url", "win.md-link"),
    command("Clear formatting", "markdown plain", "win.md-clear"),
    with(
        "Convert line endings to LF",
        "linux unix",
        "win.line-ending",
        "lf",
    ),
    with(
        "Convert line endings to CRLF",
        "dos",
        "win.line-ending",
        "crlf",
    ),
    with("Convert line endings to CR", "", "win.line-ending", "cr"),
    with("Encoding: UTF-8", "unicode", "win.encoding", "utf-8"),
    with(
        "Encoding: UTF-8 with BOM",
        "unicode",
        "win.encoding",
        "utf-8-bom",
    ),
    with("Encoding: UTF-16 LE", "unicode", "win.encoding", "utf-16le"),
    with("Encoding: UTF-16 BE", "unicode", "win.encoding", "utf-16be"),
    with(
        "Encoding: Western (CP1252)",
        "ansi latin",
        "win.encoding",
        "cp1252",
    ),
    command("Settings", "preferences options", "win.settings"),
    command(
        "About and licences",
        "version licenses credits",
        "win.about",
    ),
];

pub struct Commands {
    pub widget: gtk::Box,
    entry: gtk::SearchEntry,
    list: gtk::ListBox,
    window: std::cell::RefCell<Weak<Window>>,
}

/// Whether every word typed is in the command's name or its other words.
fn matches(command: &Command, query: &str) -> bool {
    let haystack = format!("{} {}", command.name, command.words).to_lowercase();
    query
        .split_whitespace()
        .all(|word| haystack.contains(&word.to_lowercase()))
}

impl Commands {
    pub fn new() -> Rc<Commands> {
        let entry = gtk::SearchEntry::builder()
            .placeholder_text("Type a command: word wrap, line endings, go to line…")
            .hexpand(true)
            .build();
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Browse)
            .activate_on_single_click(true)
            .css_classes(["command-list"])
            .build();
        let scroller = gtk::ScrolledWindow::builder()
            .child(&list)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_height(true)
            .max_content_height(360)
            .build();
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .width_request(520)
            .css_classes(["command-panel"])
            .visible(false)
            .build();
        widget.append(&entry);
        widget.append(&scroller);
        let commands = Rc::new(Commands {
            widget,
            entry,
            list,
            window: Default::default(),
        });

        let weak = Rc::downgrade(&commands);
        commands.entry.connect_search_changed(move |_| {
            if let Some(commands) = weak.upgrade() {
                commands.fill();
            }
        });
        let weak = Rc::downgrade(&commands);
        commands.entry.connect_activate(move |_| {
            if let Some(commands) = weak.upgrade()
                && let Some(row) = commands.list.selected_row()
            {
                commands.run(row.index() as usize);
            }
        });
        let weak = Rc::downgrade(&commands);
        commands.entry.connect_stop_search(move |_| {
            if let Some(commands) = weak.upgrade() {
                commands.hide();
            }
        });
        let weak = Rc::downgrade(&commands);
        commands.list.connect_row_activated(move |_, row| {
            if let Some(commands) = weak.upgrade() {
                commands.run(row.index() as usize);
            }
        });
        // Up and Down choose, from the search box; Esc closes
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(&commands);
        keys.connect_key_pressed(move |_, key, _, _| {
            let Some(commands) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            let step = match key {
                gdk::Key::Down => 1,
                gdk::Key::Up => -1,
                gdk::Key::Escape => {
                    commands.hide();
                    return glib::Propagation::Stop;
                }
                _ => return glib::Propagation::Proceed,
            };
            let index = commands
                .list
                .selected_row()
                .map(|row| row.index())
                .unwrap_or(-1)
                + step;
            if let Some(row) = commands.list.row_at_index(index.max(0)) {
                commands.list.select_row(Some(&row));
                // Into view, then typing goes on in the search box
                row.grab_focus();
                commands.entry.grab_focus();
                commands.entry.set_position(-1);
            }
            glib::Propagation::Stop
        });
        commands.widget.add_controller(keys);
        commands
    }

    pub fn attach(&self, window: &Rc<Window>) {
        self.window.replace(Rc::downgrade(window));
    }

    pub fn show(&self) {
        self.entry.set_text("");
        self.fill();
        self.widget.set_visible(true);
        self.entry.grab_focus();
    }

    pub fn hide(&self) {
        if !self.widget.is_visible() {
            return;
        }
        self.widget.set_visible(false);
        if let Some(tab) = self
            .window
            .borrow()
            .upgrade()
            .and_then(|window| window.selected_tab())
        {
            tab.view.grab_focus();
        }
    }

    pub fn is_shown(&self) -> bool {
        self.widget.is_visible()
    }

    /// The commands that can run now and match what's typed.
    fn found(&self) -> Vec<&'static Command> {
        let Some(window) = self.window.borrow().upgrade() else {
            return Vec::new();
        };
        let query = self.entry.text().to_string();
        COMMANDS
            .iter()
            .filter(|command| matches(command, &query) && window.can_run(command.action))
            .collect()
    }

    fn fill(&self) {
        while let Some(row) = self.list.first_child() {
            self.list.remove(&row);
        }
        let app = crate::app::app();
        for command in self.found() {
            let detailed = match command.target {
                Some(target) => gtk::gio::Action::print_detailed_name(
                    command.action,
                    Some(&target.to_variant()),
                )
                .to_string(),
                None => command.action.to_string(),
            };
            let keys = app
                .gtk
                .accels_for_action(&detailed)
                .first()
                .and_then(gtk::accelerator_parse)
                .map(|(key, modifiers)| gtk::accelerator_get_label(key, modifiers).to_string())
                .unwrap_or_default();
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            line.append(
                &gtk::Label::builder()
                    .label(command.name)
                    .xalign(0.0)
                    .hexpand(true)
                    .build(),
            );
            line.append(
                &gtk::Label::builder()
                    .label(keys)
                    .css_classes(["command-keys"])
                    .build(),
            );
            self.list.append(
                &gtk::ListBoxRow::builder()
                    .child(&line)
                    .css_classes(["command-row"])
                    .build(),
            );
        }
        if let Some(first) = self.list.row_at_index(0) {
            self.list.select_row(Some(&first));
        }
    }

    fn run(&self, index: usize) {
        let Some(command) = self.found().get(index).copied() else {
            return;
        };
        let Some(window) = self.window.borrow().upgrade() else {
            return;
        };
        self.hide();
        let target = command.target.map(|target| target.to_variant());
        // The widget's (it finds both win. and app. actions)
        if let Err(error) = WidgetExt::activate_action(&window.gtk, command.action, target.as_ref())
        {
            eprintln!("notepad: {}: {error}", command.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_commands_by_any_of_their_words() {
        let found = |query: &str| -> Vec<&str> {
            COMMANDS
                .iter()
                .filter(|command| matches(command, query))
                .map(|command| command.name)
                .collect()
        };
        assert_eq!(found("word wrap"), ["Word wrap"]);
        assert!(found("line endings").contains(&"Convert line endings to CRLF"));
        assert_eq!(found("go to"), ["Go to line…"]);
        assert_eq!(found("crlf"), ["Convert line endings to CRLF"]);
        assert!(found("versions").contains(&"Version history…"));
        assert!(found("").len() == COMMANDS.len());
    }
}
