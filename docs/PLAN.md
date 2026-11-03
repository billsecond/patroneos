# PatroneOS: project plan

PatroneOS is an Ubuntu-based operating system with its own, brand-new desktop
environment, **Patrone Desktop**, built as an installable ISO.

## What comes from Ubuntu, and what is ours

Ubuntu 26.04.1 LTS ("Resolute Raccoon") supplies everything below the desktop:
the kernel and drivers, systemd, the package archive and security updates,
NetworkManager, PipeWire audio, Bluetooth, power management, printing, and the
apps people install. We don't fork any of that.

Patrone Desktop replaces GNOME. It is a new Wayland desktop that talks to the
same standard Linux services GNOME uses, so ordinary GTK, Qt and Electron apps
run on it unchanged, and X11 apps run through Xwayland.

## Patrone Desktop components

| Component | Job | Notes |
| --- | --- | --- |
| Compositor | Draws every window, handles input, multiple monitors, window management | The core of the desktop, and the biggest piece of work |
| Shell | Panel, app launcher, dock, notifications, quick settings | Separate programs that draw on the screen through the layer-shell protocol |
| Lock screen | Locks and unlocks the session | Uses the ext-session-lock protocol |
| Settings app | Displays, input, network, sound, appearance | Talks to NetworkManager, PipeWire and UPower over D-Bus |
| Session | Starts everything at login | A session entry the login screen can pick, plus systemd user units |
| Portals | Screenshots, screen sharing, file pickers for sandboxed apps | Start from the existing GTK portal; add our own backend where needed |
| Terminal | Patrone Terminal, the default terminal | GTK4 + VTE; Kali-style look, copy on select, paste on right-click, zsh with a two-line prompt (see `docs/NEXT.md`) |
| File manager | Patrone File Manager ("File Manager"), the default file manager | GTK4 in Rust, working the way Windows users expect, in Patrone's own look: tabs, command bar, right-click menus, Folder Options (see `file-manager/README.md`) |
| Remote desktop | RDP access, including Hyper-V enhanced sessions | Clipboard sharing and desktop resizing when Patrone runs as a Hyper-V guest (see `docs/NEXT.md`) |

At first the login screen is GDM, Ubuntu's own, which can start any Wayland
session. A Patrone login screen can come later.

## Default apps

| Job | App | Notes |
| --- | --- | --- |
| Terminal | Patrone Terminal | Ours (`terminal/`) |
| File manager | Patrone File Manager, replacing Ubuntu's Files (decided 2026-10-26) | Ours (`file-manager/`); `patrone-default-apps` makes it the app for folders (`inode/directory`) |
| Archives and installers | Patrone Unpack (decided 2026-10-26), in place of Ubuntu's Files for archives and Canonical's App Center for `.deb` packages | Ours (`unpack/`); `patrone-default-apps` makes it the app for zip, tar, 7z, rar and compressed files, `.deb` packages and Flatpak files. Installs `.deb` packages with apt (pkexec, an administrator's password) and Flatpak apps for the account that opens them (no password), setting up Flathub when a runtime comes from there; `patrone-unpack` depends on `flatpak` |
| App store | GNOME Software (decided 2026-10-27), in place of Canonical's App Center (the `snap-store` snap) | Ubuntu's (universe), with its deb and Flatpak plugins and not its snap one. `patrone-software` removes App Center, and its Explore page shows Patrone's own picks from a file it installs, in place of GNOME's (`software/`) |
| Task manager | Patrone Task Manager, replacing Ubuntu's Resources (decided 2026-10-26) | Ours (`task-manager/`), working the way Windows users expect, in Patrone's own words: Processes, Performance, Usage history, Startup applications, Users, Details, Services. Ctrl+Shift+Esc opens it; `patrone-task-manager` conflicts with `resources`, and the taskbar's "Task Manager" opens it first |
| Text editor | Notepad (decided 2026-11-03), in place of GNOME's Text Editor for plain text, which stays as the fallback | Ours (`notepad/`): tabs, File, Edit and View menus, find and replace, go to line, zoom, a font of your choice (PatroneOS Mono by default), a light or dark look and a status bar; it keeps open tabs and their unsaved changes between sessions, which Settings turns off. `patrone-default-apps` makes it the app for plain text; the command is `notepad` |
| Shell | bash with Patrone's two-line prompt (decided 2026-10-26); zsh set up the same way for those who choose it | `patrone-shell-config` |
| Web browser | None preinstalled; each account picks one the first time it's needed (decided 2026-10-28, in place of Chromium, which had replaced Firefox on 2026-10-25) | Ours, `browser-chooser/` (Patrone Browser Chooser): the taskbar's Web browser button and any link opened with no browser installed ask which to install (Chromium, Firefox, Brave, Google Chrome, Microsoft Edge, Vivaldi, Opera, LibreWolf, Zen Browser, GNOME Web). Patrone Software installs it from Flathub and keeps it up to date; it becomes the default and opens. Not forked or rebranded: browsers ship security fixes about weekly, and a fork would make us responsible for every one of them. |

## Technology (decided 2026-10-25)

- **Compositor: Rust with [Smithay](https://github.com/Smithay/smithay).**
  Rust keeps the most privileged process on the desktop memory-safe, and Smithay
  is proven: System76's COSMIC desktop and the niri compositor are both built on
  it. We chose it over C with wlroots, the library behind Sway and labwc.
- **Shell and settings UI: GTK4 through gtk4-rs, with gtk4-layer-shell.** This
  gives us accessibility, input methods, high-quality text and theming without
  writing them ourselves. Patrone's look comes from our own widgets and styling.
- **License: GPL-3.0-or-later** for all of Patrone's own code. GPL code (such
  as niri's) can be borrowed, and MIT or Apache code (such as Smithay's) can
  be included as long as its notice is kept.

## Development loop

The compositor can run as a window inside the dev VM's existing GNOME desktop,
so we can rebuild and relaunch it in seconds without logging out. Once it can
run a full session, we install a session entry and choose "Patrone" at the login
screen.

**Hyper-V limitation:** Hyper-V gives Linux guests no GPU acceleration, so
rendering in the VM is done in software by Mesa. That's fine for building the
desktop, but animation smoothness and GPU-specific behavior need checking on
real hardware before a release.

## Build pipeline

- **Packages:** each component becomes a `.deb` (`patrone-compositor`,
  `patrone-shell`, `patrone-settings`, `patrone-session`, and the
  `patrone-desktop` metapackage), built in a clean chroot with `sbuild`.
  `patrone-terminal`, `patrone-shell-config`, `patrone-branding` and others
  exist already (see `packaging/README.md`). So does `patrone-desktop`: today
  it's Ubuntu's GNOME desktop with Patrone's packages and without anything
  that reports to Canonical (see `desktop/README.md`).
- **Apt repository:** [billsecond/patrone-apt](https://github.com/billsecond/patrone-apt),
  on GitHub Pages, signed with Patrone's own key (kept off GitHub), and
  published with `packaging/publish.sh` (`packaging/README.md`, "Publishing";
  `docs/NEXT.md`, section 18). Patrone runs no servers. It replaces the
  earlier plan of a private `reprepro` repository.
- **ISO:** remaster the official 26.04.1 desktop ISO. Add our packages, make
  Patrone Desktop the default session, apply branding, and repack with
  `xorriso`. Ubuntu's installer stays in place. A from-scratch image build waits
  until we outgrow remastering.
- **Test:** boot each ISO with QEMU/KVM inside the dev VM (nested virtualization
  is on), and do full installs in disposable Hyper-V VMs.

## Environments

| Machine | Purpose |
| --- | --- |
| `PatroneDev` (Hyper-V, Ubuntu 26.04.1 Desktop, 12 vCPU, 32 GB, 250 GB) | Day-to-day development. The repo lives here, everything is built here, and Claude Code runs here over SSH from the Claude app on Windows. |
| Throwaway test VMs | Install and boot each built ISO from scratch. |
| Real hardware (later) | Check GPU rendering and performance. |

## Repo layout

```
docs/                  plan, decisions, how-to notes
branding/              logo, colors and artwork, and the system files that
                       replace Ubuntu's branding (patrone-branding)
default-apps/          default browser and favorites (patrone-default-apps)
desktop/               the desktop metapackage's defaults: nothing reports
                       to Canonical (patrone-desktop)
remote-desktop/        RDP sign-in with each account's own password
                       (patrone-remote-desktop)
compositor/            the Patrone compositor
shell/                 panel, launcher, dock, notifications
  gnome/               the taskbar and Start menu for today's GNOME-based
                       desktop, as a GNOME Shell extension
                       (patrone-gnome-taskbar)
settings/              Patrone Settings: taskbar size, layout and Start
                       icon, title bar buttons, themes (patrone-settings)
gnome-settings/        Patrone's build of GNOME Settings: a Patrone page,
                       no Telemetry page (gnome-control-center)
software/              Patrone Software's picks and remotes (patrone-software)
gnome-software/        Patrone's build of GNOME Software: a Featured tab
                       (gnome-software)
terminal/              Patrone Terminal
file-manager/          Patrone File Manager (patrone-file-manager)
shell-config/          bash prompt and zsh setup (patrone-shell-config)
session/               session entry, systemd user units, defaults
packaging/             Debian packaging for each component
iso/                   ISO remaster scripts
scripts/               dev environment setup and helpers
```

## Milestones

1. Dev VM installed, reachable by SSH, toolchain and Claude Code in place, repo
   cloned. (Done.)
2. A minimal compositor running as a window inside GNOME: a background, a
   terminal that opens on launch, and windows you can move, resize and close.
   (Done 2026-10-25; input handling still to be tried by hand.)
3. The compositor runs as a real session chosen at the login screen.
4. A panel and app launcher, and a notification daemon.
5. Lock screen and a first settings app (displays, input, network, sound).
6. First Patrone ISO with Patrone Desktop as the default session.
