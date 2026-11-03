//! Notepad's settings, kept in ~/.config/patrone-notepad/settings.ini and
//! shared by every window.

use std::path::PathBuf;

use gtk::glib;

/// PatroneOS's monospace font (fonts-patroneos).
pub const DEFAULT_FONT: &str = "PatroneOS Mono";
pub const DEFAULT_FACE: &str = "Regular";
pub const DEFAULT_SIZE: u32 = 11;
/// The sizes the font size list offers, in points.
pub const SIZES: [u32; 16] = [8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub const ALL: [Theme; 3] = [Theme::Light, Theme::Dark, Theme::System];

    pub fn label(self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::System => "Same as the desktop",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }
}

/// Where files opened from elsewhere (File Manager, the command line) go.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpenIn {
    NewTab,
    NewWindow,
}

impl OpenIn {
    pub const ALL: [OpenIn; 2] = [OpenIn::NewTab, OpenIn::NewWindow];

    pub fn label(self) -> &'static str {
        match self {
            OpenIn::NewTab => "In a new tab",
            OpenIn::NewWindow => "In a new window",
        }
    }

    fn key(self) -> &'static str {
        match self {
            OpenIn::NewTab => "tab",
            OpenIn::NewWindow => "window",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Settings {
    pub theme: Theme,
    pub font_family: String,
    /// The font's face, by the name its family gives it ("Regular", "Bold").
    pub font_face: String,
    pub font_size: u32,
    pub word_wrap: bool,
    pub status_bar: bool,
    pub open_in: OpenIn,
    /// Keep the open tabs and their unsaved changes when Notepad closes,
    /// and bring them back when it starts. Off, Notepad starts with an
    /// empty tab and asks to save changes when it closes.
    pub restore_session: bool,
    pub match_case: bool,
    pub wrap_around: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: Theme::System,
            font_family: DEFAULT_FONT.into(),
            font_face: DEFAULT_FACE.into(),
            font_size: DEFAULT_SIZE,
            word_wrap: true,
            status_bar: true,
            open_in: OpenIn::NewTab,
            restore_session: true,
            match_case: false,
            wrap_around: true,
        }
    }
}

fn file() -> PathBuf {
    glib::user_config_dir()
        .join("patrone-notepad")
        .join("settings.ini")
}

impl Settings {
    pub fn load() -> Settings {
        let file = glib::KeyFile::new();
        let _ = file.load_from_file(self::file(), glib::KeyFileFlags::NONE);
        Settings::from_key_file(&file)
    }

    fn from_key_file(file: &glib::KeyFile) -> Settings {
        let default = Settings::default();
        let string =
            |group: &str, key: &str| file.string(group, key).ok().map(|value| value.to_string());
        let flag =
            |group: &str, key: &str, default: bool| file.boolean(group, key).unwrap_or(default);
        Settings {
            theme: string("appearance", "theme")
                .and_then(|key| Theme::ALL.into_iter().find(|theme| theme.key() == key))
                .unwrap_or(default.theme),
            font_family: string("font", "family")
                .filter(|family| !family.is_empty())
                .unwrap_or(default.font_family),
            font_face: string("font", "face")
                .filter(|face| !face.is_empty())
                .unwrap_or(default.font_face),
            font_size: file
                .integer("font", "size")
                .ok()
                .and_then(|size| u32::try_from(size).ok())
                .filter(|size| (4..=400).contains(size))
                .unwrap_or(default.font_size),
            word_wrap: flag("view", "word-wrap", default.word_wrap),
            status_bar: flag("view", "status-bar", default.status_bar),
            open_in: string("files", "open-in")
                .and_then(|key| OpenIn::ALL.into_iter().find(|open_in| open_in.key() == key))
                .unwrap_or(default.open_in),
            restore_session: flag("session", "restore", default.restore_session),
            match_case: flag("find", "match-case", default.match_case),
            wrap_around: flag("find", "wrap-around", default.wrap_around),
        }
    }

    fn to_key_file(&self) -> glib::KeyFile {
        let file = glib::KeyFile::new();
        file.set_string("appearance", "theme", self.theme.key());
        file.set_string("font", "family", &self.font_family);
        file.set_string("font", "face", &self.font_face);
        file.set_integer("font", "size", self.font_size as i32);
        file.set_boolean("view", "word-wrap", self.word_wrap);
        file.set_boolean("view", "status-bar", self.status_bar);
        file.set_string("files", "open-in", self.open_in.key());
        file.set_boolean("session", "restore", self.restore_session);
        file.set_boolean("find", "match-case", self.match_case);
        file.set_boolean("find", "wrap-around", self.wrap_around);
        file
    }

    pub fn save(&self) {
        let path = file();
        if let Some(folder) = path.parent() {
            let _ = std::fs::create_dir_all(folder);
        }
        if let Err(error) = self.to_key_file().save_to_file(&path) {
            eprintln!("notepad: couldn't save {}: {error}", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_back_what_it_writes() {
        let settings = Settings {
            theme: Theme::Dark,
            font_family: "Noto Sans Mono".into(),
            font_face: "Bold".into(),
            font_size: 14,
            word_wrap: false,
            status_bar: false,
            open_in: OpenIn::NewWindow,
            restore_session: false,
            match_case: true,
            wrap_around: false,
        };
        assert_eq!(Settings::from_key_file(&settings.to_key_file()), settings);
    }

    #[test]
    fn starts_with_its_defaults() {
        let settings = Settings::from_key_file(&glib::KeyFile::new());
        assert_eq!(settings, Settings::default());
        assert!(settings.restore_session);
        assert_eq!(settings.font_family, "PatroneOS Mono");
    }
}
