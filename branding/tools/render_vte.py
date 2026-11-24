"""Draws terminal text, escape codes and all, the way Patrone Terminal does
(VTE, PatroneOS Mono, Patrone's colors), and saves it as a PNG: to see the
fastfetch logo as it really looks, and to measure the shapes VTE draws for
make_terminal_logo.py.

    render_vte.py TEXT_FILE OUT.png COLUMNS ROWS [FONT]

It needs a Wayland display; with none, run it inside a headless cage:

    WLR_BACKENDS=headless WLR_RENDERER=pixman GSK_RENDERER=cairo \\
        cage -- python3 render_vte.py logo.ansi logo.png 80 24
"""

import sys

import gi

gi.require_version("Gdk", "4.0")
gi.require_version("Gtk", "4.0")
gi.require_version("Vte", "3.91")
from gi.repository import Gdk, GLib, Gtk, Pango, Vte  # noqa: E402

# Patrone Terminal's src/theme.rs (billsecond/patrone-terminal)
FONT = "PatroneOS Mono 11"
FOREGROUND = "#D6DEEB"
BACKGROUND = "#070A10"


def rgba(hex_color: str) -> Gdk.RGBA:
    color = Gdk.RGBA()
    color.parse(hex_color)
    return color


def main() -> None:
    text_path, out_path, columns, rows = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
    font = sys.argv[5] if len(sys.argv) > 5 else FONT

    def activate(app: Gtk.Application) -> None:
        window = Gtk.ApplicationWindow(application=app)
        terminal = Vte.Terminal()
        terminal.set_font(Pango.FontDescription.from_string(font))
        terminal.set_size(columns, rows)
        terminal.set_colors(rgba(FOREGROUND), rgba(BACKGROUND), None)
        window.set_child(terminal)
        text = open(text_path, "rb").read().replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
        # Without the cursor
        terminal.feed(b"\x1b[?25l" + text)
        window.present()

        def save() -> bool:
            snapshot = Gtk.Snapshot()
            Gtk.WidgetPaintable.new(terminal).snapshot(snapshot, terminal.get_width(), terminal.get_height())
            texture = window.get_native().get_renderer().render_texture(snapshot.to_node(), None)
            texture.save_to_png(out_path)
            print(out_path, terminal.get_char_width(), terminal.get_char_height())
            app.quit()
            return False

        # Once it has drawn
        GLib.timeout_add(1500, save)

    app = Gtk.Application(application_id="org.patrone.RenderVte")
    app.connect("activate", activate)
    app.run([])


if __name__ == "__main__":
    main()
