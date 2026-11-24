# Patrone Virtual Machine Setup

At each desktop login in a Hyper-V virtual machine, Virtual Machine Setup
checks what PatroneOS needs there, and if something's missing, shows a
dialog that offers to set it up. Packaged as `patrone-hyperv-setup` (Rust
and GTK 4, app ID `org.patrone.HyperVSetup`); `patrone-desktop` recommends
it, so it's in every install. The user asked for it on 2026-10-28: "When
the PatroneOS starts up, have it check for Hyper-V and if its using
Hyper-V as its host, then provide a prompt with an option to install
support for Hyper-V, also allow a disable future notifications on this
dialog box."

**The name.** Microsoft's trademark guidelines keep its marks, Hyper-V
among them, out of other products' names, while text may say what a
product is for (`docs/COMPLIANCE.md`, item 15). So since 0.2.0 the window,
the app's entry, the icon and the password prompt say "Virtual Machine
Setup" (0.1.x said "Hyper-V Support"), and the text says what it's for.
The dialog ends with "Hyper-V and Windows are trademarks of the Microsoft
group of companies." The package, the program, its units, the polkit action
and the app ID keep "hyperv": renaming them would need a migration (a
transitional package, and moving each account's "Don't show this again"
file and the file below that says Hyper-V has opened an enhanced session).

```
patrone-hyperv-setup              the dialog, whatever's set up
patrone-hyperv-setup --status     what's set up and what isn't, NAME=VALUE
patrone-hyperv-setup --at-login   the dialog only if something's missing (the login unit)
```

## When it shows

`patrone-hyperv-setup.service`, a unit of each account's own service
manager, starts with the desktop (`graphical-session.target`), only:

- in a Hyper-V VM (`ConditionVirtualization=microsoft`): on any other
  computer, nothing runs;
- not in the installer's live session, which has the disc mounted at
  `/cdrom` (`ConditionPathIsMountPoint`). Patrone's boot menu doesn't put
  `boot=casper` on the kernel's command line (casper's initramfs boots the
  disc without it), so that check alone let 0.1.0 show over the installer;
  it stays, for a menu that does;
- unless the account ticked **Don't show this again**: that writes
  `~/.config/patrone-hyperv-setup/dont-show-at-login`, which the unit checks
  for (`ConditionPathExists`), so nothing runs then either.

Then the program checks, and shows nothing when everything's set up. On a
new account's first login it waits for first-login setup to finish. While
Hyper-V hasn't opened an enhanced session with the VM (below), it waits up
to 30 seconds for one first: just after the VM starts, Hyper-V may not have
yet (on the dev VM it did 26 seconds after the service for it started).

Patrone's desktops keep running while nobody's looking at them
(`remote-desktop/README.md`), so "each login" is each time the desktop
starts: after a restart, or after logging out. Installed on a running
system, the unit starts from the first login after the account's service
manager restarts (a restart, or logging out of every session):
`dh_installsystemduser` enables it, but running service managers don't
reload.

## What it checks

| Part | Set up when | If not |
| --- | --- | --- |
| Integration services | `patrone-guest-tools` is installed ([billsecond/patrone-guest-tools](https://github.com/billsecond/patrone-guest-tools)) | **Set Up** installs it |
| Enhanced session, Patrone's side | `patrone-remote-desktop-vmconnect.service` is running: Patrone's login screen in VMConnect, then the account's desktop, with the clipboard, sound and resizing (`remote-desktop/README.md`, "Hyper-V's enhanced session") | **Set Up** installs `patrone-remote-desktop`, or updates it if it's from before 0.5.0, and turns the service on |
| Enhanced session, Windows' side | Hyper-V has opened an enhanced session with the VM | the PowerShell command for it |
| Integration services, Windows' side | the host offers all six (shutdown, time, data exchange, heartbeat, backup, guest services) | the PowerShell command for it |

**Windows' side shows from here only by what Hyper-V does.** The guest
can't read the VM's settings. But Hyper-V offers each integration service
as a VMBus device while it's on for the VM, so one that's off is missing
from `/sys/bus/vmbus/devices` (Guest services, for `Copy-VMFile`, is off for
a new VM). And Hyper-V opens an enhanced session over a Hyper-V socket to
port 3389 only when the VM uses the Hyper-V socket transport, which Linux
needs (new VMs get VMBus, which is for Windows). That connection arrives as
a VMBus device of the port's class, `00000d3d-facb-11e6-bd58-64006a7986d3`;
`62-patrone-hyperv-setup.rules` starts `patrone-hyperv-setup-seen.service`
for it, which writes `/var/lib/patrone-hyperv-setup/enhanced-session`. Until
that file is there, the dialog shows the command. It stays, so once
VMConnect has opened an enhanced session, the dialog doesn't ask again, even
on a day when nobody opens one. A VM whose transport is right but which has
never had an enhanced session (one only ever used over the network) gets
the command too; it does no harm there, and "Don't show this again" ends it.

## The dialog

A row for each part (integration services, enhanced session, the VM's
settings in Hyper-V), each with where it stands. Then, if Windows has
something to change:

```powershell
Set-VM -VMName 'NAME' -EnhancedSessionTransportType HvSocket; Get-VMIntegrationService -VMName 'NAME' | Enable-VMIntegrationService
```

whichever half is needed, with the VM's name (Hyper-V tells the guest,
through Data Exchange, in `/var/lib/hyperv/.kvp_pool_3`; `NAME` and a line
saying to replace it if it doesn't), and a **Copy** button. The transport
can't change while the VM runs, so then it says to shut the VM down first.
`Enable-VMIntegrationService` takes every service the VM has, without
naming them, since Windows translates their names. VMConnect's basic
window copies to the guest but not back, so the dialog says the command may
need typing; it's short for that.

**Set Up** runs `/usr/libexec/patrone-hyperv-setup/apply` through `pkexec`
with what's missing, which asks for an administrator's password once
(`org.patrone.hyperv-setup.policy`, `auth_admin_keep` for any session, since
Patrone's desktops have no seat). It updates apt's lists, installs what's
needed from Patrone's repository and Ubuntu's (`apt-get install`) and turns
the enhanced session's service on. Afterwards the dialog says what's left:
a restart, when it installed something (the integration services' daemons
start as their devices appear, and the login screen's session comes from
Remote Desktop as GDM starts), or Windows' side, with **Shut Down** or
**Restart** (GNOME's own dialog, which asks first). Cancelling the password
dialog goes back to the first page; a failure shows the last of what apt
said, with **Try Again**.

**Don't show this again** is saved when the window closes, whatever button
closed it. To see the dialog again, run `patrone-hyperv-setup` in a
terminal, untick it, and close it; it isn't in the Start menu, which every
computer has, Hyper-V or not.

## Testing

```
patrone-hyperv-setup --pretend-missing=guest-tools,enhanced-session,transport,services --dry-run
```

takes those parts as missing on a VM that has them, and `--dry-run` runs the
helper without `pkexec`, saying what it would do. A dry run's parts count
as set up afterwards, so the dialog goes on to its next page.

## What was checked

On PatroneDev (2026-10-28), where everything is set up, with the missing
parts pretended:

- `--status` found all six integration services and read the VM's name
  from Data Exchange. `udevadm test` on the VMBus channel of a live
  enhanced session matched the rule (`SYSTEMD_WANTS=patrone-hyperv-setup-seen.service`),
  and that unit, started, wrote the file.
- In a hidden GNOME Shell under another account, clicked with a virtual
  pointer (AT-SPI's click didn't reach the buttons), light and dark: each
  page. Set Up with `--dry-run` went to "Setting up Hyper-V support", then
  "One more step, on Windows" with Later and Shut Down. Set Up for real
  brought GNOME Shell's password dialog ("Authentication is required to set
  up Hyper-V support"), which was cancelled, and the dialog went back to its
  first page. Copy put both commands on the clipboard, a line each. Ticking
  "Don't show this again" and closing wrote the file, and `--at-login` then
  exited at once; run by hand, the box was ticked, and unticking it removed
  the file.
- With `enhanced-session` moved aside, `--at-login` showed nothing for 13
  seconds; 4 seconds after it was put back, the dialog showed, with the
  transport set.
- `apply guest-tools enhanced-session` as root: updated apt's lists, found
  the guest tools installed and turned the service on (it was on), in 1.6
  seconds. An unknown part exits 2; without root, 1.
- The login unit, started in the account's own session with everything set
  up: started, and ended without a window.

0.2.0, the new name (2026-10-29), on PatroneDev:

- Installed, the package's description, the app's entry, the icon's title
  and polkit's action say "Virtual Machine Setup" or "this virtual
  machine" (`dpkg-query`, `desktop-file-validate`, `pkaction`).
- In the hidden GNOME Shell at 1024×768, a common size for Hyper-V's basic
  window, where the prompt mostly shows, light and dark: each page has the
  new name and the trademark line. With everything missing, the first page
  fits above the taskbar both with the VM's name and without it (Data
  Exchange's pool hidden in a mount namespace), after the opening line was
  cut to one and the rows made a little tighter. Set Up as a dry run went
  to "Setting up this virtual machine", then "One more step, on Windows";
  with only the enhanced session missing, to "This virtual machine is set
  up". Set Up for real brought GNOME Shell's password dialog, "Authentication
  is required to set up this virtual machine", which was left unanswered.
- A real enhanced session from VMConnect started
  `patrone-hyperv-setup-seen.service` (twice that morning, in the journal),
  which wrote the file again.

Not checked yet: a fresh install, where the prompt really shows (after
first-login setup), Set Up with a password, and Restart and Shut Down. VMs from
`scripts/hyperv/new-test-vm.ps1` have the socket transport and Guest
services on, so nothing shows there; to see the prompt, make the VM with
Hyper-V Manager's defaults, or set the transport back to `VMBus` with the
VM off.

---

Hyper-V and Windows are trademarks of the Microsoft group of companies.
