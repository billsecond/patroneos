#!/usr/bin/env bash
# Makes a PatroneOS release from a clean checkout of main on Ubuntu 26.04:
# the disc image, its SHA256SUMS and its source code, for the website's ISO
# and source folders.
#
#   iso/make-release.sh [STEP...]
#
# The steps, in this order, all of them unless some are named:
#
# - check: what the build needs is installed, there's room for it, and this
#   checkout is main as it is on GitHub, with nothing changed (the disc
#   records its commit, and its source is that commit).
# - ubuntu: Ubuntu's 26.04.1 desktop ISO, checked against its SHA256 below.
#   It's downloaded only if UBUNTU_ISO (default
#   ~/iso/ubuntu-26.04.1-desktop-amd64.iso) isn't there already.
# - packages: every one of Patrone's packages, built from this checkout: the
#   installer app (installer/build.sh), each package in packaging/, and
#   Patrone's builds of Ubuntu's GDM, GNOME Settings, GNOME Software,
#   first-login setup and Language Support (from Ubuntu's newest version of
#   each).
# - disc: iso/build-iso.sh, from the packages this run built and nothing
#   else in out/. It needs root, so it runs this script again with sudo. A
#   new disc takes the place of the one in OUT, and that one's source goes.
# - source: iso/build-source.sh. Run again after a failure, it keeps what
#   it has fetched.
#
# Run it as yourself: it builds as you, and uses sudo only for the disc.
# Everything it makes goes in OUT (default /var/tmp/patroneos-26.04.1):
# debs/, logs/, the disc, SHA256SUMS and the source folder. At the end it
# says what to copy to the website. A step that fails stops it, with the end
# of its log (in logs/); fix what it says and run that step and the ones
# after it again. All of it takes about an hour and 30 GB, with a network
# (longer the first time, when Flutter and the crates are fetched too).
set -euo pipefail

release=26.04.1
ubuntu_name=ubuntu-$release-desktop-amd64.iso
ubuntu_url=https://releases.ubuntu.com/$release/$ubuntu_name
# From Ubuntu's https://releases.ubuntu.com/26.04.1/SHA256SUMS
ubuntu_sha256=601e30fbf5d97759367c632e2c33630665039b7e2158fd068403da3ccf1bda1f
disc_name=patroneos-$release-amd64.iso
source_name=patroneos-$release-source
steps=(check ubuntu packages disc source)
# Patrone's builds of Ubuntu's packages: their build scripts, and a package
# each one makes
declare -A rebuilds=(
    [gdm]=gdm3
    [gnome-settings]=gnome-control-center
    [gnome-software]=gnome-software
    [gnome-initial-setup]=gnome-initial-setup
    [language-selector]=language-selector-common
)

here=$(cd "$(dirname "$(realpath "$0")")" && pwd)
root=$(cd "$here/.." && pwd)
out=$(realpath -m "${OUT:-/var/tmp/patroneos-$release}")
ubuntu_iso=$(realpath -m "${UBUNTU_ISO:-$HOME/iso/$ubuntu_name}")
logs=$out/logs
# Rust comes from rustup, which isn't on the PATH of an SSH command
if [ -d "$HOME/.cargo/bin" ]; then PATH=$HOME/.cargo/bin:$PATH; fi

die() { echo "make-release.sh: $*" >&2; exit 1; }
as_yourself() {
    [ "$(id -u)" != 0 ] || die "run the $1 step as yourself, not root: it uses sudo for the disc only"
}

# run WHAT LOG COMMAND...: runs COMMAND with its output in logs/LOG.log, and
# stops with the end of it if it fails
run() {
    local what=$1 log=$logs/$2.log
    shift 2
    echo "$what (log: $log)"
    if ! "$@" >"$log" 2>&1; then
        echo "$what failed. The end of $log:" >&2
        tail -n 25 "$log" | sed 's/^/    /' >&2
        exit 1
    fi
}

check_ubuntu() {
    local sum
    echo "Checking $1"
    sum=$(sha256sum "$1" | cut -d' ' -f1)
    [ "$sum" = "$ubuntu_sha256" ] ||
        die "$1 isn't Ubuntu's $release desktop ISO: its SHA256 is $sum, not $ubuntu_sha256. Delete it and run the ubuntu step again."
}

step_check() {
    local missing=() problems=() words=() need unmet control copy rust have pkg avail
    as_yourself check
    [ "$(. /etc/os-release && echo "${UBUNTU_CODENAME:-${VERSION_CODENAME:-}}")" = resolute ] ||
        problems+=("Ubuntu 26.04 (resolute): this machine is something else")

    # Commands, and the package that has each
    for need in git:git curl:curl sha256sum:coreutils dpkg-buildpackage:dpkg-dev \
        dh:debhelper dch:devscripts pull-lp-source:ubuntu-dev-tools quilt:quilt \
        python3:python3 perl:perl clang:clang cmake:cmake ninja:ninja-build \
        pkg-config:pkgconf xorriso:xorriso mksquashfs:squashfs-tools cpio:cpio \
        unmkinitramfs:initramfs-tools-core snap:snapd rsync:rsync unzip:unzip sudo:sudo; do
        command -v "${need%%:*}" >/dev/null || missing+=("${need#*:}")
    done
    # Flutter's Linux build, for the installer app
    for pkg in libgtk-3-dev liblzma-dev; do
        [ "$(dpkg-query -W -f '${db:Status-Status}' "$pkg" 2>/dev/null)" = installed ] || missing+=("$pkg")
    done
    # Each of Patrone's packages' build dependencies, but Rust's (below)
    copy=$(mktemp)
    # (an app with a repository of its own: its pinned release's control)
    controls=("$root"/packaging/*/debian/control)
    for release in "$root"/packaging/*/release.txt; do
        [ -e "$release" ] || continue
        package=${release%/release.txt}
        package=${package##*/}
        unpacked=$(mktemp -d)
        "$root/packaging/release-source.sh" "$package" unpack "$unpacked"
        cp "$unpacked/debian/control" "$unpacked.control"
        rm -rf "$unpacked"
        controls+=("$unpacked.control")
    done
    for control in "${controls[@]}"; do
        sed -E '/^\s+(cargo|rustc)\b/d' "$control" >"$copy"
        unmet=$(dpkg-checkbuilddeps "$copy" 2>&1 |
            sed -n 's/.*[Uu]nmet build dependencies: //p' | sed 's/ ([^)]*)//g')
        if [ -n "$unmet" ]; then
            read -ra words <<<"$unmet"
            missing+=("${words[@]}")
        fi
    done
    rm -f "$copy"
    # Rust, from rustup, as new as the packages ask
    rust=$(sed -nE 's/^\s+rustc \(>= ([0-9.]+)\).*/\1/p' "${controls[@]}" | sort -V | tail -n 1)
    for control in "${controls[@]}"; do
        case $control in "$root"/*) ;; *) rm -f "$control" ;; esac
    done
    if ! command -v cargo >/dev/null || ! command -v rustc >/dev/null; then
        problems+=("Rust $rust or newer, from rustup (https://rustup.rs)")
    else
        have=$(rustc --version | cut -d' ' -f2)
        dpkg --compare-versions "$have" ge "$rust" ||
            problems+=("Rust $rust or newer: this has $have (rustup update)")
    fi
    # Ubuntu's build dependencies for the packages Patrone rebuilds
    if unmet=$(apt-get -s build-dep gdm3 gnome-control-center gnome-software gnome-initial-setup language-selector 2>&1); then
        while read -r pkg; do missing+=("$pkg"); done < <(sed -n 's/^Inst \([^ ]*\).*/\1/p' <<<"$unmet")
    else
        echo "Note: without Ubuntu's source repositories (deb-src) this can't check what GDM, GNOME Settings,"
        echo "GNOME Software, first-login setup and Language Support need to build. If something is missing, their build stops and says what."
    fi

    # Room for the build: the disc, its source and build-iso.sh's work
    mkdir -p "$out"
    avail=$(df --output=avail -BG "$out" | tail -n 1 | tr -dc 0-9)
    [ "$avail" -ge 30 ] || problems+=("30 GB free for $out: it has ${avail} GB")

    # The commit the disc records, and its source is
    git -C "$root" fetch -q origin main
    if [ "$(git -C "$root" rev-parse HEAD)" != "$(git -C "$root" rev-parse origin/main)" ]; then
        problems+=("a checkout of main as it is on GitHub: $root is at $(git -C "$root" rev-parse --short HEAD), origin/main at $(git -C "$root" rev-parse --short origin/main)")
    fi
    git -C "$root" diff --quiet HEAD ||
        problems+=("nothing changed in $root: git status lists what is")

    if [ ${#missing[@]} -gt 0 ]; then
        problems+=("these packages: sudo apt-get install $(printf '%s\n' "${missing[@]}" | sort -u | tr '\n' ' ')")
    fi
    if [ ${#problems[@]} -gt 0 ]; then
        echo "The build needs:" >&2
        printf '  - %s\n' "${problems[@]}" >&2
        exit 1
    fi
    echo "Everything the build needs is here. Building from $(git -C "$root" rev-parse --short HEAD), into $out"
}

step_ubuntu() {
    local link=$out/$ubuntu_name
    if [ ! -f "$ubuntu_iso" ]; then
        echo "Downloading $ubuntu_url to $ubuntu_iso"
        mkdir -p "$(dirname "$ubuntu_iso")"
        curl -fL --retry 3 -C - -o "$ubuntu_iso.part" "$ubuntu_url"
        mv "$ubuntu_iso.part" "$ubuntu_iso"
    fi
    check_ubuntu "$ubuntu_iso"
    # The disc step, which may run as root with another home, finds it in OUT
    mkdir -p "$out"
    if [ "$ubuntu_iso" != "$link" ]; then
        [ ! -e "$link" ] || [ -L "$link" ] || die "$link is in the way: move it, or run with UBUNTU_ISO=$link"
        ln -sfn "$ubuntu_iso" "$link"
    fi
}

step_packages() {
    local started=$logs/.packages-started changelog debian package source version options script
    as_yourself packages
    rm -rf "$out/debs"
    mkdir -p "$out/debs" "$logs"
    touch "$started"

    # The installer app first: its package (packaging/patrone-installer)
    # holds it
    run "Building the installer app" installer-app "$root/installer/build.sh"
    for changelog in "$root"/packaging/*/debian/changelog; do
        debian=${changelog%/changelog}
        package=${debian%/debian}
        package=${package##*/}
        # The Rust packages: Rust comes from rustup, not Ubuntu's packages
        options=()
        if grep -qE '\b(cargo|rustc)\b' "$debian/control"; then options=(-d); fi
        run "Building $package $(dpkg-parsechangelog -l "$changelog" -S Version)" "$package" \
            "$root/packaging/build.sh" "$package" "${options[@]}"
    done
    # The apps with repositories of their own, from their pinned releases
    for release in "$root"/packaging/*/release.txt; do
        [ -e "$release" ] || continue
        package=${release%/release.txt}
        package=${package##*/}
        run "Building $package $("$root/packaging/release-source.sh" "$package" version)" "$package" \
            "$root/packaging/build.sh" "$package" -d
    done
    for script in "${!rebuilds[@]}"; do
        run "Building Patrone's ${rebuilds[$script]}" "$script" "$root/$script/build.sh"
    done

    # What this run built, and nothing older that's in out/ (a branch's
    # build, say)
    find "$root/out" -maxdepth 1 -name '*.deb' -newer "$started" -exec cp -t "$out/debs" {} +
    for changelog in "$root"/packaging/*/debian/changelog; do
        source=$(dpkg-parsechangelog -l "$changelog" -S Source)
        version=$(dpkg-parsechangelog -l "$changelog" -S Version)
        compgen -G "$out/debs/${source}_${version#*:}_*.deb" >/dev/null || die "no $source $version in $out/debs"
    done
    for release in "$root"/packaging/*/release.txt; do
        [ -e "$release" ] || continue
        source=${release%/release.txt}
        source=${source##*/}
        version=$("$root/packaging/release-source.sh" "$source" version)
        compgen -G "$out/debs/${source}_${version#*:}_*.deb" >/dev/null || die "no $source $version in $out/debs"
    done
    for script in "${!rebuilds[@]}"; do
        version=patrone$(sed -n 's/^revision=//p' "$root/$script/build.sh")
        compgen -G "$out/debs/${rebuilds[$script]}_*${version}_*.deb" >/dev/null ||
            die "no ${rebuilds[$script]} ...$version in $out/debs"
    done
    echo "Built $(find "$out/debs" -name '*.deb' | wc -l) packages, in $out/debs"
}

step_disc() {
    if [ "$(id -u)" != 0 ]; then
        echo "The disc needs root: running the disc step with sudo"
        sudo env OUT="$out" WORK="${WORK:-}" "$here/make-release.sh" disc
        return
    fi
    [ -e "$out/$ubuntu_name" ] || die "no $out/$ubuntu_name: run the ubuntu step first"
    compgen -G "$out/debs/*.deb" >/dev/null || die "no packages in $out/debs: run the packages step first"
    check_ubuntu "$out/$ubuntu_name"
    mkdir -p "$logs"
    # An earlier disc's source goes with it: build-source.sh keeps what's
    # there, so what's no longer on the disc would stay
    rm -rf "$out/$disc_name" "$out/SHA256SUMS" "${out:?}/$source_name"
    run "Building $disc_name" disc "$here/build-iso.sh" "$out/$ubuntu_name" "$out/debs" "$out/$disc_name"
    # As Ubuntu's SHA256SUMS has it: the hash, a space and *name
    (cd "$out" && sha256sum --binary "$disc_name" >SHA256SUMS)
    # Whoever ran the other steps owns the disc too
    chown --reference="$out" "$out/$disc_name" "$out/SHA256SUMS" "$logs/disc.log"
    echo "Built $out/$disc_name"
}

step_source() {
    as_yourself source
    [ -f "$out/$disc_name" ] || die "no $out/$disc_name: run the disc step first"
    mkdir -p "$logs"
    run "Gathering the disc's source code" source "$here/build-source.sh" "$out/$disc_name" "$out/$source_name"
    echo "Gathered $out/$source_name"
}

chosen=("$@")
[ $# -gt 0 ] || chosen=("${steps[@]}")
for step in "${chosen[@]}"; do
    [[ " ${steps[*]} " == *" $step "* ]] || die "no step $step: the steps are ${steps[*]}"
done
for step in "${steps[@]}"; do
    [[ " ${chosen[*]} " == *" $step "* ]] || continue
    echo "== $step"
    "step_$step"
done

if [[ " ${chosen[*]} " == *" source "* ]]; then
    cat <<EOF

PatroneOS $release is ready. For the website, copy:
  to its ISO folder:     $out/$disc_name
                         (and its line in $out/SHA256SUMS)
  to its source folder:  $out/$source_name
EOF
fi
