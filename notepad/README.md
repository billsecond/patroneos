# Notepad

PatroneOS's plain text editor, packaged as `patrone-notepad`: a Rust and
GTK 4 app (`org.patrone.Notepad`), and the default app for plain text
(`patrone-default-apps`), with GNOME's Text Editor as the fallback.

```
notepad [--new-window] [FILE...]
patrone-notepad [--new-window] [FILE...]
```

It's one process for every window: opening a file while Notepad's running
opens it in a tab of the window last used (or a new window, if Settings
says so, or with `--new-window`). A file that's already open shows its
tab. A file that isn't there yet opens empty, and saving makes it.

## The window

- **Tabs** across the title bar, with **+** for a new one. A tab with
  unsaved changes shows a dot where its close button is, and the close
  button again when the pointer's over it. Drag a tab to move it along;
  with only one tab, dragging it moves the window. A middle click closes
  a tab. Closing the last tab closes the window.
- **File, Edit and View** menus, with **Settings** (the gear) at the end of
  their row:
  - File: New tab (Ctrl+N or Ctrl+T), New window (Ctrl+Shift+N), Open
    (Ctrl+O, several files at once), Save (Ctrl+S), Save as
    (Ctrl+Shift+S), Save all (Ctrl+Alt+S), Print (Ctrl+P), Close tab
    (Ctrl+W), Close window (Ctrl+Shift+W), Exit.
  - Edit: Undo (Ctrl+Z), Redo (Ctrl+Y or Ctrl+Shift+Z), Cut, Copy, Paste,
    Delete, Find (Ctrl+F), Find next (F3), Find previous (Shift+F3),
    Replace (Ctrl+H), Go to line (Ctrl+G), Select all (Ctrl+A), Time and
    date (F5), Font (opens Settings).
  - View: Zoom in (Ctrl++), Zoom out (Ctrl+-), Reset zoom (Ctrl+0), also
    Ctrl and the mouse wheel; Status bar; Word wrap.
- **The text**, plain, in PatroneOS Mono at 11 points unless Settings
  says otherwise. Markdown files show formatted (below).
- **The status bar**: the cursor's line and column, the number of
  characters (or how many of them are selected), the tab's zoom, and the
  file's line endings and encoding. Clicking those two changes how the
  file's saved.
- **Find and replace** in a box over the top right of the text. It finds as
  you type; Enter finds the next match and Shift+Enter the one before. The
  arrow shows the Replace row (Replace, Replace all, which Undo takes back
  in one step). The options button has Match case and Wrap around. Esc
  closes it.
- Ctrl+Tab and Ctrl+Shift+Tab (or Ctrl+Page Down and Up) go through the
  tabs. Files dropped on the text open in tabs.

## Markdown

A Markdown file (`.md`, `.markdown` and the like) shows formatted while
you edit it: headings, **bold**, *italic*, ~~strikethrough~~, inline code,
links, bulleted and numbered lists, task lists with boxes you can tick,
quotes, code blocks, tables, rules and images. Body text and headings are
in PatroneOS Sans, code, code blocks and tables in PatroneOS Mono. The
markup is hidden except on the lines being edited, where it shows faintly,
so it can be changed like any other text. Ctrl+click opens a link (a web
address in the browser, a file in its app). Images come from files, by
their path or relative to the document; nothing is fetched from the
network.

**The text doesn't change**: the formatted view only styles it and hides
the markup, so what's saved is the Markdown as typed, and a file opened
and saved comes back byte for byte. `formatted.rs` has a test that opens
every Markdown file in this repository that way and checks the bytes
(`cargo test -- --ignored`, in a desktop session). Copying copies the
Markdown, hidden markup included.

The **formatting bar** under the menus has:

| Button | Keys | What it does |
| --- | --- | --- |
| Heading list | Ctrl+Alt+1 to 6, Ctrl+Alt+0 | The lines as a heading of that level, or body text |
| Bold, Italic, Strikethrough | Ctrl+B, Ctrl+I, Ctrl+Shift+X | Wraps the selection (or the word at the cursor) in `**`, `*` or `~~`, or takes them off |
| Bulleted list, Numbered list | Ctrl+Shift+8, Ctrl+Shift+7 | `- ` or `1. ` on each selected line, or off when they all have it |
| Link | Ctrl+K | Asks for the words and the address: `[words](address)` |
| Clear formatting | Ctrl+\ | Takes the Markdown off the selection (or the line), leaving the words |
| Outline | | The document's headings; choosing one goes to it |

View › Formatted Markdown, or the status bar's Formatted/Markdown switch
(Ctrl+Shift+M), shows the tab's Markdown as typed instead, in the plain
text font. File › Export as HTML saves it as a web page (PatroneOS's fonts,
light styling). Settings › Markdown formatting turns all of it off: Markdown
files are then plain text like any other. A file over a million characters
stays plain.

## Fonts

Notepad names PatroneOS's own fonts (`fonts-patroneos`, which the package
depends on) rather than relying on the system's defaults: PatroneOS Sans
for its menus, tabs, status bar, Settings and dialogs, and for formatted
Markdown; PatroneOS Mono for plain text (the default in Settings) and
Markdown's code. Settings' font list starts with the five PatroneOS
families (Mono, Sans, Document, Serif, Classic), then every other one.

## Settings

Settings opens as a tab of its own. Every change applies at once, in every
window, and is kept in `~/.config/patrone-notepad/settings.ini`.

| Setting | Choices | Default |
| --- | --- | --- |
| Theme | Light, Dark, Same as the desktop | Same as the desktop |
| Font | Any installed family, its styles, and a size from 8 to 72 points, with a preview | PatroneOS Mono, Regular, 11 |
| Word wrap | On or off (also View › Word wrap) | On |
| Markdown formatting | On or off | On |
| Where files open | In a new tab, In a new window | In a new tab |
| Restore open tabs | On or off | On |

The status bar and find options (Match case, Wrap around) are kept there
too.

## Restoring open tabs (auto-save)

With **Restore open tabs** on, Notepad keeps every window's tabs, and
their unsaved changes, as you work: a moment after each change, and when
it closes. They come back the next time it starts, each tab as it was
(still unsaved, with its dot), at its place in the text and its zoom. The
files themselves change only when you save them.

They're kept in `~/.local/state/patrone-notepad/session/`: `session.ini`
lists the windows and tabs, and the text of a tab that has unsaved
changes, or was never saved, is in a file of its own beside it. A tab with
no unsaved changes is read from its file again.

So with it on, closing the last window asks nothing. Closing a tab still
asks about its unsaved changes, and so does closing one window while
another stays open (nothing would keep its tabs). File › Exit keeps every
window.

With it off, Notepad starts with one new tab, and closing a window (or
Exit) asks about each tab with unsaved changes in turn: Save, Don't save
or Cancel. Turning it off deletes what was kept.

## Files

Notepad reads UTF-8 (with or without a byte order mark), UTF-16 (little or
big endian, with its byte order mark) and, for anything else, CP1252, which
every byte can be read as. It notes the line endings the file uses most
(LF, CRLF or CR), shows plain lines, and saves with the same encoding and
line endings, or the ones chosen in the status bar. New tabs are UTF-8 with
LF. Saving in CP1252 text it can't hold asks first: save as UTF-8, save
anyway (those characters become question marks), or cancel.

Files are read and written through GIO, so files on network drives work
too, and a file is replaced whole when it's saved (written beside it, then
moved over it).

## Printing

File › Print prints the tab's text in Notepad's font at its own size (not
the tab's zoom), wrapped to the page, with the file's name at the top of
each page and "Page N of M" at the bottom. GTK's print dialog sets up the
page and can print to a PDF file.

## Building

```
cargo build --release
cargo test
```

It needs GTK 4.22 (`libgtk-4-dev`) and builds with Rust 1.93, Ubuntu
26.04's. The package: `./packaging/build.sh patrone-notepad -d` on the dev
VM (`packaging/README.md`).

## Windows

The same code builds for 64-bit Windows 10 and 11 as **Patrone Notepad**
(`patrone-notepad.exe`; its window titles say Patrone Notepad there too).
It installs for one account, with no administrator rights, in
`%LOCALAPPDATA%\Programs\Patrone Notepad`, with a Start menu entry, and
doesn't change which app opens any kind of file. Its folder has the GTK
runtime it needs (from MSYS2) and PatroneOS's five fonts, which it loads
for itself without installing them (`load_bundled_fonts` in `app.rs`,
through Pango's `add_font_file`). Settings and kept tabs are in
`%LOCALAPPDATA%\patrone-notepad`.

To build it, on Windows:

1. Rust from rustup (https://rustup.rs).
2. MSYS2 (https://www.msys2.org), in `%LOCALAPPDATA%\msys64` for one
   account: unpack `msys2-base-x86_64-DATE.tar.xz` from
   https://github.com/msys2/msys2-installer/releases there (check it
   against its `.sha256`), then bring it up to date, twice:
   `%LOCALAPPDATA%\msys64\usr\bin\bash.exe -lc "pacman -Syuu --noconfirm"`.
   MSYS2's own installer works too (pass its folder as `-Msys2`).
3. From this checkout:
   `powershell -ExecutionPolicy Bypass -File notepad\windows\build.ps1`.
   The first run installs MSYS2's GTK 4, gcc, pkgconf, librsvg, Adwaita
   icons and ntldd, and Rust's `stable-x86_64-pc-windows-gnu` toolchain
   (`-SkipPackages` skips that).

It builds with Rust's `x86_64-pc-windows-gnu` target against MSYS2's
MINGW64 libraries, then puts the program, every DLL it needs (found with
`ntldd`), GLib's `gdbus.exe` (which keeps it one process), gdk-pixbuf's
image loaders, GTK's schemas, the Adwaita and hicolor icons, Notepad's
icon (also as an `.ico`, drawn from the SVG at seven sizes), the PatroneOS
fonts release `fonts/release.txt` pins (checked against its checksum),
and the licences (Notepad's, its Rust crates', each MSYS2 package's, and
where their source is) in one folder laid out like MSYS2's (`bin`, `lib`,
`share`), and zips it with `install.ps1`, `uninstall.ps1` and
`README.txt`: `out\windows\patrone-notepad-VERSION-windows-x86_64.zip`.

To install that zip on a Windows PC: unpack it, run `install.ps1` with
PowerShell (`powershell -ExecutionPolicy Bypass -File .\install.ps1`), and
open Patrone Notepad from the Start menu. `uninstall.ps1` removes it
(`-RemoveSettings` its settings too).

`data/style.css` is the look; `data/over-theme.css` keeps the tabs readable
under Patrone Settings' title bar styles, and `src/live_style.rs` (the same
in Patrone's other apps) follows a new theme or title bar button style from
Patrone Settings while Notepad runs. The icon, `data/org.patrone.Notepad.svg`,
is Patrone's own: a page with a folded corner and lines of text on a
Patrone-blue tile.
