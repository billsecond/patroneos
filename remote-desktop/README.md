# Patrone Remote Desktop: one desktop per account, on the screen and remotely

Packaged as `patrone-remote-desktop`. Each account gets one desktop, which
you can sit down at or sign in to from Remote Desktop Connection, and it
works the way Windows users expect:

- **Each account has one desktop**, which keeps running whether or not
  anyone is looking at it.
- **The machine's own screen and Remote Desktop are both views of it.**
  Signing in on the screen (the "Patrone Desktop" login session) shows it
  there; signing in with Windows' Remote Desktop Connection shows the same
  desktop, with the same windows, there.
- **Whoever connects last has it.** A Remote Desktop sign-in takes the
  machine's screen back to the login screen; signing in on the screen
  disconnects the Remote Desktop client. Disconnecting leaves the desktop
  running for next time; logging out ends it.
- **Remote Desktop asks once, for the account's own password**, in Remote
  Desktop Connection's own dialog (NLA, as on Windows), and PAM checks it on
  every connection. Remote sessions never lock or blank the screen.
- **Hyper-V's enhanced session** (VMConnect) shows a login screen in its
  window, and then the account's own desktop, the same one, with the
  clipboard, sound and a screen that follows the window.

## Turning on Remote Desktop

**The desktops** (one desktop per account, which the screen, Remote Desktop
and Hyper-V's enhanced session all show) are a switch of their own:

```
sudo patrone-remote-desktop desktops on|off|status
```

On PatroneOS they're on from the start: the package turns them on when
`patrone-desktop` is installed, or arrives later (a dpkg trigger on its
files; not again once an administrator has turned them off with
`desktops off`). Installed on its own, on Ubuntu, they're off, and installing the
package changes nothing: the login screen keeps Ubuntu's sessions, and no
PAM hook, polkit rule, login-screen extension or GDM drop-in of this
package does anything. `patrone-remote-desktop enable` turns them on, and
`disable` turns them off again (except on PatroneOS, or with Hyper-V's
enhanced session on, which uses them too); Virtual Machine Setup turns them
on for the enhanced session. Either way the login screen shows the change
from its next start (a reboot): with the desktops on, "Patrone Desktop" is
its only session. `desktops off` turns off Remote Desktop and the enhanced
session with them. A flag file, `/var/lib/patrone-remote-desktop/desktops-on`,
says they're on: the polkit rule, the login-screen extension, the
screen-lock autostart (`libexec/no-lock`) and the enhanced session's
service (`ConditionPathExists`) each check it.

Before 0.6.0 the desktops were always on. Upgrading keeps them on where
they're used (PatroneOS, or Remote Desktop or the enhanced session turned
on), and gives the login screen its usual sessions back elsewhere.

Remote Desktop is off until an administrator turns it on and adds the
accounts that may use it.

**In Settings** (Patrone's build of GNOME Settings, `gnome-settings/`):
System › Remote Desktop, whose first tab is Remote Desktop, has one switch,
**Remote Desktop**. Turning it on asks for an administrator's password,
then turns Remote Desktop on and lets the account using Settings sign in.
Then it asks for that account's own password once, which it checks and
records what NLA checks, so the account can connect right away. The tab
then shows what to connect to: the computer's name and IP addresses, the
username, and a connection file to save. Turning it off (an
administrator's password again) turns Remote Desktop off for every
account; the accounts stay in the group, so turning it on again needs no
new password. Under "Other Ways to Connect", the tab says what Desktop
Sharing, Secure Shell (SSH) and GNOME's Remote Login do.

**From a terminal:**

```
sudo patrone-remote-desktop enable
sudo adduser NAME remote-desktop
```

An account added this way can connect once it has signed in on the
computer with its password after joining the group (on the login screen,
with `sudo`, over SSH), or changed its password, or confirmed it in
Settings' Remote Desktop tab: that's when Patrone records what NLA checks.

Then connect from Windows with `mstsc /v:HOSTNAME`, and give the Linux
account's username and password in Remote Desktop Connection's dialog. The
first time, it also asks you to trust the computer's certificate, which is
self-signed. A connection file with the computer and username filled in,
full screen, and resizing with the window:

```
patrone-remote-desktop rdp-file > Patrone.rdp     # copy it to Windows, double-click
```
`patrone-remote-desktop status` shows whether it's on and who's in the group;
`patrone-remote-desktop disable` turns it off.

## Hyper-V's enhanced session

When Patrone runs in a Hyper-V VM, VMConnect (Hyper-V's own window onto
the VM) can open an enhanced session: Remote Desktop over a Hyper-V
socket, with the clipboard, sound, and a screen that resizes with the
window. Only the Hyper-V host can open one. Click Connect in VMConnect's
dialog (or "Enhanced session" on its View menu), and the window shows a
login screen in Patrone's style: the computer's name, a username and a
password. Any account that may sign in at the computer signs in there with
its own password, and gets its own desktop, the same one the computer's
screen and Remote Desktop show (so the computer's own screen goes back to
the login screen). It works whether or not Remote Desktop is turned on, and
needs nothing from the remote-desktop group: like the computer's own
screen, it's only for whoever is at the Hyper-V host.

The host has to use the Hyper-V socket transport, which Linux guests need.
With the VM off, in an administrator's PowerShell on the host:

```powershell
Set-VM -VMName NAME -EnhancedSessionTransportType HvSocket
```

`scripts/hyperv/new-test-vm.ps1` does that for new test VMs.

## How it works

```
mstsc ──NLA──▶ freerdp-proxy :3389 ──NLA──▶ ┐
               (user patrone-rdp)           │ the account's GNOME Remote
                 │ patrone-login plugin     │ Desktop, :20000+uid, serving
                 ▼                          │ its one headless GNOME desktop
               auth.sock ─▶ auth (root):    │ (gnome-headless-session@USER)
                 PAM, then prepare-session  │
                                            │
screen: GDM ─▶ "Patrone Desktop" session ───┘
               console: cage + sdl-freerdp, after console.sock ─▶
               auth --console (root): prepare-session
```

1. **The desktop** is GDM's headless GNOME session for the account
   (`gnome-headless-session@USER.service`), served by the account's own
   GNOME Remote Desktop in headless mode on port 20000 + UID. Its
   credentials are a random secret kept in
   `/var/lib/patrone-remote-desktop/secrets`, never the account's password.
   In headless mode GNOME Remote Desktop accepts a new client and then stops
   the one before it, which is the whole "whoever connects last" behavior.
2. **`prepare-session`** (`libexec/prepare-session`, root) gets it ready:
   sets up the account's GNOME Remote Desktop through `grdctl --headless`,
   starts the session unless it's running, and answers with the port and
   the secret. It answers "busy" if the account is signed in on the screen
   with Ubuntu's plain GNOME session instead, which GNOME can't run beside
   this one.
3. **The screen**: the "Patrone Desktop" login session
   (`/usr/share/wayland-sessions/gnome.desktop`, see 6) runs `libexec/session`,
   which on a seat runs `libexec/console` as the account. It asks for its desktop over
   `/run/patrone-remote-desktop/console.sock`, where `auth --console` takes
   the account from the connection itself (`SO_PEERCRED`), and shows it full
   screen with FreeRDP's SDL client inside `cage`, a one-app compositor.
   When the client is disconnected, the session ends and GDM shows the login
   screen. The client runs with no session bus
   (`DBUS_SESSION_BUS_ADDRESS=disabled:`), so SDL doesn't watch the desktop's
   cursor theme. It draws the pointers the desktop sends, so it has no use for
   the theme, and SDL mishandles a change to it on a compositor without the
   cursor shape protocol, such as `cage`: it frees the theme's images but keeps
   them as its default cursor, which it shows at every change of pointer, and
   `cage` drops the client for the image that's gone (found with SDL 3.4.2;
   3.4.18's code is the same).
4. **Remote Desktop**: FreeRDP's proxy listens on port 3389 as the
   unprivileged `patrone-rdp` user (`patrone-remote-desktop.service`, config
   in `/etc/patrone-remote-desktop/proxy.ini`). It requires NLA from
   clients, as Windows does, and checks it against
   `/var/lib/patrone-remote-desktop/sam` (root's, readable by `patrone-rdp`),
   which holds the NT hash of each `remote-desktop` member's password. A
   wrong password fails right there, in Remote Desktop Connection's own
   "logon attempt failed". NLA then hands the server the password itself,
   and the `patrone-login` plugin (`src/patrone-login.c`, required: without
   it the proxy won't start) takes it before anything else in the
   connection can overwrite it. Before the proxy connects anywhere, the
   plugin hands the username and password to `auth` over
   `/run/patrone-remote-desktop/auth.sock`, which only `patrone-rdp` can
   open, and forgets the password. `auth` (`src/auth.c`, root) checks them
   with PAM (`/etc/pam.d/patrone-remote-desktop`: in the `remote-desktop`
   group, UID 1000 or above, the account's own password, the account not
   locked or expired), so a hash that still matches doesn't let in an
   account that's been locked or taken out of the group. Then it unlocks
   the account's login keyring with the password, through
   `pam_gnome_keyring` in the same PAM service, which wipes it afterwards,
   and runs `prepare-session`. The proxy connects to `127.0.0.1` on the
   answered port with NLA, signing in with the secret, and passes the session through:
   display, input, clipboard, audio, and resizing with the client's window.
5. **Keeping the hashes current**: a `pam-auth-update` profile
   (`/usr/share/pam-configs/patrone-remote-desktop`) adds `pam_exec` to
   `common-auth` and `common-password`, after the password check succeeds.
   It runs `auth --remember` as root with the password, which writes the
   account's NT hash to the SAM file if the account is in the
   `remote-desktop` group (and drops it if not). That happens at every
   password sign-in (login screen, `sudo`, SSH, Remote Desktop) and every
   password change, so the hash always matches the current password. Screen
   unlocks run as the account itself, not root, and are skipped. The same
   profile runs `auth --screen-sign-in` after a sign-in on the login screen
   (`gdm-password`, `-fingerprint`, `-smartcard`), which gets the account's
   desktop ready before GDM opens the screen's session: GDM refuses to start
   a background desktop for an account that has a graphical session open,
   and the screen's own session would be one.

6. **The login screen**: GNOME's login screen treats any other running
   desktop of the account as a conflict, and offers to end it. Here that
   would end the very desktop the screen is about to join, so the
   `patrone-login@patroneos.com` extension (`login-screen/`, for the login
   screen only, switched on by `/usr/share/gdm/dconf/96-patrone-remote-desktop`)
   leaves desktops without a seat out of that check. GDM compiles that
   folder into its settings when it starts, and so does the package's
   postinst (`/usr/share/gdm/generate-config`, as GDM's own reload does):
   otherwise a login screen started after an upgrade read the settings from
   before it, and after 0.5.5 renamed the extension, those named an ID that
   was gone. With the desktops on, Ubuntu's own session
   (`ubuntu.desktop`) is moved aside (`dpkg-divert`, by `desktops on`, to
   `*.patrone-hidden`), since a plain GNOME session started on the screen
   would end the account's desktop. Patrone Desktop is
   `gnome.desktop`, because `gnome` is the session GDM falls back to for an
   account whose own choice isn't valid (like a remembered "ubuntu"); without
   it, GDM falls back to the first session file by name, which is GDM's own
   `gnome-greeter` (a bare GNOME session). The same session runs the
   background desktops (`X-GDM-CanRunHeadless`): `libexec/session` runs the
   viewer when the session has a seat, and Ubuntu's GNOME session
   (`gnome-session --session=ubuntu`) when it has none. A `gnome.desktop`
   from any other package is moved aside too. The package installs Patrone
   Desktop's as `/usr/share/patrone-remote-desktop/gnome.desktop`, and
   `desktops on` puts it in `/usr/share/wayland-sessions`; it also installs
   the drop-in for GDM's headless sessions
   (`data/gnome-headless-session-static-user.conf`) in
   `/etc/systemd/system/gnome-headless-session@.service.d/`, and turns on
   the PAM hook (`pam-auth-update --enable`; its profile is off by
   default). `desktops off` takes each away.

7. **Permissions**: polkit, which decides what a session may do without an
   administrator's password, counts a session as local only when it has a
   seat. These desktops have none, so polkit treated them like SSH logins:
   the quick settings' network menu wouldn't open, and shutting down asked
   for a password. `data/patrone-desktops.rules`
   (`/usr/share/polkit-1/rules.d/45-patrone-desktops.rules`) gives a
   graphical session with no seat what a desktop at the machine's own screen
   gets without a password: network control (and, for administrators,
   system-wide connections, as Ubuntu allows), shutting down, restarting
   and sleeping, and mounting drives. It asks logind for the session's type,
   so SSH logins and other text sessions keep the defaults, as does every
   other action. GNOME 50 runs the desktop's shell and services as units of
   the account's own service manager (`org.gnome.Shell@ubuntu.service`, in
   its `session.slice`), outside the login session, so polkit gives them no
   session at all and, until 0.4.4, the rule never matched them: the Wired
   menu still wouldn't open. A process with no session now counts when its
   cgroup is in the account's own `session.slice` and the account's display
   session is a desktop as above. (Another of the account's processes could
   move itself there, but only one the same account already runs.) Since
   0.4.6 so does one in its `app.slice`, where apps opened from the desktop
   and Patrone Software's background service (`gnome-software.service`)
   run, and the rules cover what Patrone Software does: refreshing and
   updating Flatpak apps and apt's lists. Without them a new account's
   first login asked for a password to "update information about software"
   (2026-10-27). Since 0.4.7, installing, removing and updating software
   (Flatpak apps and bundles, apt packages through Patrone Software, snaps)
   asks for a password once, which is remembered for a few minutes and
   covers what the install needs, such as an app's runtimes, as at the
   screen (`auth_admin_keep`). polkit had given each step its own password:
   installing MailCraft asked at least 30 times (2026-10-28). 0.4.6 let
   administrators install with no password, as Flatpak's and Ubuntu's own
   rules do at the screen; the user asked for one.

8. **Settings' switch**: Patrone's GNOME Settings shows its own Remote
   Desktop tab (`gnome-settings/remote-login/`) first in System › Remote
   Desktop wherever `/usr/libexec/patrone-remote-desktop/settings` is
   installed, in place of GNOME's Remote Login tab, whose system daemon
   `enable` turns off (both would listen on port 3389). The tab runs that
   helper through `pkexec`; the helper acts
   only for pkexec's caller (`PKEXEC_UID`), never for an account it's
   given. `data/org.patrone.remote-desktop.policy` picks polkit's action by
   the helper's first argument: `status` and `password` ask nothing,
   `enable` and `disable` need an administrator (`auth_admin_keep`). Each
   allows the same for `any` as for `active`, because polkit counts
   Patrone's seatless desktops as `any`. `status`, `enable` and `disable`
   answer with `on=`, `member=`, `password=` (the SAM file has the
   account's hash) and `others=` lines. `password` hands the password from
   Settings to `auth --confirm`, which checks it with PAM through
   `/etc/pam.d/patrone-remote-desktop`, as a Remote Desktop sign-in does
   (so only a member's password counts), and writes its hash. That's the
   same check `su` makes for an account's own password, which is why it
   needs no administrator. The administrator's password in polkit's own
   dialog can't do it: polkit checks it before `enable` adds the account to
   the group, and the PAM hook keeps hashes for members only.

9. **Hyper-V's enhanced session** (0.5.0):

   ```
   VMConnect ─▶ Hyper-V host ─▶ vsock :3389 ─▶ freerdp-proxy (vmconnect.ini,
                                               user patrone-vmconnect)
       1. ─▶ greeter.sock ─▶ greeter: the login screen ─▶ vmconnect-auth.sock
                                      ─▶ auth --vmconnect (root): PAM, then
                                         prepare-session ─▶ "ok PORT SECRET"
       2. ◀─ redirection (account, port, secret), to the proxy's client only
       3. ─▶ the account's GNOME Remote Desktop on 127.0.0.1:PORT, NLA with
             the secret
   ```

   A second FreeRDP proxy (`patrone-remote-desktop-vmconnect.service`, on
   Hyper-V only, enabled at install) listens on vsock port 3389, where
   VMConnect connects. Its plugin (`src/patrone-vmconnect.c`) refuses
   anything that doesn't come from the host (vsock CID 2). Hyper-V's host
   does TLS (and NLA with the host's own account) with VMConnect itself and
   relays the rest, so that side runs in FreeRDP's VMConnect mode, without
   them. VMConnect brings no username or password, so the plugin connects
   each new connection to the login screen: `libexec/greeter`
   (`src/greeter.c`), one per connection on
   `/run/patrone-remote-desktop/greeter.sock`, which only the proxy's user
   can open. It draws the screen with cairo and Pango in the style of
   Patrone's login screen (the colors of `patrone-gdm.css`, PatroneOS Sans,
   the logo) and sends it as planar-compressed bitmap tiles. Keys come as
   positions, so it reads them with the layout the computer's own screen
   uses (`/etc/default/keyboard`, through xkbcommon). It hands the username
   and password to `auth --vmconnect` over
   `/run/patrone-remote-desktop/vmconnect-auth.sock`, which only that user
   can open, and forgets the password. PAM checks them with
   `/etc/pam.d/patrone-remote-desktop-vmconnect`: an account with UID 1000
   or above, its own password, not locked or expired, and no `/etc/nologin`;
   no remote-desktop group, as at the computer's own screen. Then `auth`
   unlocks the login keyring and `prepare-session` gets the desktop ready,
   as for Remote Desktop.

   On "ok", the login screen sends the proxy a Server Redirection PDU with
   the account, the desktop's port and its secret. The proxy's own client
   takes it, so VMConnect never sees it, and the plugin points that client
   at the account's GNOME Remote Desktop on localhost, with NLA and the
   secret, instead. VMConnect's connection stays up throughout. When the
   desktop is up, the plugin hands VMConnect's side the desktop's settings,
   as the proxy does after its first connection, and from then on the proxy
   passes the session through as for Remote Desktop: graphics, input,
   clipboard, sound and resizing.

   Sound goes over the static channel, rdpsnd (0.5.2). GNOME Remote
   Desktop's sound (FreeRDP's sound server) sends its first message, the
   formats it can play, right after asking VMConnect to open
   `AUDIO_PLAYBACK_DVC`, without waiting for the answer, and the proxy drops
   anything on a dynamic channel that isn't open yet ("channel is not
   opened, dropping packet"). VMConnect never heard the formats, and ten
   seconds later the desktop gave up ("Client did not respond to protocol
   initiation"). So the plugin answers the desktop itself that the channel
   can't be opened, as VMConnect answers for a channel it doesn't have, and
   VMConnect never sees the request; GNOME Remote Desktop then falls back
   to rdpsnd, which the proxy passes on as it comes. The microphone's
   channel (`AUDIO_INPUT`) waits for its answer before sending anything, so
   it isn't affected.

   Then the plugin tells VMConnect that the account has signed in (a Save
   Session Info PDU, "logon", version 2), as Windows does after its own
   sign-in. GNOME Remote Desktop never sends one, and without it VMConnect
   keeps its window one size even though the desktop offers it display
   control (the channel resizing goes through). With it, the window resizes
   and the desktop follows. The plugin follows dynamic channels one by one
   (`Intercept=drdynvc`) and logs whether the desktop offers display control
   and each size VMConnect asks for.

   Two things about Hyper-V's side, found on the dev VM: the host connects
   from vsock address 2, and FreeRDP has handed the socket to its transport
   before plugins see it, so the plugin finds it through the transport's
   event; and the host handles TLS and NLA with VMConnect, so the proxy runs
   VMConnect's side in FreeRDP's VMConnect mode, which its own client must
   not copy (it would negotiate twice with GNOME Remote Desktop, and the
   sign-in would loop back to the login screen).

   Network Remote Desktop is untouched: its proxy, plugin, PAM service and
   socket are separate, and run as another user (`patrone-rdp`). Neither
   proxy's user can ask the other's sign-in check.

`libexec/no-lock` runs at every GNOME login (from `/etc/xdg/autostart`). In a
session with no seat, which is how these desktops run, it turns off GNOME's
screen lock and blanking, after saving the account's own settings, and puts
them back at the next login in a plain GNOME session on the screen. The lock
button and Super+L still lock such a session, and Patrone's GDM unlocks it
(`gdm/README.md`).

## Things to know

- **The screen goes through Remote Desktop too.** Showing the desktop on the
  machine's own screen means encoding and decoding it locally, so video and
  games are slower than in a plain GNOME session. Patrone's own compositor
  will move a session between the screen and Remote Desktop without that.
  Removing the package brings Ubuntu's own session back to the login
  screen.
- **The SAM file holds password hashes**, as Windows' own account database
  does: NLA's NTLM step can't work without the NT hash. It's root's, readable
  only by the proxy's user, and only for `remote-desktop` members. Like
  Windows' hashes, anyone who gets root can use them for NLA logins
  ("pass the hash"); PAM's check of the real password at every connection
  still applies. Kerberos would avoid the hash, on machines joined to a
  domain. Without NLA, Remote Desktop Connection sends no password at all
  (it expects a sign-in screen from the server), which is why Patrone uses
  NLA.
- **A brand-new member can't connect until a password sign-in** records the
  hash, or the account confirms its password in Settings (which asks right
  after turning Remote Desktop on). Changing a password with `usermod -p` or
  by editing `/etc/shadow` skips PAM, so the next password sign-in brings
  the hash up to date.
- **Ubuntu bug 2171951:** GDM's headless session unit runs as a
  DynamicUser that the system bus can't look up, so on Ubuntu 26.04 it never
  starts ("Failed: The connection is closed"). The package's drop-in
  (`gnome-headless-session@.service.d/patrone-static-user.conf`) runs it as
  the `gdm` account; drop it once Ubuntu fixes the unit.
- **The login keyring opens with the sign-in's password.** GDM's autologin,
  which starts the desktop, has no password, so `auth` unlocks it first. A
  keyring whose password isn't the account's still asks for it once per
  desktop. That happens if the account's password was changed outside PAM.
- **The screen doesn't lock by itself yet.** Leaving the machine, sign out
  of the screen's session, or connect from elsewhere.
- **Each account's GNOME Remote Desktop listens on all interfaces**, on its
  own port, because it has no setting to listen on localhost only. It only
  accepts the account's random secret.
- **The enhanced session's login screen is Patrone's own**, drawn to look
  like the login screen (GDM's), not GDM itself: GDM's own remote login
  would start a second desktop for the account. It has a username and a
  password, no list of accounts, and types with the computer's keyboard
  layout, not the host's.
- **The enhanced session can't share the host's drives.** VMConnect offers
  them, but GNOME Remote Desktop has no drive redirection. Copying files
  through the clipboard is the way across.
- **Logs:** `journalctl -u patrone-remote-desktop` (the proxy and plugin),
  `journalctl -t patrone-remote-desktop-auth` (Remote Desktop sign-ins and
  `prepare-session`), `journalctl -t patrone-remote-desktop-console` (the
  screen's requests), `journalctl -u patrone-remote-desktop-vmconnect` (the
  enhanced session's proxy) and `journalctl -t
  patrone-remote-desktop-greeter` (its login screen).

## Building and testing

```
make           # the plugins, the helper and the enhanced session's login screen
make check     # tests that need no root and no real password
```

They need `freerdp3-dev`, `libpam0g-dev`, `libcairo2-dev`,
`libpango1.0-dev`, `libxkbcommon-dev`, `pkgconf` and `python3`, and
`freerdp-proxy` for the config check. `make check` loads the plugin and
plays the proxy's part against a stand-in helper (`tests/fake-auth.py`): no
credentials, a wrong password, a correct one, an account signed in
elsewhere, a garbled answer, and no helper at all. It also gives the real
helper requests it must refuse (Settings' password check among them),
checks that Settings' helper won't run without pkexec and that each polkit
action names it and one of its commands, runs the real proxy on the
shipped config, and runs the polkit rule under gjs with polkit stood in
for (`tests/polkit-rule-test.js`). Signing in for real, on the screen or
through a client, needs the package installed.

The enhanced session is tested end to end without Hyper-V or a real
account: FreeRDP's proxy, through its library with the plugin from the
build (`tests/test-proxy`), the real login screen
(`systemd-socket-activate` runs one per connection, as systemd does), the
stand-in helper, and a stand-in desktop that lets one username and secret in
with NLA, as GNOME Remote Desktop does (`tests/test-desktop`). A client that
connects with no username or password, as VMConnect does
(`tests/test-client`), types a username and a wrong password, then the right
one; the test checks that the login screen signed it in, that the desktop
let in that account with the secret, and that the client stayed connected
throughout, was activated three times (by the proxy, for the login screen,
for the desktop) and was told the account signed in. The stand-in desktop
sends sound as GNOME Remote Desktop does, with FreeRDP's sound server, and
the client takes it on both of sound's channels, as VMConnect does, into
FreeRDP's fake sound device (`TEST_CLIENT_SOUND=1`): the test checks that
the sound arrives over rdpsnd and that the desktop hears it was played. A
second proxy, as shipped, must refuse a client that isn't on a Hyper-V
socket. With
`TEST_CLIENT_SHOTS=PREFIX` the client saves what it shows, and with
`TEST_CLIENT_RESIZE=WxH` it takes the graphics pipeline and display control
and asks for that size, as VMConnect does when its window is resized.

How 0.5.0 was checked (2026-10-28): the tests above. In a Hyper-V VM with
the package installed, a probe from the host over a Hyper-V socket got the
proxy's answer (TLS chosen). With the test client through the proxy and the
login screen to a real account's GNOME Remote Desktop (the password check
stood in for), display control was offered and asking for 1280x800 resized
the desktop from 1024x768. Then on the dev VM, with the host switched to
the Hyper-V socket transport, the user opened VMConnect's enhanced session:
Patrone's login screen, their own password, their own desktop, and, once
the plugin told VMConnect they had signed in, a window that resizes with
the desktop following (VMConnect asked for 1562x885). Not checked: Windows
hosts other than the user's. The user then found the clipboard and resizing
working (2026-10-29), on the dev VM and in a fresh install from the next
ISO, but never any sound.

How 0.5.2's sound fix was checked (2026-10-29): with 0.5.1's plugin, the
test above failed the way the dev VM's journal showed every session: the
proxy dropped the stand-in desktop's first sound message, and the client
got no sound. With the fix, the desktop was told the dynamic channel
couldn't be opened, sent its sound over rdpsnd, and the client received
twenty blocks and confirmed playing each.

0.4.5's Settings helper was tried on the dev VM (2026-10-27), installed, as
root the way pkexec runs it, for a throwaway account with a random
password: `enable` added it to the group, a wrong password was refused
and left no hash, the right one wrote exactly one SAM line, and `disable`
then `enable` stopped and started the proxy. Through `pkexec`, `status`
and `password` ran without a prompt while `enable` asked for
authentication.

The plugin and helper are C because FreeRDP's plugin interface is a C struct
of function pointers with fixed slots, which is awkward to match from Rust.

Windows and Hyper-V are trademarks of the Microsoft group of companies.
