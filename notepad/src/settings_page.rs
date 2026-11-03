//! Settings, shown in a tab of its own: the app's theme, the font, word
//! wrap, where opened files go, whether the last session comes back, and
//! what this app is. Every change applies at once, in every window.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::glib::translate::IntoGlib;
use gtk::pango;
use gtk::prelude::*;

use crate::app::{self, app};
use crate::settings::{OpenIn, SIZES, Settings, Theme};

pub struct Page {
    pub widget: gtk::ScrolledWindow,
    theme: gtk::DropDown,
    family: gtk::DropDown,
    face: gtk::DropDown,
    size: gtk::DropDown,
    word_wrap: gtk::Switch,
    markdown: gtk::Switch,
    open_in: gtk::DropDown,
    restore: gtk::Switch,
    families: Vec<String>,
    faces: RefCell<Vec<String>>,
    /// Set while the page shows the settings, so that isn't taken for a
    /// change.
    syncing: Cell<bool>,
}

fn strings<'a>(items: impl IntoIterator<Item = &'a str>) -> gtk::StringList {
    let list = gtk::StringList::new(&[]);
    for item in items {
        list.append(item);
    }
    list
}

fn dropdown(model: &gtk::StringList) -> gtk::DropDown {
    let dropdown = gtk::DropDown::builder()
        .model(model)
        .valign(gtk::Align::Center)
        .build();
    // Typing finds an item in a long list (the fonts)
    dropdown.set_expression(Some(gtk::PropertyExpression::new(
        gtk::StringObject::static_type(),
        None::<gtk::Expression>,
        "string",
    )));
    dropdown
}

fn heading(text: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .xalign(0.0)
        .css_classes(["settings-heading"])
        .build()
}

/// A title and what it means, for a card's left side.
fn labels(title: &str, subtitle: &str) -> gtk::Box {
    let texts = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .valign(gtk::Align::Center)
        .build();
    texts.append(
        &gtk::Label::builder()
            .label(title)
            .xalign(0.0)
            .css_classes(["card-title"])
            .build(),
    );
    if !subtitle.is_empty() {
        texts.append(
            &gtk::Label::builder()
                .label(subtitle)
                .xalign(0.0)
                .wrap(true)
                .css_classes(["card-subtitle"])
                .build(),
        );
    }
    texts
}

/// A setting: an icon, its title and what it means, and its control.
fn card(icon: &str, title: &str, subtitle: &str, control: &impl IsA<gtk::Widget>) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    card.add_css_class("card");
    card.append(
        &gtk::Image::builder()
            .icon_name(icon)
            .pixel_size(20)
            .valign(gtk::Align::Center)
            .build(),
    );
    card.append(&labels(title, subtitle));
    card.append(control);
    card
}

impl Page {
    pub fn new() -> Rc<Page> {
        let theme = dropdown(&strings(Theme::ALL.iter().map(|theme| theme.label())));
        let families: Vec<String> = app::font_families()
            .iter()
            .map(|family| family.name().to_string())
            .collect();
        let family = dropdown(&strings(families.iter().map(String::as_str)));
        family.set_enable_search(true);
        family.set_width_request(220);
        let face = dropdown(&gtk::StringList::new(&[]));
        face.set_width_request(160);
        let sizes: Vec<String> = SIZES.iter().map(|size| size.to_string()).collect();
        let size = dropdown(&strings(sizes.iter().map(String::as_str)));
        size.set_width_request(90);
        let word_wrap = gtk::Switch::builder().valign(gtk::Align::Center).build();
        let markdown = gtk::Switch::builder().valign(gtk::Align::Center).build();
        let open_in = dropdown(&strings(OpenIn::ALL.iter().map(|open_in| open_in.label())));
        let restore = gtk::Switch::builder().valign(gtk::Align::Center).build();

        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .css_classes(["settings-page"])
            .build();
        content.append(
            &gtk::Label::builder()
                .label("Settings")
                .xalign(0.0)
                .css_classes(["settings-title"])
                .build(),
        );

        content.append(&heading("Look"));
        content.append(&card(
            "preferences-desktop-appearance-symbolic",
            "Theme",
            "Light, dark, or the same as the desktop",
            &theme,
        ));

        content.append(&heading("Text"));
        let font = gtk::Box::new(gtk::Orientation::Vertical, 12);
        font.add_css_class("card");
        let font_top = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        font_top.append(
            &gtk::Image::builder()
                .icon_name("preferences-desktop-font-symbolic")
                .pixel_size(20)
                .valign(gtk::Align::Center)
                .build(),
        );
        font_top.append(&labels("Font", "The font for the text in every tab"));
        font.append(&font_top);
        let grid = gtk::Grid::builder()
            .column_spacing(12)
            .row_spacing(8)
            .margin_start(36)
            .build();
        for (column, (name, control)) in [("Family", &family), ("Style", &face), ("Size", &size)]
            .into_iter()
            .enumerate()
        {
            grid.attach(
                &gtk::Label::builder()
                    .label(name)
                    .xalign(0.0)
                    .css_classes(["card-subtitle"])
                    .build(),
                column as i32,
                0,
                1,
                1,
            );
            grid.attach(control, column as i32, 1, 1, 1);
        }
        font.append(&grid);
        font.append(
            &gtk::Label::builder()
                .label("The quick brown fox jumps over the lazy dog. 0123456789")
                .xalign(0.0)
                .wrap(true)
                .margin_start(36)
                .css_classes(["np-font-preview"])
                .build(),
        );
        content.append(&font);
        content.append(&card(
            "format-justify-left-symbolic",
            "Word wrap",
            "Fit long lines to the window's width",
            &word_wrap,
        ));
        content.append(&card(
            "format-text-bold-symbolic",
            "Markdown formatting",
            "Show Markdown files (.md) formatted, with a formatting bar: headings, bold, lists, \
             links, tables, tasks, code and images. They're still saved as the Markdown they are. \
             Turned off, they're plain text like any other file.",
            &markdown,
        ));

        content.append(&heading("Files"));
        content.append(&card(
            "document-open-symbolic",
            "Where files open",
            "Files opened from other apps, such as File Manager, or from the command line",
            &open_in,
        ));

        content.append(&heading("Sessions"));
        content.append(&card(
            "document-save-symbolic",
            "Restore open tabs",
            "Notepad saves your open tabs and their unsaved changes as you work (auto-save), and \
             brings them back when it starts again. Your files change only when you save them. \
             Turned off, Notepad starts with a new tab and asks whether to save your changes when \
             you close it.",
            &restore,
        ));

        content.append(&heading("About"));
        let source =
            gtk::LinkButton::with_label("https://github.com/billsecond/patroneos", "Source code");
        source.set_valign(gtk::Align::Center);
        content.append(&card(
            crate::app::APP_ID,
            &format!("Notepad {}", env!("CARGO_PKG_VERSION")),
            "Part of PatroneOS. Free software under the GNU General Public License, version 3 or later.",
            &source,
        ));

        let widget = gtk::ScrolledWindow::builder()
            .child(&content)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .hexpand(true)
            .vexpand(true)
            .build();
        widget.add_css_class("settings-scroller");

        let page = Rc::new(Page {
            widget,
            theme,
            family,
            face,
            size,
            word_wrap,
            markdown,
            open_in,
            restore,
            families,
            faces: RefCell::default(),
            syncing: Cell::new(false),
        });
        page.connect();
        page
    }

    /// Each control changes its setting.
    fn connect(self: &Rc<Self>) {
        let on = |run: fn(&Page)| {
            let weak = Rc::downgrade(self);
            move || {
                if let Some(page) = weak.upgrade()
                    && !page.syncing.get()
                {
                    run(&page);
                }
            }
        };
        let run = on(|page| {
            let theme = Theme::ALL[(page.theme.selected() as usize).min(Theme::ALL.len() - 1)];
            app().update_settings(|settings| settings.theme = theme);
        });
        self.theme.connect_selected_notify(move |_| run());
        let run = on(|page| {
            let Some(family) = page.families.get(page.family.selected() as usize).cloned() else {
                return;
            };
            let face = regular_face(&family);
            app().update_settings(|settings| {
                settings.font_family = family;
                settings.font_face = face;
            });
        });
        self.family.connect_selected_notify(move |_| run());
        let run = on(|page| {
            let Some(face) = page
                .faces
                .borrow()
                .get(page.face.selected() as usize)
                .cloned()
            else {
                return;
            };
            app().update_settings(|settings| settings.font_face = face);
        });
        self.face.connect_selected_notify(move |_| run());
        let run = on(|page| {
            let Some(size) = SIZES.get(page.size.selected() as usize).copied() else {
                return;
            };
            app().update_settings(|settings| settings.font_size = size);
        });
        self.size.connect_selected_notify(move |_| run());
        let run = on(|page| {
            let on = page.word_wrap.is_active();
            app().update_settings(|settings| settings.word_wrap = on);
        });
        self.word_wrap.connect_active_notify(move |_| run());
        let run = on(|page| {
            let on = page.markdown.is_active();
            app().update_settings(|settings| settings.markdown = on);
        });
        self.markdown.connect_active_notify(move |_| run());
        let run = on(|page| {
            let open_in =
                OpenIn::ALL[(page.open_in.selected() as usize).min(OpenIn::ALL.len() - 1)];
            app().update_settings(|settings| settings.open_in = open_in);
        });
        self.open_in.connect_selected_notify(move |_| run());
        let run = on(|page| {
            let on = page.restore.is_active();
            app().update_settings(|settings| settings.restore_session = on);
        });
        self.restore.connect_active_notify(move |_| run());
    }

    /// Shows the settings as they are.
    pub fn sync(&self, settings: &Settings) {
        self.syncing.set(true);
        let index = |found: Option<usize>| {
            found
                .map(|index| index as u32)
                .unwrap_or(gtk::INVALID_LIST_POSITION)
        };
        self.theme.set_selected(index(
            Theme::ALL.iter().position(|theme| *theme == settings.theme),
        ));
        self.family.set_selected(index(
            self.families
                .iter()
                .position(|family| *family == settings.font_family),
        ));
        let faces: Vec<String> = app::font_faces(&settings.font_family)
            .iter()
            .map(|face| face.face_name().to_string())
            .collect();
        if *self.faces.borrow() != faces {
            self.face
                .set_model(Some(&strings(faces.iter().map(String::as_str))));
            self.faces.replace(faces);
        }
        self.face.set_selected(index(
            self.faces
                .borrow()
                .iter()
                .position(|face| *face == settings.font_face),
        ));
        // A size that isn't in the list shows the nearest one
        let nearest = SIZES
            .iter()
            .enumerate()
            .min_by_key(|(_, size)| size.abs_diff(settings.font_size))
            .map(|(index, _)| index);
        self.size.set_selected(index(nearest));
        self.word_wrap.set_active(settings.word_wrap);
        self.markdown.set_active(settings.markdown);
        self.open_in.set_selected(index(
            OpenIn::ALL
                .iter()
                .position(|open_in| *open_in == settings.open_in),
        ));
        self.restore.set_active(settings.restore_session);
        self.syncing.set(false);
    }
}

/// A family's everyday face: upright and of normal weight, if it has one.
fn regular_face(family: &str) -> String {
    let faces = app::font_faces(family);
    faces
        .iter()
        .find(|face| {
            let description = face.describe();
            description.style() == pango::Style::Normal && description.weight().into_glib() == 400
        })
        .or(faces.first())
        .map(|face| face.face_name().to_string())
        .unwrap_or_else(|| "Regular".into())
}
