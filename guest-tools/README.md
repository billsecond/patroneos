# Patrone's guest tools (Hyper-V and VirtualBox)

`patrone-guest-tools` lives in its own repository since 2026-11-24:
[billsecond/patrone-guest-tools](https://github.com/billsecond/patrone-guest-tools), with its history
from here. It installs on its own too, on Ubuntu 26.04 and systems built
on it.

PatroneOS builds it from a release of that repository:
`packaging/patrone-guest-tools/release.txt` pins the release's source tarball (its
version, address and SHA-256), `packaging/build.sh patrone-guest-tools` builds the
package from it (`packaging/release-source.sh`), and moving the pin on
`main` publishes it to Patrone's apt repository within a day.
