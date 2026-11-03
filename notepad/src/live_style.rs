//! Patrone's theme and title bar button style, followed while the app runs.
//! GTK reads the account's gtk.css once, when an app starts, so on its own
//! an open window never sees a new choice in Patrone Settings. Here the app
//! loads the style sheets gtk.css names itself, and again whenever gtk.css
//! changes. Their rules say `window:not(.patrone-live)`, and this app's
//! windows have that class, so what GTK read at the start doesn't touch
//! them; the copies loaded here have that taken out. (settings/README.md,
//! "How styles reach apps". Patrone's other apps have this same file.)

use std::fs;
use std::path::PathBuf;

use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

const CLASS: &str = "patrone-live";
const SCOPE: &str = ":not(.patrone-live)";
const BEGIN: &str = "/* Patrone Settings: theme ";
const END: &str = "/* Patrone Settings: end of theme */";
const IMPORT: &str = "@import url(\"file://";

/// The style sheets gtk.css imports between Patrone Settings' marked lines.
fn sheets(gtk_css: &str) -> Vec<PathBuf> {
    let Some(begin) = gtk_css.find(BEGIN) else {
        return Vec::new();
    };
    let Some(end) = gtk_css[begin..].find(END) else {
        return Vec::new();
    };
    gtk_css[begin..begin + end]
        .lines()
        .filter_map(|line| line.strip_prefix(IMPORT)?.strip_suffix("\");"))
        .map(PathBuf::from)
        .collect()
}

/// The CSS for this app's windows: the sheets gtk.css names, for them.
fn css_for(gtk_css: &str) -> String {
    sheets(gtk_css)
        .iter()
        .filter_map(|sheet| fs::read_to_string(sheet).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .replace(SCOPE, "")
}

fn mark(windows: &gio::ListModel) {
    for index in 0..windows.n_items() {
        if let Some(window) = windows.item(index).and_downcast::<gtk::Widget>() {
            window.add_css_class(CLASS);
        }
    }
}

/// Keeps every window of this app in the theme and button style gtk.css
/// names. Call it once, when the app starts.
pub fn follow(display: &gdk::Display) {
    let windows = gtk::Window::toplevels();
    mark(&windows);
    windows.connect_items_changed(|windows, _, _, _| mark(windows));

    let provider = gtk::CssProvider::new();
    // The sheets' dark rules follow the desktop's light or dark style
    gtk::Settings::for_display(display)
        .bind_property(
            "gtk-interface-color-scheme",
            &provider,
            "prefers-color-scheme",
        )
        .sync_create()
        .build();
    // Just under gtk.css, so the account's own rules in it win here too
    gtk::style_context_add_provider_for_display(
        display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER - 1,
    );

    let path = glib::user_config_dir().join("gtk-4.0").join("gtk.css");
    let load = {
        let path = path.clone();
        move || provider.load_from_string(&css_for(&fs::read_to_string(&path).unwrap_or_default()))
    };
    load();
    let Ok(monitor) = gio::File::for_path(&path)
        .monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
    else {
        return;
    };
    monitor.connect_changed(move |_, _, _, event| {
        // A file part-way through being written shows as Changed, then
        // ChangesDoneHint when it's whole
        if event != gio::FileMonitorEvent::Changed {
            load();
        }
    });
    // It watches for as long as the app runs
    std::mem::forget(monitor);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_sheets_between_the_marked_lines() {
        let text = "@import url(\"file:///mine.css\");\n\
                    /* Patrone Settings: theme (Patrone Settings changes these lines; your own go below) */\n\
                    @import url(\"file:///usr/share/patrone/themes/classic/gtk-4.0.css\");\n\
                    @import url(\"file:///usr/share/patrone/titlebuttons/dots/gtk-4.0.css\");\n\
                    /* Patrone Settings: end of theme */\n\
                    @import url(\"file:///also-mine.css\");\n";
        assert_eq!(
            sheets(text),
            [
                PathBuf::from("/usr/share/patrone/themes/classic/gtk-4.0.css"),
                PathBuf::from("/usr/share/patrone/titlebuttons/dots/gtk-4.0.css"),
            ]
        );
        assert!(sheets("button { margin: 1px; }\n").is_empty());
    }
}
