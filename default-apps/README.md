# Default apps

Which apps Patrone uses by default, packaged as `patrone-default-apps`
(`packaging/patrone-default-apps`).

| File | Installed as | What it does |
| --- | --- | --- |
| `mimeapps.list` | `/etc/xdg/mimeapps.list` | Web pages and links open in whichever browser is installed, and with none, in Patrone Browser Chooser (below); Patrone File Manager opens folders (Nautilus is the fallback when it isn't installed); Patrone Unpack opens archives (zip, tar, 7z, rar and single compressed files), .deb packages and Flatpak files (`.flatpak`, `.flatpakref`, `.flatpakrepo`), in place of Nautilus for archives and GNOME Software for .deb packages, which stay as the fallbacks (Canonical's App Center is gone: `patrone-software`); Notepad (`notepad/`) opens plain text, Markdown and empty files, in place of GNOME's Text Editor, which stays as the fallback; GNOME's Video Player (Showtime, which `patrone-desktop` recommends) opens videos, every type it knows, so it stays the default when another player is installed (the user, 2026-10-28: "add Gnmoe Video Player as a default as part of the install") |
| `91_patrone-default-apps.gschema.override` | `/usr/share/glib-2.0/schemas/` | The favorites (the Start menu's pinned apps): the web browser first, then Patrone Terminal and Patrone File Manager (in place of Ubuntu's Files) |

## The web browser

PatroneOS comes with no web browser (the user, 2026-10-28: "Ask the
user when they click on the browser with a modal if they want to install
Chromium, Fire Fox, Brave, etc, then use the current patrone apps to open
that up and set up the install"). The first time one's needed, **Patrone
Browser Chooser** (`browser-chooser/`, package `patrone-browser-chooser`)
asks which to install, and Patrone Software installs it from Flathub. It
comes up from:

- the taskbar's Web browser button (`fallback` in the taskbar's
  `roles.js`), and
- any link or web page opened while no browser is installed, from Help,
  first-login setup or any app: it's the last of `mimeapps.list`'s choices
  for `x-scheme-handler/http`, `https`, `text/html` and
  `application/xhtml+xml`, and opens the link once the browser's installed.

Each of those four lists names the browsers the chooser offers, Flathub's
IDs first in the chooser's order, then the same browsers as Ubuntu's snaps
(`chromium_chromium.desktop`, `firefox_firefox.desktop`) or their makers'
own packages, then the chooser. GLib takes the first one installed, so
once any of them is installed it opens links for every account that hasn't
picked its own; the chooser also makes its browser the account's own
default (`~/.config/mimeapps.list`). The chooser's `browsers.rs` has the
same IDs; change both together. A browser on neither list (one installed
from elsewhere) is offered under "On this computer" when the chooser opens,
and the taskbar's button opens it.

The favorites name the same Flathub browsers and Chromium's snap ahead of
Patrone's apps; GNOME Shell shows only those installed, so the browser an
account installs comes first.

Before 0.2.0 the package depended on Ubuntu's `chromium-browser`, a small
package that installs Canonical's Chromium snap. Upgrading leaves Chromium
installed, and it stays the default.

## Firefox and Chromium's stub packages

Ubuntu's `firefox` package is a small one that installs Firefox's snap,
every time it's installed or upgraded, and `chromium-browser` is the same
for Chromium's. `ubuntu-desktop-minimal` only recommends `firefox`, so it
can go without taking Ubuntu's desktop with it. Patrone's ISO purges both,
and stops if a browser is left in it (`iso/build-iso.sh`). On a system
installed some other way, or from an ISO made before 2026-10-28:

```
sudo snap remove firefox
sudo apt purge firefox
```

A hold (`apt-mark hold firefox`), which this page used to suggest, isn't
enough: it leaves it to every updater to honour, and Ubuntu has published an
update to the stub since 26.04's release.

Users' own choices still win: `~/.config/mimeapps.list` (Settings → Apps →
Default Apps) and their own favorites.
