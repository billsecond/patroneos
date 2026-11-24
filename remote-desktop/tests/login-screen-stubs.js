// Stand-ins for the GNOME Shell modules the login-screen extension imports,
// so tests/run.sh can run extension.js under plain gjs. run.sh puts this
// first, then extension.js without its imports, then login-screen-test.js.

import System from 'system';

class Extension {}

// The desktops' flag file (patrone-remote-desktop desktops on|off)
let desktopsOn = true;
const GLib = {
    FileTest: {EXISTS: 1 << 4},
    file_test: () => desktopsOn,
};

class InjectionManager {
    overrideMethod(prototype, name, createOverride) {
        prototype[name] = createOverride(prototype[name]);
    }

    clear() {}
}

class LoginDialog {
    // GNOME's own check, which the extension falls back to
    async _findConflictingSession() {
        return 'GNOME\'s own check';
    }
}

// logind's sessions: [id, uid, user, seat, path], and each one's properties
let sessions = [];
let properties = {};
let failing = false;

const LoginManager = {
    getLoginManager() {
        if (failing)
            throw new Error('no logind');
        return {
            listSessions: async () => sessions,
            getSession: path => properties[path],
        };
    },
};
