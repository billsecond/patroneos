// A window of its own, with no console window behind it, when it's started
// on Windows
#![cfg_attr(windows, windows_subsystem = "windows")]

//! Notepad: plain text in tabs, for PatroneOS. Tabs across the title bar,
//! File, Edit and View menus, find and replace, go to line, word wrap,
//! zoom, a font of your choice, a light or dark look, and a status bar.
//! Markdown files show formatted, with a formatting bar, and save as the
//! Markdown they are.
//! By default it keeps the open tabs and their unsaved changes when it
//! closes and brings them back the next time (Settings turns that off).
//!
//!   notepad [--new-window] [FILE...]
//!   patrone-notepad [--new-window] [FILE...]

mod app;
mod dialogs;
mod document;
mod find;
mod formatted;
mod live_style;
mod markdown;
mod print;
mod session;
mod settings;
mod settings_page;
mod tab;
mod window;

use gtk::prelude::*;
use gtk::{gio, glib};

const USAGE: &str = "Usage: notepad [--new-window] [FILE...]\n\
\n\
Opens each FILE in a tab of the Notepad window last used, or in a new\n\
window with --new-window (or when Settings says files open in new windows).\n";

fn main() -> glib::ExitCode {
    let application = gtk::Application::builder()
        .application_id(app::APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();
    application.connect_startup(app::startup);
    // Started through D-Bus with nothing to open
    application.connect_activate(|_| app::app().command_line(Vec::new(), false));
    application.connect_command_line(|_, command_line| {
        let mut files = Vec::new();
        let mut new_window = false;
        let mut options = true;
        for argument in command_line.arguments().iter().skip(1) {
            let text = argument.to_string_lossy();
            match text.as_ref() {
                "--help" | "-h" if options => {
                    command_line.print_literal(USAGE);
                    return glib::ExitCode::SUCCESS;
                }
                "--new-window" if options => new_window = true,
                "--" if options => options = false,
                _ => files.push(command_line.create_file_for_arg(argument)),
            }
        }
        app::app().command_line(files, new_window);
        glib::ExitCode::SUCCESS
    });
    application.run()
}
