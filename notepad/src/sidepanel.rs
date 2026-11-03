//! The side panel (View › Side panel, F9): the window's open tabs, the
//! files opened recently, and a Markdown file's outline, its headings, to
//! go to one.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::app::APP_NAME;
use crate::markdown;
use crate::window::{Selected, Window};

/// How many recent files it lists.
const RECENT: usize = 25;

pub struct SidePanel {
    pub widget: gtk::Revealer,
    stack: gtk::Stack,
    open: gtk::ListBox,
    recent: gtk::ListBox,
    outline: gtk::ListBox,
    window: RefCell<Weak<Window>>,
    outline_timer: RefCell<Option<glib::SourceId>>,
}

fn page(list: &gtk::ListBox) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .child(list)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build()
}

fn list() -> gtk::ListBox {
    gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["side-list"])
        .build()
}

fn clear(list: &gtk::ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn hint(list: &gtk::ListBox, text: &str) {
    list.append(
        &gtk::Label::builder()
            .label(text)
            .wrap(true)
            .max_width_chars(28)
            .xalign(0.0)
            .css_classes(["side-hint"])
            .build(),
    );
}

/// A row: its words, a smaller line under them, and what a click does.
fn row(title: &str, detail: &str, indent: i32, run: impl Fn() + 'static) -> gtk::Button {
    let texts = gtk::Box::new(gtk::Orientation::Vertical, 1);
    texts.set_margin_start(indent);
    texts.append(
        &gtk::Label::builder()
            .label(title)
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .max_width_chars(26)
            .css_classes(["side-title"])
            .build(),
    );
    if !detail.is_empty() {
        texts.append(
            &gtk::Label::builder()
                .label(detail)
                .xalign(0.0)
                .ellipsize(gtk::pango::EllipsizeMode::Start)
                .max_width_chars(30)
                .css_classes(["side-detail"])
                .build(),
        );
    }
    let button = gtk::Button::builder()
        .child(&texts)
        .css_classes(["side-row"])
        .tooltip_text(if detail.is_empty() { title } else { detail })
        .build();
    button.connect_clicked(move |_| run());
    button
}

impl SidePanel {
    pub fn new() -> Rc<SidePanel> {
        let open = list();
        let recent = list();
        let outline = list();
        let stack = gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::Crossfade)
            .vexpand(true)
            .build();
        stack.add_titled(&page(&open), Some("open"), "Open");
        stack.add_titled(&page(&recent), Some("recent"), "Recent");
        stack.add_titled(&page(&outline), Some("outline"), "Outline");
        let switcher = gtk::StackSwitcher::builder()
            .stack(&stack)
            .css_classes(["side-switcher"])
            .build();
        let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
        content.add_css_class("side-panel");
        content.set_width_request(250);
        content.append(&switcher);
        content.append(&stack);
        let widget = gtk::Revealer::builder()
            .child(&content)
            .transition_type(gtk::RevealerTransitionType::SlideRight)
            .reveal_child(false)
            .build();
        let panel = Rc::new(SidePanel {
            widget,
            stack,
            open,
            recent,
            outline,
            window: RefCell::default(),
            outline_timer: RefCell::default(),
        });
        let weak = Rc::downgrade(&panel);
        panel.stack.connect_visible_child_name_notify(move |_| {
            if let Some(panel) = weak.upgrade() {
                panel.refresh();
            }
        });
        panel
    }

    pub fn attach(&self, window: &Rc<Window>) {
        self.window.replace(Rc::downgrade(window));
    }

    pub fn set_shown(&self, shown: bool) {
        self.widget.set_reveal_child(shown);
        if shown {
            self.refresh();
        }
    }

    pub fn is_shown(&self) -> bool {
        self.widget.reveals_child()
    }

    /// Opens the panel at its outline (the formatting bar's Outline).
    pub fn show_outline(&self) {
        self.stack.set_visible_child_name("outline");
        crate::app::app().update_settings(|settings| settings.side_panel = true);
    }

    /// Lists what the page showing lists, as it is now.
    pub fn refresh(&self) {
        if !self.is_shown() {
            return;
        }
        match self.stack.visible_child_name().as_deref() {
            Some("recent") => self.fill_recent(),
            Some("outline") => self.fill_outline(),
            _ => self.fill_open(),
        }
    }

    /// The tabs changed (opened, closed, renamed, saved).
    pub fn tabs_changed(&self) {
        if self.is_shown() && self.stack.visible_child_name().as_deref() == Some("open") {
            self.fill_open();
        }
    }

    /// The text changed: the outline follows, a moment later.
    pub fn text_changed(&self) {
        if !self.is_shown() || self.stack.visible_child_name().as_deref() != Some("outline") {
            return;
        }
        if let Some(timer) = self.outline_timer.take() {
            timer.remove();
        }
        let window = self.window.borrow().clone();
        let timer = glib::timeout_add_local_once(Duration::from_millis(400), move || {
            if let Some(window) = window.upgrade() {
                window.side.outline_timer.take();
                window.side.fill_outline();
            }
        });
        self.outline_timer.replace(Some(timer));
    }

    fn fill_open(&self) {
        clear(&self.open);
        let Some(window) = self.window.borrow().upgrade() else {
            return;
        };
        let selected = window.selected_tab();
        for tab in window.tabs() {
            let mut title = tab.title();
            if tab.is_modified() {
                title.push_str("  •");
            }
            let weak_window = Rc::downgrade(&window);
            let weak_tab = Rc::downgrade(&tab);
            let button = row(&title, &tab.location(), 0, move || {
                if let (Some(window), Some(tab)) = (weak_window.upgrade(), weak_tab.upgrade()) {
                    window.select(Selected::Doc(tab));
                }
            });
            if selected
                .as_ref()
                .is_some_and(|selected| Rc::ptr_eq(selected, &tab))
            {
                button.add_css_class("current");
            }
            self.open.append(&button);
        }
    }

    fn fill_recent(&self) {
        clear(&self.recent);
        let Some(window) = self.window.borrow().upgrade() else {
            return;
        };
        let mut items: Vec<gtk::RecentInfo> = gtk::RecentManager::default()
            .items()
            .into_iter()
            .filter(|item| {
                let ours = item.has_application(&glib::prgname().unwrap_or_default())
                    || item.has_application(APP_NAME);
                let text = item.mime_type().starts_with("text/")
                    || markdown::is_markdown_name(&item.display_name());
                (ours || text) && (!item.is_local() || item.exists())
            })
            .collect();
        items.sort_by_key(|item| std::cmp::Reverse(item.modified().to_unix()));
        if items.is_empty() {
            hint(&self.recent, "Files you open and save show here.");
        }
        for item in items.into_iter().take(RECENT) {
            let uri = item.uri().to_string();
            let detail = item
                .uri_display()
                .map(|text| text.to_string())
                .unwrap_or_else(|| uri.clone());
            let weak = Rc::downgrade(&window);
            self.recent
                .append(&row(&item.display_name(), &detail, 0, move || {
                    if let Some(window) = weak.upgrade() {
                        window.open_files(vec![gio::File::for_uri(&uri)]);
                    }
                }));
        }
    }

    fn fill_outline(&self) {
        clear(&self.outline);
        let Some(window) = self.window.borrow().upgrade() else {
            return;
        };
        let Some(tab) = window.selected_tab() else {
            return;
        };
        if !tab.is_markdown() {
            hint(
                &self.outline,
                "A Markdown file's headings show here, to go to one.",
            );
            return;
        }
        let headings = markdown::parse(&tab.text()).headings;
        if headings.is_empty() {
            hint(&self.outline, "No headings yet.");
        }
        for heading in headings {
            let weak = Rc::downgrade(&tab);
            let start = heading.start as i32;
            let title = if heading.text.is_empty() {
                "(untitled)".to_string()
            } else {
                heading.text.clone()
            };
            let button = row(&title, "", 12 * (heading.level as i32 - 1), move || {
                if let Some(tab) = weak.upgrade() {
                    tab.buffer.place_cursor(&tab.buffer.iter_at_offset(start));
                    tab.view
                        .scroll_to_mark(&tab.buffer.get_insert(), 0.0, true, 0.0, 0.1);
                    tab.view.grab_focus();
                }
            });
            button.add_css_class(&format!("outline-level-{}", heading.level));
            self.outline.append(&button);
        }
    }
}
