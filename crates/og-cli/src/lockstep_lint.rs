//! The lockstep lint `og check` runs over a game that sets og.Lockstep: the
//! JavaScript calls whose results differ between peers (docs/design.md
//! §2.10, the og-net skill's §3). Math.sin, Math.pow and the rest are
//! implementation-approximated, so V8, JavaScriptCore and SpiderMonkey
//! round them differently and a peer in another browser parts from the
//! room; Math.random and the clock differ on every peer. Each is a warning
//! naming the replacement (`math` from `@origingame/engine`, og.Rng,
//! og.Time); none fails the check, since drawing code may use them. A line
//! holding `lockstep-ok` (in a comment) is not reported.
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

const MARK: &str = "lockstep-ok";

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
    let marked: Vec<bool> = src.lines().map(|l| l.contains(MARK)).collect();
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
                        "`**` with an exponent that is not an integer literal is Math.pow, rounded differently by each JavaScript engine: under og.Lockstep use math.pow (import { math } from \"@origingame/engine\")".into(),
                    );
                }
                i += 2;
            }
            c if ident_start(c) && (i == 0 || !(ident_char(b[i - 1]) || b[i - 1] == b'.')) => {
                let at = i;
                let w = word(i);
                i += w.len();
                let j = skip_ws(i);
                if b.get(j) != Some(&b'.') {
                    continue;
                }
                let k = skip_ws(j + 1);
                let member = word(k);
                match (w, member) {
                    ("Math", "random") => push(
                        at,
                        9002,
                        "Math.random() differs on every peer: under og.Lockstep draw from og.Rng (world.resource(og.Rng)), seeded the same in the room".into(),
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
                                    "Math.{m} is rounded differently by each JavaScript engine, so a peer in another browser parts from the room under og.Lockstep: {fix}"
                                ),
                            );
                        }
                    }
                    ("Date" | "performance", "now") => push(
                        at,
                        9003,
                        format!(
                            "{w}.now() is this peer's clock: under og.Lockstep a system that changes the world reads og.Time (its tick and dt) instead"
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
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
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

/// The findings in the sources under `dir` (paths relative to `root`), or
/// none when no source sets og.Lockstep up.
pub fn lint(root: &Path, dir: &Path) -> Vec<(String, Finding)> {
    let mut files = Vec::new();
    sources(dir, &mut files);
    let texts: Vec<(PathBuf, String)> = files
        .into_iter()
        .filter_map(|p| std::fs::read_to_string(&p).ok().map(|t| (p, t)))
        .collect();
    if !texts.iter().any(|(_, t)| uses_lockstep(t)) {
        return Vec::new();
    }
    texts
        .iter()
        .flat_map(|(p, t)| {
            let rel = crate::project::typescript_path(p.strip_prefix(root).unwrap_or(p));
            scan(t).into_iter().map(move |f| (rel.clone(), f))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(src: &str) -> Vec<(u32, u32, u32)> {
        scan(src)
            .into_iter()
            .map(|f| (f.line, f.col, f.code))
            .collect()
    }

    #[test]
    fn approximated_math_random_and_the_clock_are_found_where_they_are() {
        let src = "const a = Math.sin(t) + Math.sqrt(2);\n  let r = Math.random();\nconst n = Date.now() + performance.now();\nconst p = Math . pow(x, y);\n";
        assert_eq!(
            codes(src),
            [
                (1, 11, 9001),
                (2, 11, 9002),
                (3, 11, 9003),
                (3, 24, 9003),
                (4, 11, 9001)
            ]
        );
        assert!(scan(src)[0].message.contains("math.sin"));
        assert!(
            scan("Math.cbrt(8)")[0]
                .message
                .contains("math.exp, math.log")
        );
    }

    #[test]
    fn exact_math_comments_strings_and_lookalikes_are_not() {
        let src = concat!(
            "// Math.sin(x) in a comment\n",
            "/* Math.pow(2, 3) */ const s = \"Math.cos(1)\" + 'Date.now()';\n",
            "const m = Math.floor(x) + Math.abs(y) + Math.round(z) + Math.max(a, b);\n",
            "const k = og.math.sin(x) + MyMath.sin(x) + math.pow(x, 2) + Mathx.sin(1);\n",
            "const q = Math.sin(t); // lockstep-ok: drawing only\n",
            "const sq = x ** 2 + y**3; x **= 2;\n",
        );
        assert_eq!(codes(src), []);
    }

    #[test]
    fn template_text_is_skipped_and_its_expressions_are_read() {
        let src = "const s = `Math.sin(${Math.cos(a)}) {${ {b: 1}.b }} Math.pow`; Math.exp(1);";
        let found = codes(src);
        assert_eq!(found, [(1, 23, 9001), (1, 64, 9001)], "{found:?}");
    }

    #[test]
    fn a_power_that_is_not_an_integer_literal() {
        assert_eq!(codes("const a = x ** 0.5;"), [(1, 13, 9004)]);
        assert_eq!(codes("const a = x ** y;"), [(1, 13, 9004)]);
        assert_eq!(codes("const a = x ** 1e3;"), [(1, 13, 9004)]);
        assert_eq!(codes("const a = 2 ** 10;"), []);
    }

    #[test]
    fn only_a_game_with_lockstep_is_linted() {
        let dir = std::env::temp_dir().join(format!("og-lockstep-lint-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(src.join("systems")).unwrap();
        std::fs::write(src.join("main.ts"), "export const wobble = Math.sin(1);\n").unwrap();
        std::fs::write(
            src.join("types.d.ts"),
            "declare const x: number; // Math.sin\n",
        )
        .unwrap();
        assert!(lint(&dir, &src).is_empty(), "no og.Lockstep: nothing");
        std::fs::write(
            src.join("systems/net.ts"),
            "const lock = world.resource(og.Lockstep);\nconst r = Math.random();\n",
        )
        .unwrap();
        let found: Vec<(String, u32, u32)> = lint(&dir, &src)
            .into_iter()
            .map(|(f, x)| (f, x.line, x.code))
            .collect();
        assert_eq!(
            found,
            [
                ("src/main.ts".to_string(), 1, 9001),
                ("src/systems/net.ts".to_string(), 2, 9002)
            ]
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
