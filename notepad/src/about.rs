//! About: Notepad's version and licence, and the licences of what it's
//! built with, with their full texts where the installed app has them.

use std::io::Read;
use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::app::{APP_ID, APP_NAME};

/// What Notepad is built with, and under which licence.
const COMPONENTS: &[(&str, &str)] = &[
    ("GTK 4", "LGPL-2.1-or-later"),
    ("GLib and GIO", "LGPL-2.1-or-later"),
    ("Pango", "LGPL-2.1-or-later"),
    ("Cairo", "LGPL-2.1 or MPL-1.1"),
    ("HarfBuzz", "MIT"),
    ("gdk-pixbuf", "LGPL-2.1-or-later"),
    ("graphene", "MIT"),
    (
        "gtk-rs (the Rust bindings: gtk4, glib, gio, pango, cairo-rs)",
        "MIT",
    ),
    ("pulldown-cmark (Markdown)", "MIT"),
    ("Rust's standard library", "MIT or Apache-2.0"),
    (
        "PatroneOS Sans, Mono, Document, Serif and Classic",
        "SIL Open Font License 1.1",
    ),
];

/// The app's own folder (the Windows build's), above its `bin`.
fn app_folder() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.parent()?.to_path_buf())
}

/// The Rust crates' licence texts: the package's
/// `/usr/share/doc/patrone-notepad/third-party-licenses.txt.gz`, or the
/// Windows build's `licenses/rust-crates.txt`.
fn crate_licences() -> Option<String> {
    let folder = app_folder()?;
    let plain = folder.join("licenses").join("rust-crates.txt");
    if let Ok(text) = std::fs::read_to_string(&plain) {
        return Some(text);
    }
    let packed = folder
        .join("share")
        .join("doc")
        .join("patrone-notepad")
        .join("third-party-licenses.txt.gz");
    let file = gio::File::for_path(packed);
    let input = file.read(gio::Cancellable::NONE).ok()?;
    let decompressor = gio::ZlibDecompressor::new(gio::ZlibCompressorFormat::Gzip);
    let stream = gio::ConverterInputStream::new(&input, &decompressor);
    let mut text = String::new();
    stream.into_read().read_to_string(&mut text).ok()?;
    Some(text)
}

/// Where the other licences are, on this system.
fn where_the_rest_are() -> String {
    if cfg!(windows) {
        let folder = app_folder()
            .map(|folder| folder.join("licenses").display().to_string())
            .unwrap_or_else(|| "licenses".into());
        format!(
            "The GTK libraries in this app's folder come from MSYS2, unchanged and linked as \
             DLLs. Their licences, the fonts' licence (OFL.txt), and where each library's \
             source is (SOURCES.txt) are in {folder}."
        )
    } else {
        "GTK and the libraries it uses come from PatroneOS's own packages, linked as shared \
         libraries; each package's licence is in /usr/share/doc/PACKAGE/copyright, and its \
         source in the archive (apt-get source PACKAGE). The fonts' licence is \
         /usr/share/doc/fonts-patroneos/OFL.txt."
            .into()
    }
}

fn licence_text() -> String {
    let mut text = format!(
        "{APP_NAME} is free software: you can redistribute it and/or modify it under the \
         terms of the GNU General Public License as published by the Free Software \
         Foundation, either version 3 of the License, or (at your option) any later version. \
         It is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; \
         without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR \
         PURPOSE. See https://www.gnu.org/licenses/gpl-3.0.html.\n\n\
         It's built with these, each under its own licence:\n\n"
    );
    for (name, licence) in COMPONENTS {
        text.push_str(&format!("• {name}: {licence}\n"));
    }
    text.push_str(&format!("\n{}\n\n", where_the_rest_are()));
    match crate_licences() {
        Some(crates) => text.push_str(&crates),
        None => text.push_str(
            "The Rust crates' licence texts are installed with the app (third-party-licenses.txt.gz \
             in its documentation folder); this copy of it was built without them.",
        ),
    }
    text
}

/// The About window.
pub fn show(parent: &impl IsA<gtk::Window>) {
    let dialog = gtk::AboutDialog::builder()
        .transient_for(parent)
        .modal(true)
        .program_name(APP_NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .logo_icon_name(APP_ID)
        .comments("Plain text and Markdown, in tabs. Part of PatroneOS.")
        .website("https://github.com/billsecond/patroneos")
        .website_label("Source code")
        .copyright("© 2026 William Daugherty")
        .license_type(gtk::License::Custom)
        .wrap_license(true)
        .default_width(640)
        .default_height(620)
        .build();
    dialog.add_credit_section(
        "Built with",
        &COMPONENTS.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
    );
    // The texts are long: read once the window's up
    let weak = dialog.downgrade();
    glib::idle_add_local_once(move || {
        if let Some(dialog) = weak.upgrade() {
            dialog.set_license(Some(&licence_text()));
        }
    });
    dialog.present();
}
