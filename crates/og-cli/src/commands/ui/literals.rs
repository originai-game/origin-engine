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
//! TypeScript is read lexically, like the lockstep lint: comments are
//! skipped, strings kept. It finds `"#rrggbb"` strings (`hex("#…")`
//! included), `{ r: <n>, g: <n>, b: <n> }` colour objects, `radius: <n>`
//! and `radii: [<n>, …]` with a number other than 0, and `font: "<name>"`
//! naming no font role. Documents (`type screen`, `type component`) are
//! parsed: a literal `#rrggbb` paint in a prop, an `@media` override or a
//! state, and a literal `border-radius` other than 0. The default
//! argument is the project's `assets/ui`.

use og_ui::prelude::*;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use og_ui::doc::{Lit, Paint, PropValue, StateValue, UiDocument};

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub col: usize,
    /// `colour`, `radius` or `font`.
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

/// The findings of a TypeScript file.
pub fn scan_ts(file: &str, src: &str) -> Vec<Finding> {
    let masked = mask(src);
    let mut out = Vec::new();
    for (k, (line, raw)) in masked.lines().zip(src.lines()).enumerate() {
        if raw.contains(OK) {
            continue;
        }
        for (col, kind, text) in scan_line(line) {
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

/// The findings of a document.
pub fn scan_doc(file: &str, doc: &UiDocument) -> Vec<Finding> {
    let lines: Vec<&str> = doc.text.lines().collect();
    let mut out = Vec::new();
    let mut push = |line: usize, kind: &'static str, text: String| {
        if lines
            .get(line.wrapping_sub(1))
            .is_some_and(|l| l.contains(OK))
        {
            return;
        }
        out.push(Finding {
            file: file.into(),
            line,
            col: 1,
            kind,
            text,
        });
    };
    for n in &doc.nodes {
        let sets = n
            .props
            .iter()
            .chain(n.at.iter().flat_map(|(_, p)| p.iter()))
            .chain(n.rules.iter().flat_map(|rule| rule.props.iter()));
        for p in sets {
            match &p.value {
                PropValue::Lit(Lit::Paint(Paint::Color(_))) => {
                    push(p.line, "colour", format!("{} {}", p.name(), p.source))
                }
                PropValue::Lit(Lit::Num(r)) if p.name() == "radius" && *r != 0.0 => {
                    push(p.line, "radius", format!("border-radius {}", p.source))
                }
                _ => {}
            }
        }
        for g in &n.groups {
            for s in &g.states {
                for v in &s.values {
                    let what = match v {
                        StateValue::Background(Paint::Color(c)) => Some(("background", c)),
                        StateValue::Ink(Paint::Color(c)) => Some(("color", c)),
                        _ => None,
                    };
                    if let Some((prop, c)) = what {
                        push(
                            n.line,
                            "colour",
                            format!("{}:{} {prop} {}", g.name, s.name, og_ui::tokens::hex(*c)),
                        );
                    }
                }
            }
        }
    }
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if !matches!(name.as_str(), "node_modules" | "tests" | "og-out")
                && !name.starts_with('.')
            {
                collect(&p, out);
            }
        } else if (name.ends_with(".ts") && !name.ends_with(".d.ts") && !name.ends_with(".test.ts"))
            || name.ends_with(".svelte")
        {
            out.push(p);
        }
    }
}

/// Every finding under `paths` (files or directories), files in order.
pub fn scan(paths: &[PathBuf]) -> Result<Vec<Finding>, String> {
    let mut files = Vec::new();
    for p in paths {
        if p.is_dir() {
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
        if f.extension().is_some_and(|e| e == "ts") {
            out.extend(scan_ts(&name, &text));
        } else if f.extension().is_some_and(|e| e == "svelte") {
            let d = UiDocument::parse(&name, &text).map_err(|e| e.join("\n"))?;
            out.extend(scan_doc(&name, &d));
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
            "og ui literals: {} colour, {} radius, {} font",
            count("colour"),
            count("radius"),
            count("font")
        );
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(usize, &'static str, String)> {
        scan_ts("a.ts", src)
            .into_iter()
            .map(|f| (f.line, f.kind, f.text))
            .collect()
    }

    /// Ways it could fail: a token or a variable reported (noise that buries
    /// the real ones), a commented-out colour reported, `radii: [0, …]` or
    /// `font: "display"` reported, or a hex inside `hex(…)` counted twice.
    #[test]
    fn finds_written_colours_radii_and_fonts_but_not_tokens() {
        let src = r##"const a = hex("#ffcc00"); // "#000000" in a comment
const b = { r: 0.05, g: 0.06, b: 0.1, a: 1 };
const c = { r, g, b };
draw({ radius: 12, radii: [r, r, r, r] });
draw({ radius: t.panel, radii: [0, 0, 0, 0] });
draw({ radii: [999, 999, 999, 999], font: "Cinzel" });
text("x", { font: "display", color: token("ink") });
/* { r: 1, g: 1, b: 1 } */ const ok = "#fff"; // style-literal-ok
const s = "#notahex";
"##;
        assert_eq!(
            kinds(src),
            [
                (1, "colour", "\"#ffcc00\"".into()),
                (2, "colour", "{ r: 0.05, g: 0.06, b: 0.1, a: 1 }".into()),
                (4, "radius", "radius: 12".into()),
                (6, "radius", "radii: [999, 999, 999, 999]".into()),
                (6, "font", "font: \"Cinzel\"".into()),
            ]
        );
    }

    /// Ways it could fail: a `var(--token)` paint reported, a literal
    /// reported again for an `@media` block that does not set it, or a
    /// zero radius reported.
    #[test]
    fn a_document_reports_literal_paint_and_radius_once() {
        let text = "<meta name=\"component\" content=\"C\">\n<column><panel id=\"A\"/><panel id=\"B\"/></column>\n<style>\n#A { background: #102030; border-radius: 6px; }\n@media (width < 600px), (height < 600px) { #A { height: 32px; } }\n#B { background: var(--panel); border-radius: 0; }\n</style>";
        let doc = UiDocument::parse("c.svelte", text).expect("parses");
        let found: Vec<(usize, &str)> = scan_doc("c.svelte", &doc)
            .iter()
            .map(|f| (f.line, f.kind))
            .collect();
        assert_eq!(found, [(4, "colour"), (4, "radius")]);
    }
}
