//! What every window shares: the settings, the session, the look and the
//! fonts. There's one `App` while Notepad runs.

use std::cell::{Cell, Ref, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk::glib::translate::IntoGlib;
use gtk::prelude::*;
use gtk::{gdk, gio, glib, pango};

use crate::session;
use crate::settings::{Settings, Theme};
use crate::tab::Tab;
use crate::window::Window;

pub const APP_ID: &str = "org.patrone.Notepad";
/// The name in window titles and About. On Windows it's Patrone Notepad,
/// so it isn't taken for the system's own text editor.
pub const APP_NAME: &str = if cfg!(windows) {
    "Patrone Notepad"
} else {
    "Notepad"
};

/// The smallest and largest zoom, and its steps, in percent.
pub const ZOOM_MIN: u32 = 10;
pub const ZOOM_MAX: u32 = 500;
pub const ZOOM_STEP: u32 = 10;

/// How long after a change the session is written.
const SESSION_DELAY: Duration = Duration::from_millis(800);

pub struct App {
    pub gtk: gtk::Application,
    settings: RefCell<Settings>,
    windows: RefCell<Vec<Rc<Window>>>,
    /// Each tab's font, at its zoom, and the font preview's.
    fonts: gtk::CssProvider,
    next_id: Cell<u64>,
    save_timer: RefCell<Option<glib::SourceId>>,
    /// Set by Exit, which writes the session once for every window.
    quitting: Cell<bool>,
    started: Cell<bool>,
}

thread_local! {
    static APP: RefCell<Option<Rc<App>>> = const { RefCell::new(None) };
}

/// The running app.
pub fn app() -> Rc<App> {
    APP.with(|app| app.borrow().clone())
        .expect("Notepad's app is set up at startup")
}

/// Sets up the look, shortcuts and app-wide actions, once, when GTK starts.
pub fn startup(application: &gtk::Application) {
    let app = Rc::new(App {
        gtk: application.clone(),
        settings: RefCell::new(Settings::load()),
        windows: RefCell::default(),
        fonts: gtk::CssProvider::new(),
        next_id: Cell::new(1),
        save_timer: RefCell::default(),
        quitting: Cell::new(false),
        started: Cell::new(false),
    });
    APP.with(|cell| *cell.borrow_mut() = Some(app.clone()));
    load_bundled_fonts();
    app.load_style();
    app.add_actions();
    app.apply_theme();
    app.refresh_fonts();
}

impl App {
    fn load_style(&self) {
        let Some(display) = gdk::Display::default() else {
            return;
        };
        let css = gtk::CssProvider::new();
        if let Some(settings) = gtk::Settings::default() {
            // GTK's own theme: Ubuntu's Yaru doesn't load in plain GTK 4 apps.
            // Patrone's title bar button style still reaches it, from the
            // account's gtk.css.
            settings.set_gtk_theme_name(Some("Default"));
            // GTK 4.22's Default theme stays light when dark is preferred,
            // though the preference reaches GTK (and the style sheet below);
            // its dark variant has to be asked for.
            #[allow(deprecated)]
            fn follow(settings: &gtk::Settings) {
                let dark = settings.gtk_interface_color_scheme() == gtk::InterfaceColorScheme::Dark;
                settings.set_gtk_application_prefer_dark_theme(dark);
            }
            follow(&settings);
            settings.connect_gtk_interface_color_scheme_notify(follow);
            // Markdown's formatting colors follow it too
            settings.connect_gtk_interface_color_scheme_notify(|_| {
                let dark = is_dark();
                for window in app().windows() {
                    for tab in window.tabs() {
                        tab.formatter.set_dark(dark);
                    }
                }
            });
            settings
                .bind_property("gtk-interface-color-scheme", &css, "prefers-color-scheme")
                .sync_create()
                .build();
        }
        css.load_from_string(include_str!("../data/style.css"));
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        gtk::style_context_add_provider_for_display(
            &display,
            &self.fonts,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
        // The title bar style in the account's gtk.css paints the title bar;
        // these few rules, just above it, keep the tabs readable on any style.
        let over_theme = gtk::CssProvider::new();
        over_theme.load_from_string(include_str!("../data/over-theme.css"));
        gtk::style_context_add_provider_for_display(
            &display,
            &over_theme,
            gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
        );
        crate::live_style::follow(&display);
        gtk::Window::set_default_icon_name(APP_ID);
    }

    fn add_actions(&self) {
        let accels: &[(&str, &[&str])] = &[
            ("win.new-tab", &["<Control>n", "<Control>t"]),
            ("win.new-window", &["<Control><Shift>n"]),
            ("win.open", &["<Control>o"]),
            ("win.save", &["<Control>s"]),
            ("win.save-as", &["<Control><Shift>s"]),
            ("win.save-all", &["<Control><Alt>s"]),
            ("win.print", &["<Control>p"]),
            ("win.close-tab", &["<Control>w", "<Control>F4"]),
            ("win.close-window", &["<Control><Shift>w"]),
            // Undo, Cut, Copy, Paste, Delete and Select all keep the text
            // view's and the find box's own keys, which a shortcut here would
            // take from them; their menu items only show the keys.
            ("win.redo", &["<Control>y"]),
            ("win.find", &["<Control>f"]),
            ("win.find-next", &["F3"]),
            ("win.find-previous", &["<Shift>F3"]),
            ("win.replace", &["<Control>h"]),
            ("win.go-to", &["<Control>g"]),
            ("win.time-date", &["F5"]),
            (
                "win.zoom-in",
                &["<Control>plus", "<Control>equal", "<Control>KP_Add"],
            ),
            ("win.zoom-out", &["<Control>minus", "<Control>KP_Subtract"]),
            ("win.zoom-reset", &["<Control>0", "<Control>KP_0"]),
            ("win.next-tab", &["<Control>Tab", "<Control>Page_Down"]),
            // Markdown's formatting, in Markdown tabs
            ("win.md-bold", &["<Control>b"]),
            ("win.md-italic", &["<Control>i"]),
            ("win.md-strike", &["<Control><Shift>x"]),
            (
                "win.md-bullets",
                &["<Control><Shift>8", "<Control>asterisk"],
            ),
            (
                "win.md-numbers",
                &["<Control><Shift>7", "<Control>ampersand"],
            ),
            ("win.md-link", &["<Control>k"]),
            ("win.md-clear", &["<Control>backslash"]),
            ("win.md-view", &["<Control><Shift>m"]),
            ("win.md-heading::0", &["<Control><Alt>0"]),
            ("win.md-heading::1", &["<Control><Alt>1"]),
            ("win.md-heading::2", &["<Control><Alt>2"]),
            ("win.md-heading::3", &["<Control><Alt>3"]),
            ("win.md-heading::4", &["<Control><Alt>4"]),
            ("win.md-heading::5", &["<Control><Alt>5"]),
            ("win.md-heading::6", &["<Control><Alt>6"]),
            (
                "win.previous-tab",
                &[
                    "<Control><Shift>Tab",
                    "<Control><Shift>ISO_Left_Tab",
                    "<Control>Page_Up",
                ],
            ),
        ];
        for (action, keys) in accels {
            self.gtk.set_accels_for_action(action, keys);
        }

        // Word wrap and the status bar are settings, the same in every window
        let settings = self.settings.borrow();
        for (name, state) in [
            ("word-wrap", settings.word_wrap),
            ("status-bar", settings.status_bar),
        ] {
            let action = gio::SimpleAction::new_stateful(name, None, &state.to_variant());
            action.connect_activate(move |action, _| {
                let on = !action
                    .state()
                    .and_then(|state| state.get::<bool>())
                    .unwrap_or(false);
                app().update_settings(|settings| match name {
                    "word-wrap" => settings.word_wrap = on,
                    _ => settings.status_bar = on,
                });
            });
            self.gtk.add_action(&action);
        }
    }

    pub fn settings(&self) -> Ref<'_, Settings> {
        self.settings.borrow()
    }

    /// Changes the settings, saves them and applies them everywhere.
    pub fn update_settings(&self, change: impl FnOnce(&mut Settings)) {
        let before = self.settings.borrow().clone();
        let mut after = before.clone();
        change(&mut after);
        if after == before {
            return;
        }
        after.save();
        *self.settings.borrow_mut() = after.clone();
        if after.theme != before.theme {
            self.apply_theme();
        }
        if (&after.font_family, &after.font_face, after.font_size)
            != (&before.font_family, &before.font_face, before.font_size)
        {
            self.refresh_fonts();
        }
        for (name, state) in [
            ("word-wrap", after.word_wrap),
            ("status-bar", after.status_bar),
        ] {
            if let Some(action) = self
                .gtk
                .lookup_action(name)
                .and_downcast::<gio::SimpleAction>()
            {
                action.set_state(&state.to_variant());
            }
        }
        if after.restore_session != before.restore_session {
            if after.restore_session {
                self.save_session();
            } else {
                // Forgotten at once: starting fresh is what was asked for
                if let Some(timer) = self.save_timer.take() {
                    timer.remove();
                }
                session::clear(&session::dir());
                // If it's turned on again, every tab's text is written anew
                for window in self.windows() {
                    for tab in window.tabs() {
                        tab.session_key.replace(None);
                    }
                }
            }
        }
        for window in self.windows() {
            window.apply_settings(&after);
        }
    }

    fn apply_theme(&self) {
        let Some(settings) = gtk::Settings::default() else {
            return;
        };
        match self.settings.borrow().theme {
            Theme::Light => {
                settings.set_gtk_interface_color_scheme(gtk::InterfaceColorScheme::Light)
            }
            Theme::Dark => settings.set_gtk_interface_color_scheme(gtk::InterfaceColorScheme::Dark),
            // Back to the desktop's own preference
            Theme::System => settings.reset_property("gtk-interface-color-scheme"),
        }
    }

    pub fn next_id(&self) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }

    pub fn windows(&self) -> Vec<Rc<Window>> {
        self.windows.borrow().clone()
    }

    pub fn add_window(&self, window: Rc<Window>) {
        self.windows.borrow_mut().push(window);
    }

    /// A window's gone: the session goes on without it.
    pub fn forget(&self, window: &Rc<Window>) {
        self.windows
            .borrow_mut()
            .retain(|other| !Rc::ptr_eq(other, window));
        if !self.windows.borrow().is_empty() && !self.quitting.get() {
            self.save_session();
        }
    }

    /// The window last used.
    pub fn active_window(&self) -> Option<Rc<Window>> {
        let active = self.gtk.active_window();
        let windows = self.windows.borrow();
        active
            .and_then(|active| {
                windows
                    .iter()
                    .find(|window| window.gtk.upcast_ref::<gtk::Window>() == &active)
                    .cloned()
            })
            .or_else(|| windows.last().cloned())
    }

    /// The tab, in any window, that has `file` open.
    pub fn find_open(&self, file: &gio::File) -> Option<(Rc<Window>, Rc<Tab>)> {
        self.windows().into_iter().find_map(|window| {
            let tab = window.tab_with_file(file)?;
            Some((window, tab))
        })
    }

    /// `notepad [--new-window] [FILE...]`, on the first start or while
    /// Notepad's running.
    pub fn command_line(&self, files: Vec<gio::File>, new_window: bool) {
        let first = !self.started.replace(true);
        if first && self.settings.borrow().restore_session {
            self.restore();
        }
        if !files.is_empty() {
            let open_in_tab =
                !new_window && self.settings.borrow().open_in == crate::settings::OpenIn::NewTab;
            match self.active_window().filter(|_| open_in_tab) {
                Some(window) => {
                    window.open_files(files);
                    window.gtk.present();
                }
                None => {
                    let window = Window::new(None);
                    window.open_files(files);
                    window.gtk.present();
                }
            }
        } else if first && !self.windows.borrow().is_empty() && !new_window {
            // The last session's windows are showing
        } else {
            let window = Window::new(None);
            window.new_tab();
            window.gtk.present();
        }
    }

    /// Opens the windows and tabs of the last session.
    fn restore(&self) {
        for state in session::load(&session::dir()) {
            let window = Window::new(Some((state.width, state.height, state.maximized)));
            for tab in &state.tabs {
                window.restore_tab(tab);
            }
            if window.tabs().is_empty() {
                window.gtk.destroy();
                continue;
            }
            window.select_index(state.current);
            window.gtk.present();
        }
    }

    /// Writes the session a moment from now, when it's kept.
    pub fn schedule_session_save(&self) {
        if !self.settings.borrow().restore_session
            || self.quitting.get()
            || self.save_timer.borrow().is_some()
        {
            return;
        }
        let timer = glib::timeout_add_local_once(SESSION_DELAY, || {
            let app = app();
            app.save_timer.take();
            app.save_session();
        });
        self.save_timer.replace(Some(timer));
    }

    /// Writes the session now, when it's kept.
    pub fn save_session(&self) {
        if let Some(timer) = self.save_timer.take() {
            timer.remove();
        }
        if !self.settings.borrow().restore_session {
            return;
        }
        let windows = self.windows();
        let states: Vec<_> = windows.iter().filter_map(|window| window.state()).collect();
        match session::save(&session::dir(), &states) {
            Ok(()) => {
                for window in &windows {
                    for tab in window.tabs() {
                        tab.session_dirty.set(false);
                    }
                }
            }
            Err(error) => eprintln!("notepad: couldn't keep the session: {error}"),
        }
    }

    /// File › Exit: closes every window. With the session kept, it keeps
    /// them all; otherwise each asks about its unsaved changes first.
    pub fn exit(self: &Rc<Self>) {
        let app = self.clone();
        glib::spawn_future_local(async move {
            if app.settings.borrow().restore_session {
                app.save_session();
            } else {
                for window in app.windows() {
                    if !window.confirm_close_all().await {
                        return;
                    }
                }
            }
            app.quitting.set(true);
            for window in app.windows() {
                window.gtk.destroy();
            }
        });
    }

    /// Closing a window: the last one keeps its tabs in the session, when
    /// it's kept; others (and every window when it isn't) ask about unsaved
    /// changes, since nothing else would keep them.
    pub fn close_window(self: &Rc<Self>, window: &Rc<Window>) {
        // Asked once, however often the close button's pressed meanwhile
        if window.closing.replace(true) {
            return;
        }
        let app = self.clone();
        let window = window.clone();
        glib::spawn_future_local(async move {
            let last = app.windows.borrow().len() <= 1;
            if last && app.settings.borrow().restore_session {
                app.save_session();
            } else if !window.confirm_close_all().await {
                window.closing.set(false);
                return;
            }
            window.gtk.destroy();
        });
    }

    /// The font as CSS: its family, weight and style.
    fn font_css(&self) -> (String, i32, &'static str) {
        let settings = self.settings.borrow();
        let family = settings
            .font_family
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        let (weight, style) = face_style(&settings.font_family, &settings.font_face);
        (format!("\"{family}\", monospace"), weight, style)
    }

    /// The font, as Pango describes it, at its size (for printing).
    pub fn font_description(&self) -> pango::FontDescription {
        let settings = self.settings.borrow();
        let mut description = face(&settings.font_family, &settings.font_face)
            .map(|face| face.describe())
            .unwrap_or_else(|| pango::FontDescription::from_string("monospace"));
        description.set_family(&settings.font_family);
        description.set_size(settings.font_size as i32 * pango::SCALE);
        description
    }

    /// Each tab's text in the font, at the tab's zoom.
    pub fn refresh_fonts(&self) {
        let (family, weight, style) = self.font_css();
        let size = self.settings.borrow().font_size as f64;
        let rule = |selector: &str, points: f64| {
            format!(
                "{selector} {{ font-family: {family}; font-size: {points:.2}pt; font-weight: {weight}; font-style: {style}; }}\n"
            )
        };
        let mut css = rule(".np-font-preview", size);
        for window in self.windows() {
            for tab in window.tabs() {
                css.push_str(&rule(
                    &format!(".{}", tab.view_class()),
                    size * tab.zoom.get() as f64 / 100.0,
                ));
            }
        }
        self.fonts.load_from_string(&css);
    }
}

/// PatroneOS's fonts, when they come with the app rather than the system
/// (the Windows build: `share/fonts/patroneos` beside its `bin` folder),
/// loaded for this app alone. Nothing is installed.
fn load_bundled_fonts() {
    let Some(folder) = std::env::current_exe().ok().and_then(|exe| {
        Some(
            exe.parent()?
                .parent()?
                .join("share")
                .join("fonts")
                .join("patroneos"),
        )
    }) else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&folder) else {
        return;
    };
    let map = pangocairo::FontMap::default();
    for entry in entries.flatten() {
        let path = entry.path();
        let font = path.extension().is_some_and(|extension| {
            extension.eq_ignore_ascii_case("ttf") || extension.eq_ignore_ascii_case("otf")
        });
        if font && let Err(error) = map.add_font_file(&path) {
            eprintln!("notepad: couldn't load {}: {error}", path.display());
        }
    }
}

/// Whether the app shows dark.
pub fn is_dark() -> bool {
    gtk::Settings::default().is_some_and(|settings| {
        settings.gtk_interface_color_scheme() == gtk::InterfaceColorScheme::Dark
    })
}

/// PatroneOS's own font families, which the font list starts with.
pub const PATRONEOS_FONTS: [&str; 5] = [
    "PatroneOS Mono",
    "PatroneOS Sans",
    "PatroneOS Document",
    "PatroneOS Serif",
    "PatroneOS Classic",
];

/// The installed font families: PatroneOS's first, then the rest by name.
pub fn font_families() -> Vec<pango::FontFamily> {
    let map = pangocairo::FontMap::default();
    let mut families: Vec<_> = map.list_families().into_iter().collect();
    families.sort_by_key(|family| {
        let name = family.name();
        let ours = PATRONEOS_FONTS
            .iter()
            .position(|font| *font == name.as_str());
        (ours.unwrap_or(PATRONEOS_FONTS.len()), name.to_lowercase())
    });
    families
}

/// A family's faces ("Regular", "Bold"...), lightest and upright first.
pub fn font_faces(family: &str) -> Vec<pango::FontFace> {
    let Some(family) = font_families()
        .into_iter()
        .find(|candidate| candidate.name() == family)
    else {
        return Vec::new();
    };
    let mut faces: Vec<_> = family.list_faces().into_iter().collect();
    faces.sort_by_key(|face| {
        let description = face.describe();
        (
            description.style() != pango::Style::Normal,
            description.weight().into_glib(),
        )
    });
    faces
}

fn face(family: &str, name: &str) -> Option<pango::FontFace> {
    font_faces(family)
        .into_iter()
        .find(|face| face.face_name() == name)
}

/// The CSS weight and style of a family's face (400 and normal when the
/// font isn't installed).
fn face_style(family: &str, name: &str) -> (i32, &'static str) {
    let Some(face) = face(family, name) else {
        return (400, "normal");
    };
    let description = face.describe();
    let style = match description.style() {
        pango::Style::Italic => "italic",
        pango::Style::Oblique => "oblique",
        _ => "normal",
    };
    (description.weight().into_glib(), style)
}
