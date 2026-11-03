//! A Notepad window: Patrone's header bar, with the side panel's button,
//! the tabs as pills and a button for a new one, command search and
//! Settings; the File, Edit and View menus in a slim row under it; the side
//! panel (open tabs, recent files, a Markdown file's outline); the text; and
//! a status bar (the cursor's line and column, how many characters, a
//! Markdown file's words and reading time, the zoom, the line endings and
//! the encoding, each of the last three changed from there).
//!
//! Dragging a tab along the title bar reorders the tabs; with only one, it
//! moves the window, like the rest of the title bar. A middle click closes
//! an unpinned tab. Pinned tabs stay at the start, with a pin for their
//! close button. A right click has Pin, Close, Close others and Version
//! history. Settings opens as a tab of its own, at the end.
//!
//! A tab is Markdown or plain text (the status bar's switch, or View):
//! new tabs are Markdown while the Markdown setting's on, and files by
//! their names. A Markdown tab has a formatting bar under the menus (headings, bold,
//! italic, strikethrough, lists, links, clear formatting, its outline and
//! its preview beside it), and shows formatted unless the status bar's
//! switch, or View, says to show it as typed.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use gtk::prelude::*;
use gtk::{gdk, gio, glib, graphene};

use crate::app::{APP_ID, APP_NAME, ZOOM_MAX, ZOOM_MIN, ZOOM_STEP, app};
use crate::commands::Commands;
use crate::dialogs::{self, SaveChoice};
use crate::document::{self, Encoding, LineEnding};
use crate::find;
use crate::formatted;
use crate::history;
use crate::markdown;
use crate::session::{TabState, WindowState};
use crate::settings::Settings;
use crate::settings_page;
use crate::sidepanel::SidePanel;
use crate::tab::{Header, Tab};

#[derive(Clone)]
pub enum Selected {
    Doc(Rc<Tab>),
    Settings,
}

/// What a press on a tab turns into once the pointer moves.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DragMode {
    Undecided,
    Reorder,
    Done,
}

/// How far the pointer moves before a press becomes a drag.
const DRAG_THRESHOLD: f64 = 8.0;

/// The actions that act on a document, which do nothing on Settings.
const DOCUMENT_ACTIONS: [&str; 26] = [
    "save",
    "save-as",
    "print",
    "undo",
    "redo",
    "cut",
    "copy",
    "paste",
    "delete",
    "find",
    "find-next",
    "find-previous",
    "replace",
    "go-to",
    "select-all",
    "time-date",
    "zoom-in",
    "zoom-out",
    "zoom-reset",
    "line-ending",
    "encoding",
    "close-tab",
    "history",
    "pin-tab",
    "close-others",
    "text-mode",
];

/// The actions of a Markdown tab's formatting bar.
const MARKDOWN_ACTIONS: [&str; 12] = [
    "md-preview",
    "outline",
    "md-heading",
    "md-bold",
    "md-italic",
    "md-strike",
    "md-bullets",
    "md-numbers",
    "md-link",
    "md-clear",
    "md-view",
    "export-html",
];

/// The heading list's names, for levels 0 (body text) to 6.
const HEADINGS: [&str; 7] = [
    "Text",
    "Heading 1",
    "Heading 2",
    "Heading 3",
    "Heading 4",
    "Heading 5",
    "Heading 6",
];

struct Status {
    bar: gtk::Box,
    position: gtk::Label,
    characters: gtk::Label,
    /// A Markdown file's words and reading time.
    words: gtk::Label,
    zoom: gtk::Scale,
    zoom_label: gtk::Button,
    /// Set while the bar shows a tab's zoom, so that isn't taken for a
    /// change.
    syncing: Cell<bool>,
    /// The labels of the line endings' and encoding's menu buttons.
    line_ending: gtk::Label,
    encoding: gtk::Label,
    /// The tab's mode, Markdown or Plain text, with a menu to switch it
    /// and to show Markdown formatted or as typed.
    markdown: gtk::MenuButton,
    markdown_label: gtk::Label,
}

impl Status {
    fn new() -> Status {
        let item = |text: &str, tooltip: &str| {
            gtk::Label::builder()
                .label(text)
                .tooltip_text(tooltip)
                .css_classes(["status-item"])
                .build()
        };
        // Plain text like the rest of the bar (a label of its own, so the
        // button draws no arrow), with a menu of the choices
        let menu_button = |model: &gio::Menu, tooltip: &str| {
            let label = gtk::Label::new(None);
            let button = gtk::MenuButton::builder()
                .child(&label)
                .menu_model(model)
                .direction(gtk::ArrowType::Up)
                .tooltip_text(tooltip)
                .css_classes(["status-item", "status-menu"])
                .build();
            (button, label)
        };
        let position = item("Ln 1, Col 1", "Line and column");
        let characters = item("0 characters", "Characters");
        let words = item("", "Words, and how long it takes to read");
        words.set_visible(false);
        let zoom = gtk::Scale::builder()
            .orientation(gtk::Orientation::Horizontal)
            .adjustment(&gtk::Adjustment::new(
                100.0,
                ZOOM_MIN as f64,
                ZOOM_MAX as f64,
                ZOOM_STEP as f64,
                50.0,
                0.0,
            ))
            .draw_value(false)
            .width_request(110)
            .tooltip_text("Zoom")
            .css_classes(["zoom-slider"])
            .build();
        zoom.add_mark(100.0, gtk::PositionType::Bottom, None);
        let zoom_label = gtk::Button::builder()
            .label("100%")
            .action_name("win.zoom-reset")
            .tooltip_text("Reset zoom (Ctrl+0)")
            .css_classes(["status-toggle", "zoom-label"])
            .build();
        let zoom_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        zoom_box.add_css_class("status-item");
        zoom_box.append(&zoom);
        zoom_box.append(&zoom_label);
        let endings = gio::Menu::new();
        for ending in LineEnding::ALL {
            endings.append(
                Some(ending.label()),
                Some(&gio::Action::print_detailed_name(
                    "win.line-ending",
                    Some(&ending.key().to_variant()),
                )),
            );
        }
        let encodings = gio::Menu::new();
        for encoding in Encoding::ALL {
            encodings.append(
                Some(encoding.label()),
                Some(&gio::Action::print_detailed_name(
                    "win.encoding",
                    Some(&encoding.key().to_variant()),
                )),
            );
        }
        let (line_ending_button, line_ending) =
            menu_button(&endings, "Line endings, for when it's saved");
        let (encoding_button, encoding) = menu_button(&encodings, "Encoding, for when it's saved");
        let modes = gio::Menu::new();
        let kinds = gio::Menu::new();
        kinds.append(Some("Plain text"), Some("win.text-mode::plain"));
        kinds.append(Some("Markdown"), Some("win.text-mode::markdown"));
        modes.append_section(None, &kinds);
        let shows = gio::Menu::new();
        shows.append(Some("Show formatted"), Some("win.md-view"));
        shows.append(Some("Preview beside it"), Some("win.md-preview"));
        modes.append_section(None, &shows);
        let (markdown, markdown_label) = menu_button(&modes, "Plain text or Markdown");
        markdown.set_visible(false);
        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        bar.add_css_class("status-bar");
        bar.append(&position);
        bar.append(&characters);
        bar.append(&words);
        bar.append(&gtk::Box::builder().hexpand(true).build());
        bar.append(&markdown);
        bar.append(&zoom_box);
        bar.append(&line_ending_button);
        bar.append(&encoding_button);
        Status {
            bar,
            position,
            characters,
            words,
            zoom,
            zoom_label,
            syncing: Cell::new(false),
            line_ending,
            encoding,
            markdown,
            markdown_label,
        }
    }
}

/// A Markdown tab's formatting bar.
struct FormatBar {
    row: gtk::Box,
    heading: gtk::MenuButton,
}

impl FormatBar {
    fn new() -> FormatBar {
        let headings = gio::Menu::new();
        for (level, name) in HEADINGS.iter().enumerate() {
            let item = gio::MenuItem::new(
                Some(name),
                Some(&gio::Action::print_detailed_name(
                    "win.md-heading",
                    Some(&level.to_string().to_variant()),
                )),
            );
            headings.append_item(&item);
        }
        let heading = gtk::MenuButton::builder()
            .label(HEADINGS[0])
            .menu_model(&headings)
            .tooltip_text("Heading (Ctrl+Alt+1 to 6, Ctrl+Alt+0 for text)")
            .css_classes(["format-heading"])
            .build();
        let button = |icon: &str, tooltip: &str, action: &str| {
            gtk::Button::builder()
                .icon_name(icon)
                .tooltip_text(tooltip)
                .action_name(action)
                .css_classes(["format-button"])
                .build()
        };
        let separator = || {
            gtk::Separator::builder()
                .orientation(gtk::Orientation::Vertical)
                .css_classes(["format-separator"])
                .build()
        };
        let outline_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        outline_box.append(&gtk::Image::from_icon_name("view-list-symbolic"));
        outline_box.append(&gtk::Label::new(Some("Outline")));
        let outline = gtk::Button::builder()
            .child(&outline_box)
            .tooltip_text("The document's headings, in the side panel")
            .action_name("win.outline")
            .css_classes(["format-outline"])
            .build();
        let preview_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        preview_box.append(&gtk::Image::from_icon_name("view-dual-symbolic"));
        preview_box.append(&gtk::Label::new(Some("Preview")));
        let preview = gtk::ToggleButton::builder()
            .child(&preview_box)
            .tooltip_text("The document formatted, beside its Markdown (Ctrl+Alt+P)")
            .action_name("win.md-preview")
            .css_classes(["format-outline"])
            .build();
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        row.add_css_class("format-row");
        row.append(&heading);
        row.append(&separator());
        row.append(&button(
            "format-text-bold-symbolic",
            "Bold (Ctrl+B)",
            "win.md-bold",
        ));
        row.append(&button(
            "format-text-italic-symbolic",
            "Italic (Ctrl+I)",
            "win.md-italic",
        ));
        row.append(&button(
            "format-text-strikethrough-symbolic",
            "Strikethrough (Ctrl+Shift+X)",
            "win.md-strike",
        ));
        row.append(&separator());
        row.append(&button(
            "view-list-bullet-symbolic",
            "Bulleted list (Ctrl+Shift+8)",
            "win.md-bullets",
        ));
        row.append(&button(
            "view-list-ordered-symbolic",
            "Numbered list (Ctrl+Shift+7)",
            "win.md-numbers",
        ));
        row.append(&separator());
        row.append(&button(
            "insert-link-symbolic",
            "Link (Ctrl+Shift+K)",
            "win.md-link",
        ));
        row.append(&button(
            "edit-clear-all-symbolic",
            "Clear formatting (Ctrl+\\)",
            "win.md-clear",
        ));
        row.append(&gtk::Box::builder().hexpand(true).build());
        row.append(&preview);
        row.append(&outline);
        row.set_visible(false);
        FormatBar { row, heading }
    }
}

pub struct Window {
    pub gtk: gtk::ApplicationWindow,
    strip: gtk::Box,
    strip_scroller: gtk::ScrolledWindow,
    stack: gtk::Stack,
    menu_row: gtk::Box,
    format: FormatBar,
    status: Status,
    pub find: Rc<find::Panel>,
    pub side: Rc<SidePanel>,
    commands: Rc<Commands>,
    words_timer: RefCell<Option<glib::SourceId>>,
    tabs: RefCell<Vec<Rc<Tab>>>,
    settings_tab: RefCell<Option<(Rc<Header>, Rc<settings_page::Page>)>>,
    selected: RefCell<Option<Selected>>,
    /// The document tab shown before Settings, to go back to.
    last_doc: RefCell<Weak<Tab>>,
    drag: Cell<DragMode>,
    /// Asking about unsaved changes before the window closes.
    pub closing: Cell<bool>,
    me: Weak<Window>,
}

/// The row of a title bar, named `headerbar` for style sheets, so the title
/// bar style chosen in Patrone Settings (or any GTK theme) draws it as it
/// draws every other app's title bar, active and inactive.
mod header_box {
    use gtk::glib;
    use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct HeaderBox;

    #[glib::object_subclass]
    impl ObjectSubclass for HeaderBox {
        const NAME: &'static str = "NotepadHeaderBox";
        type Type = super::HeaderBox;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("headerbar");
        }
    }

    impl ObjectImpl for HeaderBox {}
    impl WidgetImpl for HeaderBox {}
    impl BoxImpl for HeaderBox {}
}

glib::wrapper! {
    pub struct HeaderBox(ObjectSubclass<header_box::HeaderBox>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

/// GTK's own minimize, maximize and close buttons for one end of the title
/// bar: in the corner the account chose, only those the window can use,
/// and in Patrone Settings' title bar button style.
fn window_controls(side: gtk::PackType) -> gtk::WindowControls {
    let controls = gtk::WindowControls::new(side);
    controls.set_valign(gtk::Align::Center);
    controls
        .bind_property("empty", &controls, "visible")
        .invert_boolean()
        .sync_create()
        .build();
    controls
}

/// A menu item that only shows its keys: the text view and the find box
/// handle them themselves.
fn item_with_keys(label: &str, action: &str, keys: &str) -> gio::MenuItem {
    let item = gio::MenuItem::new(Some(label), Some(action));
    item.set_attribute_value("accel", Some(&keys.to_variant()));
    item
}

fn menus() -> gio::Menu {
    let section = |items: &[(&str, &str)]| {
        let menu = gio::Menu::new();
        for (label, action) in items {
            menu.append(Some(label), Some(action));
        }
        menu
    };
    let file = gio::Menu::new();
    file.append_section(
        None,
        &section(&[
            ("New tab", "win.new-tab"),
            ("New window", "win.new-window"),
            ("Open…", "win.open"),
        ]),
    );
    file.append_section(
        None,
        &section(&[
            ("Save", "win.save"),
            ("Save as…", "win.save-as"),
            ("Save all", "win.save-all"),
            ("Version history…", "win.history"),
        ]),
    );
    file.append_section(
        None,
        &section(&[
            ("Print…", "win.print"),
            ("Export as HTML…", "win.export-html"),
        ]),
    );
    file.append_section(
        None,
        &section(&[
            ("Close tab", "win.close-tab"),
            ("Close window", "win.close-window"),
            ("Exit", "win.exit"),
        ]),
    );

    let edit = gio::Menu::new();
    let undo = gio::Menu::new();
    undo.append_item(&item_with_keys("Undo", "win.undo", "<Control>z"));
    undo.append(Some("Redo"), Some("win.redo"));
    edit.append_section(None, &undo);
    let clipboard = gio::Menu::new();
    clipboard.append_item(&item_with_keys("Cut", "win.cut", "<Control>x"));
    clipboard.append_item(&item_with_keys("Copy", "win.copy", "<Control>c"));
    clipboard.append_item(&item_with_keys("Paste", "win.paste", "<Control>v"));
    clipboard.append_item(&item_with_keys("Delete", "win.delete", "Delete"));
    edit.append_section(None, &clipboard);
    edit.append_section(
        None,
        &section(&[
            ("Find", "win.find"),
            ("Find next", "win.find-next"),
            ("Find previous", "win.find-previous"),
            ("Replace", "win.replace"),
            ("Go to line…", "win.go-to"),
        ]),
    );
    let select = gio::Menu::new();
    select.append_item(&item_with_keys(
        "Select all",
        "win.select-all",
        "<Control>a",
    ));
    select.append(Some("Time and date"), Some("win.time-date"));
    edit.append_section(None, &select);
    edit.append_section(None, &section(&[("Font…", "win.font")]));

    let view = gio::Menu::new();
    let zoom = section(&[
        ("Zoom in", "win.zoom-in"),
        ("Zoom out", "win.zoom-out"),
        ("Reset zoom", "win.zoom-reset"),
    ]);
    view.append_submenu(Some("Zoom"), &zoom);
    view.append_section(
        None,
        &section(&[
            ("Status bar", "app.status-bar"),
            ("Word wrap", "app.word-wrap"),
        ]),
    );
    view.append_section(
        None,
        &section(&[
            ("Side panel", "app.side-panel"),
            ("Line numbers", "app.line-numbers"),
            ("Highlight current line", "app.highlight-line"),
            ("Command search…", "win.commands"),
        ]),
    );
    view.append_section(
        None,
        &section(&[
            ("Plain text", "win.text-mode::plain"),
            ("Markdown", "win.text-mode::markdown"),
        ]),
    );
    view.append_section(
        None,
        &section(&[
            ("Formatted Markdown", "win.md-view"),
            ("Markdown preview", "win.md-preview"),
        ]),
    );

    let bar = gio::Menu::new();
    bar.append_submenu(Some("File"), &file);
    bar.append_submenu(Some("Edit"), &edit);
    bar.append_submenu(Some("View"), &view);
    bar
}

/// "1,234" for 1234.
fn thousands(number: i32) -> String {
    let digits = number.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

impl Window {
    /// A new window with no tabs yet, at `size` (width, height, maximized)
    /// if it's given.
    pub fn new(size: Option<(i32, i32, bool)>) -> Rc<Window> {
        let app = app();
        let gtk_window = gtk::ApplicationWindow::builder()
            .application(&app.gtk)
            .title(APP_NAME)
            .icon_name(APP_ID)
            .default_width(960)
            .default_height(640)
            .build();
        gtk_window.add_css_class("notepad");
        gtk_window.set_size_request(360, 240);
        if let Some((width, height, maximized)) = size {
            if width >= 360 && height >= 240 {
                gtk_window.set_default_size(width, height);
            }
            if maximized {
                gtk_window.maximize();
            }
        }

        // Patrone's header bar: the side panel's button, the tabs, a new tab
        // button, command search and Settings, and the window's buttons in
        // whichever corner the account put them
        let strip = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        strip.add_css_class("tabs");
        let strip_scroller = gtk::ScrolledWindow::builder()
            .child(&strip)
            .hscrollbar_policy(gtk::PolicyType::External)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_width(true)
            .build();
        let new_tab = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("New tab (Ctrl+N)")
            .action_name("win.new-tab")
            .valign(gtk::Align::Center)
            .css_classes(["new-tab"])
            .build();
        let bar: gtk::Box = glib::Object::new::<HeaderBox>().upcast();
        bar.add_css_class("tab-bar");
        bar.append(&window_controls(gtk::PackType::Start));
        bar.append(
            &gtk::ToggleButton::builder()
                .icon_name("sidebar-show-symbolic")
                .tooltip_text("Side panel (F9)")
                .action_name("app.side-panel")
                .valign(gtk::Align::Center)
                .css_classes(["header-button"])
                .build(),
        );
        bar.append(&strip_scroller);
        bar.append(&new_tab);
        bar.append(&gtk::Box::builder().hexpand(true).build());
        bar.append(
            &gtk::Button::builder()
                .icon_name("system-search-symbolic")
                .tooltip_text("Command search (Ctrl+K)")
                .action_name("win.commands")
                .valign(gtk::Align::Center)
                .css_classes(["header-button"])
                .build(),
        );
        bar.append(
            &gtk::Button::builder()
                .icon_name("emblem-system-symbolic")
                .tooltip_text("Settings")
                .action_name("win.settings")
                .valign(gtk::Align::Center)
                .css_classes(["header-button"])
                .build(),
        );
        bar.append(&window_controls(gtk::PackType::End));
        gtk_window.set_titlebar(Some(&gtk::WindowHandle::builder().child(&bar).build()));

        // The menus, in a slim row under it
        let menu_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        menu_row.add_css_class("menu-row");
        menu_row.append(&gtk::PopoverMenuBar::from_model(Some(&menus())));

        let stack = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::None)
            .hexpand(true)
            .vexpand(true)
            .build();
        let find = find::Panel::new();
        let commands = Commands::new();
        let side = SidePanel::new();
        let overlay = gtk::Overlay::builder()
            .child(&stack)
            .vexpand(true)
            .hexpand(true)
            .build();
        overlay.add_overlay(&find.widget);
        overlay.add_overlay(&commands.widget);
        let middle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        middle.append(&side.widget);
        middle.append(&overlay);
        let status = Status::new();
        let format = FormatBar::new();
        let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
        body.append(&menu_row);
        body.append(&format.row);
        body.append(&middle);
        body.append(&status.bar);
        gtk_window.set_child(Some(&body));

        let window = Rc::new_cyclic(|me| Window {
            gtk: gtk_window,
            strip,
            strip_scroller,
            stack,
            menu_row,
            format,
            status,
            find,
            side,
            commands,
            words_timer: RefCell::default(),
            tabs: RefCell::default(),
            settings_tab: RefCell::default(),
            selected: RefCell::default(),
            last_doc: RefCell::default(),
            drag: Cell::new(DragMode::Done),
            closing: Cell::new(false),
            me: me.clone(),
        });
        window.find.attach(&window);
        window.side.attach(&window);
        window.commands.attach(&window);
        window.add_actions();
        window.connect_status();

        // Files dropped on the text open in tabs. Caught on the way down, so
        // the text view doesn't take them as text.
        let drop = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
        drop.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(&window);
        drop.connect_drop(move |_, value, _, _| {
            let (Some(window), Ok(list)) = (weak.upgrade(), value.get::<gdk::FileList>()) else {
                return false;
            };
            window.open_files(list.files());
            true
        });
        overlay.add_controller(drop);

        let weak = Rc::downgrade(&window);
        window.gtk.connect_close_request(move |_| {
            if let Some(window) = weak.upgrade() {
                crate::app::app().close_window(&window);
            }
            glib::Propagation::Stop
        });
        let weak = Rc::downgrade(&window);
        window.gtk.connect_destroy(move |_| {
            if let Some(window) = weak.upgrade() {
                crate::app::app().forget(&window);
            }
        });
        let settings = app.settings().clone();
        window.apply_settings(&settings);
        app.add_window(window.clone());
        window
    }

    fn me(&self) -> Rc<Window> {
        self.me
            .upgrade()
            .expect("a window's actions run while it's open")
    }

    fn add_actions(&self) {
        let simple = |name: &str, run: fn(&Rc<Window>)| {
            let action = gio::SimpleAction::new(name, None);
            let weak = self.me.clone();
            action.connect_activate(move |_, _| {
                if let Some(window) = weak.upgrade() {
                    run(&window);
                }
            });
            self.gtk.add_action(&action);
        };
        simple("new-tab", |window| {
            window.new_tab();
        });
        simple("new-window", |_| {
            let window = Window::new(None);
            window.new_tab();
            window.gtk.present();
        });
        simple("open", |window| window.open_dialog());
        simple("save", |window| window.save_selected(false));
        simple("save-as", |window| window.save_selected(true));
        simple("save-all", |window| window.save_all());
        simple("print", |window| {
            if let Some(tab) = window.selected_tab() {
                crate::print::print(
                    &window.gtk,
                    &tab.title(),
                    &tab.text(),
                    &app().font_description(),
                );
            }
        });
        simple("close-tab", |window| match window.selected() {
            Some(Selected::Doc(tab)) => window.close_tab(&tab),
            Some(Selected::Settings) => window.close_settings(),
            None => {}
        });
        simple("close-window", |window| app().close_window(window));
        simple("exit", |_| app().exit());
        simple("undo", |window| {
            if let Some(tab) = window.selected_tab() {
                tab.buffer.undo();
            }
        });
        simple("redo", |window| {
            if let Some(tab) = window.selected_tab() {
                tab.buffer.redo();
            }
        });
        // Through the text view's own signals, so a formatted Markdown tab
        // copies its Markdown (formatted.rs)
        simple("cut", |window| {
            if let Some(tab) = window.selected_tab() {
                tab.view.emit_cut_clipboard();
            }
        });
        simple("copy", |window| {
            if let Some(tab) = window.selected_tab() {
                tab.view.emit_copy_clipboard();
            }
        });
        simple("paste", |window| {
            if let Some(tab) = window.selected_tab() {
                tab.buffer
                    .paste_clipboard(&tab.view.clipboard(), None, true);
                tab.view.grab_focus();
            }
        });
        simple("delete", |window| {
            if let Some(tab) = window.selected_tab() {
                tab.buffer.delete_selection(true, true);
            }
        });
        simple("select-all", |window| {
            if let Some(tab) = window.selected_tab() {
                let (start, end) = tab.buffer.bounds();
                tab.buffer.select_range(&start, &end);
                tab.view.grab_focus();
            }
        });
        simple("find", |window| window.find.show(false));
        simple("replace", |window| window.find.show(true));
        simple("find-next", |window| window.find.find_again(true));
        simple("find-previous", |window| window.find.find_again(false));
        simple("go-to", |window| {
            if let Some(tab) = window.selected_tab() {
                dialogs::go_to_line(&window.gtk, &tab);
            }
        });
        simple("time-date", |window| {
            if let Some(tab) = window.selected_tab() {
                let now = glib::DateTime::now_local()
                    .and_then(|now| now.format("%-I:%M %p %-m/%-d/%Y"))
                    .map(|text| text.to_string())
                    .unwrap_or_default();
                tab.buffer.begin_user_action();
                tab.buffer.delete_selection(true, true);
                tab.buffer.insert_interactive_at_cursor(&now, true);
                tab.buffer.end_user_action();
                tab.view.grab_focus();
            }
        });
        simple("font", |window| window.show_settings());
        simple("about", |window| crate::about::show(&window.gtk));
        simple("settings", |window| window.show_settings());
        simple("zoom-in", |window| window.zoom(Some(ZOOM_STEP as i32)));
        simple("zoom-out", |window| window.zoom(Some(-(ZOOM_STEP as i32))));
        simple("zoom-reset", |window| window.zoom(None));
        simple("next-tab", |window| window.cycle(true));
        simple("previous-tab", |window| window.cycle(false));

        // Markdown's formatting bar
        simple("md-bold", |window| {
            window.format_text(|buffer| formatted::toggle_wrap(buffer, "**"))
        });
        simple("md-italic", |window| {
            window.format_text(|buffer| formatted::toggle_wrap(buffer, "*"))
        });
        simple("md-strike", |window| {
            window.format_text(|buffer| formatted::toggle_wrap(buffer, "~~"))
        });
        simple("md-bullets", |window| {
            window.format_text(|buffer| formatted::toggle_list(buffer, false))
        });
        simple("md-numbers", |window| {
            window.format_text(|buffer| formatted::toggle_list(buffer, true))
        });
        simple("md-clear", |window| {
            window.format_text(formatted::clear_formatting)
        });
        simple("md-link", |window| window.ask_link());
        simple("outline", |window| window.side.show_outline());
        simple("commands", |window| {
            if window.commands.is_shown() {
                window.commands.hide();
            } else {
                window.commands.show();
            }
        });
        simple("history", |window| {
            if let Some(tab) = window.selected_tab() {
                // What's there now is a version too
                window.keep_version(&tab, true);
                history::show(&window.gtk, &tab);
            }
        });
        simple("pin-tab", |window| {
            if let Some(tab) = window.selected_tab() {
                window.set_pinned(&tab, !tab.header.is_pinned());
            }
        });
        simple("close-others", |window| {
            if let Some(tab) = window.selected_tab() {
                window.close_others(&tab);
            }
        });
        simple("export-html", |window| window.export_html());
        let heading = gio::SimpleAction::new("md-heading", Some(glib::VariantTy::STRING));
        let weak = self.me.clone();
        heading.connect_activate(move |_, parameter| {
            let level = parameter
                .and_then(|parameter| parameter.get::<String>())
                .and_then(|level| level.parse::<u8>().ok())
                .unwrap_or(0)
                .min(6);
            if let Some(window) = weak.upgrade() {
                window.format_text(|buffer| formatted::set_heading(buffer, level));
            }
        });
        self.gtk.add_action(&heading);
        let mode = gio::SimpleAction::new_stateful(
            "text-mode",
            Some(glib::VariantTy::STRING),
            &"plain".to_variant(),
        );
        let weak = self.me.clone();
        mode.connect_activate(move |_, parameter| {
            let Some(window) = weak.upgrade() else { return };
            let Some(tab) = window.selected_tab() else {
                return;
            };
            let markdown = parameter.and_then(|p| p.get::<String>()).as_deref() == Some("markdown");
            if markdown != tab.markdown.get() {
                tab.markdown.set(markdown);
                window.update_markdown(&tab);
                window.update_words();
                window.side.refresh();
                crate::app::app().schedule_session_save();
            }
        });
        self.gtk.add_action(&mode);
        let preview = gio::SimpleAction::new_stateful("md-preview", None, &false.to_variant());
        let weak = self.me.clone();
        preview.connect_activate(move |_, _| {
            let Some(window) = weak.upgrade() else { return };
            let Some(tab) = window.selected_tab() else {
                return;
            };
            tab.preview_on.set(!tab.preview_on.get());
            window.update_markdown(&tab);
            crate::app::app().schedule_session_save();
        });
        self.gtk.add_action(&preview);
        let view = gio::SimpleAction::new_stateful("md-view", None, &true.to_variant());
        let weak = self.me.clone();
        view.connect_activate(move |_, _| {
            let Some(window) = weak.upgrade() else { return };
            let Some(tab) = window.selected_tab() else {
                return;
            };
            tab.formatted.set(!tab.formatted.get());
            window.update_markdown(&tab);
            crate::app::app().schedule_session_save();
        });
        self.gtk.add_action(&view);

        // How the selected tab's file is written, from the status bar
        for name in ["line-ending", "encoding"] {
            let action = gio::SimpleAction::new_stateful(
                name,
                Some(glib::VariantTy::STRING),
                &"".to_variant(),
            );
            let weak = self.me.clone();
            action.connect_activate(move |action, parameter| {
                let (Some(window), Some(key)) =
                    (weak.upgrade(), parameter.and_then(|p| p.get::<String>()))
                else {
                    return;
                };
                let Some(tab) = window.selected_tab() else {
                    return;
                };
                let changed = if name == "line-ending" {
                    LineEnding::from_key(&key)
                        .filter(|ending| *ending != tab.line_ending.get())
                        .map(|ending| tab.line_ending.set(ending))
                        .is_some()
                } else {
                    Encoding::from_key(&key)
                        .filter(|encoding| *encoding != tab.encoding.get())
                        .map(|encoding| tab.encoding.set(encoding))
                        .is_some()
                };
                if changed {
                    action.set_state(&key.to_variant());
                    // It's a change to the file, saved with the rest
                    tab.buffer.set_modified(true);
                    window.update_status();
                }
            });
            self.gtk.add_action(&action);
        }
    }

    fn action(&self, name: &str) -> Option<gio::SimpleAction> {
        self.gtk
            .lookup_action(name)
            .and_downcast::<gio::SimpleAction>()
    }

    pub fn tabs(&self) -> Vec<Rc<Tab>> {
        self.tabs.borrow().clone()
    }

    pub fn selected(&self) -> Option<Selected> {
        self.selected.borrow().clone()
    }

    /// The document tab showing, if it isn't Settings.
    pub fn selected_tab(&self) -> Option<Rc<Tab>> {
        match self.selected() {
            Some(Selected::Doc(tab)) => Some(tab),
            _ => None,
        }
    }

    pub fn tab_with_file(&self, file: &gio::File) -> Option<Rc<Tab>> {
        self.tabs().into_iter().find(|tab| {
            tab.file
                .borrow()
                .as_ref()
                .is_some_and(|open| open.equal(file))
        })
    }

    /// Settings changed: the text wraps or doesn't, the status bar shows or
    /// doesn't, Settings' page shows the new choices.
    pub fn apply_settings(&self, settings: &Settings) {
        self.side.set_shown(settings.side_panel);
        for tab in self.tabs() {
            tab.set_word_wrap(settings.word_wrap);
            tab.set_line_numbers(settings.line_numbers);
            tab.set_highlight_line(settings.highlight_line);
            tab.set_preview(settings.markdown && tab.is_markdown() && tab.preview_on.get());
            let formatted = settings.markdown && tab.is_markdown() && tab.formatted.get();
            if formatted != tab.formatter.is_active() {
                tab.formatter.set_active(formatted, tab.folder());
            }
        }
        self.sync_markdown(settings);
        self.status.bar.set_visible(
            settings.status_bar && !matches!(self.selected(), Some(Selected::Settings)),
        );
        if let Some((_, page)) = self.settings_tab.borrow().as_ref() {
            page.sync(settings);
        }
        self.find.sync(settings);
    }

    /// A new, empty tab after the others, shown.
    pub fn new_tab(&self) -> Rc<Tab> {
        let tab = self.add_tab();
        self.select(Selected::Doc(tab.clone()));
        tab
    }

    /// Adds an empty tab after the others, without showing it.
    fn add_tab(&self) -> Rc<Tab> {
        let app = app();
        let tab = Tab::new(app.next_id(), app.settings().word_wrap);
        self.connect_tab(&tab);
        let after = self
            .tabs
            .borrow()
            .last()
            .map(|tab| tab.header.widget.clone());
        self.strip
            .insert_child_after(&tab.header.widget, after.as_ref());
        self.stack.add_child(&tab.widget);
        {
            let settings = app.settings();
            tab.set_line_numbers(settings.line_numbers);
            tab.set_highlight_line(settings.highlight_line);
        }
        self.tabs.borrow_mut().push(tab.clone());
        app.refresh_fonts();
        // A new tab starts in Markdown while the setting's on: its formatted
        // view is on from the first letter typed
        self.update_markdown(&tab);
        tab
    }

    /// A tab of the last session.
    pub fn restore_tab(&self, state: &TabState) {
        let file = state.uri.as_deref().map(gio::File::for_uri);
        let (text, encoding, line_ending, modified) = match (&state.text, &file) {
            (Some(text), _) => (
                text.clone(),
                state.encoding,
                state.line_ending,
                state.modified,
            ),
            // No unsaved changes: the file as it is now
            (None, Some(file)) => match file.load_contents(gio::Cancellable::NONE) {
                Ok((bytes, _)) => {
                    let decoded = document::decode(&bytes);
                    (decoded.text, decoded.encoding, decoded.line_ending, false)
                }
                Err(_) => return,
            },
            (None, None) => (String::new(), state.encoding, state.line_ending, false),
        };
        let tab = self.add_tab();
        tab.file.replace(file);
        tab.set_text(&text);
        tab.buffer.set_modified(modified);
        tab.encoding.set(encoding);
        tab.line_ending.set(line_ending);
        tab.zoom.set(state.zoom.clamp(ZOOM_MIN, ZOOM_MAX));
        tab.markdown.set(state.markdown);
        tab.formatted.set(state.formatted);
        tab.preview_on.set(state.preview);
        if state.pinned {
            self.set_pinned(&tab, true);
        }
        tab.session_key.replace(state.key.clone());
        tab.session_dirty.set(false);
        tab.place_cursor_at(state.cursor.min(tab.buffer.char_count()));
        self.update_header(&tab);
        self.update_markdown(&tab);
        app().refresh_fonts();
    }

    pub fn select_index(&self, index: usize) {
        let tab = self.tabs.borrow().get(index).cloned();
        if let Some(tab) = tab.or_else(|| self.tabs.borrow().last().cloned()) {
            self.select(Selected::Doc(tab));
        }
    }

    /// What the session keeps of this window (nothing once its tabs are
    /// all closed).
    pub fn state(&self) -> Option<WindowState> {
        let tabs = self.tabs();
        if tabs.is_empty() {
            return None;
        }
        let selected = self
            .selected_tab()
            .or_else(|| self.last_doc.borrow().upgrade());
        let current = selected
            .and_then(|selected| tabs.iter().position(|tab| Rc::ptr_eq(tab, &selected)))
            .unwrap_or(0);
        let (width, height) = self.gtk.default_size();
        Some(WindowState {
            tabs: tabs.iter().map(|tab| tab.state()).collect(),
            current,
            width,
            height,
            maximized: self.gtk.is_maximized(),
        })
    }

    fn connect_tab(&self, tab: &Rc<Tab>) {
        self.connect_header(&tab.header, Some(tab));
        let weak_window = self.me.clone();
        let weak_tab = Rc::downgrade(tab);
        tab.header.close.connect_clicked(move |_| {
            if let (Some(window), Some(tab)) = (weak_window.upgrade(), weak_tab.upgrade()) {
                window.close_tab(&tab);
            }
        });

        // Keeping the status bar, the tab's name and the session up to date
        let on_tab = |run: fn(&Window, &Rc<Tab>)| {
            let weak_window = self.me.clone();
            let weak_tab = Rc::downgrade(tab);
            move || {
                if let (Some(window), Some(tab)) = (weak_window.upgrade(), weak_tab.upgrade()) {
                    run(&window, &tab);
                }
            }
        };
        let changed = on_tab(|window, tab| {
            tab.session_dirty.set(true);
            tab.history_dirty.set(true);
            // A new note's name is its first line
            if tab.file.borrow().is_none() {
                window.update_header(tab);
            }
            if window.is_selected(tab) {
                window.update_status();
                window.schedule_words();
                window.side.text_changed();
            }
            app().schedule_session_save();
        });
        tab.buffer.connect_changed(move |_| changed());
        let modified = on_tab(|window, tab| {
            window.update_header(tab);
            app().schedule_session_save();
        });
        tab.buffer.connect_modified_changed(move |_| modified());
        let moved = on_tab(|window, tab| {
            if window.is_selected(tab) {
                window.update_status();
            }
        });
        tab.buffer.connect_cursor_position_notify(move |_| moved());
        let marked = on_tab(|window, tab| {
            if window.is_selected(tab) {
                window.update_status();
                window.sync_actions();
            }
        });
        tab.buffer.connect_mark_set(move |_, _, mark| {
            if matches!(mark.name().as_deref(), Some("insert" | "selection_bound")) {
                marked();
            }
        });
        for property in ["has-selection", "can-undo", "can-redo"] {
            let sync = on_tab(|window, tab| {
                if window.is_selected(tab) {
                    window.sync_actions();
                }
            });
            tab.buffer
                .connect_notify_local(Some(property), move |_, _| sync());
        }

        // Ctrl and the mouse wheel zoom
        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = self.me.clone();
        scroll.connect_scroll(move |controller, _, dy| {
            let ctrl = controller
                .current_event_state()
                .contains(gdk::ModifierType::CONTROL_MASK);
            match weak.upgrade() {
                Some(window) if ctrl && dy != 0.0 => {
                    let step = if dy < 0.0 {
                        ZOOM_STEP as i32
                    } else {
                        -(ZOOM_STEP as i32)
                    };
                    window.zoom(Some(step));
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        tab.view.add_controller(scroll);
    }

    /// A tab's press, middle click and drag.
    fn connect_header(&self, header: &Rc<Header>, tab: Option<&Rc<Tab>>) {
        let widget = header.widget.clone();
        let selection = || {
            let weak_window = self.me.clone();
            let weak_tab = tab.map(Rc::downgrade);
            move || -> Option<(Rc<Window>, Selected)> {
                let window = weak_window.upgrade()?;
                let selected = match &weak_tab {
                    Some(tab) => Selected::Doc(tab.upgrade()?),
                    None => Selected::Settings,
                };
                Some((window, selected))
            }
        };

        let click = gtk::GestureClick::new();
        click.set_button(gdk::BUTTON_MIDDLE);
        let target = selection();
        click.connect_pressed(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            match target() {
                // A pinned tab stays
                Some((window, Selected::Doc(tab))) if !tab.header.is_pinned() => {
                    window.close_tab(&tab)
                }
                Some((window, Selected::Settings)) => window.close_settings(),
                _ => {}
            }
        });
        widget.add_controller(click);

        // A right click: the tab's menu
        if tab.is_some() {
            let menu = gio::Menu::new();
            let first = gio::Menu::new();
            first.append(Some("Pin or unpin tab"), Some("win.pin-tab"));
            first.append(Some("Version history…"), Some("win.history"));
            menu.append_section(None, &first);
            let second = gio::Menu::new();
            second.append(Some("Close tab"), Some("win.close-tab"));
            second.append(Some("Close other tabs"), Some("win.close-others"));
            menu.append_section(None, &second);
            let popover = gtk::PopoverMenu::from_model(Some(&menu));
            popover.set_parent(&widget);
            popover.set_has_arrow(false);
            let right = gtk::GestureClick::new();
            right.set_button(gdk::BUTTON_SECONDARY);
            let target = selection();
            right.connect_pressed(move |gesture, _, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some((window, selected)) = target() {
                    window.select(selected);
                    popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                    popover.popup();
                }
            });
            widget.add_controller(right);
        }

        let drag = gtk::GestureDrag::new();
        drag.set_button(gdk::BUTTON_PRIMARY);
        let target = selection();
        let tab_widget = widget.clone();
        drag.connect_drag_begin(move |_, x, y| {
            let Some((window, selected)) = target() else {
                return;
            };
            // A press on the close button is the button's
            let on_button = tab_widget
                .pick(x, y, gtk::PickFlags::DEFAULT)
                .is_some_and(|picked| picked.ancestor(gtk::Button::static_type()).is_some());
            if on_button {
                window.drag.set(DragMode::Done);
                return;
            }
            window.drag.set(DragMode::Undecided);
            window.select(selected);
        });
        let target = selection();
        let tab_widget = widget.clone();
        drag.connect_drag_update(move |gesture, dx, dy| {
            let Some((window, selected)) = target() else {
                return;
            };
            match window.drag.get() {
                DragMode::Done => {}
                DragMode::Undecided => {
                    if dx.hypot(dy) < DRAG_THRESHOLD {
                        return;
                    }
                    // Claimed only now, so clicks still reach the close
                    // button, and before the title bar moves the window
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    match selected {
                        Selected::Doc(tab) if window.tabs.borrow().len() > 1 => {
                            window.drag.set(DragMode::Reorder);
                            window.reorder(gesture, &tab, dx);
                        }
                        _ => window.move_window(gesture, &tab_widget),
                    }
                }
                DragMode::Reorder => {
                    if let Selected::Doc(tab) = selected {
                        window.reorder(gesture, &tab, dx);
                    }
                }
            }
        });
        let weak = self.me.clone();
        drag.connect_drag_end(move |_, _, _| {
            if let Some(window) = weak.upgrade() {
                window.drag.set(DragMode::Done);
            }
        });
        widget.add_controller(drag);
    }

    /// With one tab, dragging it moves the window, like the title bar.
    fn move_window(&self, gesture: &gtk::GestureDrag, widget: &gtk::Box) {
        self.drag.set(DragMode::Done);
        let (Some((x, y)), Some(device), Some(native)) =
            (gesture.start_point(), gesture.device(), widget.native())
        else {
            return;
        };
        let Some(point) = widget.compute_point(&native, &graphene::Point::new(x as f32, y as f32))
        else {
            return;
        };
        let (tx, ty) = native.surface_transform();
        if let Some(toplevel) = native.surface().and_downcast::<gdk::Toplevel>() {
            toplevel.begin_move(
                &device,
                gesture.current_button() as i32,
                point.x() as f64 + tx,
                point.y() as f64 + ty,
                gesture.current_event_time(),
            );
        }
        gesture.reset();
    }

    /// Moves the dragged tab past the neighbors whose middle it's crossed.
    fn reorder(&self, gesture: &gtk::GestureDrag, tab: &Rc<Tab>, dx: f64) {
        let Some((start_x, _)) = gesture.start_point() else {
            return;
        };
        let tabs = self.tabs();
        let Some(from) = tabs.iter().position(|other| Rc::ptr_eq(other, tab)) else {
            return;
        };
        // The offset is in the tab's own coordinates, and the tab moves as
        // it's reordered, so this is where the pointer is over it now
        let Some(point) = tab.header.widget.compute_point(
            &self.strip,
            &graphene::Point::new((start_x + dx) as f32, 0.0),
        ) else {
            return;
        };
        let x = point.x() as f64;
        let mut to = from;
        for (index, other) in tabs.iter().enumerate() {
            let Some(bounds) = other.header.widget.compute_bounds(&self.strip) else {
                continue;
            };
            let middle = (bounds.x() + bounds.width() / 2.0) as f64;
            if index > from && x > middle {
                to = index;
            }
            if index < from && x < middle && to == from {
                to = index;
            }
        }
        if to == from {
            return;
        }
        let mut tabs = self.tabs.borrow_mut();
        let moved = tabs.remove(from);
        tabs.insert(to, moved);
        let after = (to > 0).then(|| tabs[to - 1].header.widget.clone());
        self.strip
            .reorder_child_after(&tabs[to].header.widget, after.as_ref());
        drop(tabs);
        app().schedule_session_save();
    }

    fn is_selected(&self, tab: &Rc<Tab>) -> bool {
        matches!(&*self.selected.borrow(), Some(Selected::Doc(selected)) if Rc::ptr_eq(selected, tab))
    }

    /// Shows a tab (or Settings).
    pub fn select(&self, selected: Selected) {
        for tab in self.tabs() {
            tab.header
                .set_active(matches!(&selected, Selected::Doc(other) if Rc::ptr_eq(other, &tab)));
        }
        if let Some((header, _)) = self.settings_tab.borrow().as_ref() {
            header.set_active(matches!(selected, Selected::Settings));
        }
        let status_bar = app().settings().status_bar;
        let header = match &selected {
            Selected::Doc(tab) => {
                self.stack.set_visible_child(&tab.widget);
                self.menu_row.set_visible(true);
                self.status.bar.set_visible(status_bar);
                self.last_doc.replace(Rc::downgrade(tab));
                tab.header.widget.clone()
            }
            Selected::Settings => {
                self.stack.set_visible_child_name("settings");
                self.menu_row.set_visible(false);
                self.status.bar.set_visible(false);
                self.find.hide();
                match self.settings_tab.borrow().as_ref() {
                    Some((header, _)) => header.widget.clone(),
                    None => return,
                }
            }
        };
        self.selected.replace(Some(selected.clone()));
        if let Selected::Doc(tab) = &selected
            && !self.find.has_focus()
        {
            tab.view.grab_focus();
        }
        self.find.selection_changed();
        self.update_title();
        self.update_status();
        self.sync_actions();
        self.sync_markdown(&app().settings());
        self.update_words();
        self.side.refresh();
        self.scroll_to(&header);
        app().schedule_session_save();
    }

    /// Scrolls the tabs, once they're laid out, so `header` shows.
    fn scroll_to(&self, header: &gtk::Box) {
        let strip = self.strip.clone();
        let scroller = self.strip_scroller.clone();
        let header = header.clone();
        glib::idle_add_local_once(move || {
            let Some(bounds) = header.compute_bounds(&strip) else {
                return;
            };
            let adjustment = scroller.hadjustment();
            let (left, right) = (bounds.x() as f64, (bounds.x() + bounds.width()) as f64);
            if left < adjustment.value() {
                adjustment.set_value(left);
            } else if right > adjustment.value() + adjustment.page_size() {
                adjustment.set_value(right - adjustment.page_size());
            }
        });
    }

    fn cycle(&self, forward: bool) {
        let mut order: Vec<Selected> = self.tabs().into_iter().map(Selected::Doc).collect();
        if self.settings_tab.borrow().is_some() {
            order.push(Selected::Settings);
        }
        let current = match self.selected() {
            Some(Selected::Doc(tab)) => order
                .iter()
                .position(|item| matches!(item, Selected::Doc(other) if Rc::ptr_eq(other, &tab))),
            Some(Selected::Settings) => Some(order.len() - 1),
            None => None,
        };
        let Some(current) = current else { return };
        let next = if forward {
            (current + 1) % order.len()
        } else {
            (current + order.len() - 1) % order.len()
        };
        self.select(order[next].clone());
    }

    fn update_header(&self, tab: &Rc<Tab>) {
        tab.header.set_title(&tab.title(), &tab.location());
        tab.header.set_modified(tab.is_modified());
        if self.is_selected(tab) {
            self.update_title();
        }
        self.side.tabs_changed();
    }

    fn update_title(&self) {
        let title = match self.selected() {
            Some(Selected::Doc(tab)) => {
                format!(
                    "{}{} - {APP_NAME}",
                    if tab.is_modified() { "*" } else { "" },
                    tab.title()
                )
            }
            Some(Selected::Settings) => format!("Settings - {APP_NAME}"),
            None => APP_NAME.into(),
        };
        self.gtk.set_title(Some(&title));
    }

    fn update_status(&self) {
        let Some(tab) = self.selected_tab() else {
            return;
        };
        let buffer = &tab.buffer;
        let cursor = buffer.iter_at_mark(&buffer.get_insert());
        self.status.position.set_label(&format!(
            "Ln {}, Col {}",
            thousands(cursor.line() + 1),
            thousands(cursor.line_offset() + 1)
        ));
        let total = buffer.char_count();
        let noun = |count: i32| {
            if count == 1 {
                "character"
            } else {
                "characters"
            }
        };
        let characters = match buffer.selection_bounds() {
            Some((start, end)) => {
                let selected = end.offset() - start.offset();
                format!(
                    "{} of {} {}",
                    thousands(selected),
                    thousands(total),
                    noun(total)
                )
            }
            None => format!("{} {}", thousands(total), noun(total)),
        };
        self.status.characters.set_label(&characters);
        self.status.syncing.set(true);
        self.status.zoom.set_value(tab.zoom.get() as f64);
        self.status.syncing.set(false);
        self.status
            .zoom_label
            .set_label(&format!("{}%", tab.zoom.get()));
        self.status
            .line_ending
            .set_label(tab.line_ending.get().label());
        self.status.encoding.set_label(tab.encoding.get().label());
        if let Some(action) = self.action("line-ending") {
            action.set_state(&tab.line_ending.get().key().to_variant());
        }
        if let Some(action) = self.action("encoding") {
            action.set_state(&tab.encoding.get().key().to_variant());
        }
        if self.format.row.is_visible() {
            let level = formatted::current_heading(buffer) as usize;
            self.format.heading.set_label(HEADINGS[level.min(6)]);
        }
    }

    /// Greys out what can't be done now.
    fn sync_actions(&self) {
        let tab = self.selected_tab();
        for name in DOCUMENT_ACTIONS {
            if let Some(action) = self.action(name) {
                action.set_enabled(tab.is_some() || name == "close-tab");
            }
        }
        if let Some(tab) = tab {
            let selection = tab.buffer.has_selection();
            for (name, enabled) in [
                ("undo", tab.buffer.can_undo()),
                ("redo", tab.buffer.can_redo()),
                ("cut", selection),
                ("copy", selection),
                ("delete", selection),
            ] {
                if let Some(action) = self.action(name) {
                    action.set_enabled(enabled);
                }
            }
        }
    }

    /// The status bar's zoom slider zooms the tab.
    fn connect_status(&self) {
        let weak = self.me.clone();
        self.status.zoom.connect_value_changed(move |scale| {
            let Some(window) = weak.upgrade() else { return };
            if window.status.syncing.get() {
                return;
            }
            let Some(tab) = window.selected_tab() else {
                return;
            };
            let value = ((scale.value() / ZOOM_STEP as f64).round() as u32 * ZOOM_STEP)
                .clamp(ZOOM_MIN, ZOOM_MAX);
            if value != tab.zoom.get() {
                window.zoom(Some(value as i32 - tab.zoom.get() as i32));
            }
        });
    }

    /// A Markdown file's words and reading time, a moment after it changes.
    fn schedule_words(&self) {
        if !self.status.words.is_visible() {
            return;
        }
        if let Some(timer) = self.words_timer.take() {
            timer.remove();
        }
        let weak = self.me.clone();
        let timer =
            glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                if let Some(window) = weak.upgrade() {
                    window.words_timer.take();
                    window.update_words();
                }
            });
        self.words_timer.replace(Some(timer));
    }

    fn update_words(&self) {
        let tab = self
            .selected_tab()
            .filter(|tab| tab.is_markdown() && app().settings().markdown);
        self.status.words.set_visible(tab.is_some());
        let Some(tab) = tab else { return };
        let words = tab.text().split_whitespace().count();
        // About 200 words a minute
        let minutes = words.div_ceil(200).max(1);
        let noun = if words == 1 { "word" } else { "words" };
        self.status.words.set_label(&format!(
            "{} {noun} · {minutes} min read",
            thousands(words as i32)
        ));
    }

    /// Keeps a version of the tab's text in version history, while
    /// auto-save is on: now (`force`), or if it's been a few minutes.
    pub fn keep_version(&self, tab: &Rc<Tab>, force: bool) {
        if !app().settings().restore_session || (!force && !tab.history_dirty.get()) {
            return;
        }
        // An unsaved tab's document is named after its session key
        if tab.file.borrow().is_none() && tab.session_key.borrow().is_none() {
            tab.session_key
                .replace(Some(format!("tab-{}-{}", glib::real_time(), tab.id)));
        }
        let Some(id) = tab.history_id() else { return };
        let uri = tab
            .file
            .borrow()
            .as_ref()
            .map(|file| file.uri().to_string());
        match history::record(
            &history::dir(),
            &id,
            &tab.title(),
            uri.as_deref(),
            &tab.text(),
            glib::real_time(),
            force,
        ) {
            Ok(_) => tab.history_dirty.set(false),
            Err(error) => eprintln!(
                "notepad: couldn't keep a version of {}: {error}",
                tab.title()
            ),
        }
    }

    /// Pins a tab (it moves to the end of the pinned ones, at the start) or
    /// unpins it (it moves to just after them).
    pub fn set_pinned(&self, tab: &Rc<Tab>, pinned: bool) {
        tab.header.set_pinned(pinned);
        let mut tabs = self.tabs.borrow_mut();
        let Some(from) = tabs.iter().position(|other| Rc::ptr_eq(other, tab)) else {
            return;
        };
        let moved = tabs.remove(from);
        let to = tabs
            .iter()
            .take_while(|other| other.header.is_pinned())
            .count();
        tabs.insert(to, moved);
        let after = (to > 0).then(|| tabs[to - 1].header.widget.clone());
        self.strip
            .reorder_child_after(&tabs[to].header.widget, after.as_ref());
        drop(tabs);
        self.side.tabs_changed();
        app().schedule_session_save();
    }

    /// Closes every tab but this one and the pinned ones, asking about each
    /// one's unsaved changes.
    fn close_others(&self, keep: &Rc<Tab>) {
        let window = self.me();
        let keep = keep.clone();
        glib::spawn_future_local(async move {
            for tab in window.tabs() {
                if Rc::ptr_eq(&tab, &keep) || tab.header.is_pinned() {
                    continue;
                }
                if tab.is_modified() && !window.confirm(&tab).await {
                    return;
                }
                window.remove_tab(&tab);
            }
            window.select(Selected::Doc(keep));
        });
    }

    /// Whether a command can run now: its action exists and is enabled.
    pub fn can_run(&self, detailed: &str) -> bool {
        let (group, name) = detailed.split_once('.').unwrap_or(("win", detailed));
        let action = match group {
            "app" => app().gtk.lookup_action(name),
            _ => self.gtk.lookup_action(name),
        };
        action.is_some_and(|action| action.is_enabled())
    }

    fn zoom(&self, step: Option<i32>) {
        let Some(tab) = self.selected_tab() else {
            return;
        };
        let zoom = match step {
            Some(step) => {
                (tab.zoom.get() as i32 + step).clamp(ZOOM_MIN as i32, ZOOM_MAX as i32) as u32
            }
            None => 100,
        };
        tab.zoom.set(zoom);
        app().refresh_fonts();
        self.update_status();
        app().schedule_session_save();
    }

    /// Settings, as a tab at the end; opened the first time it's asked for.
    pub fn show_settings(&self) {
        if self.settings_tab.borrow().is_none() {
            let header = Header::new("Settings", Some("emblem-system-symbolic"));
            self.connect_header(&header, None);
            let weak = self.me.clone();
            header.close.connect_clicked(move |_| {
                if let Some(window) = weak.upgrade() {
                    window.close_settings();
                }
            });
            let page = settings_page::Page::new();
            page.sync(&app().settings());
            self.strip.append(&header.widget);
            self.stack.add_named(&page.widget, Some("settings"));
            self.settings_tab.replace(Some((header, page)));
        }
        self.select(Selected::Settings);
    }

    pub fn close_settings(&self) {
        let Some((header, page)) = self.settings_tab.take() else {
            return;
        };
        self.strip.remove(&header.widget);
        self.stack.remove(&page.widget);
        if matches!(self.selected(), Some(Selected::Settings)) {
            let back = self
                .last_doc
                .borrow()
                .upgrade()
                .or_else(|| self.tabs.borrow().last().cloned());
            match back {
                Some(tab) => self.select(Selected::Doc(tab)),
                None => self.gtk.close(),
            }
        }
    }

    /// Closes a tab, asking first about unsaved changes. The last tab's
    /// window closes with it.
    pub fn close_tab(&self, tab: &Rc<Tab>) {
        let window = self.me();
        let tab = tab.clone();
        glib::spawn_future_local(async move {
            if tab.is_modified() && !window.confirm(&tab).await {
                return;
            }
            window.remove_tab(&tab);
        });
    }

    fn remove_tab(&self, tab: &Rc<Tab>) {
        let Some(index) = self
            .tabs
            .borrow()
            .iter()
            .position(|other| Rc::ptr_eq(other, tab))
        else {
            return;
        };
        self.tabs.borrow_mut().remove(index);
        self.strip.remove(&tab.header.widget);
        self.stack.remove(&tab.widget);
        self.side.tabs_changed();
        let app = app();
        if self.tabs.borrow().is_empty() {
            // The session goes on without this window; with no other
            // window, it's empty
            if app.windows().len() <= 1 {
                app.save_session();
            }
            self.gtk.destroy();
            return;
        }
        if self.is_selected(tab) || self.selected().is_none() {
            let next = self.tabs.borrow()[index.min(self.tabs.borrow().len() - 1)].clone();
            self.select(Selected::Doc(next));
        }
        app.refresh_fonts();
        app.schedule_session_save();
    }

    /// Asks whether to save a tab's changes, and saves them if so. False
    /// when the question was cancelled (or the save was).
    async fn confirm(&self, tab: &Rc<Tab>) -> bool {
        self.select(Selected::Doc(tab.clone()));
        match dialogs::ask_to_save(&self.gtk, &tab.title()).await {
            SaveChoice::Save => self.save_tab(tab, false).await,
            SaveChoice::DontSave => {
                // In version history, for a change of mind
                self.keep_version(tab, true);
                true
            }
            SaveChoice::Cancel => false,
        }
    }

    /// Asks about each tab with unsaved changes, in turn. False when one
    /// of the questions was cancelled.
    pub async fn confirm_close_all(&self) -> bool {
        for tab in self.tabs() {
            if tab.is_modified() && !self.confirm(&tab).await {
                return false;
            }
        }
        true
    }

    pub fn open_files(&self, files: Vec<gio::File>) {
        let window = self.me();
        glib::spawn_future_local(async move {
            for file in files {
                window.open_file(file).await;
            }
            if window.tabs.borrow().is_empty() {
                window.new_tab();
            }
        });
    }

    /// Opens a file in a tab, or shows the tab that has it open already. A
    /// file that isn't there yet opens empty, and saving it makes it.
    async fn open_file(&self, file: gio::File) {
        if let Some((window, tab)) = app().find_open(&file) {
            window.select(Selected::Doc(tab));
            window.gtk.present();
            return;
        }
        let name = file
            .basename()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| file.parse_name().to_string());
        let decoded = match file.load_contents_future().await {
            Ok((bytes, _)) => document::decode(&bytes),
            Err(error) if error.matches(gio::IOErrorEnum::NotFound) => document::decode(b""),
            Err(error) => {
                dialogs::error(&self.gtk, &format!("Couldn't open {name}"), error.message()).await;
                return;
            }
        };
        // Opened over an empty new tab, which isn't kept
        let tab = match self.selected_tab().filter(|tab| tab.is_blank()) {
            Some(tab) => tab,
            None => self.add_tab(),
        };
        gtk::RecentManager::default().add_item(&file.uri());
        tab.file.replace(Some(file));
        tab.set_text(&decoded.text);
        tab.encoding.set(decoded.encoding);
        tab.line_ending.set(decoded.line_ending);
        tab.markdown.set(tab.has_markdown_name());
        tab.session_dirty.set(true);
        self.update_header(&tab);
        self.update_markdown(&tab);
        self.select(Selected::Doc(tab));
    }

    fn file_filters(dialog: &gtk::FileDialog) {
        let text = gtk::FileFilter::new();
        text.set_name(Some("Text files"));
        text.add_mime_type("text/plain");
        let all = gtk::FileFilter::new();
        all.set_name(Some("All files"));
        all.add_pattern("*");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&text);
        filters.append(&all);
        dialog.set_filters(Some(&filters));
        dialog.set_default_filter(Some(&text));
    }

    /// The folder for a file dialog: the open file's, or Documents.
    fn dialog_folder(&self) -> Option<gio::File> {
        self.selected_tab()
            .and_then(|tab| tab.file.borrow().as_ref().and_then(|file| file.parent()))
            .or_else(|| {
                glib::user_special_dir(glib::UserDirectory::Documents).map(gio::File::for_path)
            })
    }

    fn open_dialog(&self) {
        let window = self.me();
        glib::spawn_future_local(async move {
            let dialog = gtk::FileDialog::builder().title("Open").modal(true).build();
            Window::file_filters(&dialog);
            if let Some(folder) = window.dialog_folder() {
                dialog.set_initial_folder(Some(&folder));
            }
            let Ok(chosen) = dialog.open_multiple_future(Some(&window.gtk)).await else {
                return;
            };
            let files = chosen
                .iter::<gio::File>()
                .filter_map(Result::ok)
                .collect::<Vec<_>>();
            window.open_files(files);
        });
    }

    fn save_selected(&self, save_as: bool) {
        let Some(tab) = self.selected_tab() else {
            return;
        };
        // Nothing to save: the file stays exactly as it is
        if !save_as && !tab.is_modified() && tab.file.borrow().is_some() {
            return;
        }
        let window = self.me();
        glib::spawn_future_local(async move {
            window.save_tab(&tab, save_as).await;
        });
    }

    fn save_all(&self) {
        let window = self.me();
        glib::spawn_future_local(async move {
            for tab in window.tabs() {
                if tab.is_modified() {
                    window.select(Selected::Doc(tab.clone()));
                    if !window.save_tab(&tab, false).await {
                        return;
                    }
                }
            }
        });
    }

    /// Saves a tab to its file, or to one chosen now (Save as, or a tab
    /// that's never been saved). False when it wasn't saved.
    pub async fn save_tab(&self, tab: &Rc<Tab>, save_as: bool) -> bool {
        let current = tab.file.borrow().clone();
        let file = match current.filter(|_| !save_as) {
            Some(file) => file,
            None => {
                let dialog = gtk::FileDialog::builder()
                    .title("Save as")
                    .accept_label("Save")
                    .modal(true)
                    .build();
                Window::file_filters(&dialog);
                match tab.file.borrow().as_ref() {
                    Some(file) => dialog.set_initial_file(Some(file)),
                    None => {
                        dialog.set_initial_name(Some(&tab.suggested_name()));
                        if let Some(folder) = self.dialog_folder() {
                            dialog.set_initial_folder(Some(&folder));
                        }
                    }
                }
                match dialog.save_future(Some(&self.gtk)).await {
                    Ok(file) => file,
                    Err(_) => return false,
                }
            }
        };
        let text = tab.text();
        let mut encoding = tab.encoding.get();
        let lost = document::unwritable(&text, encoding);
        if lost > 0 {
            match dialogs::ask_unwritable(&self.gtk, encoding, lost).await {
                Some(true) => encoding = Encoding::Utf8,
                Some(false) => {}
                None => return false,
            }
        }
        let bytes = document::encode(&text, encoding, tab.line_ending.get());
        if let Err((_, error)) = file
            .replace_contents_future(bytes, None, false, gio::FileCreateFlags::NONE)
            .await
        {
            let name = file
                .basename()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| tab.title());
            dialogs::error(&self.gtk, &format!("Couldn't save {name}"), error.message()).await;
            return false;
        }
        // Its version history goes with it to its (new) file
        let before = tab.history_id();
        let renamed = tab
            .file
            .borrow()
            .as_ref()
            .is_none_or(|old| !old.equal(&file));
        gtk::RecentManager::default().add_item(&file.uri());
        tab.file.replace(Some(file));
        if renamed {
            tab.markdown.set(tab.has_markdown_name());
        }
        tab.encoding.set(encoding);
        if let (Some(before), Some(after)) = (before, tab.history_id())
            && app().settings().restore_session
        {
            history::rename(&history::dir(), &before, &after);
        }
        self.keep_version(tab, true);
        // Typing while it saved leaves those changes unsaved
        if tab.text() == text {
            tab.buffer.set_modified(false);
        }
        self.update_header(tab);
        // Saved under a new name: Markdown now, or not any more
        self.update_markdown(tab);
        self.update_status();
        app().schedule_session_save();
        true
    }

    /// Formatted view on or off for a tab, from its name, the setting and
    /// its switch.
    fn update_markdown(&self, tab: &Rc<Tab>) {
        let settings = app().settings().clone();
        let formatted = settings.markdown && tab.is_markdown() && tab.formatted.get();
        tab.formatter.set_active(formatted, tab.folder());
        tab.set_preview(settings.markdown && tab.is_markdown() && tab.preview_on.get());
        if self.is_selected(tab) {
            self.sync_markdown(&settings);
        }
    }

    /// The formatting bar, the status bar's switch and their actions, for
    /// the tab showing.
    fn sync_markdown(&self, settings: &Settings) {
        let tab = self.selected_tab();
        let markdown = settings.markdown && tab.as_ref().is_some_and(|tab| tab.is_markdown());
        let formatted = markdown && tab.as_ref().is_some_and(|tab| tab.formatted.get());
        self.format.row.set_visible(markdown);
        // The switch shows while the Markdown setting's on, so a plain text
        // tab can become Markdown too
        self.status
            .markdown
            .set_visible(settings.markdown && settings.status_bar && tab.is_some());
        self.status
            .markdown_label
            .set_label(match (markdown, formatted) {
                (false, _) => "Plain text",
                (true, true) => "Markdown",
                (true, false) => "Markdown, as typed",
            });
        if let Some(action) = self.action("text-mode") {
            action.set_enabled(settings.markdown && tab.is_some());
            action.set_state(&(if markdown { "markdown" } else { "plain" }).to_variant());
        }
        for name in MARKDOWN_ACTIONS {
            if let Some(action) = self.action(name) {
                action.set_enabled(markdown);
            }
        }
        if let Some(action) = self.action("md-view") {
            action.set_state(&formatted.to_variant());
        }
        let preview = markdown && tab.as_ref().is_some_and(|tab| tab.preview_on.get());
        if let Some(action) = self.action("md-preview") {
            action.set_state(&preview.to_variant());
        }
        if markdown && let Some(tab) = tab {
            let level = formatted::current_heading(&tab.buffer) as usize;
            self.format.heading.set_label(HEADINGS[level.min(6)]);
        }
    }

    /// A formatting command on the selected Markdown tab's text.
    fn format_text(&self, edit: impl FnOnce(&gtk::TextBuffer)) {
        let Some(tab) = self.selected_tab().filter(|tab| tab.is_markdown()) else {
            return;
        };
        edit(&tab.buffer);
        tab.view.grab_focus();
    }

    /// Link: asks for its text and address, then puts it in.
    fn ask_link(&self) {
        let Some(tab) = self.selected_tab().filter(|tab| tab.is_markdown()) else {
            return;
        };
        let text = formatted::selected_text(&tab.buffer);
        let weak = Rc::downgrade(&tab);
        dialogs::link(&self.gtk, &text, move |text, url| {
            if let Some(tab) = weak.upgrade() {
                formatted::insert_link(&tab.buffer, &text, &url);
                tab.view.grab_focus();
            }
        });
    }

    /// Export as HTML: the Markdown tab as a web page, in a file chosen now.
    fn export_html(&self) {
        let Some(tab) = self.selected_tab().filter(|tab| tab.is_markdown()) else {
            return;
        };
        let window = self.me();
        glib::spawn_future_local(async move {
            let title = tab.title();
            let stem = title
                .rsplit_once('.')
                .map(|(stem, _)| stem)
                .unwrap_or(&title)
                .to_string();
            let dialog = gtk::FileDialog::builder()
                .title("Export as HTML")
                .accept_label("Export")
                .initial_name(format!("{stem}.html"))
                .modal(true)
                .build();
            if let Some(folder) = window.dialog_folder() {
                dialog.set_initial_folder(Some(&folder));
            }
            let Ok(file) = dialog.save_future(Some(&window.gtk)).await else {
                return;
            };
            let html = markdown::to_html(&tab.text(), &stem);
            if let Err((_, error)) = file
                .replace_contents_future(html.into_bytes(), None, false, gio::FileCreateFlags::NONE)
                .await
            {
                dialogs::error(&window.gtk, "Couldn't export it", error.message()).await;
            }
        });
    }
}
