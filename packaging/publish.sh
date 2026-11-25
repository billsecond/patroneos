#!/usr/bin/env bash
# Publishes Patrone's packages to its apt repository, billsecond/patrone-apt
# (https://billsecond.github.io/patrone-apt), where PatroneOS installs get
# their updates (patrone-desktop's /etc/apt/sources.list.d/patrone.sources).
# On the dev VM, where the signing key is.
#
#   ./packaging/publish.sh PACKAGE|DEB...
#   ./packaging/publish.sh --all
#   ./packaging/publish.sh --index
#   ./packaging/publish.sh --dry-run ...
#
# A PACKAGE is a binary package's name: its newest build in out/ is
# published. With each deb come the others built with it (the same source
# and version), so gdm3 brings libgdm1 and gir1.2-gdm-1.0. --all publishes
# the newest build of every package in out/. A package already published at
# that version or newer is skipped; --dry-run only says what would change.
# --index publishes no debs, and only writes the apps component's index
# again if it's out of date (after apps.list changes).
#
# The suite, resolute, has two components. main has every package, and is
# what PatroneOS installs use. apps is for Ubuntu and systems built on it: the
# same index, pointing at the same files, less every package apps.list doesn't
# name, so it never has Patrone's builds of Ubuntu's packages or the
# packages that make a system PatroneOS. A package whose version has
# "patrone" in it (Ubuntu's version plus patroneN: a build of Ubuntu's
# package) is never put in apps; publish.sh stops if apps.list names one.
#
# The debs go to the repository's "pool" release, and the index
# (dists/resolute: each component's Packages, and Release, signed as
# InRelease and Release.gpg) is committed to its main branch. Pushing that
# starts the workflow that puts both on GitHub Pages, a minute or two later.
# Needs gh signed in, the repository cloned in APT_REPO (default
# ~/src/patrone-apt; cloned if missing) and the key in KEYHOME (default
# ~/.local/share/patrone-apt/gnupg). OUT is where builds are looked for
# (default out/ in this checkout).
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
out=${OUT:-$root/out}
repo=${APT_REPO:-$HOME/src/patrone-apt}
github=${APT_GITHUB:-billsecond/patrone-apt}
keyhome=${KEYHOME:-$HOME/.local/share/patrone-apt/gnupg}
suite=resolute
dists=dists/$suite
index=$dists/main/binary-amd64
apps_index=$dists/apps/binary-amd64
apps_list=$root/packaging/apps.list

usage() {
    sed -n '7,10p' "$0" | sed 's/^# *//' >&2
    exit 1
}
dry_run= all= index_only= wanted=()
for arg; do
    case $arg in
        --dry-run) dry_run=1 ;;
        --all) all=1 ;;
        --index) index_only=1 ;;
        -*) usage ;;
        *) wanted+=("$arg") ;;
    esac
done
if [ -n "$index_only" ]; then
    [ -z "$all" ] && [ ${#wanted[@]} -eq 0 ] || usage
else
    [ -n "$all" ] || [ ${#wanted[@]} -gt 0 ] || usage
fi
[ -f "$apps_list" ] || { echo "No $apps_list" >&2; exit 1; }

[ -d "$repo/.git" ] || git clone -q "https://github.com/$github" "$repo"
git -C "$repo" pull -q --ff-only

# The version of package $1 the index has now, if any
published() {
    [ -f "$repo/$index/Packages" ] || return 0
    awk -v p="$1" '$1 == "Package:" { name = $2 } $1 == "Version:" && name == p { print $2 }' \
        "$repo/$index/Packages"
}

# The apps component's index from main's, Packages file $1: its stanzas for
# the packages apps.list names, as they are
apps_packages() {
    awk -v list="$apps_list" '
        BEGIN {
            while ((getline line <list) > 0) {
                sub(/#.*/, "", line)
                gsub(/[ \t]/, "", line)
                if (line != "") listed[line] = 1
            }
            RS = ""; ORS = "\n\n"
        }
        {
            name = version = ""
            n = split($0, lines, "\n")
            for (i = 1; i <= n; i++) {
                if (lines[i] ~ /^Package: /) name = substr(lines[i], 10)
                if (lines[i] ~ /^Version: /) version = substr(lines[i], 10)
            }
        }
        !(name in listed) { next }
        version ~ /patrone/ {
            printf "apps.list names %s, and %s is a build of an Ubuntu package: those never go in apps\n", name, version >"/dev/stderr"
            exit 1
        }
        { print }' "$1"
}

# Every build in out/: its package, version, architecture and source
declare -A pkg_name pkg_version pkg_arch pkg_source newest
shopt -s nullglob
for deb in "$out"/*.deb; do
    IFS=$'\t' read -r pkg_name[$deb] pkg_version[$deb] pkg_arch[$deb] pkg_source[$deb] < <(
        dpkg-deb --showformat='${Package}\t${Version}\t${Architecture}\t${source:Package}\n' -W "$deb")
    n=${pkg_name[$deb]}
    if [ -z "${newest[$n]:-}" ] || dpkg --compare-versions "${pkg_version[$deb]}" gt "${pkg_version[${newest[$n]}]}"; then
        newest[$n]=$deb
    fi
done
shopt -u nullglob

# What was asked for, with the debs built with each
chosen=()
if [ -n "$all" ]; then
    chosen=("${newest[@]}")
fi
for want in "${wanted[@]}"; do
    if [ -f "$want" ]; then
        deb=$(realpath "$want")
        [ -n "${pkg_name[$deb]:-}" ] || {
            IFS=$'\t' read -r pkg_name[$deb] pkg_version[$deb] pkg_arch[$deb] pkg_source[$deb] < <(
                dpkg-deb --showformat='${Package}\t${Version}\t${Architecture}\t${source:Package}\n' -W "$deb")
        }
        chosen+=("$deb")
    elif [ -n "${newest[$want]:-}" ]; then
        chosen+=("${newest[$want]}")
    else
        echo "No build of $want in $out" >&2
        exit 1
    fi
done
for deb in "${chosen[@]}"; do
    for other in "${!pkg_name[@]}"; do
        if [ "${pkg_source[$other]}" = "${pkg_source[$deb]}" ] && [ "${pkg_version[$other]}" = "${pkg_version[$deb]}" ]; then
            chosen+=("$other")
        fi
    done
done

# Of those, the ones newer than what's published, each under its pool name
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/pool/main"
declare -A seen
new=() summary=()
for deb in "${chosen[@]}"; do
    n=${pkg_name[$deb]} v=${pkg_version[$deb]}
    [ -z "${seen[$n]:-}" ] || continue
    seen[$n]=1
    old=$(published "$n")
    if [ -n "$old" ] && dpkg --compare-versions "$v" le "$old"; then
        echo "$n $v: skipped, $old is published"
        continue
    fi
    file=${n}_${v#*:}_${pkg_arch[$deb]}.deb
    if [[ ! $file =~ ^[A-Za-z0-9._-]+$ ]]; then
        # GitHub renames release files with other characters
        echo "$file: a name GitHub would change; publish.sh needs teaching" >&2
        exit 1
    fi
    cp "$deb" "$work/pool/main/$file"
    new+=("$n")
    summary+=("$n $v")
    echo "$n: ${old:-not published} -> $v"
done
if [ ${#new[@]} -eq 0 ] && [ ! -f "$repo/$index/Packages" ]; then
    echo "Nothing new to publish."
    exit 0
fi

# The index: what's published, less the packages replaced, and the new ones;
# and apps' from it. Written before anything is uploaded, so that apps.list
# naming a build of an Ubuntu package stops it with nothing changed.
{
    if [ -f "$repo/$index/Packages" ]; then
        awk -v replaced=" ${new[*]} " 'BEGIN { RS = ""; ORS = "\n\n" }
            { split($0, lines, "\n"); sub(/^Package: /, "", lines[1]) }
            index(replaced, " " lines[1] " ") == 0' "$repo/$index/Packages"
    fi
    [ ${#new[@]} -eq 0 ] || (cd "$work" && apt-ftparchive packages pool/main)
} >"$work/Packages"
apps_packages "$work/Packages" >"$work/apps-Packages"
if [ ${#new[@]} -eq 0 ]; then
    if cmp -s "$work/apps-Packages" "$repo/$apps_index/Packages"; then
        echo "Nothing new to publish."
        exit 0
    fi
    echo "apps: its index is out of date, and is written again"
fi
[ -z "$dry_run" ] || exit 0

# The debs first, so the index never names a file that isn't there. A file
# already in the release must be the same file: a changed build needs a new
# version.
declare -A have
if [ ${#new[@]} -gt 0 ]; then
    gh release view pool -R "$github" >/dev/null 2>&1 ||
        gh release create pool -R "$github" --latest=false --title "Package pool" \
            --notes "The .deb files of PatroneOS's apt repository (https://billsecond.github.io/patrone-apt), which publishes them with its signed index. Not for downloading by hand."
    while IFS=$'\t' read -r asset digest; do
        have[$asset]=$digest
    done < <(gh api "repos/$github/releases/tags/pool" --jq '.assets[] | [.name, (.digest // "")] | @tsv')
    for deb in "$work"/pool/main/*.deb; do
        file=${deb##*/}
        sum=sha256:$(sha256sum "$deb" | cut -d' ' -f1)
        if [ -z "${have[$file]+set}" ]; then
            gh release upload pool -R "$github" "$deb"
            continue
        fi
        if [ -z "${have[$file]}" ]; then
            # No digest from GitHub: compare the file itself
            gh release download pool -R "$github" -p "$file" -D "$work/compare"
            have[$file]=sha256:$(sha256sum "$work/compare/$file" | cut -d' ' -f1)
        fi
        if [ "${have[$file]}" != "$sum" ]; then
            echo "The pool already has a different $file: build it again with a new version" >&2
            exit 1
        fi
    done
fi

mkdir -p "$repo/$index" "$repo/$apps_index"
mv "$work/Packages" "$repo/$index/Packages"
mv "$work/apps-Packages" "$repo/$apps_index/Packages"
for packages in "$repo/$index/Packages" "$repo/$apps_index/Packages"; do
    gzip -9nkf "$packages"
    xz -kf "$packages"
done

rm -f "$repo/$dists/Release" "$repo/$dists/InRelease" "$repo/$dists/Release.gpg"
# The Origin stays as installs first saw it: apt stops updating from a
# repository whose Origin changes until it's told to accept it, and
# desktop/apt/patrone.pref matches on it. The Label is the system's name,
# PatroneOS since 2026-10-29; installs accept that change since
# patrone-desktop 0.2.6 (apt/51patrone-release-label), which had to reach
# them first.
(cd "$repo/$dists" && apt-ftparchive \
    -o APT::FTPArchive::Release::Origin=Patrone \
    -o APT::FTPArchive::Release::Label=PatroneOS \
    -o APT::FTPArchive::Release::Suite=$suite \
    -o APT::FTPArchive::Release::Codename=$suite \
    -o APT::FTPArchive::Release::Architectures=amd64 \
    -o "APT::FTPArchive::Release::Components=main apps" \
    -o "APT::FTPArchive::Release::Description=PatroneOS's packages, for Ubuntu 26.04 LTS" \
    release .) >"$work/Release"
mv "$work/Release" "$repo/$dists/Release"

export GNUPGHOME=$keyhome
key=$(gpg --list-secret-keys --with-colons | awk -F: '$1 == "fpr" { print $10; exit }')
gpg --batch --yes --local-user "$key" --digest-algo SHA512 --clearsign \
    -o "$repo/$dists/InRelease" "$repo/$dists/Release"
gpg --batch --yes --local-user "$key" --digest-algo SHA512 --armor --detach-sign \
    -o "$repo/$dists/Release.gpg" "$repo/$dists/Release"
if ! gpgv --keyring "$repo/patrone-archive-keyring.gpg" "$repo/$dists/InRelease" >/dev/null 2>&1; then
    echo "The index isn't signed by the key installs trust (patrone-archive-keyring.gpg): is KEYHOME right?" >&2
    exit 1
fi

git -C "$repo" add "$dists"
if [ ${#new[@]} -eq 0 ]; then
    git -C "$repo" commit -qm "Write the apps component's index again" \
        -m "$(awk '$1 == "Package:" { print $2 }' "$repo/$apps_index/Packages" | sort)"
    git -C "$repo" push -q
    echo "Published. GitHub Pages has it once this finishes: https://github.com/$github/actions"
    exit 0
fi
mapfile -t new < <(printf '%s\n' "${new[@]}" | sort)
mapfile -t summary < <(printf '%s\n' "${summary[@]}" | sort)
git -C "$repo" commit -qm "Publish $(IFS=,; echo "${new[*]}" | sed 's/,/, /g')" \
    -m "$(printf '%s\n' "${summary[@]}")"
git -C "$repo" push -q

# The release keeps the files of this index and the one before it
keep=$({
    cat "$repo/$index/Packages"
    git -C "$repo" show "HEAD~1:$index/Packages" 2>/dev/null || true
} | awk '$1 == "Filename:" { sub(/.*\//, "", $2); print $2 }' | sort -u)
for asset in "${!have[@]}"; do
    grep -qxF "$asset" <<<"$keep" || gh release delete-asset pool "$asset" -R "$github" --yes
done

echo "Published. GitHub Pages has it once this finishes: https://github.com/$github/actions"
