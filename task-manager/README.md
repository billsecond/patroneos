# Patrone Task Manager

Patrone Task Manager (`org.patrone.TaskManager`); on PatroneOS, `patrone-desktop` gives it Ctrl+Shift+Esc and takes Resources' place with it. `patrone-task-manager` lives in its own repository since 2026-11-24:
[billsecond/patrone-task-manager](https://github.com/billsecond/patrone-task-manager), with its history from here. It installs on its own too, on
Ubuntu 26.04 and systems built on it.

PatroneOS builds it from a release of that repository:
`packaging/patrone-task-manager/release.txt` pins the release's source tarball (its
version, address and SHA-256), `packaging/build.sh patrone-task-manager` builds the
package from it (`packaging/release-source.sh`), and moving the pin on
`main` publishes it to Patrone's apt repository within a day.
