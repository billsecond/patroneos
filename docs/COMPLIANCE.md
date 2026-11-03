# Compliance review

A review of PatroneOS against Canonical's Ubuntu policies and everyone
else's licences, trademarks and terms, and the privacy laws that touch the
website (the user, 2026-10-28: "makes sure this is complient with ubuntu
and everyone else, be very through and make sure you find ways to double
check"). Read on 2026-10-28, against the ISO built on 2026-10-27
(in `/var/tmp` on the dev VM), the repository
at `e961ef4` to `77ce19f`, the website at `77ce19f`, and
[billsecond/patrone-app-index](https://github.com/billsecond/patrone-app-index).

**Brought up to date on 2026-10-29** with what was merged that day:
this repository's #19 to #29 and the website's #7 and #8. The user asked: "if
there is anything we need to do to be more complaint and less risky for any
of the microsoft or conicial things do it now". Each item that changed says
so under **2026-10-29**.

**The system is PatroneOS since 2026-10-29** (the user: "The website will
be https://patroneos.com"). This review was written under the system's old
name, which had "Linux" in it; it now uses the new name throughout, quotes
included. Items 4 and 9 are brought up to date for the new name. The repository is
<https://github.com/billsecond/patroneos> since then (GitHub redirects its
old address).

**This is not legal advice.** Each finding is graded:

- **Confirmed problem**: a duty that isn't met, shown by the files.
- **Likely problem**: probably not met, or met only on a generous reading.
- **Needs a lawyer**: the answer turns on law or a contract that only a
  lawyer can settle; the facts are given so one can.
- **Fine**: checked, and nothing to do (or only good practice).

Nothing was published, uploaded, or sent to anyone outside these sessions.

## How it was checked

1. **An inventory first.** The ISO was mounted read-only on the dev VM, with
   its layers stacked as the installer stacks them (the installed system:
   `minimal.patrone` over Ubuntu's `minimal`; the live session: Patrone's
   live layer over Ubuntu's live, standard and minimal layers). `dpkg-query
   --admindir` on each gave the package lists; every package's archive
   component came from the VM's apt indexes (`archive.ubuntu.com`'s
   `Packages` files), not from its `Section` field, which keeps Debian's
   values. The snap seeds, the disc's own package pool (`pool/main`,
   `pool/restricted`) and Patrone's built `.deb`s were listed too. The
   [inventory](#inventory) is at the end.
2. **Primary sources.** Each package's own `/usr/share/doc/*/copyright`,
   upstream licence files, the GPL texts in `/usr/share/common-licenses`,
   and the owners' policy pages, quoted with their URLs (all read
   2026-10-28).
3. **Every finding two ways**: a second source (an archived copy, the
   owner's FAQ, how other distributions do it) and a tool or file check
   (`cargo deny` 0.20.2 with an empty allow-list, `cargo metadata`, a
   strict DEP-5 parse with python-debian, lintian, byte comparisons with
   Ubuntu's own ISO, image metadata, `gh` against the public repositories).
4. **Independent re-checks.** Five research agents covered Canonical, the
   licences, trademarks, privacy and the app index. Then three more agents,
   given only the questions and where to look, not anyone's conclusions,
   answered the important ones again from scratch. The last column of the
   summary says which findings they confirmed.

## Summary, most serious first

| # | Finding | Grade | Who acts | Re-checked |
| --- | --- | --- | --- | --- |
| [1](#1-no-source-code-is-offered-for-anything-on-the-iso) | No source code is offered for the ISO's GPL software, and Patrone's own changed builds have no public source at all | Confirmed problem, before any public download; **fixed for 26.04.1** (2026-10-28: the repository is public, and the disc's source is on the website beside it, gathered by `iso/build-source.sh`). Since 2026-10-29 the disc lists only what's on it. The snaps' own source can't all be gathered from the disc | Each new disc: the same again; a lawyer (the snaps) | Yes: confirmed |
| [2](#2-canonicals-rule-on-redistributing-ubuntus-binaries) | Canonical's policy asks derived distributions to recompile Ubuntu's binaries; Patrone ships Ubuntu's own | Needs a lawyer; a letter to Canonical is drafted (2026-10-29), not sent | The user (a lawyer first) | Yes: same count, needs a lawyer |
| [3](#3-ubuntus-logo-is-still-on-screen) | Ubuntu's logo is still on screen: Help, Software Updater, the logo icons | Confirmed problem, **fixed** (on screen 2026-10-28; on the disc 2026-10-29: none of Ubuntu's layers left, #29) | | Yes: confirmed |
| [4](#4-linux-in-the-name-needs-the-linux-foundations-sublicence) | "Linux" in the name needs the Linux Foundation's (free) sublicence | Confirmed problem, before a public release; applied for (2026-10-28); **no longer needed** since the name is PatroneOS (2026-10-29), as long as "Linux" stays descriptive and attributed | The user (withdraw or let lapse) | Yes: confirmed |
| [5](#5-the-website-keeps-full-ip-addresses-for-a-year-and-says-it-keeps-none) | The download log keeps full IP addresses for a year, "deleted" data stays in the file, and the privacy page contradicts the code | Confirmed problem | Website session | Yes: confirmed, by a test |
| [6](#6-the-app-index-republishes-artcrafts-trademarked-icon) | The app index republishes ArtCraft's trademarked, non-open-source "A" icon | Likely problem, needs a lawyer; **taken off the index** (2026-10-28) until the ArtCraft Team allows it | The user (ask them) | Yes: likely problem |
| [7](#7-timenl-asks-vendors-to-ask-first) | TimeNL's terms ask vendors to contact them before shipping its server as a default | Likely problem, **fixed** (2026-10-28: TimeNL dropped) | | Yes: likely problem |
| [8](#8-the-rust-apps-shipped-none-of-their-crates-notices) | The Rust apps shipped none of the MIT/Apache notices of the code compiled into them | Confirmed problem, **fixed** (`5b9356e`, `460713b`) | Rebuild each package | Yes: confirmed; it found the standard library gap, fixed in `460713b` |
| [9](#9-the-wallpaper-and-logo-are-ai-generated-but-labelled-gpl-3-by-william-daugherty) | The wallpaper (and almost certainly the logo) is AI-generated, but the copyright file claims it as the owner's GPL-3+ work | Confirmed problem (the label), **fixed** (2026-10-28: labelled as AI-made); needs a lawyer (ownership, and a clearance search for PATRONEOS, 2026-10-29) | The user (a lawyer, a vector master) | Yes: confirmed, image hash checked |
| [10](#10-the-installer-still-carries-canonicals-and-other-peoples-pieces) | The installer: "Automated with Landscape", 27 other companies' app logos and Windows' logo, fonts and data without their licences (and no telemetry page after all) | Confirmed (notices); likely problem (the rest): **fixed** (2026-10-28) | | Yes: confirmed; it found the extra fonts, data and Windows logo |
| [11](#11-thunderbird-in-the-live-session) | Thunderbird, Canonical's build of Mozilla's app, is in the live session | Likely problem, **fixed** (2026-10-28; the hidden layer too, 2026-10-29, #29) | | Yes: likely problem |
| [12](#12-ubuntu-pro-landscape-and-other-canonical-services) | Ubuntu Pro's client and services, hwctl, and other Canonical services | Likely problem (Pro, hwctl), **fixed** (2026-10-28); fine (the rest) | | No (research, checked on the VM) |
| [13](#13-patrones-rebuilds-of-ubuntu-packages-say-ubuntu-maintains-them) | Patrone's rebuilds of GDM, Settings and first-login setup say Ubuntu maintains them, and their copyright files don't mention Patrone | Likely problem, **fixed** (Settings `67f052b`; GDM and first-login setup 2026-10-28) | | Yes: likely problem |
| [14](#14-the-website-ubuntu-in-a-url-and-incomplete-trademark-notices) | The website: Ubuntu in a URL, incomplete trademark notices, no source link, a licence click-through to watch | Likely problem, **fixed in the code** (`fe30ef4`; Microsoft's notices and non-affiliation, website #7, 2026-10-29) | Website session | Yes: URL confirmed |
| [15](#15-windows-11s-look-and-feel) | Windows 11's look and feel, the "Start" name, the File Manager icon, "Windows-style" descriptions | Needs a lawyer (low risk); **the names, the Royal theme's name, the File Manager icon and the descriptions changed** (2026-10-29, #19 to #27, website #7 and #8); the look stays | The user | No (research, two sources each) |
| [16](#16-the-zfs-kernel-module) | ZFS's kernel module (CDDL) is installed with the GPL kernel | Needs a lawyer | The user | Yes: needs a lawyer |
| [17](#17-nvidia-and-broadcom-drivers-on-the-disc) | NVIDIA's and Broadcom's drivers are on the disc | Fine to redistribute; their indemnities need a lawyer | The user | Yes: byte-identical; needs a lawyer |
| [18](#18-smaller-notices-and-wording) | Smaller notices and wording (`/etc/legal` **fixed**, `patrone-branding`'s and `patrone-installer`'s copyright files **fixed**, Ubuntu's name in setup, Settings, Language Support and the boot menu **fixed**, #28) | Mixed | Several | In part (the copyright files, `/etc/legal`) |
| [19](#19-privacy-law-for-the-website) | Privacy law for the website: GDPR applicability, an EU/UK representative, CAN-SPAM | Needs a lawyer (applicability); cheap steps either way | The user; website session | Yes: needs a lawyer |
| [20](#20-what-the-os-contacts-and-whether-users-are-told) | What the OS contacts, and whether users are told | Fine (operators' terms); likely problem (no notice in the OS), **fixed** (2026-10-28) | Website session (its list, brought up to date) | In part (TimeNL) |
| [21](#21-the-app-index-otherwise) | The app index otherwise: licences, Flathub, GitHub's terms | Fine, with small fixes | The user | In part (ArtCraft) |
| [23](#23-the-websites-notices-and-the-javascript-notice) | The website's notices: no cookie banner needed; how the JavaScript notice, the point-of-collection notice and the privacy policy must work | Fine without a banner; requirements sent | Website session | No (research, sources quoted) |
| [24](#24-the-licence-agreement) | The licence agreement for the download and the installer, `legal/eula.md` 1.0, Texas law | **Written**; needs a lawyer before it's relied on | Website session; main session | Yes: independent review, its must-fixes applied |
| [22](#22-checked-and-fine) | Checked and fine: "based on Ubuntu", Ubuntu's archive, snaps, Secure Boot, fonts, crate licences and more | Fine | | No (research, two sources each) |

## 1. No source code is offered for anything on the ISO

**Confirmed problem, before any public download.**

The ISO holds 1,064 source packages' worth of binaries (1,050 Ubuntu's, 14
Patrone's), 155 `.deb`s in the disc's own pool and 15 snaps. Most are under
the GPL or LGPL. Distributing them obliges Patrone, as the distributor, to
give recipients the source. Nothing on the disc, in the installed system or
on the website's download page says where it is, and Patrone's part of it
isn't public.

**What Patrone changed, whose source only the private repository has:**

| Binary | Licence | Built by |
| --- | --- | --- |
| `gdm3`, `libgdm1`, `gir1.2-gdm-1.0` 50.1-0ubuntu0.1patrone1 | GPL-2+ | `gdm/build.sh` |
| `gnome-control-center`(`-data`, `-faces`) 1:50.3-0ubuntu0.2patrone9 | GPL-2+ | `gnome-settings/build.sh` |
| `gnome-initial-setup` 50.0-0ubuntu7.1patrone2 | GPL-2+ | `gnome-initial-setup/build.sh` |
| `patrone-installer` 0.1.2: Canonical's installer front end, changed | GPL-3, with `ubuntu_flavor` (MPL-2.0) changed | `installer/build.sh` |
| `patrone-branding`'s login theme (Yaru's, extended) | GPL-3+ | `branding/tools/make_gdm_theme.sh` |

Each `build.sh` runs `dpkg-buildpackage -b`, a binary-only build, so no
source package is ever made, and every Patrone copyright file's `Source:`
(and `os-release`'s `HOME_URL`) was this repository, which was private
then (`gh repo view`: `"visibility":"PRIVATE"`).

**What the licences say** (`/usr/share/common-licenses/GPL-2` and `GPL-3`):

- GPL-2 §3 allows three ways: (a) accompany the binaries with the source;
  (b) a written offer, "valid for at least three years, to give any third
  party ... a complete machine-readable copy"; (c) pass on an offer you
  received, "allowed only for noncommercial distribution and only if you
  received the program in object code or executable form with such an
  offer". And: "If distribution of executable or object code is made by
  offering access to copy from a designated place, then offering
  equivalent access to copy the source code from the same place counts as
  distribution of the source code."
- GPL-3 §6(d) lets the source sit on "a different server (operated by you
  or a third party) ... provided you maintain clear directions next to the
  object code saying where to find the Corresponding Source. Regardless of
  what server hosts the Corresponding Source, you remain obligated to
  ensure that it is available for as long as needed".
- MPL-2.0 §3.2 (for the changed `ubuntu_flavor`): the source must be made
  available, and recipients told "how they can obtain a copy".

**Why pointing at Ubuntu isn't enough.** Ubuntu ships no written offer, so
GPL-2 §3(c) isn't available, and GPL-2 has no third-party-server clause
like GPL-3's. Ubuntu's mirrors also drop superseded versions: the ISO's
`bind9` binaries are `1:9.20.24-1ubuntu0.2`, whose source is gone from
`archive.ubuntu.com/ubuntu/pool/main/b/bind9/` (it has `0ubuntu0.4`),
though Launchpad still has
<https://launchpad.net/ubuntu/+source/bind9/1:9.20.24-1ubuntu0.2> (HTTP
200). 127 of the installed system's exact versions are already missing from
the archive's indexes.

**Fix** (needs the user's decision on where to host; the rest is the main
session's and the website session's):

1. Make Patrone's own sources public at the ISO's commit: the repository,
   or a source tarball of each package and of `installer/`, `gdm/`,
   `gnome-settings/`, `gnome-initial-setup/` and `iso/`.
2. Build source packages of the seven rebuilt binaries
   (`dpkg-buildpackage` without `-b` gives the `.dsc`, `.debian.tar.xz` and
   `.orig` tarball), and keep the installer's patched checkout (Canonical's
   source at `6623407` after `patch_source.py` and `debrand_l10n.py`, the
   changed `ubuntu_flavor`, `installer/build.sh`).
3. For Ubuntu's packages: a source mirror of exactly the manifest's
   versions on the same host as the ISO, the safe reading of GPL-2 §3. An
   `iso/build-source.sh` can read the layers' manifests (and the snaps'
   `snap/manifest.yaml`, which lists the debs each was built from) and fetch
   each source package from Launchpad. About **6.3 GB** for the installed
   system and live session (summed from Ubuntu's `Sources` indexes), plus
   the pool and the snaps. A written offer (GPL-2 §3(b)) is the
   alternative. Relying on Launchpad alone, as many derivatives do, is a
   lawyer's question.
4. "Clear directions next to the object code": a line on the download page,
   in `/etc/legal`, and a `README` on the disc, saying where the source is.

The snaps count too: 13 seeded in the installed system and 17 in the live
session, among them a kernel snap with `zfs.ko`, the installer's snap with
subiquity and curtin (AGPL-3), and the `cups` snap with Ghostscript
(AGPL-3); Patrone added `chromium`, `cups` and `core26` itself. So do
Ubuntu's hidden layers ([3](#3-ubuntus-logo-is-still-on-screen)). How far
the duty reaches into Canonical's snaps is a lawyer's question; the snaps'
manifests make the list.

The build trees don't survive: `gdm/build.sh`'s `patrone1` tree is gone
and `gnome-settings`' now holds `patrone10`, so the source of the ISO's
`patrone9` could only be rebuilt from git history. Keep each shipped
version's source package.

The user's own GPL-3+ code isn't bound by this (the copyright holder can't
breach their own licence), but it's in the same packages as others' code,
and the licence promises recipients the source, so publish it too.

**In hand** (2026-10-28; the user: "You can open up the repo to be
public"):

1. Patrone's own sources are public: the repository is
   <https://github.com/billsecond/patroneos> (its history rewritten
   without the website, which has its own private repository).
2. The rebuilds' source is Ubuntu's source package of the version each is
   built from, plus the repository's build script and patches at the disc's
   commit (`gdm/`, `gnome-settings/`, `gnome-initial-setup/`), which
   `iso/build-source.sh`'s README maps package by package; the installer's
   is Canonical's `ubuntu-desktop-provision` at its commit, `ubuntu_flavor`,
   and `installer/`.
3. The source mirror: `iso/build-iso.sh` now writes `casper/sources.manifest`
   onto the disc (every source package and version on it: Patrone's layers,
   Ubuntu's three hidden beneath them, the pool, and the seeds' snaps), and
   `iso/build-source.sh` fetches all of it from Launchpad, with Patrone's
   repository at the disc's commit and the installer's upstream source,
   into one folder with a `README.txt` and `SHA256SUMS` (`iso/README.md`).
   Tried on a small test disc: it fetched a superseded version
   (`bind9 1:9.20.24-1ubuntu0.2`) from Launchpad, mapped `gdm3`'s
   `patrone2` to Ubuntu's `50.1-0ubuntu0.1`, skipped Patrone's own packages,
   and a second run kept what it had. The two list functions, run on an
   earlier build's layer and the pool, gave 958 source packages. The full
   run waits for the next ISO.
4. Directions: `SOURCE.txt` at the disc's root and `/etc/legal`
   (`patrone-branding` 0.1.21) say `https://patroneos.com/source`, and
   the licence agreement's section 1 says the same; the website serves the
   folder there and links it from the download page (the website session).

**Done for 26.04.1** (2026-10-28): the disc built from `d0c19ce` (sha256
`1137635e6f47441b...`) is on the download page with its source folder,
the website's `/source/` folder for 26.04.1 (1,093 of Ubuntu's source packages,
Patrone's repository at `d0c19ce` and the installer's upstream source: 8.3
GB, 3,325 files), and the download page links it and the repository.
Checked on the running site: the folder answers with its README, its
`SHA256SUMS` lists every file and passes `sha256sum -c` on the server, and
a file resumes (206). Still open: the snaps, listed but not fetched (a
lawyer's question, above); and doing the same for every later disc.

**2026-10-29** (#29): the disc's layers are Patrone's own (item 3), so its
`sources.manifest` lists only what's on it: 1,098 source packages and 10
snaps, down from 1,112 and 18. Firefox's, Thunderbird's, App Center's,
Security Center's, hwctl's and the `pc` and `pc-kernel` snaps (a second
kernel) are off the disc, and their source with them; two of `pc-kernel`'s
components stayed until #30. Checked on a test disc
built from that branch; the published disc is still the `d0c19ce` one, until
the next is built and its source gathered. **The snaps that stay** (bare,
core24, firmware-updater, gnome-46-2404, gtk-common-themes, mesa-2404,
prompting-client, snapd, snapd-desktop-integration and the installer's
ubuntu-desktop-bootstrap): most of their manifests name their own code only
as `source: .`, with no commit, so their complete source can't be gathered
from the disc. The Ubuntu packages they're built from can be, from their
`primed-stage-packages` and, in the base snaps, `usr/share/snappy/dpkg.yaml`.
Still a lawyer's question.

## 2. Canonical's rule on redistributing Ubuntu's binaries

**Needs a lawyer.**

Canonical's [Intellectual Property Rights Policy](https://canonical.com/legal/intellectual-property-policy)
(15 July 2015; `ubuntu.com/legal/intellectual-property-policy` redirects
there; Wayback copies of 2025-08-20 and 2026-10-23 are word for word the
same):

- "Any redistribution of modified versions of Ubuntu must be approved,
  certified or provided by Canonical if you are going to associate it with
  the Trademarks. Otherwise you must remove and replace the Trademarks and
  will need to recompile the source code to create your own binaries. This
  does not affect your rights under any open source licence applicable to
  any of the components of Ubuntu."
- "The disk, CD, installer and system images, together with Ubuntu packages
  and binary files, are in many cases copyright of Canonical (which
  copyright may be distinct from the copyright in the individual components
  therein) and can only be used in accordance with the copyright licences
  therein and this IPRights Policy."
- §2: "where any other licence grants rights, this policy does not modify
  or reduce those rights under those licences."

Patrone ships 1,520 of its 1,540 installed packages as Ubuntu built them,
plus the disc's 155 `.deb`s, shim, GRUB and the kernel (byte-identical to
Ubuntu 26.04.1's ISO) and Canonical's snaps. It removes the trademarks (item
3 is what's left) but doesn't recompile.

How others read it: the Software Freedom Conservancy
([2015](https://sfconservancy.org/news/2015/jul/15/ubuntu-ip-policy/)) says
§2's "trump clause" settles it for GPL software, but for permissively
licensed parts "full permission to redistribute Ubuntu as a whole remains
in question". In 2014 Canonical asked Linux Mint to take a licence
([LWN](https://lwn.net/Articles/586446/)); KDE's Jonathan Riddell argued
none is needed. Linux Mint and Zorin OS ship Ubuntu's binaries.

**The user's decision**: take advice, and decide whether to ask Canonical
(legal@canonical.com) for written confirmation. Rebuilding every package
is the only way to avoid the question.

**2026-10-29**: a draft letter to Canonical, asking whether this is
acceptable or what Canonical needs, is ready for a lawyer's reading. It's
kept out of this repository and hasn't been sent.

## 3. Ubuntu's logo is still on screen

**Confirmed problem.** The policy: modified versions "must remove and
replace the Trademarks"; Matt Zimmerman, then Ubuntu's CTO, [on
ubuntu-devel](https://lists.ubuntu.com/archives/ubuntu-devel/2007-January/023143.html):
"Use of the logo to refer to a derivative distribution requires prior
permission."

- **Help** is "Ubuntu Desktop Guide" with Ubuntu's logo in its title
  (`ubuntu-docs`: `gnome-help/index.page:17` puts `figures/ubuntu-logo.png`
  there, in 38 languages). docs/NEXT.md §16 item 4.
- **Software Updater** (`update-manager`, in the Start menu) shows "Ubuntu
  base" with `distributor-logo`, "Enable Ubuntu Pro..." and a Livepatch
  tip (`UpdateList.py:186`, `UpdatesAvailable.py:257`, `:1190`). Item 2.
- **Yaru's `distributor-logo` and `start-here` icons** are Ubuntu's logo
  (looked at; Yaru-blue inherits them), so any app asking for them shows
  it. Item 9. Likely problem.
- Not on screen, but installed: `base-files`' `ubuntu-logo*` pixmaps, the
  inactive spinner theme's Ubuntu wordmark, the `ubuntu-text` boot theme
  ("Ubuntu 0.1"), and `ubuntu-logo-text-dark.png`, which Plymouth copies
  into every initrd. Fine, but tidy them when it's cheap.
- **Ubuntu's own layers are still on the disc**, under Patrone's
  (`minimal.squashfs`, `minimal.standard.squashfs`,
  `minimal.standard.live.squashfs`, unchanged). Patrone's layers hide what
  they remove, but the files are still distributed: Ubuntu's base layer
  has `firefox_8763.snap`, `snap-store_1390.snap` and
  `desktop-security-center_151.snap` in its seed and Ubuntu's logo
  pixmaps, the standard layer `thunderbird_1228.snap`, the live layer
  `firefox_8803.snap`. Likely problem: the disc as a whole still carries
  Ubuntu's branding and Canonical's apps under Patrone's name, and their
  source is owed too. **Fix** (main session): self-contained layers
  (Ubuntu's base unpacked, Patrone's changes applied, packed again) so
  nothing hidden ships; it costs build time and disc space.

**Fix** (main session): the Patrone desktop metapackage in place of
`ubuntu-desktop-minimal` (§16 item 1) drops Software Updater; a Patrone
docs package that provides `ubuntu-docs`; a Patrone icon theme over Yaru's
for the two logo icons.

**Fixed on screen** (2026-10-28):

- Software Updater is gone: `patrone-desktop` 0.2.0 lists Ubuntu's desktop
  packages itself, in place of `ubuntu-desktop-minimal` and
  `ubuntu-minimal`, less `update-manager` (and Ubuntu Pro's client, item
  12), and conflicts with it (`desktop/README.md`).
- Help is GNOME Help: `patrone-branding` 0.1.19 replaces `ubuntu-docs`
  (Provides, Conflicts, Replaces, as for the wallpapers) with GNOME's own
  front page and the seven other pages Ubuntu moves out of
  `gnome-user-docs`, unchanged from GNOME's release, in GNOME's 45
  languages (CC BY-SA 3.0, its text in the copyright file). No Ubuntu
  pictures are left in `/usr/share/help`.
- The logo icons are Patrone's P: not an icon theme of Patrone's after all
  (Settings switches between Yaru's themes by name, which would drop it),
  but Yaru's own `start-here` files, which its `distributor-logo` icons
  link to, moved aside (`dpkg-divert`) with Patrone's in their place, in
  every Yaru color and the dark style, and both symbolic icons.
  `patrone-logo`, os-release's `LOGO`, is in `hicolor` too.

Checked on the dev VM, with both packages installed: GTK's own lookup
(GTK 4, in the session) gives Patrone's files for `distributor-logo`,
`start-here` and their symbolic versions (checksums match); Help's front
page belongs to `patrone-branding` and is GNOME's ("GNOME Help"), and every
language's parses as XML. **Seen in a fresh install** (2026-10-28, twice:
the discs from `439e7a9` and `d0c19ce`, in throwaway Hyper-V VMs): Help
opens as "GNOME Help" with no Ubuntu logo; Settings › About shows Patrone's
logo and "PatroneOS 0.1"; the logo icons are Patrone's files, diverted
from Yaru's; none of Ubuntu's removed packages or snaps (Firefox, App
Center, Security Center, Thunderbird, hwctl, Ubuntu Pro, Software Updater)
are installed, and nothing is left to autoremove but the install engine's
`grub-pc-bin`.

**Fixed on the disc** (2026-10-29, #29, `iso/build-iso.sh`): Patrone's two
layers stand on none of Ubuntu's. `patrone`, what the installer installs, is
the whole system, Ubuntu's base and English layers included; `patrone.live`,
the live session, holds only how it differs from `patrone`. All of Ubuntu's
layers leave the disc, with what Patrone's had hidden in them, and so does
Ubuntu's seed system for TPM-backed encryption, which Patrone's installer
doesn't offer. The disc is 5.47 GB, down from 7.23. **Checked**: on the dev
VM, `patrone.live` stacked on `patrone` matches, file by file, the live
session the build made from Ubuntu's three layers (`rsync --checksum`: no
differences in 271,618 files, with owners, modes, extended attributes and
hard links); and a fresh install in a throwaway Hyper-V VM booted the live
session, installed, and booted the new system. Still installed, as part of
Ubuntu's unchanged `base-files`: the `ubuntu-logo*` pixmaps (not on screen).

## 4. "Linux" in the name needs the Linux Foundation's sublicence

**Confirmed problem, before a public release.** The Linux Foundation's
[Linux Mark page](https://www.linuxfoundation.org/legal/the-linux-mark):
anyone offering a Linux-based product to the public under a mark
containing "Linux" needs its sublicence, registered or not. It's free,
perpetual and worldwide ([request form](https://www.linuxfoundation.org/legal/request-a-sublicense)).
Its guidance gives "Super Dooper Linux OS" as fine and "Super Dooper Linux"
alone as not. Linux Mint ("trademarked through the Linux Mark Institute")
and AlmaLinux hold one. Only the website's footer carries the attribution
today.

**Fix** (the user applies; nobody else can): apply for the name; follow
"Linux" in it with "OS" or "operating system" in titles and first
mentions, or ask the Foundation whether the name alone is acceptable. Everyone, licensed or not, should mark the first prominent
"Linux" on a page "Linux®" and say "Linux® is the registered trademark of
Linus Torvalds in the U.S. and other countries." (now in `/etc/legal`).
Once licensed, add the sublicence's legend (its §3.1): "The registered
trademark Linux® is used pursuant to a sublicense from the Linux
Foundation, the exclusive licensee of Linus Torvalds, owner of the mark on
a world-wide basis."

**Applied for** (2026-10-28: the user filled in the Foundation's request
form). Waiting on the Foundation. The attribution ("Linux® is the
registered trademark of Linus Torvalds in the U.S. and other countries.")
is now in `/etc/legal`, the licence agreement (section 4), the website's
footer and the repository's README. When it's granted, add the §3.1 legend
above beside each of them (`branding/system/legal`, `legal/eula.md` in a
new version, the website's footer, `README.md`), and follow whatever the
grant says about the name. **"OS" after the name until then?** Not
needed yet: nothing is offered to the public under the name while the
download is closed (it opens only with the source, item 1). If the download
is to open before the Foundation answers, follow "Linux" in the name with
"OS" or "operating system" in titles and first mentions until it does.

**2026-10-29: not needed for PatroneOS.** The name has no "Linux" in it,
and the sublicence is for marketing a product "using a trademark that
includes the element 'Linux'" (the [Linux Mark
page](https://www.linuxfoundation.org/legal/the-linux-mark)). The same page
says "In cases of fair use, members of the public may use or refer to a
registered trademark without a license from the owner", for "non-trademark
uses of a term that would normally occur in everyday communication". That
covers "based on Ubuntu, with the Linux kernel" and "Linux apps". What it
asks of everyone still applies: Linux® at the first prominent mention on a
page, and "Linux® is the registered trademark of Linus Torvalds in the U.S.
and other countries." (`/etc/legal`, the licence agreement, the website's
footer, `README.md`). So **withdraw the request, or let it lapse**: the
user's call. Nothing else is needed while "Linux" stays out of the name, the
logo's wordmark and the website's address. The old name, with "Linux" in
it, should stay off anything new.

## 5. The website keeps full IP addresses for a year, and says it keeps none

**Confirmed problem** (website session). At `77ce19f`:

- The download form stores the visitor's **full IP address** on every row
  (the website repo's `src/download.rs:229`), cleared after `DOWNLOAD_RETENTION_DAYS`,
  **365 by default** (`config.rs:156`). GDPR recital 30 names IP addresses
  as personal data; Art. 5(1)(c) and (e) ask for no more than needed, for
  no longer than needed. Counting downloads and stopping abuse need a
  shortened address, or none, and days, not a year.
- **"Deleted" data stays in the file.** Clearing runs `UPDATE ... SET ip =
  NULL` (`db.rs:220`, `:234`), but nothing sets `PRAGMA secure_delete` or
  runs `VACUUM`, and SQLite only overwrites freed content when
  [secure_delete is on](https://www.sqlite.org/pragma.html#pragma_secure_delete)
  (off by default; the bundled build passes no `SQLITE_SECURE_DELETE`).
  The re-check built the same rusqlite 0.37 offline and ran `db.rs`'s
  `forget` on 20 of 300 rows: **19 of the 20 cleared addresses were still
  readable in the file.** Each commit's journal file leaves the old pages
  in free disk blocks too, and backups keep everything.
- **The privacy page contradicts the code**: `privacy.html:33` says the
  site "doesn't log addresses", above the section describing the
  downloads database. The in-memory counts it says are "gone after an hour"
  are pruned only at 10,000 addresses (`signup.rs:205`), so an address can
  stay until a restart; the same goes for the "six hours" of form codes
  (`signup.rs:150`). "What stays is anonymous" keeps timestamps to the
  second. `privacy_os.html` says ODRS is "run by GNOME" (its [privacy page](https://odrs.gnome.org/privacy):
  hosted by Red Hat and CDN77) and BeaconDB "community-run" (run by one
  person, [beacondb.net/privacy](https://beacondb.net/privacy)).
- **Access requests by IP** (`forget` matches `ip = ?1`, and the request
  form offers "See what you have about me") would show one person's name
  and email to whoever shares their address.
- `compose.yaml` was still read-only with no data volume, and
  `README.md:80` still said the site writes nothing (check after the
  session's latest commit).

**Fix**: truncate the address when it's written (IPv4 to `a.b.c.0`, IPv6 to
the /48) or store none; keep names and emails 90 days unless "OK to contact
me" is ticked; `PRAGMA secure_delete=ON` and `VACUUM` after a purge; round
kept timestamps to the day; prune the limiter and token maps on every call;
answer access requests only by email to the address on record; correct the
page. Wording is in [19](#19-privacy-law-for-the-website).

**Since this review**, the website session's `fe30ef4` ("Shorten and expire
download records, and bring the website's notices up to the compliance
review") stores addresses only shortened (`db.rs` `shorten`), clears them
after 30 days, other details after 90 and contact details after 730
(`DOWNLOAD_ADDRESS_DAYS`, `_DETAILS_DAYS`, `_CONTACT_DAYS`), turns
`secure_delete` on and runs `VACUUM` after a purge, prunes the limiter and
token maps by age, and rewrites the privacy pages. Read in the code, not
tested; re-check the pages against it before the site goes live.

## 6. The app index republishes ArtCraft's trademarked icon

**Likely problem, needs a lawyer** (the user). The index's
`storytold/craft-launcher` entry serves `ai.storyteller.artcraft-launcher.svg`
and its PNGs; the SVG's git blob (`72e191f8...`) is the same as
`storytold/craft-launcher`'s `docs/brand/artcraft-mark.svg`. Patrone's
`org.patrone.Software.Featured.xml` puts it in Editor's Choice. The app's
own files:

- `NOTICE`: "The ArtCraft name, wordmark, logo and "A" mark (docs/brand/)
  are trademarks of the ArtCraft Team. They are NOT open source".
- `docs/brand/LICENSE-brand.txt`: the marks may be used "only: 1. as part of
  copies of this repository; and 2. within ArtCraft Launcher itself, its
  documentation and its official builds"; anything else "needs prior
  written permission from the ArtCraft Team (https://getartcraft.com/)".

The other 13 storytold apps' icons are their own MIT OR Apache-2.0 artwork.

**Fix**: ask the ArtCraft Team for written permission (hello@storyteller.ai,
from getartcraft.com: the user has to send it), and meanwhile remove
`storytold/craft-launcher` from `apps.list` and
`ai.storyteller.artcraft-launcher` from the featured file. Don't swap its
icon: a changed listing is closer to the "modified version" the brand
licence forbids.

**Done meanwhile** (2026-10-28): the launcher comes out of the app index's
`apps.list` with billsecond/patrone-app-index's pull request 1 (open: the
index drops and prunes it at the next build after it merges), and out of
Patrone Software's featured apps (`patrone-software` 0.1.5). Asking the
ArtCraft Team is still the user's, and whether listing it at all is fine
is a lawyer's question.

## 7. TimeNL asks vendors to ask first

**Likely problem, before a public release.** `patrone-desktop` sets the
clock from `ntppool1.time.nl` among others
(`desktop/chrony/patrone-nts.sources`). TimeNL's FAQ ("How can I use TimeNL
as a hardware vendor?", <https://www.time.nl/index_en.html>) says
hard-coding its servers into products marketed at scale is against its
terms of use, asks vendors to contact SIDN Labs first, and offers a
vendor-specific name. RFC 8633 §6.2 agrees: "Vendors MUST only preconfigure
NTP servers that they have permission to use", and the NTP Pool's [vendor
page](https://www.ntppool.org/en/vendors.html), which it points to, says
"You must get approval from the server operator before you hardcode any IP
addresses or hostnames." Cloudflare, Netnod and PTB publish no such rule
and offer their servers to anyone (fine; see 20), but a short note to each
settles it.

**Fix**: the user emails SIDN Labs and asks for a Patrone name (such as
`ntp.patrone.time.nl`), or the main session drops TimeNL until they answer;
three NTS servers remain.

**Fixed** (2026-10-28, `patrone-desktop` 0.2.0): TimeNL is dropped from
`patrone-nts.sources`, with the description, `connections.txt` and the
READMEs; Cloudflare's, Netnod's and PTB's NTS servers remain. The website's
list still names it (sent to the website session). Writing to SIDN Labs
would let it come back; nothing else needs it.

## 8. The Rust apps shipped none of their crates' notices

**Confirmed problem, fixed.** `patrone-terminal`, `-file-manager`,
`-settings`, `-unpack`, `-task-manager` and, since 2026-11-03, `-notepad` link their crates statically
(60 to 75 crates in each build, 36 of them compiled into Patrone
Terminal's binary, by `cargo deny` and `cargo metadata`, which agree), plus
Rust's standard library. The crates are MIT,
Apache-2.0, Unicode-3.0, Zlib and similar, and MIT, for one, says "The above
copyright notice and this permission notice shall be included in all copies
or substantial portions of the Software." The packages shipped only
`changelog.gz` and `copyright` in `/usr/share/doc`, and two copyright files
said "Each vendored Rust crate carries its own license files in its
directory", in `vendor/`, which isn't in the `.deb`. lintian reports
nothing either way: it can't see crates inside a binary.

**Fixed** in `5b9356e` and `460713b`: `packaging/third-party-licenses.sh`
copies every vendored crate's licence, copyright and notice files, and the
standard library's notices (rustup's `COPYRIGHT-library.html`, or Ubuntu's
`libstd-rust` copyright file), and `build.sh` installs the result as
`/usr/share/doc/PACKAGE/third-party-licenses.txt.gz`; each package's
copyright file points at it. Checked with a test build of
`patrone-unpack` 0.1.1 in a scratch copy on the VM: 64 crates, every one
with its own licence file, 147 standard-library entries, 49 KB compressed.
**It takes effect when each package is next built**; the ISO's packages
don't have it yet.

All the crates' licences are compatible with GPL-3.0-or-later (no
GPL-2.0-only, MPL, LGPL, OpenSSL, CC-BY-SA or missing licences; nothing LGPL
linked statically). The compositor and website aren't distributed; if the
website's binary or image ever is, it needs Unicode, ISC, BSD and CDLA
notices too.

## 9. The wallpaper and logo are AI-generated, but labelled GPL-3+ by William Daugherty

**Confirmed problem** (the copyright file, for wallpaper 01's family);
**likely problem** (the logo); **needs a lawyer** (who owns what). The
user; wallpapers session. The original wallpaper,
`branding/wallpapers/patrone-back-01.png` (removed from the tree by #35 in
2026-10-29's rebranding, and kept in git history:
`git show 9d24a57^:branding/wallpapers/patrone-back-01.png`), carries a C2PA
manifest: created 2026-10-25T15:59:59Z by "ChatGPT", `gpt-image`, `digitalSourceType`
`trainedAlgorithmicMedia`, signed by "OpenAI Media Service" (OpenAI OpCo,
LLC). The re-check recomputed the manifest's hash of the image: it
matches, so the committed file is exactly what OpenAI signed. Commit
`64936b4` added it ten minutes later. `branding/README.md` says the
generated wallpapers use "no ... image models, so Patrone owns them
outright", a distinction it doesn't make for wallpaper 01. The logo concept
(`branding/concepts/logo-concept-01.webp`, also removed by #35 and kept in
git history: `git show 9d24a57^:branding/concepts/logo-concept-01.webp`) has
no metadata (WebP conversion drops it), but its 1536×1024 canvas is one of
gpt-image's sizes, it was committed five minutes earlier, and the
wallpaper contains the same logo. Every logo, splash, About picture,
installer mark and the fastfetch logo comes from it.

`packaging/patrone-branding/debian/copyright` covers all of it with `Files:
* / Copyright: 2026 William Daugherty / License: GPL-3+`. OpenAI's terms
assign the output to the user ("Output may not be unique"), but the US
Copyright Office's [Part 2 report](https://www.copyright.gov/ai/) (January
2025) says purely AI-generated material isn't protected, and "prompts do not
alone provide sufficient control". So the GPL-3+ label may claim more than
exists, and the logo may be protectable only as a trademark. The generated
wallpapers are different: the wallpapers session confirmed that
`branding/tools/make_generated_wallpapers.py` (`69980ab`) draws every pixel
from code (seeded gradients, noise and geometry), uses no image model or
input image, and takes only the palette's hex values and the P's stem angle
from the concept, so they are Patrone's own work (fine).

**Fix**: record the provenance in `branding/README.md`; give the AI-made
art its own copyright stanza saying so, with a licence that doesn't
over-claim (CC0-1.0, or GPL-3+ "to the extent copyrightable"); consider a
human-drawn vector master of the logo (already on `branding/README.md`'s
list) and a trademark clearance search for PATRONE (classes 9 and 42). The
licence choice is the user's.

**Labelled** (2026-10-28; the user left it to this review, so the simplest
honest option, without changing the licence): `patrone-branding`'s
copyright file has a stanza of its own for the logo and wallpaper 01 and
everything made from them (the logo files, splash, slides' logo, logo
icons, fastfetch logo), saying they were made with an AI image generator
(the C2PA manifest's facts), that such material may not be protected by
copyright, and that GPL-3+ applies only as far as anyone holds copyright
in them. `patrone-installer`'s and first-login setup's copyright files say
the same of their P. `branding/README.md` records the provenance, beside
the generated wallpapers' note that code draws them. Still a lawyer's
question: who owns what, and whether to have the logo redrawn by hand.

**2026-10-29: the name is PatroneOS**, with new logos from the wallpapers
session, whose provenance `branding/README.md` and `patrone-branding`'s
copyright file should record the same way. **A first look for PATRONEOS**
(web searches, 2026-10-29; not a clearance search): nothing called
PatroneOS or Patrone OS turned up. The nearest names:

- **ParrotOS** ([Wikipedia](https://en.wikipedia.org/wiki/Parrot_OS)), a
  Debian-based Linux distribution with a large following. It's in the same
  market, and the names share their start, their end and "OS". The one to
  weigh.
- **PATRONTEK**, a US mark in class 9, computers and electronics
  ([TrademarkDir](https://trademarkdir.com/owner/patrontek)).
- **PATRONMANAGER**, a US registration for ticketing and donor software
  ([Justia](https://trademark.justia.com/775/33/patronmanager-77533763.html)).
- **PATRON** marks, for tequila and door hardware: unrelated goods.

Those came from aggregator sites, not the registries themselves. A clearance
search of the USPTO, EUIPO and WIPO's Global Brand Database in classes 9
and 42, with ParrotOS in mind, is still a lawyer's.

## 10. The installer still carries Canonical's and other people's pieces

**Likely problem** (main session, `installer/`, `branding/system/desktop-provision/`).

- **"Automated with Landscape"** on the "Type of installation" page
  (`libapp.so`). LANDSCAPE is one of Canonical's registered marks, and
  Landscape is a Canonical service (§16 item 11). Hide it.
- **The telemetry page** is still shown (`whitelabel.yaml` keeps
  `telemetry:`; only its picture changed), though the installer only writes
  `/var/log/installer/telemetry` locally and nothing uploads it: a consent
  question about nothing, the reason Settings' Telemetry page was hidden.
  `telemetry: visible: false`. **Correction** (2026-10-28, from the
  installer's source): the installer has no such page. `telemetry:` in
  `whitelabel.yaml` is for Ubuntu's first-boot setup (`ubuntu_init`, whose
  steps include it and Ubuntu Pro's), which Patrone doesn't use; Patrone's
  front end (`ubuntu_bootstrap`) has no telemetry step, and its English
  text has no telemetry message. It does write the local record above,
  which `desktop/connections.txt` now mentions. Nothing to fix.
- **Ubuntu's unused slides** are still in the bundle
  (`flutter_assets/assets/slides/`): Ubuntu's mascot and 27 other products'
  logos (Discord, Spotify, Steam, Teams, Slack, VS Code, Firefox and
  others). Patrone's slides use only its own logo. Drop them from the
  pubspec's assets in `patch_source.py`.
- **Fonts and data without their licences** (confirmed problem): the bundle
  has the Ubuntu font family (Ubuntu Font Licence 1.0, which requires "Each
  copy of the Font Software must contain the above copyright notice and
  this licence"), Yaru's icon font (CC BY-SA 4.0), Flutter's Material Icons
  (Google, CC BY 4.0 as Flutter ships it) and GeoNames' city and country
  data for the time zone map (CC BY 4.0), none of them in NOTICES.Z, and
  the copyright file said NOTICES.Z had every licence and labelled them all
  GPL-3. **Fixed** in `patrone-installer`'s copyright file (a stanza each,
  the UFL's text from Yaru's own copy); also copy yaru's
  `assets/fonts/UFL.txt` and `assets/LICENSE.CCBYSA` into the bundle in
  `installer/build.sh`.
- **Microsoft's Windows logo** (`assets/storage/windows.svg`, four blue
  squares, from Canonical's source) is shown beside Windows partitions.
  Microsoft's guidelines: don't use its logos "in any manner" without a
  licence. Likely problem (minor): a plain disk icon in `patch_source.py`.
- Canonical's pictures recolored blue are GPL-3 (ubuntu-desktop-provision's
  own licence), not Yaru's CC BY-SA, and the change is now recorded in the
  copyright file: fine.
- The installer finds the time zone through `geoip.ubuntu.com` (subiquity's
  default). Canonical publishes no terms for it; Patrone's privacy page
  discloses it. Fine; whether to keep it is the user's call (§16 item 11).

**Fixed** (2026-10-28, `patrone-installer` 0.1.4, `patrone-branding`
0.1.19; `installer/README.md`): `patch_source.py` takes the Landscape
choice off the "Type of installation" page, and `whitelabel.yaml` no longer
lists Landscape's pages, so they can't show; Windows' partitions get the
plain disk icon (`block-device.svg`); Ubuntu's slides are deleted from the
source and the pubspec, so they aren't in the app (`build.sh` also drops
the empty folders an earlier build left); `UFL.txt` and yaru's
`LICENSE.CCBYSA` are copied beside the fonts. The Applications page's one
choice no longer promises a web browser ("Patrone's desktop, apps and
everyday tools. You choose a web browser the first time you need one.",
`debrand_l10n.py`). Checked in the built app: `libapp.so` has no
"Automated with Landscape" and has the new wording; `windows.svg` is
byte-identical to `block-device.svg`; no files under `assets/slides`; both
licence files present. **Seen running** in both fresh installs (item 3):
"Type of installation" offers only the interactive and autoinstall-file
choices, the Applications page has the new wording, the slides are
Patrone's, and the licence page keeps Next off until it's ticked.

## 11. Thunderbird in the live session

**Likely problem** (main session). The live seed has `thunderbird_1228.snap`
(publisher Canonical) and the `thunderbird` stub and 21 locale packages.
Mozilla's [distribution policy](https://www.mozilla.org/en-US/foundation/trademarks/distribution-policy/)
covers unaltered copies of Mozilla's own builds; Canonical's snap is
Canonical's build, and redistributing it on Patrone's disc has no clear
cover. The live session only runs the installer. **Fix**: drop it from the
seed and purge `thunderbird*` in the live layer, as `e961ef4` did for
Firefox. (Firefox's stub is gone once the ISO is rebuilt; check `dpkg -l
firefox`.) Ubuntu's hidden layers still carry Firefox's and Thunderbird's
snaps ([3](#3-ubuntus-logo-is-still-on-screen)). Firefox's is Mozilla's own
build (publisher "mozilla", verified), which the policy allows, but on its
terms: "You may not collect personal information in the context of your
distribution of the software", and "you must distribute the most recent
version". A disc snapshot goes stale, and the website logs each download's
address (item 5). One more reason for self-contained layers.

**Fixed** (2026-10-28, `iso/build-iso.sh`): the live layer purges the
`thunderbird` stub (its language packs go with it), and both layers' seeds
drop Thunderbird's snap. Ubuntu's hidden standard layer still has
`thunderbird_1228.snap` (item 3). **2026-10-29**: no longer; Ubuntu's layers
are off the disc (#29).

## 12. Ubuntu Pro, Landscape and other Canonical services

- **Ubuntu Pro's client** (`ubuntu-pro-client`, required by `ubuntu-minimal`)
  is installed with its services enabled (`ua-timer`, "Ubuntu Pro
  Background Auto Attach"). Canonical's Pro terms define Ubuntu as
  "Canonical's standard version with no modifications or a version modified
  by Canonical", so Patrone systems can't rightly attach, and the services
  only confuse. Patrone already confines apt news and the ESM check to
  attached machines. **Likely problem**; drop it with the metapackage (§16
  item 1).
- **Ubuntu's archive** (`archive.ubuntu.com`, `security.ubuntu.com`) for
  updates: fine. The policy says users "receive updates to Ubuntu for
  free"; Zimmerman 2007: "The Ubuntu archives are a public service for
  anyone to use"; Linux Mint's defaults use the same servers. Consider a
  mirror if Patrone grows.
- **The Snap Store**: the snaps are Canonical's, unmodified; the
  [Snap Store terms](https://canonical.com/legal/terms-and-policies/snap-store-terms)
  don't restrict redistribution, each snap is "licensed under the terms
  identified for that Snap", and the generic-classic model is snapd's own
  fallback. There's no written grant for third-party images; a Canonical
  engineer's personal view on the Snapcraft forum (2021) was that unmodified
  snaps may be redistributed in a desktop ISO. Fine, with that caveat; link
  the Snap Store's terms and privacy notice from Patrone's pages.
- **Livepatch, motd news, release upgrades, Canonical's NTP and connectivity
  check**: not installed or turned off (verified). `motd-news.timer` still
  fails quietly every day (docs/NEXT.md §11).
- **hwctl** (Canonical's hardware-certification client) is installed as a
  deb and a snap, idle, and sends hardware details to `hw.ubuntu.com` when
  used. Remove it or disclose it.

**Fixed** (2026-10-28, `patrone-desktop` 0.2.0): Ubuntu Pro's client and
`hwctl` are gone. `patrone-desktop` no longer depends on Ubuntu's
metapackages (which need Ubuntu Pro's client and Software Updater) but
lists their packages itself, and conflicts with `ubuntu-pro-client`,
`update-manager` and `hwctl`, so an upgrade takes them, and the
metapackages, off. Its old Ubuntu Pro drop-ins are gone, and Ubuntu Pro's
apt hook, which it had moved aside, is removed rather than put back. The
ISO's seed drops `hwctl`'s snap, the installer's layer purges what nothing
needs any more (16 packages, Ubuntu's release upgrader among them;
`iso/README.md` lists them), and both layers purge what the removed
packages left. Checked on the dev VM: a dry run, then
the install, removed exactly 12 packages (`ubuntu-pro-client` and its
translations, `ubuntu-advantage-desktop-daemon`, `update-manager`,
`update-manager-core`, `update-notifier`, `update-notifier-common`,
`ubuntu-release-upgrader-gtk`, `ubuntu-docs`, `hwctl`, `ubuntu-minimal`,
`ubuntu-desktop-minimal`); `dpkg --audit` is clean; Ubuntu Pro's units are
gone; `apt-get -s autoremove` offers only their libraries and an old
kernel. Ubuntu Pro's apt hook stayed there as a removed package's
configuration file (inert: both its commands check for Pro's files first),
which is why the ISO and `patrone-desktop` remove it.

## 13. Patrone's rebuilds of Ubuntu packages say Ubuntu maintains them

**Likely problem** (main session; `gnome-settings/` is being edited by the
Remote Desktop and desktop-bubble sessions). `gdm3`, `gnome-control-center`
and `gnome-initial-setup` as Patrone builds them keep `Maintainer: Ubuntu
Developers <ubuntu-devel-discuss@lists.ubuntu.com>`, which credits Ubuntu
with Patrone's changes and sends bug reports to Ubuntu's list (the
changelog entries are Patrone's). Debian's [derivatives guidelines](https://wiki.debian.org/Derivatives/Guidelines)
move the old maintainer to `XSBC-Original-Maintainer` and name the new one.
Their copyright files are Ubuntu's and never mention Patrone, though
gnome-control-center now contains Patrone's `cc-patrone-panel.c` and
`cc-patrone-remote-login.c`, and gnome-initial-setup's
`patrone-software.svg` embeds ten Craft apps' icons (MIT OR Apache-2.0,
whose notice it should carry). The versions' "ubuntu" (`0ubuntu0.1patrone1`)
is fine: the convention, and Pop!_OS does the same.

**Fix**: in each `build.sh`, set `Maintainer:` to Patrone with
`XSBC-Original-Maintainer: Ubuntu Developers`, and append a `Files:` stanza
for Patrone's files to `debian/copyright`. **Done for GNOME Settings** since
this review (`67f052b`, revision 12; read, not built); still to do for
`gdm/build.sh` and `gnome-initial-setup/build.sh`.

**Fixed for GDM and first-login setup** (2026-10-28): `gdm/build.sh`
(revision 2) and `gnome-initial-setup/build.sh` (revision 3) name Patrone
as maintainer and Ubuntu Developers as the original one. GDM's copyright
file is plain text, so it gets a paragraph naming Patrone's patch;
first-login setup's gets two stanzas: Patrone's P in place of Ubuntu's logo
and mascot, and `patrone-software.svg`, with GNOME Software's icon (GPL-2+)
and the ten Craft apps' icons, under MIT with its notice and their
holders' lines as their own `LICENSE-MIT` files give them ("ArtCraft Team
and the ... contributors", "the MailCraft contributors"). Checked in the
built packages: `Maintainer` and `Original-Maintainer` are as above in
`gdm3`, `libgdm1` and `gnome-initial-setup`, GDM's copyright file ends with
the paragraph, and first-login setup's passes a strict DEP-5 parse.

Patrone's build of GNOME Software (2026-10-31) does the same from its first
revision: `gnome-software/build.sh` names Patrone as maintainer and Debian's
GNOME team, whose package Ubuntu ships as it is, as the original one, and adds
a stanza for Patrone's patch and its new page, which keeps the copyright lines
of the page it's adapted from. Checked in the built package, whose copyright
file passes a strict DEP-5 parse.

## 14. The website: Ubuntu in a URL, and incomplete trademark notices

**Likely problem** (website session).

- **`/ubuntu`** (the website repo's `src/main.rs:92`): the policy needs Canonical's
  permission for "any Trademark in a domain name or URL". Rename it (for
  example `/foundation`).
- **No source link** next to the ISO download (item 1).
- **The licence click-through**: the real agreement must not restrict what
  the GPL allows: GPL-3 §10, "You may not impose any further restrictions on
  the exercise of the rights granted or affirmed under this License", and
  GPL-2 §6. It's now written: [24](#24-the-licence-agreement).
- **Notices**: the footer covers Ubuntu, Linux and Windows. Missing:
  GNOME, Flathub and Flatpak (all GNOME Foundation marks), Debian's
  requested disclaimer ("Include a disclaimer of sponsorship, affiliation,
  or endorsement by Debian on your website", [debian.org/trademark](https://www.debian.org/trademark)),
  Microsoft's form ("... are trademarks of the Microsoft group of
  companies"), and ® on each page's first standalone "Linux"
  (`index.html:10`, `desktop.html:9`, `ubuntu.html:22`). Suggested footer:

  > PatroneOS is an independent project, not affiliated with, sponsored
  > by or endorsed by Canonical Ltd., Microsoft, the GNOME Foundation, the
  > Debian project, Mozilla, Google or the Linux Foundation. Linux® is the
  > registered trademark of Linus Torvalds in the U.S. and other countries.
  > Ubuntu and Canonical are registered trademarks of Canonical Ltd.
  > Microsoft, Microsoft 365, PowerShell and Windows are trademarks of the
  > Microsoft group of companies. The GNOME name and logo are trademarks of
  > the GNOME Foundation; Flathub and Flatpak are registered trademarks of
  > the GNOME Foundation. Debian is a registered trademark owned by Software
  > in the Public Interest, Inc. Chromium and Google are trademarks of
  > Google LLC. Other names are trademarks of their owners.

- `desktop.html:191` still says the boot screen "spins three interlinked
  rings"; it's one ring now.

**Since this review**, `fe30ef4` renamed `/ubuntu` to `/foundation`, widened
the footer's notices, and made the download page show the source's address
and stay closed until the source is published ("The download opens once
PatroneOS's source code is published alongside it"). Read in the code,
not tested.

**2026-10-29** (website #7 and #8): the footer says Patrone isn't
affiliated with, sponsored by or endorsed by Canonical, Microsoft, the GNOME
Foundation, the Debian project, Mozilla, Google, OffSec or the Linux
Foundation, and names Windows, Hyper-V and Start as the Microsoft group's
trademarks; the home page's Windows section adds that Patrone isn't
affiliated with or endorsed by Microsoft. The site uses the OS's new names
(item 15) and leads with Patrone's words ("Works the way Windows users
expect", "Press the Super key"), and its screenshots were retaken from
this repository at `ecdb5f8`, with the new names and File Manager's new icon.
The website session checked every page for the new words and none of the
old ones.

## 15. Windows 11's look and feel

**Needs a lawyer; low to moderate risk** (the user). Microsoft's
[trademark guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks)
allow truthful references in text and compatibility claims; they bar
Microsoft's marks in product names and its "logos, icons, or designs, in
any manner", and count trade dress as a brand asset. Microsoft's current
trademark list includes **Start** and **Start Design**, Windows, Segoe and
Hyper-V.

- Descriptive mentions ("If you've used Windows...", File Manager's "On
  Windows:" hints): fine; Linux Mint and Zorin OS compare themselves to
  Windows the same way.
- No Microsoft file, font or logo is used; the Start icons are Patrone's
  own; `Segoe UI` is only a fontconfig alias, as Ubuntu's own fontconfig
  has. Fine.
- "Start" for the button is probably descriptive; **"Start" as the menu's
  name** and "Pin to Start" lean on the listed mark.
- Strings copied verbatim from Windows 11's Start menu ("Type here to
  search", "Pinned", "All apps", "Recommended", "Best match"), File
  Explorer's labels ("This PC", "Local Disk") and Task Manager's page names;
  the **Royal** theme is Windows XP's look, a letter from XP Media Center's
  "Royale"; File Manager's icon is File Explorer's yellow-folder shape.
- Package descriptions say "Windows-style" (`patrone-remote-desktop`'s title
  line; reworded in 0.5.3, with a trademark line) and "Windows 11's File
  Explorer"; docs/NEXT.md calls the brief "an exact replica of Windows 11's
  File Explorer".

US courts have mostly refused to protect interface layouts (Apple v.
Microsoft, 1994; Lotus v. Borland, 1995), and trade dress in a product's
design needs proof of acquired distinctiveness (Wal-Mart v. Samara, 2000).
Microsoft's one fight with a Linux vendor was over the name (Lindows,
settled in 2004). **To lower the risk**: Patrone's own words for the copied
strings and the menu, another name for the Royal theme, a redrawn File
Manager icon, descriptions that lead with Patrone's words ("works the way
Windows users expect"), and rewording the docs before the repository goes
public. Zorin OS sells "Windows 11-like" layouts with only a generic
trademark footer.

**2026-10-29**: what "To lower the risk" lists is done, in the OS and on
the website (#19 to #27; website #7 and #8, item 14):

- **The menu** (taskbar 0.4.5, #21): the button's tooltip and screen-reader
  name are "Menu", and the docs call it "the Patrone menu". "Type here to
  search" is "Search apps, files and the web"; Pinned, All apps,
  Recommended and Best match are Favorites, All applications, Recent and Top
  result; Pin to Start is Add to Favorites.
- **File Manager** (0.3.6, #22; the taskbar's Files pin, 0.4.6, #25): "This
  PC" is Computer, "Local Disk" is System drive, and Computer's sections
  are Drives and Network drives. A new app icon, drawn for Patrone (a folder
  in Patrone's blues with the slant of the P), replaces File Explorer's
  yellow-folder shape. The copyright file no longer calls the art "in the
  style of Windows 11's Fluent design".
- **Task Manager** (0.2.1 and 0.2.2, #20 and #23): App history is Usage
  history, Startup apps is Startup applications, and its commands are Run a
  program, End, End process and its children, Show in File Manager and Show
  details.
- **The Royal theme is Sapphire** (Patrone Settings 0.3.7, GNOME Settings
  revision 14, #24). Its look is unchanged.
- **Hyper-V Setup is Virtual Machine Setup** (`patrone-hyperv-setup` 0.2.0,
  #26): Microsoft's guidelines bar its marks in product names. Hyper-V
  stays in the text as a truthful reference, with "Hyper-V and Windows are
  trademarks of the Microsoft group of companies."
- **Descriptions lead with Patrone's words.** Remote Desktop's
  "Windows-style sessions" is "one desktop per account, on the screen and
  remotely", with a trademark line (0.5.3, #19). File Manager, Task Manager
  and the taskbar "work the way Windows users expect", and docs/NEXT.md and
  docs/PLAN.md no longer call File Manager "an exact replica of Windows 11's
  File Explorer" (#27).

Left as it was: the look itself. That covers the taskbar's and the menu's
layout, File Manager's layout and its Fluent-like folder and file icons, and
the Sapphire theme's green Start button. Also left: "Start button" in
Patrone Settings and GNOME Settings' Patrone page, the jump lists' "Pinned"
folders and "Top apps" in the menu's search, `Segoe UI` as a fontconfig
alias, and internal names (`this-pc`, `royal`, `org.patrone.HyperVSetup`).
Still a lawyer's question, now at the low end: what's left is layout and
look, which courts have mostly not protected.

## 16. The ZFS kernel module

**Needs a lawyer** (the user). The installed system has
`linux-main-modules-zfs-7.0.0-30-generic`
(`/usr/lib/modules/7.0.0-30-generic/ubuntu/dkms/zfs/zfs.ko.zst`, CDDL) with
the GPL-2 kernel, and the disc's pool has `zfsutils-linux` and friends.
Canonical ships them on its own legal analysis (2016); the Software Freedom
Conservancy and the FSF say distributing the combination infringes.
Distributing the ISO makes Patrone a distributor of the combination too
([Canonical](https://ubuntu.com/blog/zfs-licensing-and-linux),
[Conservancy](https://sfconservancy.org/blog/2016/feb/25/zfs-and-linux/),
[FSF](https://www.fsf.org/licensing/zfs-and-linux)). Of the installed
system's 6,903 kernel modules it's the only one with a non-GPL licence tag.
Separately, CDDL §3.1 asks whoever distributes the module to say how to get
its source, and no CDDL text ships with it (likely problem, inherited from
Ubuntu; item 1 covers it). **Options**: accept Canonical's position, or
purge the ZFS modules and tools from both layers and the pool and hide the
installer's ZFS choice.

## 17. NVIDIA and Broadcom drivers on the disc

**Fine to redistribute; the obligations need a lawyer** (the user). The
disc's `pool/restricted` (38 `.deb`s, 933 MB, byte-identical to Ubuntu
26.04.1's) holds NVIDIA's 580 and 595 drivers, prebuilt module objects for
the 7.0.0-30 kernel, and `broadcom-sta-dkms`. None is installed by default.

- NVIDIA's licence (each package's copyright file; NVIDIA's own page, "v.
  February 25, 2025") lets you "Distribute the SOFTWARE provided for use
  with operating system kernels distributed under the terms of an
  OSI-approved open source license ... provided that (i) the binary files
  thereof are not modified in any way ... and (ii) this Agreement is
  provided to each SOFTWARE recipient." Both hold. But §2.11 makes the
  distributor "defend, indemnify and hold harmless NVIDIA".
- Broadcom's lets you "reproduce and distribute the Software complete,
  unmodified and only for use with a Broadcom Product", with the agreement
  attached, and "agrees to defend and indemnify Broadcom".
- The prebuilt NVIDIA module objects are linked against the kernel on the
  user's machine, Canonical's arrangement; the GPL-2 question is the same as
  Ubuntu's (the FSF calls distributing such drivers a violation).
- Taking these licences on yourself also means (the re-check's reading of
  the files): NVIDIA's export and sanctions duties (§15), marking it as
  commercial software before supplying the US Government (§16, hard for a
  public download), binding arbitration with a class-action waiver (§10);
  Broadcom's covenant never to assert your own IP against its software
  (§2.5), confidentiality of its documentation and California courts. And
  `/etc/legal` said everything was free software (now fixed).

**Options**: keep them (accepting the indemnities and export rules), or drop
`pool/restricted` from the ISO in `iso/build-iso.sh`; the installer's
"third-party drivers" choice then downloads them when online.

Firmware and microcode in the installed system are all from Ubuntu's main,
and their terms allow unmodified redistribution with the notice, which
ships: fine. Thirteen firmware files are marked `License: Unknown` (in
`linux-firmware-misc` and `-qlogic`); Ubuntu ships them too: low risk, a
lawyer's call whether to drop them. Codecs: MP3's patents expired in 2017
(fine); Mesa and Canonical's Chromium snap are built with H.264, HEVC and
AAC support, as in Ubuntu: low risk, not checked further.

## 18. Smaller notices and wording

- **`/etc/legal`** said "The programs included with it are free software"
  (the firmware isn't) and had no trademark notices. **Fixed**
  (`branding/system/legal`): independent project, not affiliated with
  Canonical, the GNOME Foundation, Microsoft or the Linux Foundation; most
  programs free software, some firmware under its makers' terms; the Linux,
  Ubuntu, GNOME and Windows notices. Takes effect at the next
  `patrone-branding` build.
- **`patrone-branding`'s copyright file** claimed Yaru's login theme
  (GPL-3+, with an LGPL-2.1 part) as Patrone's. **Fixed**, then mended
  (`460713b`: the first edit had left the Plymouth images' GPL-2+ without
  its text; a strict DEP-5 parse now passes for all seven files changed).
- **`patrone-installer`'s copyright file** didn't say what Patrone changed
  in Canonical's installer. **Fixed.**
- **The login-screen extension** (`remote-desktop/login-screen/.../extension.js:16`)
  is "GNOME's _findConflictingSession, without sessions that have no seat",
  rewritten from GNOME Shell's `js/gdm/loginDialog.js` (lines 1374-1397 in
  GNOME Shell 50), but credited only William Daugherty. **Fixed** by the
  Remote Desktop session since this review (`67f052b`, with
  `patrone-remote-desktop` 0.4.8's copyright file and a description that
  leads with Patrone's own words).
- **Text pointing at Ubuntu**: first-login setup's "View release notes"
  opens Ubuntu's notes for version 0.1; Language Support's "The recommended
  value for Ubuntu is IBus"; Settings › Desktop's "No Ubuntu settings found
  ... report a bug in Launchpad"; the GRUB menu's "PatroneOS GNU/Linux".
  §16 items 6, 8, 12 and 14. Likely problem (minor): they imply Ubuntu
  supports Patrone. **Fixed** (2026-10-29, #28):
  - First-login setup's release notes link is hidden until Patrone has
    release notes (`patrone4`).
  - GNOME Settings' error state says "No desktop settings found", with no
    Launchpad (`patrone15`).
  - Language Support is rebuilt with Patrone's words: its tip, its error
    message and its help (`language-selector/`, `patrone1`).
  - The boot menu says "PatroneOS": `patrone-branding` 0.1.22 moves
    GRUB's `10_linux` and `10_linux_zfs` aside and runs them without "
    GNU/Linux".
  - Checked on the dev VM, and in a fresh install from a test disc.
- **JetBrains Mono**: Ubuntu's `fonts-jetbrains-mono` copyright file says
  Apache-2, but the font is OFL 1.1 and no OFL text ships. Ubuntu's mistake,
  harmless to Patrone. Since `patrone-terminal` 0.1.8 (2026-10-29) Patrone no
  longer installs it: the terminal's font is PatroneOS Mono.
- **MailCraft** belongs to UCS Care Group (`UCS-Care-Group/mailcraft`,
  GOVERNANCE.md: "Owner | UCS Care Group"), which billsecond helps maintain;
  `software/README.md` and docs/NEXT.md call it billsecond's. The Craft
  apps are ArtCraft's (the `storytold` organisation, "Learning Machines
  LLC" in their listings), not billsecond's.

## 19. Privacy law for the website

The owner is a private individual in the US with a free, non-commercial
site open to the world.

- **GDPR and UK GDPR: needs a lawyer.** Art. 3(2)(a) reaches "the offering
  of goods or services, irrespective of whether a payment ... is
  required" to people in the EU; recital 23 says "mere accessibility" isn't
  enough. A free OS offered worldwide, with a privacy page that addresses
  EU and UK readers and European time servers in the OS, leans towards
  applying, without settling it. If it applies, Art. 27 asks for an EU and a
  UK representative unless processing is "occasional", which an
  always-open form isn't. **Comply anyway; it's cheap**: the fixes in item
  5, plus the page's missing Art. 13 items: the legal basis for each use
  (consent for the beta list and the optional details; legitimate interest
  for a shortened download record), a contact address besides the form,
  restriction and portability, that the data sits in the US with Microsoft
  365 as processor, and "no decisions are made about you automatically".
- **US laws: fine.** CCPA/CPRA covers businesses run for profit above its
  thresholds (Cal. Civ. Code 1798.140(d)); CalOPPA covers sites "operated
  for commercial purposes" (B&P 22577(c)), but add its Do Not Track line
  anyway ("This site doesn't track you across sites, so Do Not Track
  signals change nothing here"); Virginia's and Colorado's need 100,000
  residents; COPPA covers commercial sites.
- **CAN-SPAM: needs a lawyer, low stakes.** Requested beta notices about a
  free OS are probably not "commercial electronic mail"; name the sender,
  give a one-line opt-out and a postal address (a PO box will do) in every
  bulk email anyway. **PECR/ePrivacy**: consent is collected (the download
  form's box is unticked by default); the first email should say how the
  address got on the list and offer one-step removal, since nothing
  confirms the address belongs to whoever typed it.
- **Cookies: fine.** None, and no browser storage; no banner needed.
- **Logs**: Caddy keeps no access log, but its error log records visitors'
  addresses when the site is down, in a journal limited by size, not age.
  Use Caddy's `ip_mask` filter and journald's `MaxRetentionSec=7day`, and
  say so plainly.

## 20. What the OS contacts, and whether users are told

**The operators' terms: fine**, apart from TimeNL (item 7).

| Service | Operator | Terms for a distribution's default |
| --- | --- | --- |
| `time.cloudflare.com` (NTS) | Cloudflare (US) | Website and Online Services Terms: no rule on defaults; don't overburden |
| `nts.netnod.se` | Netnod (Sweden) | Free, "for users anywhere in the world"; no vendor rule |
| `ptbtime1.ptb.de` | PTB (Germany) | Free, no registration; no rate limiting |
| `ntppool1.time.nl` | SIDN Labs (Netherlands) | Ask first (item 7) |
| `nmcheck.gnome.org` | GNOME | No published terms; GNOME Shell uses it itself |
| Flathub | Flathub | No rule on adding the remote; Patrone's `flathub.flatpakrepo` is byte-identical to Flathub's |
| `billsecond.github.io/patrone-app-index` | GitHub | Pages: soft 100 GB/month, "primarily as a showcase"; about 0.35 MB per install; GitHub logs visitors' IP addresses |
| Craft apps' release downloads | GitHub | "no limit on the total size of a release, nor bandwidth usage" |
| Ubuntu's archive, Snap Store, `geoip.ubuntu.com` | Canonical | Item 12 |
| LVFS, ODRS, BeaconDB (location, off by default) | LVFS, GNOME (hosted by Red Hat and CDN77), one volunteer | No rule on defaults |
| Chromium's Google services | Google, through the key in Canonical's snap | Safe Browsing, suggestions, Translate, spell check |

Courtesy notes to Netnod, PTB and GNOME Infrastructure would meet RFC
8633's spirit.

**No notice in the OS: likely problem (good practice).** Patrone receives
nothing itself; each operator above is its own controller. Ship the
website's list in the OS (for example
`/usr/share/doc/patrone-desktop/PRIVACY.md`, linked from Settings › About
and first-login setup), and add what `privacy_os.html` leaves out: hwctl,
Chromium's Google services, GitHub's address logging and the installer's
telemetry page.

**Fixed** (2026-10-28, `patrone-desktop` 0.2.0):
`/usr/share/doc/patrone-desktop/connections.txt` (`desktop/connections.txt`)
lists each connection and who runs it, as the website's page does, for
this version: no browser preinstalled, no Ubuntu Pro or `hwctl`, GitHub's
address logging, the installer's local record. `/etc/legal`, shown at a
text console's login, says where it is. Not linked from Settings › About
or first-login setup: both are rebuilds other sessions are changing. The
website's page still describes Chromium, `hwctl`, Ubuntu Pro's tools and a
telemetry question in the installer, which this release doesn't have; sent
to the website session.

## 21. The app index otherwise

- The 13 other storytold apps and MailCraft are `MIT OR Apache-2.0`, with
  CC0 metadata; listing their names, descriptions and icons is allowed, and
  screenshots are linked, not copied. The index's appstream gives every app
  its licence correctly, so GNOME Software shows them as free. **Fine.**
- **The icons travel without their licence text**: in each entry
  `files/share/doc` is a link into `/app/extra`, so the licences arrive only
  after install, and the index repository has no LICENSE or NOTICE.
  **Likely problem, low risk.** Keep `share/doc`, or copy each app's
  LICENSE files into `files/share/licenses/<id>/`, in `build-index.sh`, and
  add a NOTICE section to the index's README.
- The index holds only listings (1,430 files, 5 MB). `build-index.sh`
  rewrites only `files/bin` and `files/share`, so a future app with another
  top-level folder would be copied whole and Pages would serve app code:
  turn every other folder into a link and fail above about 2 MB.
- GitHub Actions: a daily job publishing the repository's own index is
  within the terms. Pin `actions/checkout` by commit, write down how to
  rotate the signing key (no passphrase, as an Actions secret), and keep
  push access to `main` to the owner.

## 22. Checked and fine

- **"Based on Ubuntu"** in `os-release`, `/etc/legal` and the website:
  the policy allows referring to Ubuntu without implying endorsement, and
  Zimmerman (2007): "'based on Ubuntu ...' requires no prior permission".
  Never a "*buntu" name or "Ubuntu X".
- **"ubuntu" in version strings** and the kernel's `#30-Ubuntu`: factual.
  Package names, dpkg's vendor, apport's crash database and `BOOTX64.CSV`
  saying "ubuntu" are functional (optional tidy-ups).
- **Secure Boot**: shim, GRUB and the kernel are Ubuntu's, unchanged and
  signed by Canonical; Mint and Zorin boot the same way. Patrone can't
  change GRUB or the kernel and keep Secure Boot.
- **The disc's apt repository** (`dists/`, `pool/`) is Ubuntu's, signed
  with a one-time key embedded in the disc's own `cdrom.sources`. Its
  `Release` files and package names say Ubuntu, and must: changing them
  would break the signature. They're part of item 2 (Ubuntu's own
  binaries), not branding. The disc's volume name and `.disk/info` say
  PatroneOS.
- **The `ubuntu` link at the disc's root** (Ubuntu's `ubuntu -> .`, a
  Debian-era convention that Windows shows as a 0 KB file named UBUNTU,
  found by the user) was used by nothing on Patrone's disc: apt and the
  installer use `file:///cdrom`, and neither the initrd nor `md5sum.txt`
  goes through it. The main session took it off in `cd60745`, from the
  next ISO build.
- **Ubuntu's fonts** (Ubuntu Font Licence 1.0) are unmodified; if Patrone
  ever changes them, the licence requires a new name. **PatroneOS's fonts**
  (`fonts-patroneos`, 2026-10-29, in place of Patrone Sans): drawn from
  scratch, OFL 1.1 with PatroneOS's copyright only and no Reserved Font
  Name; the name tables hold only PatroneOS's own names, and the source,
  FONTLOG and licence are public (billsecond/patroneos-fonts).
- **Yaru's themes, icons and sounds** are used unmodified, and the login
  theme's pieces are Yaru's GPL-3+ files, not its CC BY-SA icons. Patrone's
  own icons are its own drawings.
- **GNOME Software as "Patrone Software"**: only the launcher's name
  changes, its metainfo still says "The GNOME Project", and the website says
  it's GNOME Software. Fine; add the GNOME credit (item 14).
- **Chromium**: Canonical's snap, unmodified, called Chromium everywhere;
  its "This Space Intentionally Blank" terms dialog is Chromium's
  placeholder, not Google's terms.
- **Debian** (`ID_LIKE=debian`), **Cloudflare** (words only), **Kali** (only
  in docs and comments), **JetBrains Mono**, **Hyper-V** (docs and a GDM
  patch only), **VirtualBox**: the new `patrone-guest-tools` depends on
  Ubuntu's `virtualbox-guest-utils` and `virtualbox-guest-x11` (7.2.6,
  multiverse). The guest utilities are GPL-3.0 with Oracle's SPDX headers;
  the package sits in multiverse (Debian's contrib) only because the
  VirtualBox host's BIOS, built from the same source, needs the non-free
  Open Watcom compiler. Fine, shipped unchanged under their name; they join
  the source duty (item 1), and Oracle's Extension Pack must never be
  shipped.
- **Microsoft's Segoe UI** alias, **cookies**, **children** (16+ and no
  commercial site).
- **Cursor themes** (2026-10-29), which the new Mouse pointer choice lists
  (Patrone Settings 0.3.8, GNOME Settings revision 16): Patrone's own,
  Patrone Blue and, since `patrone-branding` 0.1.25, Patrone Black and
  Patrone White (GPL-3+, all three drawn by `branding/tools/make_cursors.py`
  from the same shapes, Patrone's own design), Adwaita (`adwaita-icon-theme`,
  CC-BY-SA-3.0 or LGPL-3), Yaru (`yaru-theme-icon`, CC-BY-SA-4.0), and KDE's
  Breeze Dark and Breeze Light (`breeze-cursor-theme`, universe, LGPL-3+,
  by Uri Herrera and the KDE VDG), which `patrone-desktop` 0.2.5 recommends
  so there's a white pointer. Each is its project's own design, unmodified;
  none copies Windows' or macOS's cursors. Left out: Bibata and phinger,
  whose busy pointers are rainbow wheels like macOS's (and Bibata is 178 MB
  installed), and Oxygen (dated). DMZ (`dmz-cursor-theme`, CC-BY-SA-3.0,
  openSUSE's) and X.Org's whiteglass, redglass and handhelds
  (`xcursor-themes`, MIT/X11) stay installed, as Ubuntu's desktop has them,
  but aren't listed: they have only the old X11 cursor names, so GNOME
  shows Patrone's pointers in their place (`settings/README.md`, "Mouse
  pointer").

## 23. The website's notices, and the JavaScript notice

The user, 2026-10-28: "make sure the website follows legal compliance also,
including the javascript notification." What the law asks of the site as it
is (no cookies, no analytics, no third-party requests; a download form with
optional name, email and use, a separate unticked "OK to contact me", and a
record of each download), sent to the website session:

- **A cookie or consent banner isn't required, and shouldn't be added.**
  ePrivacy Art. 5(3) (PECR reg. 6 in the UK) needs consent only for storing
  or reading things on the device that aren't strictly necessary for what
  the user asked for. Nothing on the site needs consent, so a "we use
  cookies, accept?" banner would be untrue. Fine without one.
- **The JavaScript notice** (the website repo's `static/notice.js`) is an information
  notice, not a consent request: no Accept or Reject, no "by continuing you
  agree" (continued browsing isn't consent, EDPB Guidelines 05/2020). It
  must not block or cover the page or the form's own notice; it should be a
  labelled region with a real Close button that Escape also closes, without
  stealing focus. The same words must be on the page in plain HTML without
  JavaScript (GDPR Art. 12, "easily accessible"); the footer does that.
- **Remembering its dismissal in localStorage is allowed without consent.**
  localStorage is within Art. 5(3) (EDPB Guidelines 2/2023), but storing
  only the choice the user made by clicking Close is the user-interface
  exemption (Article 29 Working Party Opinion 04/2012). Conditions: only a
  version string, no identifier, never sent to the site, named in the
  privacy policy, and a new version shown when the policy changes (which is
  also CalOPPA's "how you're told of changes").
- **At the point of collection** (GDPR Art. 13, "at the time when personal
  data are obtained"), in plain HTML above each form's button: who collects
  it, what is always recorded and why, what is optional and how long each
  is kept, what the contact box means and how to stop, and links to the
  policy and the request form. The licence box and the contact box stay
  separate, and the contact box unticked (Art. 7(2) and 7(4)).
- **The privacy policy** must have the Art. 13 items (controller and a
  contact besides the form, purposes and legal bases, recipients, the US
  location and Microsoft's safeguard, retention per category, every right
  and how to use it, the right to complain, what's optional, no automated
  decisions) and CalOPPA's (categories collected, Do Not Track, no
  third-party tracking, how changes are announced, an effective date, a
  "Privacy" link on the home page), plus the localStorage key.
- **The scripts**: no law requires a notice for first-party scripts.
  `notice.js` is the only one that stores anything (`download.js` and
  `pin.js` store and send nothing), so only it is named in the policy. Any
  third-party script later would need a real consent banner, with an equal
  "Reject", before it loads.
- **Public download counts**: aggregates only, never the "Other" text.
- If "the javascript notification" meant **browser push notifications**:
  they need a click before the browser's permission prompt and create
  personal data held by the browser maker's push service. Not recommended;
  the email opt-in covers release news.

The website session's `fe30ef4` already has the plain-HTML notice and the
versioned dismissal; the rest was sent before its next redeploy.

## 24. The licence agreement

The user asked for "a strong EULA in the website for the software that the
user must click before downloading, and ... that same one up on the install
page", and chose Texas law. It's [legal/eula.md](../legal/eula.md), version
1.0 of 29 October 2026: one file for the website's click-through and the
installer's page, in the website renderer's Markdown, its version on the
first line. [legal/README.md](../legal/README.md) has its format, what it
may and may not say, and the points a lawyer must check.

It is strong where Patrone can set terms, and stops where the free licences
begin:

- **The free licences come first** (section 1): each program keeps its
  licence, which wins in any conflict; declining the agreement doesn't
  affect them; anyone a copy is shared with gets the programs under their
  licences whether or not they accept; the source is never behind the
  click-through. That's what GPL-3 §10 and GPL-2 §6 require.
- **The name and logos** (3): truthful references and unchanged copies are
  fine; modified versions take their own name and drop the logos (rights
  the free licences don't grant: Apache-2.0 §6, MPL-2.0 §2.3).
- **No warranty and limits on liability** (5, 7): in capitals, naming
  negligence (Texas's fair-notice rule), a US$10 cap, with the liability
  the law won't let anyone exclude carved out first.
- **Promises are the sharer's** (8), on GPL-3 §7(f)'s pattern, for
  Patrone's own programs and marks only; **fair use of the download** (9),
  which never reaches the source; the **non-free parts** (2) under their
  makers' licences, with NVIDIA's U.S. Government notice; **privacy**,
  **services run by others**, **changes**, **ending**, **Texas law** with
  consumers' own protections kept, and **contact** through the site's
  request form.

An independent reviewer checked the draft against GPL-2, GPL-3, MPL-2.0 and
Apache-2.0, the NVIDIA and Broadcom licences, clickwrap case law, and
Texas, EU and UK consumer law, and ran it through a copy of the site's
renderer. Its seven must-fixes are in version 1.0: ending the agreement
can't stop anyone sharing unchanged copies; the indemnity is narrowed to
§7(f); the drivers' licences are pointed to where they really are; the
liability clauses name negligence and put the carve-outs first; the
services paragraph matches the ISO without a browser (`8e0f189`); and a
real contact replaces a missing one.

**The website shows it** since `fe1f631` (patrone-web 0.10.0): in full on
the download page, with "It includes no warranty and limits our liability
(sections 5 and 7)" and a link to `/licence/1.0` above the checkbox, and on
that page of its own; the version accepted is recorded as before (checked
on the running site: `/licence/1.0` answers 200 with the sections as
headings, and the download page has the line and the link). Still for the
installer: write the accepted version to the installed system and leave
the live session ungated. And a plain copy of the NVIDIA and Broadcom
licences on the disc and the download page would settle section 2. **Needs a lawyer before it's relied on**:
the nine points in `legal/README.md`.

## Decisions for the user

The user's answers (2026-10-28): open the repository to the public, and
"what ever you need for the compliance" for the rest, so this review took
the simplest safe option where no lawyer or the user personally is needed.

1. **Where the source goes** (item 1): the repository is public; the source
   of everything on the disc, which `iso/build-source.sh` gathers, goes
   beside the ISO on the website, from the next ISO on. Nothing should be
   downloadable before both.
2. **Canonical** (item 2): take advice on the binaries clause, and decide
   whether to write to Canonical. A draft letter is ready (2026-10-29), kept
   outside this repository.
3. **The Linux sublicence** (item 4): applied for; waiting on the Linux
   Foundation. Since the name is PatroneOS (2026-10-29) it isn't needed:
   withdraw the request, or let it lapse.
4. **ArtCraft's icon** (item 6): coming off the app index (its pull request
   waits for the user to merge it) until the ArtCraft Team gives
   permission; asking them is the user's.
5. **TimeNL** (item 7): dropped. Writing to SIDN Labs would bring it back.
6. **The AI-made art** (item 9): labelled honestly; who owns it, and a
   hand-drawn logo, are for a lawyer and the user.
7. **ZFS** (item 16) and **NVIDIA/Broadcom** (item 17): keep or drop.
8. **How close to Windows** (item 15): the menu's name, the copied strings,
   the Royal theme, the File Manager icon. Changed, at the user's word
   (2026-10-29: "do it now"); the look stays.
9. **GDPR** (item 19): whether to appoint EU and UK representatives.
10. **The licence agreement** (item 24): written, with Texas law as you
    chose.

## For a lawyer

1. Canonical's binaries clause, and whether to send the draft letter to
   Canonical (item 2).
2. Whether the source mirror is needed, or Launchpad is enough, and how far
   the duty reaches into Canonical's snaps, whose manifests don't record
   the commit of their own code (item 1).
3. ArtCraft's trademarked icon (item 6).
4. Who owns the AI-made logo and wallpaper, and a clearance search for
   PATRONEOS in classes 9 and 42, with ParrotOS in mind (item 9).
5. Whether the look that's left is safe, now that the names are Patrone's
   (item 15).
6. ZFS's kernel module (item 16), and NVIDIA's and Broadcom's indemnities
   and terms (item 17).
7. Whether GDPR and UK law apply to the website, and the representatives
   (item 19).
8. The licence agreement's nine points (`legal/README.md`, item 24).

## Fixes made in this review

- `5b9356e` and `460713b`: the Rust packages' third-party notices
  (`packaging/third-party-licenses.sh`, `build.sh`, five copyright files);
  Yaru's login theme in `patrone-branding`'s copyright file; what Patrone
  changed in `patrone-installer`'s.
- The commit that adds this file: the installer bundle's fonts and data in
  `patrone-installer`'s copyright file (the Ubuntu fonts with the UFL's
  text, Yaru's icon font, Material Icons, GeoNames' data, and a header that
  no longer says NOTICES.Z has every licence), and `/etc/legal`'s wording.
  All seven copyright files changed pass a strict DEP-5 parse (python-debian,
  `strict=True`, every licence with its text).

Each takes effect when its package is next built and the ISO rebuilt.

Other sessions acted on their findings while this review was finishing:
the website session in `fe30ef4` (item 5 and 14) and the Remote Desktop
session in `67f052b` (items 13 and 18).

**2026-10-29**, merged that day:

| PR | What | Item |
| --- | --- | --- |
| #19 | Remote Desktop described in Patrone's words, with a trademark line (0.5.3) | 15 |
| #20, #23 | Task Manager's page names and commands in Patrone's words (0.2.1, 0.2.2) | 15 |
| #21 | The Patrone menu: "Menu", Favorites, All applications, Recent, Top result (taskbar 0.4.5) | 15 |
| #22, #25 | File Manager: Computer, System drive, its own icon (0.3.6; taskbar 0.4.6) | 15 |
| #24 | The Royal theme is Sapphire (Settings 0.3.7, GNOME Settings revision 14) | 15 |
| #26 | Hyper-V Setup is Virtual Machine Setup, with a trademark line (0.2.0) | 15 |
| #27 | The project notes in Patrone's words | 15 |
| #28 | Ubuntu's name out of first-login setup, GNOME Settings, Language Support and the boot menu | 13, 18 |
| #29 | The disc's layers are Patrone's own; Ubuntu's TPM seed system and its kernel snap gone | 1, 3, 11 |
| #30 (open) | The kernel snap's two leftover components go too | 1, 17 |
| Website #7, #8 | Patrone's names and words, Microsoft's trademarks and non-affiliation, new screenshots | 14, 15 |

Each reaches installs when the daily publish builds its new version, and
the disc when the next ISO is built.

## Inventory

**2026-10-29**, from a test disc built from #29 with #28's packages
(Patrone's own layers only): the installed system has 1,535 packages from
899 source packages, and the live session 1,833 from 1,049. Nine of them
are Patrone's rebuilds of Ubuntu packages (GDM's three, GNOME Settings'
three, first-login setup, and Language Support's two), and 16 are Patrone's
own in each. The seeds have 9 snaps (installed) and 10 (live, adding
`ubuntu-desktop-bootstrap`). The live seed also kept two of `pc-kernel`'s
NVIDIA driver components, 16 KB each, after `pc-kernel` itself went;
#30 drops them. The disc's pool still has its 155 `.deb`s. Not broken down
by archive component again.

From the ISO built 2026-10-27, mounted read-only on the dev VM in
`/var/tmp/patrone-compliance/` (`inv/` has the lists):

| | Installed system | Live session |
| --- | --- | --- |
| Packages | 1,540 | 1,845 |
| From Ubuntu's main (with Patrone's rebuilds of main packages) | 1,493 | 1,808 |
| From Ubuntu's universe | 34 | 24 |
| From restricted or multiverse | 0 | 0 |
| Patrone's own (not in Ubuntu's archive) | 13 | 13 (`patrone-installer` in place of `patrone-remote-desktop`) |
| Patrone's rebuilds of Ubuntu packages | 7 | 7 |
| Source packages | 908 | 1,052 (1,064 together) |
| Seeded snaps | 13 | 15 (adds `thunderbird`, `ubuntu-desktop-bootstrap`) |

The disc's own pool, which the installer installs from: 117 `.deb`s from
main (GRUB, shim, the kernel, ZFS, compilers, drivers' helpers) and 38 from
restricted (NVIDIA, Broadcom). The universe packages are Patrone's
additions and their dependencies: `7zip`, `bat`, `btop`, `cage`,
`chromium-browser` (the stub), `fastfetch`, `flatpak`,
`freerdp-proxy`, `freerdp-sdl`, `fzf`,
`gnome-software` and its plugins, `ncdu`, `ripgrep`, `tree`, the zsh
plugins, and libraries (`libflatpak0`, `libostree-1-1`, `libsdl3-*`,
`libwlroots-0.19` and others); the live session adds `gnome-kiosk`.

To make the lists again: mount the ISO and its layers as `iso/README.md`
describes, then `dpkg-query --admindir=ROOT/var/lib/dpkg -W` on each, and
look each version up in the apt indexes' `Packages` files for its component.
