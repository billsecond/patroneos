# Patrone Terminal

The default terminal of PatroneOS: GTK4 and VTE, written in Rust.

- **Copy on select:** selected text goes to the clipboard as soon as you stop
  selecting.
- **Paste on right-click.** Middle-click pastes the last selection, as usual
  on Linux.
- **Links:** Ctrl+click opens web addresses, and links that programs mark with
  the OSC 8 escape sequence. They underline when the pointer is over them.
- **Escape clears the command line** at a shell's prompt, as in Windows'
  terminals (the user asked for it, 2026-10-27): the terminal sends the
  shell Ctrl+E then Ctrl+U, which bash, zsh and fish all take. Only while a
  shell waits at its prompt in front, so vim, less, htop and mc still get
  Escape: that's when the PTY's foreground process group is a shell's own
  (a shell gives each command a group of its own). sudo runs its command on
  a PTY of its own, so for a root shell from `sudo su` the terminal looks at
  what's in front on sudo's, through `/proc`. A switch in Preferences turns
  it off, for a shell's vi mode or Escape as Alt (Escape then `.`).
- **Asks before closing** a tab or window where a program other than the
  shell is still running, and names the program.
- **Patrone Dark:** a near-black theme with the Patrone blues, in PatroneOS
  Mono, PatroneOS's own monospace font.
- **See-through background**, as in Kali's terminal: the desktop shows
  faintly behind the text, 15% by default (the user asked for it,
  2026-10-26). The title bar and tab bar stay solid. GTK's built-in theme
  paints the scrolled window and the `vte-terminal` node, so `theme::CSS`
  clears those along with the window.
- **Preferences** (menu, or Ctrl+Comma): font, background transparency
  (0%, solid, to 60%), lines of scrollback, and switches for copy on select,
  paste on right-click and Escape clearing the command line. They're saved in
  `~/.config/patrone-terminal/settings.ini` and apply to open terminals at
  once.
- Tabs, 100,000 lines of scrollback, and xterm-style `-e` and `-T` options so
  it can be the `x-terminal-emulator` alternative. Its priority there is 30,
  under Ubuntu's own terminals (Ptyxis and GNOME Terminal, 40), so installing
  it on Ubuntu doesn't change which terminal other programs open; PatroneOS
  has no other terminal, and a Patrone Desktop session picks it through
  `xdg-terminal-exec`.

| Keys | Action |
| --- | --- |
| Ctrl+Shift+C / Ctrl+Shift+V | Copy / paste |
| Shift+Insert | Paste |
| Ctrl+click | Open a link |
| Escape | Clear the command being typed (at a shell's prompt) |
| Ctrl+Shift+T / Ctrl+Shift+N | New tab / new window, in the current directory |
| Ctrl+Shift+W | Close tab |
| Ctrl+Page Down / Ctrl+Page Up | Next / previous tab |
| Ctrl+Plus / Ctrl+Minus / Ctrl+0 | Bigger / smaller / normal text |
| Ctrl+Comma | Preferences |

New tabs open in the current tab's directory when the shell reports it with
the OSC 7 escape sequence. Patrone's prompts for bash and zsh do.

All windows normally share one process: running `patrone-terminal` again
opens a window in the one already running. `--standalone` (`-s`) starts a
separate process, for example inside a nested compositor.

## Build and try it

On the dev VM (the libraries come from `scripts/bootstrap-dev-vm.sh`):

```
cargo run --manifest-path terminal/Cargo.toml
cargo test --manifest-path terminal/Cargo.toml
```

To install it for your user, with a launcher entry and icon in GNOME:

```
./terminal/install-dev.sh             # the app
./terminal/install-dev.sh --default   # and use it instead of Ubuntu's terminal
./terminal/install-dev.sh --autostart # and open it whenever you log in
./terminal/install-dev.sh --bashrc    # and give bash Patrone's prompt
./terminal/install-dev.sh --zshrc     # and use Patrone's zshrc and zprofile
```

This copy is in the account (`~/.local/bin/patrone-terminal`, with its own
launcher, autostart entry and icon in `~/.local/share` and
`~/.config/autostart`), and it wins over the package's: once
`patrone-terminal` is installed, move those four files away, or the old
build keeps running. That happened on the dev VM on 2026-10-26: 0.1.3 was
installed but the account's copy from that morning ran, so the user saw no
transparency and the old icon. They were moved to
`~/.local/share/patrone-dev-backup/terminal/`.

`--default` points Ctrl+Alt+T and "Open in Terminal" at Patrone Terminal
(through `~/.config/xdg-terminals.list`), swaps it into the dock, and hides
Ubuntu's terminal (Ptyxis) from the app grid. Deleting
`~/.local/share/applications/org.gnome.Ptyxis.desktop` brings that back.

`--autostart` writes `~/.config/autostart/org.patrone.Terminal.desktop` from
`data/autostart/`, so GNOME opens a terminal at every login. The package
installs the same entry for everyone as
`/etc/xdg/autostart/org.patrone.Terminal.desktop`. Its script stops if
"Terminal at sign-in" in Patrone Settings is Off (the taskbar's
`sign-in-terminal` setting), and where there's no taskbar at all: on
Ubuntu, with the terminal installed on its own, it opens only when it's
asked for. GNOME's login screen
doesn't run autostart entries, and a Patrone Desktop session opens its own
terminal from the compositor.

The package is built with `packaging/build.sh patrone-terminal` (see
`packaging/README.md`). The crate keeps `rust-version` at Ubuntu's rustc
(1.93), which builds the package.

To try it without touching a desktop session, GTK's Broadway backend draws it
in a web page: run `gtk4-broadwayd --port 8087 :7`, start the terminal with
`GDK_BACKEND=broadway BROADWAY_DISPLAY=:7`, and open `http://localhost:8087`
(tunnel the port over SSH from Windows). Mouse input works fully; letter keys
typed by browser automation don't arrive.

## The shell

The terminal runs your login shell. In Patrone that's **bash**, with a
two-line prompt in the Patrone colors, blue for normal users and red for
root:

```
┌──(user㉿host)-[~/src]
└─$
```

A failed command's exit status shows on its own line under the command, and the
prompt reports the directory to the terminal. It's `bashrc` in [billsecond/patrone-shell-config](https://github.com/billsecond/patrone-shell-config),
loaded from `~/.bash_aliases` (its `bash_aliases`), which Ubuntu's
own `~/.bashrc` already reads, so Ubuntu's bash setup stays as it is. Both
ship in `patrone-shell-config`.

New accounts get these from `/etc/skel`, but root's account comes before
the package, so installing it copies `.bash_aliases` and `.zshrc` to
`/root` too, unless root has them already. Before that (0.2.0), a root
shell from `sudo su`, `sudo -i` or `su -` read only Ubuntu's root
`.bashrc`, whose plain `root@host:dir#` prompt Patrone Dark draws in its
pale blue text color, with the blue cursor after it: the user saw a blue
prompt where root's should be red (2026-10-27). What each way to root
reads:

| Command | Root's shell | Reads |
| --- | --- | --- |
| `sudo su`, `sudo bash` | bash | `/etc/bash.bashrc`, `/root/.bashrc` → `/root/.bash_aliases` |
| `sudo -i`, `su -`, `sudo su -` | login bash | `/etc/profile` (and `/etc/bash.bashrc`, `/etc/profile.d`), `/root/.profile` → `/root/.bashrc` → `/root/.bash_aliases` |
| `sudo -s` from zsh | zsh, `HOME=/root` | `/etc/zsh/zshrc`, `/root/.zshrc` |

sudo (Ubuntu's is sudo-rs) resets the environment, so nothing of the
user's own prompt comes along.

zsh is there for those who want it: its `zshrc` has the same prompt
plus completion menus, shared history, `zsh-autosuggestions` and
`zsh-syntax-highlighting`, and its `zprofile` makes login shells read
`~/.profile`, where Ubuntu keeps PATH additions such as `~/.local/bin`. To
switch: `chsh -s /usr/bin/zsh`; to switch back: `chsh -s /bin/bash`.
