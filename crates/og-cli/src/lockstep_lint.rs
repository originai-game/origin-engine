//! The determinism lint `og check` runs over every game: the JavaScript a
//! game can reach that the engine does not own, so that a replay, a save
//! restored on the other host, a playtest's end hash or a lockstep peer
//! differs (docs/design.md §2.4 and §2.10).
//! Game code runs in a Worker or in bun with the whole of JavaScript
//! in scope: nothing is hidden from it, so this report is what tells an
//! author. Math.sin, Math.pow and the rest are implementation-approximated,
//! so V8, JavaScriptCore and SpiderMonkey round them differently;
//! Math.random, the clock and timers differ on every run; `fetch`, sockets
//! and the browser's storage reach outside the host ABI, where neither the
//! replay nor the platform sees them; `localeCompare`, the `toLocale...`
//! methods and Intl order and format by the host's locale and Unicode data,
//! and WeakRef and FinalizationRegistry observe the garbage collector.
//! Each is a warning naming the
//! replacement (`math` from `@origingame/engine`, og.Rng, og.Time,
//! og.platform); none fails the check, since drawing code may use the maths
//! and the clock. A line holding `og-ok` or `lockstep-ok` (in a comment) is
//! not reported.
//!
//! The scan is lexical: comments, strings and template text are skipped, an
//! identifier is matched whole, and `x ** y` is reported unless y is an
//! integer literal. It reads the source files under the entry's directory.

use std::path::{Path, PathBuf};

/// One finding: 1-based line and column, a code and the message.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub line: u32,
    pub col: u32,
    pub code: u32,
    pub message: String,
}

/// Math members that differ between JavaScript engines, with their
/// replacement in `math` when it has one.
const APPROXIMATED: &[(&str, Option<&str>)] = &[
    ("sin", Some("math.sin")),
    ("cos", Some("math.cos")),
    ("tan", Some("math.tan")),
    ("asin", Some("math.asin")),
    ("acos", Some("math.acos")),
    ("atan", Some("math.atan")),
    ("atan2", Some("math.atan2")),
    ("exp", Some("math.exp")),
    ("log", Some("math.log")),
    ("pow", Some("math.pow")),
    ("sinh", None),
    ("cosh", None),
    ("tanh", None),
    ("asinh", None),
    ("acosh", None),
    ("atanh", None),
    ("expm1", None),
    ("log1p", None),
    ("log2", None),
    ("log10", None),
    ("cbrt", None),
    ("hypot", None),
];

const MARKS: [&str; 2] = ["lockstep-ok", "og-ok"];

/// Globals that reach outside the host ABI when called or constructed.
const HOST_CALLS: &[&str] = &["fetch", "WebSocket", "XMLHttpRequest", "EventSource"];
/// The browser's storage, reached through a member.
const HOST_STORES: &[&str] = &["localStorage", "sessionStorage", "indexedDB"];
/// Timers: they fire by the host's clock, outside the tick.
const TIMERS: &[&str] = &["setTimeout", "setInterval", "requestAnimationFrame"];
/// String, number and date methods that answer by the host's locale and
/// its Unicode data.
const LOCALE_MEMBERS: &[&str] = &[
    "localeCompare",
    "toLocaleString",
    "toLocaleUpperCase",
    "toLocaleLowerCase",
    "toLocaleDateString",
    "toLocaleTimeString",
];
/// What observes the garbage collector.
const COLLECTED: &[&str] = &["WeakRef", "FinalizationRegistry"];
/// What names the global object.
const GLOBALS: &[&str] = &["globalThis", "self", "window"];

fn host_reach(name: &str) -> String {
    format!(
        "{name} reaches outside the host ABI: a replay does not record it, the other host may not have it, and the platform sees nothing of it; saves, scores, rooms and purchases go through og.platform, assets through og.load"
    )
}

fn locale(name: &str) -> String {
    format!(
        "{name} orders and formats by the host's locale and its Unicode data, different on another machine or browser, so a sort or a string built with it differs between peers and on the other host: where a system that changes the world runs it, compare strings with < and > (code units, the same everywhere) and build numbers' text yourself"
    )
}

fn timer(name: &str) -> String {
    format!(
        "{name} fires by the host's clock, outside the tick, so a replay does not repeat it: count ticks in a component or a resource (og.Time), or use og.tween"
    )
}

fn ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}

fn ident_char(c: u8) -> bool {
    ident_start(c) || c.is_ascii_digit()
}

/// The findings in one source text.
pub fn scan(src: &str) -> Vec<Finding> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    // Line starts, for positions.
    let mut starts = vec![0usize];
    for (i, &c) in b.iter().enumerate() {
        if c == b'\n' {
            starts.push(i + 1);
        }
    }
    let pos = |at: usize| -> (u32, u32) {
        let line = starts.partition_point(|&s| s <= at);
        (line as u32, (at - starts[line - 1] + 1) as u32)
    };
    let marked: Vec<bool> = src
        .lines()
        .map(|l| MARKS.iter().any(|m| l.contains(m)))
        .collect();
    let mut push = |at: usize, code: u32, message: String| {
        let (line, col) = pos(at);
        if !marked.get(line as usize - 1).copied().unwrap_or(false) {
            out.push(Finding {
                line,
                col,
                code,
                message,
            });
        }
    };
    // Each open template literal's brace depth at its `${`.
    let mut templates: Vec<u32> = Vec::new();
    let mut depth = 0u32;
    let mut i = 0;
    let skip_ws = |mut j: usize| {
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        j
    };
    let word = |j: usize| {
        let mut k = j;
        while k < b.len() && ident_char(b[k]) {
            k += 1;
        }
        &src[j..k]
    };
    // Inside a template's text from `i` to its end or `${`: returns where code resumes.
    let template_text = |mut j: usize, templates: &mut Vec<u32>, depth: u32| {
        while j < b.len() {
            match b[j] {
                b'\\' => j += 2,
                b'`' => return j + 1,
                b'$' if b.get(j + 1) == Some(&b'{') => {
                    templates.push(depth);
                    return j + 2;
                }
                _ => j += 1,
            }
        }
        j
    };
    while i < b.len() {
        let c = b[i];
        match c {
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i < b.len() && !(b[i] == b'*' && b.get(i + 1) == Some(&b'/')) {
                    i += 1;
                }
                i += 2;
            }
            b'"' | b'\'' => {
                i += 1;
                while i < b.len() && b[i] != c && b[i] != b'\n' {
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
                i += 1;
            }
            b'`' => i = template_text(i + 1, &mut templates, depth),
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                if templates.last() == Some(&depth) {
                    templates.pop();
                    i = template_text(i + 1, &mut templates, depth);
                } else {
                    depth = depth.saturating_sub(1);
                    i += 1;
                }
            }
            b'*' if b.get(i + 1) == Some(&b'*') && b.get(i + 2) != Some(&b'=') => {
                let at = i;
                let j = skip_ws(i + 2);
                let mut k = j;
                while k < b.len() && (b[k].is_ascii_digit() || b[k] == b'_') {
                    k += 1;
                }
                let integer = k > j && !(k < b.len() && (ident_char(b[k]) || b[k] == b'.'));
                if !integer {
                    push(
                        at,
                        9004,
                        "`**` with an exponent that is not an integer literal is Math.pow, rounded differently by each JavaScript engine: use math.pow (import { math } from \"@origingame/engine\")".into(),
                    );
                }
                i += 2;
            }
            c if ident_start(c) && i > 0 && b[i - 1] == b'.' => {
                // A member's name: the locale's methods, on any value.
                let at = i;
                let w = word(i);
                i += w.len();
                if LOCALE_MEMBERS.contains(&w) && b.get(skip_ws(i)) == Some(&b'(') {
                    push(at, 9007, locale(w));
                }
            }
            c if ident_start(c) && (i == 0 || !ident_char(b[i - 1])) => {
                let at = i;
                let w = word(i);
                i += w.len();
                let j = skip_ws(i);
                if b.get(j) == Some(&b'(') {
                    // A call or a construction of a global; `function fetch(`
                    // declares the game's own.
                    let before = src[..at].trim_end();
                    if before.ends_with("function") {
                        continue;
                    }
                    if w == "Date"
                        && b.get(skip_ws(j + 1)) == Some(&b')')
                        && before.ends_with("new")
                    {
                        push(
                            at,
                            9003,
                            "new Date() is the host's clock, different on every run: a system that changes the world reads og.Time (its tick and dt); a date to show the player is og.platform.now".into(),
                        );
                    } else if HOST_CALLS.contains(&w) {
                        push(at, 9005, host_reach(w));
                    } else if TIMERS.contains(&w) {
                        push(at, 9006, timer(w));
                    } else if COLLECTED.contains(&w) && before.ends_with("new") {
                        push(
                            at,
                            9007,
                            format!(
                                "{w} observes the garbage collector, which runs at different times on every run and every peer: hold the object, or keep its handle in a component"
                            ),
                        );
                    }
                    continue;
                }
                if b.get(j) != Some(&b'.') {
                    continue;
                }
                let k = skip_ws(j + 1);
                let member = word(k);
                if HOST_STORES.contains(&w) {
                    push(at, 9005, host_reach(w));
                    continue;
                }
                if GLOBALS.contains(&w) {
                    if HOST_CALLS.contains(&member) || HOST_STORES.contains(&member) {
                        push(at, 9005, host_reach(member));
                    } else if TIMERS.contains(&member) {
                        push(at, 9006, timer(member));
                    }
                    continue;
                }
                match (w, member) {
                    ("crypto", "getRandomValues" | "randomUUID") => push(
                        at,
                        9002,
                        format!(
                            "crypto.{member} differs on every run and every peer: draw from og.Rng (world.resource(og.Rng)), seeded by the engine"
                        ),
                    ),
                    ("Intl", m) if !m.is_empty() => push(at, 9007, locale(&format!("Intl.{m}"))),
                    ("navigator", "sendBeacon") => push(at, 9005, host_reach("navigator.sendBeacon")),
                    ("Math", "random") => push(
                        at,
                        9002,
                        "Math.random() differs on every run and every peer: draw from og.Rng (world.resource(og.Rng)), seeded by the engine and the same in a room".into(),
                    ),
                    ("Math", m) => {
                        if let Some((_, fix)) = APPROXIMATED.iter().find(|(n, _)| *n == m) {
                            let fix = match fix {
                                Some(f) => format!("use {f} (import {{ math }} from \"@origingame/engine\")"),
                                None => "write it with math.exp, math.log and math.sqrt (import { math } from \"@origingame/engine\")".into(),
                            };
                            push(
                                at,
                                9001,
                                format!(
                                    "Math.{m} is rounded differently by each JavaScript engine, so a replay on the other host differs and a peer in another browser parts from the room: {fix}"
                                ),
                            );
                        }
                    }
                    ("Date" | "performance", "now") => push(
                        at,
                        9003,
                        format!(
                            "{w}.now() is the host's clock, different on every run and every peer: a system that changes the world reads og.Time (its tick and dt) instead"
                        ),
                    ),
                    _ => {}
                }
            }
            _ => i += 1,
        }
    }
    out
}

/// Whether a source text sets og.Lockstep up (the game plays over a room).
pub fn uses_lockstep(src: &str) -> bool {
    src.contains("og.Lockstep")
}

/// The game's source files: `.ts` and `.tsx` under `dir`, sorted, without
/// `node_modules`, dot directories and `.d.ts` files.
pub(crate) fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if name != "node_modules" && !name.starts_with('.') {
                sources(&p, out);
            }
        } else if (name.ends_with(".ts") || name.ends_with(".tsx")) && !name.ends_with(".d.ts") {
            out.push(p);
        }
    }
}

/// The findings in the sources under `dir` (paths relative to `root`), and
/// whether a source sets og.Lockstep up (the report then speaks of peers).
pub fn lint(root: &Path, dir: &Path) -> (bool, Vec<(String, Finding)>) {
    let mut files = Vec::new();
    sources(dir, &mut files);
    let texts: Vec<(PathBuf, String)> = files
        .into_iter()
        .filter_map(|p| std::fs::read_to_string(&p).ok().map(|t| (p, t)))
        .collect();
    let lockstep = texts.iter().any(|(_, t)| uses_lockstep(t));
    let found = texts
        .iter()
        .flat_map(|(p, t)| {
            let rel = crate::project::typescript_path(p.strip_prefix(root).unwrap_or(p));
            scan(t).into_iter().map(move |f| (rel.clone(), f))
        })
        .collect();
    (lockstep, found)
}
