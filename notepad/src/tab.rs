//! A tab: its place in the title bar (`Header`) and, for a document, its
//! text (`Tab`).

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::document::{Encoding, LineEnding};
use crate::formatted::Formatter;
use crate::markdown;
use crate::session::TabState;

/// A tab in the title bar: its name and a close button, which shows a dot
/// while the tab has unsaved changes, and the close button again when the
/// pointer's over the tab. A pinned tab shows a pin instead, and stays at
/// the start.
pub struct Header {
    pub widget: gtk::Box,
    label: gtk::Label,
    pub close: gtk::Button,
    mark: gtk::Stack,
    modified: Cell<bool>,
    hovered: Cell<bool>,
    pinned: Cell<bool>,
}

impl Header {
    pub fn new(title: &str, icon: Option<&str>) -> Rc<Header> {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        widget.add_css_class("tab");
        if let Some(icon) = icon {
            widget.append(&gtk::Image::builder().icon_name(icon).pixel_size(16).build());
        }
        let label = gtk::Label::builder()
            .label(title)
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .width_chars(12)
            .max_width_chars(22)
            .build();
        widget.append(&label);
        let mark = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::None)
            .build();
        mark.add_named(
            &gtk::Image::builder()
                .icon_name("window-close-symbolic")
                .pixel_size(12)
                .build(),
            Some("close"),
        );
        let dot = gtk::Box::builder()
            .css_classes(["modified-dot"])
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();
        mark.add_named(&dot, Some("dot"));
        mark.add_named(
            &gtk::Image::builder()
                .icon_name("view-pin-symbolic")
                .pixel_size(12)
                .build(),
            Some("pin"),
        );
        let close = gtk::Button::builder()
            .child(&mark)
            .css_classes(["tab-close"])
            .valign(gtk::Align::Center)
            .tooltip_text("Close tab (Ctrl+W)")
            .focus_on_click(false)
            .build();
        widget.append(&close);
        let header = Rc::new(Header {
            widget,
            label,
            close,
            mark,
            modified: Cell::new(false),
            hovered: Cell::new(false),
            pinned: Cell::new(false),
        });
        let motion = gtk::EventControllerMotion::new();
        let weak = Rc::downgrade(&header);
        motion.connect_enter(move |_, _, _| {
            if let Some(header) = weak.upgrade() {
                header.hovered.set(true);
                header.update_mark();
            }
        });
        let weak = Rc::downgrade(&header);
        motion.connect_leave(move |_| {
            if let Some(header) = weak.upgrade() {
                header.hovered.set(false);
                header.update_mark();
            }
        });
        header.widget.add_controller(motion);
        header.update_mark();
        header
    }

    pub fn set_title(&self, title: &str, tooltip: &str) {
        self.label.set_label(title);
        self.widget.set_tooltip_text(Some(tooltip));
    }

    pub fn set_modified(&self, modified: bool) {
        self.modified.set(modified);
        self.update_mark();
    }

    pub fn set_pinned(&self, pinned: bool) {
        self.pinned.set(pinned);
        if pinned {
            self.widget.add_css_class("pinned");
        } else {
            self.widget.remove_css_class("pinned");
        }
        self.update_mark();
    }

    pub fn is_pinned(&self) -> bool {
        self.pinned.get()
    }

    pub fn set_active(&self, active: bool) {
        if active {
            self.widget.add_css_class("active");
        } else {
            self.widget.remove_css_class("active");
        }
    }

    fn update_mark(&self) {
        let (name, tooltip) = if self.pinned.get() && !self.hovered.get() {
            ("pin", "Pinned. Close tab (Ctrl+W)")
        } else if self.modified.get() && !self.hovered.get() {
            ("dot", "Unsaved changes. Close tab (Ctrl+W)")
        } else if self.modified.get() {
            ("close", "Unsaved changes. Close tab (Ctrl+W)")
        } else {
            ("close", "Close tab (Ctrl+W)")
        };
        self.mark.set_visible_child_name(name);
        self.close.set_tooltip_text(Some(tooltip));
    }
}

/// A Markdown tab's preview beside its text: the same document, formatted
/// throughout and read only, scrolling with it.
pub struct Preview {
    pub scroller: gtk::ScrolledWindow,
    buffer: gtk::TextBuffer,
    formatter: Rc<Formatter>,
}

/// A document tab: the text, its file and how the file's written.
pub struct Tab {
    pub id: u64,
    pub buffer: gtk::TextBuffer,
    pub view: gtk::TextView,
    pub scroller: gtk::ScrolledWindow,
    /// What the window shows for the tab: the text, and its preview beside
    /// it when that's on.
    pub widget: gtk::Paned,
    pub header: Rc<Header>,
    pub file: RefCell<Option<gio::File>>,
    pub encoding: Cell<Encoding>,
    pub line_ending: Cell<LineEnding>,
    /// Percent.
    pub zoom: Cell<u32>,
    /// The session's file for this tab's text, once it has one.
    pub session_key: RefCell<Option<String>>,
    /// The text changed since the session last kept it.
    pub session_dirty: Cell<bool>,
    /// The text changed since version history last kept it.
    pub history_dirty: Cell<bool>,
    /// A Markdown file's formatted view.
    pub formatter: Rc<Formatter>,
    /// Whether a Markdown file shows formatted (or as typed).
    pub formatted: Cell<bool>,
    /// Whether a Markdown file shows its preview beside it.
    pub preview_on: Cell<bool>,
    preview: RefCell<Option<Preview>>,
    preview_timer: RefCell<Option<glib::SourceId>>,
    gutter: RefCell<Option<gtk::DrawingArea>>,
    /// The line the cursor's on, shaded when that's on.
    current_line: gtk::TextTag,
    highlight: Cell<bool>,
}

impl Tab {
    pub fn new(id: u64, word_wrap: bool) -> Rc<Tab> {
        let buffer = gtk::TextBuffer::new(None);
        // Made before the formatted view's tags, so theirs win over it
        let current_line = gtk::TextTag::new(Some("current-line"));
        buffer.tag_table().add(&current_line);
        let view = gtk::TextView::builder()
            .buffer(&buffer)
            .accepts_tab(true)
            .left_margin(12)
            .right_margin(12)
            .top_margin(8)
            .bottom_margin(8)
            .css_classes(["editor", format!("np-view-{id}").as_str()])
            .build();
        let scroller = gtk::ScrolledWindow::builder()
            .child(&view)
            .hexpand(true)
            .vexpand(true)
            .build();
        let widget = gtk::Paned::builder()
            .orientation(gtk::Orientation::Horizontal)
            .start_child(&scroller)
            .resize_start_child(true)
            .shrink_start_child(false)
            .resize_end_child(true)
            .shrink_end_child(false)
            .wide_handle(true)
            .hexpand(true)
            .vexpand(true)
            .build();
        let formatter = Formatter::new(&view, &scroller, false);
        let tab = Rc::new(Tab {
            id,
            buffer,
            view,
            scroller,
            widget,
            header: Header::new("Untitled", None),
            file: RefCell::default(),
            encoding: Cell::new(Encoding::Utf8),
            line_ending: Cell::new(LineEnding::Lf),
            zoom: Cell::new(100),
            session_key: RefCell::default(),
            session_dirty: Cell::new(true),
            history_dirty: Cell::new(false),
            formatter,
            formatted: Cell::new(true),
            preview_on: Cell::new(false),
            preview: RefCell::default(),
            preview_timer: RefCell::default(),
            gutter: RefCell::default(),
            current_line,
            highlight: Cell::new(false),
        });
        tab.set_word_wrap(word_wrap);
        tab.set_dark(crate::app::is_dark());
        let weak = Rc::downgrade(&tab);
        tab.buffer.connect_cursor_position_notify(move |_| {
            if let Some(tab) = weak.upgrade() {
                tab.update_current_line();
            }
        });
        let weak = Rc::downgrade(&tab);
        tab.buffer.connect_changed(move |_| {
            if let Some(tab) = weak.upgrade() {
                tab.schedule_preview();
            }
        });
        // The preview scrolls with the text
        let weak = Rc::downgrade(&tab);
        tab.scroller
            .vadjustment()
            .connect_value_changed(move |adjustment| {
                if let Some(tab) = weak.upgrade() {
                    tab.follow_scroll(adjustment);
                }
            });
        tab
    }

    pub fn view_class(&self) -> String {
        format!("np-view-{}", self.id)
    }

    pub fn set_word_wrap(&self, on: bool) {
        let mode = if on {
            gtk::WrapMode::WordChar
        } else {
            gtk::WrapMode::None
        };
        self.view.set_wrap_mode(mode);
    }

    /// The formatting's and the current line's colors, light or dark.
    pub fn set_dark(&self, dark: bool) {
        self.formatter.set_dark(dark);
        if let Some(preview) = self.preview.borrow().as_ref() {
            preview.formatter.set_dark(dark);
        }
        let shade = if dark {
            gdk::RGBA::new(0.30, 0.63, 0.99, 0.09)
        } else {
            gdk::RGBA::new(0.16, 0.55, 0.98, 0.07)
        };
        self.current_line
            .set_paragraph_background_rgba(Some(&shade));
    }

    /// Line numbers at the text's left, or not.
    pub fn set_line_numbers(&self, on: bool) {
        if on == self.gutter.borrow().is_some() {
            return;
        }
        if on {
            let gutter = crate::gutter::new(&self.view, &self.scroller);
            self.view
                .set_gutter(gtk::TextWindowType::Left, Some(&gutter));
            self.gutter.replace(Some(gutter));
        } else {
            self.view
                .set_gutter(gtk::TextWindowType::Left, None::<&gtk::Widget>);
            self.gutter.replace(None);
        }
    }

    /// The cursor's line shaded, or not.
    pub fn set_highlight_line(&self, on: bool) {
        self.highlight.set(on);
        self.update_current_line();
    }

    fn update_current_line(&self) {
        let (start, end) = self.buffer.bounds();
        self.buffer.remove_tag(&self.current_line, &start, &end);
        if !self.highlight.get() {
            return;
        }
        let cursor = self.buffer.iter_at_mark(&self.buffer.get_insert());
        let Some(line_start) = self.buffer.iter_at_line(cursor.line()) else {
            return;
        };
        let mut line_end = line_start;
        line_end.forward_line();
        self.buffer
            .apply_tag(&self.current_line, &line_start, &line_end);
    }

    /// The file's name; for a tab never saved, its first line (or
    /// "Untitled").
    pub fn title(&self) -> String {
        if let Some(name) = self.file.borrow().as_ref().and_then(|file| file.basename()) {
            return name.to_string_lossy().into_owned();
        }
        self.first_line().unwrap_or_else(|| "Untitled".into())
    }

    /// The first line with words on it, cut short, for a new note's name.
    fn first_line(&self) -> Option<String> {
        let mut line = self.buffer.start_iter();
        for _ in 0..20 {
            let mut end = line;
            if !end.ends_line() {
                end.forward_to_line_end();
            }
            let text = self.buffer.text(&line, &end, false);
            let words = text
                .trim()
                .trim_start_matches(['#', '>', '-', '*', ' '])
                .trim();
            if !words.is_empty() {
                let mut name: String = words.chars().take(30).collect();
                if words.chars().count() > 30 {
                    name = format!("{}…", name.trim_end());
                }
                return Some(name);
            }
            if !line.forward_line() {
                break;
            }
        }
        None
    }

    /// A file name to suggest when it's first saved: its first line's words.
    pub fn suggested_name(&self) -> String {
        let name: String = self
            .first_line()
            .unwrap_or_default()
            .trim_end_matches('…')
            .chars()
            .filter(|c| {
                !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
                    && !c.is_control()
            })
            .collect();
        let name = name.trim().trim_end_matches('.');
        if name.is_empty() {
            "Untitled.txt".into()
        } else {
            format!("{name}.txt")
        }
    }

    /// Where the file is, for the tab's tooltip.
    pub fn location(&self) -> String {
        self.file
            .borrow()
            .as_ref()
            .map(|file| file.parse_name().to_string())
            .unwrap_or_else(|| "Not saved yet".into())
    }

    /// A Markdown file, by its name.
    pub fn is_markdown(&self) -> bool {
        self.file
            .borrow()
            .as_ref()
            .and_then(|file| file.basename())
            .is_some_and(|name| markdown::is_markdown_name(&name.to_string_lossy()))
    }

    /// The file's folder, for images and links relative to it.
    pub fn folder(&self) -> Option<PathBuf> {
        self.file
            .borrow()
            .as_ref()
            .and_then(|file| file.parent())
            .and_then(|folder| folder.path())
    }

    pub fn is_modified(&self) -> bool {
        self.buffer.is_modified()
    }

    /// A new tab nobody's typed in yet.
    pub fn is_blank(&self) -> bool {
        self.file.borrow().is_none() && !self.is_modified() && self.buffer.char_count() == 0
    }

    pub fn text(&self) -> String {
        let (start, end) = self.buffer.bounds();
        self.buffer.text(&start, &end, true).to_string()
    }

    /// Puts `text` in the tab as though it had just been opened: no undo
    /// back past it, and no unsaved changes.
    pub fn set_text(&self, text: &str) {
        self.buffer.begin_irreversible_action();
        self.buffer.set_text(text);
        self.buffer.end_irreversible_action();
        self.buffer.place_cursor(&self.buffer.start_iter());
        self.buffer.set_modified(false);
    }

    /// Puts `text` in the tab as an edit Undo takes back (a restored
    /// version).
    pub fn replace_text(&self, text: &str) {
        let (mut start, mut end) = self.buffer.bounds();
        self.buffer.begin_user_action();
        self.buffer.delete(&mut start, &mut end);
        self.buffer.insert(&mut start, text);
        self.buffer.end_user_action();
        self.buffer.place_cursor(&self.buffer.start_iter());
    }

    pub fn cursor_offset(&self) -> i32 {
        self.buffer.iter_at_mark(&self.buffer.get_insert()).offset()
    }

    /// Moves the cursor and brings it into view, once the tab's drawn.
    pub fn place_cursor_at(&self, offset: i32) {
        let iter = self.buffer.iter_at_offset(offset);
        self.buffer.place_cursor(&iter);
        let view = self.view.clone();
        let buffer = self.buffer.clone();
        glib::idle_add_local_once(move || {
            view.scroll_to_mark(&buffer.get_insert(), 0.1, false, 0.0, 0.0);
        });
    }

    /// The preview beside the text, or not.
    pub fn set_preview(&self, on: bool) {
        let showing = self.widget.end_child().is_some();
        if on == showing {
            return;
        }
        if !on {
            self.widget.set_end_child(None::<&gtk::Widget>);
            return;
        }
        if self.preview.borrow().is_none() {
            let buffer = gtk::TextBuffer::new(None);
            let view = gtk::TextView::builder()
                .buffer(&buffer)
                .editable(false)
                .cursor_visible(false)
                .wrap_mode(gtk::WrapMode::WordChar)
                .left_margin(16)
                .right_margin(16)
                .top_margin(8)
                .bottom_margin(8)
                .css_classes(["editor", "preview", &self.view_class()])
                .build();
            let scroller = gtk::ScrolledWindow::builder()
                .child(&view)
                .hexpand(true)
                .vexpand(true)
                .build();
            let formatter = Formatter::new(&view, &scroller, true);
            formatter.set_dark(crate::app::is_dark());
            self.preview.replace(Some(Preview {
                scroller,
                buffer,
                formatter,
            }));
        }
        if let Some(preview) = self.preview.borrow().as_ref() {
            self.widget.set_end_child(Some(&preview.scroller));
        }
        self.update_preview();
        // Half and half, once it's laid out
        let paned = self.widget.clone();
        glib::idle_add_local_once(move || {
            let width = paned.width();
            paned.set_position(if width > 100 { width / 2 } else { 480 });
        });
    }

    fn schedule_preview(&self) {
        if self.widget.end_child().is_none() {
            return;
        }
        if let Some(timer) = self.preview_timer.take() {
            timer.remove();
        }
        let buffer = self.buffer.clone();
        let preview = self
            .preview
            .borrow()
            .as_ref()
            .map(|preview| (preview.buffer.clone(), preview.formatter.clone()));
        let Some((preview_buffer, formatter)) = preview else {
            return;
        };
        let folder = self.folder();
        let timer = glib::timeout_add_local_once(Duration::from_millis(300), move || {
            let (start, end) = buffer.bounds();
            preview_buffer.set_text(&buffer.text(&start, &end, true));
            formatter.set_active(true, folder);
            formatter.refresh();
        });
        self.preview_timer.replace(Some(timer));
    }

    fn update_preview(&self) {
        if let Some(preview) = self.preview.borrow().as_ref() {
            preview.buffer.set_text(&self.text());
            preview.formatter.set_active(true, self.folder());
            preview.formatter.refresh();
        }
    }

    fn follow_scroll(&self, adjustment: &gtk::Adjustment) {
        if self.widget.end_child().is_none() {
            return;
        }
        if let Some(preview) = self.preview.borrow().as_ref() {
            let range = adjustment.upper() - adjustment.page_size();
            let fraction = if range > 0.0 {
                adjustment.value() / range
            } else {
                0.0
            };
            let target = preview.scroller.vadjustment();
            target.set_value(fraction * (target.upper() - target.page_size()).max(0.0));
        }
    }

    /// What the session keeps of this tab. Its text goes in only when it
    /// can't be read from the file again (unsaved changes, or no file yet),
    /// and is written only when it's changed since the last time.
    pub fn state(&self) -> TabState {
        let uri = self
            .file
            .borrow()
            .as_ref()
            .map(|file| file.uri().to_string());
        let modified = self.is_modified();
        let needs_text = modified || (uri.is_none() && self.buffer.char_count() > 0);
        let (key, text) = if needs_text {
            let mut key = self.session_key.borrow_mut();
            let new = key.is_none();
            let key = key
                .get_or_insert_with(|| format!("tab-{}-{}", glib::real_time(), self.id))
                .clone();
            let text = (new || self.session_dirty.get()).then(|| self.text());
            (Some(key), text)
        } else {
            self.session_key.replace(None);
            (None, None)
        };
        TabState {
            uri,
            modified,
            key,
            text,
            encoding: self.encoding.get(),
            line_ending: self.line_ending.get(),
            cursor: self.cursor_offset(),
            zoom: self.zoom.get(),
            formatted: self.formatted.get(),
            preview: self.preview_on.get(),
            pinned: self.header.is_pinned(),
        }
    }

    /// The tab's document in version history: its file's, or its own while
    /// it has none.
    pub fn history_id(&self) -> Option<String> {
        let uri = self
            .file
            .borrow()
            .as_ref()
            .map(|file| file.uri().to_string());
        let key = self.session_key.borrow().clone();
        crate::history::document_id(uri.as_deref(), key.as_deref())
    }
}
