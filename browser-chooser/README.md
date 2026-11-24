# Patrone Browser Chooser

PatroneOS comes with no web browser. The first time one's needed,
Patrone Browser Chooser asks which to install, has Patrone Software install
it, makes it the default for the web and opens it, with the link that was
waiting. Packaged as `patrone-browser-chooser` (Rust and GTK 4, app ID
`org.patrone.BrowserChooser`). The user asked for it on 2026-10-28: "Ask
the user when they click on the browser with a modal if they want to
install Chromium, Fire Fox, Brave, etc, then use the current patrone apps
to open that up and set up the install."

```
patrone-browser-chooser [LINK...]   ask, install, then open each LINK
patrone-browser-chooser --list      the browsers it offers, and which are installed
```

## Where it comes up

- **The taskbar's Web browser button.** The slot opens the default app for
  web links; with no browser installed, that's the chooser, and the slot
  still shows (`fallback` in `shell/gnome/patrone-taskbar@patroneos.com/roles.js`).
- **Any link opened with no browser installed**, from Help, first-login
  setup or any app. `patrone-default-apps` makes the chooser the last
  choice in `/etc/xdg/mimeapps.list` for `x-scheme-handler/http`, `https`,
  `text/html` and `application/xhtml+xml`, after the browsers it offers
  (`default-apps/README.md`). It keeps the links and opens them in the
  browser once it's installed. A second link while it's open goes to the
  same window.

Its launcher is hidden (`NoDisplay`): it isn't in the Start menu, and as it
isn't in the `WebBrowser` category, nothing offers it as a browser.

## What it offers

All from Flathub, which Patrone Software installs from: it leaves out GNOME
Software's Snap Store plugin (`software/README.md`), so Flathub is the one
source it has for every browser. Patrone hosts no apps.

| Browser | Flathub ID |
| --- | --- |
| Chromium | `org.chromium.Chromium` |
| Firefox | `org.mozilla.firefox` |
| Brave | `com.brave.Browser` |
| Google Chrome | `com.google.Chrome` |
| Microsoft Edge | `com.microsoft.Edge` |
| Vivaldi | `com.vivaldi.Vivaldi` |
| Opera | `com.opera.Opera` |
| LibreWolf | `io.gitlab.librewolf-community` |
| Zen Browser | `app.zen_browser.zen` |
| GNOME Web | `org.gnome.Epiphany` |

The user's three first, then the other well-known ones. Each row has the
icon from Flathub's catalog (which Flatpak keeps in
`/var/lib/flatpak/appstream/flathub/`), one line about it and **Install**,
or **Open** if it's installed already (from Flathub, as a snap or as its
maker's package). A browser installed some other way that isn't on the
list shows first, under "On this computer", with **Use**. Flathub's Google
Chrome and Microsoft Edge are packaged by Flathub's community, not Google
or Microsoft, and download the browser from its maker as they install;
Patrone Software shows that on their pages.

## Installing

Picking a browser runs

```
gnome-software --install=system/flatpak/flathub/<ID>/stable --interaction=full
```

so Patrone Software opens on the browser's page, already installing it for
everyone on the computer. On Patrone's desktops that asks for an
administrator's password once (`patroneDesktopOnceActions` in
`data/patrone-desktops.rules` in [billsecond/patrone-remote-desktop](https://github.com/billsecond/patrone-remote-desktop)). The chooser shows
"Installing ..." meanwhile, with **Choose another browser** and **Close**
(closing doesn't stop the install).

On a new system Patrone Software may not have fetched Flathub's list of
apps yet (it does shortly after the first sign-in), and can't find the
browser without it. The chooser fetches it first then
(`flatpak update --system --appstream flathub`, which Patrone's desktops
may do with no password).

It checks every second for the browser's launcher, which Flatpak adds as
the install finishes. Then it makes the browser the account's default for
the four types above (as Settings → Apps → Default Apps does, and the
taskbar's Web browser menu), opens it with the waiting links, or none, and
closes.

## Checked

See "Patrone Browser Chooser" in `docs/NEXT.md`.
