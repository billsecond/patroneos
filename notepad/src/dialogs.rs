//! The questions Notepad asks, and Go to line.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};

use crate::document::Encoding;
use crate::tab::Tab;

pub enum SaveChoice {
    Save,
    DontSave,
    Cancel,
}

/// "Do you want to save changes to …?" Save, Don't save, or Cancel.
pub async fn ask_to_save(parent: &gtk::ApplicationWindow, title: &str) -> SaveChoice {
    let dialog = gtk::AlertDialog::builder()
        .modal(true)
        .message(format!("Do you want to save changes to {title}?"))
        .build();
    dialog.set_buttons(&["Save", "Don't save", "Cancel"]);
    dialog.set_default_button(0);
    dialog.set_cancel_button(2);
    match dialog.choose_future(Some(parent)).await {
        Ok(0) => SaveChoice::Save,
        Ok(1) => SaveChoice::DontSave,
        _ => SaveChoice::Cancel,
    }
}

/// Some characters can't be written in the file's encoding: save it as
/// UTF-8 (`Some(true)`), save it anyway with question marks for them
/// (`Some(false)`), or don't save (`None`).
pub async fn ask_unwritable(
    parent: &gtk::ApplicationWindow,
    encoding: Encoding,
    count: usize,
) -> Option<bool> {
    let characters = if count == 1 {
        "1 character".to_string()
    } else {
        format!("{count} characters")
    };
    let dialog = gtk::AlertDialog::builder()
        .modal(true)
        .message(format!(
            "{characters} can't be saved in {}",
            encoding.label()
        ))
        .detail("Saved in this encoding, each becomes a question mark. UTF-8 keeps them all.")
        .build();
    dialog.set_buttons(&["Save as UTF-8", "Save anyway", "Cancel"]);
    dialog.set_default_button(0);
    dialog.set_cancel_button(2);
    match dialog.choose_future(Some(parent)).await {
        Ok(0) => Some(true),
        Ok(1) => Some(false),
        _ => None,
    }
}

pub async fn error(parent: &gtk::ApplicationWindow, message: &str, detail: &str) {
    let dialog = gtk::AlertDialog::builder()
        .modal(true)
        .message(message)
        .detail(detail)
        .build();
    dialog.set_buttons(&["OK"]);
    let _ = dialog.choose_future(Some(parent)).await;
}

/// Go to line: a line number, which the cursor moves to.
pub fn go_to_line(parent: &gtk::ApplicationWindow, tab: &Rc<Tab>) {
    let buffer = &tab.buffer;
    let lines = buffer.line_count();
    let current = buffer.iter_at_mark(&buffer.get_insert()).line() + 1;
    let dialog = gtk::Window::builder()
        .title("Go to line")
        .modal(true)
        .transient_for(parent)
        .resizable(false)
        .default_width(340)
        .build();
    dialog.add_css_class("go-to-dialog");
    let entry = gtk::Entry::builder()
        .text(current.to_string())
        .input_purpose(gtk::InputPurpose::Digits)
        .activates_default(true)
        .build();
    let problem = gtk::Label::builder()
        .xalign(0.0)
        .wrap(true)
        .css_classes(["go-to-problem"])
        .visible(false)
        .build();
    let cancel = gtk::Button::with_label("Cancel");
    let go = gtk::Button::builder()
        .label("Go to")
        .css_classes(["suggested-action"])
        .build();
    let buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .homogeneous(true)
        .build();
    buttons.append(&go);
    buttons.append(&cancel);
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .css_classes(["go-to-content"])
        .build();
    content.append(
        &gtk::Label::builder()
            .label("Line number")
            .xalign(0.0)
            .build(),
    );
    content.append(&entry);
    content.append(&problem);
    content.append(&buttons);
    dialog.set_child(Some(&content));
    dialog.set_default_widget(Some(&go));

    let tab = tab.clone();
    let weak_dialog = dialog.downgrade();
    let typed = entry.clone();
    go.connect_clicked(move |_| {
        let Some(dialog) = weak_dialog.upgrade() else {
            return;
        };
        match typed.text().trim().parse::<i32>() {
            Ok(line) if (1..=lines).contains(&line) => {
                if let Some(iter) = tab.buffer.iter_at_line(line - 1) {
                    tab.buffer.place_cursor(&iter);
                    tab.view
                        .scroll_to_mark(&tab.buffer.get_insert(), 0.1, false, 0.0, 0.0);
                }
                dialog.close();
                tab.view.grab_focus();
            }
            _ => {
                problem.set_label(&if lines == 1 {
                    "There's only line 1.".to_string()
                } else {
                    format!("Type a line number from 1 to {lines}.")
                });
                problem.set_visible(true);
                typed.grab_focus();
                typed.select_region(0, -1);
            }
        }
    });
    let weak_dialog = dialog.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(dialog) = weak_dialog.upgrade() {
            dialog.close();
        }
    });
    let keys = gtk::EventControllerKey::new();
    let weak_dialog = dialog.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape
            && let Some(dialog) = weak_dialog.upgrade()
        {
            dialog.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    dialog.add_controller(keys);
    dialog.present();
    entry.grab_focus();
    entry.select_region(0, -1);
}
