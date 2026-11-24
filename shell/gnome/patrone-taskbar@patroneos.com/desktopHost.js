// The desktop, drawn by Patrone File Manager (`patrone-file-manager
// --desktop`) the way Explorer draws Windows' desktop (the user,
// 2026-10-27: "The desktop and the roll out menu must be an extension of
// the file explorer app, while supporting the desktop and all it can do"),
// in place of Ubuntu's desktop icons. File Manager draws two windows: one
// the size of the primary screen, with the icons, and the desktop files
// panel's, the size of the screen's free area (desktopPanel.js slides it and
// keeps the pill).
//
// A Wayland app can't make its window the desktop or keep it above the
// others, so the taskbar starts File Manager's desktop as a Wayland client
// of its own, as the Recycle Bin's drop target, knows the two windows by
// their titles, and places and stacks them. The two talk over File
// Manager's standard input and output, a line at a time (the list is in
// src/desktop.rs in billsecond/patrone-file-manager). While it runs, Ubuntu's desktop icons
// extension is off, through GNOME's disabled-extensions; the taskbar turns
// it back on if File Manager's desktop goes.

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import Meta from 'gi://Meta';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';

const DESKTOP_TITLE = 'Patrone Desktop';
const PANEL_TITLE = 'Patrone Desktop Panel';
const DING_UUID = 'ding@rastersoft.com';
// Started again when it ends on its own, waiting longer each time it ends
// soon after starting; after the last, it's given up on
const RESTART_DELAYS = [1, 2, 5, 10, 30];
const SOON = 60; // seconds

export class DesktopHost {
    /**
     * @param {Gio.Settings} settings - the taskbar's settings
     * @param {object} recycleBin - the taskbar's Recycle Bin (recycleBin.js)
     */
    constructor(settings, recycleBin) {
        this._settings = settings;
        this._bin = recycleBin;
        this._listeners = new Map();
        this._client = null;
        this._input = null;
        this._failures = 0;
        this._restartId = 0;
        this._binLater = 0;
        this._shields = new Set();
        this._encoder = new TextEncoder();
        /** File Manager's windows, once they show. */
        this.desktopWindow = null;
        this.panelWindow = null;
        // Whether the panel's window has shown and been set up
        this._panelSetUp = false;
        /** How many items the desktop has. */
        this.count = 0;

        global.display.connectObject(
            'window-created', (_, window) => this._watch(window),
            'workareas-changed', () => this._place(),
            'restacked', () => this._keepDesktopDown(), this);
        Main.layoutManager.connectObject('monitors-changed', () => this._place(), this);
        this._settings.connectObject(
            'changed::desktop-files', () => {
                this._sendPanel();
                this._queueBin();
            },
            'changed::show-recycle-bin', () => this._queueBin(), this);
        this._bin?.actor.connectObject(
            'notify::x', () => this._queueBin(),
            'notify::y', () => this._queueBin(),
            'notify::width', () => this._queueBin(),
            'notify::height', () => this._queueBin(),
            'notify::hover', () => this.shieldPointer('bin', this._bin.actor.hover), this);
        this._start();
    }

    /**
     * File Manager's windows stop taking the pointer while it's on one of
     * the taskbar's own things over them (the panel's pill, the Recycle
     * Bin) or one of those is dragged. With a window's surface under them,
     * GNOME 50 ended their drag gestures as soon as they were pressed: a
     * drag of the pill opened the panel like a click, and the Recycle Bin
     * couldn't be moved. Clicks elsewhere on the desktop are unaffected.
     *
     * @param {string} owner - what asks: 'pill', 'bin'
     * @param {boolean} shield - whether it's over them now
     */
    shieldPointer(owner, shield) {
        if (shield)
            this._shields.add(owner);
        else
            this._shields.delete(owner);
        this._applyShield();
    }

    _applyShield() {
        const reactive = this._shields.size === 0;
        // (Meta.SurfaceActor isn't introspected: by its type's name)
        const walk = actor => {
            if (GObject.type_name(actor.constructor.$gtype).startsWith('MetaSurfaceActor'))
                actor.reactive = reactive;
            for (const child of actor.get_children())
                walk(child);
        };
        for (const window of [this.desktopWindow, this.panelWindow]) {
            const actor = window?.get_compositor_private();
            if (actor)
                walk(actor);
        }
    }

    destroy() {
        this._destroyed = true;
        if (this._binLater)
            global.compositor.get_laters().remove(this._binLater);
        global.display.disconnectObject(this);
        Main.layoutManager.disconnectObject(this);
        this._settings.disconnectObject(this);
        this._bin?.actor.disconnectObject(this);
        this._stop();
        this._listeners.clear();
    }

    /**
     * Calls back when File Manager's desktop says something: 'count',
     * 'open' and 'close' (asks for the panel to show or go), and
     * 'panel-window' (the panel's window came or went).
     *
     * @param {string} name - what
     * @param {Function|null} callback - called with no arguments; null stops
     */
    on(name, callback) {
        if (callback)
            this._listeners.set(name, callback);
        else
            this._listeners.delete(name);
    }

    _emit(name) {
        this._listeners.get(name)?.();
    }

    /** Whether File Manager's desktop is up. */
    get running() {
        return !!this.desktopWindow;
    }

    /** The panel's window actor, while it has one. */
    get panelActor() {
        return this._panelSetUp ? this.panelWindow?.get_compositor_private() ?? null : null;
    }

    /**
     * Tells File Manager's desktop something: 'open' or 'close' (the panel
     * is opening or closing).
     *
     * @param {string} line
     */
    tell(line) {
        if (!this._input)
            return;
        try {
            this._input.write_all(this._encoder.encode(`${line}\n`), null);
        } catch (e) {
            console.warn(`Patrone taskbar: can't reach File Manager's desktop: ${e.message}`);
        }
    }

    /**
     * Shows the panel's window, in place and above the windows, and gives it
     * the keyboard. Above only while it's open: GNOME counts every window
     * that's above, minimized or not, and one this big made it take each new
     * window for covered, leaving it behind the one in front with no keyboard.
     */
    showPanel() {
        const window = this.panelWindow;
        if (!window || !this._panelSetUp)
            return;
        this._placePanel();
        Main.wm.skipNextEffect(window.get_compositor_private());
        window.make_above();
        window.unminimize();
        window.activate(global.get_current_time());
    }

    /**
     * Puts the panel's window away: not above, and minimized, since a window
     * that's there takes the keyboard when the last one closes, and one that
     * covers the screen's free area leaves GNOME no free place for new
     * windows. Not until it has first shown (minimized, it wouldn't, and the
     * first open would have no icons), and not while the overview is open:
     * desktopPanel.js asks again once it's closed.
     */
    hidePanel() {
        const window = this.panelWindow;
        if (!window || !this._panelSetUp)
            return;
        if (window.is_above())
            window.unmake_above();
        if (window.minimized || Main.overview.visible)
            return;
        Main.wm.skipNextEffect(window.get_compositor_private());
        // GNOME won't minimize a window that's not in the window list. All
        // at once, so nothing sees it there, but the overview would: its
        // windows take it for a new one, and leave a handler behind.
        window.show_in_window_list();
        window.minimize();
        this._hideFromWindowList(window);
    }

    _start() {
        if (this._client || this._restartId || this._destroyed)
            return;
        const program = GLib.find_program_in_path('patrone-file-manager');
        if (!program) {
            // No File Manager: Ubuntu's desktop icons, if the taskbar had
            // turned them off
            this._replaceDing(false);
            return;
        }
        try {
            const launcher = new Gio.SubprocessLauncher({
                flags: Gio.SubprocessFlags.STDIN_PIPE | Gio.SubprocessFlags.STDOUT_PIPE,
            });
            const client = Meta.WaylandClient.new_subprocess(global.context, launcher,
                [program, '--desktop']);
            const process = client.get_subprocess();
            this._client = client;
            this._startedAt = GLib.get_monotonic_time();
            this._input = process.get_stdin_pipe();
            this._read(client, Gio.DataInputStream.new(process.get_stdout_pipe()));
            process.wait_async(null, () => this._ended(client));
            this._sendArea();
            this._sendBin();
            this._sendPanel();
        } catch (e) {
            console.warn(`Patrone taskbar: can't start File Manager's desktop: ${e.message}`);
        }
    }

    _stop() {
        if (this._restartId)
            GLib.source_remove(this._restartId);
        this._restartId = 0;
        const client = this._client;
        this._client = null;
        this._input = null;
        this._forget();
        client?.get_subprocess()?.force_exit();
    }

    // Ended on its own: again in a moment, unless it keeps ending
    _ended(client) {
        if (this._client !== client)
            return;
        this._client = null;
        this._input = null;
        this._forget();
        if (this._destroyed)
            return;
        const soon = GLib.get_monotonic_time() - this._startedAt < SOON * GLib.USEC_PER_SEC;
        this._failures = soon ? this._failures + 1 : 1;
        if (this._failures > RESTART_DELAYS.length) {
            console.warn('Patrone taskbar: File Manager\'s desktop keeps ending; Ubuntu\'s desktop icons are back');
            this._replaceDing(false);
            return;
        }
        this._restartId = GLib.timeout_add_seconds(GLib.PRIORITY_LOW,
            RESTART_DELAYS[this._failures - 1], () => {
                this._restartId = 0;
                this._start();
                return GLib.SOURCE_REMOVE;
            });
    }

    _forget() {
        for (const window of [this.desktopWindow, this.panelWindow])
            window?.disconnectObject(this);
        const hadPanel = !!this.panelWindow;
        this.desktopWindow = null;
        this.panelWindow = null;
        this._panelSetUp = false;
        if (hadPanel)
            this._emit('panel-window');
    }

    _read(client, stream) {
        stream.read_line_async(GLib.PRIORITY_DEFAULT, null, (_, result) => {
            if (this._client !== client)
                return;
            let line;
            try {
                [line] = stream.read_line_finish_utf8(result);
            } catch {
                return;
            }
            if (line === null)
                return;
            this._handle(line.trim());
            this._read(client, stream);
        });
    }

    _handle(line) {
        const [what, ...rest] = line.split(/\s+/);
        switch (what) {
        case 'count':
            this.count = Number(rest[0]) || 0;
            this._emit('count');
            break;
        case 'open':
        case 'close':
            this._emit(what);
            break;
        case 'show-bin':
            this._settings.set_boolean('show-recycle-bin', true);
            break;
        }
    }

    // The screen and its free area, once there's a screen (over Remote
    // Desktop there's none for a moment after sign-in)
    _where() {
        const monitor = Main.layoutManager.primaryMonitor;
        if (!monitor)
            return null;
        const area = Main.layoutManager.getWorkAreaForMonitor(Main.layoutManager.primaryIndex);
        return area ? {monitor, area} : null;
    }

    // The desktop window's corner: the screen's, or where GNOME keeps the
    // window (inside the free area)
    _origin() {
        const where = this._where();
        if (!where)
            return null;
        const frame = this.desktopWindow?.get_frame_rect();
        return frame ? [frame.x, frame.y] : [where.monitor.x, where.monitor.y];
    }

    _sendArea() {
        const where = this._where();
        const origin = this._origin();
        if (!where || !origin)
            return;
        const {area} = where;
        this.tell(`area ${area.x - origin[0]} ${area.y - origin[1]} ${area.width} ${area.height}`);
    }

    // The Recycle Bin: whether it's on the desktop, where it sits there
    // (no icon goes under it, on the panel too), and its size
    _sendBin() {
        const origin = this._origin();
        const actor = this._bin?.actor;
        if (!origin || !actor)
            return;
        const [, , width, height] = actor.get_preferred_size();
        const [x, y] = this._bin.homePosition?.() ?? [actor.x, actor.y];
        const shown = this._settings.get_boolean('show-recycle-bin') &&
            this._settings.get_string('desktop-files') !== 'hidden' ? 1 : 0;
        this.tell(`bin ${shown} ${Math.round(x - origin[0])} ${Math.round(y - origin[1])} ` +
            `${Math.round(width)} ${Math.round(height)}`);
    }

    _queueBin() {
        if (this._binLater)
            return;
        this._binLater = global.compositor.get_laters().add(Meta.LaterType.BEFORE_REDRAW, () => {
            this._binLater = 0;
            this._sendBin();
            return GLib.SOURCE_REMOVE;
        });
    }

    _sendPanel() {
        const mode = this._settings.get_string('desktop-files');
        this.tell(`panel ${{panel: 'on', hidden: 'clean'}[mode] ?? 'off'}`);
    }

    _place() {
        this._sendArea();
        this._queueBin();
        this._placeDesktop();
        this._placePanel();
    }

    // The desktop's window covers the primary screen
    _placeDesktop() {
        const monitor = Main.layoutManager.primaryMonitor;
        if (monitor)
            this.desktopWindow?.move_resize_frame(false, monitor.x, monitor.y, monitor.width, monitor.height);
    }

    // The panel's window covers the screen's free area
    _placePanel() {
        const where = this._where();
        if (!where || !this.panelWindow)
            return;
        const {area} = where;
        this.panelWindow.move_resize_frame(false, area.x, area.y, area.width, area.height);
    }

    // File Manager's windows, known by their titles (which GTK sets before
    // they show, but GNOME may learn after it makes the window)
    _watch(window) {
        if (!window || !this._client?.owns_window(window))
            return;
        window.connectObject(
            'notify::title', () => this._adopt(window),
            'unmanaged', () => this._unmanaged(window), this);
        this._adopt(window);
    }

    _adopt(window) {
        const title = window.get_title();
        if (title === DESKTOP_TITLE && this.desktopWindow !== window) {
            this.desktopWindow = window;
            this._whenShown(window, () => this._setUpDesktop(window));
        } else if (title === PANEL_TITLE && this.panelWindow !== window) {
            this.panelWindow = window;
            this._whenShown(window, () => this._setUpPanel(window));
        }
    }

    _unmanaged(window) {
        if (window === this.desktopWindow)
            this.desktopWindow = null;
        if (window === this.panelWindow) {
            this.panelWindow = null;
            this._panelSetUp = false;
            this._emit('panel-window');
        }
    }

    // No opening animation, and the rest once it's shown: made a desktop
    // window, or moved, as soon as it was created, the drop target never
    // showed (recycleBin.js). Showing gives it the keyboard, which goes back
    // where it was.
    _whenShown(window, setUp) {
        if (window.mapped) {
            setUp();
            return;
        }
        const actor = window.get_compositor_private();
        if (actor)
            Main.wm.skipNextEffect(actor);
        const focused = global.display.focus_window;
        const id = window.connect('shown', () => {
            window.disconnect(id);
            setUp();
            if (global.display.focus_window === window) {
                if (focused && focused !== window && !focused.minimized)
                    focused.activate(global.get_current_time());
                else
                    global.display.unset_input_focus(global.get_current_time());
            }
        });
    }

    _hideFromWindowList(window) {
        if (window.hide_from_window_list)
            window.hide_from_window_list();
        else
            this._client?.hide_from_window_list(window);
    }

    // The desktop: a desktop window (no taskbar button, Alt+Tab, or
    // moving), on every workspace, under every other window and the
    // Recycle Bin's drop target
    _setUpDesktop(window) {
        this._hideFromWindowList(window);
        window.set_type?.(Meta.WindowType.DESKTOP);
        window.stick();
        this._placeDesktop();
        window.lower();
        // Where it ends up sets where its icons go
        window.connectObject('position-changed', () => {
            this._sendArea();
            this._queueBin();
        }, this);
        this._sendArea();
        this._queueBin();
        this._applyShield();
        this._replaceDing(true);
    }

    // The panel: on every workspace, put away until it opens (desktopPanel.js)
    _setUpPanel(window) {
        this._hideFromWindowList(window);
        window.stick();
        this._placePanel();
        this._panelSetUp = true;
        this.hidePanel();
        this._applyShield();
        this._emit('panel-window');
    }

    // Clicking the desktop raises it above the Recycle Bin's drop target
    // (also a desktop window), which then can't take drops: down it goes
    _keepDesktopDown() {
        const actor = this.desktopWindow?.get_compositor_private();
        if (!actor)
            return;
        const children = global.window_group.get_children();
        const own = children.indexOf(actor);
        const another = children.some((child, index) => index < own &&
            child instanceof Meta.WindowActor &&
            child.meta_window?.get_window_type() === Meta.WindowType.DESKTOP);
        if (another)
            this.desktopWindow.lower();
    }

    // Ubuntu's desktop icons go while File Manager draws the desktop, and
    // come back (only if the taskbar turned them off) when it can't
    _replaceDing(replace) {
        const disabled = global.settings.get_strv('disabled-extensions');
        if (replace) {
            if (!disabled.includes(DING_UUID)) {
                this._settings.set_boolean('ding-replaced', true);
                global.settings.set_strv('disabled-extensions', [...disabled, DING_UUID]);
            }
        } else if (this._settings.get_boolean('ding-replaced')) {
            this._settings.set_boolean('ding-replaced', false);
            if (disabled.includes(DING_UUID)) {
                global.settings.set_strv('disabled-extensions',
                    disabled.filter(uuid => uuid !== DING_UUID));
            }
        }
    }
}
