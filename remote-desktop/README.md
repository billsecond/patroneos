# Patrone Remote Desktop

Patrone Remote Desktop: one desktop per account, on the screen and remotely; on PatroneOS its desktops are on from the start. `patrone-remote-desktop` lives in its own repository since 2026-11-24:
[billsecond/patrone-remote-desktop](https://github.com/billsecond/patrone-remote-desktop), with its history from here. It installs on its own too, on
Ubuntu 26.04 and systems built on it.

PatroneOS builds it from a release of that repository:
`packaging/patrone-remote-desktop/release.txt` pins the release's source tarball (its
version, address and SHA-256), `packaging/build.sh patrone-remote-desktop` builds the
package from it (`packaging/release-source.sh`), and moving the pin on
`main` publishes it to Patrone's apt repository within a day.
