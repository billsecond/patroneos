#!/bin/bash
# Gathers the source code of everything on a PatroneOS disc image, to
# publish beside it (docs/COMPLIANCE.md, item 1: GPL-2 asks for the source
# "from the same place" as the binaries).
#
#   iso/build-source.sh ISO OUT-DIR
#
# Reads the disc's casper/sources.manifest, which iso/build-iso.sh writes,
# and puts in OUT-DIR:
#
# - ubuntu/: Ubuntu's source package of each of the disc's packages, at its
#   exact version, from Launchpad (which keeps every version; Ubuntu's
#   mirrors keep only the newest). Patrone's rebuilds of Ubuntu's packages
#   (versions ending in patroneN) get the version of Ubuntu's they're built
#   from; Patrone's changes to them are in patrone/.
# - patrone/: PatroneOS's repository at the disc's commit (its own
#   packages, its changes to Ubuntu's, and the scripts that make the disc),
#   and the installer's upstream source: Canonical's ubuntu-desktop-provision
#   at the commit installer/build.sh names, and ubuntu_flavor from pub.dev.
# - README.txt, saying what's where, and SHA256SUMS.
#
# Runs as anyone, with a network, in this checkout (which needs the disc's
# commit). Takes about half an hour and 7 GB. Run it again if anything
# failed (missing.txt lists it): what's there already is kept. JOBS (default
# 6) is how many packages it fetches at once.
set -euo pipefail

[ $# = 2 ] || { sed -n '6p' "$0" >&2; exit 2; }
iso=$(realpath "$1")
out=$(realpath -m "$2")
root=$(cd "$(dirname "$0")/.." && pwd)
jobs=${JOBS:-6}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$out/ubuntu" "$out/patrone"

xorriso -osirrox on -indev "$iso" -extract /casper/sources.manifest "$work/sources.manifest" 2>/dev/null
# The repository is billsecond/patroneos since 2026-10-29; discs from before
# then name it billsecond/patrone-linux
commit=$(sed -nE 's,^# https://github.com/billsecond/(patrone-linux|patroneos) at ([0-9a-f]{40}).*,\2,p' "$work/sources.manifest")
if [ -z "$commit" ] || ! git -C "$root" cat-file -e "$commit^{commit}" 2>/dev/null; then
    echo "build-source.sh: the disc's commit (${commit:-not recorded}) isn't in this checkout; fetch it first" >&2
    exit 1
fi
if grep -qE '^# https://github.com/billsecond/(patrone-linux|patroneos) at .*not committed' "$work/sources.manifest"; then
    echo "build-source.sh: the disc was built with changes that were never committed, so its source isn't all in the repository" >&2
    exit 1
fi
short=${commit:0:12}

# Patrone's own source packages, which only its repository has
own=" "
# (an app with a repository of its own has a release.txt instead: its
# source package is named after its folder)
releases=()
for dir in $(git -C "$root" ls-tree -d --name-only "$commit" packaging/); do
    if git -C "$root" cat-file -e "$commit:$dir/release.txt" 2> /dev/null; then
        own+="${dir##*/} "
        releases+=("$dir")
    else
        own+="$(git -C "$root" show "$commit:$dir/debian/control" | sed -n 's/^Source: *//p') "
    fi
done

# Ubuntu's: what to fetch, and Patrone's rebuilds, for the README
: >"$work/fetch"
: >"$work/rebuilds"
awk -F'\t' '$1 == "deb" {print $2 "\t" $3}' "$work/sources.manifest" |
    while IFS=$'\t' read -r name version; do
        case $own in *" $name "*) continue ;; esac
        if [[ $version =~ ^(.*)patrone[0-9]+$ ]]; then
            printf '%s %s: Ubuntu'"'"'s %s, rebuilt by Patrone\n' "$name" "$version" "${BASH_REMATCH[1]}" >>"$work/rebuilds"
            version=${BASH_REMATCH[1]}
        fi
        printf '%s\t%s\n' "$name" "$version" >>"$work/fetch"
    done
sort -u -o "$work/fetch" "$work/fetch"
echo "$(wc -l <"$work/fetch") of Ubuntu's source packages to have"

# fetch NAME VERSION: one source package into ubuntu/, unless it's there and
# whole already
fetch() {
    local dsc="$1_${2#*:}.dsc"
    if [ -s "$out/ubuntu/$dsc" ] && python3 -I "$work/check.py" "$out/ubuntu/$dsc" 2>/dev/null; then
        return
    fi
    if ! (cd "$out/ubuntu" && pull-lp-source --download-only "$1" "$2" >/dev/null 2>&1) ||
        ! python3 -I "$work/check.py" "$out/ubuntu/$dsc" 2>/dev/null; then
        printf '%s\t%s\n' "$1" "$2" >>"$out/missing.txt"
    fi
}
# check.py DSC: every file the .dsc names is beside it, with its checksum
cat >"$work/check.py" <<'EOF'
import hashlib
import sys
from pathlib import Path

from debian.deb822 import Dsc

dsc = Path(sys.argv[1])
with dsc.open(encoding='utf-8') as f:
    files = Dsc(f)['Checksums-Sha256']
for entry in files:
    path = dsc.parent / entry['name']
    if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']:
        sys.exit(f'{path}: missing or changed')
EOF
export -f fetch
export out work
rm -f "$out/missing.txt"
xargs -P "$jobs" -L 1 bash -c 'fetch "$@"' _ <"$work/fetch"

# Patrone's own source, and the installer's upstream source
git -C "$root" archive --format=tar.gz --prefix="patroneos-$short/" \
    -o "$out/patrone/patroneos-$short.tar.gz" "$commit"
# The apps with repositories of their own: the release tarball each
# release.txt pinned at that commit, checked against its SHA-256
app_tarballs=()
for dir in "${releases[@]}"; do
    pin=$(git -C "$root" show "$commit:$dir/release.txt")
    url=$(sed -n 's/^url=//p' <<<"$pin")
    sum=$(sed -n 's/^sha256=//p' <<<"$pin")
    file=$out/patrone/${url##*/}
    if ! echo "$sum  $file" | sha256sum -c --quiet 2> /dev/null; then
        curl -fsSL -o "$file" "$url"
        echo "$sum  $file" | sha256sum -c --quiet ||
            { echo "build-source.sh: $url isn't the release $dir/release.txt pins" >&2; exit 1; }
    fi
    app_tarballs+=("${url##*/}")
done
build=$(git -C "$root" show "$commit:installer/build.sh")
provision=$(sed -n 's/^provision_commit=//p' <<<"$build")
flavor=$(sed -n 's/^flavor_version=//p' <<<"$build")
[ -s "$out/patrone/ubuntu-desktop-provision-$provision.tar.gz" ] ||
    curl -fsSL -o "$out/patrone/ubuntu-desktop-provision-$provision.tar.gz" \
        "https://github.com/canonical/ubuntu-desktop-provision/archive/$provision.tar.gz"
[ -s "$out/patrone/ubuntu_flavor-$flavor.tar.gz" ] ||
    curl -fsSL -o "$out/patrone/ubuntu_flavor-$flavor.tar.gz" \
        "https://pub.dev/api/archives/ubuntu_flavor-$flavor.tar.gz"

cp "$work/sources.manifest" "$out/sources.manifest"
{
    cat <<EOF
The source code of PatroneOS 26.04.1
========================================

Everything on the disc image, as casper/sources.manifest on it lists
(sources.manifest here), built from https://github.com/billsecond/patroneos
at $commit.

ubuntu/
    Ubuntu's source package of each of the disc's packages, at the exact
    version on the disc: its .dsc and the files the .dsc names (unpack one
    with "dpkg-source -x NAME_VERSION.dsc"). Patrone's rebuilds of Ubuntu's
    packages are built from Ubuntu's versions here, with Patrone's changes
    from patrone/:

EOF
    sort -u "$work/rebuilds" | sed 's/^/        /'
    cat <<EOF

patrone/
    patroneos-$short.tar.gz: PatroneOS's repository at that
    commit: Patrone's own packages (packaging/, and the folders it names),
    its changes to Ubuntu's (gdm/, gnome-settings/, gnome-software/,
    gnome-initial-setup/, language-selector/, installer/), and the scripts
    that make the disc (iso/). Each package's
    build is packaging/build.sh; the rebuilds' are their folders' build.sh.
    Rust packages fetch their crates, whose versions Cargo.lock fixes, from
    crates.io.
EOF
    if (( ${#app_tarballs[@]} )); then
        cat <<EOF

    ${app_tarballs[*]}:
    the apps with repositories of their own, each at the release its
    packaging/PACKAGE/release.txt pins: their source and debian/, which
    packaging/build.sh builds the same way.
EOF
    fi
    cat <<EOF

    ubuntu-desktop-provision-$provision.tar.gz: Canonical's installer front
    end at the commit Patrone's installer is built from
    (installer/build.sh, with installer/patch_source.py and
    installer/debrand_l10n.py), and ubuntu_flavor-$flavor.tar.gz, from
    pub.dev, which installer/build.sh renames PatroneOS. Its Dart and
    Flutter packages' versions are in the source's pubspec.lock files.

The snaps on the disc are Canonical's and their publishers', unchanged;
each snap's snap/manifest.yaml lists what it was built from. They are:

EOF
    awk -F'\t' '$1 == "snap" {print "    " $2 ", revision " $3}' "$work/sources.manifest"
    cat <<EOF

SHA256SUMS has the checksum of every file here.
EOF
} >"$out/README.txt"

(cd "$out" && find . -type f ! -name SHA256SUMS -printf '%P\0' | sort -z | xargs -0 sha256sum) >"$work/SHA256SUMS"
mv "$work/SHA256SUMS" "$out/SHA256SUMS"
du -sh "$out"
if [ -s "$out/missing.txt" ]; then
    echo "build-source.sh: $(wc -l <"$out/missing.txt") source packages failed (missing.txt); run it again" >&2
    exit 1
fi
