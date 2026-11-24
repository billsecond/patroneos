# Patrone Taskbar (GNOME Shell)

Patrone Desktop's taskbar for the GNOME-based desktop Patrone runs today,
packaged as `patrone-gnome-taskbar`. It's a GNOME Shell extension
(`patrone-taskbar@patroneos.com`), on for every account, until Patrone's own
shell (`shell/`, GTK4) replaces GNOME Shell. The design carries over to that
shell.

Its ID was `patrone-taskbar@` and the system's old name until 0.4.8. The
session mode (`patrone-desktop` 0.2.6) names the new one, and the taskbar
puts it in place of the old one in each account's own lists of GNOME
Shell's extensions when it starts, so an account that turned it off keeps
it off (from its next sign-in). `Breaks` makes `patrone-desktop` and both
Settings apps, which find its icons by its folder, upgrade with it.

Installing the package turns it on for every account, but GNOME Shell only
loads extensions when it starts: log out from inside the desktop (the
Patrone menu → the account picture → Sign out, or GNOME's top bar's power
menu → Log Out) and sign in again. Disconnecting Remote Desktop or signing
out of the screen's session leaves the desktop, and its old shell, running.
The same goes for every update: the desktop keeps running the old taskbar,
whose settings a new Patrone Settings may not know, until the next sign-in.
So the taskbar watches its own `metadata.json` (which every package update
replaces) and then shows a notification, "The taskbar was updated", with a
Sign out button. (The user hit this twice: after 0.2.0 Patrone Settings'
clicks did nothing, and after 0.3.0 the layouts, menu button icons and Royal
didn't show.)

## What it does

One compact bar along the bottom of the primary monitor, in place of GNOME's
top bar, which it hides: medium (42 pixels) unless Patrone Settings makes it
small (32) or large (52), its icons to match. Its look follows Patrone
Settings' theme (see `settings/README.md`): Patrone's own, Modern (flat),
Classic (plain and square), Royal (glossy royal blue, a green menu button)
or Studio (translucent, the buttons on a rounded shelf).

In Patrone, Modern and Studio the bar takes its **color from the
wallpaper** (`tint.js`): the picture's average hue, at the theme's own
lightness and see-through, with its saturation capped so a vivid picture
still gives a calm bar. Patrone's navy wallpaper gives a navy bar, Graphite
a graphite one, the light wallpaper a pale blue one. The picture is the one
the desktop shows in the current style (`picture-uri-dark` in the dark
style), decoded at 64 by 36 in a thread when it changes; a plain color
background gives its color, and a slideshow the theme's grey. The `color`
setting at `theme` keeps the theme's grey. Classic and Royal keep their own
colors. (The user, 2026-10-26: the grey bar didn't match the wallpaper.)

From the left:

- **Menu** (the Patrone "P", or the Super key) opens the Patrone menu with
  the search box focused, so typing searches right away (below).
- **Search** opens the same menu in search mode: the most used apps and
  recent files, then results as you type.
- **Pinned items**: apps, and **slots** for a job: web browser, terminal,
  file manager, mail, documents, spreadsheets, presentations, PDF files and
  art (the defaults, in that order). A slot shows Patrone's icon for its
  job (a globe, a prompt, a folder, an envelope, a page, a sheet of cells,
  a screen with a chart, a red-banded page, a palette, in
  `icons/pin-*.svg`), or with the `pin-icons` setting at `app`, the icon of
  the app it opens. It opens whichever app is the default for its job, and
  that app's windows show under it, not as a separate button. For most
  that's the system default for its file types (Settings → Apps → Default
  Apps): WordCraft for documents while it's the default for .docx and
  .odt, LibreOffice Writer once that is (the user, 2026-10-26: "just icons
  if those are the default apps, otherwise if libreoffice is installed and
  its the primary app for each make those the primary"). The terminal is
  Patrone Terminal and art is ArtCraft (else LibreOffice Draw, GIMP,
  Krita, Inkscape or Pinta) unless changed in the slot's menu. A slot no
  installed app can do isn't shown, except the web browser's: PatroneOS
   comes with no browser, so until one's installed the slot opens
  **Patrone Browser Chooser** (`browser-chooser/`), which asks which
  browser to install and has Patrone Software install it. It's the last of
  the system's choices for web links, so whenever it's the default and a
  browser it doesn't know of is installed, the slot opens that browser
  instead (`fallback` in `roles.js`); its menu never offers the chooser as
  a browser. With Patrone's icons, Settings shows Patrone's gear too, so it
  matches.
- **Running apps** that aren't pinned, in the order they started. An app
  shows the moment it's opened, its icon pulsing until its first window
  appears, as on Windows, so a slow one (a snap or a Flatpak starting for
  the first time) still shows it was clicked. A short line under an app
  means it's running, a longer blue one that it's in front.
- On the right, the **tray**: the top bar's own status icons and menus
  (network, sound, power, input sources, other apps' indicators), then the
  clock (time over date, the way Windows users expect, right-aligned with
  even space above and below), with the calendar and notifications behind
  it, and the **Show desktop** sliver at the far right.

Clicking a button works as on Windows: it starts the app, shows its window,
or minimizes it when it's already in front; with several windows each click
brings the next. A middle click, or Shift/Ctrl with a click, opens a new
window. Hovering shows the app's name.

**Dragging**: a button dragged along the bar moves there (a gap shows
where); an app that's only running is pinned where it's dropped, and so is
an app dragged from the Patrone menu (Favorites or All applications). In the
Patrone menu, Favorites are dragged to reorder them, a blue line marking where.

### Right-click: jump lists

As on Windows 11: the app's **Recent** files (those GTK recorded for it, then
recent files of types it opens; not for browsers), or for the file manager its
**Pinned** folders (the file manager's bookmarks, else Home, Documents,
Downloads...); its **Tasks** (the app's own actions: New Window, New
Incognito Window...); its **Open windows** when it has several; then the app
itself (a new window), **Pin to taskbar** / **Unpin from taskbar**, for a slot
**Default web browser** (and so on) to change which app the job uses, and
**Close window** / **Close all windows**. Right-click on the empty bar:
Task Manager, Show desktop, and Taskbar settings (Patrone Settings, when
it's installed).

### The Patrone menu

A compact menu (540 pixels wide), laid out the way Windows users expect:

- **Search box**, focused when the menu opens. Typing searches apps at once,
  then, as typing pauses, everything GNOME's own search covers (Settings,
  Files, Calculator, Characters, Clocks...: the same providers as the
  Activities overview, so apps that add a search provider show up here too).
  Results: **Top result** first, then by kind, and **Search the web** last.
  Up and Down move through them, Enter opens the selected one: the top result,
  or a web search when nothing else matched. Web searches go to the
  `web-search-url` setting (Google by default) in the default browser.
- **Favorites**: GNOME's favorites (the same list as the Activities overview's
  dash), up to 18. **All applications** lists every app, A to Z.
- **Recent**: recently used files, and the `suggested-apps` among them
  (Software, Patrone's app store, by default; 0.4.1, the user's ask on
  2026-10-27: "letting it move over time so people get familiar with
  it"). Each suggested app counts as opened when it was first suggested
  (`suggested-apps-since`), so it starts at the top, moves down as files
  are opened, and drops off after six newer ones, like any recent file.
  The search button's **Recent** list has them too.
- At the bottom, as on Windows 11: the **account** (picture and name), whose
  menu shows who's signed in, Change account settings (Settings → Users),
  **Lock** and **Sign out**; Settings; and **power**, whose menu has
  **Disconnect** while the desktop is seen through Remote Desktop (as on a
  Windows remote session: it ends the connection, or the screen's view, and
  leaves the desktop running for next time), then **Sleep**, **Shut down**
  and **Restart**, whichever the system allows. With none of those, the
  power button is hidden.

Right-click an app in the menu to add it to Favorites or pin it to the taskbar.

### The Recycle Bin on the desktop

Patrone File Manager draws the desktop's files (below, "The desktop and the
panel: File Manager's windows"); before 0.4.0 Ubuntu's desktop icons
extension did, with its Home and Trash icons turned off
(`92_patrone-taskbar.gschema.override`, `show-home` and `show-trash`). The
taskbar draws its own **Recycle Bin** (`recycleBin.js`), as the user asked
on 2026-10-26, so it can stay docked to a corner: bottom right unless
moved, inside the taskbar.

- Double-click opens it in the file manager (`trash:///`, which opens in
  whatever opens folders: Patrone File Manager).
- Its icon is Patrone's own (`icons/recycle-bin*.svg`; the user asked for
  one "more modern than windows/mac"): a Patrone Blue bin under a floating
  lid, with a white loop of an arrow on it, and when something's in it,
  two sheets of paper lifting the lid.
- Right-click: **Open**, **Empty Recycle Bin** (after asking; Cancel is the
  default), and **Keep docked to** Bottom right, Bottom left, Top right, Top
  left, or Not docked.
- Dragged while docked, it docks to the nearest corner; not docked, it stays
  where it's dropped.
- It sits above the wallpaper and the desktop icons' window and below every
  other window, and follows the free area when the screen size changes.
  That window starts as an ordinary one and only then becomes a desktop
  window and lowers itself, which GNOME Shell doesn't count as a restack;
  0.3.3 missed that, so the Recycle Bin stayed under it and couldn't be
  clicked or dragged (the user found it, 2026-10-26). It now follows each
  window's type, and knows the desktop icons' window by their own
  `customJS_ding` mark too. While it's dragged it holds the pointer, or the
  window the pointer passes over (that full-screen one) takes the drag.
  0.3.4 still didn't work: on Wayland the desktop icons extension takes its
  window out of `global.get_window_actors()` (its `gnomeShellOverride.js`),
  so the Recycle Bin looked there and never found it. 0.3.5 looks at the
  window group's own children. Checked once with the real desktop icons
  running in the throwaway shell: the Recycle Bin above their window, its
  own menu on right-click, and dragging.

- **Files and folders dropped on it go to the trash** (0.3.7; the user: "I
  can't drag into the trash a folder"). GNOME Shell's actors take no drops
  from apps, so the taskbar runs `dropTarget.js` (GJS and GTK 4, like the
  desktop icons' own window) as a Wayland client of its own: a window the
  Recycle Bin's size, made a desktop window once it shows (no taskbar
  button, Alt+Tab or focus), on every workspace, right under the Recycle
  Bin. While an app drags something (`dnd-enter` to `dnd-leave` from
  `global.backend.get_dnd()`) the Recycle Bin stops taking the pointer, so
  the drop reaches that window, which trashes what it's given; the Recycle
  Bin lights up while the drag is over it. Drawn fully clear, GTK gave the
  window no buffer and it never showed, so its background is 1% black.
- **No desktop icon goes under it** (0.3.7: a folder moved under it
  couldn't be clicked): File Manager's desktop is told where it sits
  (`homePosition()`, its place in its corner even while it rides on the
  bubble) and keeps those places free. (Ubuntu's desktop icons took whole
  margins through their area API, `setMarginsForExtension`, which is still
  called in case they run.) It stays above File Manager's desktop window,
  and so does its drop target: clicking the desktop raises that window,
  and the taskbar lowers it again (`desktopHost.js`, `_keepDesktopDown`).
- **Remove from desktop** in its menu turns `show-recycle-bin` off; Patrone
  Settings' Desktop page brings it back, and so does **Show Recycle Bin** in
  the desktop's right-click menu, which File Manager draws since 0.4.0 and
  shows the item in while the bin's off the desktop (it asks the taskbar,
  `show-bin`). The same item is still added to GNOME's own desktop menu
  (0.3.11), which other screens than the primary one show: GNOME makes one
  for each screen and wallpaper (`ui/backgroundMenu.js`), so the Recycle
  Bin overrides `BackgroundMenu.prototype.open` through GNOME's
  `InjectionManager` and adds the item, under Change Background, as each
  opens.
- **Over Remote Desktop the desktop starts with no screen**, until the
  client's arrives. 0.3.10 read the screen's size then, got nothing, and
  the error took the whole taskbar down at the user's sign-in
  (2026-10-27): GNOME's top bar came back, with the Recycle Bin and the
  terminal at sign-in but no taskbar. It never showed in the throwaway
  shell, which always had a screen and no desktop icons, whose area API is
  what led there. 0.3.11 places the Recycle Bin and the bubble only once
  there's a screen, and starts the extras (the Recycle Bin, the windows
  placed at sign-in, the bubble) through `startExtra` (`util.js`), so an
  error in one is logged and the taskbar carries on.
- Every two seconds it checks it's still right above the desktop windows,
  and moves back if not: once more, on the user's desktop, it was under
  Ubuntu's desktop icons, and that session can't be looked into from here.

Not done: docked top left it can still sit where the first desktop file
would; the reserved strip moves that file over one column.

### Desktop files: on the desktop, in a panel, or a clean desktop

Where the desktop's files show (the `desktop-files` setting, Patrone
Settings → Desktop → Desktop files, and the same on GNOME Settings' Desktop
and Patrone pages). It changes at once; signing out isn't needed.

- **On the desktop**, the default: the files and the Recycle Bin, as usual.
- **In a panel** (the user, 2026-10-30, in place of the bubble of 2026-10-26:
  "instead of a bubble, make it a box that comes out (like a full layer of
  the screen, it should close once something on top of it spawns open"): a
  pill at the screen's left edge opens them.
- **Clean desktop** (the user, the same day: "it doesn't show fiels on the
  desktop including the recycle bin ... I just hate clutter on the desktop
  and thats not how it should be used"): nothing on the desktop at all, no
  files, no Recycle Bin, no pill. The same panel shows the files when asked,
  optionally from a corner.

The default stays on the desktop: it's what a new user expects, and a clean
desktop with no corner picked has nothing to find. An account's own choice
of the setting this replaced (`desktop-bubble`, the panel's yes-or-no) is
carried over once, at sign-in (`extension.js`); an account that never chose
gets the default, so its files are back on the desktop.

The choices are on GNOME Settings' own Desktop page, where the desktop's
settings are looked for
(`gnome-settings/patches/patrone-desktop-switches.patch`; with Patrone's
taskbar they take the place of Ubuntu's Desktop Icons settings, whose switch
would bring Ubuntu's icons back beside Patrone's, and the page has no Dock
settings, for the same reason).

#### The corner of a clean desktop

The user: "allowing the user to pick (optionally) a corner that is not
pre-occupied with any configuration that shows the files until they point
the mouse to that same corner again." (`desktopCorner.js`, and the
`desktop-files-corner` setting: `none` or a corner of the primary screen.)

- **Pointing at it shows the files**, and pointing at it again hides them.
  The pointer has to stay 150 milliseconds, so brushing past does nothing,
  and has to leave before it can point again, so the files don't go as soon
  as they've come. The panel slides in from that corner's side, left or
  right. Everything else about the panel is the same: a window opening on
  top closes it, and so do Escape and its Close button. The corner does
  nothing in the overview, under a dialog of the shell's, over a fullscreen
  window, or while the mouse button is held (a window dragged there to tile
  it).
- **Only free corners are offered.** The taskbar works out what uses each
  corner of the primary screen and keeps it in `desktop-corners-in-use`,
  from which both Settings apps offer the rest, so they can't disagree:
  - the **taskbar**: both corners on its edge (the Menu button at one end,
    Show Desktop at the other), bottom or top with its layout;
  - **window buttons**: the corner with Close, since a maximized window's
    Close is in the screen's corner (`button-layout`; top right by default);
  - **GNOME's hot corner**, when it's on (`enable-hot-corners`, off in
    Ubuntu's settings, top left, or top right right to left);
  - **another screen** beside the corner, where the pointer goes on instead
    of stopping.

  The pill isn't in a corner, and a clean desktop has none. So by default
  (the taskbar along the bottom, Close at the top right) the one free corner
  is the top left. With the taskbar along the top, it's the two bottom ones.
- **A corner that gets taken stands down**, rather than fight: it stops
  showing the files at once, says so once in a notification, and Settings
  says what took it and offers the free ones. The choice isn't moved, so it
  works again if the other thing goes away. This follows the settings above
  and the screens as they change, with no signing out.

#### Reaching the Recycle Bin and files on a clean desktop

The Recycle Bin comes in on the panel, wherever that's opened from, and it's
always in File Manager's navigation pane. With no corner, or with windows in
the way: the empty desktop's right-click menu has **Show desktop files**,
**Open Desktop folder** and **Open Recycle Bin**, and the taskbar's has
**Show desktop files**. `show-recycle-bin` keeps its value but means nothing
here, so the Settings apps hide it, and the bin's menu doesn't offer to put
it on the desktop.

#### The panel

- **The pill**: a small Patrone-blue tab at the middle of the primary
  screen's left edge, shown whenever no window covers its spot (a window
  moved over it, or maximized, hides it). Hovering shows how many items the
  desktop has. Until it's first opened, it bounces every six seconds with
  a tip, "Your desktop files are in here: click, or drag it out"
  (`desktop-bubble-introduced` remembers it was opened).
- **Clicking it, or dragging it out**, slides the panel out of the edge (a
  corner on the right brings it from that edge).
  Dragged, it follows the pointer; let go more than a quarter of the way
  out and it opens, else it goes back. It takes 260 milliseconds with no
  overshoot (a whole layer has nowhere to bounce to), and 200 back. It runs
  on a `Clutter.Timeline` of its own, not `ease()`: with no graphics card
  (Patrone's Hyper-V VMs, for one) GNOME ends every `ease()` at once, and
  the panel would just appear. The account's Reduce Animation setting
  (`enable-animations`) still counts: off, it goes at once. Past the left
  edge it's cut off, so it can't show on a screen to the left.
- **What it is**: a full layer of the screen's free area, over the windows,
  with the taskbar in front of it, the account's wallpaper behind the icons
  (`Background.BackgroundManager`, the same picture as the desktop's), and
  each icon where it would be on the desktop, at the desktop's icon size.
  So it looks like the desktop pulled over the windows, and the icons'
  white names read as they do on the desktop. Because it looks that much
  like the desktop, a small bar at its top right says what it is: Desktop,
  how many items, **Open in File Manager** (which also reaches icons past
  the screen's last column), and **Close**. The pill rides on its edge as a
  handle, and with the panel open it's at the screen's right edge.
- **It closes by itself as soon as a window opens on top of it** (the
  user's ask). Above the windows, it would hide that window, so any
  normal window, dialog or utility window shown on this workspace closes
  it (`_windowShown`, on the window's `shown` signal, so also one coming
  back from being minimized), and the keyboard goes to that window even if
  GNOME kept it from taking it: back where it was, the window from before
  would come up over it. The panel's own menus aren't windows that count.
  So opening a file or folder closes it when its window shows, not when
  it's clicked, and a slow app leaves the panel up until it has something
  to show. File Manager's dialogs (Properties, delete) count too.
- **It also closes** with Escape (a rename takes the first one), the Close
  button, the pill or corner, a press on the taskbar or another part of the shell,
  anything else taking the keyboard, a change of workspace, the overview,
  or the screens changing. The keyboard goes back where it was, unless
  that's on another workspace: the workspace you changed to stays.
  **A click on its empty part doesn't close it**: there's no outside to
  click, and that's the desktop's own, for the rubber band and the
  right-click menu.
- **Patrone File Manager draws the icons** (the user, 2026-10-27: "The
  desktop and the roll out menu must be an extension of the file explorer
  app"), in a window of its own that the taskbar hosts (next section),
  clear but for them and the bar. The taskbar keeps the pill, slides that
  window's actor (`translation-x`, with a clip for what's past the edge)
  and the wallpaper under it, and draws a soft shadow on the windows along
  the moving edge.
- **The files** are those in the Desktop folder (`xdg-user-dir DESKTOP`),
  hidden ones left out, as on the desktop: the same places, so turning the
  panel on or off moves nothing, and icons dragged about the panel are
  where they were left on the desktop too. Everything File Manager's desktop
  does works here ([billsecond/patrone-file-manager](https://github.com/billsecond/patrone-file-manager)'s README, "The desktop"): opening, its
  right-click menus (on the empty part, the desktop's own, with Open
  Desktop folder first), renaming in place, keys, the rubber band. A new
  folder or a paste on the empty desktop opens the panel on what it made.
  With nothing on the desktop it says so.
- **Dragging**: the panel hides the windows, so files an app drags, held
  over the pill for half a second, open it to drop them in, or close it, to
  reach the windows under it.
- **The Recycle Bin** comes in on the panel, in the place it has on the
  desktop, above the windows (`recycleBin.hold()`), even while it's off the
  desktop (its menu's last item then reads **Show on desktop**), and goes
  back when it closes (`release()`). It still opens, empties and docks as
  usual while it's there. Files dropped on it go to the Recycle Bin: the
  panel's window takes those drops, since the bin's own drop target stays
  under the panel while it's there.
- **Keyboard**: the panel has it while it's open, with File Manager's
  keys: arrows, Enter, F2, Delete and the rest.
- **Closed, its window is minimized and not above** (`hidePanel`), not just
  unseen: a window that's there, as big as the free area, would leave GNOME
  no free place for new windows, and takes the keyboard when the last one
  closes. And GNOME counts every window that's above, minimized or not:
  left above, this one made it take each new window for covered, so a window
  started from a terminal, say, opened behind the one in front with no
  keyboard. `hidePanel` does nothing until the window has first shown, or
  it never would, and the first open would have only the wallpaper.
  GNOME won't minimize a window that's off the window list, so it's put
  back on it for that one step, which the overview mustn't see: its
  windows take it for a new one and leave a handler behind (a closing
  panel waits for the overview to close). Turned off (as at every screen
  lock), the panel stops taking File Manager's desktop's calls before that
  desktop ends, or the calls queued checks on its removed pill (0.4.3).
- **Ubuntu's desktop icons** are off while File Manager draws the desktop
  (next section). Should they run anyway, their windows (marked
  `customJS_ding`) wait in a hidden actor of the panel's, where they
  neither draw nor take clicks (clipping their actors to nothing didn't
  work: mutter still drew them), and aren't let keep the keyboard.

Not done: the panel on screens other than the primary one. The pill still
hides while a window covers it; the taskbar's right-click menu reaches the
panel then.

### The desktop and the panel: File Manager's windows

Since 0.4.0 Patrone File Manager draws the desktop (`patrone-file-manager
--desktop`; [billsecond/patrone-file-manager](https://github.com/billsecond/patrone-file-manager)'s README, "The desktop"), and the taskbar
hosts it (`desktopHost.js`): a Wayland app can't make its window the
desktop or keep one above the others, so the shell does.

- **It starts File Manager's desktop** as a Wayland client of its own
  (`Meta.WaylandClient`, as the Recycle Bin's drop target), when it starts
  and again whenever it ends on its own: after 1, 2, 5, 10 and 30 seconds
  when it keeps ending within a minute, then it gives up.
- **It knows the two windows by their titles**, and only from that client:
  "Patrone Desktop", made a desktop window (no taskbar button, Alt+Tab or
  moving; on every workspace; under every window, the Recycle Bin and its
  drop target) as big as the primary screen; and "Patrone Desktop Panel",
  as big as the screen's free area, off the window list, on every
  workspace, and minimized unless the panel is open, when it's above the
  windows (desktopPanel.js). Neither plays
  GNOME's opening animation. Showing gives a window the keyboard; it goes
  back where it was.
- **They talk over File Manager's standard input and output**, a line at a
  time (the list is at the top of `src/desktop.rs` in [billsecond/patrone-file-manager](https://github.com/billsecond/patrone-file-manager)): the
  screen's free area, the Recycle Bin (whether it's on the desktop, its
  place there, its size), the `desktop-files` setting and the panel
  opening and closing go to File Manager; the item count (the pill's tip
  and the panel's bar), asks to open or close the panel, and Show Recycle
  Bin come back. File Manager keeps those two pipes to itself, so the apps
  it opens
  can't write into them.
- **Ubuntu's desktop icons go**: once File Manager's desktop shows, the
  taskbar adds `ding@rastersoft.com` to GNOME's `disabled-extensions`
  (unless it's there already) and remembers that it did (`ding-replaced`).
  If File Manager isn't installed, or its desktop keeps ending, it takes it
  out again, and only then: an account that turned them off itself keeps
  them off.
- **The pill and the Recycle Bin can be dragged over File Manager's
  windows** because those windows stop taking the pointer while it's on
  one of them, or one is dragged (`shieldPointer`): with a client's
  surface under them, GNOME 50 ended their drag gestures the moment they
  were pressed (dragging the pill opened the panel like a click, and the
  Recycle Bin couldn't be moved). Not while an app drags something, so
  the panel can take the drop.

### The terminal at sign-in

Patrone Terminal opens at every sign-in (`patrone-terminal`'s autostart
entry), and the taskbar puts that window over the wallpaper's logo, which
shows through its see-through background, as the user arranged it
(2026-10-26: "when the GUI loads ... the terminal loads up in this position
and size making sure its sensitive to the logo backdrop placement"). Or,
the user's other choice, at its own size in the middle of the screen
("or it loads normal size, dead center"): the `sign-in-terminal` setting,
`logo` or `center`, which Patrone Settings' Desktop page and GNOME
Settings' Patrone page set ("Terminal at sign-in", Over the logo or
Centered). A third choice (the user, 2026-10-30), `off` (Off), keeps it
closed: the autostart script reads the setting and stops there, and a
terminal opened by hand soon after signing in isn't placed. On Wayland an
app can't place its own window, so the shell does it (`startupWindows.js`).

- The place is set in the wallpaper's picture, not the screen: over the
  logo, from just right of the `>_` prompt at the bottom left to just left
  of the dashed line at the top right. In the 1672×941 original that's
  (190, 81)–(1393, 777), which at the user's 1536×988 is exactly their
  window, and at 1920×1080 is 218, 93, 1382×799. GNOME scales the picture to
  cover the screen (`picture-options` 'zoom'), cropping its sides on a
  narrower screen or its top and bottom on a wider one, so the terminal
  follows the logo at any size, and keeps 16 pixels inside the free area.
  Patrone Badge, with its logo at the bottom right, gets a terminal that
  stops above it; another picture, the same fractions of the screen.
- Centered keeps the terminal's own size (920×659 in the throwaway shell)
  and puts it in the middle of the free area, above the taskbar. GNOME often does the same for a first window, but not with
  other windows already open.
- Only the first Patrone Terminal window within 30 seconds of the desktop
  starting, and only at sign-in: GNOME Shell turns extensions off on the
  lock screen and on again after, and then it does nothing. The choice is
  read as the window shows, so a new one is for the next sign-in.
- Until the user moves, resizes, maximizes or tiles it, it keeps its place
  (over the logo, or in the middle) when the free area changes (a Remote
  Desktop window resized, the taskbar's size or layout). GNOME itself
  nudges windows out of a new taskbar's way just before it says the free
  area changed, so a move just before that isn't counted as the user's.

Checked in the throwaway shell (2026-10-26, a private copy of
`tests/headless.sh`, the dark style, the 8K wallpaper) at 1920×1080,
1536×988, 1280×1024 and 2560×1080; following the taskbar to the top and
back; a title bar drag, a move by a script and maximizing, each left
alone after; a second window left to GNOME; and the extension turned off
and on again, as after the lock screen. Centered at 1920×1080, and
Patrone Settings' Desktop page run in that shell: both cards, and clicking
one set the setting. Not tried at a real sign-in.

### Settings

`org.gnome.shell.extensions.patrone-taskbar` (Patrone Settings sets the
size; the theme is Patrone Settings' own `org.patrone.desktop theme`, which
the taskbar follows when it's installed):

| Key | Default | |
| --- | --- | --- |
| `pinned` | `['role:web', 'role:terminal', 'role:files', 'role:mail', 'role:documents', 'role:spreadsheets', 'role:presentations', 'role:pdf', 'role:art']` | Pinned items in order: desktop file IDs, or `role:` slots |
| `size` | `'medium'` | `small`, `medium` or `large` |
| `layout` | `'left'` | `left`, `center`, `split` or `top` |
| `start-icon` | `'patrone'` | The menu button's icon, from `icons/` |
| `color` | `'wallpaper'` | `wallpaper` (the wallpaper's hue, in Patrone, Modern and Studio) or `theme` (the theme's grey) |
| `pin-icons` | `'patrone'` | `patrone` (Patrone's icons for the web, terminal, files and mail slots) or `app` (the icons of the apps they open) |
| `show-recycle-bin` | `true` | Show the Recycle Bin on the desktop |
| `recycle-bin-dock` | `'bottom-right'` | `bottom-right`, `bottom-left`, `top-right`, `top-left`, or `none` (where it's dropped) |
| `recycle-bin-position` | `(1.0, 1.0)` | Where it is when not docked, across and down the free area from 0 to 1 |
| `desktop-files` | `'desktop'` | Where the desktop's files show: `desktop`, `panel` or `hidden` (a clean desktop) |
| `desktop-files-corner` | `'none'` | The corner that shows them on a clean desktop |
| `desktop-corners-in-use` | `[]` | What uses each corner, `corner:user`; the taskbar keeps it, for the Settings apps |
| `desktop-bubble` | `false` | Replaced by `desktop-files`; read once to carry an account's choice over |
| `desktop-bubble-introduced` | `false` | Whether the panel has been opened yet; until then its pill bounces with a tip |
| `ding-replaced` | `false` | Whether the taskbar turned Ubuntu's desktop icons off (for File Manager's desktop), so it may turn them back on |
| `terminal-app` | `''` | The terminal slot's app; empty for Patrone Terminal (or another terminal) |
| `art-app` | `''` | The art slot's app; empty for ArtCraft, else the first drawing app installed |
| `web-search-url` | `'https://www.google.com/search?q=%s'` | Where web searches go |
| `sign-in-terminal` | `'logo'` | Whether the terminal opens at sign-in and where it goes: `logo` (over the wallpaper's logo), `center` (its own size, in the middle) or `off` (it doesn't open) |

## Decisions (with the user, 2026-10-26)

- Left-aligned, as Windows 11 can be set ("option C"), and **compact**: the
  user wanted it smaller than the first mockup, to take less of the screen.
  Built at 32 pixels, then made a third larger, icons and all, once they'd
  used it (42 pixels).
- The account picture opens Lock and Sign out, and the power button Sleep,
  Shut down and Restart, as on Windows 11, with Disconnect for Remote
  Desktop.
- Icons are dragged to reorder them, on the taskbar and in the Patrone menu.
- Patrone Settings sets the size (small, medium, large), the layout, the
  menu button's icon and a theme. Themes are a handful of looks with plain
  names: Patrone's own, Modern (flat and round), Classic (plain and square),
  Royal (glossy royal blue, a green menu button with no word on it) and
  Studio (soft and translucent). They change only the look: the layout
  (Left, Centered, Split with the menu button on the left and the apps in the middle,
  or Top, along the top of the screen) and which corner the window buttons
  go in are separate choices, which themes don't change.
- The menu button is the logo's "P", or one of seven other icons (a
  one-color P, an orb, a grid, three lines, a circle, a spark, a prompt);
  the one-color ones take the bar's text color. Clicking it lets you type to
  search at once. The search button stays, hooked to GNOME's search
  providers.
- Pinned slots open the default app for their job, which then shows under the
  slot instead of with its own icon; slots can be unpinned, and running apps
  pinned. The slots' icons are Patrone's own, with a setting for the apps'
  own instead (the user: "make our overridden icons look better, and allow
  an option to use the default app icons").
- The bar's color comes from the wallpaper, with a setting for the theme's
  own.
- Right-click gives Windows 11-style jump lists (recent files, tasks) along
  with the configuration (change the app, unpin).
- Enter with no match searches the web.
- No top bar: everything it had moves into the taskbar.
- Built as a GNOME Shell extension now, while the desktop is GNOME-based.
- Light or dark follows the system style (Settings → Appearance), with
  Patrone blue (`#288bfa`) for focus and selection.
- (2026-10-26, replaced 2026-10-30, below) The desktop's files go in a bubble out of the left edge, opened from a
  pill there, with the Recycle Bin riding on its edge. Asked: the files show
  only in the bubble; the pill shows while its own spot is clear and the
  bubble opens over windows; on by default, the pill bouncing with a tip
  until it's first opened.
- (2026-10-30) The bubble is now a panel, off by default: a full layer of
  the screen that slides out, and closes when a window opens on top of it
  (the rest was left to judgment: the wallpaper behind the icons, their
  places from the desktop, the small bar, a click on its empty part
  staying open, and sliding even with no graphics card).
- (2026-10-30) A third choice, a clean desktop, with an optional corner:
  only free corners are offered, a corner that's taken stands down, and the
  panel comes in from that corner's side (left to judgment too, with the
  list of what counts as taken and the default staying on the desktop).
- (2026-10-27) The desktop and the panel are Patrone File Manager's, as
  Windows' desktop is Explorer's, with Windows' desktop right-click menu
  and dragging to and from File Manager; the Recycle Bin stays the
  taskbar's.

## How it works

| File | |
| --- | --- |
| `extension.js` | Turns the taskbar on and off |
| `taskbar.js` | The bar, its buttons, the tray, the clock, Show desktop, the Super key |
| `startMenu.js` | The Patrone menu and its search |
| `jumpList.js` | The buttons' right-click menus |
| `roles.js` | The slots, and which app each one opens |
| `tint.js` | The bar's color from the wallpaper |
| `recycleBin.js` | The Recycle Bin on the desktop |
| `startupWindows.js` | The terminal opened at sign-in, put over the wallpaper's logo |
| `dropTarget.js` | The Recycle Bin's drop target, a small GTK 4 window run with `gjs -m` (never imported by GNOME Shell) |
| `desktopHost.js` | Patrone File Manager's desktop: starts it, makes its windows the desktop and the panel, talks to it, turns Ubuntu's desktop icons off |
| `desktopCorner.js` | What uses the screen's corners, and the corner that shows the files on a clean desktop |
| `desktopPanel.js` | The desktop files panel's pill, its sliding, wallpaper and closing (File Manager draws the icons), and the Recycle Bin on its edge |
| `recent.js` | Recent files (`~/.local/share/recently-used.xbel`) and pinned folders (`~/.config/gtk-3.0/bookmarks`) |
| `common.css`, `stylesheet-light.css`, `stylesheet-dark.css` | Sizes and shapes in the first, colors in the others (GNOME Shell loads one of the two, which imports the first). St lets an importing sheet's rules beat the imported sheet's, however specific, so a color that replaces one the light and dark sheets set (the royal theme's) goes in both of them, after it |
| `icons/` | The menu button's icons: `patrone-start.svg` and `start-orb.svg` in color, `patrone-start-light.svg` (the P in light tones, for Royal's green button, where the blues washed out), the `-symbolic` ones drawn in the bar's text color. The four with the P are drawn from the PatroneOS logo by `branding/tools/make_logos.py`, its slit and stem gap opened up for the button's 20 to 32 pixels: change that, not them. The Recycle Bin's, `recycle-bin.svg` and `recycle-bin-full.svg`. The slots' icons, `pin-web.svg`, `pin-terminal.svg`, `pin-files.svg` and `pin-mail.svg` (the terminal's and files' are Patrone Terminal's and Patrone File Manager's own) |
| `tests/headless.sh` | The throwaway GNOME Shell (below) |

- **Dragging** uses GNOME Shell's own drag and drop (`ui/dnd.js`): the
  buttons and the Patrone menu's tiles and rows are draggable; the taskbar's buttons
  box and the menu's Favorites grid take drops. A drop on the taskbar rewrites
  `pinned`; one in Favorites moves the favorite.

- **The top bar** stays alive but hidden, and stops reserving space. Its
  status area (`Main.panel._rightBox`, `_centerBox`) moves into the tray
  whole, so every indicator, including ones other extensions add later,
  comes along; their menus open upward. Turning the extension off puts it
  all back.
- **The screen-sharing indicator** is hidden in a desktop with no screen of
  its own, which is how Patrone's desktops run (see
  [billsecond/patrone-remote-desktop](https://github.com/billsecond/patrone-remote-desktop)): there the Remote Desktop connection the
  desktop is seen through counts as screen sharing, so the indicator is
  always on, and any click on it (GNOME stops the sharing on a press of
  any button) cuts the connection. The user hit this by right-clicking
  near it. Elsewhere it shows, as a small orange pill.
- **The Super key**: GNOME's own handler for it (it opens the Activities
  overview) is blocked, and the taskbar's opens the Patrone menu. The overview
  is still there (Super+A for the app grid).
- **Search** reuses the overview's own search providers
  (`Main.overview.searchController`), minus Ubuntu's web search provider,
  since the menu has its own.
- GNOME Shell disables extensions on the lock screen and enables them again
  after, which takes the tray back and forth with it.

## Trying it

`tests/headless.sh` runs it in a throwaway GNOME Shell with no screen, with
its own D-Bus session, settings and Wayland display under
`/tmp/patrone-taskbar-test`, in Ubuntu's session mode like the real
desktop, so it never touches anyone's desktop:

```
tests/headless.sh start 1600x900 light   # or dark
tests/headless.sh start none             # no screen, as over Remote Desktop at sign-in
tests/headless.sh run gnome-text-editor  # an app, for running buttons
tests/headless.sh click 23 883           # the menu button
tests/headless.sh type sett
tests/headless.sh drag 631 879 752 879   # press, hold, move, let go
tests/headless.sh shot /tmp/shot.png
tests/headless.sh close-apps             # only the apps it started
tests/headless.sh stop
```

It also fills the recent files list with a few sample files, gives the
shell a Desktop folder of its own with a few more
(`/tmp/patrone-taskbar-test/desktop`, through `user-dirs.dirs`), so the
desktop files panel never shows or trashes the account's real desktop files,
and leaves out two of Ubuntu's extensions that reach outside the test: the
desktop icons
(when they start, they end every desktop-icons process of the account,
the real desktop's included; early test runs made the real desktop's icons
restart this way) and the snap permission prompts. A helper
extension, `tests/unsafe@patroneos.com`, turns on GNOME Shell's unsafe mode
in that shell, which is what lets the script run JavaScript there and take
screenshots; never install it on a real desktop.

The taskbar starts File Manager's desktop with whatever
`patrone-file-manager` is first on the shell's `PATH`: to try a build,
start the script with a folder holding a link to it first
(`PATH=DIR:$PATH tests/headless.sh start`). File Manager's settings in
that shell are its own (the test's dconf), so icon places and View start
fresh each time.

What the tests covered (2026-10-26, GNOME Shell 50.1, light and dark): the
bar replacing the top bar; buttons packed left, the running and focused
lines; the Patrone menu opening from the button and from GNOME's overlay-key
signal (a simulated key press doesn't trigger GNOME's Super key detection
in a headless shell, so the key itself is untested there); typing "sett"
giving Settings as the top result, Settings pages, a Clocks result and the
web; Favorites, Recent, All applications and the power options; jump lists for
the web, files and a running app; the quick settings and calendar menus
opening upward from the tray; click to minimize and restore; and turning the
extension off (the top bar comes back) and on again. Later, with
`tests/headless.sh drag`: reordering the taskbar's buttons, pinning a
running app by dragging it in, and reordering the menu's Favorites; and,
with Patrone Settings running in the shell, all three sizes and four
themes.

From a code review of the same versions (2026-10-30), in this shell, which
had missed it because the panel was open in the other tests: with Text
Editor in front and the panel closed, Calculator started with no activation
token (as from a terminal) opened behind it with no keyboard in the panel and
clean modes, and in front on the desktop. Now it's in front with the
keyboard in all three, and the panel is above only while it's open. Also:
the panel's window going through its first show with `hidePanel` called
early, and a change of workspace with the panel open staying on the new
workspace, with the keyboard on the window there.

A clean desktop and its corner (taskbar 0.4.12 with File Manager 0.3.10,
Patrone Settings 0.3.13 and GNOME Settings `patrone21`, 2026-10-30, unpacked
from their packages and run in this throwaway shell on the dev VM, nothing
installed there). `desktop-files` at `desktop` in a fresh account. The
carry-over: `desktop-bubble` true gave `panel`, false gave `desktop`, and a
choice in `desktop-files` wasn't overwritten. Clean: no icons, pill or
Recycle Bin. At the top left, with the default settings the only corner
offered: pointing showed the files, staying did nothing more, leaving and
pointing again hid them and then showed them, a touch of 60 milliseconds did
nothing, Escape closed it. GNOME's hot corner turned on: the corner stood
down with a notification, the overview opened instead, and turned off it
worked again. Close moved to the left: top left taken, top right free, and
with that picked the panel came in from the right (a frame mid-slide) and
went back the same way. The taskbar along the top: both top corners taken,
and the bottom right worked. The right-click menus: Show desktop files from
the desktop's and the taskbar's, Open Recycle Bin opening it in File Manager,
and no way to put the bin on the desktop from its menu. Patrone Settings by
clicks, offering No corner and Top left, and saying what took the corner;
GNOME Settings' Desktop page with its two lists, picking No Corner by
keyboard; its Patrone page showing both groups. Not tried: a real screen, a
fullscreen window over the corner, a screen beside the corner, and right to
left.

The desktop files panel (the same versions, 2026-10-30, the
branch's taskbar and File Manager in this throwaway shell on the dev VM,
1600×900, nothing installed there): the setting off in a fresh account,
with the files on the desktop and no pill, and an explicit choice kept over
that default. Turned on, the pill and its tip, an empty desktop,
and the window minimized while closed. Opened by a click and by a drag past
a quarter, a shorter drag going back, a drag back closing it; a frame in
the middle of the slide (the shadow, the pill on the edge, the bin and bar
going along) with no graphics card; light and dark, and with the taskbar
along the top, the window right under it at 1600×857. Each way of closing:
a new window (it goes on top with the keyboard, even started with no
activation token), double-clicking a text file and an image with Enter
(when their windows showed), Escape, the Close button, the pill, a click on
the taskbar, a change of workspace and the overview (no errors, and put
away once it closed). A click on the empty part leaving it open, and the
desktop's own right-click menu there; rename in place, the first Escape
cancelling only that. Dragging an icon (its place saved with the desktop's),
a drop on the bin (File Manager took it; the test's folder has no Recycle
Bin, so it asked to delete, and that dialog closed the panel), the bin off
the desktop riding on the panel, a new folder on the closed desktop opening
the panel with its name being edited, an empty desktop, and turning it off
while open, with the files back in their places. Not tried: on a real
screen with a graphics card, a screen to the left of the primary one
(where the clip matters), files from another app held over the pill, and
a screen reader on the bar.

The desktop bubble (2026-10-26, 1600×900, light and dark): the pill and
its first-run tip; the bubble with 1, 7, 20 and 60 files (the last at half
the screen, with "21 more"); dragging it halfway out (the bubble at 41%,
the Recycle Bin on its way from its corner) and letting go; the Recycle
Bin on its edge and back in its corner after Escape; a click outside
closing it and one on its empty part not; both right-click menus; a click
on a file opening it (WordCraft for an .odt). With a stand-in for the
desktop icons' window (a Text Editor window marked as they mark theirs,
made a desktop window and lowered, and left out of
`global.get_window_actors()`): hidden, its keyboard focus taken away, a
right-click on the empty desktop giving GNOME's menu, still hidden after
another app restacked the windows, and back above the wallpaper, under the
other windows, when the setting was turned off. A window moved over the
pill hid it. Patrone Settings' Desktop page, run in the shell, turned the
bubble and the Recycle Bin off and on. Not tried: the real desktop icons
extension (it mustn't run in the test shell), and opening a launcher.

File Manager's desktop and bubble (0.4.0 with File Manager 0.3.0,
2026-10-27, 1600×900, light and dark, and with no screen), in a copy of
`tests/headless.sh` with its own folder and display and a link to File
Manager's build first on `PATH`, all with real pointer and key events:
both windows found and set up (a desktop window as big as the free area,
the bubble's above the others and unseen); the pill opening the bubble by
a click and by a drag, and a drag back closing it; the bubble with 7 and
60 files (half the screen, "19 more"), the Recycle Bin on its edge, the
pill and the bin under the bubble's menus; Escape closing it with the
keyboard going nowhere; both right-click menus in the bubble; New → Folder
renamed in place there; with the bubble off, the icons down the left side,
moved by dragging, a rubber band, the arrow keys and Enter (a File Manager
window), View (small, large, Auto arrange), F2, Ctrl+C and Ctrl+V; drags
between File Manager and the desktop and the bubble with no key, Ctrl and
Shift, on one drive and between two (/tmp and /dev/shm), a conflict
asking Replace or Skip, Ctrl+Z on the desktop putting a moved file back;
files held over the pill opening the bubble and dropped in it; Show
Recycle Bin from the desktop's menu; the Recycle Bin dragged to another
corner over the desktop; a drop on the Recycle Bin reaching its drop
target (GIO can't trash in /tmp, where the test's Desktop folder is);
Allow Launching on a launcher, and an untrusted one asking first and then
starting its app; Templates in New; a thumbnail; the taskbar turned off
and on (the desktop ended and came back) and File Manager's desktop killed
(back in a second); `ding-replaced` turning Ubuntu's desktop icons off and
back on in the test's own settings (never running them). With no screen:
no errors, the windows waiting unshown; a screen arriving later wasn't
tried.

## Not done yet

- Window thumbnails when hovering a running app.
- A taskbar on other monitors (it's on the primary one only).
- Notification pop-ups still appear at the top right, not above the clock.
