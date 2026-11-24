#!/usr/bin/env bash
# Turns on GNOME Remote Desktop's own modes so Windows' Remote Desktop
# Connection (mstsc) can connect to this VM with clipboard sharing. This does
# the same thing as Settings → System → Remote Desktop. Patrone's own Remote
# Desktop, where each account signs in once with its own password, is the
# patrone-remote-desktop package instead (see billsecond/patrone-remote-desktop's README); it
# replaces Remote Login on port 3389.
#
#   ./setup-remote-desktop.sh sharing   Desktop Sharing: shows the session you're
#                                       logged in to, at the VM's screen size.
#   ./setup-remote-desktop.sh login     Remote Login: a separate session sized
#                                       to the client window (resizing it
#                                       afterwards not yet confirmed).
#                                       Uses sudo. Signing in ends your console
#                                       session, if you have one.
#
# Run it as yourself, not with sudo. grdctl asks for the RDP username and
# password to set; they're separate from your Linux password. With both modes
# on, Remote Login has port 3389 and Desktop Sharing moves to 3390.
set -euo pipefail

if [[ $EUID -eq 0 ]]; then
    echo "Run this as yourself, not with sudo. It uses sudo where it needs to." >&2
    exit 1
fi

# Makes a self-signed TLS certificate, which RDP requires, unless one exists.
# $2 is the command prefix that runs as the certificate's owner.
make_certificate() {
    local dir=$1 as_owner=$2
    if $as_owner test -f "$dir/rdp-tls.crt"; then
        return
    fi
    $as_owner mkdir -p "$dir"
    $as_owner openssl req -x509 -newkey rsa:4096 -nodes -days 3650 -quiet \
        -subj "/CN=$(hostname)" -keyout "$dir/rdp-tls.key" -out "$dir/rdp-tls.crt"
    $as_owner chmod 600 "$dir/rdp-tls.key"
}

setup_sharing() {
    local dir=~/.local/share/gnome-remote-desktop
    make_certificate "$dir" ""
    grdctl rdp set-tls-cert "$dir/rdp-tls.crt"
    grdctl rdp set-tls-key "$dir/rdp-tls.key"
    grdctl rdp disable-view-only
    echo "Choose a username and password for Desktop Sharing:"
    grdctl rdp set-credentials
    grdctl rdp enable
    systemctl --user enable gnome-remote-desktop.service
    systemctl --user restart gnome-remote-desktop.service
    grdctl status
}

setup_login() {
    local dir=/var/lib/gnome-remote-desktop/certificates
    make_certificate "$dir" "sudo -u gnome-remote-desktop"
    sudo grdctl --system rdp set-tls-cert "$dir/rdp-tls.crt"
    sudo grdctl --system rdp set-tls-key "$dir/rdp-tls.key"
    echo "Choose a username and password for Remote Login (separate from Desktop"
    echo "Sharing's). After connecting, you sign in at the login screen with your"
    echo "normal Linux account."
    sudo grdctl --system rdp set-credentials
    sudo grdctl --system rdp enable
    sudo systemctl enable gnome-remote-desktop.service
    sudo systemctl restart gnome-remote-desktop.service
    sudo grdctl --system status
}

case ${1:-} in
    sharing) setup_sharing ;;
    login) setup_login ;;
    *)
        echo "Usage: $0 sharing|login" >&2
        exit 1
        ;;
esac

echo
echo "Connect from Windows with: mstsc /v:$(hostname).local"
