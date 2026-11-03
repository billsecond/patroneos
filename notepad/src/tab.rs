//! A tab: its place in the title bar (`Header`) and, for a document, its
//! text (`Tab`).

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::document::{Encoding, LineEnding};
use crate::formatted::Formatter;
use crate::markdown;
use crate::session::TabState;

/// A tab in the title bar: its name and a close button, which shows a dot
/// while the tab has unsaved changes, and the close button again when the
/// pointer's over the tab.
pub struct Header {
    pub widget: gtk::Box,
    label: gtk::Label,
    pub close: gtk::Button,
    mark: gtk::Stack,
    modified: Cell<bool>,
    hovered: Cell<bool>,
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
            .width_chars(14)
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

    pub fn set_active(&self, active: bool) {
        if active {
            self.widget.add_css_class("active");
        } else {
            self.widget.remove_css_class("active");
        }
    }

    fn update_mark(&self) {
        let dot = self.modified.get() && !self.hovered.get();
        self.mark
            .set_visible_child_name(if dot { "dot" } else { "close" });
        self.close.set_tooltip_text(Some(if self.modified.get() {
            "Unsaved changes. Close tab (Ctrl+W)"
        } else {
            "Close tab (Ctrl+W)"
        }));
    }
}

/// A document tab: the text, its file and how the file's written.
pub struct Tab {
    pub id: u64,
    pub buffer: gtk::TextBuffer,
    pub view: gtk::TextView,
    pub scroller: gtk::ScrolledWindow,
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
    /// A Markdown file's formatted view.
    pub formatter: Rc<Formatter>,
    /// Whether a Markdown file shows formatted (or as typed).
    pub formatted: Cell<bool>,
}

impl Tab {
    pub fn new(id: u64, word_wrap: bool) -> Rc<Tab> {
        let buffer = gtk::TextBuffer::new(None);
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
        let formatter = Formatter::new(&view, &scroller);
        formatter.set_dark(crate::app::is_dark());
        let tab = Rc::new(Tab {
            id,
            buffer,
            view,
            scroller,
            header: Header::new("Untitled", None),
            file: RefCell::default(),
            encoding: Cell::new(Encoding::Utf8),
            line_ending: Cell::new(LineEnding::Lf),
            zoom: Cell::new(100),
            session_key: RefCell::default(),
            session_dirty: Cell::new(true),
            formatter,
            formatted: Cell::new(true),
        });
        tab.set_word_wrap(word_wrap);
        tab
    }

    pub fn view_class(&self) -> String {
        format!("np-view-{}", self.id)
    }

    pub fn set_word_wrap(&self, on: bool) {
        self.view.set_wrap_mode(if on {
            gtk::WrapMode::WordChar
        } else {
            gtk::WrapMode::None
        });
    }

    /// The file's name, or "Untitled".
    pub fn title(&self) -> String {
        self.file
            .borrow()
            .as_ref()
            .and_then(|file| file.basename())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into())
    }

    /// Where the file is, for the tab's tooltip.
    pub fn location(&self) -> String {
        self.file
            .borrow()
            .as_ref()
            .map(|file| file.parse_name().to_string())
            .unwrap_or_else(|| "Untitled".into())
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
        }
    }
}
