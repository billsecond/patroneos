# Next steps

**PatroneOS since 2026-10-29.** The user renamed the system PatroneOS: "The
website will be https://patroneos.com." The name is one word wherever it
appears; "Patrone" stays the short name (Patrone Terminal, `patrone-*`
packages, app IDs, paths, `ID=patrone`). Entries below from before then
use the new name too, quotes included (the user: "I want to be sure we
remove anything that says" the old name).

**A second EFI folder on PatroneDev.** GRUB's updates (`grub-install`, run
by `grub-efi-amd64-signed`'s postinst) name the EFI folder after the first
word of os-release's NAME, now PatroneOS. So the next GRUB update on an
existing install puts GRUB in `\EFI\patroneos\`, with a boot entry of its
own, beside the folder it was installed in: `\EFI\ubuntu\` on PatroneDev
(from before it was Patrone), `\EFI\patrone\` on earlier test installs. Both
boot, so it's left as it is (the main session, 2026-10-29: the only
existing installs are PatroneDev and throwaway VMs). Which folder a fresh
install starts in is for the next fresh install to show.

Last updated 2026-10-27, after the ISO's installer, first-login setup and a
search of the installed system for Ubuntu's name (section 16 lists what's
left of it).

## Where things stand

Done:

- `PatroneDev` VM set up by `scripts/bootstrap-dev-vm.sh` (packaging, ISO and
  QEMU tools, Wayland and GTK4 libraries, Patrone Terminal's libraries, fonts
  and zsh plugins, Hyper-V tools). Rust 1.99.1 (stable) and 1.93 (Ubuntu's
  rustc, for package builds) through rustup. Claude Code, `gh` and `git`
  signed in.
- Stack decided: Rust + Smithay compositor, GTK4 shell (see `docs/PLAN.md`).
  Default apps decided, including **no web browser preinstalled** (since
  2026-10-28; it was Chromium in place of Firefox: section 4). License:
  **GPL-3.0-or-later** (`LICENSE`), set in the crates and Debian packaging;
  the compositor keeps Smithay's MIT notice in `compositor/NOTICE`.
- **Milestone 2: the compositor** (`compositor/`) runs nested in GNOME with
  the wallpaper and a terminal inside it (section 2).
- **Patrone Terminal** (`terminal/`, see its README): copy on select, paste on
  right-click, Ctrl+click links, tabs, close confirmation, Preferences,
  `--standalone`, placeholder icon. Installed for the user with
  `install-dev.sh --zshrc --bashrc --default`, which also made it the terminal of
  Ctrl+Alt+T, the dock and the app grid in place of Ptyxis.
- **Shell setup** (`shell-config/`): the user wants **bash as the default,
  with the Kali-style prompt** (2026-10-26). `shell-config/bashrc` is that
  prompt, loaded from `~/.bash_aliases`; installed for the user
  (`install-dev.sh --bashrc`) and checked in an interactive bash in a pty.
  `shell-config/zshrc` and `zprofile` remain for anyone who picks zsh.
- **Packages** (`packaging/`): `patrone-terminal` (0.1.1, with a login
  autostart entry), `patrone-shell-config`, `patrone-branding`,
  `patrone-default-apps` (no browser since 0.2.0: section 4) and
  `patrone-remote-desktop` build on the VM. Everything the user asks for goes
  into these as Patrone's system-wide defaults (they said so, 2026-10-26);
  `scripts/hyperv/` is only dev-VM tooling.
- **Installed on the VM** (2026-10-26, by the user): `patrone-branding`
  0.1.0, `patrone-terminal` 0.1.1, `patrone-default-apps` 0.1.0 with the
  Chromium 156 snap; Firefox's snap removed and its stub held. Checked:
  `default.plymouth` and `gdm-theme.gresource` point at Patrone's,
  `/etc/os-release` says PatroneOS. Not rebooted since, so the login
  screen and the user's session still showed Ubuntu's look.
- **Branding** (section 6): `patrone-branding` replaces Ubuntu's name and
  logos (boot splash, boot menu, login screen, About, console banner, login
  message), restyles the login screen, makes blue the accent color (and, in
  0.1.1, Yaru's blue GTK theme and icons) and turns Ubuntu's dock off.
- **Terminal at login:** GNOME opens Patrone Terminal at every login, for
  everyone (`patrone-terminal`'s `/etc/xdg/autostart` entry). The Patrone
  Desktop session's compositor already opens one.
- **Patrone Remote Desktop** (section 3, `remote-desktop/README.md`): each
  account in the `remote-desktop` group signs in once, with its own Linux
  password checked by PAM; nothing stores the password. Built, unit-tested,
  not installed or tried with a real client yet.
- **Hyper-V and VirtualBox guest support** (section 17,
  `guest-tools/README.md`, package `patrone-guest-tools` 0.1.0, 2026-10-28):
  in a fresh install from an ISO with it, all six Hyper-V integration
  services were OK and checked from the host (file copy, production
  checkpoints, KVP, dynamic memory, shutdown, the host's clock). VirtualBox
  is packaged but untried (the user chose not to install VirtualBox).
  Enhanced Session is `patrone-remote-desktop` 0.5.0's (below).
- **Virtual Machine Setup** (section 17, `hyperv-setup/README.md`, package
  `patrone-hyperv-setup` 0.2.0, 2026-10-29): at login in a Hyper-V VM, a
  dialog offers to set up what's missing (the guest tools, the enhanced
  session) with one password, and shows the PowerShell command for Windows'
  side; "Don't show this again" per account. Checked on PatroneDev with the
  missing parts pretended; a fresh install is still to check.
- **Hyper-V's enhanced session** (`remote-desktop/README.md`, package
  `patrone-remote-desktop` 0.5.0, 2026-10-28): VMConnect shows Patrone's
  login screen, then the account's own desktop, resizing with the window.
  The user found the clipboard and resizing working on the dev VM and in a
  fresh install from the ISO, but no sound; 0.5.2 sends sound over the
  static rdpsnd channel instead (2026-10-29), still for the user to try.
- **Patrone's own updates** (section 18, 2026-10-28): an apt repository on
  GitHub Pages, [billsecond/patrone-apt](https://github.com/billsecond/patrone-apt),
  signed with a key that's only on the dev VM, published with
  `packaging/publish.sh`. `patrone-desktop` 0.2.1 installs its source, key
  and pins (Patrone's builds of GDM, GNOME Settings and first-login setup
  win over Ubuntu's newer ones), and the dev VM rebuilds those three daily
  when Ubuntu updates them (`packaging/ubuntu-updates.sh`). Merges publish
  themselves within a day: the VM builds and publishes whatever on `main`
  has a newer version than the repository (`packaging/main-updates.sh`,
  section 18). Section 18 also has the plan for moving to 28.04.
- **Patrone taskbar** (section 7, `shell/gnome/README.md`, package
  `patrone-gnome-taskbar`): a compact taskbar at the bottom, laid out the
  way Windows users expect, in place of GNOME's top bar, with the Patrone
  menu (type to search), role slots, jump lists and the tray. Built, and
  tried in a headless GNOME Shell on the VM; not installed yet.
- **Patrone File Manager** (`file-manager/`, package `patrone-file-manager`,
  section 8): a file manager in Rust and GTK4 that works the way Windows
  users expect, with Patrone-drawn icons. `patrone-default-apps` 0.1.2
  makes it the app for folders and puts it in Files' place in the
  favorites. The user installed
  0.1.0 and asked for changes; 0.2.0 (tab dragging between windows, typed
  commands, system folder hints, a new Permissions tab) is built and tried
  with real input in a headless GNOME Shell, not installed yet.
- **fastfetch** shows the Patrone logo for the user (`branding/fastfetch/`,
  installed with `branding/fastfetch/install-dev.sh`): the mark traced from
  the concept by `branding/tools/make_terminal_logo.py`, with a gradient in
  the Patrone blues. Checked with plain `fastfetch` in Patrone Terminal. (At
  first it was only previewed with `-c`/`--logo`, so the user's plain
  `fastfetch` still showed Ubuntu's logo and looked like a false claim. Say
  exactly how something was checked.)

Waiting on the user:

- **Notepad** (the user, 2026-11-03: "build a notepad application ... and
  please build it in RUST", "allowing me to turn off the auto-save option";
  `notepad/README.md`, package `patrone-notepad` 0.1.0): a Rust and GTK 4
  text editor with tabs in the title bar, File, Edit and View menus,
  Settings as a tab, find and replace, go to line, zoom, printing, a font
  of your choice (PatroneOS Mono by default) and a status bar. By default
  it keeps open tabs and their unsaved changes between sessions
  (Settings → Restore open tabs turns that off). `patrone-default-apps`
  0.2.3 makes it the app for plain text, with GNOME's Text Editor as the
  fallback, and `patrone-desktop` 0.2.8 recommends it; the command is
  `notepad`. On PatroneDev, `patrone-notepad` 0.1.0 and
  `patrone-default-apps` 0.2.3 are installed from the branch build (not
  `patrone-desktop` 0.2.8, which only adds the recommendation). Checked
  with the installed package in a throwaway GNOME Shell (its own state
  folder too): found in the Start menu, opened, tabs (new, switching,
  dragging, closing one with unsaved changes), Open, Save, Save as, files
  opened through `gio open` going to the running window, find and
  replace, go to line, zoom, dark theme, printing to PDF (3 pages, header
  and footer), and Restore open
  tabs both ways (on: tabs and unsaved text came back after closing; off:
  closing asked to save, and it started with an empty tab), and its look
  in dark with the user's own theme (Studio, Wide buttons). In the user's
  own desktop it was only launched over SSH (it ran, and its session file
  had the tab), not looked at: the VM's screen shows the login screen, so
  `vmshot.ps1` can't see that desktop. Not tried: dropping files on the
  window, a real printer, and Patrone Settings' other themes and title bar
  buttons. Other installs, once it's published:
  `sudo apt update && sudo apt install patrone-notepad`.

  **0.2.0 adds Markdown and PatroneOS's fonts** (the user: "allow it to
  open markdown and edit in markdown just like notepad does but better";
  "And use the patrone os font's too"): `.md` files show formatted, with a
  formatting bar, an outline, a switch to the raw Markdown and Export as
  HTML (`notepad/README.md`, "Markdown"). Installed on PatroneDev from the
  branch, with `patrone-default-apps` 0.2.3 rebuilt (it now sends
  `text/markdown` to Notepad). Checked: every Markdown file in the repo
  (35) shown formatted comes back byte for byte (the ignored test in
  `formatted.rs`, run in the throwaway shell's display); in the app, six
  of them (README.md, docs/PLAN.md, NEXT.md, COMPLIANCE.md,
  branding/README.md, notepad/README.md) opened, moved through, edited
  back to the same text and saved, identical; the formatted view, ticking
  tasks, images, rules, tables, code blocks, the formatting keys, the
  outline, Link, Export as HTML, turning formatting off, and dark mode.
  `fc-match` gives each of the five PatroneOS families itself, and the
  font list starts with them.

  **A Windows build** (the user: "Then install it here on this windows
  environment as well as discribe how to install it on another
  environemtn"): `notepad/windows/build.ps1` makes a zip whose
  `install.ps1` installs Patrone Notepad for one account (no
  administrator, a Start menu entry, no file associations). Not built or
  run yet: running MSYS2 on the user's PC (unpacked in
  `%LOCALAPPDATA%\msys64`, its checksum checked) was refused by Claude
  Code's permission check, so it waits for the user to allow that or to
  run `build.ps1` themselves. The scripts only parse so far, and whether
  Pango loads the bundled fonts on Windows (`add_font_file`) is untried.

- **Patrone Software's Featured tab** (the user, 2026-10-31: "please feature
  only for now, the MailCraft, the Artcraft Launcher and the Artcraft PDF
  and the Portal for TEams, Wordcraft, Gridcraft. Add the ScreeGrabCraft
  project too. Make it a Featured tab in front of explore";
  `software/README.md`, `gnome-software/README.md`). The tab shows MailCraft,
  PdfCraft, Portal for Teams, WordCraft, GridCraft and ScreenGrabCraft. A tab
  means rebuilding
  GNOME Software, so Patrone now has five builds of Ubuntu's packages
  (`gnome-software/`).
  - **ScreenGrabCraft is the sixth.** `billsecond/screengrabcraft` went public
    on 2026-10-31, on the user's word. Read first: all 5 commits, all 255 file
    names ever, its 3 screenshots, its settings. No keys, tokens, private
    addresses, personal e-mail or stored secrets.
  - **The ArtCraft Launcher stays out** (the user, 2026-10-31) until they
    have the ArtCraft Team's written permission (`docs/COMPLIANCE.md`, item
    6); then it's `apps.list` and the featured list.
  - **Checked** on PatroneDev with the installed packages, in the throwaway
    shell: the tab comes first, opens first, and comes back when the app is
    reopened; Alt+1 to 4; banners, arrows and tiles open an app, and Back
    returns; search; `--mode=`; dark mode; a narrow window; the empty page,
    with the list moved aside. Explore no longer repeats the picks. With
    ScreenGrabCraft's real release, served from the VM, through the real
    `build-index.sh` into an unsigned test index: six tiles, Install on its
    tile, and `screengrabcraft-cli` 0.1.1 ran.
  - **Not checked:** running ScreenGrabCraft's tray and capture, the user's
    own desktop, a fresh install from an ISO, the daily job's first publish
    of `gnome-software`, and the banners turning by themselves.

- **Drives and mounts in File Manager** (the user, 2026-10-28: "add under
  computer Mounts where the drives are ... and add the ability to mount
  other drives/resources"; `file-manager/README.md`, Drives and mounts).
  This PC lists every drive with where it's mounted ("Mounted at /",
  "Mounted at /run/media/..."), and drives that aren't mounted with their
  size; opening one mounts it, and Mount, Eject and Unmount are in the
  menus, the command bar and the navigation pane. Map network drive
  (`\\server\share`, `sftp://`...) signs in when asked and can reconnect
  at each sign-in; right-click an `.iso` or `.img` to Mount it. Since
  0.3.3 the sign-in is Patrone's own, readable in dark mode (the user:
  "Connecting to a network share looks horible"), and the Network page's
  computers have computer icons. Since 0.3.4 This PC shows "Drive mounts"
  (`/run/media/USER`, where USB drives appear) and "Other mounts" (`/mnt`)
  (the user: "also show /media too"), and a shared folder on the Network
  page can be mounted or mapped from its right-click menu (the user:
  "Allow me to right click on a network folder and mount it"); a network
  drive then shows under This PC with a folder path every app can open.
  Since 0.3.5 the first visit to the Network page after signing in no
  longer freezes File Manager for about 10 seconds: it says "Looking for
  computers and devices on the network..." until GVfs is ready. Since
  0.3.6 This PC is "Computer" and Local Disk "System drive", with a new
  blue icon (`docs/COMPLIANCE.md`, item 15).
  `patrone-file-manager` 0.3.5 is installed on PatroneDev: close every
  File Manager window to start using it (the desktop's icons switch at
  the next sign-in). `patrone-remote-desktop` 0.5.1 lets Patrone's
  seat-less desktops mount USB drives and disk images with no password
  and internal drives with one; its polkit rule is already in place on
  PatroneDev (checked with `pkcheck` against the desktop session), but
  the package isn't installed, because installing it restarts Remote
  Desktop and drops the connection:
  `sudo apt install ~/src/patrone-fm-drives/out/patrone-remote-desktop_0.5.1_amd64.deb`
  when that's fine. Tried with real input in a headless shell as the
  `claude` account (a fake internal disk, `.iso` and `.img` files, a
  stand-in network share, an SMB address with no server); not tried with
  a real USB stick or SMB server, or in the user's own desktop.

- **No web browser preinstalled; the user picks one** (the user,
  2026-10-28: "Ask the user when they click on the browser with a modal if
  they want to install Chromium, Fire Fox, Brave, etc, then use the current
  patrone apps to open that up and set up the install"; section 4). The
  taskbar's Web browser button, or the first link opened, brings up
  Patrone Browser Chooser; the pick installs through Patrone Software from
  Flathub, becomes the default and opens. Tried end to end in a fresh
  install in `PatroneBrowserTest` (off now; start it to look, or delete
  it). The next ISO has it. On PatroneDev, which has Chromium's snap,
  Chromium stays the default:
  `sudo apt install ~/src/patrone-linux/out/patrone-browser-chooser_0.1.0_amd64.deb
  ~/src/patrone-linux/out/patrone-default-apps_0.2.0_all.deb
  ~/src/patrone-linux/out/patrone-gnome-taskbar_0.4.4_all.deb`.

- **Desktop files** (the user, 2026-10-30: "Make the desktop files bubble
  off on default, and instead of a bubble, make it a box that comes out",
  then "allow for another option ... so that it doesn't show fiels on the
  desktop including the recycle bin"): Settings → **Desktop** → **Desktop
  Files**: On the Desktop (the default), In a Panel, or Clean Desktop, with
  an optional corner that shows the files and hides them again, offering
  only corners nothing else uses. It takes the place of Ubuntu's Desktop
  Icons and Dock settings there. Taskbar 0.4.12 and File Manager 0.3.10 go
  together; Patrone Settings 0.3.13 and GNOME Settings `patrone21` have the
  words. Checked in the throwaway shell, not yet in a fresh install
  (shell/gnome/README.md, "Desktop files").

- **Remote Desktop first in Settings** (the user, 2026-10-28: "make the
  Remote Desktop as the main and first option that enables rdp, call it by
  Remote Desktop, and add explain what the otehers actually do"): System ›
  Remote Desktop opens on a **Remote Desktop** tab with one **Remote
  Desktop** switch, then how to connect, then "Other Ways to Connect",
  which says what Desktop Sharing (now the second tab), Secure Shell and
  GNOME's Remote Login do. It's in `patrone12` and later, as in the
  install line above (`gnome-settings/README.md`); `patrone12` also makes
  Patrone, not Ubuntu, the package's maintainer (`docs/COMPLIANCE.md`,
  item 13). Tried end to end in a fresh install.

- **Install taskbar 0.4.3: File Manager draws the desktop and the
  bubble** (the user, 2026-10-27: "The desktop and the roll out menu must
  be an extension of the file explorer app, while supporting the desktop
  and all it can do"; after trying 0.3.12's bubble: "The icons are all
  different, and I can't drag and drop them around"; sections 7 and 8).
  The VM has File Manager 0.3.1 and taskbar 0.4.2 (whose pill froze the
  desktop with the bin off it until 0.4.2's fix, and whose bubble then
  opened without the bin); 0.4.3 puts the bin back on the bubble's edge
  and stops the bubble's checks outliving a screen lock. From SSH or a
  terminal:
  `sudo apt install ~/src/patrone-linux/out/patrone-gnome-taskbar_0.4.3_all.deb`,
  then sign out from inside the desktop and back in. Ubuntu's desktop
  icons go (the taskbar turns them off); the bubble and, with it off
  (Patrone Settings → Desktop), the desktop itself show File Manager's
  icons. Right-click the desktop for the menu Windows users know (View,
  Sort by, New, Paste, Undo...); drag icons about the bubble or the desktop, onto the
  Recycle Bin, into File Manager or another app, with Ctrl (copy) or
  Shift (move); hold files from File Manager over the pill to open the
  bubble. 0.3.1 has 0.2.10's fixes too. Checked only in a throwaway shell.

- **The first Patrone ISO** (2026-10-27, `iso/README.md`): `iso/build-iso.sh`
  remasters Ubuntu 26.04.1's desktop ISO with two Patrone layers, what the
  installer installs and the live session, so the boot splash, live
  desktop, installer (patrone-branding 0.1.9) and installed system are
  Patrone's, with no Ptyxis and no App Center, and since 2026-10-28 no web
  browser (section 4; it seeded Chromium in place of Firefox before) (the
  user: "IT SHOULDN'T SAY UBUNTU"; "the other terminal
  shouldn't even show up"). `scripts/hyperv/new-test-vm.ps1` made the
  `PatroneTest` VM (Secure Boot on). The user watched the first installs;
  for later ones they asked for the install to be done here, in Hyper-V
  (2026-10-27: "Please make sure that you also do this install yourself").
  If they have a Remote Desktop window open to `PatroneTest` (an `mstsc`
  window named after its address), leave it alone and use another VM. Found
  on the way:
  - **The installer is Patrone's build of Ubuntu's** (`installer/README.md`,
    package `patrone-installer`, live session only). Ubuntu's front end
    takes its name from a fixed list of Ubuntu's flavors
    (`UbuntuFlavor.fromName`), so the user chose rebuilding it with Ubuntu's
    flavor named PatroneOS; checked in QEMU: "Welcome to PatroneOS
    ". Its look and slides are `patrone-branding`'s
    (`branding/system/desktop-provision/`). A new Ubuntu ISO means
    rebuilding it at that snap's commit.
  - **Swapping the ISO in Hyper-V:** inserting it right after replacing
    the file fails ("the object was not found") and leaves the drive empty
    while `Get-VMDvdDrive` still shows the path for a moment, so the VM
    falls to network boot. `scripts/hyperv/swap-iso.ps1 -New <file>` does
    the swap: it inserts again until `DvdMediaType` is `ISO`, then puts the
    DVD first in the boot order.
  - **The first install failed** on the ISO's leftover `/cdrom` folder
    (`FileExistsError` in subiquity's `configure_for_install`); the build
    now removes it. The user's notes from watching it (fixed in
    `0ba7950`, `b4b0c4c`): no orange or Ubuntu logos in the installer,
    a plain spinner, the first initial and last name as the username, a
    single ring at boot.
  - **Remote Desktop breaks automatic login.** With `patrone-remote-desktop`,
    an account that logs in by itself (the live session; an install with
    "log in automatically") falls back to the login screen: GDM refuses
    the account's headless desktop because the screen's session is already
    open ("There's already an opened session for user ..."). The ISO's
    live session leaves the package out; installs should require the
    password until it's fixed (section 3).
  - **patrone-branding asks about `/etc/issue`** when it's installed: dpkg
    takes Ubuntu's copy, moved aside by the diversion, as deleted by the
    user. The ISO build answers with `--force-confnew`; an install with apt
    on a running system asks.
  - **"Install PatroneOS" boots to the installer alone**, full screen
    in a GNOME Kiosk session with no desktop behind it (`installer/README.md`,
    "Installing with nothing else"); "Try PatroneOS" is the live
    desktop. The kiosk session doesn't switch to a text console with
    Ctrl+Alt+F3.
  - **First-login setup is Patrone's build of Ubuntu's**
    (`gnome-initial-setup/README.md`): "Choose how PatroneOS looks",
    Patrone Software in place of App Center, Patrone's pictures. Patrone
    Terminal waits for it to finish before it opens at the first login, so
    it no longer covers the setup's pages.
  - **Settings › System › Remote Login has "Enable Remote Desktop"**
    (`e0eb760`, from a side session): the user couldn't find how to turn on
    Remote Desktop in an install. They tried it in `PatroneTest`.
  - **A search of the installed system for Ubuntu's name** (the user,
    2026-10-27: "a full search on the name and references across the OS")
    found the snaps and notices fixed in `98c51cd` and the bigger pieces in
    section 16. The user's rule for it: saying PatroneOS is based on
    Ubuntu is welcome where it explains what's different ("don't dog
    ubuntu, just explain the differences").
- **The Craft apps show and install in Patrone Software** (done
  2026-10-27, `patrone-software` 0.1.4, installed; the user: "Looks
  great"). They come from Patrone's app index,
  [billsecond/patrone-app-index](https://github.com/billsecond/patrone-app-index)
  (a separate public repository, on GitHub Pages): only each app's listing
  and a link to the tarball on its own GitHub release, which Flatpak
  downloads at install time; a daily GitHub Actions job follows new
  releases. The user's terms: host only the index, never apps or
  installers, nothing to maintain. Don't propose Flathub submissions or
  hosting the apps (an earlier attempt at both went against those terms).
  Checked: a system-wide install of MailCraft from the published, signed
  index ran (`mailcraft-cli --version`), and Patrone Software showed
  PdfCraft's page with Install. Its signing key's backup is on the VM in
  `~/.local/share/patrone-app-index/gnupg`. AniCraft (no releases) isn't in
  it; see `software/README.md`, "The Craft apps".

- **The desktop bubble** (the user's design, 2026-10-26;
  `shell/gnome/README.md`, "The desktop bubble"): in the first step's
  packages (taskbar 0.4.3; before 0.4.0 the taskbar drew the bubble
  itself, with its own icons and no dragging). The user
  installed 0.3.10 with Settings 0.3.5 and GNOME Settings patrone7 on
  2026-10-27, but at sign-in the taskbar failed (the desktop has no screen
  for a moment over Remote Desktop; README, "The Recycle Bin on the
  desktop"), so GNOME's top bar showed and the bubble never started.
  0.3.11 (installed) fixed that; 0.3.12 put the bin half over the bubble's
  edge, and on it even while it's off the desktop. After signing in, a
  blue pill at the left edge bounces with a tip until it's clicked. Click
  it or drag it out: the desktop's files in a bubble, the Recycle Bin on
  its edge. Patrone Settings → Desktop (or GNOME Settings → Patrone) turns
  it off.

- **The terminal at sign-in, over the wallpaper's logo or centered** (the
  user's asks, 2026-10-26; `shell/gnome/README.md`, "The terminal at
  sign-in"): in the step above's packages. After signing in again,
  Patrone Terminal should open over the logo,
  between the `>_` prompt at the bottom left and the dashed line at the top
  right. Patrone Settings → Desktop → Terminal at sign-in (or GNOME
  Settings → Patrone) picks Centered instead, from the next sign-in.
  Checked only in a throwaway shell, never at a real sign-in.

- **Install Patrone Task Manager 0.2.0** (section 15). The user has 0.1.0
  and `patrone-desktop` 0.1.2 installed already (Resources is gone):
  `sudo apt install ~/src/patrone-linux/out/patrone-task-manager_0.2.0_amd64.deb`,
  then close Task Manager so the new version starts. Try Ctrl+Shift+Esc, the taskbar's right-click "Task Manager", End
  on an app, and Start or Stop on a system service (asks for the password).
  On the Users page, select Base and choose Sign out: the password dialog
  should appear (cancel it, or give the password to really sign Base out).
  That dialog couldn't be shown in a throwaway shell.

- **Install the new packages, turn on Patrone Remote Desktop, reboot, then
  connect** (needs their sudo password; section 3 has the steps). Their
  favorites and icon theme were already reset to Patrone's defaults
  (2026-10-26, over SSH).
- **Chromium's first run** shows a "Chromium Additional Terms of Service"
  dialog that says "This Space Intentionally Blank": Chromium's placeholder
  where Google Chrome shows Google's terms (the strings sit with "Welcome to
  Chromium" in Chromium 156's resources). Accepting it is harmless. Find what
  triggers it and whether a policy or flag skips it, then add that to
  `patrone-default-apps`, since every new Patrone user would see it.
- **Login screen style:** the user asked for an "enterprise and/or school"
  look. It now keeps the list of users, restyled. Ask whether they'd rather
  have the username-and-password form schools and companies use
  (`disable-user-list=true` in `system/gdm/95-patrone-settings`), and
  whether to add a banner message.
- **Switch the login shell back to bash:** on 2026-10-26 the user ran
  `chsh -s /usr/bin/zsh` on my earlier advice, then decided on bash. They run
  `chsh -s /bin/bash` (asks for their password).
- **Installed 2026-10-26 by the user**: `patrone-gnome-taskbar` 0.3.0,
  `patrone-settings` 0.2.0, `patrone-branding` 0.1.5, `patrone-remote-desktop`
  0.4.3, `patrone-desktop` 0.1.1, `patrone-file-manager` 0.2.0 and
  Patrone's GNOME Settings (`...patrone1`), and GNOME Settings' build
  dependencies. Still to hear how they went: the Super key, the network
  menu's chevron, Shut down in a real session, Chromium's pages in Noto
  Sans and the wallpapers in Settings → Appearance.
- **The user's report on 0.3.0 (2026-10-26)**: "The themes are not the
  correct theme, the icons in the buttons are inconsistent, the start and
  buttons don't show the change, The taskbar layout doesn't change. Classic
  is royal, Royal is something else." Their GNOME Shell had started at
  15:26 and 0.3.0 was installed at 15:35, so the desktop ran 0.2.0's
  taskbar (the second time: Patrone Settings' first clicks hit the same
  thing at 14:59). And the title bar buttons did differ from app to app
  (section 9). Fixed in this round: install `patrone-gnome-taskbar` 0.3.1
  (it now says when to sign out), `patrone-settings` 0.3.0 (button styles
  of their own), `patrone-file-manager` 0.2.3 (GTK's own window buttons)
  and GNOME Settings `...patrone2` (the style choice, and pictures that
  were nearly invisible in a window of ordinary height), **then sign out
  from inside the desktop and back in**.
- **The user's next report (2026-10-26)**: the theme didn't change window
  title bars (only Classic and Royal had any), the Dots buttons touched
  File Manager's window edge, and the Start menu's default icons "look like
  shit, make clear ones". Then: the taskbar's grey should match the
  wallpaper; "the taskbar took about 30 seconds to update that I was
  running a new app"; and "make our overridden icons look better, and allow
  an option to use the default app icons". Fixed in this round: install
  `patrone-settings` 0.3.1 (title bars for every theme, a gap at the
  window's edge, the sign-in entry that writes the style sheets runs
  again, a new icon, the Color and Pin icons choices),
  `patrone-gnome-taskbar` 0.3.2 (its color from the wallpaper, Patrone's
  own pin icons or the apps', apps showing the moment they're opened),
  `patrone-terminal` 0.1.2 and `patrone-file-manager` 0.2.5 (new icons) and
  GNOME Settings `...patrone3` (the two new choices), then sign out and back
  in. Open apps keep their title bars until they're reopened: GTK reads
  `gtk.css` once. File Manager's tab bar doesn't follow the theme's title
  bar colors yet.
- **The 30 seconds were the whole desktop stalling, not the taskbar**: the
  user journal at that time has at-spi "Disabling unresponsive app" for
  GNOME Shell's own process, then `JS ERROR: Gio.IOErrorEnum: Timeout was
  reached` 25 seconds later (D-Bus's timeout) with no stack, then the RDP
  connection dropping. The taskbar makes no blocking calls, and in the
  throwaway shell apps reached it 20 ms after their windows. Which call
  stalled the shell isn't known; the session runs Ubuntu's ding,
  tiling-assistant, snapd-search-provider and web-search-provider
  extensions. If it happens again, note the time and what was clicked.
- **Installed 2026-10-26 by the user, then signed in again**: the round
  above (taskbar 0.3.2, Settings 0.3.1, Terminal 0.1.2, File Manager 0.2.5,
  GNOME Settings `...patrone3`); the taskbar loaded with no errors. Next,
  install `patrone-gnome-taskbar` 0.3.3 (the Recycle Bin docked to a corner
  of the desktop, and no Home icon, the user's asks), `patrone-terminal`
  0.1.3 (the see-through background, "like kali does it"), with
  `patrone-unpack` 0.1.1 and `patrone-default-apps` 0.1.3 (section 13),
  then sign out and back in. Checked in the throwaway shell only: each
  corner, dragging, double-click opening File Manager on Recycle Bin, the
  menu and the Empty dialog (cancelled; emptying wasn't tried there, since
  the trash service also empties other drives' trash); the desktop icons
  extension can't run there, so hiding Home was checked by compiling the
  override with the system's schemas.
- **Describe the rest of the desktop design** (desktop icons, windows, look
  and feel, login and lock screens; the taskbar and Start menu are decided,
  section 7) before milestone 4; then mockups, then `docs/DESIGN.md`.
- **Confirm in the RDP session** that resizing the `mstsc` window resizes the
  desktop, and that copy and paste work both ways. Set 200% scale in Settings
  → Displays if text is too small.
- **ASCII logo file (optional now).** The user shared an ASCII-art version of
  the logo as an image in chat, never saved to disk. The fastfetch logo is now
  traced from the concept instead; their file would only matter for
  `/etc/issue` or if they prefer that look.
- **Try the compositor's input handling** (section 2).
- **File Manager 0.2.10's fixes** (section 8; the user runs 0.2.9) come with
  0.3.1 in the first step: a name being typed (after Compress, New folder,
  New file or F2) is kept by a click anywhere else, a new ZIP shows its
  icon and size at once, and going back or up (toolbar, mouse buttons,
  keys, address bar, tree) selects the folder you came out of.
- **Install Patrone Unpack** (section 13), from SSH or a terminal:
  `sudo apt install ~/src/patrone-linux/out/patrone-unpack_0.1.1_amd64.deb
  ~/src/patrone-linux/out/patrone-default-apps_0.1.3_all.deb` (apt adds
  Flatpak, which `patrone-unpack` depends on). Then double-click
  `~/Downloads/pdfcraft-0.9.0-linux-x86_64.flatpak` in File Manager: Install
  needs no password, adds Flathub for its runtime the first time, and Open
  starts PdfCraft (it reaches Start after the next sign-in, since Flatpak is
  new to the session). Also open a zip or tar.gz (Extract all, Extract here)
  and a `.deb`: Install asks for your password with "Installing software for
  everyone on this computer needs an administrator's password".
- **PatroneOS's fonts are the system's** (section 14): once the PR is
  merged, the daily job publishes `fonts-patroneos` and the rest within a
  day. On PatroneDev they're installed already (2026-10-29); sign out and
  back in to see PatroneOS Sans on the desktop and in apps, and PatroneOS
  Mono in Patrone Terminal.

## 1. Patrone Terminal

Verified by hand (through GTK's Broadway backend, see the terminal README):
copy on select, right-click paste, link detection on hover, the close
confirmation (and that confirming stops the program), Preferences saving and
applying, tabs, closing a tab. Not exercised: Ctrl+click actually opening a
browser (it would have opened one on the user's screen), keyboard shortcuts
(Broadway doesn't receive letter keys from browser automation).

Next:

- Ask the user how it looks and feels in the RDP session, especially the
  palette in `terminal/src/theme.rs` and the font size at 4K.
- A real icon once the Patrone mark has a vector master.
- Install the packages for real (with sudo) to check the alternatives and
  `/etc/skel` behaviour on a live system, then move package builds into
  `sbuild`.

Findings:

- `vte4` 0.10 pairs with `gtk4` 0.11; the crate features are `v0_84` and
  `gnome_50`. Everything builds with Rust 1.93, so `rust-version = "1.93"`.
- Ubuntu's Yaru theme fails to load in plain (non-libadwaita) GTK4 apps on the
  VM: "resource /com/ubuntu/themes/Yaru/4.0/gtk.css does not exist". The
  terminal uses GTK's built-in `Default` theme, dark. The shell and settings
  apps will hit the same thing.
- GTK 4.22 draws with Vulkan (Mesa lavapipe) in the VM. When the console
  session ended under an open test window, GTK logged 500 MB of "surface
  lost" warnings in 30 seconds before exiting. Send test windows' output to
  `/dev/null`.
- Windows opened over SSH don't take keyboard focus, so they don't interrupt
  the user. The user's desktop is now the Remote Login session; the console
  shows the login screen, so `scripts/hyperv/vmshot.ps1` no longer shows it.
- `xdg-terminal-exec` caches its choice in `~/.cache/xdg-terminal-exec`;
  delete it after changing `xdg-terminals.list`.
- `dh_clean` deletes `*.orig` files, including each vendored crate's
  `Cargo.toml.orig` that cargo checksums: `dh_clean -XCargo.toml.orig`.
- Don't run `pkill -f <pattern>` over SSH: the pattern also matches the SSH
  command's own shell, which kills the connection. Anchor it (`^/path`) or
  kill by PID.

## 2. Compositor: milestone 2 done, next milestone 3

`compositor/` (see its README) runs nested in GNOME through Smithay 0.7's
winit backend, with OpenGL ES on Mesa llvmpipe. It draws the wallpaper,
starts `patrone-terminal --standalone` inside itself (falling back to
`ptyxis --standalone`, then `foot`), and handles xdg-shell windows: client
move and resize, Alt/Super+drag, Super+Q, click to focus, popups. Based on
Smithay's `smallvil` (MIT, notice kept in `compositor/NOTICE`). Smithay and
its `anvil` example are MIT; niri is GPL-3, which Patrone's GPL-3.0-or-later
license now lets us borrow from.

Verified: it builds, 13 tests, Clippy and rustfmt pass; its `--screenshot`
frames show the wallpaper, a centered Patrone Terminal, two windows
cascading, focus passing on when a window closes, and the `#030303`
fallback. Not verified (needs input in the user's session): moving,
resizing, the shortcuts, click to focus, menus, clipboard, cursors. Ask the
user to try `cargo run --manifest-path compositor/Cargo.toml -- --modifier alt`
in the RDP session.

Findings:

- EGL works for nested clients in the Remote Login session (llvmpipe, GLES
  3.2); the "failed to get driver name / ZINK" lines are only Mesa's GPU probe.
- GNOME stops sending frame callbacks while its screen is locked, which froze
  `eglSwapBuffers` at Mesa's default swap interval. The compositor turns
  vsync off and times its own frames (60 a second at most, only on change).
- GNOME keeps Super+drag for itself (`mouse-button-modifier`), so nested runs
  use `--modifier alt`.
- Reading back the window's framebuffer broke the next swap, so screenshots
  draw the frame again off screen.
- Ctrl+C or SIGTERM leaves the `wayland-1` socket file behind; the next run
  reuses it.

**Maximize and fullscreen** (`src/maximize.rs`) and **xdg-decoration**
(apps are always told to draw their own title bars) are done. Verified with
`tools/window-states.py`, a GTK window that requests each state on a timer
and prints what it gets: maximize and fullscreen give exactly the output size
(1600×900), restoring returns 640×400, leaving fullscreen returns to
maximized when it was, and a window that opens maximized is placed at the
top left and, once restored, centered to the pixel (measured from a
screenshot). `WAYLAND_DEBUG` showed `zxdg_decoration_manager_v1` offered and
only two window capabilities (maximize, fullscreen); GTK then greys out its
minimize button. Not verified (needs input): Super+Up and Super+F11, and
dragging a maximized window off its maximized state.

**Milestone 3 runs on the real screen** (2026-10-26, on the Hyper-V console
from tty3): `src/udev.rs` is a
DRM/KMS + GBM + libinput backend with a libseat session (based on anvil, cut
down to one GPU and one screen), chosen automatically when there's no
`WAYLAND_DISPLAY`/`DISPLAY`. It accepts software rendering (anvil refuses
it): on Hyper-V, Mesa's `kms_swrast` and `dri_gbm` are installed, which is
how Mutter draws on `hyperv_drm` (`/dev/dri/card1`, no render node). It
draws the cursor from the xcursor theme, switches virtual terminals with
Ctrl+Alt+F1–F12, quits with Super+Shift+E, takes relative and absolute
pointer motion, and picks the largest mode on Hyper-V (whose "preferred" mode
is the 1024×768 boot mode). `data/` and `install-dev-session.sh` add a
"Patrone" login-screen session. Verified: it builds (debug and release), 21
tests pass (mode choice, VT keys, quit shortcut, CLI), Clippy and rustfmt are
clean, nested mode still behaves the same (window-states sequence and a
screenshot), and over SSH it finds `/dev/dri/card1` and fails with a clear
message, since only the session on the screen may open it.

**Console test results.** The user logged in on tty3 in VMConnect (Ctrl+Alt+F3
can be sent with `scripts/hyperv/vminput.ps1 -Combo Ctrl+Alt+F3`) and ran
`~/src/patrone-linux/compositor/target/release/patrone-compositor 2> ~/patrone.log`.
It took the screen at 1920×1080 ("using /dev/dri/card1 (hyperv_drm), drawing
on the CPU"), drew the wallpaper, Patrone Terminal and the pointer, and a
click (sent with `vminput.ps1 -Click`) focused the terminal, which then took
keyboard input; `vmshot.ps1` showed each change. Findings:

- The first run froze after its first frames: no vblank arrived and the
  compositor waited forever. A 50 ms vblank timeout now keeps it going; on
  the second run the display did report vblank ("the display sends vblank
  events") and no timeout was logged, so the cause isn't fully pinned down.
  Watch the log for "no vblank" warnings.
- Harmless warnings on hyperv_drm: "Unable to become drm master" (logind
  already made the fd master), "failed to create signaled syncobj" (no sync
  objects), "Failed to destroy old mode property blob".
- Stopping it with SIGTERM from SSH hands the screen back cleanly; the
  console returns to its text prompt.
- Started from a terminal inside the RDP desktop, it runs nested instead, as
  designed: the log's first line says which.
- Moving a window by its title bar works (the user did it). It left old
  pointer images on screen: Hyper-V's driver copies each frame to its VRAM
  without waiting for Mesa to finish drawing it (dumb buffers have no
  fences). The compositor now waits on the frame's sync point before showing
  it, as anvil does with `renderer_sync`. Not retested on screen yet.
- Patrone Terminal looked unfocused (no text cursor) until clicked, though
  new windows are meant to get focus. Check whether that's real.

Not tried yet: dragging and resizing windows, the shortcuts, and switching
virtual terminals while it runs. Then: the login-screen session
(`install-dev-session.sh`, needs sudo; a login there as the same user ends
the Remote Login session), primary selection, and a keyboard layout from the
system settings.

## 3. Hyper-V guest support

`scripts/hyperv/README.md` has the user-facing steps.

Findings:

- **Working:** Remote Desktop Connection to GNOME Remote Desktop, set up with
  `scripts/hyperv/guest/setup-remote-desktop.sh`:
  - `sharing` (Desktop Sharing): the console session, fixed size. The session
    negotiates the clipboard channel, audio and H.264 graphics.
  - `login` (Remote Login): a separate session on a virtual monitor the size
    of the client, 3840×2160 at 100% scale on the user's 4K monitors. Signing
    in as the user who is logged in on the console **ends the console
    session**, so anything running there (like `claude remote-control`) has
    to be restarted in the remote session.
  - Each mode has its own RDP credentials, and `mstsc` remembers credentials
    per address: after switching modes it offered the old ones ("Could not
    find user in SAM database"). **More choices → Use a different account**.
  - The first attempt of each connection fails with "connection reset by
    peer" during NLA: `mstsc` drops it while asking to trust the self-signed
    certificate, then reconnects.
- GNOME 50 is **Wayland-only**, so Microsoft's xrdp recipe can't work.
- **GNOME Remote Desktop** 50.2 (from its source): NLA only; resizes only on
  virtual monitors (Remote Login, or Desktop Sharing's "extend" mode); no
  vsock.
- **Hyper-V daemons** were inactive because `linux-cloud-tools` was installed
  after boot, after udev had created the devices their units wait for. The
  bootstrap script re-triggers udev. `hv-fcopy-daemon` stays off: its unit
  waits for `/dev/vmbus/hv_fcopy`, which the 7.0 kernel's UIO-based fcopy
  doesn't create (it's only for `Copy-VMFile`). `patrone-guest-tools`
  starts it on the UIO device (section 17).
- **Host:** `EnableEnhancedSessionMode` is True, but `PatroneDev` uses the
  `VMBus` transport. Linux guests need `HvSocket`.
- `freerdp-proxy` (FreeRDP 3.32) runs headless and could sit between VMConnect
  and GNOME Remote Desktop (TLS in, NLA out).
- **Patrone Remote Desktop** (`remote-desktop/`, package
  `patrone-remote-desktop`; its README has the design). The user's
  requirements (2026-10-26): `mstsc` asks for the credentials, those are the
  Linux account's, they open the desktop with no second sign-in, the
  password is checked against the account each time and **never stored**
  (they rejected GNOME keeping its own copy), any account in a
  `remote-desktop` group may use it, off by default, and RDP sessions never
  lock. How:
  - GNOME Remote Desktop 50 can only check its own stored credentials or
    Kerberos (`grdctl rdp set-auth-methods`), never PAM. RDP's NLA can't
    check against PAM either: its NTLM step needs the server to hold the
    password's NT hash already, and `/etc/shadow`'s hashes can't produce it.
    So clients get TLS without NLA, and the password is checked when it
    arrives in the client info.
  - FreeRDP 3.32's proxy (Ubuntu's `freerdp-proxy`) on 3389 as user
    `patrone-rdp`, with Patrone's `patrone-login` plugin (C). Read in
    FreeRDP's source: in TLS mode its `ServerPeerLogon` filter runs before
    the client info arrives, so it can't see the password; the
    `ServerFetchTargetAddr` filter runs after, before the proxy connects on,
    and can pick a custom target (the proxy frees `target_address`). The
    client's settings (`peer->context->settings`) hold the plain password;
    the target connection's settings (`pdata->pc`) are separate.
  - A root helper per request (`patrone-remote-desktop-auth.socket`,
    `Accept=yes`) does PAM (`/etc/pam.d/patrone-remote-desktop`: group,
    UID ≥ 1000, `common-auth`, `common-account`), then `prepare-session`
    sets up the account's GNOME Remote Desktop with `grdctl --headless` (port
    20000 + UID, credentials a random secret in
    `/var/lib/patrone-remote-desktop/secrets`) and starts
    `gnome-headless-session@USER.service`.
  - `grdctl ... set-credentials USER` reads the password from stdin when it
    isn't a terminal, so the secret never appears on a command line. With no
    TPM it stores it in plain text in `credentials.ini` (mode 600): fine for
    a random secret, which is why it must never be a real password.
  - `no-lock` (from `/etc/xdg/autostart`) turns the lock and blanking off in
    seatless sessions, saving the account's own settings, and restores them
    on the console. The user's saved settings were set to GNOME's defaults,
    since an earlier per-user script had already forced them off.
  - Checked: `make check` (also run by the package build), 38 checks: the
    plugin refuses no credentials, a wrong password, "busy", a garbled
    answer and an unreachable helper, always clearing the password; on
    success it targets `127.0.0.1:21000` as the account with the secret,
    re-applies it before connecting and forgets it at session end; the real
    helper refuses malformed requests, odd names and unknown accounts.
    Scripts pass `bash -n`/`sh -n`, the units `systemd-analyze verify`, the
    autostart entry `desktop-file-validate`. The package's contents,
    dependencies, conffiles and not-enabled-by-default install were
    inspected. Built against headers extracted from `freerdp3-dev`,
    `libwinpr3-dev` and `libpam0g-dev` (same versions as the installed
    libraries) because the dev packages weren't installed;
    `bootstrap-dev-vm.sh` now installs them.
  - **Not checked yet:** a real sign-in. Whether `mstsc` prompts for and
    sends the password to a server without NLA, the proxy and plugin running
    under systemd's sandboxing, `prepare-session` as root (`runuser`,
    `systemctl --user` in the account's manager, the headless session
    starting), and display, clipboard and resizing through the proxy.
  - Steps for the user, from SSH (`ssh patronedev`) after logging out of the
    Remote Login desktop (one desktop per account, and turning it on stops
    Remote Login): `sudo apt install` the three new debs in `out/`
    (`patrone-branding_0.1.1`, `patrone-default-apps_0.1.1`,
    `patrone-remote-desktop_0.1.0`), `sudo patrone-remote-desktop enable`,
    `sudo adduser wdaugherty remote-desktop`, `sudo reboot`. Then delete the
    old saved RDP credentials on Windows (`cmdkey
    /delete:TERMSRV/DevDistro-PatroneLinux.local`) and connect. Logs:
    `journalctl -u patrone-remote-desktop` and `journalctl -t
    patrone-remote-desktop-auth` (the user is in `adm`, so readable over SSH).
- **First real try (2026-10-26):** the proxy never started. FreeRDP's INI
  parser rejects a file whose comments start with `#` ("failed to parse ini
  file"); fixed in 0.1.1 with `;`, and `make check` now runs the real
  `freerdp-proxy` on the shipped file (it fails on the old one). With 3389
  free, the user's own Desktop Sharing daemon (set up earlier with
  `setup-remote-desktop.sh sharing`) took it once they logged in on the
  console, so their `mstsc` reached Desktop Sharing: the same session,
  mirrored on both screens.
- **Session model: the user requires Windows behavior** (2026-10-26): one
  session per account; an RDP sign-in takes over the account's existing
  session, and the machine's own screen stops showing it while it's
  remote; never a second session. GNOME can't: its Remote Login kills the
  local session, and Desktop Sharing mirrors it (GNOME's own issue
  gnome-remote-desktop#92 asks for takeover). Proposed: sessions always run
  on the machine's seat (never headless). `gdm-new-session USER` without
  `--headless` asks GDM (`LocalDisplayFactory.CreateUserDisplay`) for a
  normal session; the proxy joins it through the account's own GNOME Remote
  Desktop in Desktop Sharing's `extend` mode (a virtual monitor the client's
  size); the machine's screen switches to the login screen; signing in
  there switches back to the same session (GDM's usual behavior) and ends
  the RDP connection; disconnecting locks the session. **The deciding
  unknown:** whether a GNOME session keeps drawing to its virtual monitor
  and taking input over RDP while another VT (the login screen) is on the
  machine's screen. Test that first, with the user.
- **Built (0.2.0): Windows-style sessions.** The user, 2026-10-26: "I want
  to make it MUCH MORE LIKE WINDOWS". The VT-switch plan above is dead:
  Mutter 50.1's `meta_renderer_pause` (run when its session stops being the
  one on screen) inhibits the frame clock of every view, RDP virtual
  monitors included, so a background session freezes for RDP. Instead, each
  account's one desktop is always GDM's headless GNOME session, and the
  machine's screen is a viewer: the "Patrone Desktop" login session
  (`/usr/share/wayland-sessions/patrone-desktop.desktop`) runs
  `libexec/console`, which asks `auth --console` (over `console.sock`, account
  from `SO_PEERCRED`, no password since GDM checked it) for the desktop and
  shows it with `sdl-freerdp` in `cage`. GNOME Remote Desktop 50.2's headless
  mode accepts a new client and then stops the others
  (`on_session_post_connect` → `maybe_stop_session`, from its source), which
  gives "last one in wins" in both directions. `prepare-session` answers
  "busy" only for a plain GNOME session on the screen (it now looks for
  `libexec/console` in the session's cgroup: logind's `Desktop` is empty for
  GDM's sessions). Checked: the package builds, `make check` passes
  (including `auth --console` refusing without a socket), the viewer check
  calls the user's live GNOME session (5) a plain desktop. Not tried yet:
  everything on the real screen and through `mstsc`. Possible snags: `cage`
  (wlroots) on `hyperv_drm` (the viewer sets `WLR_NO_HARDWARE_CURSORS=1`;
  `WLR_RENDERER=pixman` if it can't render), and whether FreeRDP's
  `/from-stdin:force` reads the secret through `cage`. The user's own GNOME
  Desktop Sharing was disabled from starting at login (it took 3389 before).
  Still to do: lock the screen after idle (end the viewer, back to the login
  screen), and Patrone's own compositor doing this natively.
- **Second try (0.2.0, 2026-10-26):** the proxy ran and listened on 3389,
  but `mstsc` sent no password: the plugin logged "sent no username and
  password". To a server without NLA, Remote Desktop Connection leaves
  sign-in to the server's own screen. 0.2.1 adds `patrone-remote-desktop
  rdp-file` (and `scripts/hyperv/PatroneDev.rdp`) with
  `enablecredsspsupport:i:0` and `prompt for credentials on client:i:1`,
  which should make it ask on Windows and send the credentials. Unverified:
  Microsoft's current RDP file reference doesn't list the second setting.
  The plugin now logs whether a username came without a password. If the
  file doesn't work, the fallback is a sign-in screen served by Patrone
  itself for clients that send nothing (as Windows does without NLA). Also,
  the user was signed in on the screen with Ubuntu's session (GDM
  preselects the last one), which would have been "busy"; they pick Patrone
  Desktop from the gear menu.
- **Third try (0.2.3, 2026-10-26): the RDP file doesn't help.** The proxy
  logged plain `mstsc` twice as "no username or password" and
  `PatroneDev.rdp` as "the username "wdaugherty" but no password": with
  `enablecredsspsupport:i:0` Remote Desktop Connection sends the file's
  username and leaves the password to a sign-in screen on the server. It
  doesn't send passwords to a server without NLA. Two ways forward, put to
  the user: (A) a Patrone sign-in screen served over RDP to clients that
  send nothing (what Windows does without NLA; nothing stored; the prompt
  is in the RDP window; needs a login screen plus a hand-off to the
  desktop, perhaps RDP server redirection), or (B) NLA as Windows does for
  local accounts: FreeRDP's proxy checks NLA against a SAM file of NT hashes
  (`[Server] SamFile=`), which a PAM hook keeps in step with the Linux
  password (on every password change and password sign-in), and PAM still
  checks the real password from CredSSP's TSCredentials on every
  connection. B stores a hash, which the user rejected for GNOME's own copy;
  it can't go stale, though. GNOME's Remote Login as the sign-in screen:
  undocumented when the user already has a desktop (one report says it
  kills the other session).
- **The user chose B (NLA) on 2026-10-26; built as 0.3.0.** The proxy
  requires NLA and checks it against `/var/lib/patrone-remote-desktop/sam`
  (WinPR's format, `user:::NTHASH:`; FreeRDP's NTLM retries the lookup
  without the client's domain, and matches usernames case-sensitively). NLA
  writes the decrypted TSPasswordCreds into the peer's settings; the client
  info that follows clears `FreeRDP_Password` (`rdp_read_info_string` sets it
  to NULL first), so the plugin takes the credentials in its
  `ServerPeerLogon` filter, which runs right after NLA. `auth --remember`
  (from `pam_exec quiet seteuid expose_authtok`, added to `common-auth` and
  `common-password` as "Additional" by a `pam-auth-update` profile) writes
  `remote-desktop` members' hashes with WinPR's `NTOWFv1W`; `pam_exec` runs
  password-type commands only in the update step, when `PAM_AUTHTOK` is the
  new password. Checked: `make check`, 50 checks (NLA path in the plugin, the
  textbook NT hashes of "password" and "Password1", read back with WinPR's
  SAM reader, one entry per account, others kept); a throwaway proxy on
  3399 with a test SAM: FreeRDP's client with the right made-up password got
  "Accepted client" (NLA passed), a wrong one `ERRCONNECT_LOGON_FAILURE`
  (NTLM's MIC check failed). Not yet: `pam-auth-update` on the VM, a real
  hash from a real sign-in, and `mstsc`.
- **0.3.0 on the VM (2026-10-26):** NLA and PAM both worked with the
  user's real password through `mstsc` ("signed in wdaugherty from
  10.0.0.57"). First refusal: they were signed in on the screen with
  Ubuntu's session, so "busy". Second: the headless desktop never started,
  `gdm-new-session: Failed: The connection is closed`, which is Ubuntu bug
  2171951 (filed October 2026): GDM's `gnome-headless-session@.service` runs
  as a DynamicUser, and systemd starts the system bus daemon with
  `SYSTEMD_NSS_DYNAMIC_BYPASS=1`, so the bus can't resolve it and drops the
  connection. 0.3.1 ships the bug's workaround as a drop-in (run it as
  `gdm`), and `prepare-session` now gives up as soon as the unit fails.
- **0.3.1 on the VM:** Remote Desktop reached the background desktop. Then
  the user signed in on the screen, the login screen chose Ubuntu's
  session (remembered), and GNOME's login screen ended the background
  desktop: gnome-shell's `gdm/loginDialog.js` `_findConflictingSession`
  treats any other active or online wayland/x11 session of the account as a
  conflict, shows `ConflictingSessionDialog`, and "continue" calls GDM's
  `StopConflictingSession`. It does so whichever session was chosen, so
  Patrone Desktop would have hit it too. 0.4.0: a login-screen extension
  (`session-modes: ["gdm"]`, enabled in GDM's dconf; GNOME Shell inits
  extensions in every mode, and the gdm mode, built on "restrictive", takes
  them from `enabled-extensions`) overrides `_findConflictingSession` to skip
  seatless sessions, falling back to GNOME's on an error; tested under gjs
  with GNOME Shell's modules stubbed (and the test fails if the seat check
  is removed). Ubuntu's `ubuntu.desktop` is diverted aside; the hidden
  `patrone-gnome.desktop` runs background desktops (GDM's daemon ignores
  `NoDisplay`, libgdm's menu honours it; headless needs
  `X-GDM-CanRunHeadless`). With `ubuntu` gone, GDM's fallback (`gnome`, else
  the first session file by name) is `patrone-desktop`. Not tried yet:
  needs a reboot (GDM reads it all at start).
- **0.4.0 after a reboot:** still "busy". With `ubuntu.desktop` diverted,
  the user's remembered "ubuntu" was invalid, GDM fell back (`gnome`, else
  the first session file by name), and its search includes its own
  `/usr/share/gdm/greeter/wayland-sessions/`, where `gnome-greeter.desktop`
  (`Exec=gnome-session`, `X-GDM-CanRunHeadless=true`) sorts before
  `patrone-desktop`: the screen got a bare GNOME session
  (`XDG_SESSION_DESKTOP=gnome-greeter`). AccountsService still says
  "ubuntu" (GDM didn't save the fallback). 0.4.1: Patrone Desktop is
  `gnome.desktop` (GDM's first fallback, for local and headless), running
  `libexec/session`, which picks the viewer on a seat and
  `gnome-session --session=ubuntu` without one (tested against the live
  seat0 session and a seatless one). Other packages' `gnome.desktop` is
  diverted too.
- **0.4.1 on the VM:** RDP works, and signing in on the screen no longer
  ends the desktop (the login-screen extension works). The screen's viewer
  failed two ways. (1) Screen first: GDM refused the background desktop,
  "There's already an opened session for user": `CreateUserDisplay` calls
  `gdm_find_graphical_sessions_for_username`, which counts any logind
  session of type wayland/x11/mir that's active or online, and the screen's
  own session is one. 0.4.2 starts the desktop from the PAM profile during
  the login screen's password check (`auth --screen-sign-in`, only for
  `gdm-password`/`-fingerprint`/`-smartcard`, not `gdm-autologin`, which GDM
  uses for the background desktop itself), before GDM opens the screen's
  session. (2) With the desktop already running (RDP first): `cage` failed
  `xwayland/xwm.c:592` (SDL had picked X11 through cage's Xwayland), and the
  client crashed with it. 0.4.2 runs the client with `DISPLAY` unset and
  `SDL_VIDEO_DRIVER=wayland`; checked that SDL starts on Wayland that way
  (on the background desktop's display, against a closed port).
- **0.4.2 works end to end (the user, 2026-10-26: "IT WORKS!!!").** One
  NLA prompt with the Linux password, the same desktop on the VM's screen
  and over RDP, each taking it over from the other, no desktop lost.
- **Multiple monitors (the user's question, 2026-10-26).** The desktop's
  monitors are virtual: GNOME Remote Desktop builds one per monitor in the
  client's layout (its "Layout manager", up to a maximum count; layouts can
  change mid-session over the display-control channel). Over RDP with
  "Use all my monitors" the desktop should get the client's monitors;
  connecting from a one-monitor client moves every window onto it, and
  going back to more monitors doesn't put them back (GNOME doesn't remember
  per-layout positions; Windows does, and Patrone's compositor could). Not
  yet seen through the proxy. The screen's viewer runs the client
  full-screen without `/multimon`, and `cage` extends across all outputs by
  default (`-m extend`), so a multi-monitor PC would likely show one monitor
  or one wide one. To do: `/multimon` in `libexec/console`, tested with a
  QEMU VM with several displays inside the dev VM (Hyper-V's console has
  one).
- Packaging lessons: debhelper replaces `#DEBHELPER#` even inside comments
  (0.2.1's install failed; `build.sh` now refuses it), and for units
  installed with `--no-start` its generated `preinst` stops them on every
  upgrade without starting them again (0.2.3's `postinst` restarts enabled
  ones).
- The earlier per-user attempt (a `headless` mode in
  `setup-remote-desktop.sh` with the Linux password as GNOME's credentials,
  and `rdp-no-lock.sh`) was removed: it stored the password, which the user
  rejected.

Plan:

1. **Dev VM:** `mstsc` with Remote Login gives clipboard and full resolution.
   Resizing with the window is expected but unconfirmed: the session was
   still 3840×2160 with no resize logged when last checked. The VMConnect
   enhanced session works on the dev VM since 2026-10-28
   (`patrone-remote-desktop` 0.5.0, with the HvSocket transport).
2. **Patrone ISO:** done for the integration services by
   `patrone-guest-tools` (section 17), with the HWE kernel's cloud tools
   rather than `linux-cloud-tools-virtual`. Enhanced Session (RDP over
   vsock) is `patrone-remote-desktop` 0.5.0's; the next ISO has it.
3. **Patrone Desktop:** GNOME Remote Desktop only works with Mutter, so
   Patrone needs its own RDP path: implement the screen-cast and
   remote-desktop D-Bus APIs GNOME Remote Desktop uses, or write
   `patrone-remote-desktop` on FreeRDP's server library, listening on vsock
   as well as TCP. Clipboard through RDP's clipboard channel, resizing
   through display control, and `hyperv_drm` resolution changes in the basic
   console.

## 4. The web browser: none preinstalled, the user picks one

**Since 2026-10-28** PatroneOS comes with no browser (the user: "Ask
the user when they click on the browser with a modal if they want to
install Chromium, Fire Fox, Brave, etc, then use the current patrone apps
to open that up and set up the install"). **Patrone Browser Chooser**
(`browser-chooser/`, `patrone-browser-chooser` 0.1.0) comes up from the
taskbar's Web browser button and from any link opened while no browser is
installed. It offers Chromium, Firefox, Brave, Google Chrome, Microsoft
Edge, Vivaldi, Opera, LibreWolf, Zen Browser and GNOME Web, all from
Flathub (Patrone Software doesn't browse the Snap Store), and runs
`gnome-software --install=system/flatpak/flathub/<ID>/stable`, so Patrone
Software opens on the browser's page already installing it. When the
browser's launcher appears it becomes the account's default for the web,
opens with the waiting link, and the chooser closes. Also changed:
`patrone-default-apps` 0.2.0 (no `chromium-browser`; the chooser last in
`/etc/xdg/mimeapps.list`'s web lists, after the browsers it offers),
`patrone-gnome-taskbar` 0.4.4 (the Web browser slot shows with no browser
and opens the chooser), and `iso/build-iso.sh` (no Chromium in the seed,
`chromium-browser` purged with `firefox`, and the build stops if a browser
is left in a layer).

Checked (2026-10-28) in a fresh install of an ISO built with these, in a
new Hyper-V VM, `PatroneBrowserTest` (now off; delete it when done with
it):

- The installed system had no browser: no `firefox` or `chromium-browser`
  package, no browser snap (the seed has none), no Flatpak apps. Both
  layers' manifests list only `patrone-browser-chooser`.
- At first login, the taskbar showed the Web browser slot. First-login
  setup's release notes (its own web view) → "open in browser" brought up
  the chooser with "Then it opens documentation.ubuntu.com."; Not now
  closed it; the taskbar button by itself brought it up too.
- Firefox → Install opened Patrone Software on Firefox's page (Flathub,
  Verified), already installing; it asked for the password once
  ("Authentication is required to install software"). When it finished,
  Firefox opened by itself on the release notes link and the chooser was
  gone; the taskbar slot showed Firefox's window.
- `gio mime x-scheme-handler/https` and `text/html` gave
  `org.mozilla.firefox.desktop`; the account's `~/.config/mimeapps.list`
  had it for all four web types. `xdg-open https://example.com/` from the
  terminal opened a tab in Firefox.
- Flathub's catalog was already there at the first click: Patrone Software
  fetched it at sign-in (08:40:40, a minute before). So the chooser's own
  fetch for a catalog that's missing (`flatpak update --system --appstream
  flathub`) wasn't needed; run by hand in the desktop it finished with no
  password prompt.
- **Nothing brings Firefox back** (the user worried it would): after
  first-login setup, `snap list` had no browser (bare, core24,
  firmware-updater, gnome-46-2404, gtk-common-themes, hwctl, mesa-2404,
  prompting-client, snapd, snapd-desktop-integration); dpkg knew no
  `chromium-browser` and had `firefox` as not installed (`un`);
  `apt-cache policy firefox` showed no installed version (the stub
  `1:1snap1-0ubuntu9.1` is only a candidate). After `sudo apt update`,
  `sudo apt full-upgrade -s` would upgrade 88 packages, install none and
  remove none (two held back by phasing: `ubuntu-pro-client` and its
  `-l10n`), with no browser in its output. `snap refresh --list` offered
  only the installed system snaps. Patrone Software's Updates page had
  System Updates and Files, and no browser among the system updates.

Not checked: the slot's right-click menu with no browser (it should leave
the chooser out of "Default web browser"); a second link while the chooser
is open; the "On this computer" list (a browser installed from elsewhere);
Chrome's and Edge's installs, which download from Google and Microsoft;
the dark style. Found on the way:

- **The installer still says "web browser"**: its Applications page
  describes the default selection as "Just the essentials, web browser and
  basic utilities" (Ubuntu's text, in `ubuntu_provision`'s strings). It
  needs a line in `installer/debrand_l10n.py` and the installer rebuilt.
- **First-login setup's "View release notes"** shows Ubuntu's release
  notes (documentation.ubuntu.com, with Ubuntu's logo), for section 16.
- **Flatpak apps can't read Patrone's title button style**: Firefox logged
  "Failed to import: ... /usr/share/patrone/titlebuttons/wide/gtk-3.0.css:
  No such file", the account's `gtk.css` importing a file outside the
  sandbox. Harmless (Firefox draws its own title bar), but GTK 3 Flatpak
  apps won't get the style.

### Before 2026-10-28: Chromium instead of Firefox

Decided 2026-10-25: Canonical's Chromium snap, kept current by snapd
(stable was 156, two days old, when checked), customized with enterprise
policies rather than forked or rebranded.

Done 2026-10-26 (the user asked to replace Firefox): `patrone-default-apps`
(`default-apps/`) depends on Ubuntu's `chromium-browser` stub, which
installs the snap, makes Chromium the default browser through
`/etc/xdg/mimeapps.list`, and puts it in the favorites in place of Firefox.
Checked: it builds; with a stand-in `chromium_chromium.desktop`,
`gio mime` picks Chromium for `https` and `text/html`; the favorites
override gives the new list in both the `ubuntu` session and plain GNOME.
Not checked until the user installs it: the snap installing, links opening in
Chromium. Findings:

- Ubuntu's `firefox` package is a stub that installs the snap. Its
  `preinst` runs `snap install firefox` on every install **and upgrade**.
  `ubuntu-desktop-minimal` only recommends it, so since 2026-10-28 the ISO
  purges it instead of holding it (a hold relies on every updater honouring
  it, and `1:1snap1-0ubuntu9.1` is in `resolute-updates`).
- Ubuntu's `ubuntu-mimeapps.list` and `gnome-mimeapps.list` in
  `/usr/share/applications` name Firefox first. A list in `/etc/xdg` is read
  before them (the session's `XDG_CONFIG_DIRS` is
  `/etc/xdg/xdg-ubuntu:/etc/xdg`), and a user's
  `~/.config/mimeapps.list` before that.

What was still to do then, Chromium's enterprise policies (homepage,
bookmarks, search engine, `BrowserThemeColor` in
`/etc/chromium-browser/policies/managed/`, read through the snap's
`etc-chromium-browser-policies` plug), no longer applies: there's no
preinstalled browser to set up, and Flathub's Chromium reads its policies
from inside its sandbox.

## 5. fastfetch logo

`branding/fastfetch/` has the system-wide config (fastfetch 2.57 reads
`/etc/xdg/fastfetch/config.jsonc`) and the traced logo; `patrone-branding`
installs both. With Patrone's own `os-release` (`ID=patrone`), fastfetch
would otherwise fall back to Ubuntu's logo through `ID_LIKE`.

## 6. Branding (patrone-branding)

The user asked (2026-10-26) to remove the Ubuntu logo everywhere, then for a
login screen and avatar that don't look like Ubuntu, "more enterprise and/or
school". `branding/README.md` has the table of what replaces what. What was
checked: the package builds; its file list, conffiles and maintainer scripts
were inspected; the settings override compiles with `glib-compile-schemas
--strict` against Ubuntu's schemas and gives the new defaults (with no user
settings: the Patrone wallpaper, the dock in `disabled-extensions`); the
login theme has all 11 of Yaru's resources plus Patrone's rules at the end of
`gdm.css`; `agetty --show-issue` renders the console banner as
"PatroneOS 0.1" over "DevDistro-PatroneLinux (10.0.0.61) on tty1". Not
checked until the user installs it and reboots: the boot splash, the boot
menu name, the login screen's look, Settings → About, the login message, and
that the dock goes away.

After the reboot: `vmshot.ps1` on the login screen (before anyone logs in on
the console); `cat /etc/os-release`; `update-alternatives --query
default.plymouth`, `text.plymouth` and `gdm-theme.gresource`; `grep -i
patrone /boot/grub/grub.cfg`; an SSH login for the message; the RDP desktop
for the dock, the accent and the terminal opening by itself.

Findings:

- Ubuntu sets the wallpaper, lock-screen picture and favorites again for its
  own session (`[org.gnome.desktop.background:ubuntu]` and so on), and the
  user's GNOME session runs with `XDG_CURRENT_DESKTOP=ubuntu:GNOME`, so a
  plain default loses there. Patrone's overrides set both; checked with
  `gsettings get` under both desktop names against a scratch copy of the
  compiled schemas.
- Ubuntu's login screen looked like Ubuntu through four things: its logo,
  the orange accent (`accent-color='orange'` from
  `10_ubuntu-settings.gschema.override`, used 211 times in Yaru's login
  CSS), Yaru's warm grey background (`#lockDialogGroup`, `#222222`) and the
  Ubuntu Sans typeface.
- Plymouth's initramfs hook copies the theme folder with `cp -a`, keeping
  symlinks as symlinks, and copies another folder only if `ImageDir` names
  it. Links from the Patrone theme to the spinner's images would have
  dangled at boot, so the package copies them.
- GDM's greeter (`gnome-session-service --session=gnome-login`) doesn't run
  `/etc/xdg/autostart` entries; the user's GNOME session does (it ran
  `evolution-alarm-notify` and `snap userd`, the greeter didn't). No systemd
  `app-*@autostart` units exist: GNOME 50's session starts them itself.
- `desktop-file-validate` rejects `NotShowIn=Patrone;`: unregistered desktop
  names need an `X-` prefix. `XDG_CURRENT_DESKTOP=Patrone` works anyway, but
  matters if Patrone's session ever reads autostart entries.
- Non-interactive SSH commands run in the user's login shell, still zsh,
  which doesn't load `~/.cargo/env`: prefix `PATH=$HOME/.cargo/bin:$PATH` for
  builds over SSH.
- The dock is turned off through `disabled-extensions`, whose description
  (`gsettings describe org.gnome.shell disabled-extensions`) says it applies
  "even if loaded as part of the current mode" (the `ubuntu` mode loads the
  dock). Confirm after the reboot. A session that's
  already running keeps its dock until the next login.

Later: the desktop's own typeface (still Ubuntu Sans; part of the design
discussion), the lock screen inside a GNOME session (Yaru's, with the new
blue accent), and a Patrone-made avatar set for Settings → Users.

## 7. Patrone taskbar (patrone-gnome-taskbar)

The user's taskbar decisions (2026-10-26) and how it works are in
`shell/gnome/README.md`. In short: a taskbar that works the way Windows
users expect, left-aligned and compact ("smaller, not taking up so much
real estate"; built at 32 pixels, then made a third larger, 42, once the
user had used it), in place of GNOME's top bar; the logo's "P" opens the
Patrone menu, where typing searches at once (GNOME's search providers, then
the web); a search button; pinned slots that open the default web browser,
terminal, file manager and mail app, with that app's windows under the
slot; running apps, pinnable; right-click jump lists that also hold the
configuration. Built as a GNOME
Shell extension for now (the user said "Build it out"); the design carries
over to Patrone's own shell.

Checked in a headless GNOME Shell 50.1 on the VM (`shell/gnome/tests/
headless.sh`, light and dark): see the README's "Trying it" for the list.
The package builds and installs its schema with the system's, where a system
extension finds it; its override compiles with Ubuntu's schemas and turns the
extension on under both desktop names. Not checked until installed: the real
Super key (a simulated one doesn't reach GNOME's Super-key detection in a
headless shell; GNOME's overlay-key signal was tested instead), the power
options a real session allows, and the RDP and console views.

To install and load it:

```
sudo apt install ~/src/patrone-linux/out/patrone-gnome-taskbar_0.2.0_all.deb
```

then **log out from inside the desktop** and sign in again: GNOME Shell
loads extensions only when it starts. With the taskbar, that's Start → the
account picture → Sign out (in 0.1.2 and before, Start → power → Sign out);
with GNOME's top bar, its power menu, top right, → Log Out.
With Patrone's Windows-style sessions, disconnecting Remote Desktop or
signing out of the screen's session only disconnects, and the desktop, with
its old shell, keeps running; until it's restarted, `gnome-extensions info
patrone-taskbar@patroneos.com` in it says the extension "doesn't exist". An
account that has its own `enabled-extensions` list keeps it;
`gnome-extensions enable patrone-taskbar@patroneos.com` adds the taskbar.

Findings:

- GNOME Shell 50 picks `stylesheet-light.css` or `stylesheet-dark.css` from
  an extension by the system style, and St's CSS supports `@import`, so the
  sizes live once in `common.css`.
- A hidden actor tracked as chrome still reserves its space (the top bar's
  strut stays), so the taskbar untracks `panelBox` while it's hidden.
- Clutter's `BinLayout` only honors a child's alignment where the child
  expands, and expanding sideways propagates up and stretches the button
  along the bar: the taskbar buttons' insides expand vertically only.
- The overlay-key handler is blocked with
  `GObject.signal_handlers_block_matched(global.display, {signalId:
  'overlay-key'})` and unblocked when the extension turns off.
- A throwaway shell's document portal mounts a FUSE folder in its runtime
  folder; `headless.sh stop` ends everything on the test's D-Bus session and
  unmounts it.
- Ubuntu's desktop icons extension (DING) ends every desktop-icons process
  of the account when it starts, across sessions: each throwaway shell made
  the user's real desktop relaunch its icons (seen in the user journal,
  "Launching DING process" at each test start). `headless.sh` now turns DING
  and the snap permission prompts off in the throwaway shell.
- **0.1.1 in the user's desktop** (2026-10-26, over Remote Desktop): "some
  strange icons in the middle", and a right-click on what looked like empty
  space "logged me out". The desktop wasn't logged out (its shell kept
  running); the Remote Desktop connection was cut three times, each logged
  by GNOME Remote Desktop as "Disconnected by EIS" with
  `ERRINFO_RPC_INITIATED_DISCONNECT`. The icons were GNOME's screen-sharing
  indicator (`ui/status/remoteAccess.js`): a shared-screen icon and a stop
  square, unstyled outside the top bar (the theme styles it under
  `#panel`), about 100 pixels wide, and stopping the sharing on a press of
  any button. In a desktop with no seat (mutter: "No seat assigned, running
  headlessly") the Remote Desktop connection is itself a screen-sharing
  handle, so the indicator is always on. A throwaway shell has no client,
  so it never showed there. 0.1.2 hides it when
  `global.backend.is_headless()`. A right-click on the bar's real empty
  space opened the taskbar's own menu (Task Manager started at 13:09).
- GNOME's top bar buttons (`PanelMenu.ButtonBox`) set `x_expand`, but
  `St.Bin` containers don't pass it up, so the tray didn't stretch; 0.1.2
  sets `x_expand = false` on the moved boxes anyway.
- **0.1.3 in the user's desktop**: the quick settings' "Wired" chevron did
  nothing. Not the taskbar: GNOME binds the network toggles' `reactive` to
  polkit's `org.freedesktop.NetworkManager.network-control`, which is
  "yes" only for local sessions, and polkit counts a session as local only
  when it has a seat. Patrone's desktops have none (logind: Remote=yes, no
  seat), so polkit treated them like SSH logins. Fixed in
  `patrone-remote-desktop` 0.4.3 (section 3, `remote-desktop/README.md`
  step 7).
- **0.2.0** (the user asked for dragging, a taskbar size setting, and
  themes): dragging uses GNOME's `ui/dnd.js`. `set_size()` fixes an
  actor's height, which Clutter then reports as its preferred height, so
  the bar never took a new size until `_relayout` dropped it first. St
  can't take negative margins (the classic Start button had one, and the
  whole bar's layout came out at x = 4294967296), and a style sheet's own
  rules beat those of a sheet it imports, however specific (the classic
  bar's colors, and the light style's screen-sharing pill, lost to the
  light sheet's until moved into it). In the throwaway shell, an app misses
  a click whose pointer arrives with it; `headless.sh click` now moves
  first, and the first click into a new window was still lost.
- `pkill -f NAME` matched the SSH command that ran it, and could have hit
  the same app on the real desktop: `headless.sh close-apps` ends only the
  apps it started.
- **0.3.0** (the user, 2026-10-26: Classic "shouldn't look like windows 10
  royal, it should just look a bit more basic", then "create one that is
  just like the royal theme but not 'start'"; "allow the P to be replaced
  with a series of other icons"; "the taskbar should have its own set of
  possible layouts"): Royal is the old Classic without the "start" label,
  Classic is plain and square, and Modern got a look of its own (rounder
  buttons, no line, blue indicators) once it no longer centered the
  buttons. Layouts are their own setting: Left, Centered, Split and Top.
  At the top, menus open downward (`updateArrowSide`, the tray menus too),
  tooltips go below, and the indicators run along the screen's edge. Eight
  Start icons; the `-symbolic` ones, loaded as file icons, take the bar's
  text color. Checked in the throwaway shell: each theme in light and dark,
  each layout, Start opening downward from the top, every icon (white on
  Royal). A highlight on the tray at the top was the test's own pointer,
  left where the tray landed.
- **0.4.0** (2026-10-27, with File Manager 0.3.0): the desktop and the
  bubble are File Manager's (section 8). The taskbar starts
  `patrone-file-manager --desktop` as its own Wayland client
  (`desktopHost.js`), makes "Patrone Desktop" a desktop window and keeps
  "Patrone Desktop Bubble" above the windows, unseen until the pill opens
  it; the two talk over File Manager's standard input and output. It
  turns Ubuntu's desktop icons off through `disabled-extensions` once
  File Manager's desktop shows (`ding-replaced` remembers, so they come
  back only if the taskbar turned them off and File Manager's desktop
  can't run). `bubbleShape.js` moved into File Manager
  (`src/bubble.rs`). Found while testing: with a client's surface under
  them, GNOME 50 ended the pill's and the Recycle Bin's drag gestures the
  moment they were pressed (a drag opened the bubble like a click; the
  bin couldn't be moved), so File Manager's windows stop taking the
  pointer while it's on one of them (`shieldPointer`). And the bubble's
  window was first half the screen and clear outside the bubble, which
  swallowed drops meant for the File Manager window under it; it's the
  bubble's size now. Details and what was checked: `shell/gnome/README.md`,
  "The desktop and the bubble: File Manager's windows".

## 8. Patrone File Manager (patrone-file-manager)

Built 2026-10-26 at the user's request: a file manager that works the way
Windows users expect, with the options and right-click menus they know, in
Patrone's own look, "All RUST if it makes sense". The user named it **File
Manager** ("Patrone File Manager"): app ID `org.patrone.FileManager`,
command `patrone-file-manager`. Rust and gtk4-rs without libadwaita; GTK's
Default theme restyled by `data/style.css`, light and dark. For icons the
user asked for "nice versions of what Microsoft uses ... professional, but
not exact copies", so every icon is drawn for Patrone (`tools/make_icons.py`
for the color icons, `src/glyph.rs` for the line glyphs) and none of
Microsoft's files are used.
`file-manager/README.md` lists what it does.

Verified on the dev VM:

- It builds; 12 unit tests pass (sizes, natural sort, sorting and grouping,
  search matching, ZIP and TAR round trips, every glyph path); Clippy and
  rustfmt are clean. The packages build (`patrone-file-manager` 0.1.0, whose
  build runs the tests; `patrone-default-apps` 0.1.2); their contents and
  dependencies were inspected; the desktop files validate. In a scratch XDG
  setup with a stand-in binary on `PATH`, `gio mime inode/directory` picks
  File Manager from the new `/etc/xdg/mimeapps.list`, and the favorites
  override compiles against Ubuntu's schemas and puts File Manager in Files'
  place under both `GNOME` and `ubuntu:GNOME`.
- By hand through GTK's Broadway backend (mouse clicks and single key
  presses): Details and Large icons, switching views keeping the selection,
  each folder's view remembered across restarts, Home, This PC with drive
  usage, Gallery, the Recycle Bin page, the item and background right-click
  menus, Open with (opening on hover), Show more options, the View and See
  more dropdowns, the Details pane, Properties (General, Permissions),
  Folder Options (Show hidden files applied and kept), copy and paste
  ("notes - Copy"), a paste conflict through "Compare info" to "keep both"
  (`budget (2).csv`), the progress window, Delete to the Recycle Bin and Undo
  Delete restoring the file, New → Folder renaming in place, F2 and Ctrl+Z,
  search as you type and Escape, Ctrl+T and closing a tab, Compress to ZIP
  and Extract All, and dark mode.
- A screenshot under Wayland, from the app itself
  (`PATRONE_FM_SCREENSHOT`) in a throwaway headless GNOME Shell: the
  client-side title bar with tabs and window buttons, following the session's
  dark preference.

Not verified yet: drag and drop, Ctrl+C, Ctrl+V and Ctrl+W from the keyboard
(the browser keeps them in Broadway), network places and mounting, removable
drives and Eject, Share, the Open with dialog, Properties' Apply (rename,
Read-only, Hidden, permissions), restoring windows at sign-in,
`org.freedesktop.FileManager1`, and daily use in the user's own desktop.

Findings:

- A GTK 4.20+ app style sheet sees `@media (prefers-color-scheme: dark)`
  only when the provider's `prefers-color-scheme` is bound to
  `GtkSettings:gtk-interface-color-scheme`; without it the app's palette
  stayed light while GTK's own theme went dark.
- `GtkStack` sizes to its largest page unless `hhomogeneous` is off; a
  hidden page's minimum width pushed the visible one off screen inside a
  `GtkPaned`.
- `GtkGridView` items take right-clicks before a bubble-phase gesture on the
  grid sees them: catch them in the capture phase. List cells keep the
  position they were bound at when items are added above them; read
  `GtkListItem:position` instead. GIO reports a new file twice (created,
  then changed), and `GtkDirectoryList` replaces the item, dropping its
  selection; a directory monitor also reported the folder itself once, so
  the view filters out anything not inside the folder it shows.
- Test runs in a private `dbus-run-session` start their own dconf service,
  which the desktop's own one overwrites in `~/.config/dconf/user`: give
  tests their own database with `DCONF_PROFILE` (a file with
  `user-db:pfm-test`).
- Papers 50's `papers-thumbnailer` exits 1 for every PDF on the VM, even run
  by hand, so PDFs show their icon instead of a thumbnail.
- Nautilus owns `org.freedesktop.FileManager1` through D-Bus activation
  (`/usr/share/dbus-1/services/org.freedesktop.FileManager1.service`), and
  Ubuntu's desktop-icons extension starts it at sign-in, so a browser's "Show
  in folder" will keep opening Nautilus until Patrone's ISO leaves Nautilus
  out (or the package diverts that service file).

### 0.2.0 (2026-10-26, after the user tried 0.1.0)

The user's notes on 0.1.0, and what changed:

- "The tab is too large on the initial one, and it must be draggable to
  change the location of the window, just like the header bar ... figure
  out what happens if I want to attach that tab to another file explorer":
  tabs have a fixed width; a window's only tab moves the window; with
  several, dragging along the strip reorders and pulling a tab out (or Ctrl)
  carries it: dropped on another window's title bar it joins that window,
  anywhere else it opens in its own window, and an emptied window closes.
  The tab menu's "Move to new window" now moves the tab, and "Move to
  window" lists the others.
- "I can't seem to close multiple tabs": the tab's press handler claimed
  every left click, which cancelled its own close button's click. Fixed.
- A "hint" option showing what system folders are for, with a banner naming
  the Windows equivalent; the user then asked for it as a View option ("there
  is no help menu"): View → Show → System folder hints, and Folder Options →
  View. Tags beside system folders and well-known files, a banner inside
  them, the table in `file-manager/src/hints.rs`.
- "The permissions window is a bit strange ... it should allow us to mark
  something as executable. If the help/hint area is on ... show what the
  command would be": a new Permissions tab (owner and group, Read/Write/Run
  for owner, group and everyone else, "Allow running this file as a
  program", special permissions, apply inside a folder, the mode as `0755`
  and `-rwxr-xr-x`), changes as administrator through pkexec when needed,
  and with hints on the `chmod`/`chown` commands, with `sudo` when needed.
- Text looked too large and icons too small, and the special folders'
  emblems too small: the text is 9 pt (Windows' size) in Patrone's Noto
  Sans (section 10), list icons are drawn at 20
  px (the artwork's margins make the item about 16), the emblems are 1.4
  times larger, and the system drive and its "P" are larger.
- At `/`, known folders get their own icons (a gear for `/etc`, a terminal
  for the program folders, and so on).
- The empty-folder message sits in the middle of the view.
- A command typed in the address bar runs in that folder, as in Windows:
  apps with windows directly, anything else in a terminal that stays open;
  `cmd`, `notepad`, `explorer` and the like work; `cd PLACE` navigates.

Verified on the dev VM, in a throwaway headless GNOME Shell driven with real
pointer and keyboard input through Mutter's RemoteDesktop API and watched
through its ScreenCast stream (see the README's testing notes):

- Dragging a window's only tab moved the window by exactly the drag.
  Dragging the second of two tabs to the left reordered them. Pulling a tab
  down and dropping it on the desktop opened it in a new window (a floating
  copy of the tab follows the pointer, the original fades). Ctrl-dragging
  that window's only tab onto the first window's title bar added it there
  (with the blue insertion line), and the emptied window closed. Clicking ×
  on an inactive tab closed it. "+" opens a tab.
- `touch made-by-command.txt` typed in the address bar ran in a terminal in
  that folder and created the file; `notepad notes.txt` opened Text Editor
  on the folder's `notes.txt` with no terminal.
- Screenshots: `/` in Details with hints on (tags, banner, the new icons),
  the Permissions tab of a script and of `/etc` (with the "root owns this"
  notice and `sudo chown`/`chmod` commands), the empty-folder message, the
  sample folder at the new sizes, and an icon sheet of every folder and
  drive icon.
- 14 unit tests pass (new: hint lookup, symbolic and recursive modes);
  Clippy and rustfmt are clean; the 0.2.0 package builds.

Not verified yet: applying permission changes (directly or through pkexec's
password prompt, which needs the user), dropping files on a tab, and the tab
menu's "Move to window".

Findings:

- GNOME's Wayland ends a drag dropped where nothing accepts it with
  `GDK_DRAG_CANCEL_ERROR`, not `NO_TARGET`, so tab tear-off checks both.
- GTK 4.22 can't use `xdg_toplevel_drag_v1` (Mutter has it, GTK doesn't use
  it), so a carried tab is a floating copy and the new window opens where
  GNOME places new windows, not under the pointer.
- Mutter's `NotifyPointerMotionAbsolute` takes the stream path as a string
  (`(sdd)`), not an object path.
- `gsettings set` hung in a private `dbus-run-session`; GLib's keyfile
  backend (`GSETTINGS_BACKEND=keyfile`, `XDG_CONFIG_HOME` pointing at a
  scratch folder) avoids dconf for tests altogether.
- Pango adds a hyphen where it breaks a word, which reads as part of a path;
  path and command labels turn hyphens off.

### 0.2.1 (2026-10-26, the user's notes on 0.2.0)

- "I can't close the folder options menu": a closed dialog was never freed
  (its own close button and Escape handler held the window strongly), so
  reopening Folder Options brought back the old one, whose OK and Cancel
  were dead. Dialogs now hold their window weakly, and Folder Options only
  reuses a dialog that's still on screen. Verified with real clicks: close,
  reopen, Cancel; reopen, OK.
- "Instead of using a tag ... add a column ... use a field that blends in":
  a Purpose column after Name in Details while hints are on (plain text like
  the other columns, sortable, with the explanation as its tooltip); Tiles
  and Content add it after the type; the other views put it in the tooltip.
  Screenshots of `/` in Details and Tiles.
- "In the path bar, I should be able to see the full path as breadcrumbs to
  a point (just like in windows)": the breadcrumbs start at This PC and the
  drive for every local folder (they used to start at the home or user
  folder); a layout manager (`CrumbLayout` in `addressbar.rs`) shows as many
  as fit from the end and folds the rest into a « button that lists them.
  Verified with real clicks: a parent breadcrumb goes up, « lists the folded
  folders nearest first, and a maximized window shows the whole path.
- Asked how to give a user named "base" access: there's no such account on
  the VM. The Permissions tab follows Linux's owner, group and everyone
  else; giving one more named person access needs an ACL (`setfacl`, in the
  `acl` package, which is installed), which the tab doesn't offer yet.

### 0.2.2 (2026-10-26)

The user asked for Windows-style per-person access in Permissions ("Add...
a user or group with Read/Write/Run ... using ACLs"). The tab now reads the
item's ACL (`getfacl`), lists everyone given access by name under Owner,
Group and Everyone else with their own boxes and a remove button, and adds
people and groups from "Give access to". Apply removes entries (`setfacl
-x`, or `-b -k` when none are left) before `chmod`, and sets the rest after
it (`setfacl -m g::...,u:NAME:...`), because with an ACL chmod's group bits
are the mask; a folder also gets default entries so new items inherit them.
The Group row shows the owning group's own rights (getfacl's `group::`), not
the mask.

Verified with real clicks in the headless GNOME Shell, reading the result
with `getfacl` each time: adding root to a file (`user:root:r--`, the
group's `rw-` kept), giving root Write (`rw-`), removing root (back to a
plain `-rw-rw-r--`); adding root to a folder (access and default entries; a
file made inside afterwards inherited root's entry) and removing it again
(defaults gone too). With hints on, an item with an ACL showed its
`setfacl -m` command. 15 unit tests pass (new: reading getfacl's output).

Not verified yet: the pkexec path for someone else's items, groups added by
name (only root, the VM's one other account, was tried), and "Apply to
everything inside" together with named entries.

### 0.2.4 (2026-10-26)

Found on the user's VM by the Terminal session: GNOME 50's session skipped
the "reopen folders at sign-in" entry ("sets X-GNOME-Autostart-Phase, but
gnome-session no longer manages session services"). systemd's
xdg-autostart generator still makes a unit for such an entry, but marks it
NotShowIn=GNOME, so its ExecCondition fails under `ubuntu:GNOME`. The entry
no longer sets the phase. Checked by running
`/usr/lib/systemd/user-generators/systemd-xdg-autostart-generator` on the old
and new entries: the old unit's condition exits 1 for `ubuntu:GNOME`, the
new unit has none. Not yet seen across a real sign-in.

### 0.2.6 (2026-10-26)

The user picked a more 3D title bar style in Patrone Settings (Studio, Wide
buttons); Terminal's title bar turned raised but File Manager's stayed flat.
Patrone Settings' styles draw GTK's `headerbar` (raised gradient when
active, flat under `:backdrop`), and File Manager's tab strip and dialog
title bars were plain boxes painted flat by its own style sheet. They are
now boxes whose CSS node is `headerbar` (`util::header_box`), with File
Manager's look as the default that the account's gtk.css overrides. Classic
and Royal also restyle labels and buttons inside a header bar (bold, white),
so a small style sheet (`data/over-theme.css`) at priority USER + 1 keeps
tab names in their usual weight and the selected tab's text and close
button in File Manager's colors.

Verified in the headless GNOME Shell with the account's gtk.css set as
Patrone Settings sets it: Studio with Wide buttons in dark mode, two windows
(the active one a gradient from #42424a to #313135, the inactive one flat
#2c2c30, Studio's colors), and Royal with Glossy buttons in light mode
(glossy blue when active, pale blue when not, the selected tab readable;
a Properties dialog's title bar royal blue with a bold white title).

### 0.2.7 (2026-10-26)

- At the user's go-ahead on the Unpack session's offer: Extract All… and a
  new Extract here cover every archive Patrone Unpack opens, through
  `patrone-unpack --extract-to` / `--extract-here` when it's on PATH; ZIP
  and TAR keep File Manager's own extraction (Extract here without Unpack
  makes a folder named after the archive, " (2)" when taken). Unpack is a
  single-instance app whose process can return before the work is done, so
  for Extract All… File Manager makes the destination folder first and
  opens it while the files arrive.
- "Make the searchbox smaller and a fixed size, allowing more space for the
  folder path": 240 pixels, not stretching.

Verified with real clicks in the headless GNOME Shell, with Unpack 0.1.0's
files unpacked to a scratch folder on PATH (not installed): a `.tar.zst`'s
right-click menu offered Extract All… and Extract here, and Extract here
left its one folder, `stuff`, next to it; Extract all from the command bar
on a `.tar.xz` suggested `arc-xz`, extracted `stuff` into it and showed the
folder. Screenshots of the address row at 820 pixels and maximized (the
whole path from This PC, the search box 240 pixels). Not checked: rar and
7z (no tool on the VM to make them), single compressed files, and Extract
here on several archives at once.

### 0.2.8 (2026-10-26)

- "Remove the old filemanager": `apt-get -s remove nautilus` also removes
  `ubuntu-session`, `xdg-desktop-portal-gnome` (GNOME 50's file dialogs for
  Snap and Flatpak apps are Nautilus), `ubuntu-desktop-minimal` and
  `patrone-desktop`, so Nautilus is retired instead: patrone-file-manager
  diverts its menu entry (replaced by a hidden one with no MimeType) and its
  `org.freedesktop.FileManager1` D-Bus service (replaced by one that runs
  `patrone-file-manager --gapplication-service`; the app now waits 10 s for
  the call, and owns the name with ALLOW_REPLACEMENT | REPLACE). Nautilus
  still runs on demand: DING (desktop icons) activates org.gnome.Nautilus
  at sign-in, and Nautilus then holds FileManager1 itself until it idles
  out. DING (`use-nemo` false) sends "Show in Files" and Properties through
  FileManager1 (ShowItems, ShowItemProperties: File Manager answers) and
  copy, move and trash on the desktop through Nautilus's FileOperations2,
  which keeps working since Nautilus stays. The chain that blocks removal:
  xdg-desktop-portal-gnome Depends `nautilus (>= 47~beta)`, ubuntu-session
  Depends xdg-desktop-portal-gnome, ubuntu-desktop-minimal Depends both, and
  patrone-desktop Depends ubuntu-desktop-minimal. The account's
  `org.gnome.shell favorite-apps` still lists org.gnome.Nautilus.desktop;
  that's the user's to change. Verified in the headless shell: with the new service file first in
  XDG_DATA_DIRS and no File Manager running, a FileManager1.ShowItems call
  started it and showed the folder with the file selected.
- "It didn't open flatpak": the user's own mimeapps.list said
  nm-connection-editor opens application/vnd.flatpak, set by File Manager's
  Open with: it pre-selected the first of "More apps" (Advanced Network
  Configuration), so Always made that the default. Removed from
  ~/.config/mimeapps.list on the VM (backup beside it,
  mimeapps.list.before-pfm-fix); Unpack is now the default. Open with
  selects nothing when nothing is suggested, and Always / Just once wait for
  a pick (checked on screen with an unknown file type).
- Home's Recent and Favorites columns were laid out per row from their
  text, so they didn't line up; every column now asks for a fixed width.
  Checked on screen.
- `util::content_type` asked FileInfos for an attribute they lacked,
  logging "GFileInfo created without standard::content-type".

### 0.2.9 (2026-10-26)

At the Patrone Sans session's request: the style sheet no longer names a
font family (it said Noto Sans), so File Manager uses the desktop font that
Patrone Settings → Theme → Font sets (Noto Sans or Patrone Sans), still at
9 points; monospace stays Ubuntu Sans Mono. Checked in the headless shell
with GTK reading a test setting directly (`GDK_DEBUG=no-portals`, keyfile
`font-name='DejaVu Serif 11'`): File Manager drew in DejaVu Serif. (Without
`no-portals`, GTK takes the font from the desktop portal, which reads the
account's real setting.)

### 0.2.10 (2026-10-26)

- "When I compress a file, and click exit or out of it with the mouse it
  should commit. Its still highlighted": the name editor (after Compress,
  New folder, New file and F2) only closed when it lost keyboard focus, and
  a click on a file, on empty space or on the title bar doesn't move focus.
  Now a press anywhere in the window outside the editor, or the window
  losing focus, keeps the name as typed; Enter keeps it and Esc cancels, as
  before (`filecell.rs`, `RenameHooks`: a capture-phase click on the window
  and an `is-active` handler, both removed when the editor closes). The
  click still does what it was for: clicking another file commits and
  selects that file. Checked with real clicks in the headless shell:
  Compress then a click on another file and on the status bar, New folder
  then a click on the title bar, New text document then another window
  opening, F2 then a click on another File Manager window, F2 then Esc, F2
  then Enter.
- A new archive showed as "Empty document" with a blank icon and 0 bytes:
  GtkDirectoryList re-reads a file on created, renamed and
  attribute-changed reports but ignores content changes, and the archive
  was created empty and filled in place. It's now built as a hidden
  `.NAME.part` and hard-linked to its name when finished (a link fails
  rather than replace a file that took the name meanwhile; drives without
  links get a checked rename), so it appears whole. Copies don't have this
  problem: GIO sets their times at the end, an attribute change. Files that
  other programs write in place still show their old size until a refresh.
- A renamed or pasted item that sorts out of view was scrolled to one row
  short: a row that has just arrived has no height yet, so GTK scrolled to
  a zero-height row (or, when the whole folder arrived at once, put the
  item at the top with the ones above it hidden). The scroll now waits for
  the folder to finish loading and then for two frame-clock ticks, when
  the rows are laid out (`FolderView::scroll_when_laid_out`).
- "If a user goes back either in the tree, the back button on their mouse,
  or the back button on the toolbar, make sure the folder that got us
  there is highlighted": `Tab::display` sees both places, and when the one
  left is inside the one shown it selects the child that leads there
  (`child_toward`, through `select_when_present`), so Back, Forward, Up,
  Alt+Left/Up, Backspace, breadcrumbs, the « menu, the folder tree and
  "Previous locations" all do it; the list gets the keyboard focus on that
  item. The mouse's Back and Forward buttons (8 and 9) did nothing before:
  a capture-phase click on the window now runs win.back / win.forward.
  Checked with real input in the headless shell: Up, toolbar Back, mouse
  Back and Forward (evdev BTN_SIDE/BTN_EXTRA through Mutter's
  RemoteDesktop), Alt+Left, Backspace and Alt+Up each selected the folder
  left; the tree's home entry selected pfm-sample; « → Local Disk from
  ~/pfm-sample/Reports selected home with bin…etc still in view;
  /usr/share/icons → Alt+Up selected icons among 299 items; Down then moved
  to the next item. Back to Home, This PC or Gallery selects nothing (they
  aren't folder lists).
- The selected text in the name editor was faint in a selected row (the
  themes draw entry selections dimly); `over-theme.css` gives it the
  accent colour.

### 0.3.0 (2026-10-27): the desktop

The user, 2026-10-27: "The desktop and the roll out menu must be an
extension of the file explorer app, while supporting the desktop and all it
can do", with a desktop menu that has what Windows users look for ("all
of the items that windows would have such as creating a new file, folder,
and other things"), File
Manager's look for the files, and "From the file explorer app, I can't copy
and drag items to copy/move them to the desktop (shift/control-whatever)".

- `patrone-file-manager --desktop` (`src/desktop.rs`) draws the desktop and
  the desktop bubble, hosted by the taskbar (0.4.0, section 7): File
  Manager's own `Window` in a desktop mode (no chrome, the Desktop folder
  only, dialogs with no parent), whose view moves into the bubble's window
  while the bubble is on. Its own process, not on D-Bus; folders open in
  File Manager's windows. `README.md`, "The desktop", has the whole of it.
- `src/desktopgrid.rs` places File Manager's cells itself: down the left
  side first, each where it was put (by name in `desktop-positions`,
  carried over a rename), none under the Recycle Bin, or in the bubble's
  places (`src/bubble.rs`), drawing the bubble behind them. Selection by
  click, Ctrl, Shift, rubber band and arrow keys; moving icons by
  dragging; drops from elsewhere land where they're let go.
- The desktop menu (`menus::desktop_menu`): View, Sort by, Refresh,
  Paste, Paste shortcut, Undo/Redo, New with Templates, Show Recycle Bin,
  Display settings, Personalize, Open in Terminal. Launchers: Allow
  Launching, and an untrusted one asks before it runs.
- Ctrl and Shift while dragging from one app's window to another's: the
  window under the pointer doesn't have the keyboard and never saw them, so
  a drop from File Manager on the desktop always moved. GNOME picks the
  drag's action from the keys and the drop carries it; `dnd::drop_action`
  uses that, in every drop handler.
- Checked with real pointer and key events in a throwaway GNOME Shell
  hosted by the new taskbar; the list is in `shell/gnome/README.md`
  ("Trying it"). Not checked on the user's desktop, nor at a real sign-in
  (where the desktop has no screen for a moment over Remote Desktop: in the
  test shell with no screen, no errors and the windows waiting, but a
  screen arriving later couldn't be tried). Not done: the desktop on
  other screens than the primary one, Align icons to grid off, undo
  shared with File Manager's windows.

0.3.1 (2026-10-27, the user: "I can't drag and drop these anywhere",
after trying taskbar 0.3.12's bubble, which 0.3.0 replaces): icons dragged
about the bubble go where they're let go (`desktop-bubble-order`), and
files dropped on the Recycle Bin riding on the bubble's edge go to the
Recycle Bin (the bubble takes those drops; the bin's own drop target is
under the desktop then). Checked in a copy of the test shell kept in
`~/.cache` (in /tmp GIO can't trash): a bubble icon moved onto another's
place and the rest shifting, one dropped on a folder going into it, one
dropped on the bin on the bubble's edge and one on the bin in its corner
both landing in the Recycle Bin, and a desktop icon dropped on Text
Editor opening there (the first try was lost, as the test shell's first
click into a new window often is).

Next: the user's verdict on 0.3.1 and the desktop, then the gaps in the
README's "Not done yet" (group headers in the icon views, the command bar
folding into "See more" when narrow, Gallery's timeline), and FileManager1.

## 9. Patrone Settings (patrone-settings)

Asked for 2026-10-26: Patrone's own settings, first the taskbar's size
(small, medium or large; medium was the size then), then where the
minimize, maximize and close buttons go, shown in pictures (top right or
top left, "a user option" that themes don't change), and themes: "a
handful of options", each in Patrone's own design and with a name of its
own. `settings/README.md` has the pages, the four themes (Patrone, Modern,
Classic, Studio) and how they reach apps. It's a window of its own, GTK 4 in Rust without
libadwaita (the VM has no libadwaita headers): GNOME Settings takes no
pages from other programs.

Checked in the throwaway shell: see the README. Not checked: on the real
desktop, the theme in apps that aren't GTK (Chromium's own title bar), and
Studio's and Classic's look in GTK 3 apps.

**0.2.0** adds the taskbar's layout and Start icon to the Taskbar page, and
Royal (section 7), and two options for GNOME Settings' Patrone page:
`--set-theme` and `--export-previews` (the pictures, drawn at build time
without a display and saved through a GDK texture, since this cairo has no
PNG writer). The title bar buttons' picture lost its title word, which ran
into the buttons.

**The Patrone page in GNOME Settings** (the user, 2026-10-26: "I am not
seeing the Patrone settings in the settings menu"; they chose a Patrone
build of GNOME Settings over shortcuts or making Patrone Settings the main
Settings). `gnome-settings/` has the page (C, libadwaita, GPL-2.0-or-later
like GNOME Settings), the patches that register it and hide the Telemetry
page, and `build.sh`, which builds Ubuntu's source with them as
`gnome-control-center` `1:50.3-0ubuntu0.2patrone1`. Built on the VM once
the user had installed the build dependencies: the patches applied
cleanly, the page compiled with no warnings, and `dh_missing` found nothing
left out. Checked by running the built Settings, unpacked, in the throwaway
shell: the Patrone row after Ubuntu's Desktop page, every section with its
pictures and the current values picked; clicking Large, Orb, Top Left and
Royal set `size`, `start-icon`, `button-layout` and `theme` (and Royal's
`@import` in `gtk.css`, through `patrone-settings --set-theme`); setting
the layout elsewhere moved the page's pick to Top, while the taskbar moved
to the top and Settings' own buttons to the top left; Privacy & Security
has no Telemetry row. (There the sidebar said "Ubuntu Desktop": the
unpacked package's own desktop file came first. Installed, Ubuntu's file is
diverted by `patrone-branding`.) Not checked until installed: the real
desktop.

**Title bar button styles (patrone-settings 0.3.0, GNOME Settings
`...patrone2`, File Manager 0.2.3).** The user found "the icons in the
buttons are inconsistent". In the throwaway shell, five apps under each
theme showed why: libadwaita apps (Text Editor, Calculator), plain GTK 4
apps that Ubuntu's Yaru styles (Patrone Settings, Patrone Terminal), GTK 3
apps (Disks) and File Manager (its own buttons) each kept parts
of their own look; under Classic and Modern Patrone Settings' buttons
vanished, and under Studio Terminal's were big grey circles and Disks'
disappeared. Asked which look the Patrone theme should have, the user said
to make it configurable, with descriptive names, not "windows" or "gnome".
So the style is its own setting (`org.patrone.desktop title-buttons`):
Wide, Round, Dots, Glossy, or the theme's own (the default). Each style's
sheets set every property of the buttons and their symbols in every state
(Yaru gives the symbols a 30-pixel minimum, which showed as the big
circles), the themes keep only their title bars, File Manager uses GTK's
window controls, and `patrone-settings --apply` runs at every sign-in so
accounts get the sheets (and move off Modern's and Studio's, which went
away). Checked in the throwaway shell: each style in those five apps, the
same in each; File Manager in Wide and Dots, top right and top left; the
page's Style section, a click writing the setting and both `gtk.css`
files, and a change made elsewhere moving the pick.

Also found: in a window of ordinary height the page's pictures were nearly
invisible (a flow box gives a picture that can shrink its minimum height,
none); they now have a height of their own. And the taskbar (0.3.1) shows
"The taskbar was updated", with Sign out, when a package update replaces
its files, since the running desktop keeps the old code until the next
sign-in; checked by renaming a copy of `metadata.json` over it, as dpkg
does.

## 10. Typeface and Settings' Ubuntu wording (patrone-branding 0.1.3, 0.1.4)

Asked for 2026-10-26: a font "closer to what Microsoft has and closer to
what we added to MailCraft", everywhere, the browser included. MailCraft
draws with Windows' Segoe UI on Windows and its bundled Inter elsewhere,
so 0.1.3 made it Inter (Ubuntu's `fonts-inter`). Before installing it, the
user asked for "the same font as the file explorer" as the system default:
File Manager 0.1.0, which they had installed, set its text in Noto Sans. So
0.1.4 makes it Noto Sans (`fonts-noto-core`, already a dependency) and
drops `fonts-inter`, and File Manager 0.2.0 names Noto Sans again (it had
switched to Inter first, falling back to Noto Sans). How it's set
everywhere: `branding/README.md`, "Typeface". The system's size stays 11
pt, GNOME's; the File Manager's 9 pt is Windows' size for its own lists.

Checked for 0.1.4, on the VM: the GNOME defaults compile with Ubuntu's
schemas (`--strict`) and give Noto Sans 11 and Noto Sans Bold 11 for titles
under both desktop names; fontconfig with the rule first and Ubuntu's own
after (as installed) gives Noto Sans for sans-serif, system-ui and Segoe
UI, Bold for Segoe UI Semibold (Ubuntu alone gives Regular), and Noto Sans
Display for Segoe UI Variable Display; Settings in the throwaway shell
with the font set draws in Noto Sans. Ubuntu's Noto Sans has only regular
and bold, so titles are bold rather than semibold. Not checked until
installed: Chromium itself (its snap reads `/etc/fonts/snap-override`, per
the GNOME platform snap's `desktop-launch`), and the login screen.

Findings: `fc-match sans-serif` reads the hyphen as the start of a size
and asks for "sans"; test with `fc-match :family=sans-serif`. Chromium uses
a font substituted for one asked for by name only through a strong
(`binding="same"`) alias. Ubuntu's `56-language-selector-prefer.conf` puts
Noto Sans first for sans-serif, so Patrone's rule is numbered 05.

The user also saw "Ubuntu Desktop" in Settings: GNOME Settings names each
page from its desktop file, so `patrone-branding` puts its own
`gnome-ubuntu-panel.desktop` ("Desktop") in place of Ubuntu's. The rest of
Settings' Ubuntu wording is Canonical's reporting (section 11).

## 11. Nothing to Canonical (patrone-desktop)

Asked for 2026-10-26: drop whoopsie and ubuntu-insights "so it sends nothing
to Canonical". `patrone-desktop` 0.1.0 is Patrone's desktop metapackage:
Ubuntu's desktop and `patrone-branding`, Patrone's apps recommended, and the
programs that report to Canonical removed. `desktop/README.md` lists what
reports to Canonical, what happens to each, and what else still reaches
Canonical's servers.

Then the user chose to move the rest off Canonical too: "point the network
check and the clock at non-Canonical servers, turn off Ubuntu release-upgrade
offers, and skip the Ubuntu Pro page in first-login setup". That's 0.1.1:
NTS servers from Cloudflare, Netnod, PTB and SIDN Labs; GNOME's
`nmcheck.gnome.org`; `Prompt=never` in a `release-upgrades.d` drop-in; and
`ubuntu_pro` in setup's skip list (`desktop/README.md` has how each works).

Checked for 0.1.1 on the VM, without installing it:

- `NetworkManager --print-config` with Ubuntu's `conf.d` and Patrone's file
  gives GNOME's URI, which answers with NetworkManager's default
  "NetworkManager is online".
- A copy of chronyd run with `-Q` (it measures and doesn't set the clock; a
  copy, because AppArmor keeps the real one to its own folders) read
  Patrone's sources and `nocerttimecheck 1`, and all four servers answered
  over NTS, the clock 0.1 ms off. `openssl s_client` completed each
  server's NTS key exchange with a valid certificate.
- Ubuntu's `check-new-release`, with Patrone's drop-in bound into
  `release-upgrades.d` by bwrap and no network: "Prompt is set to never so
  upgrading is not possible", before any download. Without it, it went
  looking.
- First-login setup with 0.1.1's `vendor.conf` bound over
  `/etc/gnome-initial-setup/vendor.conf`: Welcome, Location Services,
  Appearance, Ready to go. No Ubuntu Pro or "Help Improve Ubuntu" page, and
  no password prompt.
- `apt-get -s install` of 0.1.1 over the installed 0.1.0 upgrades it and
  removes nothing.

Not checked until installed: the postinst's chrony steps (debconf, ucf,
removing Ubuntu's pools file) and the restarts, which need root.

Checked after the user installed 0.1.0: whoopsie, ubuntu-insights,
ubuntu-report and apport-gtk are gone (their settings files kept, as
`apt remove` does), apport stays, and systemd loads both Ubuntu Pro
drop-ins. The user's session still lists ubuntu-insights' two timers, whose
units are gone, until the next sign-in.

Checked on the VM for 0.1.0, before it was installed:

- `apt-get -s install` (a simulation) removes exactly apport-gtk,
  ubuntu-insights, ubuntu-report and whoopsie. `ubuntu-desktop-minimal`,
  Settings, `ubuntu-dev-tools` and apport stay. Without apport in
  Recommends, apt would have left apport to `apt autoremove`.
- First-login setup in the throwaway shell, with Patrone's `vendor.conf`
  bound over Ubuntu's by bwrap: Welcome, Ubuntu Pro, Location Services, then
  Appearance. With Ubuntu's, "Help Improve Ubuntu" comes after Location
  Services. (Setup reads `/etc/gnome-initial-setup/vendor.conf` before
  `/usr/share`'s, per its source.)
- The Ubuntu Pro drop-in passes `systemd-analyze verify`, and its condition
  fails on the VM, which isn't attached to Ubuntu Pro.

Not checked until installed: the removal itself, and systemd applying the
drop-ins after `daemon-reload`.

Findings:

- Nothing had been sent from the VM. There's no Insights consent file, so
  its weekly upload had nothing to send, and the crash reports in
  `/var/crash` aren't marked for upload. `/etc/whoopsie` did have
  `report_metrics=true`.
- Ubuntu builds Settings' Telemetry page (Privacy & Security → Telemetry:
  "Send error reports to Canonical", system reports) with whoopsie and
  Insights support compiled in, and shows it whether or not they're
  installed (`debian/patches/ubuntu/telemetry-Repurpose-diagnostics-panel-for-all-telemetry-a.patch`).
  Settings depends on `whoopsie-preferences` and `libinsights0`, so those
  stay. Hiding the page takes a Patrone build of Settings.
- Neither Settings nor first-login setup calls libinsights' upload. They
  record consent and build reports locally, and ubuntu-insights' timer
  uploaded them.
- `motd-news` is already off: there's no `/etc/default/motd-news`, so
  `ENABLED` isn't 1. But since `patrone-branding` moved `50-motd-news`
  aside, `motd-news.timer` starts a script that isn't there, so
  `motd-news.service` will fail twice a day. Fix in `patrone-branding`: a
  drop-in with `ConditionPathExists=/etc/update-motd.d/50-motd-news`.
- First-login setup still says Ubuntu: the Welcome page shows 26.04's
  raccoon, the Appearance page says "Choose how Ubuntu looks", and the last
  page offers "Ubuntu's App Center". A job for `patrone-branding`.
- Setup reads `/etc/gnome-initial-setup/vendor.conf` and ignores
  `/usr/share`'s when it's there, so once `patrone-desktop` is installed,
  test a new skip list by binding it over the `/etc` file.
- `shell/gnome/tests/headless.sh` uses one fixed folder
  (`/tmp/patrone-taskbar-test`) and display, so two sessions running it at
  once replace each other's test shell (it happened on 2026-10-26). Run a
  copy with its own `base` and `display` meanwhile; the script could take
  them from the environment.
- A process started through `bwrap` survives `headless.sh close-apps`,
  which kills only bwrap. Later launches of the same app then just raise
  the old window.
- `firefox` is only recommended by `ubuntu-desktop-minimal` 1.570.4, so the
  ISO now purges it (2026-10-28).
- `/var/crash` holds a root-only `patrone-remote-desktop.0.crash` from 10:03
  that day: apport's name for a package that failed to install or
  configure. Read it with sudo.

Still reaching Canonical, by choice: the update archive and the Snap Store
(Chromium).

## 12. Wallpaper family (patrone-branding 0.1.4)

Asked for 2026-10-26: "a few new versions of the background, one that is a
light mode version, and several that have no logo, several that have small
logos, and maybe a few other interesting versions". Eleven variants of
wallpaper 01, all in Settings → Appearance (`branding/README.md`, "The
family", has previews and how they're made): light; no logo (dark, light,
and Clean without the corner details); a small P (dark and light), a small
badge; Midnight, Blue and Graphite color schemes; and Watermark, a giant
faint P. The light ones are paired with their dark ones, and the default
is now a pair too: new accounts start in Ubuntu's light style, so they see
the light version first.

`tools/make_wallpapers.py` makes them from the original, on Windows: the VM
has no NumPy or OpenCV. Its parts come from `tools/wallpaper_layers.py`,
taken out of `upscale_wallpaper.py`, which still makes the 8K wallpaper
byte for byte (checked).

The user installed 0.1.4 and asked to "remove the ubuntu images". GNOME
Shell depends on `ubuntu-wallpapers` (Ubuntu's patch, for its default
wallpaper), so 0.1.5 provides it, and conflicts with and replaces
`ubuntu-wallpapers` and `ubuntu-wallpapers-resolute`: 63 MB of images, and
both lists in Settings. Only GNOME's default-wallpaper settings point at
those files (Ubuntu's override, which Patrone's beats); first-login setup
doesn't use them. `apt-get -s install` of 0.1.5 removes exactly those two
packages. An account whose wallpaper was one of Ubuntu's gets a plain color.

Checked: the package builds (63 MB, 33 of them the 8K original), every file
the list names is in it, and the list parses; in the throwaway shell,
Settings → Appearance shows all nine entries, the pairs as split tiles,
with the default pair ticked and the style previews in light and dark.
Not checked until installed: switching between them on the real desktop.

Findings:

- JPEG brings the dark gradients' banding back even at quality 97 with
  full-resolution color: it smooths away the grain that dithered them.
- An 8×8 ordered dither stops banding better than random grain (its
  average stays 0.05 levels from the true gradient, against 0.6) and
  makes a 4K PNG 2.9 MB instead of 8.
- Painting the logo out with inpainting leaves its outline, and the
  bottom-left details' box overlaps the logo's, so its left rule counted
  as a detail. The cleared background refills the logo's whole area from
  its surroundings, and the details leave the logo's box alone.

## 13. Patrone Unpack (patrone-unpack)

Asked for 2026-10-26: "a minimal flatpack app that can be used to uncompress
and install things. Make it default, we removed ubuntus". Asked what
"flatpack" meant, the user chose a normal Patrone `.deb` app that also
installs Flatpak apps (Patrone had no Flatpak at all). "Ubuntu's": Ubuntu
26.04 ships no archive app (File Roller was never installed); archives
opened in Nautilus, which File Manager replaced, and `.deb` packages in
Canonical's App Center (`snap-store`). Patrone Unpack (`unpack/`, its
README has the details) does both: an archive opens as a folder with
Extract all (Windows' dialog) and Extract here; a `.deb` or Flatpak file
opens with what it is, what else installing it changes (apt's simulation),
and Install. `patrone-default-apps` 0.1.3 makes it the default for all of
them, with Nautilus and App Center as fallbacks.

Then the user's actual need came through (via the main session): a
downloaded `.flatpak` bundle (PdfCraft 0.9.0, in `~/Downloads`) showed
GNOME's "No Apps Available" with a "Find more in App Center" button. 0.1.1
installs Flatpak apps for the account that opens them (`--user`, no
password), shows a bundle's icon, version, developer, size and permissions,
sets Flathub up for the account (from Flathub's own `.flatpakrepo`, shipped
in the package) when the runtime isn't installed and comes from there, and
depends on `flatpak`.

How: Rust and GTK 4; libarchive for every archive format, rar included
(its own free code, no `unrar`), linked by its run-time name; extraction
written by Unpack itself so nothing lands outside the destination; `.deb`
installs through `pkexec` and a root helper running apt, under a polkit
action of its own (`auth_admin`, every time); Flatpak through `flatpak
install --user`, which needs no password.

Checked on the VM: the package builds and its build runs 19 tests (real zip,
7z and tar archives made with libarchive's writer, paths that try to leave
the destination, passwords, apt's output, Flatpak files); Clippy and
rustfmt are clean; the desktop file validates; `gio mime` picks Unpack for
every type with the new list and the session's paths. In a throwaway GNOME
Shell with real clicks: the archive window, Extract all into the suggested
folder (and the folder opened in File Manager), Extract here naming
"(2)", the replace/skip question, opening a file from the archive,
`--extract-here`, a `.deb` up to polkit's password prompt and Cancel, a
`.deb` apt can't install, a `.flatpakref` without Flatpak, light and dark.
For 0.1.1, the user's PdfCraft bundle **installed end to end** through the
window, with Ubuntu's Flatpak unpacked into a scratch folder and scratch
user and system folders (nothing reached the user's account; all deleted
after): Flathub added, PdfCraft with its runtime and graphics drivers
installed, progress, and Open started PdfCraft. Not checked: a real `.deb`
install (needs the password), the installed policy's message, a Flatpak
install with the installed package's own paths, `.flatpakref` installs, rar
files.

Findings:

- libarchive's mtree reader takes any line of text for an archive: a
  gzipped one-line text file listed as an "mtree" archive holding an empty
  file. Unpack turns on the other formats one by one instead of
  `archive_read_support_format_all`.
- GTK 4.22's Default theme stayed light in a dark session although
  `gtk-interface-color-scheme` was dark (an app style sheet bound to it went
  dark). Unpack sets `gtk-application-prefer-dark-theme` from it. Other
  plain GTK 4 apps on the Default theme may need the same.
- `apt-get --simulate install ./file.deb` works without root and lists
  `Inst`/`Remv` lines; apt 3's solver adds "Reached two conflicting
  assignments" and its reasoning to the error, which Unpack leaves out when
  the unmet dependency is already listed.
- Ubuntu's Flatpak lets `sudo` members install without a password only in
  local sessions (with a seat). Patrone's seatless desktops get Flatpak's
  defaults for `--system`: an administrator's password. Hence `--user`.
- Flatpak bundles hold a `runtime-repo` (Flathub's `.flatpakrepo` for
  PdfCraft), icons as PNGs (`icon-64`, `icon-128`), the metadata keyfile and
  gzipped AppStream XML in the superblock's first dictionary; PdfCraft's has
  no `installed-size`.
- Flatpak's CLI can run unpacked from its `.deb` for tests
  (`LD_LIBRARY_PATH`, `FLATPAK_SYSTEM_DIR`, `FLATPAK_USER_DIR`), but even a
  read-only `flatpak list` fails until the system folder has an OSTree repo
  (`repo/config` and its folders), and `--system` changes go through the
  system helper over D-Bus.
- **Left for later:** GNOME's "Open With" dialog (xdg-desktop-portal-gnome
  50.0) has "Find more in App Center", which starts
  `snap-store_snap-store.desktop`, else GNOME Software. App Center is still
  installed on the VM, so it works there; without it the button fails
  ("Failed to start App Center").
- To reach a throwaway shell's polkit agent, start the app from inside the
  shell (`Gio.Subprocess` through `headless.sh eval`): pkexec asks the agent
  of its own login session, and each SSH command is a session of its own.
- File Manager has its own Extract All for zip and tar; it could hand 7z,
  rar and Extract here to `patrone-unpack --extract-here` /
  `--extract-to DIR` (its session was told).

## 14. PatroneOS's fonts (fonts-patroneos)

Asked for 2026-10-28 and 29: PatroneOS's own font families, drawn from
scratch, as the system's main fonts, under the OFL 1.1, in a public
repository with a release people can download. Done:

- **The fonts**: PatroneOS Sans (the system font), Mono (the console font,
  with box drawing), Document, Serif and Classic, version 1.0.0, in their own
  public repository, `billsecond/patroneos-fonts`: the drawing code, its
  build, the 28 TrueType fonts, WOFF2 web fonts, OFL.txt and FONTLOG.
  Release v1.0.0 has `PatroneOS-Fonts-1.0.0.zip` (the five families, OFL.txt
  and FONTLOG.txt). `./build.sh` there rebuilds them byte for byte, checked
  from a fresh clone; fontbakery's universal checks find no errors or fails.
  Patrone Sans (Noto Sans with a tailed l, from 2026-10-26) is retired.
- **`fonts-patroneos` 1.0.0** installs the release `fonts/release.txt` pins
  in `/usr/share/fonts/truetype/patroneos/`: `packaging/build.sh` fetches it
  and checks its SHA-256 first, so the build runs offline. It recommends
  `fonts-noto-core`, `fonts-liberation` and `fonts-crosextra-caladea`.
  **`fonts-patrone-sans` 2.0.0** is transitional: it depends on
  `fonts-patroneos`, so upgrading installs that and removes Patrone Sans. A
  new release: change `version`, `url` and `sha256` in `fonts/release.txt`
  and bump `fonts-patroneos`.
- **The defaults** (`patrone-branding` 0.1.24): GNOME's interface font
  PatroneOS Sans 11, document font PatroneOS Document 11, monospace
  PatroneOS Mono 11, title bars PatroneOS Sans SemiBold 11, and the login
  screen PatroneOS Sans. fontconfig (`05-patrone-fonts.conf`): sans-serif,
  serif, monospace, system-ui and ui-monospace are PatroneOS Sans, Serif and
  Mono; Calibri is PatroneOS Document; Arial and Helvetica are PatroneOS
  Classic; "Patrone Sans" is PatroneOS Sans. Times New Roman stays Liberation
  Serif and Cambria Caladea (fontconfig's own metric aliases). LibreOffice
  starts new documents in PatroneOS Document
  (`system/libreoffice/patroneos-fonts.xcd`).
- **The apps**: Patrone Terminal 0.1.8 defaults to PatroneOS Mono 11, and a
  saved "JetBrains Mono 11" (the old default, saved with any other change)
  gives way to it. Patrone Settings 0.3.10 and GNOME Settings `...patrone18`:
  the Font card is PatroneOS Sans (with PatroneOS Document for documents) or
  Noto Sans, and a desktop set to Patrone Sans shows PatroneOS Sans picked.
  File Manager 0.3.8 draws its monospace text in PatroneOS Mono; Remote
  Desktop 0.5.7 its sign-in screen in PatroneOS Sans.

Checked on PatroneDev, 2026-10-29 (packages installed by the claude
account): the upgrade from `fonts-patrone-sans` 1.0.1 (Patrone Sans's files
gone, Caladea added); `fc-match` for every name above; in a throwaway shell
(`headless.sh`), the shell's font and GNOME's four font settings, Patrone
Terminal drawing in PatroneOS Mono, and both settings apps' Font cards (each
click checked in the settings); `cargo test` (Settings 19, Terminal 14, three
new); GNOME Settings revision 18 compiled with no warnings in the page;
`lintian` on both font packages (no tags); LibreOffice (its Writer, installed
for the test and removed after): new documents in PatroneOS Document, and
Calibri, Helvetica, Times New Roman and Cambria drawn as above. Not checked:
`patrone-remote-desktop` installed (built only: upgrading it restarts the
remote desktop services); Chromium and Flatpak apps; a fresh install from
an ISO; the user's own session, which shows the new fonts after signing in
again.

Findings:

- On `fc-match`'s command line a hyphen starts a size: ask for `system\-ui`
  or `:family=system-ui`, or it matches "system".
- LibreOffice's default font lists (`VCL/DefaultFonts`) are in an
  extensible group: each value needs `oor:type="xs:string"`, or LibreOffice
  fails to start (a RuntimeException). The first version did that.
- LibreOffice matches Arial itself, before fontconfig, with the
  metric-compatible font it knows for it (from `fonts-liberation`); neither
  its substitution table nor a new entry for PatroneOS Classic changes that.
  Its letters have the same widths as PatroneOS Classic's, so documents lay
  out the same; elsewhere Arial is PatroneOS Classic.
- With GNOME's slight hinting, FreeType ignores a font's TrueType
  instructions (found with Patrone Sans), so the fonts' ttfautohint hints
  matter only under full hinting and on other systems.
- PatroneDev's `gh` sign-in has expired: the fonts repository was pushed,
  and its release made, from Windows.

## 15. Patrone Task Manager (patrone-task-manager)

The user asked (2026-10-26) for "a more windows like task manager,
replacing the current one", which was Ubuntu's Resources. `task-manager/`
is a Rust and GTK 4 app that works the way Windows users expect:
Processes (apps, background processes, system processes, each with its
processes under it), Performance, Usage history, Startup applications,
Users, Details and Services. `task-manager/README.md` has how it works and what was learned.

- **Replacing Resources:** `patrone-task-manager` has `Conflicts` and
  `Replaces: resources`. Resources is only recommended by
  `ubuntu-desktop-minimal` (`apt-cache rdepends --installed resources`), so
  `apt-get install -s` of the package removes Resources and nothing else.
  `patrone-desktop` 0.1.2 recommends `patrone-task-manager`.
- **Ctrl+Shift+Esc:** a custom shortcut in a Patrone dconf database
  (`/etc/dconf/db/patrone.d/`, compiled by `dconf update` in postinst, as
  ibus does) read through a vendor profile,
  `/usr/share/dconf/profile/user`. Not a gschema override: a custom
  shortcut's values live at a path of its own, which an override can't
  reach, and overriding the shared `custom-keybinding` schema would start
  every new shortcut made in Settings as Task Manager's. Nothing else
  Patrone ships sets `custom-keybindings`. An account with custom
  shortcuts of its own keeps its own list, without this one.
- **The taskbar** already lists `org.patrone.TaskManager.desktop` first in
  its "Task Manager" menu item (main session).

Checked, all in a private copy of the throwaway GNOME Shell
(`/tmp/patrone-tm-test`, display `wayland-patrone-tm`, so other sessions'
use of the shared one was untouched), with clicks and keys from the
shell's virtual devices:

- Every page, with screenshots: Processes (sections, totals in the column
  headers, shading, a terminal's shells under the terminal), Performance
  (CPU, memory, two disks, Ethernet; this VM has no graphics card that
  reports how busy it is, so no GPU page), App history (from the journal),
  Startup apps, Users (two accounts), Details, Services.
- End task on a stand-in app (a `sleep` in an `app-gnome-…` scope) ended
  it at once; End task on a plain process asked first, and ended it. The
  right-click menu, search, Properties and Run new task (it started the
  program) worked.
- A Startup apps switch wrote the per-account copy with `Hidden=true` and
  removed it again (in the test shell's own config folder).
- Stop on a throwaway user service (`systemd-run --user`) stopped it, from
  a copy run on the user's session bus (the test shell's own bus has no
  systemd). System services' Start, Stop and Restart weren't tried: they
  ask for the user's password.
- **Ctrl+Shift+Esc** opened Task Manager: the keyfile compiled with
  `dconf compile`, a profile found through `XDG_DATA_DIRS` (as
  `/usr/share/dconf/profile/user` will be, but pointing at a test file
  with `file-db:`), and `gsd-media-keys` running in that shell. The
  installed `system-db:patrone` and postinst's `dconf update` need root,
  so they weren't tried.
- The package builds with `build.sh -d` (9 tests pass) and with Rust 1.93,
  Ubuntu's.
- Dark: checked with the user's real appearance settings (a copy on their
  session bus): consistent after the stylesheet change below.
- Its own use: about 4% of one core on Processes, 3% on Performance, in
  this VM where GTK draws in software.

Not checked: the installed package itself, the shortcut in the user's real
session, a graphics card's page, and per-process network on a busy
browser (UDP and QUIC aren't counted; the README says why). Once, running
on the user's session bus, the app quit while switching to "Your
services" with nothing in its log and no kernel trap logged; it didn't
happen again in ten more tries, with gdb and without.

Findings (more in `task-manager/README.md`):

- GTK lists keep their place by a row and follow it as rows move, so a list
  re-sorted every second wandered off and hid the first section's header.
  Making GTK re-anchor right after rows change crashed it with a divide by
  zero (`libgtk-4.so.1.2200.4`); `table.rs` instead keeps a list that's at
  the top there, safely.
- A `PopoverMenu` made and opened at once is measured without its
  separators and cuts off its last items: make each menu once.
- In the user's session GTK draws its parts dark while
  `@media (prefers-color-scheme: dark)` rules don't apply (theme
  `Yaru-blue-dark`, which plain GTK 4 apps can't load), so a stylesheet
  keyed to that media query showed light panels with light text. Task
  Manager's tints come from `currentColor`. File Manager's stylesheet uses
  that media query; worth checking it in the user's session.
- At 18:31:03 on 2026-10-26 the dev install of Patrone Terminal was
  removed (`~/.local/bin/patrone-terminal`,
  `~/.config/autostart/org.patrone.Terminal.desktop` and the launcher in
  `~/.local/share/applications`, within 40 ms), leaving the package's.
  Not Task Manager's doing; noted in case it wasn't meant.

**0.2.0 (2026-10-26): Sign out and Disconnect on the Users page.** The
user asked to be able to sign accounts out from the Users page, with the
administrator's password asked for at that moment. Sign out of another account is logind's `TerminateUser` with
interactive authorization (polkit's `org.freedesktop.login1.manage`,
`auth_admin_keep`, so an administrator's password is asked for then);
your own account signs out through the desktop's own log-out. Disconnect
ends only the sessions showing an account's desktop on a screen
(`TerminateSession`), leaving the headless desktop running. Both ask first.
`task-manager/README.md` has the details.

Checked in a throwaway shell: the buttons (Disconnect only where a desktop
is shown on a screen), the confirmations for Base and for the user's own
account, and the failure path: with no password agent there, Sign out of
Base said "Base wasn't signed out ... Nothing was changed", and Base's
sessions (701 to 703) were untouched. Not checked: polkit's password dialog
itself (a test shell started over SSH has no working agent: its SSH
session has ended), signing out or disconnecting for real (no real
session was ended), and the own-account log-out. The Sessions column also
changed: "Desktop, on the screen, 1 terminal sign-in"; it used to call
headless desktops remote.

## 16. Ubuntu's name: what the search found, and what's left

The user, 2026-10-27, after a fresh install from the ISO: "a full search on
the name and references across the OS". A subagent searched the installed
system's files (the ISO's layers, mounted read-only: text, binaries'
strings, GNOME's settings as the session reads them). The user's rule for
the fixes: saying PatroneOS is based on Ubuntu is welcome where it
explains what's different ("don't dog ubuntu, just explain the
differences"); `os-release` already says "0.1 (based on Ubuntu 26.04.1
LTS)" and `/etc/legal` now does too.

**Fixed** (`98c51cd`, `6747c7b`, `3ee6d60`, `1591e4c`, `afcd314`;
patrone-branding 0.1.11, patrone-desktop 0.1.6, gnome-initial-setup
patrone2, patrone-remote-desktop 0.4.6):

- **App Center, Security Center and Firefox came back** in every install:
  Ubuntu's layers are preseeded (their snaps set up already), so the ISO's
  changes to the snap seed were ignored, and Chromium, the taskbar's first
  favorite, wasn't installed. The ISO build resets and preseeds again
  (`iso/README.md`), and drops Security Center (Ubuntu Pro's and the snaps'
  settings) too.
- `/etc/lsb-release` said Ubuntu 26.04.1 LTS, which is why the installer
  took an earlier install for Ubuntu (os-prober reads it before
  os-release); now PatroneOS 0.1, codename `resolute` kept for tools.
- "Ubuntu Pro is available" at login (`update-notifier`), Ubuntu Pro's
  desktop service and password prompts, its advert in `apt upgrade` (the
  ESM hook, moved aside), the ESM notes in the text console's login
  message, `/etc/legal`, Ubuntu's web search (`client=ubuntu`) and App
  Center's search in the Start menu, and leftover settings of removed
  packages (ubuntu-report's autostart entry showed in Task Manager).
- **Password prompts at first login**, found by the install that checked
  the above: polkit's journal named who asked. First-login setup asked
  five times "to change your privacy settings": Ubuntu Insights' page,
  skipped but still made, kept calling the crash reporter's settings
  (`whoopsie-preferences`), which without the crash reporter need an
  administrator; Patrone's build no longer calls them
  (`gnome-initial-setup/README.md`). And Patrone Software's background
  service asked "to update information about software": Patrone's desktop
  is the account's off-screen session (Remote Desktop's design, section
  3), which logind makes the account's display session, so polkit treats
  the account's apps as remote. `45-patrone-desktops.rules` now gives them
  Flatpak's and PackageKit's actions as at the screen
  (`remote-desktop/README.md`, "Permissions"). Other actions an app asks
  for still get the remote answer; add them there as they turn up.
- **Two things the name change broke**: automatic security updates
  (unattended-upgrades named the allowed origins by the system's ID, now
  "Patrone"), and Software & Updates' view of the archive (python-apt had
  no template for "patrone").

**Checked** in a fresh install of the last ISO (sha256 `0576b0fd...`) in
Hyper-V, in `PatroneTest2` (the user had Remote Desktop open on
`PatroneTest`), 2026-10-27, with the test account and "Erase disk": the
installer called the earlier install "PatroneOS (0.1)"; the firmware
boot entry is "PatroneOS", in `\EFI\patrone\`; `snap list` has Chromium
and no App Center, Firefox or Security Center; `/etc/lsb-release` and
`/etc/legal` are Patrone's; `update-notifier`, Ubuntu Pro's desktop service
and `whoopsie` aren't installed, and GNOME's Package Updater didn't come
in; no Ubuntu Pro notice at login, no Ubuntu or Pro entries in
`/etc/xdg/autostart`, no ESM apt hook; unattended-upgrades' allowed origins
name Ubuntu's archive; first-login setup ran with no password prompt
(polkit's journal has none) and Patrone Software's Flathub refresh at login
went through; Patrone Terminal opened after setup, not before. Not
checked: a real unattended upgrade (it needs root), the Software & Updates
window, `apt upgrade`'s output (only the hook's absence), and Help.

**Left, bigger** (each needs a package of Patrone's or a patch in a
rebuild; recommendations first):

1. **Done** (2026-10-28, compliance review, `docs/COMPLIANCE.md` items 3
   and 12): `patrone-desktop` 0.2.0 lists Ubuntu's desktop packages itself
   in place of `ubuntu-desktop-minimal` and `ubuntu-minimal`, less Ubuntu
   Pro's client, Software Updater, `hwctl` and Ubuntu's Help
   (`desktop/README.md`). `yelp`, `language-selector-gnome` and
   `ubuntu-settings` stay: GNOME's Help viewer, the language installer and
   Ubuntu's GNOME settings, none of them Ubuntu's logo.
2. **Done** with item 1: Software Updater is gone. Patrone Software has the
   updates.
3. **Done** with item 1: Ubuntu Pro's client and services are gone.
4. **Done**: Help is GNOME Help. `patrone-branding` 0.1.19 replaces
   `ubuntu-docs` with GNOME's own front page and the seven other pages
   Ubuntu moves out of `gnome-user-docs`, in GNOME's 45 languages
   (`branding/tools/make_help.sh`).
5. **Paths into App Center, now gone**: "App Details" on a snap in the
   shell's app menu and in Settings › Apps (they log "Snap store is not
   installed!" and do nothing), "Find More in App Center" in Open With,
   `snapd-desktop-integration`'s "App Updates" notification, and
   `prompting-client`'s "Open in App Center". Recommended: point them at
   Patrone Software in the GNOME Shell and Settings rebuilds; check whether
   Patrone Software lists snaps first.
6. **Settings › Desktop** (Ubuntu's page, renamed): its Dock section, for
   the dock Patrone turns off, is hidden while Patrone's taskbar is
   installed (GNOME Settings `patrone13`, the user's OK, 2026-10-28). Its error
   state (shown only without Patrone's taskbar) said "No Ubuntu settings
   found" and asked for a bug report in Launchpad: Patrone's words since
   GNOME Settings `patrone15` (`gnome-settings/README.md`).
7. **Fonts:** the monospace default is Ubuntu Sans Mono (Ubuntu's setting,
   which `90_patrone` keeps), and `fonts-ubuntu` puts four Ubuntu families
   in every font list. The user's call which monospace font is Patrone's
   (JetBrains Mono is installed already, for the terminal).
8. **Boot names:** **done** (`patrone-branding` 0.1.22): GRUB's menu says
   "PatroneOS", not "PatroneOS GNU/Linux". GRUB's `10_linux` and
   `10_linux_zfs`, which add "GNU/Linux" to any name but Ubuntu's, are moved
   aside and run by Patrone's scripts without it
   (`branding/system/grub/grub-script`). The firmware's boot entry is already Patrone's: the
   installer named it "PatroneOS", in `\EFI\patrone\` (Hyper-V's boot
   order after the install of 2026-10-27). Only shim's fallback list
   (`BOOTX64.CSV`, "This is the boot entry for ubuntu") still says ubuntu;
   the firmware uses it only if that entry is lost.
9. **Done** (`patrone-branding` 0.1.19): Yaru's `start-here` and
   `distributor-logo` icons, and their symbolic versions, are Patrone's P,
   in every color of Yaru's icons and its dark style
   (`branding/tools/make_logo_icons.sh`; Yaru's moved aside). Not an icon
   theme of Patrone's: Settings switches between Yaru's themes by name, so
   one on top would be switched away. `patrone-logo` (os-release's `LOGO`)
   is in `hicolor` too; GTK already found it in `/usr/share/pixmaps`.
10. **The taskbar's favorites** name apps the ISO doesn't install:
    Thunderbird, LibreOffice Writer and Rhythmbox (and Chromium, fixed
    above). The user's call: put them in the ISO or drop them.
11. **The installer** finds the time zone through Canonical's location
    service (`geoip.ubuntu.com`, subiquity's default): the user's call (it's
    what fills in the time zone by itself). "Automated with Landscape" is
    gone (`patrone-installer` 0.1.4, with the Windows logo and Ubuntu's
    unused slides; `installer/README.md`). The last slide, "Almost there", has its text
    beside its one-line title, not in the right-hand column like the
    others (`branding/system/desktop-provision/slides/`).
12. **Links to Ubuntu or nowhere:** first-login setup's "View release
    notes" opens Ubuntu's release notes for version 0.1
    (`https://www.ubuntu.com/getubuntu/releasenotes?os=ubuntu&ver=0.1`,
    `pages/welcome/gis-welcome-page.c`), and os-release's home, support and
    bug links (Settings › About) are this repository, which users couldn't
    open while it was private (it's public since 2026-10-28). The release
    notes link is hidden until Patrone has release notes (first-login setup
    `patrone4`). Left: point os-release at Patrone's website.
13. **First-login setup runs under the taskbar:** its last page's "Open
    Patrone Software" button, and "You can change this later in the
    Settings app" on the others, are hidden behind the taskbar at the
    bottom of the screen. The setup window should keep clear of it (or the
    taskbar stay out of the first login).
14. **Smaller:** Language Support's tooltip "The recommended value for
    Ubuntu is IBus" is **done**: Patrone builds Language Support with its
    own words (`language-selector/`). Translations (`.mo`
    files, en_GB and others) carry their own Ubuntu strings, not searched;
    the kernel says `#30-Ubuntu` in `uname -a`, which is accurate (it's
    Ubuntu's kernel) and stays.

Also still open from the ISO work: Remote Desktop breaks automatic login
(section 3), `patrone-branding` asks about `/etc/issue` when installed with
apt on a running system, and the version: the ISO says 26.04.1 and the
installed system 0.1. The user's call which one Patrone shows.

## 17. Hyper-V and VirtualBox guest support (patrone-guest-tools)

The user, 2026-10-28: "hyper-v and virtualbox support just like a windows
guest os would have". `guest-tools/README.md` has the design and what was
checked; in short:

- **Hyper-V, checked from the host** in a fresh install (`PatroneGuestTools`,
  a new VM; its live session too): shutdown, heartbeat, time sync, KVP
  (the host sees "PatroneOS" and the IP addresses), VSS (a
  `ProductionOnly` checkpoint with someone signed in), the Guest Service
  Interface (`Copy-VMFile`; on Linux the destination is a folder) and
  dynamic memory. The daemons are Ubuntu's (`linux-cloud-tools-generic-hwe-26.04`,
  main), run from any installed kernel's tools.
- **Three things Ubuntu 26.04 gets wrong on Hyper-V**, fixed in the
  package: file copy never starts (Ubuntu's unit waits for a device Linux
  6.10 dropped, and the disc's 7.0.0-30 kernel doesn't even give the
  channel to `uio_hv_generic`); the daemons fail in the live session (they
  need the running kernel's own tools); and production checkpoints fail
  whenever someone is signed in, because `hv_vss_daemon` tries to freeze
  the FUSE clipboard mounts of FreeRDP and GNOME Remote Desktop, both part
  of Patrone Remote Desktop, and gives up.
- **The host's clock** is one of chrony's sources (`PHC0`). The user's
  Windows host had Windows Time stopped and its clock 6 s behind; after a
  checkpoint the guest followed it for a while. Worth starting Windows Time
  on the host (`Start-Service W32Time`; the user's call).
- **Enhanced Session** is built (2026-10-28), in `patrone-remote-desktop`
  0.5.0 rather than here: a second proxy on vsock 3389 shows Patrone's login
  screen and then the account's own desktop (the user chose a login screen
  in the window over NLA). See `remote-desktop/README.md`, "Hyper-V's
  enhanced session".
- **VirtualBox is packaged, not tried** (the user, 2026-10-28: don't install
  VirtualBox anywhere for this). Ubuntu's guest utilities (7.2.6,
  **multiverse**: Debian's `contrib`, only because the host's BIOS needs a
  non-free compiler; the guest tools are GPL-3.0), VBoxClient started in
  Wayland sessions (Ubuntu's packages only start it from `Xsession.d`), and
  every account in `vboxsf`. Expect the clipboard not to work from a
  Windows host until Ubuntu has 7.2.8, and no drag and drop on Wayland.
- **For the ISO build:** apt 3.2 upgrades by source package, so the HWE
  cloud tools brought the archive's kernel (7.0.0-42) into the image next
  to the disc's; the ISO grew by about 560 MB. And apt left recommended
  packages that need an installed one upgraded out (`vim` isn't in the
  ISO). `iso/build-iso.sh`'s chroot mounts share propagation with the dev
  VM: two builds at once took the dev VM's own `/dev/pts` away
  (2026-10-28, remounted by hand); run builds one at a time, or in
  `unshare --mount --propagation private`, until the script isolates its
  mounts.
- Also seen: the installer warns "The computer is not plugged in to a
  power source" in Hyper-V, which shows the guest a battery.

### Patrone Virtual Machine Setup: the prompt at login

0.2.0 (2026-10-29) renamed it from "Hyper-V Setup" (the window said
"Hyper-V Support"): Microsoft's trademark guidelines keep Hyper-V out of
product names (`docs/COMPLIANCE.md`, item 15). The package, program, units
and app ID keep "hyperv", which a rename would have to migrate.

The user, 2026-10-28: "When the PatroneOS starts up, have it check for
Hyper-V and if its using Hyper-V as its host, then provide a prompt with an
option to install support for Hyper-V, also allow a disable future
notifications on this dialog box." `hyperv-setup/`, package
`patrone-hyperv-setup` 0.1.0, recommended by `patrone-desktop` 0.2.2. At
each desktop login in a Hyper-V VM (not the live session), if something's
missing, a dialog offers to install it with one password: the guest tools
and the enhanced session's service (installed, updated from before 0.5.0,
or turned on). For Windows' side it shows the PowerShell command, with the
VM's name and a Copy button: `Set-VM ... -EnhancedSessionTransportType
HvSocket` until Hyper-V has opened an enhanced session with the VM (a udev
rule sees the Hyper-V socket's channel arrive), and
`Enable-VMIntegrationService` when an integration service is off for it.
Nothing shows when everything's set up; "Don't show this again" is per
account. `hyperv-setup/README.md` has the design and what was checked.

- **0.1.0 showed over the installer** in the live session of the ISO from
  main 439e7a9 (the compliance session, 2026-10-28): it skipped the live
  session by `boot=casper`, which Patrone's boot menu doesn't pass (casper's
  initramfs boots without it). 0.1.1 also skips it when the disc is
  mounted at `/cdrom`, where casper puts it. Checked on PatroneDev with a
  disc faked at `/cdrom` in a private mount namespace; a live boot of the
  next ISO is still to check.
- **Checked on PatroneDev**, with the missing parts pretended
  (`--pretend-missing`), in a hidden GNOME Shell: every page, light and
  dark; Set Up as a dry run; GNOME Shell's password dialog for the real
  one, cancelled; Copy; "Don't show this again" both ways; the wait at
  login for Hyper-V's first enhanced session. The root helper ran for real,
  and the login unit ran in the user's session and showed nothing, as
  everything's set up there.
- **Not yet checked: a fresh install**, where the prompt really shows. VMs
  from `scripts/hyperv/new-test-vm.ps1` have the socket transport and Guest
  services on, so with the next ISO nothing is missing and nothing shows.
  To see it, make the VM with Hyper-V Manager's defaults (transport VMBus,
  Guest services off), or, with the VM off,
  `Set-VM -VMName NAME -EnhancedSessionTransportType VMBus`. Expect the
  dialog after first-login setup (or 30 seconds after login, while it waits
  for Hyper-V's first enhanced session), with the Set-VM and
  Enable-VMIntegrationService lines. Installs from before `patrone-desktop`
  0.2.1 have no Patrone repository, so Set Up can't install from it there.
- **Found, not fixed** (section 18, "Found on the way"): the daily
  automatic updates don't run in Hyper-V VMs, which have a battery. A fix
  belongs in `patrone-guest-tools`.

## 18. Updates: Patrone's apt repository, Ubuntu's updates, the next LTS

The user, 2026-10-28, on the recommendation of a Patrone update channel on
GitHub, a check that rebuilds Patrone's changed pieces when Ubuntu updates
them, and no offer of Ubuntu's next release: "Thanks, do it."

**Patrone's apt repository:**
[billsecond/patrone-apt](https://github.com/billsecond/patrone-apt), on
GitHub Pages (`https://billsecond.github.io/patrone-apt`), one suite,
`resolute`, signed with its own Ed25519 key (fingerprint `8FE6 1B83 753C
2636 CD74 0259 4FC3 52E3 D384 225F`). The private key is only on the dev
VM, in `~/.local/share/patrone-apt/gnupg`, backed up beside it as
`secret-key-backup.asc`: like the app index's, but never on GitHub. The
index is in git; the debs are files of a release (`pool`), since
`patrone-branding` alone is about 90 MB, near git's limit for one file. A
workflow puts both on Pages, checking every deb against the signed index.
`packaging/publish.sh` publishes (`packaging/README.md`, "Publishing").
`patrone-desktop` 0.2.1 installs the source, key and pins, so installs get
Patrone's updates through Patrone Software and the daily automatic updates
(`desktop/README.md`, "Updates"). Installs from before 0.2.1 don't have the
source yet: they need 0.2.1 installed once by hand, or a new install.

**Patrone's builds win over Ubuntu's:** `patrone.pref` gives Patrone's
repository priority 600, over Ubuntu's 500, and Ubuntu's versions of the
nine packages Patrone builds from Ubuntu's 90, under what's installed. So
Ubuntu's next update to GDM, GNOME Settings or first-login setup doesn't
replace Patrone's build, changes and all; Patrone's rebuild of it does. The
trade: Ubuntu's fixes to those three arrive through Patrone, a day or so
after Ubuntu's.

**Keeping up with Ubuntu:** `packaging/ubuntu-updates.sh` runs daily on the
dev VM (a user timer, `scripts/vm/patrone-ubuntu-updates.timer`): when
Ubuntu has published a newer GDM, GNOME Settings or first-login setup for
resolute, it builds it again with Patrone's changes from a clean checkout
of `main` and publishes it; a build that fails becomes an issue in
billsecond/patrone-apt. The installer front end isn't rebuilt there (it has
to match the snap on the ISO), but the script opens an issue when the snap's
`26.04/stable` channel moves past revision 665. A script on the VM rather
than GitHub Actions because the key stays off GitHub and the VM has the
build dependencies; a daily GitHub Actions check in billsecond/patrone-apt
opens an issue for each package Ubuntu has updated, and closes it when the
rebuild is published, so one that's behind shows even while the VM is off.

**At first, merges weren't published by themselves.** Nothing built `main`
and published it: the daily job covered only Ubuntu's updates to those
three packages, so each merge that bumped a package's version had to be
built from `main` on the VM and published by hand. By the evening of
2026-10-28 the repository had fallen behind `main` this way,
`patrone-desktop` 0.2.1 against main's 0.2.3 and five packages more, and a
fresh install found it.
They're published since, built from a clean checkout of `main` at
`d0c19ce`: Desktop 0.2.3, Default Apps 0.2.1, File Manager 0.3.3, Hyper-V
Setup 0.1.1 (new), Remote Desktop 0.5.1 and Branding 0.1.21. PatroneDev's
apt read them from the live repository as the candidates and downloaded
them with their sums checked.

**Since 2026-10-29, merges publish themselves**, the user's choice ("Yes,
daily"). `packaging/main-updates.sh`, run first by the daily job, builds
each package whose version on `main` is newer than the published one, and a
new `revision=` of Patrone's GDM, Settings or first-login changes (built
from the Ubuntu version already published), and publishes only what it has
just built, only from a checkout of `main` as it is on GitHub. A failure is
in the journal, fails the run, and becomes an issue in billsecond/patrone-apt
that closes once that version is published. `patrone-installer` stays by
hand (an issue says so). Checked on PatroneDev (2026-10-29), against a
local copy of patrone-apt with a stand-in for `gh`, and a copy of
this repository whose `main` was this change: three packages made to look
behind (Default Apps, first-login setup by revision, the installer) were
built and published as one signed commit, and the installer got its issue,
in 55 seconds. A version bump with a missing build dependency published
nothing, opened an issue, failed the run, and Ubuntu's part still ran. Its
fixed version was published and the issue closed by itself. A changed
checkout was refused. A Rust package (Unpack) built from a shell without
cargo on its `PATH`. Then a full run against the live repository and
GitHub's issues: everything on `main` was published already, so it
published nothing and changed nothing. Not checked: a run started by the
timer itself with this change, which happens at the first daily run after
the merge.

**Checked on PatroneDev** (2026-10-28), first with a local copy of the
repository served on the VM and a stand-in for `gh`: `publish.sh --all`
published main's 25 packages (the debs uploaded, the index written, signed
and committed), and `assemble-site.sh` put the site together (119 MB) with
every deb's sum checked. Root's apt, with the source, key and pins: `apt
update` took the signed index, `apt-cache policy` showed Patrone's at 600 and
Ubuntu's GDM, Settings and first-login setup at 90, and `apt-get install
--only-upgrade` upgraded `patrone-terminal` 0.1.4 to 0.1.5 and first-login
setup from Ubuntu's build to Patrone's, from the repository.
`unattended-upgrade --dry-run` would install Patrone's packages too. In a
separate apt root, a stand-in for a newer Ubuntu GDM (`50.1-0ubuntu0.9`)
didn't replace Patrone's build with the pins, did without them, and didn't
with the pins even with Patrone's repository unreachable. `ubuntu-updates.sh`
found first-login setup behind (the test index said it was built from
`50.0-0ubuntu7`), built Ubuntu's `50.0-0ubuntu7.1` with Patrone's changes in
22 seconds and published it.

Then for real, once billsecond/patrone-apt was public: `publish.sh`
published main's 25 packages (Branding 0.1.20 and Remote Desktop 0.5.0
built from a clean checkout of `main`), and Pages served the signed index,
the debs and the key. On PatroneDev, with `patrone-desktop` 0.2.1 installed:
`apt update` read `https://billsecond.github.io/patrone-apt`, Patrone's
builds were the candidates at 600, and `gnome-control-center-faces` was
upgraded from it (GDM `patrone2`, the taskbar and Default Apps were left
for the user). The daily GitHub check found all three of Patrone's builds
current with Ubuntu's.

**Ubuntu's release upgrades are off**, as before: `patrone-desktop`'s
`release-upgrades.d/patrone.cfg` says `Prompt=never`, and Ubuntu's upgrader
reads that folder after `/etc/update-manager/release-upgrades`, so it wins.
Checked on PatroneDev (2026-10-28): `do-release-upgrade -c` answers "Prompt
is set to never so upgrading is not possible", and the login message's
check (`check-new-release`) finds nothing.

**Found on the way:** the daily automatic updates never run on Hyper-V VMs.
Hyper-V shows the guest a battery (section 17), and unattended-upgrades
stops when "System is on battery power" (`Unattended-Upgrade::OnlyOnACPower`,
on by default; seen in `unattended-upgrade --dry-run --debug` on
PatroneDev). Ubuntu has the same. Worth a fix in `patrone-guest-tools`:
turn `OnlyOnACPower` off under Hyper-V (`ConditionVirtualization`).

### Moving to the next LTS (28.04, 2028)

Ubuntu 28.04 LTS comes out in April 2028; Ubuntu offers upgrades from 26.04
once 28.04.1 is out, around August 2028. 26.04's standard support runs to
2031, so there's no hurry. The plan:

1. **Nothing offers Ubuntu's release** in the meantime (`Prompt=never`), so
   no one lands on plain Ubuntu 28.04 without Patrone's packages.
2. **A new suite** in Patrone's apt repository, named after 28.04's
   codename, beside `resolute`. Everything is built again on a 28.04 dev VM:
   the Rust packages with 28.04's `rustc` (raise `rust-version`), GDM, GNOME
   Settings and first-login setup from 28.04's sources with the patches
   brought up to date, the taskbar for 28.04's GNOME Shell (`shell-version`
   in its metadata, and its API), the installer at 28.04's snap commit.
   `patrone-branding`'s diversions and the python-apt templates are checked
   against 28.04's `base-files`, `distro-info` and python-apt;
   `patrone-desktop`'s package list against 28.04's `ubuntu-desktop-minimal`
   and `ubuntu-minimal`; the list of packages Patrone pins before Ubuntu's
   against what Patrone builds then.
3. **A 28.04 ISO**, tried with fresh installs in throwaway VMs, as now.
4. **The move for existing installs**, recommended: Ubuntu's own release
   upgrader, driven by Patrone. `patrone-desktop` for 26.04 gets an update
   that points `/etc/update-manager/meta-release` at a `meta-release` file
   of Patrone's (on the repository's GitHub Pages site) that lists 28.04
   only when Patrone is ready, sets `Prompt=lts`, and adds Patrone's
   settings for the upgrader in `release-upgrades.d` (the upgrader reads
   that folder too: which metapackage is the desktop, `patrone-desktop` in
   place of `ubuntu-desktop`). The 28.04 `patrone-desktop` points Patrone's
   source at the new suite. Ubuntu's upgrader knows how to move between
   Ubuntu releases (held packages, removed ones, the reboot); a tool of
   Patrone's that changes the suites and runs `apt full-upgrade` is the
   fallback if it can't be made to accept Patrone. Either way, it's tried on
   a copy of a 26.04 install first.
5. **The `resolute` suite stays** for installs that haven't moved, with
   fixes and the rebuilds of Ubuntu's updates, for as long as Patrone
   supports 26.04.

## Suggested order

Next (the user, 2026-10-27): "tomorrow we will talk about next big steps".
Section 16 lists what's left of Ubuntu's name, with a recommendation for
each; the Patrone desktop metapackage (its item 1) is the biggest.

The user's quick steps first (bash login shell, RDP checks, trying the
compositor, the desktop design). Then the compositor's next features and
milestone 3, the Chromium defaults package, then Patrone's own RDP path.
