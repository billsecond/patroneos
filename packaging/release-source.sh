#!/bin/bash
# Patrone's apps that have repositories of their own (Notepad's is
# billsecond/patrone-notepad) are built here from a release of theirs:
# packaging/PACKAGE/release.txt pins it, with its version, its source
# tarball's address and that tarball's SHA-256, as fonts/release.txt pins
# PatroneOS's fonts. Moving the pin on main publishes the release
# (main-updates.sh), and the disc's source has the tarball (build-source.sh).
#
#   packaging/release-source.sh PACKAGE version   the pinned version
#   packaging/release-source.sh PACKAGE tarball   the tarball's path, fetched
#                                                 and checked (out/sources/)
#   packaging/release-source.sh PACKAGE unpack DIR
#                                                 its source in DIR, made
#                                                 afresh (debian/ included)
#
# It stops if the tarball isn't the one pinned, or its debian/changelog
# isn't the pinned version.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
package=${1:?Usage: $0 PACKAGE version|tarball|unpack DIR}
what=${2:?Usage: $0 PACKAGE version|tarball|unpack DIR}
release=$root/packaging/$package/release.txt
[[ -f $release ]] || { echo "$release: no such file ($package is built from this repository)" >&2; exit 1; }

field() {
    local value
    value=$(sed -n "s/^$1=//p" "$release")
    [[ -n $value ]] || { echo "$release has no $1=" >&2; exit 1; }
    echo "$value"
}
version=$(field version)

tarball() {
    local url sum file
    url=$(field url)
    sum=$(field sha256)
    file=$root/out/sources/${url##*/}
    mkdir -p "$root/out/sources"
    if [[ ! -f $file ]] || ! echo "$sum  $file" | sha256sum -c --quiet 2> /dev/null; then
        curl -fsSL "$url" -o "$file.part"
        if ! echo "$sum  $file.part" | sha256sum -c --quiet; then
            rm -f "$file.part"
            echo "$url isn't the release $release pins (its SHA-256 differs)" >&2
            exit 1
        fi
        mv "$file.part" "$file"
    fi
    echo "$file"
}

case $what in
    version)
        echo "$version"
        ;;
    tarball)
        tarball
        ;;
    unpack)
        dir=${3:?Usage: $0 PACKAGE unpack DIR}
        file=$(tarball)
        rm -rf "$dir"
        mkdir -p "$dir"
        tar -C "$dir" --strip-components=1 -xzf "$file"
        have=$(dpkg-parsechangelog -l "$dir/debian/changelog" -S Version)
        if [[ $have != "$version" ]]; then
            echo "${file##*/}: its debian/changelog says $have, but $release pins $version" >&2
            exit 1
        fi
        ;;
    *)
        echo "Usage: $0 PACKAGE version|tarball|unpack DIR" >&2
        exit 1
        ;;
esac
