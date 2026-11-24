# Patrone's desktop

`patrone-desktop` (`packaging/patrone-desktop`) is the package that makes an
Ubuntu install Patrone's desktop. It depends on `patrone-branding` and on
Ubuntu's desktop, which it lists itself (below), recommends Patrone's apps,
and leaves out everything that reports to Canonical. It also recommends a
few tools everyone gets (the user asked for them, 2026-10-26): `fastfetch`,
`nano` and `vim` (Ubuntu installs only `vim-tiny`), `btop` and `htop`,
`tree`, `ripgrep` (`rg`), `fzf`, `bat` (Ubuntu names its command `batcat`),
`ncdu` and `7zip` (`7z`). Since 0.2.5 it recommends KDE's Breeze cursors
(`breeze-cursor-theme`, LGPL-3+), Breeze Dark and Breeze Light, so the Mouse
pointer choice (Patrone Settings, and GNOME Settings' Patrone page) had a
white pointer: Patrone's, Adwaita and Yaru were all dark, and the other
cursor themes Ubuntu's desktop brings (DMZ, X.Org's) can't be offered
(`settings/README.md`, "Mouse pointer"; `docs/COMPLIANCE.md`, item 22).
Since `patrone-branding` 0.1.25, Patrone White is one too. Its app store is GNOME Software, with Patrone's
picks (`patrone-software`, `software/README.md`), in place of Canonical's App
Center.

| File | Installed as | What it does |
| --- | --- | --- |
| `gnome-initial-setup/vendor.conf` | `/etc/gnome-initial-setup/vendor.conf` | First-login setup skips Ubuntu's "Help improve Ubuntu" page (`ubuntu_insights`), which asks to send system and error reports to Canonical, and its Ubuntu Pro page (`ubuntu_pro`) |
| `chrony/patrone-nts.sources` | `/etc/chrony/sources.d/` | The clock's servers: Cloudflare (`time.cloudflare.com`), Netnod (`nts.netnod.se`), and PTB (`ptbtime1.ptb.de`), each authenticated with NTS, in place of Ubuntu's `ntp.ubuntu.com` pools. Not TimeNL (SIDN Labs), whose terms ask vendors to ask first (`docs/COMPLIANCE.md`, item 7) |
| `chrony/patrone-nts.conf` | `/etc/chrony/conf.d/` | `nocerttimecheck 1`: a computer whose clock starts far off can still use NTS (below) |
| `networkmanager/30-patrone-connectivity.conf` | `/usr/lib/NetworkManager/conf.d/` | NetworkManager's check for Wi-Fi sign-in pages asks GNOME's server (`nmcheck.gnome.org`) in place of `connectivity-check.ubuntu.com`. It comes after Ubuntu's `20-connectivity-ubuntu.conf`, so it wins |
| `update-manager/patrone.cfg` | `/etc/update-manager/release-upgrades.d/` | `Prompt=never`: nothing offers to upgrade to Ubuntu's next release, or asks `changelogs.ubuntu.com` whether there is one. Patrone moves to a new Ubuntu base itself |
| `apt/patrone.sources` | `/etc/apt/sources.list.d/` | Patrone's own updates: its apt repository on GitHub Pages, `https://billsecond.github.io/patrone-apt` (below) |
| `apt/patrone-archive-keyring.gpg` | `/usr/share/keyrings/` | The key that repository is signed with, and the only one apt takes for it (`Signed-By`) |
| `apt/patrone.pref` | `/etc/apt/preferences.d/` | Patrone's builds come before Ubuntu's, even Ubuntu's newer versions (below) |
| `apt/51patrone-release-label` | `/etc/apt/apt.conf.d/` | Accepts a change of a repository's Label (`Acquire::AllowReleaseInfoChange::Label`): Patrone's repository's Label is to take the system's new name, and apt otherwise stops updating from it, in apt and in Patrone Software, until someone runs `apt-get update --allow-releaseinfo-change`. Origin, Suite and Codename changes are still refused (0.2.6) |
| `apt/52patrone-unattended-upgrades` | `/etc/apt/apt.conf.d/` | The daily automatic updates install from Ubuntu's archive (whose origins Ubuntu's own file names by the system's ID, now "Patrone") and from Patrone's repository |
| `dconf/50-patrone-task-manager-shortcut` | `/etc/dconf/db/patrone.d/` | Ctrl+Shift+Esc opens Patrone Task Manager: a custom shortcut, compiled into Patrone's dconf database by postinst (`dconf update`), which each account can change or remove in Settings. From `patrone-task-manager` before 0.2.5, which installs on its own now (`task-manager/README.md`, "Ctrl+Shift+Esc") |
| `dconf/profile-user` | `/usr/share/dconf/profile/user` | The vendor dconf profile that reads that database after each account's own settings (an administrator's `/etc/dconf/profile/user` takes its place) |
| `connections.txt` | `/usr/share/doc/patrone-desktop/connections.txt` | What the system connects to, and who runs each service, for users (`/etc/legal` points to it): the website's list (`/privacy/patroneos`), kept to what this version does. Change both together |
| `session/make-session-mode` | `/usr/libexec/patrone-desktop/` | Writes `/usr/share/gnome-shell/modes/ubuntu.json`: Ubuntu's session mode (moved aside by preinst to `/usr/share/patrone-desktop/ubuntu.json.ubuntu`, out of the modes folder: before 0.2.6 it sat beside Patrone's as `ubuntu.json.ubuntu`, which GNOME Shell also takes for the "ubuntu" mode, and when it found that first the taskbar loaded only if the account's own list named it) with Patrone's taskbar among the session's own extensions (below). Run by postinst, and again by a trigger whenever `gnome-shell-common` brings a new one; if Ubuntu's file changes shape, or the result isn't valid JSON, Ubuntu's is used as it is |

### Ubuntu's desktop, listed here

Ubuntu's desktop is two metapackages, `ubuntu-desktop-minimal` and
`ubuntu-minimal`, and they need what Patrone leaves out: Ubuntu Pro's client
(`ubuntu-pro-client`, with its services and its apt hook), Software Updater
(`update-manager`, which showed Ubuntu's logo, Ubuntu Pro's rows and a
Livepatch tip) and, through it, `update-notifier` and
`ubuntu-release-upgrader-gtk`. So since 0.2.0, `patrone-desktop` takes their
place (docs/COMPLIANCE.md, items 3 and 12; docs/NEXT.md §16): its Depends
and Recommends are theirs (1.570.4), less those, `hwctl` (Canonical's
hardware-certification tool), Ubuntu's Help (`ubuntu-docs`, whose place
`patrone-branding` takes), `network-manager-config-connectivity-ubuntu`
(Ubuntu's server for the network check, which Patrone's asks GNOME's), the
reporting packages (below), Firefox and Ptyxis; and it recommends
`gnome-user-docs`, GNOME's Help, which `ubuntu-docs` used to bring.
`Conflicts` on `ubuntu-pro-client`, `update-manager` and `hwctl` takes them,
and Ubuntu's metapackages with them, off a system that has them. Updates
come from Patrone Software, and Settings falls back to it.

When Ubuntu's metapackages change (a new Ubuntu base), compare their lists
with these: `apt-cache show ubuntu-desktop-minimal ubuntu-minimal`.

### Updates

Patrone's packages come from its own apt repository,
[billsecond/patrone-apt](https://github.com/billsecond/patrone-apt), on
GitHub Pages: Patrone runs no servers. Since 0.2.1, `patrone-desktop`
installs its source and key, so every install gets Patrone's updates the
way it gets Ubuntu's: Patrone Software shows them, and the daily automatic
updates install them (`52patrone-unattended-upgrades` allows the `Patrone`
origin). `packaging/publish.sh` publishes (`packaging/README.md`,
"Publishing").

Patrone builds five of Ubuntu's packages with its own changes: GDM
(`gdm/`), GNOME Settings (`gnome-settings/`), GNOME Software
(`gnome-software/`), first-login setup (`gnome-initial-setup/`) and Language
Support (`language-selector/`). Their versions are Ubuntu's plus `patroneN`, so
Ubuntu's next update of one is newer and would replace Patrone's build.
`patrone.pref` stops that: Patrone's repository has priority 600, over
Ubuntu's 500, so its builds win whatever the versions; and Ubuntu's versions
of those packages have 90, under the 100 of what's installed, so they don't
replace Patrone's build even before apt has read Patrone's repository. The
dev VM builds each of Ubuntu's updates to them again, with Patrone's
changes, and publishes it (`packaging/ubuntu-updates.sh`, daily); until it
does, installs keep Patrone's previous build, so Ubuntu's fixes to those
packages arrive through Patrone, a day or so later. A daily check in the
repository opens an issue whenever one is behind.

Nothing offers Ubuntu's next release (`update-manager/patrone.cfg`,
`Prompt=never`, which `do-release-upgrade` and the login message read):
Patrone moves to a new Ubuntu base itself (`docs/NEXT.md`, "Updates").

### The taskbar is part of the session

For the first minute after GNOME Shell starts, a crash makes GNOME turn
`disable-user-extensions` on (`extensionSystem.js`: the
`gnome-shell-disable-extensions` file in the runtime folder), which turns off
every extension but the session mode's own. Patrone's taskbar was one of the
others, so after the crashes of 2026-10-27 GNOME's top bar came back, and
stayed (the user: "I should never see that bar"). As one of the session's
own extensions (0.1.5) it always loads. The other side of that: a taskbar
that crashed GNOME Shell at sign-in would keep doing it, with no safety
switch. `disabled-extensions` still turns it off, over SSH if need be:
`gsettings set org.gnome.shell disabled-extensions "['patrone-taskbar@patroneos.com']"`
in the account's session.

Ubuntu's GNOME Shell doesn't load a session-mode extension from an account's
own folder, so a copy installed there (to try unreleased changes) is ignored:
`shell/gnome/tests/headless.sh` mounts Ubuntu's own mode file in its
throwaway shell so that its copy runs.

### The clock

Ubuntu's chrony keeps its pools in `ubuntu-ntp-pools.sources`, which no
package owns: chrony's postinst writes it with ucf when its debconf question
`chrony/configure_ubuntu_pools_in_sourcesd` is yes (the default), and removes
it when the answer is no. `patrone-desktop`'s postinst answers no and removes
the file the same way; its postrm answers yes and puts the file back.

NTS checks each server's certificate against the clock, so a computer whose
clock starts far off (a dead clock battery) can't set it. Ubuntu gets around
that with `ntp-bootstrap.ubuntu.com`, whose certificate is valid for decades.
Patrone has no such server, so `nocerttimecheck 1` skips the certificates'
dates until the clock is first set. chrony's manual calls that a last resort:
for that first update, an attacker holding an expired certificate's key for
one of these servers could feed it a wrong time.

## What reports to Canonical, and what happens to it

| Package | What it does | Patrone |
| --- | --- | --- |
| `whoopsie` | Sends crash reports to errors.ubuntu.com | Removed (`Conflicts`) |
| `ubuntu-insights` | Collects and uploads system reports, from a weekly timer | Removed |
| `ubuntu-report` | Sends a system report after a release upgrade | Removed |
| `apport-gtk` | The crash dialog whose Send button hands reports to whoopsie | Removed |
| `apport` | Writes crash reports to `/var/crash` | Kept, and recommended so that removing `apport-gtk` doesn't leave it to `apt autoremove`: the reports stay on the machine for debugging, and nothing sends them |
| `whoopsie-preferences`, `libinsights0` | Record the choices in Settings and first-login setup | Kept: Settings and first-login setup depend on them. They write a setting and never upload anything |

All four removed packages were only recommended by Ubuntu's desktop, so
`patrone-desktop`'s list leaves them out.

Settings → Privacy & Security → Telemetry still shows "Send error reports to
Canonical" and the system-report switches: Ubuntu builds that page into
Settings itself, and nothing on the system hides it at run time. With the
programs that send reports gone, the switches do nothing. Hiding the page
takes a Patrone build of Settings.

## What still reaches Canonical's servers

None of these sends reports about the machine. They fetch what the system
needs:

- **Updates:** Ubuntu's archive (`archive.ubuntu.com`, `security.ubuntu.com`).
- **Snaps:** the Snap Store, which keeps the system's snaps (such as the
  firmware updater) up to date.
- **Flatpak apps:** Flathub, from Patrone Software, which keeps the web
  browser up to date (PatroneOS comes with none: Patrone Browser
  Chooser asks which to install).

Ubuntu Pro's client isn't installed (above): nothing fetches its news or
checks for its extended updates.

Ubuntu's daily news for the login message (`motd-news`) is already off:
its script runs only when `/etc/default/motd-news` sets `ENABLED=1`, and that
file isn't there.
