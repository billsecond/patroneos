//! A Markdown tab's formatted view: headings, bold, italic, struck-through
//! text, links, code, quotes, lists and tables styled, the markup hidden
//! except on the lines being edited, and checkboxes for tasks, pictures for
//! images and lines for rules laid over the text. None of it changes the
//! text, so what's saved is the Markdown as it was typed. Also the
//! formatting commands, which do change the text, as typing would.
//!
//! Body text and headings are in PatroneOS Sans, code, code blocks and
//! tables in PatroneOS Mono.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, gio, glib, pango};

use crate::markdown::{self, Parsed, Style};

pub const SANS: &str = "PatroneOS Sans";
pub const MONO: &str = "PatroneOS Mono";
/// Bigger documents stay plain: restyling them as you type would lag.
const LIMIT: usize = 1_000_000;
/// How long after typing the view is restyled.
const DELAY: Duration = Duration::from_millis(150);
const HEADING_SCALES: [f64; 6] = [1.9, 1.55, 1.3, 1.15, 1.05, 1.0];
/// The biggest an image is shown, in pixels.
const IMAGE_HEIGHT: i32 = 420;

struct Tags {
    body: gtk::TextTag,
    headings: Vec<gtk::TextTag>,
    quote: gtk::TextTag,
    code_block: gtk::TextTag,
    table: gtk::TextTag,
    table_head: gtk::TextTag,
    strong: gtk::TextTag,
    emphasis: gtk::TextTag,
    strike: gtk::TextTag,
    code: gtk::TextTag,
    link: gtk::TextTag,
    list: gtk::TextTag,
    done: gtk::TextTag,
    mark: gtk::TextTag,
    /// Text kept in its place but not seen: a task's box under its
    /// checkbox, a rule's dashes under its line. It's drawn in the page's
    /// color (GTK still draws text that's fully transparent).
    clear: gtk::TextTag,
    hidden: gtk::TextTag,
}

impl Tags {
    /// Made in order of priority, lowest first: later tags win.
    fn new(buffer: &gtk::TextBuffer) -> Tags {
        let table = buffer.tag_table();
        let add = |tag: gtk::TextTag| {
            table.add(&tag);
            tag
        };
        let body = add(gtk::TextTag::builder().family(SANS).build());
        let quote = add(gtk::TextTag::builder()
            .left_margin(30)
            .style(pango::Style::Italic)
            .build());
        let code_block = add(gtk::TextTag::builder().family(MONO).build());
        let table_tag = add(gtk::TextTag::builder().family(MONO).build());
        let table_head = add(gtk::TextTag::builder().weight(700).build());
        let headings = HEADING_SCALES
            .iter()
            .map(|scale| {
                add(gtk::TextTag::builder()
                    .family(SANS)
                    .weight(700)
                    .scale(*scale)
                    .pixels_above_lines(6)
                    .pixels_below_lines(4)
                    .build())
            })
            .collect();
        Tags {
            body,
            headings,
            quote,
            code_block,
            table: table_tag,
            table_head,
            strong: add(gtk::TextTag::builder().weight(700).build()),
            emphasis: add(gtk::TextTag::builder().style(pango::Style::Italic).build()),
            strike: add(gtk::TextTag::builder().strikethrough(true).build()),
            code: add(gtk::TextTag::builder().family(MONO).build()),
            link: add(gtk::TextTag::builder()
                .underline(pango::Underline::Single)
                .build()),
            list: add(gtk::TextTag::builder().weight(700).build()),
            done: add(gtk::TextTag::builder().strikethrough(true).build()),
            mark: add(gtk::TextTag::builder().build()),
            clear: add(gtk::TextTag::builder().build()),
            hidden: add(gtk::TextTag::builder().invisible(true).build()),
        }
    }

    fn of(&self, style: Style) -> &gtk::TextTag {
        match style {
            Style::Heading(level) => &self.headings[(level.clamp(1, 6) - 1) as usize],
            Style::Strong => &self.strong,
            Style::Emphasis => &self.emphasis,
            Style::Strike => &self.strike,
            Style::Code => &self.code,
            Style::CodeBlock => &self.code_block,
            Style::Link => &self.link,
            Style::Quote => &self.quote,
            Style::ListMarker => &self.list,
            Style::Table => &self.table,
            Style::TableHead => &self.table_head,
            Style::Mark => &self.mark,
            Style::Done => &self.done,
        }
    }

    /// The colors, for a light or dark look.
    fn paint(&self, dark: bool) {
        let rgba = |r: u8, g: u8, b: u8, a: f32| {
            gdk::RGBA::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
        };
        // The page is style.css's --surface
        let (accent, faint, shade, block, page) = if dark {
            (
                rgba(111, 179, 255, 1.0),
                rgba(139, 144, 151, 1.0),
                rgba(255, 255, 255, 0.09),
                rgba(255, 255, 255, 0.05),
                rgba(31, 32, 35, 1.0),
            )
        } else {
            (
                rgba(26, 111, 212, 1.0),
                rgba(130, 136, 145, 1.0),
                rgba(0, 0, 0, 0.06),
                rgba(0, 0, 0, 0.035),
                rgba(255, 255, 255, 1.0),
            )
        };
        self.clear.set_foreground_rgba(Some(&page));
        let quote = gdk::RGBA::new(accent.red(), accent.green(), accent.blue(), 0.08);
        self.link.set_foreground_rgba(Some(&accent));
        self.list.set_foreground_rgba(Some(&accent));
        self.mark.set_foreground_rgba(Some(&faint));
        self.done.set_foreground_rgba(Some(&faint));
        self.quote.set_foreground_rgba(Some(&faint));
        self.quote.set_paragraph_background_rgba(Some(&quote));
        self.code.set_background_rgba(Some(&shade));
        self.code_block.set_paragraph_background_rgba(Some(&block));
        self.table.set_paragraph_background_rgba(Some(&block));
    }
}

pub struct Formatter {
    view: gtk::TextView,
    buffer: gtk::TextBuffer,
    tags: Tags,
    active: Cell<bool>,
    parsed: RefCell<Parsed>,
    /// The lines shown with their markup, the cursor's, once the text's
    /// been clicked or typed in.
    edited: Cell<Option<(usize, usize)>>,
    touched: Cell<bool>,
    /// The document's folder, for images and links given relative to it.
    folder: RefCell<Option<PathBuf>>,
    timer: RefCell<Option<glib::SourceId>>,
    checks: RefCell<Vec<gtk::CheckButton>>,
    rules: RefCell<Vec<gtk::Separator>>,
    /// Each image shown: its picture and the tag that makes room for it.
    pictures: RefCell<Vec<(gtk::Picture, gtk::TextTag)>>,
    images: RefCell<Vec<(usize, gdk::Texture)>>,
    textures: RefCell<HashMap<PathBuf, Option<gdk::Texture>>>,
    /// Set while the view sets its checkboxes, so that isn't a click.
    syncing: Cell<bool>,
    me: Weak<Formatter>,
}

impl Formatter {
    pub fn new(view: &gtk::TextView, scroller: &gtk::ScrolledWindow) -> Rc<Formatter> {
        let buffer = view.buffer();
        let formatter = Rc::new_cyclic(|me| Formatter {
            view: view.clone(),
            tags: Tags::new(&buffer),
            buffer,
            active: Cell::new(false),
            parsed: RefCell::default(),
            edited: Cell::new(None),
            touched: Cell::new(false),
            folder: RefCell::default(),
            timer: RefCell::default(),
            checks: RefCell::default(),
            rules: RefCell::default(),
            pictures: RefCell::default(),
            images: RefCell::default(),
            textures: RefCell::default(),
            syncing: Cell::new(false),
            me: me.clone(),
        });
        formatter.connect(scroller);
        formatter
    }

    fn connect(&self, scroller: &gtk::ScrolledWindow) {
        let on = |run: fn(&Formatter)| {
            let weak = self.me.clone();
            move || {
                if let Some(formatter) = weak.upgrade()
                    && formatter.active.get()
                {
                    run(&formatter);
                }
            }
        };
        let changed = on(|formatter| formatter.schedule(DELAY));
        self.buffer.connect_changed(move |_| changed());
        // The cursor onto another line shows that line's markup
        let moved = on(|formatter| {
            if formatter.touched.get() && formatter.edited_lines() != formatter.edited.get() {
                formatter.schedule(Duration::ZERO);
            }
        });
        self.buffer.connect_mark_set(move |_, _, mark| {
            if matches!(mark.name().as_deref(), Some("insert" | "selection_bound")) {
                moved();
            }
        });
        // A wider or narrower window, or another font or zoom, moves what's
        // laid over the text
        let place = on(|formatter| formatter.place());
        scroller
            .hadjustment()
            .connect_page_size_notify(move |_| place());
        let place = on(|formatter| formatter.place());
        scroller
            .vadjustment()
            .connect_upper_notify(move |_| place());

        // Typing or clicking in the text starts showing the edited line's
        // markup; Ctrl+click opens a link
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = self.me.clone();
        keys.connect_key_pressed(move |_, _, _, _| {
            if let Some(formatter) = weak.upgrade() {
                formatter.touch();
            }
            glib::Propagation::Proceed
        });
        self.view.add_controller(keys);
        let click = gtk::GestureClick::new();
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = self.me.clone();
        click.connect_pressed(move |gesture, _, x, y| {
            let Some(formatter) = weak.upgrade() else {
                return;
            };
            // A task's checkbox: ticked here, so the click doesn't also move
            // the text cursor onto its line
            if let Some(index) = formatter.check_at(x, y) {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                let checked = formatter.checks.borrow()[index].is_active();
                formatter.toggle_task(index, !checked);
                return;
            }
            let ctrl = gesture
                .current_event_state()
                .contains(gdk::ModifierType::CONTROL_MASK);
            if ctrl
                && formatter.active.get()
                && let Some(url) = formatter.link_at_point(x, y)
            {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                formatter.open(&url);
                return;
            }
            formatter.touch();
        });
        self.view.add_controller(click);
        self.view.set_has_tooltip(true);
        let weak = self.me.clone();
        self.view
            .connect_query_tooltip(move |_, x, y, keyboard, tooltip| {
                let Some(formatter) = weak.upgrade() else {
                    return false;
                };
                if keyboard || !formatter.active.get() {
                    return false;
                }
                match formatter.link_at_point(x as f64, y as f64) {
                    Some(url) => {
                        tooltip.set_text(Some(&format!("{url}\nCtrl+click to open")));
                        true
                    }
                    None => false,
                }
            });

        // Copying copies the Markdown, hidden markup and all
        let weak = self.me.clone();
        self.view.connect_copy_clipboard(move |view| {
            if let Some(formatter) = weak.upgrade()
                && formatter.active.get()
                && let Some(text) = formatter.selected_markdown()
            {
                view.clipboard().set_text(&text);
                view.stop_signal_emission_by_name("copy-clipboard");
            }
        });
        let weak = self.me.clone();
        self.view.connect_cut_clipboard(move |view| {
            if let Some(formatter) = weak.upgrade()
                && formatter.active.get()
                && let Some(text) = formatter.selected_markdown()
            {
                view.clipboard().set_text(&text);
                formatter.buffer.delete_selection(true, view.is_editable());
                view.stop_signal_emission_by_name("cut-clipboard");
            }
        });
    }

    fn touch(&self) {
        if !self.touched.replace(true) && self.active.get() {
            self.schedule(Duration::ZERO);
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.get()
    }

    /// Formatted view on or off. `folder` is the document's, for images and
    /// links given relative to it.
    pub fn set_active(&self, active: bool, folder: Option<PathBuf>) {
        if *self.folder.borrow() != folder {
            self.folder.replace(folder);
            self.textures.borrow_mut().clear();
        }
        let was = self.active.replace(active);
        if active {
            if !was {
                self.touched.set(false);
            }
            self.refresh();
        } else if was {
            self.clear();
        }
    }

    pub fn set_dark(&self, dark: bool) {
        self.tags.paint(dark);
    }

    fn schedule(&self, delay: Duration) {
        if let Some(timer) = self.timer.take() {
            timer.remove();
        }
        let weak = self.me.clone();
        let timer = glib::timeout_add_local_once(delay, move || {
            if let Some(formatter) = weak.upgrade() {
                formatter.timer.take();
                formatter.refresh();
            }
        });
        self.timer.replace(Some(timer));
    }

    /// The cursor's lines, once the text's been clicked or typed in.
    fn edited_lines(&self) -> Option<(usize, usize)> {
        if !self.touched.get() {
            return None;
        }
        let insert = self.buffer.iter_at_mark(&self.buffer.get_insert()).line() as usize;
        let bound = self
            .buffer
            .iter_at_mark(&self.buffer.selection_bound())
            .line() as usize;
        Some((insert.min(bound), insert.max(bound)))
    }

    fn iter(&self, offset: usize) -> gtk::TextIter {
        self.buffer.iter_at_offset(offset as i32)
    }

    fn apply(&self, tag: &gtk::TextTag, range: &std::ops::Range<usize>) {
        self.buffer
            .apply_tag(tag, &self.iter(range.start), &self.iter(range.end));
    }

    /// Styles the whole text again, from what's in it now.
    pub fn refresh(&self) {
        if let Some(timer) = self.timer.take() {
            timer.remove();
        }
        if !self.active.get() {
            return;
        }
        let (start, end) = self.buffer.bounds();
        let text = self.buffer.text(&start, &end, true);
        self.buffer.remove_all_tags(&start, &end);
        if text.len() > LIMIT {
            self.parsed.replace(Parsed::default());
            self.images.borrow_mut().clear();
            self.place();
            return;
        }
        let parsed = markdown::parse(&text);
        let edited = self.edited_lines();
        self.edited.set(edited);
        let is_edited =
            |line: usize| edited.is_some_and(|(first, last)| (first..=last).contains(&line));

        self.buffer.apply_tag(&self.tags.body, &start, &end);
        for span in &parsed.spans {
            self.apply(self.tags.of(span.style), &span.range);
        }
        for marker in &parsed.markers {
            let tag = if is_edited(marker.line) {
                &self.tags.mark
            } else {
                &self.tags.hidden
            };
            self.apply(tag, &marker.range);
        }
        for task in &parsed.tasks {
            if !is_edited(task.line) {
                self.apply(&self.tags.clear, &(task.start..task.start + 3));
            }
        }
        for rule in &parsed.rules {
            if !is_edited(rule.line) {
                self.apply(&self.tags.clear, &rule.range);
            }
        }
        let mut images = Vec::new();
        for image in &parsed.images {
            match self.texture(&image.url) {
                Some(texture) => {
                    if !is_edited(image.line) {
                        self.apply(&self.tags.hidden, &image.range);
                    }
                    images.push((image.line, texture));
                }
                None => self.apply(&self.tags.link, &image.range),
            }
        }
        self.images.replace(images);
        self.parsed.replace(parsed);
        self.place();
    }

    /// Takes the formatting off: plain text, nothing laid over it.
    fn clear(&self) {
        if let Some(timer) = self.timer.take() {
            timer.remove();
        }
        let (start, end) = self.buffer.bounds();
        self.buffer.remove_all_tags(&start, &end);
        self.parsed.replace(Parsed::default());
        self.images.borrow_mut().clear();
        self.place();
    }

    fn put(&self, widget: &impl IsA<gtk::Widget>, x: i32, y: i32) {
        if widget.parent().is_none() {
            self.view.add_overlay(widget, x, y);
        } else {
            self.view.move_overlay(widget, x, y);
        }
        widget.set_visible(true);
    }

    /// The width text can use, inside the view's margins.
    fn text_width(&self) -> i32 {
        let width = self.view.width() - self.view.left_margin() - self.view.right_margin();
        if width > 40 { width } else { 600 }
    }

    /// Puts the checkboxes, rules and images where their text is now.
    fn place(&self) {
        let parsed = self.parsed.borrow();
        let edited = self.edited.get();
        let shown = |line: usize| {
            self.active.get() && !edited.is_some_and(|(first, last)| (first..=last).contains(&line))
        };

        let mut checks = self.checks.borrow_mut();
        while checks.len() < parsed.tasks.len() {
            let index = checks.len();
            let check = gtk::CheckButton::builder()
                .css_classes(["md-task"])
                .focus_on_click(false)
                .visible(false)
                .build();
            let weak = self.me.clone();
            check.connect_toggled(move |check| {
                if let Some(formatter) = weak.upgrade()
                    && !formatter.syncing.get()
                {
                    formatter.toggle_task(index, check.is_active());
                }
            });
            checks.push(check);
        }
        for (index, check) in checks.iter().enumerate() {
            match parsed.tasks.get(index) {
                Some(task) if shown(task.line) => {
                    self.syncing.set(true);
                    check.set_active(task.checked);
                    self.syncing.set(false);
                    let left = self.view.iter_location(&self.iter(task.start));
                    let right = self.view.iter_location(&self.iter(task.start + 3));
                    let (_, width, _, _) = check.measure(gtk::Orientation::Horizontal, -1);
                    let (_, height, _, _) = check.measure(gtk::Orientation::Vertical, -1);
                    let x = left.x() + ((right.x() - left.x()) - width) / 2;
                    let y = left.y() + (left.height() - height) / 2;
                    self.put(check, x, y);
                }
                _ => check.set_visible(false),
            }
        }
        drop(checks);

        let mut rules = self.rules.borrow_mut();
        while rules.len() < parsed.rules.len() {
            let rule = gtk::Separator::builder()
                .orientation(gtk::Orientation::Horizontal)
                .css_classes(["md-rule"])
                .visible(false)
                .build();
            rules.push(rule);
        }
        for (index, rule) in rules.iter().enumerate() {
            match parsed.rules.get(index) {
                Some(found) if shown(found.line) => {
                    let place = self.view.iter_location(&self.iter(found.range.start));
                    rule.set_size_request(self.text_width(), 1);
                    self.put(
                        rule,
                        self.view.left_margin(),
                        place.y() + place.height() / 2,
                    );
                }
                _ => rule.set_visible(false),
            }
        }
        drop(rules);

        let images = self.images.borrow();
        let mut pictures = self.pictures.borrow_mut();
        while pictures.len() < images.len() {
            let picture = gtk::Picture::builder()
                .can_shrink(true)
                .content_fit(gtk::ContentFit::Contain)
                .css_classes(["md-image"])
                .visible(false)
                .build();
            let room = gtk::TextTag::new(None);
            self.buffer.tag_table().add(&room);
            pictures.push((picture, room));
        }
        for (index, (picture, room)) in pictures.iter().enumerate() {
            let Some((line, texture)) = images.get(index).filter(|_| self.active.get()) else {
                picture.set_visible(false);
                continue;
            };
            let (tw, th) = (
                texture.width().max(1) as f64,
                texture.height().max(1) as f64,
            );
            let scale = (self.text_width() as f64 / tw)
                .min(IMAGE_HEIGHT as f64 / th)
                .min(1.0);
            let (width, height) = ((tw * scale) as i32, (th * scale) as i32);
            // Room under the image's line, which the picture fills
            if room.pixels_below_lines() != height + 10 {
                room.set_pixels_below_lines(height + 10);
            }
            let Some(start) = self.buffer.iter_at_line(*line as i32) else {
                continue;
            };
            let mut end = start;
            if !end.ends_line() {
                end.forward_to_line_end();
            }
            if !start.has_tag(room) {
                self.buffer.apply_tag(room, &start, &end);
            }
            let bottom = self.view.iter_location(&end);
            picture.set_paintable(Some(texture));
            picture.set_size_request(width, height);
            self.put(
                picture,
                self.view.left_margin(),
                bottom.y() + bottom.height() + 4,
            );
        }
    }

    /// An image, from its address: a file, by its path or relative to the
    /// document. Nothing is fetched from the network.
    fn texture(&self, url: &str) -> Option<gdk::Texture> {
        let path = if url.starts_with("file:") {
            glib::filename_from_uri(url).ok()?.0
        } else if url.contains("://") || url.starts_with("data:") || url.is_empty() {
            return None;
        } else {
            let path = PathBuf::from(glib::Uri::unescape_string(url, None::<&str>)?.as_str());
            if path.is_absolute() {
                path
            } else {
                self.folder.borrow().as_ref()?.join(path)
            }
        };
        self.textures
            .borrow_mut()
            .entry(path.clone())
            .or_insert_with(|| gdk::Texture::from_filename(&path).ok())
            .clone()
    }

    fn toggle_task(&self, index: usize, checked: bool) {
        let Some(task) = self.parsed.borrow().tasks.get(index).cloned() else {
            return;
        };
        let box_text = self
            .buffer
            .text(&self.iter(task.start), &self.iter(task.start + 3), true);
        if !matches!(box_text.as_str(), "[ ]" | "[x]" | "[X]") {
            return;
        }
        let mut start = self.iter(task.start + 1);
        let mut end = self.iter(task.start + 2);
        self.buffer.begin_user_action();
        self.buffer.delete(&mut start, &mut end);
        self.buffer
            .insert(&mut start, if checked { "x" } else { " " });
        self.buffer.end_user_action();
    }

    /// The task checkbox at a point in the view, if there's one showing.
    fn check_at(&self, x: f64, y: f64) -> Option<usize> {
        let picked = self.view.pick(x, y, gtk::PickFlags::DEFAULT)?;
        let check = if picked.is::<gtk::CheckButton>() {
            picked
        } else {
            picked.ancestor(gtk::CheckButton::static_type())?
        };
        self.checks.borrow().iter().position(|candidate| {
            candidate.upcast_ref::<gtk::Widget>() == &check && candidate.is_visible()
        })
    }

    fn link_at_point(&self, x: f64, y: f64) -> Option<String> {
        let (bx, by) =
            self.view
                .window_to_buffer_coords(gtk::TextWindowType::Widget, x as i32, y as i32);
        let offset = self.view.iter_at_location(bx, by)?.offset() as usize;
        self.parsed
            .borrow()
            .links
            .iter()
            .find(|link| link.range.contains(&offset))
            .map(|link| link.url.clone())
    }

    /// Opens a link: a web address in the browser, a file (by its path or
    /// relative to the document) in its app.
    fn open(&self, url: &str) {
        let parent = self.view.root().and_downcast::<gtk::Window>();
        if url.contains("://") || url.starts_with("mailto:") {
            gtk::UriLauncher::new(url).launch(parent.as_ref(), gio::Cancellable::NONE, |_| {});
            return;
        }
        let path = url.split('#').next().unwrap_or_default();
        if path.is_empty() {
            return;
        }
        let path = Path::new(path);
        let path = match self.folder.borrow().as_ref() {
            Some(folder) if !path.is_absolute() => folder.join(path),
            _ => path.to_path_buf(),
        };
        let file = gio::File::for_path(path);
        gtk::FileLauncher::new(Some(&file)).launch(parent.as_ref(), gio::Cancellable::NONE, |_| {});
    }

    fn selected_markdown(&self) -> Option<String> {
        let (start, end) = self.buffer.selection_bounds()?;
        Some(self.buffer.text(&start, &end, true).to_string())
    }
}

// --- The formatting commands -------------------------------------------------

fn text_between(buffer: &gtk::TextBuffer, start: i32, end: i32) -> String {
    let count = buffer.char_count();
    buffer
        .text(
            &buffer.iter_at_offset(start.clamp(0, count)),
            &buffer.iter_at_offset(end.clamp(0, count)),
            true,
        )
        .to_string()
}

/// Whether `text` is wrapped in `marker` (italic's single star, not bold's
/// two).
fn wrapped_in(text: &str, marker: &str) -> bool {
    let length = marker.chars().count();
    text.chars().count() >= 2 * length
        && text.starts_with(marker)
        && text.ends_with(marker)
        && (marker != "*" || text.starts_with("***") || !text.starts_with("**"))
}

/// Bold, italic or strikethrough on the selection, or off when it has it
/// already; with nothing selected, the word at the cursor, or a pair of
/// markers to type between.
pub fn toggle_wrap(buffer: &gtk::TextBuffer, marker: &str) {
    let length = marker.chars().count() as i32;
    let (start, end) = match buffer.selection_bounds() {
        Some((start, end)) => (start.offset(), end.offset()),
        None => {
            let cursor = buffer.iter_at_mark(&buffer.get_insert());
            if cursor.inside_word() || cursor.ends_word() {
                let mut start = cursor;
                if !start.starts_word() {
                    start.backward_word_start();
                }
                let mut end = cursor;
                if !end.ends_word() {
                    end.forward_word_end();
                }
                (start.offset(), end.offset())
            } else {
                buffer.begin_user_action();
                buffer.insert_at_cursor(&format!("{marker}{marker}"));
                buffer.end_user_action();
                let inside = buffer.iter_at_offset(cursor_offset(buffer) - length);
                buffer.place_cursor(&inside);
                return;
            }
        }
    };
    let selected = text_between(buffer, start, end);
    let before = text_between(buffer, start - length, start);
    let after = text_between(buffer, end, end + length);
    let beyond = text_between(buffer, start - length - 1, start - length);
    buffer.begin_user_action();
    let (new_start, new_end) = if wrapped_in(&selected, marker) {
        let mut from = buffer.iter_at_offset(end - length);
        let mut to = buffer.iter_at_offset(end);
        buffer.delete(&mut from, &mut to);
        let mut from = buffer.iter_at_offset(start);
        let mut to = buffer.iter_at_offset(start + length);
        buffer.delete(&mut from, &mut to);
        (start, end - 2 * length)
    } else if before == marker && after == marker && !(marker == "*" && beyond == "*") {
        let mut from = buffer.iter_at_offset(end);
        let mut to = buffer.iter_at_offset(end + length);
        buffer.delete(&mut from, &mut to);
        let mut from = buffer.iter_at_offset(start - length);
        let mut to = buffer.iter_at_offset(start);
        buffer.delete(&mut from, &mut to);
        (start - length, end - length)
    } else {
        buffer.insert(&mut buffer.iter_at_offset(end), marker);
        buffer.insert(&mut buffer.iter_at_offset(start), marker);
        (start + length, end + length)
    };
    buffer.end_user_action();
    buffer.select_range(
        &buffer.iter_at_offset(new_end),
        &buffer.iter_at_offset(new_start),
    );
}

fn cursor_offset(buffer: &gtk::TextBuffer) -> i32 {
    buffer.iter_at_mark(&buffer.get_insert()).offset()
}

/// The lines the selection (or the cursor) is on.
fn selected_lines(buffer: &gtk::TextBuffer) -> std::ops::RangeInclusive<i32> {
    let insert = buffer.iter_at_mark(&buffer.get_insert());
    let bound = buffer.iter_at_mark(&buffer.selection_bound());
    let (first, last) = if insert.offset() <= bound.offset() {
        (insert, bound)
    } else {
        (bound, insert)
    };
    // A selection that ends at the start of a line leaves that line out
    let last_line = if last.starts_line() && last.line() > first.line() {
        last.line() - 1
    } else {
        last.line()
    };
    first.line()..=last_line
}

fn line_text(buffer: &gtk::TextBuffer, line: i32) -> String {
    let Some(start) = buffer.iter_at_line(line) else {
        return String::new();
    };
    let mut end = start;
    if !end.ends_line() {
        end.forward_to_line_end();
    }
    buffer.text(&start, &end, true).to_string()
}

fn replace_line(buffer: &gtk::TextBuffer, line: i32, text: &str) {
    let Some(mut start) = buffer.iter_at_line(line) else {
        return;
    };
    let mut end = start;
    if !end.ends_line() {
        end.forward_to_line_end();
    }
    if buffer.text(&start, &end, true) == text {
        return;
    }
    buffer.delete(&mut start, &mut end);
    buffer.insert(&mut start, text);
}

/// The cursor line's heading level (0 for body text).
pub fn current_heading(buffer: &gtk::TextBuffer) -> u8 {
    let line = buffer.iter_at_mark(&buffer.get_insert()).line();
    markdown::heading_level(&line_text(buffer, line))
}

/// The selected lines as headings of `level`, or body text for 0.
pub fn set_heading(buffer: &gtk::TextBuffer, level: u8) {
    buffer.begin_user_action();
    for line in selected_lines(buffer) {
        let text = line_text(buffer, line);
        replace_line(buffer, line, &markdown::with_heading(&text, level));
    }
    buffer.end_user_action();
}

/// A bulleted or numbered list on the selected lines, or off.
pub fn toggle_list(buffer: &gtk::TextBuffer, numbered: bool) {
    let lines: Vec<i32> = selected_lines(buffer).collect();
    let texts: Vec<String> = lines.iter().map(|&line| line_text(buffer, line)).collect();
    let borrowed: Vec<&str> = texts.iter().map(String::as_str).collect();
    let changed = markdown::toggle_list(&borrowed, numbered);
    buffer.begin_user_action();
    for (line, text) in lines.iter().zip(&changed) {
        replace_line(buffer, *line, text);
    }
    buffer.end_user_action();
}

/// What a new link starts with: the selected text.
pub fn selected_text(buffer: &gtk::TextBuffer) -> String {
    buffer
        .selection_bounds()
        .map(|(start, end)| buffer.text(&start, &end, true).to_string())
        .unwrap_or_default()
}

/// A link in place of the selection, or at the cursor.
pub fn insert_link(buffer: &gtk::TextBuffer, text: &str, url: &str) {
    let text = if text.is_empty() { url } else { text };
    buffer.begin_user_action();
    buffer.delete_selection(true, true);
    buffer.insert_at_cursor(&format!("[{text}]({url})"));
    buffer.end_user_action();
}

/// Takes the Markdown out of the selection (or the cursor's line): bold,
/// italic, strikethrough, code, links, headings, quotes, lists and task
/// boxes, leaving their text.
pub fn clear_formatting(buffer: &gtk::TextBuffer) {
    let (from, to) = match buffer.selection_bounds() {
        Some((start, end)) => (start.offset() as usize, end.offset() as usize),
        None => {
            let line = buffer.iter_at_mark(&buffer.get_insert()).line();
            let start = buffer
                .iter_at_line(line)
                .map(|iter| iter.offset())
                .unwrap_or(0);
            let end = start + line_text(buffer, line).chars().count() as i32;
            (start as usize, end as usize)
        }
    };
    let (start, end) = buffer.bounds();
    let parsed = markdown::parse(&buffer.text(&start, &end, true));
    let mut ranges: Vec<_> = parsed
        .removable
        .into_iter()
        .filter(|range| range.start >= from && range.end <= to)
        .collect();
    ranges.sort_by_key(|range| std::cmp::Reverse(range.start));
    ranges.dedup();
    buffer.begin_user_action();
    for range in ranges {
        let mut start = buffer.iter_at_offset(range.start as i32);
        let mut end = buffer.iter_at_offset(range.end as i32);
        buffer.delete(&mut start, &mut end);
    }
    buffer.end_user_action();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This repository's Markdown files.
    fn markdown_files() -> Vec<PathBuf> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut files = Vec::new();
        let mut folders = vec![root.clone()];
        while let Some(folder) = folders.pop() {
            let Ok(entries) = std::fs::read_dir(&folder) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();
                if path.is_dir() {
                    if !name.starts_with('.')
                        && !["target", "out", "vendor", "node_modules"].contains(&name.as_str())
                    {
                        folders.push(path);
                    }
                } else if markdown::is_markdown_name(&name) {
                    files.push(path);
                }
            }
        }
        files.sort();
        files
    }

    /// Showing a file formatted, editing lines' markup showing and hiding,
    /// and going back to plain, never changes its text: what's saved is the
    /// file's own bytes.
    #[test]
    #[ignore = "needs a display; run with --ignored in a desktop session"]
    fn formatting_keeps_every_byte() {
        gtk::init().expect("a display");
        let files = markdown_files();
        assert!(
            files.len() > 10,
            "found only {} Markdown files",
            files.len()
        );
        for path in files {
            let bytes = std::fs::read(&path).unwrap();
            let decoded = crate::document::decode(&bytes);
            let buffer = gtk::TextBuffer::new(None);
            let view = gtk::TextView::with_buffer(&buffer);
            let scroller = gtk::ScrolledWindow::builder().child(&view).build();
            buffer.begin_irreversible_action();
            buffer.set_text(&decoded.text);
            buffer.end_irreversible_action();
            buffer.set_modified(false);

            let formatter = Formatter::new(&view, &scroller);
            formatter.set_dark(false);
            formatter.set_active(true, path.parent().map(Path::to_path_buf));
            let (start, end) = buffer.bounds();
            let shown = buffer.text(&start, &end, false);
            // With the cursor halfway down, as when editing there
            formatter.touched.set(true);
            buffer.place_cursor(&buffer.iter_at_offset(buffer.char_count() / 2));
            formatter.refresh();
            formatter.set_dark(true);
            formatter.set_active(false, None);
            formatter.set_active(true, path.parent().map(Path::to_path_buf));

            let (start, end) = buffer.bounds();
            let text = buffer.text(&start, &end, true);
            let name = path.display();
            assert!(
                !buffer.is_modified(),
                "{name}: formatting marked it changed"
            );
            assert!(
                !buffer.can_undo(),
                "{name}: formatting left something to undo"
            );
            assert_eq!(text.as_str(), decoded.text, "{name}: its text changed");
            assert_eq!(
                crate::document::encode(&text, decoded.encoding, decoded.line_ending),
                bytes,
                "{name}: saving it would change it"
            );
            if decoded.text.contains("**") || decoded.text.contains("\n#") {
                assert!(
                    shown.len() < decoded.text.len(),
                    "{name}: no markup was hidden"
                );
            }
        }
    }

    #[test]
    fn knows_bold_from_italic() {
        assert!(wrapped_in("**bold**", "**"));
        assert!(wrapped_in("*it*", "*"));
        assert!(!wrapped_in("**bold**", "*"));
        assert!(wrapped_in("***both***", "*"));
        assert!(!wrapped_in("*", "*"));
    }
}
