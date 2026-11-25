# What PatroneOS changes in Ubuntu

Everything Patrone adds to Ubuntu 26.04.1, builds again with its own
changes, takes out or configures differently, in one list. Each line points
to the folder whose README has the details. Versions aren't here: each
package's is the top line of `packaging/PACKAGE/debian/changelog`.

**Keep this list current.** A new package, a new rebuild of one of
Ubuntu's, or something more that Patrone removes or replaces goes here in
the same change.

## Patrone's own packages

| Package | What it is | Source |
| --- | --- | --- |
| `patrone-desktop` | The metapackage that makes an Ubuntu install Patrone's desktop: Ubuntu's desktop packages, listed itself, less what reports to Canonical, plus Patrone's system settings ([below](#system-settings)) and a few tools everyone gets | `desktop/` |
| `patrone-branding` | The name, logos, wallpaper, boot splash, boot menu, login screen theme, `os-release` and `lsb-release`, `/etc/issue`, the message of the day, `/etc/legal`, mouse pointers, font settings, the installer's look, and Help in place of Ubuntu's (`ubuntu-docs`) | `branding/` |
| `patrone-gnome-taskbar` | The taskbar and Start menu, as a GNOME Shell extension, until Patrone's own shell replaces GNOME Shell | `shell/gnome/` |
| `patrone-settings` | Patrone Settings: taskbar, desktop, wallpaper, title bar buttons, theme, mouse pointer; also the engine behind GNOME Settings' Patrone page | `settings/` |
| `patrone-terminal` | Patrone Terminal, the default terminal, in place of Ptyxis | [billsecond/patrone-terminal](https://github.com/billsecond/patrone-terminal) |
| `patrone-shell-config` | bash with Patrone's two-line prompt, zsh set up the same way, and pointers for Windows commands | [billsecond/patrone-shell-config](https://github.com/billsecond/patrone-shell-config) |
| `patrone-file-manager` | File Manager, the app for folders, in place of Ubuntu's Files | [billsecond/patrone-file-manager](https://github.com/billsecond/patrone-file-manager) |
| `patrone-task-manager` | Task Manager, in place of Resources; Ctrl+Shift+Esc opens it | [billsecond/patrone-task-manager](https://github.com/billsecond/patrone-task-manager) |
| `patrone-notepad` | Notepad, the app for plain text and Markdown, in place of GNOME's Text Editor (which stays as the fallback) | [billsecond/patrone-notepad](https://github.com/billsecond/patrone-notepad) |
| `patrone-unpack` | Patrone Unpack: archives, `.deb` packages and Flatpak files | [billsecond/patrone-unpack](https://github.com/billsecond/patrone-unpack) |
| `patrone-software` | Patrone Software: GNOME Software with Patrone's picks, in place of Canonical's App Center | `software/` |
| `patrone-browser-chooser` | No browser comes installed; this asks which to install the first time one's needed | `browser-chooser/` |
| `patrone-default-apps` | Which app opens what (`mimeapps.list`), and the Start menu's pinned apps | `default-apps/` |
| `patrone-remote-desktop` | One desktop per account, on the screen and over Remote Desktop, signed in with the account's own password | [billsecond/patrone-remote-desktop](https://github.com/billsecond/patrone-remote-desktop) |
| `patrone-installer` | Canonical's installer front end, built again as PatroneOS's ([below](#ubuntus-packages-built-again)) | `installer/` |
| `patrone-guest-tools` | Hyper-V and VirtualBox guest support | [billsecond/patrone-guest-tools](https://github.com/billsecond/patrone-guest-tools) |
| `patrone-hyperv-setup` | Virtual Machine Setup: offers what PatroneOS needs under Hyper-V | [billsecond/patrone-hyperv-setup](https://github.com/billsecond/patrone-hyperv-setup) |
| `fonts-patroneos` | PatroneOS Sans, Mono, Document, Serif and Classic, from a release of [billsecond/patroneos-fonts](https://github.com/billsecond/patroneos-fonts) | `fonts/` |
| `fonts-patrone-sans` | Transitional package for `fonts-patroneos` | `packaging/fonts-patrone-sans/` |

Not shipped yet: Patrone's compositor (`compositor/`, Rust and Smithay).
Today's desktop is GNOME Shell with Patrone's taskbar.

## Patrone's apps on any Linux

Each app's repository also releases it as a Flatpak, on the GNOME 50
runtime, and Patrone's app index
([billsecond/patrone-app-index](https://github.com/billsecond/patrone-app-index))
lists them, so they install and update on any Linux with Flatpak:

| App | ID | In the sandbox |
| --- | --- | --- |
| Notepad | `org.patrone.Notepad` | Portals only, PatroneOS's fonts in it |
| Patrone Unpack | `org.patrone.Unpack` | Home and removable drives; installs Flatpak files with the computer's flatpak, but not `.deb` packages |
| Patrone Terminal | `org.patrone.Terminal` | Runs the computer's shell (flatpak-spawn --host), with VTE 0.84 built in |
| Patrone Task Manager | `org.patrone.TaskManager` | The computer's processes, through a small helper on it (glibc 2.34 or newer) |
| Patrone File Manager | `org.patrone.FileManager` | Every file and drive, the computer's programs, files opened through the portal; no desktop |

Shell config, the guest tools, Virtual Machine Setup and Remote Desktop set
up the system itself, so they're `.deb` packages only, for Ubuntu 26.04 and
systems built on it.

## Ubuntu's packages, built again

Each is Ubuntu's package with Patrone's changes, its version Ubuntu's plus
`patroneN`. `desktop/apt/patrone.pref` keeps Patrone's builds ahead of
Ubuntu's, and `packaging/ubuntu-updates.sh` builds each of Ubuntu's updates
again with the same changes. Each names Patrone as its maintainer and
Ubuntu Developers as the original one (`docs/COMPLIANCE.md`, item 13).

| Package | Patrone's changes | Source |
| --- | --- | --- |
| GDM (`gdm3`) | No 10-second stall of the boot splash on Hyper-V; a locked desktop can be unlocked; no login screen over the disc's installer | `gdm/` |
| GNOME Settings (`gnome-control-center`) | A Patrone page; no Telemetry page; Patrone's logo on About; Remote Desktop first under System › Remote Desktop | `gnome-settings/` |
| GNOME Software (`gnome-software`) | A Featured tab, before Explore | `gnome-software/` |
| First-login setup (`gnome-initial-setup`) | Says PatroneOS, not Ubuntu; offers Patrone Software, not App Center; Patrone's pictures in place of Ubuntu's logo, mascot and orange | `gnome-initial-setup/` |
| Language Support (`language-selector`) | Its tip, error and help no longer name Ubuntu or send bug reports to Launchpad | `language-selector/` |
| The installer (`ubuntu-desktop-provision`'s front end, as `patrone-installer`) | A PatroneOS flavor: its name, wordmark, P and blue in place of Ubuntu's; a plain spinner; Landscape, other companies' logos and Windows' logo out; a licence page | `installer/` |

## What Patrone takes out

| What | Why | Where |
| --- | --- | --- |
| Ubuntu Pro's client, its services and its apt hook; `ubuntu-advantage-desktop-daemon` | Canonical's service | `patrone-desktop` (Conflicts) |
| Software Updater (`update-manager`), `update-notifier`, the release upgrader | Ubuntu's logo, Pro rows and release upgrades; Patrone Software shows updates | `patrone-desktop` (Conflicts, Provides) |
| `hwctl` | Canonical's hardware-certification tool | `patrone-desktop` (Conflicts), the ISO's seed |
| `apport-gtk`, `ubuntu-insights`, `ubuntu-report`'s autostart entry | They report to Canonical | `patrone-desktop` (Conflicts), the ISO |
| First-login setup's Ubuntu Pro and "Help improve Ubuntu" pages | The same | `desktop/gnome-initial-setup/vendor.conf` |
| `ubuntu-desktop`, `ubuntu-desktop-minimal`, `ubuntu-minimal` | They need what's above; `patrone-desktop` lists their packages itself | `patrone-desktop` |
| Ubuntu's Help (`ubuntu-docs`) | Ubuntu's name and logo; `patrone-branding` takes its place, and GNOME's Help comes with `patrone-desktop` | `patrone-branding` |
| `network-manager-config-connectivity-ubuntu` | Asks Ubuntu's server; Patrone asks GNOME's | `patrone-desktop` |
| Resources | Task Manager replaces it | `patrone-task-manager` (Conflicts) |
| Ptyxis | Patrone Terminal replaces it | The ISO |
| Firefox and Chromium (stubs and snaps) | No browser comes installed | The ISO |
| Thunderbird (stub, language packs, snap) | Canonical's build of Mozilla's app | The ISO |
| App Center (`snap-store`) and Security Center snaps | Patrone Software replaces App Center | The ISO, `patrone-software` |
| The seed system for TPM-backed encryption | The installer doesn't offer it | The ISO |

## System settings

All from `patrone-desktop` (`desktop/README.md` has the table):

- **The clock:** NTS-authenticated time from Cloudflare, Netnod and PTB, in
  place of `ntp.ubuntu.com` (not TimeNL: `docs/COMPLIANCE.md`, item 7).
- **The network check:** GNOME's `nmcheck.gnome.org`, in place of Ubuntu's.
- **Release upgrades:** never offered; Patrone moves to a new Ubuntu base
  itself.
- **Updates:** Patrone's apt repository and its key, Patrone's builds first,
  changes of the repository's label accepted, and automatic updates from
  the Patrone origin.
- **The session mode:** Ubuntu's GNOME session mode, with Patrone's taskbar
  among its own extensions.
- **`connections.txt`:** what the system connects to, and who runs each.
- **Tools everyone gets:** fastfetch, nano, vim, btop, htop, tree, ripgrep,
  fzf, bat, ncdu, 7zip, Breeze cursors, GNOME's Video Player and Help.

## The disc

`iso/build-iso.sh` remasters Ubuntu's 26.04.1 desktop ISO (`iso/README.md`):

- Patrone's two layers, the installed system and the live session, in
  place of all of Ubuntu's.
- The snap seed reset and preseeded again; the build stops if a browser is
  found.
- What nothing needs any more is removed, and what Patrone's packages remove
  is purged.
- Patrone's boot splash, and its live layer as the one to boot.
- The disc's list of sources (`casper/sources.manifest`) and the source
  bundle beside it (`iso/build-source.sh`).

## Repositories

PatroneOS is made in fifteen repositories, thirteen public and two private:

| Repository | Visibility | Created | What it is |
| --- | --- | --- | --- |
| [billsecond/patroneos](https://github.com/billsecond/patroneos) | Public | 2026-10-28 | The OS itself: desktop, apps, packaging, ISO (this one) |
| [billsecond/patrone-apt](https://github.com/billsecond/patrone-apt) | Public | 2026-10-28 | The apt repository installs get Patrone's updates from, on GitHub Pages (`packaging/publish.sh`); its `apps` component has only Patrone's apps, for Ubuntu (`packaging/apps.list`) |
| [billsecond/patrone-app-index](https://github.com/billsecond/patrone-app-index) | Public | 2026-10-27 | Patrone Software's app listings, which link to each app's own releases; it hosts no apps |
| [billsecond/patrone-notepad](https://github.com/billsecond/patrone-notepad) | Public | 2026-11-24 | Notepad, with its history from this repository; `packaging/patrone-notepad/release.txt` pins the release PatroneOS builds |
| [billsecond/patrone-unpack](https://github.com/billsecond/patrone-unpack) | Public | 2026-11-24 | Patrone Unpack, the same way (`packaging/patrone-unpack/release.txt`) |
| [billsecond/patrone-shell-config](https://github.com/billsecond/patrone-shell-config) | Public | 2026-11-24 | The bash and zsh setup, the same way (`packaging/patrone-shell-config/release.txt`) |
| [billsecond/patrone-guest-tools](https://github.com/billsecond/patrone-guest-tools) | Public | 2026-11-24 | The Hyper-V and VirtualBox guest tools, the same way (`packaging/patrone-guest-tools/release.txt`) |
| [billsecond/patrone-terminal](https://github.com/billsecond/patrone-terminal) | Public | 2026-11-24 | Patrone Terminal, the same way (`packaging/patrone-terminal/release.txt`) |
| [billsecond/patrone-file-manager](https://github.com/billsecond/patrone-file-manager) | Public | 2026-11-24 | Patrone File Manager, the same way (`packaging/patrone-file-manager/release.txt`) |
| [billsecond/patrone-task-manager](https://github.com/billsecond/patrone-task-manager) | Public | 2026-11-24 | Patrone Task Manager, the same way (`packaging/patrone-task-manager/release.txt`) |
| [billsecond/patrone-remote-desktop](https://github.com/billsecond/patrone-remote-desktop) | Public | 2026-11-24 | Patrone Remote Desktop, the same way (`packaging/patrone-remote-desktop/release.txt`) |
| [billsecond/patrone-hyperv-setup](https://github.com/billsecond/patrone-hyperv-setup) | Public | 2026-11-24 | Patrone Virtual Machine Setup, the same way (`packaging/patrone-hyperv-setup/release.txt`) |
| [billsecond/patroneos-fonts](https://github.com/billsecond/patroneos-fonts) | Public | 2026-10-29 | The five PatroneOS font families; `fonts-patroneos` packages a release of them (`fonts/release.txt`) |
| `billsecond/patrone-website` | Private | 2026-10-28 | patroneos.com and its admin dashboard |
| `billsecond/patroneos-archive` | Private | 2026-10-25 | The OS's history from before it went public, website included |

## Not part of the system

The website, the app index, the Discord set (`branding/discord/`) and the
development scripts (`scripts/`).
