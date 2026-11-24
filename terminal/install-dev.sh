#!/usr/bin/env bash
# Builds Patrone Terminal and installs it for the current user, so it can be
# tried in the dev VM's GNOME session before it's packaged. No sudo needed.
#
#   ./terminal/install-dev.sh            install the app and its launcher entry
#   ./terminal/install-dev.sh --default  also make it GNOME's terminal in place of
#                                        Ubuntu's (Ptyxis), for this user only
#   ./terminal/install-dev.sh --autostart  also open it whenever you log in
#   ./terminal/install-dev.sh --bashrc   also give bash Patrone's prompt
#   ./terminal/install-dev.sh --zshrc    also install Patrone's zshrc as ~/.zshrc
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
# Patrone's shell setup, for --bashrc and --zshrc: a clone of
# billsecond/patrone-shell-config, beside this repository's clone unless
# PATRONE_SHELL_CONFIG says where
config=${PATRONE_SHELL_CONFIG:-$here/../../patrone-shell-config}
make_default=false
autostart=false
install_bashrc=false
install_zshrc=false
for arg in "$@"; do
    case $arg in
        --default) make_default=true ;;
        --autostart) autostart=true ;;
        --bashrc) install_bashrc=true ;;
        --zshrc) install_zshrc=true ;;
        *) echo "Unknown option: $arg" >&2; exit 1 ;;
    esac
done

if { $install_bashrc || $install_zshrc; } && [[ ! -f $config/bashrc ]]; then
    echo "--bashrc and --zshrc need Patrone's shell setup: clone" >&2
    echo "https://github.com/billsecond/patrone-shell-config to $config, or set PATRONE_SHELL_CONFIG" >&2
    exit 1
fi

cargo install --locked --path "$here" --root ~/.local

apps=~/.local/share/applications
mkdir -p "$apps"
sed "s|^Exec=patrone-terminal|Exec=$HOME/.local/bin/patrone-terminal|" \
    "$here/data/org.patrone.Terminal.desktop" > "$apps/org.patrone.Terminal.desktop"
update-desktop-database "$apps" 2>/dev/null || true
icons=~/.local/share/icons/hicolor/scalable/apps
mkdir -p "$icons"
cp "$here/data/org.patrone.Terminal.svg" "$icons/"
echo "Installed ~/.local/bin/patrone-terminal, its icon and its launcher entry."

if $make_default; then
    # Ctrl+Alt+T, "Open in Terminal" and Terminal=true apps go through
    # xdg-terminal-exec, which reads this list and caches its answer.
    echo org.patrone.Terminal.desktop > ~/.config/xdg-terminals.list
    rm -f ~/.cache/xdg-terminal-exec

    # Hide Ubuntu's terminal from the app grid and search. Deleting
    # ~/.local/share/applications/org.gnome.Ptyxis.desktop brings it back.
    if [[ -f /usr/share/applications/org.gnome.Ptyxis.desktop ]]; then
        sed '/^\[Desktop Entry\]$/a NoDisplay=true' \
            /usr/share/applications/org.gnome.Ptyxis.desktop > "$apps/org.gnome.Ptyxis.desktop"
    fi

    # Put Patrone Terminal in the dock in place of Ubuntu's terminal.
    favorites=$(gsettings get org.gnome.shell favorite-apps)
    favorites=$(python3 -c '
import ast, sys
apps = ast.literal_eval(sys.argv[1].removeprefix("@as "))
apps = [app for app in apps if app != "org.gnome.Ptyxis.desktop"]
if "org.patrone.Terminal.desktop" not in apps:
    apps.append("org.patrone.Terminal.desktop")
print(apps)
' "$favorites")
    gsettings set org.gnome.shell favorite-apps "$favorites"
    echo "Patrone Terminal now replaces Ubuntu's terminal: Ctrl+Alt+T, the dock and the app grid."
fi

if $autostart; then
    mkdir -p ~/.config/autostart
    sed "s|^Exec=patrone-terminal|Exec=$HOME/.local/bin/patrone-terminal|" \
        "$here/data/autostart/org.patrone.Terminal.desktop" > ~/.config/autostart/org.patrone.Terminal.desktop
    echo "Patrone Terminal now opens whenever you log in."
fi

if $install_bashrc; then
    mkdir -p ~/.local/share/patrone
    cp "$config/bashrc" ~/.local/share/patrone/bashrc
    # Ubuntu's ~/.bashrc loads ~/.bash_aliases; add the loader to it once.
    if ! grep -qs 'patrone-shell-config' ~/.bash_aliases; then
        if [[ -s ~/.bash_aliases ]]; then
            echo >> ~/.bash_aliases
            sed -n '/^# Patrone/,$p' "$config/bash_aliases" >> ~/.bash_aliases
        else
            cp "$config/bash_aliases" ~/.bash_aliases
        fi
    fi
    echo "Installed Patrone's bash prompt. New terminals use it."
fi

if $install_zshrc; then
    for file in zshrc zprofile; do
        if [[ -e ~/.$file ]] && ! cmp -s ~/.$file "$config/$file"; then
            cp ~/.$file ~/.$file.before-patrone
            echo "Saved the old ~/.$file as ~/.$file.before-patrone."
        fi
        cp "$config/$file" ~/.$file
    done
    echo "Installed ~/.zshrc and ~/.zprofile. To make zsh your login shell: chsh -s /usr/bin/zsh"
fi
