// Runs data/patrone-desktops.rules under plain gjs with polkit stood in for:
// gjs -m tests/polkit-rule-test.js data/patrone-desktops.rules

import GLib from 'gi://GLib';
import System from 'system';

const [ok, contents] = GLib.file_get_contents(ARGV[0]);
if (!ok)
    throw new Error(`can't read ${ARGV[0]}`);
const source = new TextDecoder().decode(contents);

// logind's sessions, by ID
const sessions = {
    desktop: {Type: 'wayland', Seat: '', Active: 'yes'},
    x11desktop: {Type: 'x11', Seat: '', Active: 'yes'},
    ssh: {Type: 'tty', Seat: '', Active: 'yes'},
    away: {Type: 'wayland', Seat: '', Active: 'no'},
};
// Accounts: their IDs and display sessions
const users = {
    william: {uid: '1000', display: 'desktop'},
    remote: {uid: '1001', display: 'ssh'},
    nobody: {uid: '1002', display: ''},
    gone: {uid: '1003', display: 'away'},
};
// Processes' cgroups, by PID: as GNOME 50 runs the desktop's shell (no
// session), and others
const cgroups = {
    100: '0::/user.slice/user-1000.slice/user@1000.service/session.slice/org.gnome.Shell@ubuntu.service\n',
    101: '0::/user.slice/user-1000.slice/user@1000.service/init.scope\n',
    102: '0::/user.slice/user-1001.slice/user@1001.service/session.slice/org.gnome.Shell@ubuntu.service\n',
    103: '0::/user.slice/user-1002.slice/user@1002.service/session.slice/org.gnome.Shell@ubuntu.service\n',
    104: '0::/user.slice/user-1003.slice/user@1003.service/session.slice/org.gnome.Shell@ubuntu.service\n',
};
let rule = null;
let desktopsOn = true;
const polkit = {
    Result: {YES: 'yes', NO: 'no', NOT_HANDLED: 'not handled'},
    addRule(fn) {
        rule = fn;
    },
    spawn(argv) {
        const command = argv.join(' ');
        if (command === '/usr/bin/test -e /var/lib/patrone-remote-desktop/desktops-on') {
            if (!desktopsOn)
                throw new Error('exit status 1');
            return '';
        }
        if (argv[0] === '/usr/bin/loginctl' && argv[1] === 'show-session') {
            const props = sessions[argv[2]];
            if (!props)
                throw new Error('no such session');
            return Object.entries(props).map(([key, value]) => `${key}=${value}\n`).join('');
        }
        if (argv[0] === '/usr/bin/loginctl' && argv[1] === 'show-user') {
            if (!users[argv[2]])
                throw new Error('no such user');
            return `${users[argv[2]].display}\n`;
        }
        if (argv[0] === '/usr/bin/id' && argv[1] === '-u') {
            if (!users[argv[2]])
                throw new Error('no such user');
            return `${users[argv[2]].uid}\n`;
        }
        const pid = /^\/usr\/bin\/cat \/proc\/(\d+)\/cgroup$/.exec(command)?.[1];
        if (pid !== undefined) {
            if (!cgroups[pid])
                throw new Error('no such process');
            return cgroups[pid];
        }
        throw new Error(`unexpected command ${command}`);
    },
};
new Function('polkit', source)(polkit);

function subject(params) {
    const groups = params.groups ?? [];
    return {
        local: false,
        active: true,
        seat: '',
        session: 'desktop',
        user: 'william',
        pid: 99,
        ...params,
        isInGroup: group => groups.includes(group),
    };
}

let failures = 0;
function expect(test, actionId, params, expected) {
    const result = rule({id: actionId}, subject(params));
    if (result === expected) {
        print(`ok   polkit: ${test}`);
    } else {
        print(`FAIL polkit: ${test} (got "${result}", expected "${expected}")`);
        failures++;
    }
}

const NM = 'org.freedesktop.NetworkManager';
expect('the desktop may control the network', `${NM}.network-control`, {}, 'yes');
expect('an X11 desktop too', `${NM}.network-control`, {session: 'x11desktop'}, 'yes');
expect('the desktop may shut down', 'org.freedesktop.login1.power-off', {}, 'yes');
expect('the desktop may mount a drive', 'org.freedesktop.udisks2.filesystem-mount', {}, 'yes');
expect('not with other sessions signed in', 'org.freedesktop.login1.power-off-multiple-sessions',
    {}, 'not handled');
expect('administrator-only actions stay so', `${NM}.reload`, {}, 'not handled');
expect('system connections: administrators', `${NM}.settings.modify.system`,
    {groups: ['sudo']}, 'yes');
expect('system connections: netdev', `${NM}.settings.modify.system`,
    {groups: ['netdev']}, 'yes');
expect('system connections: not others', `${NM}.settings.modify.system`, {}, 'not handled');
expect('not an SSH login', `${NM}.network-control`, {session: 'ssh'}, 'not handled');
expect('not a session with a seat (local: the defaults)', `${NM}.network-control`,
    {local: true, seat: 'seat0'}, 'not handled');
expect('not a seat it was given anyway', `${NM}.network-control`, {seat: 'seat0'}, 'not handled');
expect('not an inactive desktop', `${NM}.network-control`, {active: false}, 'not handled');
expect('not with no session, for a process it can\'t look at', `${NM}.network-control`,
    {session: '', active: false}, 'not handled');
expect('not when logind doesn\'t know the session', `${NM}.network-control`,
    {session: 'gone'}, 'not handled');
expect('not when logind says the session isn\'t active', `${NM}.network-control`,
    {session: 'away'}, 'not handled');

// GNOME 50: the desktop's shell runs in the account's session.slice, with no
// session, while the account's display session is the desktop
const service = {session: '', active: false, pid: 100};
expect('a desktop service with no session (GNOME 50)', `${NM}.network-control`, service, 'yes');
expect('it may shut down too', 'org.freedesktop.login1.power-off', service, 'yes');
expect('but no other action', `${NM}.reload`, service, 'not handled');
expect('not outside session.slice', `${NM}.network-control`,
    {...service, pid: 101}, 'not handled');
expect('not in another account\'s session.slice', `${NM}.network-control`,
    {...service, pid: 102}, 'not handled');
expect('not when the display session is an SSH login', `${NM}.network-control`,
    {...service, user: 'remote', pid: 102}, 'not handled');
expect('not with no display session', `${NM}.network-control`,
    {...service, user: 'nobody', pid: 103}, 'not handled');
expect('not when the display session isn\'t active', `${NM}.network-control`,
    {...service, user: 'gone', pid: 104}, 'not handled');

desktopsOn = false;
expect('nothing while the desktops are off', `${NM}.network-control`, {}, 'not handled');
expect('not for a desktop service either', `${NM}.network-control`, service, 'not handled');
desktopsOn = true;

if (failures)
    System.exit(1);
