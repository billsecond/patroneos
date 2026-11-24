# Patrone File Manager

The file manager of PatroneOS ("File Manager" in menus), written in
Rust with GTK4. It works the way Windows users expect, with the layout,
right-click menus, options and keyboard shortcuts they know, in Patrone's
own look and colors. App ID
`org.patrone.FileManager`, command `patrone-file-manager`, package
`patrone-file-manager`.

## What it does

- **Window buttons:** GTK's own minimize, maximize and close, as in every
  other app, so they go in the corner the account chose (Settings' Patrone
  page) and look the way its title bar button style says. (They were drawn
  here, Windows 11's, and so ignored both.)
- **Window:** tabs in the title bar (new, close, middle-click to close, a
  right-click menu with Duplicate, Move to new window, Move to window ▸,
  Close other tabs and Close tabs to the right). The title bar, and the
  dialogs', take the title bar style chosen in Patrone Settings like every
  other app's (raised when active, flat when not): they're `headerbar`
  nodes for style sheets (`util::header_box`), and `data/over-theme.css`
  keeps the tabs readable on any style. Back, Forward, Up and
  Refresh (and the mouse's own Back and Forward buttons); going back or up
  by any route selects the folder you came out of, scrolled into view and
  with the keyboard focus, as on Windows; breadcrumbs of the whole path from Computer (Computer › System drive
  (/) › home › you › folder), any of which goes there when clicked, with
  chevrons listing each folder's subfolders; when the path doesn't fit, its
  start folds into a « button that lists it, as on Windows. Clicking the
  empty space turns it into a path field. "Previous locations"; the search
  box.
- **Dragging tabs**, as in Windows 11: a window's only tab moves the window,
  like the title bar. With several tabs, dragging along the strip reorders
  them; pulling a tab out of the strip (or holding Ctrl) picks it up, and
  dropping it on another File Manager window's title bar adds it there
  (a blue line shows where), while dropping it anywhere else opens it in a
  window of its own. A window whose last tab leaves closes. Files dragged
  over a tab switch to it; dropped on a tab, they go into its folder.
  Wayland doesn't let apps place windows, so a torn-off tab's window opens
  where GNOME puts new windows, not under the pointer.
- **Commands in the address bar**, as in Windows: what isn't a place runs in
  the folder on screen. Apps with a window of their own start directly
  (`gnome-text-editor notes.txt`); anything else runs in a terminal through
  your shell, so pipes and `sudo` work, and the terminal stays open there.
  Windows' names work too: `cmd`, `powershell` and `wt` open a terminal
  there, `notepad`, `calc`, `taskmgr`, `control` and `regedit` open their
  Patrone counterparts, `explorer` a new window; `cd PLACE` goes there.
- **Command bar:** New (folder, text document, anything in Templates);
  Cut, Copy, Paste, Rename, Share and Delete; Sort, View and Filter; buttons
  for the selection (Empty Recycle Bin, Restore, Extract all, Set as
  background, Map network drive, Mount, Eject or Unmount, Disconnect);
  "See more" (Undo, Redo, Pin to Quick
  access, Copy path, Select all/none, Invert selection, Open in Terminal,
  Properties, Options); and Details.
- **Navigation pane:** Home, Gallery, your home folder; the folders pinned to
  Quick access; Computer with its drives, Network and the Recycle Bin. Folders
  expand (chevrons appear on hover); middle-click opens a new tab; files can
  be dropped on any folder or drive.
- **Pages:** Home (Quick access tiles of pinned and frequent folders,
  Favorites, Recent files), Gallery (every picture and video in Pictures,
  newest first), Computer (drives with usage bars, network drives), the
  Recycle Bin (Original location and Date deleted columns, Restore, Empty).
- **Drives and mounts** (`src/drives.rs`, `src/thispc.rs`): Linux has no
  drive letters; each drive's files appear in a folder of the one tree
  under `/`, its mount. Computer lists every drive, mounted or not: a tile
  says where the drive is mounted ("Mounted at /run/media/..."; the
  system drive at `/`), or its size and "Not mounted". Opening one mounts
  it, as does Mount in its menus, the command bar and the navigation pane;
  Eject (USB drives, discs, disk images) or Unmount (internal drives) puts
  it away. Internal drives ask for an administrator's password once
  (udisks' `filesystem-mount-system`; Patrone Remote Desktop's polkit rule
  gives Patrone's seat-less desktops the same as a desktop at the screen).
  Computer and the navigation pane follow GIO's volume monitor, which
  `drives::monitor()` keeps for the app's life: GIO frees it, and the
  signals connected to it, with its last reference.
  **Map network drive** takes `\\server\share` (a Windows share,
  `smb://`) or any address GIO opens (`sftp://`, `ftp://`, `dav://`,
  `nfs://`...), signs in when the server asks (the password can be
  remembered in the keyring), and with "Reconnect at sign-in" keeps the
  address in `mapped-drives`: `--restore`, run at each sign-in, connects
  it again. A mapped drive keeps its own name and address in Computer's
  Network drives, connected or not, and opens to the folder mapped.
  Disconnect unmounts a network drive and forgets it. **Disk images**
  (`.iso`, `.img`): Mount (right-click, or the command bar) sets the image
  up read-only with `udisksctl loop-setup`, mounts its file system and
  opens it; Eject unmounts it, and udisks detaches the image then (it
  sets loop devices to clear themselves). Not done: VHD/VHDX images, and
  mounting internal drives at sign-in.
  **Mount folders**: Computer and the navigation pane show "Drive mounts",
  the folder the account's drives are mounted in (`/run/media/USER`, as
  Ubuntu 26.04's udisks builds it; `/media/USER` where udisks is built the
  older way; `/media` until the first drive makes it), always, so it's
  learned where drives appear, and "Other mounts" (`/mnt`) when it has
  anything in it. **Network shares as drives**: on the Network page a
  shared folder's menu and the command bar have Mount and Map network
  drive (a computer's has Map network drive, filled in as `\\computer\`).
  GIO's mount is enough for every app: GVfs's FUSE folder
  (`/run/user/UID/gvfs/smb-share:server=...,share=...`) is the drive's
  "Mounted at" path, which programs that don't use GIO (a terminal, `cat`)
  read and write; no root mount into /media is needed, and the password,
  when remembered, is in the keyring.
  Tested as the `claude` account in a private headless GNOME Shell (as in
  Testing without a desktop session below), with a `scsi_debug` disk as
  an internal drive, `.iso` and `.img` files, GVfs's `localtest:` backend
  as a network share, a temporary polkit rule letting `claude` mount, and
  an SMB address with no server for the sign-in dialog.
- **Signing in to mount** (`src/signin.rs`): every mount File Manager
  starts uses its own `gio::MountOperation`, which answers GVfs's
  questions with File Manager's dialogs instead of GTK's (whose words were
  white on light grey in dark mode, with its buttons in a strip of their
  own). A network folder asks as Windows does: "Enter network
  credentials", the computer it's for, User name, Password (with a show
  button), "Remember my credentials" (kept in the keyring by GVfs), and
  Domain and guest sign-in under More choices; a wrong password asks again
  and says so. An encrypted drive asks for its password; other questions
  (an unknown SSH host key) show GIO's words with its choices as buttons.
  The handlers stop the signal's emission: GIO's own handler, which runs
  after them, answers "unhandled" at once. Unmounting still uses GTK's
  operation, for its list of the apps keeping a drive busy. Computers on
  the Network page (GVfs's shortcuts) show as computers, not blank files.
- **Network places without freezing** (`FolderView::load`): GVfs answers
  GtkDirectoryList's watch on a folder synchronously, and the first time a
  place's backend is used (the Network page after signing in: about 10
  seconds on PatroneDev) it starts the backend before answering, with the
  window frozen meanwhile. So a folder that isn't on a local disk is asked
  about asynchronously first, with "Looking for computers and devices on
  the network..." (or "Loading...") showing; GVfs keeps the answer, and the
  watch then returns at once. A location given on the command line is
  checked asynchronously too.
- **Views:** Extra large, Large, Medium and Small icons, List, Details,
  Tiles and Content, plus Compact view. Details has sortable, resizable,
  reorderable columns (right-click a header for more columns). Sort and
  Group by every column, with Windows' groups (Today, Yesterday, Earlier this
  week...; Tiny, Small, Medium... for sizes). Each folder remembers its own
  view, as on Windows; Pictures and Videos open in Large icons, Downloads
  grouped by date.
- **Right-click menus in Windows 11's form:** the icon row (Cut, Copy,
  Rename, Share, Delete), then Open, Open with, Open in new tab and window,
  Pin to Quick access, Add to Favorites, Compress to (ZIP, TAR), Extract
  All… and Extract here, Copy as path, Set as desktop background, Properties, Open in
  Terminal, and "Show more options" for the full classic menu with Send to.
  The background menu has View, Sort by, Group by, Undo, Paste, Paste
  shortcut, New and Properties.
- **Panes:** Details (a large thumbnail and the properties of what's
  selected, or of the folder) and Preview (pictures, text, video and sound,
  and thumbnails of PDFs and other files).
- **Operations** through GIO, so they work on local drives, removable
  drives, network shares and the Recycle Bin: copy, move, delete to the
  Recycle Bin or for good, restore, rename in place (F2; as on Windows,
  Enter, a click anywhere else or switching windows keeps the name and Esc
  cancels), new folder, new file, shortcuts (symbolic links), compress to
  ZIP or TAR (built as a hidden `.NAME.part` and given its name when
  finished) and extract.
  ZIP and TAR extract here; every other archive Patrone Unpack opens (7z,
  rar, tar.xz, tar.zst, cpio, cab, single files such as `notes.txt.gz`)
  goes to it when it's installed: Extract All… runs
  `patrone-unpack --extract-to FOLDER` (File Manager makes the folder and
  shows it while the files arrive), and Extract here runs
  `patrone-unpack --extract-here`, which keeps one folder or file as it is
  and puts anything else in a new folder named after the archive.
  Copies into the same folder become "name - Copy". Folders merge.
  Conflicts are asked about once for the whole operation (Replace, Skip, or
  "Let me decide for each file", where keeping both adds a number). Errors
  offer Try Again, Skip or Cancel. Long operations show Windows' progress
  window with pause, cancel and "More details". **Undo and Redo** (Ctrl+Z,
  Ctrl+Y) cover all of them, shared by every window.
- **Drag and drop** between folders, tabs' breadcrumbs, the navigation pane
  and other apps: a move within a drive, a copy to another drive; Shift
  forces a move, Ctrl a copy.
- **Folder Options** (See more → Options) with Windows' General, View and
  Search tabs: where windows open, same or new window, single- or
  double-click, recent files and frequent folders (and clearing them),
  hidden files, file name extensions, thumbnails, tooltips, the status bar,
  item check boxes, typing to select or to search, restoring windows at
  sign-in, and search options. OK, Cancel, Apply and Restore Defaults behave
  as on Windows.
- **Properties** (Alt+Enter): General (name, type, the app it opens with
  and Change..., location, size and size on disk, what a folder contains,
  dates, Read-only and Hidden), Permissions and Details. Drives show a chart
  of used and free space.
- **Permissions**, Linux's version of Windows' Security tab: the owner and
  group (changeable), what the owner, the group and everyone else may do
  (Read, Write, Run; for folders List contents, Add and delete, Open),
  "Allow running this file as a program", the special permissions (set user
  ID, set group ID, sticky), for folders "Apply to everything inside", and
  the mode in numbers and letters (`0755`, `-rwxr-xr-x`). **Other people and
  groups** get access by name, as with Windows' Add... button: "Give access
  to" adds a row with their own Read, Write and Run boxes (starting, as on
  Windows, with read, and open for a folder or a program) and a button to
  take it away. These are ACLs (`getfacl` and `setfacl` from the `acl`
  package); a folder hands them on to new items inside, and the mode shows
  `+` as `ls` does. A change you may not make yourself is made as
  administrator after the system asks for your password (pkexec).
- **System folder hints** (View → Show, or Folder Options → View), for
  people who know Windows: Details gains a Purpose column saying what system
  folders and well-known files are for (`/etc` "Settings", `/usr/bin`
  "Programs", `~/.config` "App settings"; pointing at it explains more),
  Tiles and Content add it after the type ("File folder · Settings"), the
  other views put it in the tooltip, and inside one a banner explains it and names
  what plays the same part on Windows (`/etc`: the Registry's
  HKEY_LOCAL_MACHINE and `C:\ProgramData`). Permissions then also shows the
  `chmod` and `chown` commands for its settings, with `sudo` when they need
  it. The table is in `src/hints.rs`.
- **System folders have their own icons** wherever they're shown, as
  Windows' do: a gear for `/etc` and `~/.config`, a terminal for the
  program folders, a power button for `/boot`, a chip for `/dev` and `/sys`,
  layers for the libraries, people for `/home`, a shield for `/root`, a
  package for `/opt`, `/usr` and snaps, a clock for `/tmp`, a log for
  `/var/log`, and so on.
- **Search:** names anywhere, ignoring case, with `*` and `?` wildcards,
  through subfolders; optionally inside text files and ZIP files. Results
  stream in, with a Folder column.
- **Light and dark** follow the desktop's preference. Thumbnails follow the
  freedesktop.org thumbnail spec (shared with other apps), made here for
  pictures and by the system's thumbnailers for everything else.
- `org.freedesktop.FileManager1` on D-Bus, so apps can ask it to show a file
  in its folder (a browser's "Show in folder"). On PatroneOS a D-Bus
  service file starts it for that (`--gapplication-service`) in place of
  Nautilus's.
- **On PatroneOS, GNOME's Files (Nautilus) is retired, not removed:**
  removing the package would take Ubuntu's session (`ubuntu-session`) and
  GNOME's file dialogs for Snap and Flatpak apps (`xdg-desktop-portal-gnome`,
  which Nautilus draws) with it. `patrone-default-apps` diverts two of its
  files (`dpkg-divert`, kept as `*.nautilus`): its menu entry, replaced by a
  hidden one that opens no file types (`default-apps/retire-nautilus/`), and
  its `FileManager1` D-Bus service. This package did it itself before
  0.3.11; installed on its own, on Ubuntu, it leaves Files as it is, and
  takes `FileManager1` only while it's running.

Hidden files (names starting with a dot, and names listed in a folder's
`.hidden` file) stay hidden unless Folder Options or View → Show → Hidden
items says otherwise, and show faded when they're shown. Hidden in
Properties adds the name to `.hidden`, which every GTK app honors.

## The desktop

Since 0.3.0 File Manager draws the desktop and, since 0.3.10, the desktop
files panel, as
Explorer draws Windows' desktop (the user, 2026-10-27: "The desktop and the
roll out menu must be an extension of the file explorer app, while
supporting the desktop and all it can do"), in place of Ubuntu's desktop
icons extension and the taskbar's own bubble, as it was. `patrone-file-manager
--desktop` (`src/desktop.rs`) shows the Desktop folder (`xdg-user-dir
DESKTOP`, made if missing) with File Manager's own view, cells, menus,
commands and undo, so the icons, thumbnails, names, selection, renaming,
dragging, conflicts and undo are File Manager's.

- **The taskbar runs it.** A Wayland app can't make its window the desktop
  or keep it above the others, so Patrone's taskbar starts it as a Wayland
  client of its own and finds its windows by their titles ("Patrone
  Desktop", "Patrone Desktop Panel"; `shell/gnome/README.md`, "The desktop
  and the panel"). The two talk over the desktop's standard input and
  output, a line at a time (the list is at the top of `src/desktop.rs`):
  the screen's free area, the Recycle Bin's place and size and whether
  it's on the desktop, whether the files are in the panel (and whether the
  desktop is a clean one) and the panel opening and closing, one way; the item count and requests to open or close it, the
  other. It runs on its own (not on D-Bus, not
  sharing a process with File Manager's windows), so the taskbar can start
  and stop it, and a fault in either never closes the other. Opening a
  folder from the desktop opens a File Manager window; dialogs (conflicts,
  Properties, delete) belong to no window, so GNOME centres them.
- **Two windows.** One as big as the screen, with the icons on it (a
  desktop window: under every other window, on every workspace, no taskbar
  button), and, while the taskbar's `desktop-files` setting has them off
  the desktop, the desktop files panel's, as big as the screen's free area,
  which takes the
  icons and leaves the desktop empty but still taking right-clicks, drops
  and keys. It's clear but for the icons and a small bar (Desktop, how many
  items, Open in File Manager, Close), since the taskbar slides it out with
  the wallpaper under it (`shell/gnome/README.md`, "The desktop files
  panel"), and the icons keep their desktop places (`Layout::Panel`
  differs from the desktop only in always showing them, saying so when
  there are none, and taking drops on the Recycle Bin). The view moves
  between the windows (`Window::content`), and the panel's window runs the
  desktop window's commands (`win.*`). New, Paste and drops on the empty
  desktop open the panel on what they made.
- **Icons on the desktop** (`src/desktopgrid.rs`, a widget of its own in
  place of a list view): down the left side first, then the next column,
  as on Windows; each stays where it's put (dragged about the desktop, or
  dropped there from elsewhere) unless Auto arrange is on. Places are kept
  by name in the `desktop-positions` setting, as columns and rows, so they
  survive a change of icon size; a renamed icon keeps its place. None goes
  under the Recycle Bin. Names are white with a shadow over the wallpaper;
  selection is File Manager's blue, stronger so it shows on a dark picture.
- **Selecting:** click, Ctrl+click, Shift+click, a rubber band from empty
  space (with Ctrl, adding), the arrow keys, Home and End, Ctrl+Space, and
  typing a name's start. Enter or a double-click opens (one click with
  Folder Options' single-click).
- **Right-click on the desktop**, as Windows 11's: View (Large, Medium or
  Small icons, Auto arrange icons, Show desktop icons), Sort by (Name, Size,
  Item type, Date modified; with Auto arrange off it arranges the icons
  once, as on Windows), Refresh, Paste, Paste shortcut, Undo and Redo, New
  (Folder, Shortcut, Text Document and the Templates folder's files),
  Show Recycle Bin while it's off the desktop, Display settings (GNOME
  Settings' Displays), Personalize (Patrone Settings' Wallpaper page, else
  GNOME's Appearance) and Open in Terminal. With the panel on, Show desktop
  files (while it's closed) and Open Desktop folder come first, with Open
  Recycle Bin on a clean desktop, which has no other way to it on screen,
  and Show desktop icons is left out (the panel always shows them). On an
  icon, File Manager's own menu (without Open in new tab).
- **Keys:** File Manager's, from the desktop or the panel: Delete,
  Shift+Delete, F2, Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V, Ctrl+Z, Ctrl+Y,
  Ctrl+Shift+N, F5, Alt+Enter, Ctrl+Shift+C, Shift+F10 and the Menu key.
  Ctrl+mouse wheel changes the icon size. Escape closes the panel.
- **Dragging** works between the desktop (or the panel) and File Manager
  both ways with Windows' rules (`src/dnd.rs`): a move on the same drive, a
  copy to another, Shift to move and Ctrl to copy. Holding Ctrl or Shift
  over a window that isn't in front reaches it through GNOME, which picks
  the drag's action from the keys and hands it to the drop
  (`dnd::drop_action`); before, a window that didn't have the keyboard
  never saw them. Files held over the panel's pill for half a second open
  the panel, to drop them in, or close it, for the windows under it.
  Dropped on the Recycle Bin riding on the panel, they go to the Recycle
  Bin: its own drop target stays under the panel while it's there, so the
  panel takes those drops itself (the bin lies inside the panel's window).
  Anything File Manager drags carries
  the files' addresses, so other apps take them too (Text Editor opens a
  text file dropped on it).
- **Launchers** (`.desktop` files), as Ubuntu's desktop icons did them: one
  the account allowed to run (trusted and executable) shows its app's icon
  and name and starts the app; any other shows as a file, and opening it
  asks first (Allow and Run, Open as Text, Cancel). Allow Launching in its
  right-click menu turns that on or off (`metadata::trusted`, and the
  owner's run bit).

## Keyboard

| Keys | Action |
| --- | --- |
| Ctrl+T / Ctrl+W / Ctrl+N | New tab / close tab / new window |
| Ctrl+Tab, Ctrl+Shift+Tab, Ctrl+1 to 9 | Switch tabs |
| Alt+Left / Alt+Right / Alt+Up, Backspace | Back / Forward / Up, Back |
| Mouse Back / Forward buttons | Back / Forward |
| F5 or Ctrl+R | Refresh |
| Ctrl+L, Alt+D or F4 | Type a path in the address bar |
| Ctrl+E, Ctrl+F or F3 | Search |
| Ctrl+Shift+N | New folder |
| F2 | Rename |
| Delete or Ctrl+D / Shift+Delete | Delete to the Recycle Bin / delete for good |
| Ctrl+C / Ctrl+X / Ctrl+V | Copy / cut / paste |
| Ctrl+Z / Ctrl+Y | Undo / redo |
| Ctrl+A | Select all |
| Ctrl+Shift+C | Copy as path |
| Alt+Enter | Properties |
| Ctrl+Shift+1 to 8, Ctrl+mouse wheel | Change the view |
| Alt+P / Alt+Shift+P | Preview pane / details pane |
| Ctrl+Shift+E | Expand the navigation pane to the open folder |
| Shift+F10 or the Menu key | Right-click menu |
| F11 | Full screen |
| Ctrl while dragging a tab | Pick the tab up, even a window's only one |

Typing in a file list selects the item that starts with what was typed.

## Settings

Settings live in GSettings (`org.patrone.FileManager`, see
`data/org.patrone.FileManager.gschema.xml`), so Patrone or an administrator
can change the defaults for everyone with a `.gschema.override` file, and
`gsettings` can read and change them. Folder Options writes only what was
changed. Each folder's own view is kept in
`~/.local/share/patrone-file-manager/folder-views.ini`, and the visit
counts behind "frequent folders" in `history.ini` beside it. Pinned folders
and Favorites are settings (`pinned-folders`, `favorite-files`). The
desktop's View and Sort by are `desktop-icon-size`, `desktop-auto-arrange`,
`desktop-show-icons` and `desktop-sort`, its icons' places
`desktop-positions`.

## Icons

Every icon is original artwork drawn for Patrone; nothing is copied from
Microsoft. The color icons (folders,
file types, drives, places) are SVGs in `data/icons`, drawn by
`tools/make_icons.py`; `tools/preview_icons.py` renders them all on light
and dark backgrounds for checking. The app's own icon,
`data/org.patrone.FileManager.svg`, is drawn by hand (not by the script):
a folder in Patrone's blues, its tab on the right and its front cut on the
slant of the Patrone P (since 0.3.6; it was a yellow folder with a blue
band, File Explorer's shape, until the compliance review, item 15). The
taskbar's Files pin is a copy of it
(`shell/gnome/patrone-taskbar@patroneos.com/icons/pin-files.svg`), so a new
app icon goes there too. The line glyphs of the command bar and
menus are vector paths in `src/glyph.rs`, drawn in the text color so they
follow light and dark mode. The text is the desktop's font (Patrone
Settings → Theme → Font: PatroneOS Sans or Noto Sans), at 9 points;
monospace is PatroneOS Mono. File lists draw
their icons at 20 pixels, which with the artwork's margins makes the item
itself about Windows' 16.

## Build and try it

On the dev VM (GTK 4.22 and the GLib tools come from the desktop and
`scripts/bootstrap-dev-vm.sh`):

```
cargo run --manifest-path file-manager/Cargo.toml -- ~/Documents
cargo test --manifest-path file-manager/Cargo.toml
```

`cargo run` uses the settings schema compiled into the build folder, so
nothing has to be installed. To install it for your user, with a launcher
entry, icon and settings:

```
./file-manager/install-dev.sh             # the app
./file-manager/install-dev.sh --default   # and open folders with it, and put it in the dock, in place of Files
```

The package is built with `packaging/build.sh patrone-file-manager -d` (see
`packaging/README.md`). `patrone-default-apps` makes it the app for folders
system-wide (`inode/directory` in `/etc/xdg/mimeapps.list`) and puts it in
the favorites in Files' place, so the taskbar's Files button opens it.

Command line: `patrone-file-manager [--new-window] [--select] [LOCATION...]`;
a file opens in its app, `--select` shows items selected in their folders.
Every window shares one process. `--desktop` draws the desktop, in a
process of its own; only the taskbar starts it.

## Testing without a desktop session

- **GTK's Broadway backend** shows the app in a web page, with working mouse
  clicks, menus and dialogs, and single key presses (letters typed as text,
  Ctrl+C and Ctrl+W don't arrive; browsers keep those). Run
  `gtk4-broadwayd --port 8087 :7`, start the app with
  `GDK_BACKEND=broadway BROADWAY_DISPLAY=:7 dbus-run-session -- patrone-file-manager`,
  and open `http://localhost:8087` (tunnel the port over SSH from Windows).
- **A screenshot under Wayland:** inside a throwaway
  `dbus-run-session bwrap --die-with-parent --dev-bind / / --bind EMPTY /usr/share/gnome-shell/extensions/ding@rastersoft.com gnome-shell --headless --virtual-monitor 1600x1000 --wayland --no-x11 --wayland-display NAME`
  (EMPTY is an empty folder; it hides Ubuntu's desktop icons, whose
  start-up ends every desktop-icons process of the account and so
  restarted the real desktop's icons, as `shell/gnome/tests/headless.sh`
  found), run the app with `WAYLAND_DISPLAY=NAME PATRONE_FM_SCREENSHOT=out.png`; it
  draws its first window into `out.png` after
  `PATRONE_FM_SCREENSHOT_DELAY` milliseconds (2000) and quits.
  `PATRONE_FM_COLOR_SCHEME=dark` or `light` forces a color scheme. With
  `PATRONE_FM_SCREENSHOT_DIALOG=1` it draws a dialog open by then instead
  (open Properties with `gdbus call --session --dest org.patrone.FileManager
  --object-path /org/freedesktop/FileManager1 --method
  org.freedesktop.FileManager1.ShowItemProperties "['file:///etc']" ""`),
  and `PATRONE_FM_SCREENSHOT_PAGE=N` shows its Nth tab first.
- **Real mouse and keyboard input:** in that headless GNOME Shell, Mutter's
  RemoteDesktop and ScreenCast D-Bus APIs move and click the pointer
  (`NotifyPointerMotionAbsolute` takes the stream's path as a string),
  type, and stream the screen to PipeWire, where
  `gst-launch-1.0 pipewiresrc path=NODE ! videoconvert ! pngenc snapshot=true ! filesink`
  saves a screenshot. That's how tab dragging was tried: on GNOME's Wayland a
  drag dropped where nothing accepts it ends as an error, not "no target".
- **Settings without dconf:** `GSETTINGS_BACKEND=keyfile` with
  `XDG_CONFIG_HOME` pointing at a scratch folder reads
  `glib-2.0/settings/keyfile` there.
- Test runs in a private D-Bus session get their own dconf service, which
  fights the desktop's over `~/.config/dconf/user`. Give them their own
  database: `DCONF_PROFILE=` a file containing `user-db:pfm-test`.
- **The desktop** needs the taskbar to host it: run
  `shell/gnome/tests/headless.sh start` with a folder holding a
  `patrone-file-manager` link to the build first on `PATH`, and the
  taskbar in that shell starts it (a copy of the script with its own
  `base` and `display` keeps clear of anyone else's test shell). Its
  `drag`, `click` and `eval` drive it; a drag with a key held, or one that
  pauses on the way, is a few more lines of the same `eval`. The shell's
  Desktop folder is in `/tmp`, where GIO can't use the Recycle Bin.

## Not done yet

- Group headers show in Details and Content; the icon views and List sort by
  group but don't show the headers (GTK's grid view has no sections).
- A narrow window clips the command bar instead of moving buttons into
  "See more".
- Windows' Gallery timeline and collections, OneDrive-style cloud states,
  libraries, and indexed search (search reads the disk each time).
- Folders don't show previews of their contents on their icons.
- The desktop: icons on the primary screen only (other screens show the
  wallpaper and GNOME's own menu); Windows' "Align icons to grid" off (free
  placement); undo shared with File Manager's windows (each keeps its own:
  Ctrl+Z on the desktop undoes what was done there, a drop from a window
  included); the name editor growing to more lines for a long name.
