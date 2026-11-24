// Where the desktop's files show, off the desktop: the desktop files panel
// (the user's design, 2026-10-30, in place of the bubble of 2026-10-26). The
// desktop-files setting has them on the desktop (the default), in the panel,
// which a pill opens, or 'hidden': a clean desktop, with nothing on it, not
// even the Recycle Bin, where the panel opens from a corner (desktopCorner.js,
// the user's next ask that day), or the desktop's or the taskbar's right-click
// menu.
//
// The panel slides out of the left edge of the primary screen, or the right
// for a corner on that side, as a full layer over the windows: the screen's
// free area, the wallpaper behind the icons, each icon where it would be on
// the desktop, the taskbar still in front. The pill at that edge, shown while
// no window covers its spot, opens it when clicked or dragged out, and rides
// on the panel's edge to push it back. The Recycle Bin comes in on the panel.
// Until the pill is first opened, it bounces now and then with a tip saying
// what it does.
//
// It closes by itself as soon as a window opens on top of it (above the
// windows, it would hide that window), and when anything else takes the
// keyboard, the workspace changes or the overview opens; Escape, its Close
// button, the pill, the corner and a click on the taskbar close it too. A
// click on its empty part doesn't: that's the desktop's own, for the rubber
// band and the right-click menu.
//
// Patrone File Manager draws the icons, in a window of its own, clear but for
// them and its small bar (desktopHost.js, src/desktop.rs in billsecond/patrone-file-manager); this
// keeps the pill, slides that window's actor out of the edge with the
// wallpaper under it, and carries the Recycle Bin. Ubuntu's desktop icons
// extension is off while File Manager draws the desktop; if it runs anyway,
// its windows are kept out of sight.

import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import St from 'gi://St';

import * as Background from 'resource:///org/gnome/shell/ui/background.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

import {DesktopCorner} from './desktopCorner.js';

const PILL_WIDTH = 12;
const PILL_HEIGHT = 72;
// A window this close to the pill counts as covering it
const PILL_MARGIN = 6;

const OPEN_TIME = 260;
const CLOSE_TIME = 200;
// The shadow the panel's edge drops on the windows while it slides
const EDGE_WIDTH = 24;
// What closes it by showing: not menus or tooltips, which are the panel's or
// the shell's own
const OPENS_OVER = [
    Meta.WindowType.NORMAL,
    Meta.WindowType.DIALOG,
    Meta.WindowType.MODAL_DIALOG,
    Meta.WindowType.UTILITY,
];
// Until it's first opened, the pill bounces this often (seconds), its tip
// showing for a moment (milliseconds)
const BOUNCE_EVERY = 6;
const TIP_TIME = 3500;
const HOP = 10;
const TIP_FIRST = 'Your desktop files are in here: click, or drag it out';
// Files an app drags, held over the pill this long (milliseconds), open the
// panel, so they can be dropped in it, or close it, for the windows under it
const DRAG_OPEN_TIME = 500;

export class DesktopPanel {
    /**
     * @param {Gio.Settings} settings - the taskbar's settings
     * @param {object} recycleBin - the taskbar's Recycle Bin (recycleBin.js)
     * @param {object} host - File Manager's desktop (desktopHost.js)
     */
    constructor(settings, recycleBin, host) {
        this._settings = settings;
        this._bin = recycleBin;
        this._host = host;
        this._interface = new Gio.Settings({schema_id: 'org.gnome.desktop.interface'});
        this._open = false;
        this._showing = false;
        // Which side it comes from: -1 the left, 1 the right
        this._side = -1;
        this._coverLater = 0;
        this._desktopsLater = 0;
        this._bounceId = 0;
        this._firstBounceId = 0;
        this._tipId = 0;

        // The pill, its tip and the Recycle Bin while it's on the panel:
        // above the windows, below the taskbar and the shell's menus. Among
        // the windows' own actors, so that while the panel is open they go
        // right above its window and under its menus and tooltips (which
        // are windows above it); Mutter puts what isn't a window above the
        // windows at every restack, and _restackLayer moves it back down.
        this._layer = new St.Widget({reactive: false, visible: false});
        this._layer.add_constraint(new Clutter.BindConstraint({
            source: global.stage,
            coordinate: Clutter.BindCoordinate.SIZE,
        }));
        global.window_group.add_child(this._layer);

        // The wallpaper, right under the panel's clear window while it's
        // open, so the icons sit on it as they do on the desktop
        this._wall = new St.Widget({reactive: false, visible: false});
        this._wall.add_constraint(new Clutter.BindConstraint({
            source: global.stage,
            coordinate: Clutter.BindCoordinate.SIZE,
        }));
        global.window_group.add_child(this._wall);

        this._edge = new St.Widget({style_class: 'patrone-desktop-panel-edge', visible: false});
        this._layer.add_child(this._edge);

        this._pillIcon = new St.Icon({icon_name: 'pan-end-symbolic', style_class: 'patrone-desktop-pill-icon'});
        this._pill = new St.Bin({
            style_class: 'patrone-desktop-pill',
            reactive: true,
            track_hover: true,
            child: this._pillIcon,
            accessible_name: 'Desktop',
            opacity: 0,
        });
        this._pill.connect('notify::hover', () => {
            if (this._pill.hover)
                this._showTip(0);
            else if (!this._tipId)
                this._hideTip();
            this._shieldPointer();
        });
        this._layer.add_child(this._pill);

        const click = new Clutter.ClickGesture();
        click.connect('recognize', () => (this._open ? this.close() : this.open()));
        this._pill.add_action(click);
        this._pan = new Clutter.PanGesture();
        this._pan.connect('recognize', () => this._dragStarted());
        this._pan.connect('pan-update', () => this._dragged());
        this._pan.connect('end', () => this._dragEnded(false));
        this._pan.connect('cancel', () => this._dragEnded(true));
        this._pill.add_action(this._pan);

        this._tip = new St.Label({style_class: 'patrone-taskbar-tooltip', visible: false});
        this._layer.add_child(this._tip);

        // Where Ubuntu's desktop icons wait, unseen, if they run
        // (_hideDesktopIcons)
        this._parking = new Clutter.Actor({visible: false});
        Main.uiGroup.add_child(this._parking);

        // Something an app drags (files from File Manager), held over the
        // pill, opens the panel to drop it in, as Windows opens a window
        // when a drag waits over its taskbar button; held over it with the
        // panel open, it closes the panel, for the windows under it
        this._dragOpenId = 0;
        this._appDragging = false;
        global.backend.get_dnd().connectObject(
            'dnd-enter', () => {
                this._appDragging = true;
                this._shieldPointer();
            },
            'dnd-position-change', (_, x, y) => this._appDragAt(x, y),
            'dnd-leave', () => {
                this._appDragging = false;
                this._shieldPointer();
                this._appDragAt(-1, -1);
            }, this);

        this._host.on('open', () => this.open());
        this._host.on('close', () => this.close());
        this._host.on('panel-window', () => {
            if (!this._host.panelWindow)
                this.close(false);
            this._queueChecks();
        });

        this._settings.connectObject(
            'changed::desktop-files', () => {
                this.close(false);
                this._sync();
            },
            'changed::show-recycle-bin', () => this.close(false),
            'changed::recycle-bin-dock', () => this._follow(),
            'changed::recycle-bin-position', () => this._follow(),
            'changed::desktop-bubble-introduced', () => this._introduce(), this);
        global.display.connectObject(
            'workareas-changed', () => this._sync(),
            'restacked', () => this._queueChecks(),
            'window-created', (_, window) => this._watchWindow(window),
            'notify::focus-window', () => this._focusChanged(), this);
        for (const window of global.display.list_all_windows())
            this._watchWindow(window);
        global.workspace_manager.connectObject('active-workspace-changed', () => {
            this.close(false);
            this._queueChecks();
        }, this);
        Main.layoutManager.connectObject('monitors-changed', () => {
            this.close(false);
            this._sync();
        }, this);
        Main.overview.connectObject(
            'showing', () => this._sync(),
            'hidden', () => this._sync(), this);

        this._sync();
        this._introduce();
        this._corner = new DesktopCorner(settings, side => this.toggle(side));
    }

    destroy() {
        // Nothing more is queued from here on, and File Manager's desktop
        // (ended right after this, when the taskbar is turned off, as GNOME
        // does at every screen lock) no longer calls in: its last calls
        // queued checks that ran on the destroyed pill and layer.
        this._destroyed = true;
        this._corner.destroy();
        for (const name of ['open', 'close', 'panel-window'])
            this._host.on(name, null);
        this.close(false);
        this._stopIntroduction();
        for (const id of [this._tipId, this._dragOpenId]) {
            if (id)
                GLib.source_remove(id);
        }
        global.backend.get_dnd().disconnectObject(this);
        for (const later of [this._coverLater, this._desktopsLater]) {
            if (later)
                global.compositor.get_laters().remove(later);
        }
        this._settings.disconnectObject(this);
        global.display.disconnectObject(this);
        global.workspace_manager.disconnectObject(this);
        Main.layoutManager.disconnectObject(this);
        Main.overview.disconnectObject(this);
        // The windows go back before their hiding place goes, or they'd go
        // with it
        this._hideDesktopIcons(false);
        this._parking.destroy();
        this._wall.destroy();
        this._layer.destroy();
    }

    get _mode() {
        return this._settings.get_string('desktop-files');
    }

    // The files are off the desktop
    get _on() {
        return this._mode !== 'desktop';
    }

    get _scale() {
        return St.ThemeContext.get_for_stage(global.stage).scale_factor;
    }

    // None until there's a screen: over Remote Desktop, a moment after
    // sign-in (monitors-changed syncs it then)
    _area() {
        if (!Main.layoutManager.primaryMonitor)
            return null;
        return Main.layoutManager.getWorkAreaForMonitor(Main.layoutManager.primaryIndex);
    }

    // How far out the panel is, from 0 to 1
    get _grown() {
        const actor = this._host.panelActor;
        const area = this._area();
        return this._showing && actor && area ? 1 - Math.abs(actor.translation_x) / area.width : 0;
    }

    // Shows or hides it all for the setting, the overview and the screen
    _sync() {
        const on = this._on;
        this._hideDesktopIcons(on || this._host.running);
        if (!on || Main.overview.visible)
            this.close(false);
        if (!this._showing)
            this._host.hidePanel();
        this._layer.visible = on && !Main.overview.visible;
        this._follow();
        this._queueChecks();
    }

    /**
     * Opens the panel, sliding out of an edge.
     *
     * @param {number} side - -1 for the left, 1 for the right
     */
    open(side = -1) {
        if (this._prepare(side))
            this._settle(true);
    }

    /**
     * Opens the panel, or closes it if it's open or opening.
     *
     * @param {number} side - where it comes from: -1 the left, 1 the right
     */
    toggle(side) {
        if (this._showing && this._open)
            this.close();
        else
            this.open(side);
    }

    /**
     * Closes the panel.
     *
     * @param {boolean} animate - slide back into the edge, or go at once
     */
    close(animate = true) {
        if (!this._showing)
            return;
        if (animate && this._host.panelActor) {
            this._settle(false);
            return;
        }
        this._stopSlide();
        this._closed();
    }

    // The panel's window, shown out of sight past the edge, ready to slide,
    // with the keyboard
    _prepare(side = -1) {
        if (this._showing)
            return true;
        const actor = this._host.panelActor;
        const area = this._area();
        if (!this._layer.visible || !actor || !area)
            return false;
        this._showing = true;
        this._side = side;
        this._edge.set_style_class_name(`patrone-desktop-panel-edge${side > 0 ? ' right' : ''}`);
        this._returnFocus = global.display.focus_window;
        actor.translation_x = side * area.width;
        actor.connectObject('notify::translation-x', () => this._follow(), this);
        this._showWall();
        this._host.showPanel();
        this._host.tell('open');
        this._pill.add_style_class_name('open');
        this._pillIcon.icon_name = 'pan-start-symbolic';
        // (Opened by File Manager, say for a new file, a window may have
        // been covering it)
        if (this._mode === 'panel') {
            this._pill.remove_transition('opacity');
            this._pill.opacity = 255;
            this._pill.show();
        }
        this._hideTip();
        this._holdBin();
        this._restackLayer();
        // A click on the taskbar or a menu of the shell's closes it too
        global.stage.connectObject('captured-event', (_, event) => this._stageEvent(event), this);
        if (!this._settings.get_boolean('desktop-bubble-introduced'))
            this._settings.set_boolean('desktop-bubble-introduced', true);
        return true;
    }

    _showWall() {
        this._wallManager = new Background.BackgroundManager({
            container: this._wall,
            monitorIndex: Main.layoutManager.primaryIndex,
        });
        this._wall.show();
    }

    _hideWall() {
        this._wall.hide();
        this._wallManager?.destroy();
        this._wallManager = null;
    }

    // Slides it all the way out, or back. It doesn't overshoot: a whole
    // layer of the screen has nowhere to bounce to. By a timeline of its own,
    // not ease(): with no graphics card (a VM, say) GNOME ends every ease()
    // at once, and the panel would just appear. The slide says where it came
    // from, and it's short. The account's own setting for animations counts.
    _settle(open) {
        const actor = this._host.panelActor;
        const area = this._area();
        if (!actor || !area) {
            this._closed();
            return;
        }
        this._open = open;
        this._stopSlide();
        const to = open ? 0 : this._side * area.width;
        if (!this._interface.get_boolean('enable-animations')) {
            actor.translation_x = to;
            this._settled(open);
            return;
        }
        const from = actor.translation_x;
        const slide = new Clutter.Timeline({
            actor: this._layer,
            duration: open ? OPEN_TIME : CLOSE_TIME,
            progress_mode: open ? Clutter.AnimationMode.EASE_OUT_CUBIC : Clutter.AnimationMode.EASE_IN_CUBIC,
        });
        slide.connect('new-frame', () => {
            actor.translation_x = from + (to - from) * slide.get_progress();
        });
        slide.connect('completed', () => {
            actor.translation_x = to;
            this._slide = null;
            this._settled(open);
        });
        this._slide = slide;
        slide.start();
    }

    _stopSlide() {
        this._slide?.stop();
        this._slide = null;
    }

    _settled(open) {
        if (!open)
            this._closed();
        else if (!this._dragGrab)
            // It slid out from under the pointer; File Manager's windows
            // wait for the pointer to leave it (desktopHost.js)
            this._pill.sync_hover();
    }

    _closed() {
        const actor = this._host.panelActor;
        const panelWindow = this._host.panelWindow;
        const hadKeyboard = !!panelWindow && global.display.focus_window === panelWindow;
        const area = this._area();
        // Out of sight first: its window goes before the next frame
        if (actor && area)
            actor.translation_x = this._side * area.width;
        this._open = false;
        this._showing = false;
        global.stage.disconnectObject(this);
        actor?.disconnectObject(this);
        this._host.tell('close');
        this._host.hidePanel();
        this._hideWall();
        this._edge.hide();
        this._pill.remove_style_class_name('open');
        this._pillIcon.icon_name = 'pan-end-symbolic';
        this._releaseBin();
        // The keyboard back where it was, or on the window that just opened.
        // Not on another workspace: activating a window there goes back to
        // it, after a change of workspace closed the panel. Failing that it's
        // GNOME's pick, or nowhere if that's the panel, still going while the
        // overview is open, or the Recycle Bin's drop target.
        const back = this._returnFocus;
        this._returnFocus = null;
        if (hadKeyboard) {
            const workspace = global.workspace_manager.get_active_workspace();
            const focus = global.display.focus_window;
            if (back && back !== panelWindow && !back.minimized &&
                global.display.list_all_windows().includes(back) &&
                back.located_on_workspace(workspace))
                back.activate(global.get_current_time());
            else if (focus === panelWindow || focus?.get_window_type() === Meta.WindowType.DESKTOP)
                global.display.unset_input_focus(global.get_current_time());
        }
        this._follow();
        this._queueChecks();
    }

    // A window shown on top of the panel closes it. Above the windows, it
    // would hide that window.
    _windowShown(window) {
        const panelWindow = this._host.panelWindow;
        if (!this._showing || window === panelWindow || window.get_transient_for() === panelWindow ||
            !OPENS_OVER.includes(window.get_window_type()) ||
            !window.located_on_workspace(global.workspace_manager.get_active_workspace()))
            return;
        // The keyboard goes to it, even if GNOME kept it from taking it: back
        // where it was, the window from before would come up over it
        this._returnFocus = window;
        this.close();
    }

    // Like a menu: anything else taking the keyboard closes it. The
    // panel's own menus are popups of its window, which keeps the keyboard.
    _focusChanged() {
        const focus = global.display.focus_window;
        const panelWindow = this._host.panelWindow;
        if (!this._showing) {
            this._keepFocusOffDesktop();
            return;
        }
        if (focus && focus !== panelWindow && focus.get_transient_for() !== panelWindow)
            this.close();
    }

    // A press on the shell itself (the taskbar, its menus) closes it; one
    // on a window is the windows' business (it takes the keyboard), and
    // the pill and the Recycle Bin have their own
    _stageEvent(event) {
        const type = event.type();
        if (type !== Clutter.EventType.BUTTON_PRESS && type !== Clutter.EventType.TOUCH_BEGIN)
            return Clutter.EVENT_PROPAGATE;
        const target = global.stage.get_event_actor(event);
        const inside = actor => {
            for (let current = target; current; current = current.get_parent()) {
                if (current === actor)
                    return true;
            }
            return false;
        };
        if (!inside(global.window_group) && !inside(this._pill) &&
            !(this._bin?.actor && inside(this._bin.actor)) && !this._isMenu(target))
            this.close();
        return Clutter.EVENT_PROPAGATE;
    }

    // The Recycle Bin's menu, opened while it's on the panel
    _isMenu(actor) {
        for (let current = actor; current; current = current.get_parent()) {
            if (current.has_style_class_name?.('patrone-menu'))
                return true;
        }
        return false;
    }

    // Dragging the pill pulls the panel out (or pushes it back) with the
    // pointer; let go, it opens if it was pulled out far enough
    _dragStarted() {
        if (!this._prepare()) {
            this._dragFrom = null;
            return;
        }
        this._stopSlide();
        this._dragFrom = this._grown;
        this._dragGrab = global.stage.grab(this._pill);
        this._shieldPointer();
    }

    // An app's drag at (x, y), or gone (-1, -1): waiting over the pill, it
    // opens the panel, or closes an open one
    _appDragAt(x, y) {
        const [px, py] = this._pill.get_transformed_position();
        const [width, height] = this._pill.get_transformed_size();
        const over = this._pill.visible &&
            x >= px && x < px + width && y >= py && y < py + height;
        if (!over) {
            if (this._dragOpenId)
                GLib.source_remove(this._dragOpenId);
            this._dragOpenId = 0;
            return;
        }
        if (this._dragOpenId)
            return;
        this._dragOpenId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, DRAG_OPEN_TIME, () => {
            this._dragOpenId = 0;
            if (this._showing)
                this.close();
            else
                this.open();
            return GLib.SOURCE_REMOVE;
        });
    }

    // While the pointer's on the pill, or the pill is dragged, File
    // Manager's windows under it don't take the pointer (desktopHost.js)
    // (Not while an app drags something: the panel's window must take its
    // drop.)
    _shieldPointer() {
        this._host.shieldPointer('pill',
            (this._pill.hover || !!this._dragGrab) && !this._appDragging);
    }

    _dragged() {
        const actor = this._host.panelActor;
        const area = this._area();
        if (this._dragFrom === null || this._dragFrom === undefined || !actor || !area)
            return;
        const delta = this._pan.get_accumulated_delta_abs().get_x();
        const grown = Math.min(Math.max(this._dragFrom + delta / area.width, 0), 1);
        actor.translation_x = (grown - 1) * area.width;
    }

    _dragEnded(cancelled) {
        this._dragGrab?.dismiss();
        this._dragGrab = null;
        this._shieldPointer();
        if (this._dragFrom === null || this._dragFrom === undefined)
            return;
        const wasOpen = this._dragFrom >= 0.5;
        const reached = this._grown;
        this._dragFrom = null;
        this._settle(cancelled ? wasOpen : reached > (wasOpen ? 0.75 : 0.25));
    }

    // What moves with the panel's edge: the part of its window and of the
    // wallpaper past the screen's edge is cut off (it would show on a screen
    // beside it), and the pill, the shadow and the Recycle Bin go along. The
    // Recycle Bin is on the panel where it is on the desktop.
    _follow() {
        const area = this._area();
        if (!area)
            return;
        const monitor = Main.layoutManager.primaryMonitor;
        const scale = this._scale;
        const [pillWidth, pillHeight] = [PILL_WIDTH * scale, PILL_HEIGHT * scale];
        const grown = this._grown;
        const reach = Math.round(area.width * grown);
        const hidden = area.width - reach;
        const fromRight = this._side > 0;
        const x = this._showing ? Math.max(area.x + reach - pillWidth, area.x) : area.x;
        this._layer.set_clip(monitor.x, monitor.y, monitor.width, monitor.height);
        this._pill.set_size(pillWidth, pillHeight);
        this._pill.set_position(Math.round(x), Math.round(area.y + area.height / 2 - pillHeight / 2));
        if (!this._showing)
            return;

        const actor = this._host.panelActor;
        const edgeWidth = EDGE_WIDTH * scale;
        actor?.set_clip(fromRight ? 0 : hidden, 0, reach, area.height);
        this._wall.translation_x = this._side * hidden;
        this._wall.set_clip(fromRight ? area.x : area.x + hidden, area.y, reach, area.height);
        this._edge.visible = grown > 0 && grown < 1;
        this._edge.set_position(fromRight ? area.x + hidden - edgeWidth : area.x + reach, area.y);
        this._edge.set_size(edgeWidth, area.height);
        if (this._binHeld) {
            const home = this._bin.homePosition();
            if (home)
                this._bin.hold(this._layer, home[0] + this._side * hidden, home[1]);
        }
    }

    // It comes in on the panel, even while it's off the desktop (the user,
    // 2026-10-27)
    _holdBin() {
        if (!this._bin?.hold)
            return;
        this._binHeld = true;
        this._follow();
    }

    _releaseBin() {
        if (!this._binHeld)
            return;
        this._binHeld = false;
        this._bin.release();
    }

    // Ubuntu's desktop icons extension's windows, which cover the screen,
    // by that extension's own mark: the Recycle Bin's drop target and File
    // Manager's desktop are desktop windows too, and must stay
    _isDesktopIcons(window) {
        return !!window?.customJS_ding;
    }

    _watchWindow(window) {
        window?.connectObject(
            'position-changed', () => this._queueCoverCheck(),
            'size-changed', () => this._queueCoverCheck(),
            'notify::minimized', () => this._queueCoverCheck(),
            'notify::window-type', () => this._queueChecks(),
            'shown', () => this._windowShown(window), this._layer);
        this._queueChecks();
    }

    _queueChecks() {
        this._queueCoverCheck();
        if (this._desktopsLater || this._destroyed)
            return;
        this._desktopsLater = global.compositor.get_laters().add(Meta.LaterType.BEFORE_REDRAW, () => {
            this._desktopsLater = 0;
            this._hideDesktopIcons(this._on || this._host.running);
            this._restackLayer();
            return GLib.SOURCE_REMOVE;
        });
    }

    // Right above the panel's window while it's open, the wallpaper right
    // under it, else above them all
    _restackLayer() {
        const actor = this._host.panelActor;
        if (this._showing && actor?.get_parent() === global.window_group) {
            global.window_group.set_child_above_sibling(this._layer, actor);
            global.window_group.set_child_below_sibling(this._wall, actor);
        } else {
            global.window_group.set_child_above_sibling(this._layer, null);
        }
    }

    // Ubuntu's desktop icons' windows wait in a hidden group of their own,
    // where they neither draw nor take clicks. (Clipped to nothing, they
    // still drew.) Mutter keeps restacking them there, as it does windows
    // GNOME Shell's own effects move. They're the window group's own
    // children, since that extension takes its windows out of
    // global.get_window_actors() on Wayland.
    _hideDesktopIcons(hide) {
        if (hide) {
            for (const actor of global.window_group.get_children()) {
                if (actor instanceof Meta.WindowActor && this._isDesktopIcons(actor.meta_window)) {
                    global.window_group.remove_child(actor);
                    this._parking.add_child(actor);
                }
            }
            this._keepFocusOffDesktop();
            return;
        }
        // Back right above the wallpaper, under every other window
        for (const actor of this._parking.get_children().reverse()) {
            this._parking.remove_child(actor);
            global.window_group.add_child(actor);
            global.window_group.set_child_above_sibling(actor, Main.layoutManager._backgroundGroup);
        }
    }

    // Unseen, Ubuntu's desktop icons' window mustn't take the keyboard (it
    // gets it when the last window closes), or keys would act on hidden
    // files
    _keepFocusOffDesktop() {
        const window = global.display.focus_window;
        if (this._isDesktopIcons(window) && this._parking.get_n_children() > 0)
            global.display.unset_input_focus(global.get_current_time());
    }

    _queueCoverCheck() {
        if (this._coverLater || this._destroyed)
            return;
        this._coverLater = global.compositor.get_laters().add(Meta.LaterType.BEFORE_REDRAW, () => {
            this._coverLater = 0;
            this._checkCover();
            return GLib.SOURCE_REMOVE;
        });
    }

    // The pill shows while no window covers its spot, and while there's a
    // panel to open; a clean desktop has none
    _checkCover() {
        const area = this._area();
        if (!area || this._showing)
            return;
        const scale = this._scale;
        const margin = PILL_MARGIN * scale;
        const top = area.y + area.height / 2 - PILL_HEIGHT * scale / 2 - margin;
        const bottom = top + PILL_HEIGHT * scale + 2 * margin;
        const right = area.x + (PILL_WIDTH + HOP) * scale + margin;
        const workspace = global.workspace_manager.get_active_workspace();
        const panelWindow = this._host.panelWindow;
        const covered = global.display.list_all_windows().some(window => {
            if (window === panelWindow || this._isDesktopIcons(window) ||
                window.get_window_type() === Meta.WindowType.DESKTOP ||
                window.minimized || !window.located_on_workspace(workspace) ||
                !window.showing_on_its_workspace() || !window.get_compositor_private()?.visible)
                return false;
            const frame = window.get_frame_rect();
            return frame.x < right && frame.x + frame.width > area.x &&
                frame.y < bottom && frame.y + frame.height > top;
        });
        const show = this._layer.visible && this._mode === 'panel' && !covered && !!panelWindow;
        if (show === this._pill.visible && this._pill.opacity === (show ? 255 : 0))
            return;
        this._pill.remove_transition('opacity');
        if (show) {
            this._pill.show();
            this._pill.ease({opacity: 255, duration: 180, mode: Clutter.AnimationMode.EASE_OUT_QUAD});
        } else {
            this._hideTip();
            this._pill.ease({
                opacity: 0,
                duration: 120,
                mode: Clutter.AnimationMode.EASE_OUT_QUAD,
                onComplete: () => this._pill.hide(),
            });
        }
    }

    // Until it's first opened, the pill bounces now and then with its tip
    _introduce() {
        if (this._settings.get_boolean('desktop-bubble-introduced')) {
            this._stopIntroduction();
            return;
        }
        if (this._bounceId)
            return;
        this._firstBounceId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1500, () => {
            this._firstBounceId = 0;
            this._bounce();
            return GLib.SOURCE_REMOVE;
        });
        this._bounceId = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, BOUNCE_EVERY, () => {
            this._bounce();
            return GLib.SOURCE_CONTINUE;
        });
    }

    _stopIntroduction() {
        for (const id of [this._bounceId, this._firstBounceId]) {
            if (id)
                GLib.source_remove(id);
        }
        this._bounceId = this._firstBounceId = 0;
        this._pill.remove_transition('translation-x');
        this._pill.translation_x = 0;
    }

    _bounce() {
        if (!this._layer.visible || !this._pill.visible || this._pill.opacity < 255 || this._showing)
            return;
        this._pill.ease({
            translation_x: HOP * this._scale,
            duration: 160,
            mode: Clutter.AnimationMode.EASE_OUT_QUAD,
            onComplete: () => this._pill.ease({
                translation_x: 0,
                duration: 500,
                mode: Clutter.AnimationMode.EASE_OUT_BOUNCE,
            }),
        });
        this._showTip(TIP_TIME);
    }

    /**
     * Shows the tip beside the pill.
     *
     * @param {number} time - how long, in milliseconds; 0 while it's hovered
     */
    _showTip(time) {
        if (this._showing || !this._pill.visible)
            return;
        const count = this._host.count;
        this._tip.text = this._settings.get_boolean('desktop-bubble-introduced')
            ? `Desktop: ${count === 1 ? '1 item' : `${count} items`}`
            : TIP_FIRST;
        if (!this._tip.visible) {
            this._tip.opacity = 0;
            this._tip.show();
            this._tip.ease({opacity: 255, duration: 120, mode: Clutter.AnimationMode.EASE_OUT_QUAD});
        }
        // Measured once shown, so with its style
        const [, height] = this._tip.get_preferred_height(-1);
        this._tip.set_position(
            Math.round(this._pill.x + this._pill.width + 8 * this._scale),
            Math.round(this._pill.y + (this._pill.height - height) / 2));
        this._layer.set_child_above_sibling(this._tip, null);
        if (this._tipId)
            GLib.source_remove(this._tipId);
        this._tipId = 0;
        if (time > 0) {
            this._tipId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, time, () => {
                this._tipId = 0;
                if (!this._pill.hover)
                    this._hideTip();
                return GLib.SOURCE_REMOVE;
            });
        }
    }

    _hideTip() {
        if (this._tipId)
            GLib.source_remove(this._tipId);
        this._tipId = 0;
        this._tip.hide();
    }
}
