# Notepad

PatroneOS's plain text editor (`patrone-notepad`, `org.patrone.Notepad`)
lives in its own repository since 2026-11-24:
[billsecond/patrone-notepad](https://github.com/billsecond/patrone-notepad),
with its history from here. It installs on its own too, on Ubuntu 26.04
and systems built on it, and has a Windows build.

PatroneOS builds it from a release of that repository:
`packaging/patrone-notepad/release.txt` pins the release's source tarball
(its version, address and SHA-256), `packaging/build.sh patrone-notepad`
builds the package from it (`packaging/release-source.sh`), and moving the
pin on `main` publishes it to Patrone's apt repository within a day.
`patrone-default-apps` makes it the app for plain text.
