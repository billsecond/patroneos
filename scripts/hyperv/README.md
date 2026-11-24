# Hyper-V helpers (Windows host)

PowerShell scripts for driving a Hyper-V VM from the Windows host without
touching its console window. They use Hyper-V's WMI API, so run them from an
elevated PowerShell 7 prompt. They're useful for scripted install tests of
Patrone ISOs in throwaway VMs.

| Script | What it does |
| --- | --- |
| `vmshot.ps1` | Saves a PNG of the VM's screen at its current resolution. |
| `vminput.ps1` | Types text, presses keys and key combos, and clicks at screen coordinates. `-Info` prints the VM's resolution. |
| `waitssh.ps1` | Finds a VM on the LAN by MAC address and waits until its SSH port opens. |

Examples:

```
.\vminput.ps1 -VMName PatroneDev -Info
.\vmshot.ps1 -VMName PatroneDev -Out screen.png
.\vminput.ps1 -VMName PatroneDev -Combo Ctrl+Alt+T
.\vminput.ps1 -VMName PatroneDev -Text "uname -a" -Keys Enter
.\vminput.ps1 -VMName PatroneDev -Click 942,728 -ShotWidth 1024 -ShotHeight 768
```

Notes:

- `vminput.ps1` types one key at a time with a US keyboard layout. Hyper-V's
  own `TypeText` method drops and garbles characters, and the stray keys can
  trigger desktop shortcuts.
- If the VM console window was opened from an elevated process, desktop
  automation tools can't send input to it, but these scripts still work.

## Inside the guest: clipboard and resizing

Hyper-V's basic console has no clipboard sharing and a fixed screen size.
Both come from RDP. GNOME 50 has no X11 session, so the xrdp recipe from
Microsoft's linux-vm-tools can't work. GNOME Remote Desktop is the RDP server
instead. Two things about it matter:

- It only accepts connections that use network-level authentication (NLA).
- It resizes the remote screen only when it gives the client a virtual
  monitor: in **Remote Login** sessions, or in Desktop Sharing's "extend"
  mode. Desktop Sharing normally mirrors the VM's screen at its current size.

### Remote Desktop Connection

**Patrone's own Remote Desktop** is the `patrone-remote-desktop` package:
each account signs in once, with its own Linux username and password, and
lands on its own desktop, which never locks while connected. See
[billsecond/patrone-remote-desktop](https://github.com/billsecond/patrone-remote-desktop). It takes port 3389 in place of Remote Login:

```
sudo patrone-remote-desktop enable
sudo adduser $USER remote-desktop
```

After that, sign in once with your password on the VM (the login screen,
`sudo`, or SSH with a password) so Patrone records what NLA checks. Then
connect with `mstsc /v:DevDistro-PatroneLinux.local`, or double-click
`PatroneDev.rdp` (in this folder), which fills in the address and username
and opens full screen.

GNOME Remote Desktop's own modes are below, for comparison and for plain
Ubuntu. Nothing listens on port 3389 until one is configured, so Remote
Desktop Connection can't reach the VM. In the VM, run one of these as
yourself (not with sudo). Each asks you to choose an RDP username and
password:

```
./scripts/hyperv/guest/setup-remote-desktop.sh sharing   # Desktop Sharing
./scripts/hyperv/guest/setup-remote-desktop.sh login     # Remote Login (uses sudo)
```

- **Desktop Sharing** shows the session you're logged in to, at the VM's
  screen size. Clipboard works; resizing doesn't.
- **Remote Login** starts a separate session on a virtual monitor the size of
  the client window (3840×2160 on a 4K screen). It should follow the window
  when it's resized (GNOME Remote Desktop's source supports that), but that
  hasn't been confirmed yet. After the RDP password you sign in at the login screen with
  your Linux account. If you're already logged in on the VM's console, that
  console session is ended. The new session starts at 100% scale; set 200% in
  Settings → Displays on a 4K screen.

Then on Windows, run `mstsc /v:DevDistro-PatroneLinux.local` (or the VM's IP
address). With both modes on, Remote Login has port 3389 and Desktop Sharing
moves to 3390 (`mstsc /v:DevDistro-PatroneLinux.local:3390`).

The modes have separate RDP usernames and passwords. Remote Desktop
Connection remembers credentials per address, so after switching modes it
may offer the wrong ones: pick **More choices → Use a different account**.
GNOME Remote Desktop logs "Could not find user in SAM database" when the
username doesn't match.

Settings → System → Remote Desktop does the same thing through the GUI.

### Hyper-V enhanced session

`patrone-remote-desktop` 0.5.0 serves VMConnect's enhanced session on vsock
port 3389: Patrone's login screen, then the account's own desktop
([billsecond/patrone-remote-desktop](https://github.com/billsecond/patrone-remote-desktop)). The host needs the Hyper-V socket transport,
with the VM shut down:

```
Set-VM -VMName PatroneDev -EnhancedSessionTransportType HvSocket
```

`guest/setup-enhanced-session.sh` and its `patrone-hyperv-rdp-bridge.service`
are from before that, a bridge to GNOME's own Remote Login that never worked
end to end; Patrone Remote Desktop's service stops the bridge if both are
installed.
