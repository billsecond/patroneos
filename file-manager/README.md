# Patrone File Manager

Patrone File Manager, the app for folders (`org.patrone.FileManager`); it also draws the desktop and its files panel for the taskbar. `patrone-file-manager` lives in its own repository since 2026-11-24:
[billsecond/patrone-file-manager](https://github.com/billsecond/patrone-file-manager), with its history from here. It installs on its own too, on
Ubuntu 26.04 and systems built on it.

PatroneOS builds it from a release of that repository:
`packaging/patrone-file-manager/release.txt` pins the release's source tarball (its
version, address and SHA-256), `packaging/build.sh patrone-file-manager` builds the
package from it (`packaging/release-source.sh`), and moving the pin on
`main` publishes it to Patrone's apt repository within a day.
