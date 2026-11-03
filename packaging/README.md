# Packaging

Debian packaging for each Patrone component, kept apart from the component's
source. `build.sh` puts the two together and builds the `.deb` on the dev VM:

```
./packaging/build.sh patrone-terminal -d
./packaging/build.sh patrone-shell-config
./packaging/build.sh patrone-branding
./packaging/build.sh patrone-default-apps
./packaging/build.sh patrone-remote-desktop
./packaging/build.sh patrone-gnome-taskbar
./packaging/build.sh patrone-file-manager -d
./packaging/build.sh patrone-settings -d
./packaging/build.sh patrone-desktop
./packaging/build.sh patrone-unpack -d
./packaging/build.sh patrone-task-manager -d
./packaging/build.sh patrone-notepad -d
./packaging/build.sh patrone-browser-chooser -d
./packaging/build.sh fonts-patroneos
./packaging/build.sh patrone-guest-tools
./packaging/build.sh patrone-hyperv-setup -d
./gnome-settings/build.sh
./gnome-software/build.sh
```

The packages land in `out/`. Install them with
`sudo apt install ./out/patrone-terminal_0.1.1_amd64.deb` (apt pulls in the
dependencies), and publish them to Patrone's apt repository with
`./packaging/publish.sh` (below, "Publishing").

| Package | Source | What it installs |
| --- | --- | --- |
| `patrone-terminal` | `terminal/` | The terminal, its launcher entry and icon, and an autostart entry that opens it at every login. Registers the `x-terminal-emulator` alternative (priority 60) and is the terminal `xdg-terminal-exec` picks in a Patrone Desktop session. |
| `patrone-shell-config` | `shell-config/` | Patrone's bash prompt (`/usr/share/patrone/bashrc`, loaded from `/etc/skel/.bash_aliases`), and `/etc/skel/.zshrc` and `.zprofile` for users who choose zsh. Bash stays the default shell. Also Patrone's `/usr/lib/command-not-found` and `/usr/share/patrone/windows-commands.tsv`: a Windows command typed in bash or zsh says it's invalid and names the Linux command to use. See `shell-config/README.md`. |
| `patrone-branding` | `branding/` | Patrone's name and logos in place of Ubuntu's: `os-release`, the boot splash (graphical and text), the boot menu name, the login screen logo, Settings → About, `/etc/issue` and the login message. Also the wallpaper, its family of variants, seventeen generated wallpapers and thirteen gallery wallpapers (listed in Settings → Appearance) in place of Ubuntu's wallpapers, which it replaces, the dock turned off, Patrone Sans as the typeface (GNOME's font settings, and fontconfig for sans-serif, system-ui and Segoe UI, snaps and Flatpak apps included (`/usr/share/fonts/patrone-fontconfig`, found through `FONTCONFIG_PATH` from `/etc/environment.d`); it depends on `fonts-patrone-sans`), Settings' "Ubuntu Desktop" page named "Desktop", and fastfetch's config and logo. See `branding/README.md`. |
| `patrone-default-apps` | `default-apps/` | No browser preinstalled: whichever browser is installed is the default for the web (`/etc/xdg/mimeapps.list`) and first in the favorites, and with none, Patrone Browser Chooser opens web links; Patrone File Manager for folders and in the favorites in place of Files; Patrone Unpack for archives, `.deb` packages and Flatpak files. Depends on `patrone-browser-chooser` (before 0.2.0, on `chromium-browser`, which installs the Chromium snap). See `default-apps/README.md`, also for removing Firefox. |
| `patrone-browser-chooser` | `browser-chooser/` | Patrone Browser Chooser: the first time a browser's needed (the taskbar's Web browser button, or a link opened with none installed) it asks which to install, from Chromium, Firefox, Brave, Google Chrome, Microsoft Edge, Vivaldi, Opera, LibreWolf, Zen Browser and GNOME Web; Patrone Software installs it from Flathub (`gnome-software --install`), and it becomes the default and opens. Its launcher (hidden from the Start menu) and icon. Depends on `flatpak` and `gnome-software-plugin-flatpak`; builds with `libgtk-4-dev`. See `browser-chooser/README.md`. |
| `patrone-remote-desktop` | `remote-desktop/` | Remote Desktop where each account in the `remote-desktop` group signs in with its own Linux password, checked by PAM: FreeRDP's proxy with Patrone's sign-in plugin, a root sign-in helper, and the `patrone-remote-desktop` command. Off until `patrone-remote-desktop enable`. Also a polkit rule that gives these seatless desktops what a desktop at the screen gets without a password (network control, power, drives). Builds with `freerdp3-dev` and `libpam0g-dev`. See `remote-desktop/README.md`. |
| `patrone-file-manager` | `file-manager/` | Patrone File Manager, its launcher entry, icon and settings schema (`org.patrone.FileManager`), and an autostart entry that reopens folders at sign-in only when Folder Options says to. Builds with `libgtk-4-dev` and GLib's resource and schema compilers. See `file-manager/README.md`. |
| `patrone-gnome-taskbar` | `shell/gnome/` | Patrone's taskbar and Start menu in place of GNOME's top bar: a GNOME Shell extension in `/usr/share/gnome-shell/extensions/`, its settings schema, and an override that turns it on for everyone. Loads when the desktop next starts: log out from inside it (disconnecting leaves it running). See `shell/gnome/README.md`. |
| `patrone-settings` | `settings/` | Patrone Settings: the taskbar's size, layout and Start button icon, the title bar buttons' corner and style, the desktop's font (PatroneOS Sans or Noto Sans; recommends `fonts-patroneos`), and five themes, with their style sheets for GTK apps in `/usr/share/patrone/themes` and `/usr/share/patrone/titlebuttons`, an autostart entry that writes the account's style sheets at every sign-in, the `org.patrone.desktop` schema, and the choices' pictures in `/usr/share/patrone/settings/previews` (drawn at build time) for GNOME Settings' Patrone page. Builds with `libgtk-4-dev`. See `settings/README.md`. |
| `gnome-control-center`, `gnome-control-center-data` (Patrone's build) | `gnome-settings/` | Ubuntu's GNOME Settings with a Patrone page (the same choices as Patrone Settings) and no Telemetry page, versioned Ubuntu's plus `patrone1`. Built by `gnome-settings/build.sh`, not `build.sh`: it fetches Ubuntu's source first. Builds with Ubuntu's build dependencies for `gnome-control-center`. See `gnome-settings/README.md`. |
| `gdm3`, `libgdm1`, `gir1.2-gdm-1.0` (Patrone's build) | `gdm/` | Ubuntu's GDM, counting Hyper-V's synthetic video as the boot display so the login screen doesn't wait 10 seconds for a primary GPU on a Hyper-V VM, versioned Ubuntu's plus `patrone1`. Built by `gdm/build.sh`, not `build.sh`: it fetches Ubuntu's source first. Builds with Ubuntu's build dependencies for `gdm3`. See `gdm/README.md`. |
| `language-selector-common`, `language-selector-gnome` (Patrone's build) | `language-selector/` | Ubuntu's Language Support without its words that point at Ubuntu: the input method's tip, the error message's Launchpad link and the help, versioned Ubuntu's plus `patrone1`. Built by `language-selector/build.sh`, not `build.sh`: it fetches Ubuntu's source first. Builds with Ubuntu's build dependencies for `language-selector`. See `language-selector/README.md`. |
| `patrone-desktop` | `desktop/` | Patrone's desktop metapackage: lists Ubuntu's desktop packages itself, in place of Ubuntu's metapackages, less Ubuntu Pro, Software Updater and hwctl, depends on `patrone-branding`, recommends Patrone's apps, and removes what reports to Canonical (whoopsie, ubuntu-insights, ubuntu-report, and apport-gtk's crash dialog) through `Conflicts`. First-login setup skips Ubuntu's reporting and Ubuntu Pro pages. The clock comes from NTS servers run by Cloudflare, Netnod and PTB instead of `ntp.ubuntu.com`, NetworkManager checks for Wi-Fi sign-in pages with GNOME's server, and nothing offers to upgrade to Ubuntu's next release. See `desktop/README.md`. |
| `patrone-unpack` | `unpack/` | Patrone Unpack: archives (zip, tar with gzip, xz, bzip2 or zstd, 7z, rar, single compressed files) open as folders with Extract all and Extract here; `.deb` packages and Flatpak files (`.flatpakref`, `.flatpak`, `.flatpakrepo`) open with what they are and Install (Flatpak apps for the account, with no password; `.deb` packages for everyone). Also its launcher entry and icon, the root helper that installs a `.deb` with apt (`/usr/libexec/patrone-unpack/install-deb`, run by pkexec) and its polkit action (`org.patrone.unpack.install-deb`, an administrator's password every time), and Flathub's `.flatpakrepo` (`/usr/share/patrone-unpack/`), added to an account when a Flatpak app's runtime comes from Flathub. Depends on `flatpak`. Builds with `libgtk-4-dev` and links Ubuntu's `libarchive13t64` by its run-time name, so libarchive's development package isn't needed. See `unpack/README.md`. |
| `patrone-task-manager` | `task-manager/` | Patrone Task Manager, in the place of Ubuntu's Resources (`Conflicts` and `Replaces: resources`; Resources is only recommended by `ubuntu-desktop-minimal`, so Ubuntu's desktop stays): apps and processes with what each uses, performance graphs, each app's processor time since the computer started, startup applications, users (with Sign out and Disconnect, through logind, so polkit asks for an administrator's password for another account), every process, and services. Also its launcher entry and icon, and Ctrl+Shift+Esc: a custom shortcut in a Patrone dconf database (`/etc/dconf/db/patrone.d/`, compiled by `dconf update` in postinst) read through a vendor profile, `/usr/share/dconf/profile/user` (an administrator's `/etc/dconf/profile/user` takes its place). Depends on `dconf-cli`; recommends `pkexec`. Builds with `libgtk-4-dev`. See `task-manager/README.md`. |
| `patrone-notepad` | `notepad/` | Notepad, a plain text editor: tabs across the title bar, File, Edit and View menus, find and replace, go to line, word wrap, zoom, a font of your choice (PatroneOS Mono by default; recommends `fonts-patroneos`), printing, a light or dark look, and a status bar with the line and column, characters, zoom, line endings and encoding. By default it keeps the open tabs and their unsaved changes between sessions (`~/.local/state/patrone-notepad`), which Settings turns off. `/usr/bin/patrone-notepad` and the `notepad` command, its launcher entry and icon; `patrone-default-apps` makes it the app for plain text. Builds with `libgtk-4-dev`. See `notepad/README.md`. |
| `fonts-patroneos` | `fonts/` | PatroneOS's own fonts, Sans, Mono, Document, Serif and Classic (28 static TrueType fonts), in `/usr/share/fonts/truetype/patroneos/`, with their FONTLOG and the OFL 1.1. They're drawn and built in their own repository, [billsecond/patroneos-fonts](https://github.com/billsecond/patroneos-fonts); this package installs the release `fonts/release.txt` pins, which `build.sh` fetches and checks against its SHA-256 before building (the build itself runs offline). Recommends Noto (other scripts), Liberation and Caladea (the classic serif fonts). See `fonts/README.md`. |
| `fonts-patrone-sans` | `fonts/` | Transitional: Patrone Sans gave way to PatroneOS's fonts, and this package now only depends on `fonts-patroneos`, so upgrading from Patrone Sans installs them and removes it. |
| `patrone-software` | `software/` | GNOME Software as Patrone's app store, in place of Ubuntu's App Center (Canonical's `snap-store` snap, which its postinst removes when it's first installed). It depends on Patrone's build of `gnome-software` and its Flatpak plugin, not the snap one, so Canonical's Snap Store isn't browsed. Its Featured tab shows Patrone's picks (`/usr/share/swcatalog/xml/org.patrone.Software.Featured.xml`) in place of GNOME's, which preinst diverts to `org.gnome.Software.Featured.xml.gnome`. See `software/README.md`. |
| `gnome-software`, `gnome-software-common` and its plugins (Patrone's build) | `gnome-software/` | Ubuntu's GNOME Software with a Featured tab before Explore, where it opens, versioned Ubuntu's plus `patrone1`. Built by `gnome-software/build.sh`, not `build.sh`: it fetches Ubuntu's source first. Builds with Ubuntu's build dependencies for `gnome-software`. See `gnome-software/README.md`. |
| `patrone-guest-tools` | `guest-tools/` | Hyper-V's integration services and VirtualBox's Guest Additions, each only under its own hypervisor (`ConditionVirtualization`). Hyper-V: Ubuntu's `linux-cloud-tools` daemons for the HWE kernel (key-value exchange, VSS), Patrone's unit for file copy (`Copy-VMFile`; Ubuntu's waits for a device Linux 6.10 dropped), and the host's clock as one of chrony's sources. VirtualBox: recommends Ubuntu's `virtualbox-guest-utils` and `virtualbox-guest-x11` (multiverse), starts VBoxClient in Wayland sessions (user units), and adds every account to `vboxsf` and `vboxdrmipc` so shared folders work. See `guest-tools/README.md`. |
| `patrone-hyperv-setup` | `hyperv-setup/` | Virtual Machine Setup: at each desktop login in a Hyper-V VM (a user unit, `ConditionVirtualization=microsoft`, not in the installer's live session), a dialog that offers to set up what's missing with one password (a root helper run by `pkexec`, polkit action `org.patrone.hyperv-setup.apply`): `patrone-guest-tools`, and `patrone-remote-desktop`'s service for VMConnect's enhanced session; and the PowerShell command for Windows' side (the Hyper-V socket transport, integration services turned off), with a Copy button. Shows nothing when everything's set up, and "Don't show this again" turns it off for the account. A udev rule notes when Hyper-V opens an enhanced session, which is how it knows the transport is set. Depends on `pkexec`; builds with `libgtk-4-dev`. See `hyperv-setup/README.md`. |

Notes:

- Both are native packages (`3.0 (native)`): Patrone is its own upstream.
- Rust packages build with Ubuntu's `cargo` and `rustc` (1.93), so the crates
  declare `rust-version = "1.93"`. `build.sh` vendors the crates so the build
  needs no network, and `third-party-licenses.sh` copies every vendored
  crate's licence and copyright notices into
  `/usr/share/doc/PACKAGE/third-party-licenses.txt.gz`, as their licences
  ask for copies of the program (`docs/COMPLIANCE.md`). On the dev VM Rust comes from rustup, so pass `-d` to skip
  the Build-Depends check, or install Ubuntu's `cargo` and `rustc`.
- `patrone-shell-config` leaves Ubuntu's shell setup as it is: Ubuntu's
  `/etc/skel/.bashrc` already loads `~/.bash_aliases`, which no package owns,
  so that's where the prompt hooks in. The one file it replaces is
  `/usr/lib/command-not-found`, which its `preinst` moves aside with
  `dpkg-divert` (to `*.ubuntu`), so `command-not-found`'s updates land there.
- `patrone-branding` has to replace files that belong to Ubuntu's packages
  (`/usr/lib/os-release`, `/etc/issue` and others in `base-files`). Its
  `preinst` moves each aside with `dpkg-divert` (to `*.ubuntu`), so Ubuntu's
  updates land there instead of over Patrone's, and `postrm` moves them back
  when the package is removed. Three of Ubuntu's login-message scripts (help
  links, news, Ubuntu Pro) are moved aside with no replacement. The boot
  splash is chosen through the `default.plymouth` and `text.plymouth`
  alternatives (priority 200), and installing or removing the package
  rebuilds the initramfs and the GRUB menu.
- Ubuntu sets some GNOME defaults twice: plainly and again for its own
  session (`[schema:ubuntu]` in `10_ubuntu-settings.gschema.override`).
  The session-specific value beats any plain default, so Patrone's overrides
  set both.
- Later these builds move into a clean `sbuild` chroot (see `docs/PLAN.md`).

## Publishing

Installs get Patrone's updates from its apt repository,
[billsecond/patrone-apt](https://github.com/billsecond/patrone-apt), on
GitHub Pages (`https://billsecond.github.io/patrone-apt`): `patrone-desktop`
installs its source, key and pins (`desktop/README.md`, "Updates").
`publish.sh` puts builds there, on the dev VM, where the signing key is:

```
./packaging/publish.sh patrone-terminal       # its newest build in out/
./packaging/publish.sh gdm3                    # with libgdm1 and the rest of that build
./packaging/publish.sh out/patrone-desktop_0.2.1_all.deb
./packaging/publish.sh --all                   # the newest build of every package in out/
./packaging/publish.sh --dry-run --all         # what would change
```

It uploads the new debs to the repository's `pool` release, writes the
index (`dists/resolute`) and signs it, commits and pushes it to
`~/src/patrone-apt`, and GitHub Pages has it a minute or two later. A
package already published at that version or newer is skipped. Every
install takes what's published, by itself, within a day.

**Merges publish themselves, within a day.** `main-updates.sh` builds every
package whose version on `main` is newer than the published one, and a new
revision of Patrone's GDM, GNOME Settings or first-login setup changes (the
`revision=` line in their `build.sh`), and publishes what built. It runs
once a day on the dev VM, first in the daily job below, from that job's
clean checkout of `main`, and it won't run anywhere but a checkout of `main`
as it is on GitHub. It publishes only the debs it has just built. So a
change that should reach installs needs a version bump, as always.

```
./packaging/main-updates.sh --check    # only say what's behind
```

A build or publish that fails is in the job's output (`journalctl --user -u
patrone-ubuntu-updates`), fails the run, and becomes an issue in
billsecond/patrone-apt titled "PACKAGE VERSION from main: ...", which
closes by itself once that version, or a newer one, is published.
`patrone-installer` isn't built there, since it needs `installer/build.sh`
first: a new version of it on `main` becomes an issue saying to build it by
hand.

To publish one sooner, on the dev VM, in a clean, up-to-date checkout of
`main` (`~/src/patrone-linux`, or a fresh worktree of `origin/main` when
that one is on a branch):

```
export PATH=$HOME/.cargo/bin:$PATH             # over SSH, for the Rust packages
./packaging/build.sh patrone-file-manager -d
./packaging/publish.sh patrone-file-manager
```

Name the package rather than using `--all`: `out/` keeps builds from
branches too, and `--all` takes the newest build of each, merged or not.

The key is on the dev VM only, in `~/.local/share/patrone-apt/gnupg`
(Ed25519, fingerprint `8FE6 1B83 753C 2636 CD74 0259 4FC3 52E3 D384 225F`),
backed up as `~/.local/share/patrone-apt/secret-key-backup.asc`. Its public
half is `desktop/apt/patrone-archive-keyring.gpg`.

### Ubuntu's updates to the packages Patrone rebuilds

GDM, GNOME Settings, GNOME Software, first-login setup and Language Support
are Ubuntu's packages with Patrone's changes, versioned Ubuntu's plus `patroneN`, and installs keep
Patrone's build even when Ubuntu has a newer one (`desktop/apt/patrone.pref`).
So each of Ubuntu's updates to them has to be built again with Patrone's
changes. `ubuntu-updates.sh` does that: it compares Patrone's builds with
what Ubuntu has published for resolute (Launchpad), builds what's behind
with `gdm/build.sh`, `gnome-settings/build.sh`, `gnome-software/build.sh`,
`gnome-initial-setup/build.sh` or `language-selector/build.sh` at Ubuntu's
new version, from a clean
checkout of `main`, and publishes it. It also says when the installer
snap's channel moves past the revision `installer/build.sh` follows (that
one waits for the next ISO: `installer/README.md`). What it can't do, a
build whose patches no longer apply for one, becomes an issue in
billsecond/patrone-apt.

```
./packaging/ubuntu-updates.sh --check    # only say what's behind
./packaging/ubuntu-updates.sh            # build and publish it
```

It runs daily on the dev VM as a user timer
(`scripts/vm/patrone-ubuntu-updates.timer`, whose service file says how to
install it), while the account is signed in, after `main-updates.sh`
(above), which it runs first. Separately, a daily GitHub
Actions check in billsecond/patrone-apt opens an issue for each package
Ubuntu has updated and closes it when Patrone's rebuild is published, so a
build that's behind shows up even while the VM is off.

A script on the VM rather than a GitHub Actions build, because the signing
key stays off GitHub, and the VM already has Ubuntu 26.04 with every build
dependency these three need.
