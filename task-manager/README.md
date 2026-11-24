# Patrone Task Manager

Patrone's task manager, packaged as `patrone-task-manager`: a Rust and GTK 4
app that shows what's running and how hard the computer is working, and
works the way people who have used Windows expect. On PatroneOS it takes
the place of Ubuntu's Resources app (`net.nokyan.Resources`), which
`patrone-desktop` conflicts with, and Ctrl+Shift+Esc opens it. Installed
on its own, on Ubuntu, it changes neither: Resources stays, and the
shortcut is one to add in Settings (below).

```
patrone-task-manager [--page processes|performance|history|startup|users|details|services]
```

It's a single-instance app (`org.patrone.TaskManager`): opening it again,
or pressing Ctrl+Shift+Esc, brings the open window forward and shows the
page asked for. The launcher's actions open Performance, Startup
applications and Services. The taskbar's right-click "Task Manager" opens
it first (`TASK_MANAGERS` in the taskbar's `taskbar.js`).

## Pages

| Page | What it shows | Where it comes from |
| --- | --- | --- |
| Processes | Apps, then Background processes, then System processes, each a row with its processes under it; Status (Suspended, Ended), CPU, Memory, Disk and Network, sortable, cells shaded pale yellow to orange as they get busier. The column headers carry the whole computer's figures ("12%" above "CPU") | `/proc` (below) |
| Performance | CPU (overall, or each processor), Memory (with its composition: in use, modified, cached, free), each disk, each network adapter, and graphics cards that say how busy they are; each with a minute's graph and figures like Windows' | `/proc/stat`, `/proc/meminfo`, `/proc/diskstats`, `/proc/net/dev`, `/sys` (cpufreq, caches, block devices, adapters, `gpu_busy_percent`), `getifaddrs` |
| Usage history | Processor time, peak memory and times opened for each app since the computer started | systemd's "Consumed … CPU time" records for each app's scope in your journal (`journalctl --user --boot`), plus the scopes still running (`cpu.stat`, `memory.peak`) |
| Startup applications | Each autostart entry with a switch, the package it came from, whether it's everyone's or yours, and its command | `/etc/xdg/autostart`, `~/.config/autostart`, `dpkg-query -S` |
| Users | Each signed-in account, its state and sessions (its desktop, whether it's shown on the screen, terminal sign-ins), what its processes use, and its apps and processes under it. Sign out and Disconnect (below) | logind's `ListSessions` over D-Bus |
| Details | Every process, kernel threads too: name, PID, status, account, CPU, private memory, threads, command line | `/proc` |
| Services | System services or your own (a list to choose from): name, main PID, description, status, and how each starts (Automatically, Manually, When needed, Never) | systemd's `ListUnitsByPatterns` and `ListUnitFilesByPatterns` over D-Bus |

Commands: Run a program (a program or command, or a folder, file or web
address, optionally as an administrator through pkexec), End, End process
and its children, Set priority (Details), Show in File Manager (selects the
program's file through `org.freedesktop.FileManager1`), Show details,
Properties, and Start, Stop and Restart for services. The search box in the
title bar filters every list by name, command line or PID. The menu sets the
update speed (twice a second, once a second, every 4 seconds, paused). The
window keeps its size, page and update speed in
`~/.config/patrone-task-manager/state.ini`.

The pages' names are plain words of Patrone's own. Usage history and
Startup applications (GNOME's own term) were renamed on 2026-10-29, after
the compliance review (`docs/COMPLIANCE.md`, item 15) found their earlier
names copied from Windows 11; `--page history` and `--page startup` still
open them.

## How processes are grouped

The desktop already groups them. Every app opened from the desktop runs in
a systemd scope of its own, named after its desktop entry
(`app-gnome-org.patrone.Terminal-1234.scope`; snaps run in
`snap.chromium.chromium-….scope`), and every service in a unit of its own.
Task Manager reads each process's control group (`/proc/PID/cgroup`) and
groups by the last unit in it:

- An **app** is a scope whose desktop entry is shown in the app menus (not
  `NoDisplay`). Its name and icon come from the entry.
- **Background processes** are the account's other units: its services,
  sign-in sessions, autostart helpers.
- **System processes** are everything else: system services and other
  accounts' units.

Terminals start each shell in a scope of its own beside theirs
(`vte-spawn-UUID.scope`), so those join the group of the terminal that
started them, as Windows shows a terminal's shells under it. Kernel threads
are left off the Processes page and listed in Details.

A process's control group is read again for its first ten seconds, since the
desktop starts an app and then moves it into its scope.

## What the figures are

- **CPU** is a share of the whole computer, like Windows: 100% is every
  processor busy.
- **Memory** is private memory: resident memory less what's shared with
  other processes (`/proc/PID/statm`), close to Windows' "active private
  working set". Properties also shows resident and virtual memory.
- **Disk** is bytes read from and written to disk per second
  (`/proc/PID/io`). Linux shows that only for your own processes, so other
  accounts' are blank.
- **Network** per process is the TCP traffic of your own processes. Linux
  doesn't count network use per process: Task Manager asks the kernel's
  socket diagnostics for every TCP connection's bytes sent and received
  (what `ss -ti` reads; no administrator rights needed) and matches the
  connections to processes through their open files (`/proc/PID/fd`).
  **UDP traffic isn't counted**, which includes QUIC, the protocol browsers
  use for some sites, so a browser can show less than it uses. The
  Performance page's network figures are each adapter's own, and complete.
- A process's owner is read from the `Uid:` line of `/proc/PID/status`:
  `/proc/PID` itself belongs to root for processes that make themselves
  non-dumpable, as terminals do.

## Ending tasks

End on an app ends it at once, as people expect: each process gets
SIGTERM, and SIGKILL three seconds later if it's still running (checked by
PID and start time, so a new process that got the same PID is left alone).
Anything else (a background or system process, or any process tree) asks
first, since ending it can stop the desktop or the computer from working.
Processes of other accounts are asked through `pkexec kill -TERM`, which
asks for an administrator's password; so is a priority above Normal
(`pkexec renice`). PID 1 and kernel threads are never ended.

## Signing out and disconnecting accounts

The Users page has Sign out and Disconnect, as buttons above the list and in
an account's right-click menu (the user asked for this on 2026-10-26, with a
password asked for at that moment). Each asks first, as Windows does:
"Sign out Base? Unsaved work in their apps will be lost.", with Cancel and
Sign out.

- **Sign out another account**: logind's `TerminateUser` over D-Bus, with
  interactive authorization allowed (`ALLOW_INTERACTIVE_AUTHORIZATION`). Its
  polkit action, `org.freedesktop.login1.manage`, is `auth_admin_keep`, so
  polkit asks for an administrator's password right then, in the desktop's
  own dialog. Task Manager never asks for or sees a password, and adds no
  polkit rule (Patrone Remote Desktop's rule for seatless desktops covers
  network, power and drives, not this).
- **Sign out your own account**: the desktop's own log-out
  (`org.gnome.SessionManager.Logout` with mode 1, since Task Manager has
  already asked), which lets apps speak up about unsaved work.
- **Disconnect**: in Patrone an account's desktop is a graphical session
  with no seat (GDM's headless session) that keeps running, and signing in
  on the screen adds a session on a seat that shows it. Disconnect ends only
  those screen sessions (`TerminateSession`, same password rule), so the
  screen goes back to the sign-in screen and the desktop keeps running with
  its apps, as Disconnect does on Windows. It's off for an account whose
  desktop isn't shown on a screen, and for a plain desktop on a seat with
  nothing behind it, where ending the session would end the desktop.
  Disconnecting a Remote Desktop client isn't offered: Patrone Remote
  Desktop's connections aren't logind sessions (the headless desktop is),
  so there's nothing to end that wouldn't sign the account out.
- If the password isn't given or the request fails, it says so ("Base
  wasn't signed out. To sign out another account, an administrator's
  password is needed, and it wasn't given. Nothing was changed.") and
  changes nothing.

## Startup applications

Turning an entry off writes your own copy of it in `~/.config/autostart`
with `Hidden=true` (and `X-Patrone-Turned-Off=true`, so Task Manager knows
the copy is its own), as the autostart standard says; turning it back on
removes that copy. Your own entries get `Hidden` removed instead. Changes
take effect at the next sign-in, which the page says.

Entries the desktop never starts as apps aren't listed: those for other
desktops (`OnlyShowIn`, `NotShowIn`), those systemd runs as a service in
their place (`X-GNOME-HiddenUnderSystemd`), and the desktop's own parts
(`X-GNOME-Autostart-Phase`), which GNOME 50's session hands to systemd.
Almost every entry is `NoDisplay`, so that isn't a reason to hide one.

## Ctrl+Shift+Esc

On PatroneOS, a custom shortcut of the desktop's media keys ("Task
Manager", running `patrone-task-manager`), shipped as a default in a dconf
database of Patrone's own, the way ibus ships its settings. It's
`patrone-desktop`'s since 0.2.9 (this package's before 0.2.5), so the task
manager installed on its own changes no system settings; there, Settings >
Keyboard > Custom Shortcuts can add it, with `patrone-task-manager` as the
command.

| File | Installed as |
| --- | --- |
| `desktop/dconf/50-patrone-task-manager-shortcut` | `/etc/dconf/db/patrone.d/50-patrone-task-manager-shortcut`, compiled into `/etc/dconf/db/patrone` by `dconf update` in `patrone-desktop`'s postinst (and removed in its postrm) |
| `desktop/dconf/profile-user` | `/usr/share/dconf/profile/user`: `user-db:user`, then `system-db:patrone` |

Upgrading this package from before 0.2.5 removes its old keyfile
(`50-patrone-task-manager`, through `rm_conffile`) and compiles the
database again, or removes it where nothing's left.

Why not a schema override, as for Patrone's other defaults: a custom
shortcut lives at a path of its own
(`/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/patrone-task-manager/`),
and only dconf can give values to a path. Overriding the shared
`custom-keybinding` schema instead would make every shortcut a person adds
in Settings start out as Task Manager's: Settings resets a new shortcut's
keys to the schema's defaults. Nothing else Patrone ships sets
`custom-keybindings`, and nothing on Ubuntu 26.04 uses Ctrl+Shift+Esc.

dconf looks for the `user` profile in `/etc/dconf/profile` first, then in
each system data folder (`/usr/share/dconf/profile`), so an administrator's
own `/etc/dconf/profile/user` takes the place of Patrone's. Before this,
there was no `user` profile at all (only ibus's), so accounts read only
their own settings.

- Each account can change or remove the shortcut in Settings > Keyboard >
  Custom Shortcuts; that's written to the account's own settings.
- An account that already has custom shortcuts of its own keeps its own
  list, which doesn't have this one: `custom-keybindings` is a single list.
  Adding Task Manager's path to that list, or making a custom shortcut in
  Settings, gives it back.
- The profile is read when a program starts, so an account sees the
  shortcut from its next sign-in.

## Icon

`data/org.patrone.TaskManager.svg`: a white pulse line over a faint graph
on the Patrone Blue tile (`#4da0fc` to `#1d5cc4`) of Patrone Settings'
icon, with strokes bold enough to read at 16 pixels.

## Build and test

```
cargo build --release
cargo test --release          # formatting, stat parsing, unit names, scales
./packaging/build.sh patrone-task-manager -d   # on the dev VM
```

Testing without a desktop session: the throwaway GNOME Shell
(`shell/gnome/tests/headless.sh`) works, with the Task Manager's desktop
entry and icon copied into its `data` folder. Its fixed folder is shared
with other sessions, so this app was tried in a copy of the script with its
own folder (`/tmp/patrone-tm-test`) and display (`wayland-patrone-tm`):
`sed` the script's `here=`, `base=` and `display=` lines. Things to know:

- The throwaway shell has a D-Bus session of its own, so "Your services"
  lists nothing there: the account's systemd isn't on that bus. Run a copy
  with `DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus` (and the test
  shell's `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR`) to try it.
- Ctrl+Shift+Esc there needs the desktop's media-keys daemon:
  `dconf compile` the keyfile into a test database, point a profile in a
  test data folder at it (`file-db:PATH`), and run
  `/usr/libexec/gsd-media-keys` in the shell with that folder first in
  `XDG_DATA_DIRS` and the app on its `PATH`; then press the keys with
  Clutter's virtual keyboard in `headless.sh eval`.
- Use `pgrep -x patrone-task-ma` (the 15-character command name), not
  `pgrep -f`, whose pattern also matches the SSH command's own shell.
- polkit's password dialog can't be shown there: GNOME Shell's polkit agent
  answers for its own sign-in session, and a test shell started over SSH
  belongs to an SSH session that has ended (logind's State `closing`), so a
  request that needs a password fails at once with "interactive
  authentication required" (and `pkexec` refuses to run: "Refusing to
  render service to dead parents"). Try password prompts on a real desktop,
  and cancel them.

## Findings

- **GTK lists follow a row.** A list keeps its place by a row (its anchor)
  and follows that row when rows are added above it or the order changes
  (`gtklistitemmanager.c`), so a list re-sorted every second wandered off,
  and the first section's header ("Apps") sat just above the top. While a
  list is at the top, `table.rs` anchors it on the first row again after
  each change (`scroll_to`) and puts back to the top any scroll GTK makes
  while laying out the rows: GTK makes those with its own handler blocked
  (`gtk_list_base_set_adjustment_values`), so that's safe. Scrolling down
  lets the list go.
- **Never make a GTK list re-anchor right after its rows change.** Emitting
  the adjustment's `value-changed` then made GTK divide by the height of
  rows not laid out yet (`gtk_list_view_get_position_from_allocation`):
  SIGFPE, exit code 136, in `libgtk-4.so.1.2200.4`.
- **Make a list's menu once.** A `PopoverMenu` made and opened at once came
  out short by about 15 pixels per separator, cutting off the last items:
  GTK shows a section's separator only once it has looked up the section's
  actions. Each list now makes its menu with the list.
- **GTK's dark style doesn't always come with `prefers-color-scheme`.** In
  the user's session (`color-scheme` `prefer-dark`, theme `Yaru-blue-dark`,
  which plain GTK 4 apps can't load) GTK drew its own parts dark while a
  stylesheet's `@media (prefers-color-scheme: dark)` rules didn't apply,
  leaving light panels with light text. Every tint in `data/style.css` is
  drawn from `currentColor` instead, as Patrone Settings' are.
- `gio-unix` (where gtk-rs 0.22 keeps `DesktopAppInfo`) isn't in the VM's
  crate cache, so desktop entries are read with `glib::KeyFile`
  (`src/desktop.rs`).
- Right-clicking a cell selects its row through the selection model: the
  `listitem.select` action didn't reach the row from the cell's child.
- Task Manager's own use, measured on the VM where GTK draws in software:
  about 4% of one core on Processes and 3% on Performance. Reading 500
  processes takes 12 to 24 ms a second; GTK's three renderers (Vulkan,
  GL, cairo) were within a few milliseconds of each other.
