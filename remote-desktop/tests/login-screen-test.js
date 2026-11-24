// The login-screen extension's conflict check. run.sh appends this to the
// stand-ins and extension.js.

let failures = 0;

function expect(condition, what) {
    print(`${condition ? 'ok  ' : 'FAIL'} login screen: ${what}`);
    if (!condition)
        failures++;
}

function session(id, user, seat, type, state) {
    const path = `/org/freedesktop/login1/session/_${id}`;
    properties[path] = {Id: id, Type: type, State: state};
    return [id, 1000, user, seat, path];
}

// With the desktops off, the extension stays out of the way
desktopsOn = false;
const off = new PatroneLoginExtension();
off.enable();
expect(await new LoginDialog()._findConflictingSession('10') === 'GNOME\'s own check',
    'with the desktops off, GNOME\'s own check');
off.disable();
desktopsOn = true;

const extension = new PatroneLoginExtension();
extension.enable();
const dialog = new LoginDialog();

// Signing in on the screen (session 10) while the account's desktop runs
// with no seat (session 5): not a conflict
sessions = [
    session('10', 'alice', 'seat0', 'wayland', 'opening'),
    session('5', 'alice', '', 'wayland', 'online'),
];
expect(await dialog._findConflictingSession('10') === null,
    'the account\'s seatless desktop is no conflict');

// A plain desktop on another seat still is
sessions.push(session('7', 'alice', 'seat1', 'wayland', 'active'));
expect((await dialog._findConflictingSession('10'))?.Id === '7',
    'a desktop on a seat still conflicts');

// Other accounts', and text consoles, never do
sessions = [
    session('10', 'alice', 'seat0', 'wayland', 'opening'),
    session('8', 'bob', 'seat1', 'wayland', 'active'),
    session('9', 'alice', 'seat0', 'tty', 'active'),
];
expect(await dialog._findConflictingSession('10') === null,
    'other accounts and text consoles are no conflict');

// If its check fails, GNOME's own runs
failing = true;
expect(await dialog._findConflictingSession('10') === 'GNOME\'s own check',
    'falls back to GNOME\'s check on an error');
failing = false;

extension.disable();
System.exit(failures ? 1 : 0);
