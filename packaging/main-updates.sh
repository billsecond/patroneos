#!/usr/bin/env bash
# Publishes what's merged into main and not yet in Patrone's apt repository,
# so a merge reaches installs within a day without anyone publishing it by
# hand. Two kinds of package:
#
# - Patrone's own (packaging/*/debian/changelog): each whose version on main
#   is newer than the published one is built (packaging/build.sh). An app
#   with a repository of its own has packaging/*/release.txt instead: the
#   release it pins, when newer, is built from its tarball.
# - Patrone's builds of Ubuntu's GDM, GNOME Settings, GNOME Software,
#   first-login setup and Language Support: a newer revision of Patrone's
#   changes in gdm/build.sh, gnome-settings/build.sh, gnome-software/build.sh,
#   gnome-initial-setup/build.sh or language-selector/build.sh is built again
#   from the Ubuntu version already published, or Ubuntu's newest for one
#   never published (Ubuntu's own updates to them are ubuntu-updates.sh's).
#
# Then it publishes what built, all at once, and only the debs it has just
# built, never another build in out/ (packaging/publish.sh).
#
#   ./packaging/main-updates.sh            (on the dev VM)
#   ./packaging/main-updates.sh --check    (only say what's behind)
#
# ubuntu-updates.sh runs it daily, in its clean checkout of main. It runs only
# in a checkout of main as it is on GitHub, with nothing changed.
# patrone-installer isn't built here: it needs installer/build.sh first, by
# hand (installer/README.md). A build that fails, and the installer, are said
# in the output (the journal, for the daily run) and become issues in
# billsecond/patrone-apt, which close here once that version, or a newer one,
# is published. A build or publish that fails also ends the run with an error.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
repo=${APT_REPO:-$HOME/src/patrone-apt}
github=${APT_GITHUB:-billsecond/patrone-apt}
logs=${LOGS:-$root/out/logs}
check_only=
[ "${1:-}" != --check ] || check_only=1
index=$repo/dists/resolute/main/binary-amd64/Packages
# Rust comes from rustup on the dev VM, which isn't on the PATH of an SSH
# command or a systemd service
if [ -d "$HOME/.cargo/bin" ]; then PATH=$HOME/.cargo/bin:$PATH; fi

# Only main, as it is on GitHub
git -C "$root" fetch -q origin main
if [ "$(git -C "$root" rev-parse HEAD)" != "$(git -C "$root" rev-parse origin/main)" ] ||
    ! git -C "$root" diff --quiet HEAD; then
    echo "$root isn't main as it is on GitHub, with nothing changed: not publishing from it" >&2
    exit 1
fi

mkdir -p "$logs"
exec 8>"$logs/.main-updates.lock"
flock -n 8 || { echo "Already running" >&2; exit 0; }

# An issue in the apt repository, unless one with that title is open
flag() {
    local title=$1 body=$2
    if [ -n "$(gh issue list -R "$github" --state open --search "\"$title\" in:title" \
        --json title --jq ".[] | select(.title == \"$title\") | .title")" ]; then
        return
    fi
    gh issue create -R "$github" --title "$title" --body "$body" >/dev/null
    echo "Opened an issue: $title"
}

# The newest published version of each source package
declare -A published
read_index() {
    published=()
    [ -f "$index" ] || return 0
    local source version
    while read -r source version; do
        if [ -z "${published[$source]:-}" ] || dpkg --compare-versions "$version" gt "${published[$source]}"; then
            published[$source]=$version
        fi
    done < <(awk '
        $1 == "Package:" { package = $2 }
        $1 == "Source:" { source = $2 }
        $1 == "Version:" { version = $2 }
        /^$/ { if (package) print (source ? source : package), version; package = source = version = "" }
        END { if (package) print (source ? source : package), version }' "$index")
}

[ -d "$repo/.git" ] || git clone -q "https://github.com/$github" "$repo"
git -C "$repo" pull -q --ff-only
read_index

failed=0
behind=0
debs=()
built=()
# Builds SOURCE at VERSION with the command that follows, and keeps its deb
# to publish; a build that fails is said and flagged. With no VERSION (a
# rebuild never published before), it's the newest build of SOURCE in out/.
# The deb may be named after the source (gdm3) or start with it
# (language-selector-common).
build() {
    local source=$1 version=$2 deb
    shift 2
    local log=$logs/$source-${version#*:}.log
    [ -n "$version" ] || log=$logs/$source-new.log
    echo "Building $source ${version:-at the newest Ubuntu version} (log: $log)"
    if "$@" >"$log" 2>&1; then
        if [ -z "$version" ]; then
            version=$(for deb in "$root"/out/"$source"*.deb; do
                dpkg-deb --showformat='${source:Package}\t${Version}\n' -W "$deb"
            done | awk -F'\t' -v s="$source" '$1 == s {print $2}' | sort -V | tail -n 1)
        fi
        for deb in "$root/out/${source}_${version#*:}_"*.deb "$root/out/${source}-"*"_${version#*:}_"*.deb; do
            if [ -f "$deb" ]; then
                debs+=("$deb")
                built+=("$source $version")
                return
            fi
        done
        echo "The build left no ${source}_${version#*:} deb in out/" >>"$log"
    fi
    echo "$source $version didn't build. The end of its log:"
    tail -n 20 "$log" | sed 's/^/    /'
    flag "$source $version from main: didn't build" \
        "packaging/main-updates.sh on the dev VM couldn't build $source $version, which is on main and newer than what Patrone's apt repository has. The log is $log on the VM; its end:

\`\`\`
$(tail -n 40 "$log")
\`\`\`

Installs keep the version that's published until this one, or a newer one, is. Then this issue closes by itself."
    failed=1
}

# Patrone's own packages
for changelog in "$root"/packaging/*/debian/changelog; do
    debian=${changelog%/changelog}
    package=${debian%/debian}
    package=${package##*/}
    source=$(dpkg-parsechangelog -l "$changelog" -S Source)
    version=$(dpkg-parsechangelog -l "$changelog" -S Version)
    have=${published[$source]:-}
    if [ -n "$have" ] && dpkg --compare-versions "$version" le "$have"; then
        continue
    fi
    echo "$source: main has $version, the repository ${have:-none}"
    behind=$((behind + 1))
    [ -z "$check_only" ] || continue
    if [ "$package" = patrone-installer ]; then
        flag "$source $version from main: build it by hand" \
            "$source $version is on main and newer than what Patrone's apt repository has. The daily job doesn't build it, because it needs installer/build.sh first: on the dev VM, build it as installer/README.md says, then \`./packaging/build.sh patrone-installer\` and \`./packaging/publish.sh patrone-installer\`. This issue closes by itself once it's published."
        continue
    fi
    # The Rust packages: Rust comes from rustup, not Ubuntu's packages
    options=()
    if grep -qE '\b(cargo|rustc)\b' "$debian/control"; then options=(-d); fi
    build "$source" "$version" nice -n 19 "$root/packaging/build.sh" "$package" "${options[@]}"
done

# Patrone's apps with repositories of their own: the release each
# packaging/PACKAGE/release.txt pins (release-source.sh), built from its
# tarball. Moving a pin on main publishes it.
for release in "$root"/packaging/*/release.txt; do
    [ -e "$release" ] || continue
    package=${release%/release.txt}
    package=${package##*/}
    source=$package
    version=$("$root/packaging/release-source.sh" "$package" version)
    have=${published[$source]:-}
    if [ -n "$have" ] && dpkg --compare-versions "$version" le "$have"; then
        continue
    fi
    echo "$source: main pins $version, the repository ${have:-none}"
    behind=$((behind + 1))
    [ -z "$check_only" ] || continue
    # Rust comes from rustup, not Ubuntu's packages, so the Build-Depends
    # check is skipped (the release's own CI builds it with Ubuntu's)
    build "$source" "$version" nice -n 19 "$root/packaging/build.sh" "$package" -d
done

# Patrone's builds of Ubuntu's packages: a new revision of Patrone's changes
declare -A rebuild=(
    [gdm3]=gdm/build.sh
    [gnome-control-center]=gnome-settings/build.sh
    [gnome-software]=gnome-software/build.sh
    [gnome-initial-setup]=gnome-initial-setup/build.sh
    [language-selector]=language-selector/build.sh
)
for source in "${!rebuild[@]}"; do
    have=${published[$source]:-}
    revision=$(sed -n 's/^revision=//p' "$root/${rebuild[$source]}")
    if [ -z "$have" ]; then
        # Never published (Language Support, before its first): built from
        # Ubuntu's newest version for resolute, as its build script does by
        # itself
        echo "$source: main has revision $revision of Patrone's changes, the repository none"
        behind=$((behind + 1))
        [ -z "$check_only" ] || continue
        build "$source" "" nice -n 19 "$root/${rebuild[$source]}"
        continue
    fi
    [[ $have == *patrone* ]] || continue
    [ "$revision" -gt "${have##*patrone}" ] || continue
    version=${have%patrone*}patrone$revision
    echo "$source: main has revision $revision of Patrone's changes, the repository $have"
    behind=$((behind + 1))
    [ -z "$check_only" ] || continue
    build "$source" "$version" nice -n 19 "$root/${rebuild[$source]}" "${have%patrone*}"
done

if [ "${#debs[@]}" -gt 0 ]; then
    log=$logs/publish-$(date +%F-%H%M%S).log
    if "$root/packaging/publish.sh" "${debs[@]}" >"$log" 2>&1; then
        cat "$log"
    else
        echo "Publishing failed. The end of its log ($log):"
        tail -n 20 "$log" | sed 's/^/    /'
        for item in "${built[@]}"; do
            flag "$item from main: didn't publish" \
                "$item built on the dev VM, but packaging/publish.sh couldn't publish it. The log is $log on the VM; its end:

\`\`\`
$(tail -n 20 "$log")
\`\`\`

The next daily run tries again. This issue closes by itself once it's published."
        done
        failed=1
    fi
fi

# Close the issues about what's published now
if [ -z "$check_only" ]; then
    read_index
    gh issue list -R "$github" --state open --search '"from main" in:title' \
        --json number,title --jq '.[] | "\(.number) \(.title)"' |
        while read -r number source version rest; do
            [[ $rest == "from main:"* ]] || continue
            have=${published[$source]:-}
            if [ -n "$have" ] && dpkg --compare-versions "$have" ge "$version"; then
                gh issue close -R "$github" "$number" --comment "Published: $source $have." >/dev/null
                echo "Closed issue #$number: $source $have is published"
            fi
        done
fi
[ "$behind" -gt 0 ] || echo "Everything on main is published"
exit "$failed"
