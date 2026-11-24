//! `og ui literals [path...] [--json]` (docs/ui.md §6, §5): the style
//! values written into screens and components (the engine's built-in ones
//! or a game's) instead of taken from the style. A component holds
//! structure and behaviour; its look comes from a style pack's tokens
//! and part dressing, so a colour, a corner radius or a font family it
//! writes itself is a value no style pack can change. Each finding names
//! the file, line and column, what was found and the token to use.
//!
//! It reports and never refuses: it exits 0 when it ran, whatever it found,
//! and 1 when it could not read a path. A line holding `style-literal-ok`
//! (in a comment) is not reported, for a value that is not the style's (a
//! physical black, a debug colour).
//!
//! TypeScript and TSX are read lexically, like the lockstep lint: comments
//! are skipped, strings kept. It finds `"#rrggbb"` strings (`hex("#…")`
//! included), `{ r: <n>, g: <n>, b: <n> }` colour objects, `radius: <n>`
//! and `radii: [<n>, …]` with a number other than 0, `font: "<name>"`
//! naming no font role, and in class strings an arbitrary hex colour
//! (`bg-[#0af]`), a fixed corner (`rounded-lg`, `rounded-t-md`) and the
//! retired cuts (`nine-socket`, `nine-panel-cut`). The walk never follows
//! a directory symlink and skips `.og/`, `cooked/`, `og-out/` and
//! installs. The default argument is the project's `assets/ui`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub col: usize,
    /// `colour`, `radius`, `font` or `class`.
    pub kind: &'static str,
    /// The value as written.
    pub text: String,
}

impl Finding {
    fn hint(&self) -> &'static str {
        match (self.kind, self.file.ends_with(".ogt")) {
            ("colour", false) => "a palette role: token(\"panel\")",
            ("colour", true) => "a palette role: var(--panel)",
            ("radius", false) => {
                "the style's radius scale (og.ui.frameTheme().tokens) or a dressed part"
            }
            // A document's border-radius takes px only: its corner is the
            // part's, dressed in the style pack.
            ("radius", true) => "a part (`part frame`) whose corner the style pack dresses",
            ("class", _) => {
                "a token class: a palette role (bg-surface, text-fg) or a radius role (rounded-control, rounded-card, rounded-panel, rounded-capsule)"
            }
            _ => "a font role: body, display, mono or rounded",
        }
    }
}

const FONT_ROLES: &[&str] = &["body", "display", "mono", "rounded"];
const OK: &str = "style-literal-ok";

/// The source with its comments blanked (newlines kept), its strings and
/// template text as they are.
fn mask(src: &str) -> String {
    #[derive(PartialEq)]
    enum S {
        Code,
        Line,
        Block,
        Str(char),
    }
    let mut s = S::Code;
    let mut out = String::with_capacity(src.len());
    let mut it = src.chars().peekable();
    while let Some(c) = it.next() {
        match s {
            S::Code => match c {
                '/' if it.peek() == Some(&'/') => {
                    s = S::Line;
                    out.push(' ');
                }
                '/' if it.peek() == Some(&'*') => {
                    s = S::Block;
                    out.push(' ');
                }
                '"' | '\'' | '`' => {
                    s = S::Str(c);
                    out.push(c);
                }
                c => out.push(c),
            },
            S::Line => {
                if c == '\n' {
                    s = S::Code;
                    out.push('\n');
                } else {
                    out.push(' ');
                }
            }
            S::Block => {
                if c == '*' && it.peek() == Some(&'/') {
                    it.next();
                    s = S::Code;
                    out.push_str("  ");
                } else {
                    out.push(if c == '\n' { '\n' } else { ' ' });
                }
            }
            S::Str(q) => {
                out.push(c);
                if c == '\\' {
                    if let Some(n) = it.next() {
                        out.push(n);
                    }
                } else if c == q || (c == '\n' && q != '`') {
                    s = S::Code;
                }
            }
        }
    }
    out
}

fn ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

fn ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    i
}

/// A numeric literal at `i`: its end and whether it is zero.
fn number(b: &[u8], i: usize) -> Option<(usize, bool)> {
    let mut j = i;
    while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'.') {
        j += 1;
    }
    if j == i || (j < b.len() && ident(b[j])) {
        return None;
    }
    let zero = b[i..j].iter().all(|c| *c == b'0' || *c == b'.');
    Some((j, zero))
}

/// `key` as a whole word at `i`, then `:`; the index after the colon.
fn key(b: &[u8], i: usize, key: &str) -> Option<usize> {
    let k = key.as_bytes();
    if !b[i..].starts_with(k) || (i > 0 && (ident(b[i - 1]) || b[i - 1] == b'.')) {
        return None;
    }
    let j = ws(b, i + k.len());
    (j < b.len() && b[j] == b':').then(|| ws(b, j + 1))
}

/// The findings of one line of masked TypeScript (0-based columns).
fn scan_line(line: &str) -> Vec<(usize, &'static str, String)> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        // "#rrggbb"
        if (c == b'"' || c == b'\'') && b.get(i + 1) == Some(&b'#') {
            let mut j = i + 2;
            while j < b.len() && b[j].is_ascii_hexdigit() {
                j += 1;
            }
            if j < b.len() && b[j] == c && matches!(j - i - 2, 3 | 4 | 6 | 8) {
                out.push((i, "colour", line[i..=j].to_string()));
                i = j + 1;
                continue;
            }
        }
        // { r: 0.1, g: 0.2, b: …
        if c == b'{' {
            let mut j = ws(b, i + 1);
            let mut ok = true;
            for k in ["r", "g", "b"] {
                match key(b, j, k).and_then(|n| number(b, n)) {
                    Some((end, _)) => {
                        j = ws(b, end);
                        if k != "b" {
                            if j < b.len() && b[j] == b',' {
                                j = ws(b, j + 1);
                            } else {
                                ok = false;
                                break;
                            }
                        }
                    }
                    None => {
                        ok = false;
                        break;
                    }
                }
            }
            if ok {
                let end = line[i..].find('}').map_or(j, |e| i + e + 1);
                out.push((i, "colour", line[i..end].to_string()));
                i = end;
                continue;
            }
        }
        if let Some(n) = key(b, i, "radius")
            && let Some((end, false)) = number(b, n)
        {
            out.push((i, "radius", line[i..end].to_string()));
            i = end;
            continue;
        }
        if let Some(n) = key(b, i, "radii")
            && n < b.len()
            && b[n] == b'['
        {
            let close = line[n..].find(']').map(|e| n + e + 1);
            let lit = line[n + 1..close.map_or(b.len(), |e| e - 1)]
                .split(',')
                .any(|v| {
                    let v = v.trim().as_bytes();
                    number(v, 0).is_some_and(|(e, zero)| e == v.len() && !zero)
                });
            if lit && let Some(end) = close {
                out.push((i, "radius", line[i..end].to_string()));
                i = end;
                continue;
            }
        }
        if let Some(n) = key(b, i, "font")
            && n < b.len()
            && (b[n] == b'"' || b[n] == b'\'')
            && let Some(e) = line[n + 1..].find(b[n] as char)
        {
            let name = &line[n + 1..n + 1 + e];
            if !FONT_ROLES.contains(&name) {
                out.push((i, "font", line[i..n + 2 + e].to_string()));
            }
            i = n + 2 + e;
            continue;
        }
        i += 1;
    }
    out
}

/// A class utility that writes a look no pack can change: an arbitrary
/// hex colour, a fixed corner or a retired nine cut.
fn literal_class(token: &str) -> bool {
    // The utility after its variants (`hover:`, `data-on:`), outside brackets.
    let mut depth = 0i32;
    let mut start = 0;
    for (i, c) in token.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            ':' if depth == 0 => start = i + 1,
            _ => {}
        }
    }
    let u = token[start..].trim_start_matches(['-', '!']);
    if u.contains("-[#") {
        return true;
    }
    if matches!(
        u,
        "nine-socket" | "nine-panel-cut" | "nine-plate-cut" | "nine-chip-cut"
    ) {
        return true;
    }
    const SIDES: &[&str] = &[
        "t", "r", "b", "l", "s", "e", "tl", "tr", "bl", "br", "ss", "se", "es", "ee",
    ];
    const SIZES: &[&str] = &["xs", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl"];
    if u == "rounded" {
        return true;
    }
    let Some(rest) = u.strip_prefix("rounded-") else {
        return false;
    };
    if SIZES.contains(&rest) || SIDES.contains(&rest) || rest.starts_with('[') {
        return true;
    }
    rest.split_once('-').is_some_and(|(side, size)| {
        SIDES.contains(&side) && (SIZES.contains(&size) || size.starts_with('['))
    })
}

/// The class findings of one line of masked source: the tokens inside its
/// strings and template text (0-based columns).
fn scan_classes(line: &str) -> Vec<(usize, &'static str, String)> {
    let mut out = Vec::new();
    let mut quote: Option<char> = None;
    let mut tok_start: Option<usize> = None;
    let flush = |from: Option<usize>, to: usize, out: &mut Vec<(usize, &'static str, String)>| {
        if let Some(f) = from {
            let t = &line[f..to];
            if literal_class(t) {
                out.push((f, "class", t.to_string()));
            }
        }
    };
    for (i, c) in line.char_indices() {
        match quote {
            None => {
                if matches!(c, '"' | '\'' | '`') {
                    quote = Some(c);
                }
            }
            Some(q) => {
                if c == q || c.is_whitespace() || c == '$' || c == '{' || c == '}' {
                    flush(tok_start.take(), i, &mut out);
                    if c == q {
                        quote = None;
                    }
                } else if tok_start.is_none() {
                    tok_start = Some(i);
                }
            }
        }
    }
    flush(tok_start.take(), line.len(), &mut out);
    out
}

/// The findings of a TypeScript file.
pub fn scan_ts(file: &str, src: &str) -> Vec<Finding> {
    let masked = mask(src);
    let mut out = Vec::new();
    for (k, (line, raw)) in masked.lines().zip(src.lines()).enumerate() {
        if raw.contains(OK) {
            continue;
        }
        let mut found = scan_line(line);
        found.extend(scan_classes(line));
        found.sort_by_key(|f| f.0);
        for (col, kind, text) in found {
            out.push(Finding {
                file: file.into(),
                line: k + 1,
                col: line[..col].chars().count() + 1,
                kind,
                text,
            });
        }
    }
    out
}

/// A TypeScript or TSX source, not a declaration or a test.
fn source(name: &str) -> bool {
    (name.ends_with(".ts") || name.ends_with(".tsx"))
        && !name.ends_with(".d.ts")
        && !name.ends_with(".test.ts")
        && !name.ends_with(".test.tsx")
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        // The entry's own type: a directory symlink is never followed.
        let Ok(kind) = e.file_type() else {
            continue;
        };
        if kind.is_dir() {
            if !matches!(
                name.as_str(),
                "node_modules" | "tests" | "og-out" | "cooked"
            ) && !name.starts_with('.')
            {
                collect(&p, out);
            }
        } else if kind.is_file() && source(&name) {
            out.push(p);
        }
    }
}

/// Every finding under `paths` (files or directories), files in order.
pub fn scan(paths: &[PathBuf]) -> Result<Vec<Finding>, String> {
    let mut files = Vec::new();
    for p in paths {
        let meta = std::fs::symlink_metadata(p);
        if meta.as_ref().is_ok_and(|m| m.is_dir()) {
            collect(p, &mut files);
        } else if p.is_file() {
            files.push(p.clone());
        } else {
            return Err(format!("{}: no such file or directory", p.display()));
        }
    }
    files.sort();
    files.dedup();
    let mut out = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).map_err(|e| format!("{}: {e}", f.display()))?;
        let name = f.to_string_lossy().replace('\\', "/");
        if f.extension().is_some_and(|e| e == "ts" || e == "tsx") {
            out.extend(scan_ts(&name, &text));
        }
    }
    Ok(out)
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut json = false;
    let mut paths: Vec<PathBuf> = Vec::new();
    for a in args {
        match *a {
            "--json" => json = true,
            a if a.starts_with('-') => {
                eprintln!(
                    "og ui literals: unknown option `{a}`. usage: og ui literals [path...] [--json]"
                );
                return ExitCode::from(2);
            }
            a => paths.push(a.into()),
        }
    }
    if paths.is_empty() {
        paths = ["assets/ui"]
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .collect();
        if paths.is_empty() {
            eprintln!("og ui literals: no assets/ui here; name the documents to read");
            return ExitCode::FAILURE;
        }
    }
    let found = match scan(&paths) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("og ui literals: {e}");
            return ExitCode::FAILURE;
        }
    };
    if json {
        let rows: Vec<serde_json::Value> = found
            .iter()
            .map(|f| {
                serde_json::json!({
                    "file": f.file, "line": f.line, "col": f.col,
                    "kind": f.kind, "text": f.text, "use": f.hint(),
                })
            })
            .collect();
        println!("{}", serde_json::Value::Array(rows));
    } else {
        for f in &found {
            println!(
                "{}:{}:{}: {} `{}`: use {}",
                f.file,
                f.line,
                f.col,
                f.kind,
                f.text,
                f.hint()
            );
        }
        let count = |k: &str| found.iter().filter(|f| f.kind == k).count();
        println!(
            "og ui literals: {} colour, {} radius, {} font, {} class",
            count("colour"),
            count("radius"),
            count("font"),
            count("class")
        );
    }
    ExitCode::SUCCESS
}
