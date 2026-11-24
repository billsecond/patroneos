# Patrone's GDM

Patrone builds Ubuntu's GDM (`gdm3`) again with three changes: the login screen
doesn't keep a stalled boot splash on screen for 10 seconds on a Hyper-V VM
(below), a desktop can be unlocked after its screen is locked
([Unlocking a desktop](#unlocking-a-desktop)), and the disc's installer doesn't
get a login screen over it ([The disc's login screen](#the-discs-login-screen)).

The user saw it on the dev VM (2026-10-27): the boot splash's spinner turned
for only a few seconds, then the logo stood still until the login screen.
The 10:07 boot's journal, on the monotonic clock:

| Seconds after boot | |
| --- | --- |
| 2.4 | `hyperv_drm`, Hyper-V's display driver, starts (from the initramfs) |
| 4.5 | GDM starts and runs `plymouth deactivate`: the splash stays on screen but stops animating |
| 15.0 | GDM: "It appears that your system does not have a primary GPU! Proceeding with any GPU" |
| 15.5 | The login screen's GNOME Shell starts |

(The wall-clock times looked like 15 seconds because chrony stepped the clock
4.7 seconds forward in between.)

GDM 50.1 waits up to 10 seconds at startup (`SEAT0_GRAPHICS_CHECK_TIMEOUT`
in `daemon/gdm-local-display-factory.c`) for the display the computer booted
on, and in `udev_is_settled()` it only recognizes one that's a platform
device or a PCI or DRM device marked by the firmware (`boot_vga` or
`boot_display`). A generation 2 Hyper-V VM's only display, Hyper-V's
synthetic video, is on the VMBus, so GDM never recognizes it and always waits
the full 10 seconds.

| File | |
| --- | --- |
| `patches/patrone-hyperv-boot-display.patch` | Counts a display on the VMBus as the boot display: `hyperv_drm` takes over the framebuffer the VM's firmware booted on. Other computers are checked as before |
| `patches/patrone-unlock-seatless-session.patch` | Unlocks a session with no seat after its lock screen takes the password |
| `patches/patrone-splash-quit-fallback.patch` | Quits the boot splash the usual way when a session is slow to register, so the screen stays on that session |
| `build.sh` | Fetches Ubuntu's source, adds the patch, versions it Ubuntu's version plus `patrone` and the revision, and builds. From revision 2 it names Patrone as the packages' maintainer and Ubuntu Developers as the original one, as Debian's guidelines for derivatives have it, and adds Patrone's patch to the copyright file (`docs/COMPLIANCE.md`, item 13) |

GDM still deactivates the splash as it starts: Plymouth has to let go of the
screen before GNOME Shell can take it. Without the wait, the splash stands
still only while the login screen starts, about a second.

## Unlocking a desktop

Patrone's desktops have no seat: the screen shows them through a viewer in a
session of its own ([billsecond/patrone-remote-desktop](https://github.com/billsecond/patrone-remote-desktop)). Nothing locks them by itself
(`libexec/no-lock`), but the lock button and Super+L do. GDM then checks the
password, asks logind to activate the session on its display's seat, and only
after that unlocks it. logind refuses (`SessionNotOnSeat: Session 3 not on seat
seat0`), so GDM stopped there: the lock screen took the right password, cleared
it and stayed, and only restarting got back to the desktop. After a password
check that didn't come from a login screen, GDM now also unlocks the account's
graphical sessions with no seat, and then carries on as before. It can't go by
who asked: GNOME Shell runs as a user service, outside any logind session, and
an account with a Patrone desktop has two sessions, the desktop and the screen's
viewer, so GDM's own guess at the session can be the wrong one.

`switch_to_compatible_user_session()` itself has to stay as it is: at sign-in,
that failed activation is how GDM learns it can't switch to the desktop, so it
goes on to start the screen's viewer.

`50.1-0ubuntu0.1patrone4`, on a fresh install in a QEMU VM (2026-10-30): the
lock button and Super+L lock the desktop, the right password unlocks it, a wrong
one doesn't, and signing out afterwards leaves nothing behind. That holds with
the desktop started by the screen's sign-in, and with it started by a Remote
Desktop sign-in and the screen signing in after. The journal still shows the
`SessionNotOnSeat` line at each unlock, from the unchanged path that runs after.

## The disc's login screen

"Install PatroneOS" boots to the installer alone, in a GNOME Kiosk session that
GDM logs in by itself (`installer/README.md`). About one boot in seven showed the
login screen with "Live session user" in its place. Clicking the account went
straight to the installer, which had been running behind it all along.

Once a session registers, GDM gives it 10 seconds to register its display too,
which quits the splash with `plymouth quit --retain-splash`. If that hasn't
come, a fallback runs a plain `plymouth quit`, and Plymouth takes the screen
back to the first VT. GDM reads that as someone switching to an empty VT and
starts a login screen there. GNOME Shell registers its display at once, so
desktops and login screens never get to the fallback. GNOME Kiosk doesn't
register: `gdm-wayland-session` does it for it after the same 10 seconds, so
the two timers race, and which one wins comes down to fractions of a second.
The fallback now quits the splash the way the usual path does. The session
already has the screen by then, and nothing shows the splash's last frame.

Checked on the disc built from a55bff4, in QEMU, booting it with a GDM swapped
in and its debug log on the serial port (2026-10-30). As it is, 3 of 20 boots
showed the login screen, most of them with the machine busy. With the fallback
forced to run first, after 3 seconds, both boots did, and the log has `VT changed
from 2 to 1` and `creating new display on seat0 because of VT change`. With that
same forcing and this patch, neither did, and there was no VT change.
`50.1-0ubuntu0.1patrone5` as built went to the installer on all 5 boots with the
machine busy. Installed on a fresh install, it left the login screen, signing in
and out, locking and unlocking, and Remote Desktop's sign-in working as before.

## Building

On the dev VM, with Ubuntu's build dependencies for `gdm3` installed
(`dpkg-checkbuilddeps` in the unpacked source lists any that are missing):

```
./gdm/build.sh
```

It fetches the newest resolute source (`pull-lp-source`, the one step that
needs the network), or a given version (`build.sh 50.1-0ubuntu0.1`). Install
the three packages that go together:

```
sudo apt install ./out/gdm3_*patrone1_amd64.deb ./out/libgdm1_*patrone1_amd64.deb ./out/gir1.2-gdm-1.0_*patrone1_amd64.deb
```

GDM loads it when it next starts, which is at the next boot. An Ubuntu
update of GDM replaces these packages until `build.sh` is run again.

## How it was checked

`50.1-0ubuntu0.1patrone1`, built on the dev VM (2026-10-27): the patch
applied cleanly, the patched file compiled with no warnings, and the built
`/usr/sbin/gdm3` has the new check's message in it. Before building, GDM's
check was run on the VM's real display through the same libgudev calls
`udev_is_settled()` makes: as Ubuntu builds it, it found no primary GPU;
with the patch's rule, it accepted `card1` (`hyperv_drm`) at once. Not yet
checked: a boot with it installed. After one, `journalctl -b -o
short-monotonic | grep -E 'gdm.service|primary GPU|greeter'` should show no
"primary GPU" warning and the login screen's session about a second after
GDM starts.
