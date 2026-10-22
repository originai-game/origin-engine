//! Signals from outside reach og's child: SIGINT, SIGTERM and SIGHUP sent to
//! og (a supervisor stopping it, a closed terminal, Ctrl-C) are passed on to
//! the bun process og runs, or to its whole process group, so neither og
//! dies and leaves the child running nor the child dies before it cleaned
//! up. `og run --device` passes them to its bun process, which undoes its
//! setup; `og dev` starts its game in a process group of its own and passes
//! them to the group, so whatever the game process started (a Worker's
//! process, a browser, a server) stops with it.

/// Where a signal goes: one process, or every process of a group.
#[derive(Clone, Copy, Debug)]
// Windows console events reach attached children directly, not through POSIX pids.
#[cfg_attr(not(unix), allow(dead_code))]
pub enum Target {
    Process(u32),
    /// A process group, by its leader's pid (a child spawned with
    /// `process_group(0)`).
    Group(u32),
}

#[cfg(unix)]
mod imp {
    use super::Target;
    use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

    /// What `kill` is given: the child's pid, or minus its group's (0: none).
    static CHILD: AtomicI32 = AtomicI32::new(0);
    /// The last signal that came, 0 before one did.
    static GOT: AtomicI32 = AtomicI32::new(0);
    /// The handlers the signals had before `start`, restored by `stop`.
    static PREVIOUS: [AtomicUsize; 3] = [
        AtomicUsize::new(0),
        AtomicUsize::new(0),
        AtomicUsize::new(0),
    ];
    pub const SIGNALS: [libc::c_int; 3] = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP];

    extern "C" fn pass_on(sig: libc::c_int) {
        GOT.store(sig, Ordering::SeqCst);
        let to = CHILD.load(Ordering::SeqCst);
        // SAFETY: kill, signal and raise are async-signal-safe.
        unsafe {
            if to != 0 {
                libc::kill(to, sig);
            } else {
                libc::signal(sig, libc::SIG_DFL);
                libc::raise(sig);
            }
        }
    }

    fn pid_of(target: Target) -> i32 {
        match target {
            Target::Process(pid) => pid as i32,
            Target::Group(pid) => -(pid as i32),
        }
    }

    pub fn start(target: Target) {
        CHILD.store(pid_of(target), Ordering::SeqCst);
        for (sig, prev) in SIGNALS.iter().zip(&PREVIOUS) {
            // SAFETY: installs a handler that only calls async-signal-safe functions.
            let old = unsafe {
                libc::signal(
                    *sig,
                    pass_on as extern "C" fn(libc::c_int) as libc::sighandler_t,
                )
            };
            prev.store(old, Ordering::SeqCst);
        }
    }

    pub fn stop() {
        for (sig, prev) in SIGNALS.iter().zip(&PREVIOUS) {
            // SAFETY: puts back the handler `start` replaced.
            unsafe { libc::signal(*sig, prev.load(Ordering::SeqCst)) };
        }
        CHILD.store(0, Ordering::SeqCst);
    }

    pub fn received() -> Option<i32> {
        match GOT.load(Ordering::SeqCst) {
            0 => None,
            s => Some(s),
        }
    }

    pub fn send(target: Target, sig: i32) {
        // SAFETY: kill has no memory effects.
        unsafe { libc::kill(pid_of(target), sig) };
    }

    pub fn alive(target: Target) -> bool {
        // SAFETY: signal 0 only checks that the target exists.
        unsafe { libc::kill(pid_of(target), 0) == 0 }
    }

    pub fn reraise(sig: i32) -> ! {
        // SAFETY: back to the default action, which ends the process.
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
            libc::raise(sig);
        }
        std::process::exit(128 + sig)
    }

    pub const KILL: i32 = libc::SIGKILL;
}

#[cfg(not(unix))]
mod imp {
    use super::Target;
    pub fn start(_target: Target) {}
    pub fn stop() {}
    pub fn received() -> Option<i32> {
        None
    }
    pub fn send(_target: Target, _sig: i32) {}
    pub fn alive(_target: Target) -> bool {
        false
    }
    pub fn reraise(sig: i32) -> ! {
        std::process::exit(128 + sig)
    }
    pub const KILL: i32 = 9;
}

/// From now on SIGINT, SIGTERM and SIGHUP to og go to `target` (and are
/// remembered for [`received`]); before a child exists, og's own default
/// action applies.
pub fn start(target: Target) {
    imp::start(target);
}

/// The handlers before [`start`] again.
pub fn stop() {
    imp::stop();
}

/// The signal og was sent since [`start`], if one was.
pub fn received() -> Option<i32> {
    imp::received()
}

/// After a signal: waits up to `grace` for `target` to end (it was sent the
/// signal already), then kills what is left of it, then ends og as the
/// signal would have (its default action, so a supervisor sees the signal).
pub fn finish(
    target: Target,
    sig: i32,
    grace: std::time::Duration,
    mut ended: impl FnMut() -> bool,
) -> ! {
    let t0 = std::time::Instant::now();
    // `ended` reaps the child; a group lives on while any of it does.
    while !(ended() && !imp::alive(target)) && t0.elapsed() < grace {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if imp::alive(target) {
        imp::send(target, imp::KILL);
        let t1 = std::time::Instant::now();
        while !ended() && t1.elapsed() < std::time::Duration::from_secs(2) {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
    stop();
    imp::reraise(sig)
}
