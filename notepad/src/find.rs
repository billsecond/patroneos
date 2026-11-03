//! Find and replace: a box over the top right of the text. Typing finds as
//! you go; Enter finds the next match and Shift+Enter the one before; the
//! arrow opens the Replace row; the options button has Match case and Wrap
//! around; Esc closes it.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use gtk::prelude::*;
use gtk::{gdk, glib};

use crate::app::app;
use crate::settings::Settings;
use crate::window::Window;

pub struct Panel {
    pub widget: gtk::Box,
    expand: gtk::ToggleButton,
    find: gtk::Entry,
    replace: gtk::Entry,
    replace_row: gtk::Box,
    match_case: gtk::CheckButton,
    wrap_around: gtk::CheckButton,
    window: RefCell<Weak<Window>>,
    /// Set while the panel itself changes the selection or the options.
    busy: std::cell::Cell<bool>,
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .css_classes(["find-button"])
        .build()
}

impl Panel {
    pub fn new() -> Rc<Panel> {
        let expand = gtk::ToggleButton::builder()
            .icon_name("pan-end-symbolic")
            .tooltip_text("Show Replace")
            .css_classes(["find-button"])
            .build();
        let find = gtk::Entry::builder()
            .placeholder_text("Find")
            .hexpand(true)
            .width_chars(24)
            .build();
        let previous = icon_button("go-up-symbolic", "Find previous (Shift+F3)");
        let next = icon_button("go-down-symbolic", "Find next (F3)");
        let match_case = gtk::CheckButton::with_label("Match case");
        let wrap_around = gtk::CheckButton::with_label("Wrap around");
        let options_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        options_box.add_css_class("find-options");
        options_box.append(&match_case);
        options_box.append(&wrap_around);
        let options = gtk::MenuButton::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("Options")
            .popover(&gtk::Popover::builder().child(&options_box).build())
            .css_classes(["find-button"])
            .build();
        let close = icon_button("window-close-symbolic", "Close (Esc)");

        let find_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        find_row.append(&expand);
        find_row.append(&find);
        find_row.append(&previous);
        find_row.append(&next);
        find_row.append(&options);
        find_row.append(&close);

        let replace = gtk::Entry::builder()
            .placeholder_text("Replace")
            .hexpand(true)
            .build();
        let replace_one = gtk::Button::builder()
            .label("Replace")
            .css_classes(["find-text-button"])
            .build();
        let replace_all = gtk::Button::builder()
            .label("Replace all")
            .css_classes(["find-text-button"])
            .build();
        let replace_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        replace_row.add_css_class("replace-row");
        replace_row.append(&replace);
        replace_row.append(&replace_one);
        replace_row.append(&replace_all);
        replace_row.set_visible(false);

        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .halign(gtk::Align::End)
            .valign(gtk::Align::Start)
            .css_classes(["find-panel"])
            .visible(false)
            .build();
        widget.append(&find_row);
        widget.append(&replace_row);

        let panel = Rc::new(Panel {
            widget,
            expand,
            find,
            replace,
            replace_row,
            match_case,
            wrap_around,
            window: RefCell::default(),
            busy: std::cell::Cell::new(false),
        });

        let on = |run: fn(&Panel)| {
            let weak = Rc::downgrade(&panel);
            move || {
                if let Some(panel) = weak.upgrade() {
                    run(&panel);
                }
            }
        };
        let run = on(|panel| panel.set_replace_shown(panel.expand.is_active()));
        panel.expand.connect_toggled(move |_| run());
        // Finding as you type: from where the last match started, so the
        // match grows with the word
        let run = on(|panel| {
            if !panel.busy.get() {
                panel.search(true, true);
            }
        });
        panel.find.connect_changed(move |_| run());
        let run = on(|panel| {
            panel.search(true, false);
        });
        panel.find.connect_activate(move |_| run());
        let run = on(|panel| {
            panel.search(false, false);
        });
        previous.connect_clicked(move |_| run());
        let run = on(|panel| {
            panel.search(true, false);
        });
        next.connect_clicked(move |_| run());
        let run = on(|panel| panel.replace_one());
        replace_one.connect_clicked(move |_| run());
        let run = on(|panel| panel.replace_one());
        panel.replace.connect_activate(move |_| run());
        let run = on(|panel| panel.replace_all());
        replace_all.connect_clicked(move |_| run());
        let run = on(|panel| panel.hide());
        close.connect_clicked(move |_| run());
        let run = on(|panel| panel.save_options());
        panel.match_case.connect_toggled(move |_| run());
        let run = on(|panel| panel.save_options());
        panel.wrap_around.connect_toggled(move |_| run());

        // Esc closes; Shift+Enter finds the match before
        let keys = gtk::EventControllerKey::new();
        let weak = Rc::downgrade(&panel);
        keys.connect_key_pressed(move |_, key, _, state| {
            let Some(panel) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            match key {
                gdk::Key::Escape => {
                    panel.hide();
                    glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter
                    if state.contains(gdk::ModifierType::SHIFT_MASK) && panel.find.has_focus() =>
                {
                    panel.search(false, false);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        panel.widget.add_controller(keys);
        panel
    }

    pub fn attach(&self, window: &Rc<Window>) {
        self.window.replace(Rc::downgrade(window));
    }

    pub fn sync(&self, settings: &Settings) {
        self.busy.set(true);
        self.match_case.set_active(settings.match_case);
        self.wrap_around.set_active(settings.wrap_around);
        self.busy.set(false);
    }

    fn save_options(&self) {
        if self.busy.get() {
            return;
        }
        let (match_case, wrap_around) = (self.match_case.is_active(), self.wrap_around.is_active());
        app().update_settings(|settings| {
            settings.match_case = match_case;
            settings.wrap_around = wrap_around;
        });
    }

    fn set_replace_shown(&self, shown: bool) {
        self.replace_row.set_visible(shown);
        self.expand.set_icon_name(if shown {
            "pan-down-symbolic"
        } else {
            "pan-end-symbolic"
        });
        self.expand.set_tooltip_text(Some(if shown {
            "Hide Replace"
        } else {
            "Show Replace"
        }));
    }

    pub fn has_focus(&self) -> bool {
        self.widget.is_visible() && (self.find.has_focus() || self.replace.has_focus())
    }

    /// Opens the box (with the Replace row for Replace), with the selected
    /// text to find if it's on one line.
    pub fn show(&self, replace: bool) {
        let Some(tab) = self
            .window
            .borrow()
            .upgrade()
            .and_then(|window| window.selected_tab())
        else {
            return;
        };
        if let Some((start, end)) = tab.buffer.selection_bounds() {
            let text = tab.buffer.text(&start, &end, false);
            if !text.contains('\n') && text.chars().count() <= 200 {
                self.busy.set(true);
                self.find.set_text(&text);
                self.busy.set(false);
            }
        }
        self.widget.set_visible(true);
        if replace {
            self.expand.set_active(true);
        }
        self.set_replace_shown(self.expand.is_active());
        self.find.grab_focus();
        self.find.select_region(0, -1);
    }

    pub fn hide(&self) {
        if !self.widget.is_visible() {
            return;
        }
        let refocus = self.has_focus();
        self.widget.set_visible(false);
        if refocus
            && let Some(tab) = self
                .window
                .borrow()
                .upgrade()
                .and_then(|window| window.selected_tab())
        {
            tab.view.grab_focus();
        }
    }

    /// F3 and Shift+F3: the next or previous match of what was last looked
    /// for, opening the box when there's nothing to look for yet.
    pub fn find_again(&self, forward: bool) {
        if self.find.text().is_empty() {
            self.show(false);
        } else {
            self.search(forward, false);
        }
    }

    /// Another tab is showing: its matches aren't marked as missing yet.
    pub fn selection_changed(&self) {
        self.find.remove_css_class("error");
    }

    fn flags(&self) -> gtk::TextSearchFlags {
        let mut flags = gtk::TextSearchFlags::TEXT_ONLY | gtk::TextSearchFlags::VISIBLE_ONLY;
        if !self.match_case.is_active() {
            flags |= gtk::TextSearchFlags::CASE_INSENSITIVE;
        }
        flags
    }

    /// Selects the next (or previous) match after the selection, or from
    /// its start when `from_start` (finding as you type). False when
    /// there's none.
    fn search(&self, forward: bool, from_start: bool) -> bool {
        let Some(tab) = self
            .window
            .borrow()
            .upgrade()
            .and_then(|window| window.selected_tab())
        else {
            return false;
        };
        let needle = self.find.text().to_string();
        if needle.is_empty() {
            self.find.remove_css_class("error");
            return false;
        }
        let buffer = &tab.buffer;
        let (start, end) = buffer.selection_bounds().unwrap_or_else(|| {
            let cursor = buffer.iter_at_mark(&buffer.get_insert());
            (cursor, cursor)
        });
        let flags = self.flags();
        let wrap = self.wrap_around.is_active();
        let found = if forward {
            let from = if from_start { start } else { end };
            from.forward_search(&needle, flags, None).or_else(|| {
                wrap.then(|| buffer.start_iter().forward_search(&needle, flags, None))
                    .flatten()
            })
        } else {
            start.backward_search(&needle, flags, None).or_else(|| {
                wrap.then(|| buffer.end_iter().backward_search(&needle, flags, None))
                    .flatten()
            })
        };
        match found {
            Some((match_start, match_end)) => {
                self.busy.set(true);
                buffer.select_range(&match_end, &match_start);
                self.busy.set(false);
                tab.view
                    .scroll_to_mark(&buffer.get_insert(), 0.1, false, 0.0, 0.0);
                self.find.remove_css_class("error");
                true
            }
            None => {
                self.find.add_css_class("error");
                false
            }
        }
    }

    /// Replaces the selected match and finds the next; with none selected,
    /// finds it first.
    fn replace_one(&self) {
        let Some(tab) = self
            .window
            .borrow()
            .upgrade()
            .and_then(|window| window.selected_tab())
        else {
            return;
        };
        let needle = self.find.text().to_string();
        if needle.is_empty() {
            return;
        }
        let buffer = &tab.buffer;
        if let Some((mut start, mut end)) = buffer.selection_bounds() {
            let selected = buffer.text(&start, &end, false).to_string();
            let same = if self.match_case.is_active() {
                selected == needle
            } else {
                selected.to_lowercase() == needle.to_lowercase()
            };
            if same {
                buffer.begin_user_action();
                buffer.delete(&mut start, &mut end);
                buffer.insert(&mut start, &self.replace.text());
                buffer.end_user_action();
                buffer.place_cursor(&start);
            }
        }
        self.search(true, false);
    }

    /// Replaces every match, as one change Undo takes back.
    fn replace_all(&self) {
        let Some(tab) = self
            .window
            .borrow()
            .upgrade()
            .and_then(|window| window.selected_tab())
        else {
            return;
        };
        let needle = self.find.text().to_string();
        if needle.is_empty() {
            return;
        }
        let replacement = self.replace.text().to_string();
        let buffer = &tab.buffer;
        let flags = self.flags();
        let mut from = buffer.start_iter();
        let mut count = 0;
        buffer.begin_user_action();
        while let Some((mut start, mut end)) = from.forward_search(&needle, flags, None) {
            buffer.delete(&mut start, &mut end);
            buffer.insert(&mut start, &replacement);
            from = start;
            count += 1;
        }
        buffer.end_user_action();
        if count == 0 {
            self.find.add_css_class("error");
        } else {
            self.find.remove_css_class("error");
        }
    }
}
