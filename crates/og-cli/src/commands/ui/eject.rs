//! `og ui eject <name> [project] [--force]` copies one of the engine's TSX
//! components, screens or controls into the game, as the game's own file,
//! and points the game's imports of it at the copy. `og ui eject --list
//! [project]` prints every name with the first line of its doc comment.
//!
//! A name is the file's path under the engine package's `src/ui/components/`
//! (`hud/stat-bar`, `card/hand`), or `screens/<name>` or `controls/<name>`.
//! The copy is `assets/ui/<name>.tsx`:
//!
//! - its relative imports become the engine's package paths
//!   (`../controls/button` is `@origingame/engine/ui/controls/button`), so it
//!   keeps using the built-in pieces it composes;
//! - the strings it reads from its tables are copied into one table beside
//!   it, `assets/ui/<name>.en.ogt`, each key under `game.` (`card.hand.play`
//!   is `game.card.hand.play`, in the table and in the code), so the game's
//!   text never competes with the engine's table for a key;
//! - every import of it in the game's `.ts`/`.tsx` (by name from
//!   `@origingame/engine/ui/components`, or by its package path) imports the
//!   copy instead.
//!
//! It writes nothing when the copy or its table exists, unless `--force`; an
//! unknown name exits 1 with the closest names.

use crate::project;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const BARREL: &str = "@origingame/engine/ui/components";
const PACKAGE_UI: &str = "@origingame/engine/ui/";
/// The prefix a copied table's keys take.
const PREFIX: &str = "game.";

/// One ejectable file: its name, its path under `src/ui`, its exported
/// function and the first line of its doc comment.
#[derive(Debug)]
pub(super) struct Entry {
    pub name: String,
    rel: String,
    summary: String,
}

/// Every component, screen and control of the package's `src/ui`, by name.
pub(super) fn library(ui: &Path) -> Vec<Entry> {
    let mut out = Vec::new();
    for (dir, named) in [
        ("components", ""),
        ("screens", "screens/"),
        ("controls", "controls/"),
    ] {
        let mut files = Vec::new();
        walk(&ui.join(dir), &mut files);
        for f in files {
            let Ok(rest) = f.strip_prefix(ui.join(dir)) else {
                continue;
            };
            let stem = slash(rest).trim_end_matches(".tsx").to_string();
            if stem == "index" {
                continue;
            }
            let text = std::fs::read_to_string(&f).unwrap_or_default();
            out.push(Entry {
                name: format!("{named}{stem}"),
                rel: format!("{dir}/{stem}"),
                summary: summary(&text),
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if e.file_type().is_ok_and(|t| t.is_dir()) {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "tsx") {
            out.push(p);
        }
    }
}

fn slash(p: &Path) -> String {
    p.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// The first sentence of the last doc comment before `export function`.
fn summary(text: &str) -> String {
    let head = &text[..text.find("export function").unwrap_or(0)];
    let Some(start) = head.rfind("/**") else {
        return String::new();
    };
    let body = &head[start + 3..];
    let body = &body[..body.find("*/").unwrap_or(body.len())];
    let words: Vec<&str> = body
        .lines()
        .map(|l| l.trim().trim_start_matches('*').trim())
        .filter(|l| !l.is_empty())
        .collect();
    let line = words.join(" ");
    let end = line
        .char_indices()
        .find(|&(i, c)| {
            (c == '.' || c == ':') && line[i + 1..].starts_with(' ') || i + 1 == line.len()
        })
        .map_or(line.len(), |(i, _)| i);
    line[..end].trim_end_matches(['.', ':']).to_string()
}

/// Levenshtein distance, for the closest names to a mistyped one.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let next = (row[j + 1] + 1)
                .min(row[j] + 1)
                .min(prev + usize::from(ca != *cb));
            prev = row[j + 1];
            row[j + 1] = next;
        }
    }
    row[b.len()]
}

/// The names nearest `name`, at most three: those a few edits from it, or
/// whose last segment or a word of their summary is a few edits from its
/// last segment (`settings` finds `screens/settings`).
fn closest<'a>(name: &str, lib: &'a [Entry]) -> Vec<&'a str> {
    let short = name.rsplit('/').next().unwrap_or(name);
    // How far `a` is from `b`, when near enough to suggest; containment
    // counts only for words of four letters or more.
    let near = |a: &str, b: &str| {
        let d = if b.len().min(a.len()) >= 4 && (b.contains(a) || a.contains(b)) {
            0
        } else {
            distance(a, b)
        };
        (d <= (b.len().max(a.len()) / 3).max(1)).then_some(d)
    };
    let mut scored: Vec<(usize, &str)> = lib
        .iter()
        .filter_map(|e| {
            let last = e.name.rsplit('/').next().unwrap_or(&e.name);
            let words = e
                .summary
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2)
                .filter_map(|w| near(short, w).map(|d| d + 1))
                .min();
            let d = [near(name, &e.name), near(short, last), words]
                .into_iter()
                .flatten()
                .min()?;
            Some((d, e.name.as_str()))
        })
        .collect();
    scored.sort();
    scored.into_iter().take(3).map(|(_, n)| n).collect()
}

/// What an eject wrote: the copy, its locale string tables, and the game files
/// whose imports now point at the copy.
#[derive(Debug)]
pub(super) struct Ejected {
    pub copy: PathBuf,
    pub table: Option<PathBuf>,
    pub extra_tables: Vec<PathBuf>,
    pub rewired: Vec<PathBuf>,
}

/// All exported symbols of `text` (functions, types, interfaces, and non-metadata
/// consts), requiring at least one exported function.
fn exported_symbols(text: &str, source: &Path) -> Result<BTreeSet<String>, String> {
    let mut funcs = 0usize;
    let mut out = BTreeSet::new();
    fn ident(rest: &str) -> Option<&str> {
        rest.trim_start()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
            .filter(|s| !s.is_empty())
    }
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("export function ") {
            if let Some(name) = ident(rest) {
                out.insert(name.to_string());
                funcs += 1;
            }
        } else if let Some(rest) = t
            .strip_prefix("export interface ")
            .or_else(|| t.strip_prefix("export type "))
        {
            if let Some(name) = ident(rest) {
                out.insert(name.to_string());
            }
        } else if let Some(rest) = t.strip_prefix("export const ")
            && let Some(name) = ident(rest)
            && !matches!(name, "fixtures" | "actions" | "screen")
        {
            out.insert(name.to_string());
        }
    }
    if funcs == 0 {
        return Err(format!("{}: exports no function", source.display()));
    }
    Ok(out)
}

/// Barrel aliases in `components/index.tsx` pointing to `entry_rel`
/// (e.g. `export { Menu as DropdownMenu } from "../controls/menu"` -> `DropdownMenu => Menu`).
fn barrel_aliases(ui: &Path, entry_rel: &str) -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    let index = ui.join("components/index.tsx");
    let Ok(text) = std::fs::read_to_string(&index) else {
        return map;
    };
    let comp_dir = ui.join("components");
    for line in text.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix("export {") else {
            continue;
        };
        let Some((names, tail)) = rest.split_once('}') else {
            continue;
        };
        let Some(from) = tail.trim().strip_prefix("from ").and_then(|s| {
            let q = s.chars().next().filter(|c| matches!(c, '"' | '\''))?;
            s[1..].split(q).next()
        }) else {
            continue;
        };
        let resolved = normalize(&comp_dir.join(from));
        let Ok(rel) = resolved.strip_prefix(ui) else {
            continue;
        };
        if slash(rel) != entry_rel {
            continue;
        }
        for item in names.split(',') {
            let item = item.trim().trim_start_matches("type ").trim();
            if let Some((orig, alias)) = item.split_once(" as ") {
                map.insert(alias.trim().to_string(), orig.trim().to_string());
            }
        }
    }
    map
}

fn parse_table_lines(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let rest = l.strip_prefix("string \"")?;
            let key = &rest[..rest.find('"')?];
            Some((key.to_string(), l.to_string()))
        })
        .collect()
}

fn build_locale_table(
    locale: &str,
    lines: &[(String, String)],
    used: &BTreeSet<String>,
) -> Option<String> {
    let mut out = format!("type strings\nversion 1\n---\nlocale \"{locale}\"\n");
    let mut seen = BTreeSet::new();
    for (key, line) in lines {
        if used.contains(key) && seen.insert(key.clone()) {
            out.push_str(&line.replacen(
                &format!("string \"{key}\""),
                &format!("string \"{PREFIX}{key}\""),
                1,
            ));
            out.push('\n');
        }
    }
    (!seen.is_empty()).then_some(out)
}

/// Copies `name` from the package's `ui` directory into `project`.
pub(super) fn eject_from(
    ui: &Path,
    name: &str,
    project: &Path,
    force: bool,
) -> Result<Ejected, String> {
    let ui = &normalize(ui);
    let lib = library(ui);
    let Some(entry) = lib.iter().find(|e| e.name == name) else {
        let near = closest(name, &lib);
        return Err(if near.is_empty() {
            format!("no built-in '{name}' (og ui eject --list names them)")
        } else {
            format!(
                "no built-in '{name}'; the closest: {} (og ui eject --list names them all)",
                near.join(", ")
            )
        });
    };
    if !project.join("og-project.json").is_file() {
        return Err(format!(
            "{}: no og-project.json; run it in a game's directory or pass the project",
            project.display()
        ));
    }
    let source = ui.join(format!("{}.tsx", entry.rel));
    let text =
        std::fs::read_to_string(&source).map_err(|e| format!("{}: {e}", source.display()))?;
    let exports = exported_symbols(&text, &source)?;
    let aliases = barrel_aliases(ui, &entry.rel);
    let source_dir = source.parent().expect("a file's directory");
    let copy = project.join("assets/ui").join(format!("{name}.tsx"));
    let table_path = project.join("assets/ui").join(format!("{name}.en.ogt"));
    let stem = name.rsplit('/').next().unwrap_or(name);

    // The tables it imports, their sibling locales (`<base>.<locale>.ogt`),
    // their keys, and the code with its keys renamed.
    let mut code = String::new();
    let mut tables = Vec::new();
    let mut sibling_locales: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for line in text.split_inclusive('\n') {
        let t = line.trim();
        if let Some(rel) = t
            .strip_prefix("import \"")
            .and_then(|r| r.strip_suffix("\";"))
            .filter(|r| r.ends_with(".ogt"))
        {
            let path = source_dir.join(rel);
            tables.push(
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?,
            );
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                && let Some(base) = file_name.strip_suffix(".en.ogt")
                && let Some(parent) = path.parent()
                && let Ok(entries) = std::fs::read_dir(parent)
            {
                let prefix = format!("{base}.");
                let mut peers: Vec<_> = entries.flatten().map(|e| e.path()).collect();
                peers.sort();
                for peer in peers {
                    let Some(peer_name) = peer.file_name().and_then(|n| n.to_str()) else {
                        continue;
                    };
                    let Some(loc) = peer_name
                        .strip_prefix(&prefix)
                        .and_then(|s| s.strip_suffix(".ogt"))
                    else {
                        continue;
                    };
                    if loc.is_empty() || loc == "en" {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(&peer) {
                        sibling_locales
                            .entry(loc.to_string())
                            .or_default()
                            .push(content);
                    }
                }
            }
            if tables.len() == 1 {
                code.push_str(&format!("import \"./{stem}.en.ogt\";\n"));
            }
            continue;
        }
        code.push_str(line);
    }
    let lines: Vec<(String, String)> = tables.iter().flat_map(|t| parse_table_lines(t)).collect();
    let mut used = BTreeSet::new();
    for (key, _) in &lines {
        for q in ['"', '\'', '`'] {
            let from = format!("{q}{key}{q}");
            if code.contains(&from) {
                code = code.replace(&from, &format!("{q}{PREFIX}{key}{q}"));
                used.insert(key.clone());
            }
        }
    }
    // A key built at run time: `t(`kit.part.${id}`)` takes every key under
    // its literal prefix.
    let mut prefixes = BTreeSet::new();
    for piece in code.split('`').skip(1).step_by(2) {
        if let Some(head) = piece.split("${").next()
            && piece.contains("${")
            && !head.is_empty()
            && lines.iter().any(|(k, _)| k.starts_with(head))
        {
            prefixes.insert(head.to_string());
        }
    }
    for p in &prefixes {
        code = code.replace(&format!("`{p}${{"), &format!("`{PREFIX}{p}${{"));
        for (k, _) in &lines {
            if k.starts_with(p.as_str()) {
                used.insert(k.clone());
            }
        }
    }
    let mut code = relative_imports(&code, source_dir, ui);
    let table = (!used.is_empty())
        .then(|| build_locale_table("en", &lines, &used))
        .flatten();
    if table.is_none() && !tables.is_empty() {
        code = code.replace(&format!("import \"./{stem}.en.ogt\";\n"), "");
    }
    let mut extra_writes = Vec::new();
    let mut extra_imports = String::new();
    if table.is_some() {
        for (locale, texts) in &sibling_locales {
            let loc_lines: Vec<(String, String)> =
                texts.iter().flat_map(|t| parse_table_lines(t)).collect();
            if let Some(built) = build_locale_table(locale, &loc_lines, &used) {
                let path = project
                    .join("assets/ui")
                    .join(format!("{name}.{locale}.ogt"));
                extra_writes.push((path, built));
                extra_imports.push_str(&format!("import \"./{stem}.{locale}.ogt\";\n"));
            }
        }
        if !extra_imports.is_empty() {
            let en_import = format!("import \"./{stem}.en.ogt\";\n");
            code = code.replacen(&en_import, &format!("{en_import}{extra_imports}"), 1);
        }
    }

    let mut writes = vec![(copy.clone(), code)];
    if let Some(t) = &table {
        writes.push((table_path.clone(), t.clone()));
    }
    let extra_tables: Vec<PathBuf> = extra_writes.iter().map(|(p, _)| p.clone()).collect();
    writes.extend(extra_writes);
    for (path, _) in &writes {
        if path.exists() && !force {
            return Err(format!(
                "{} already exists; nothing written (--force overwrites it)",
                path.display()
            ));
        }
    }
    // The game's imports of it, rewired before anything is written.
    let package_path = format!("{PACKAGE_UI}{}", entry.rel);
    let mut sources = Vec::new();
    game_sources(project, &mut sources);
    let mut rewires = Vec::new();
    for file in sources {
        let Ok(before) = std::fs::read_to_string(&file) else {
            continue;
        };
        let target = relative(file.parent().expect("a file's directory"), &copy);
        let after = rewire(&before, &exports, &aliases, &package_path, &target);
        if after != before {
            rewires.push((file, after));
        }
    }
    for (path, text) in writes.iter().chain(&rewires) {
        std::fs::create_dir_all(path.parent().expect("a file's directory"))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(Ejected {
        copy,
        table: table.map(|_| table_path),
        extra_tables,
        rewired: rewires.into_iter().map(|(p, _)| p).collect(),
    })
}

/// The copy's relative imports as the package's paths.
fn relative_imports(code: &str, dir: &Path, ui: &Path) -> String {
    let mut out = String::with_capacity(code.len());
    let mut rest = code;
    loop {
        let found = [("from \".", '"'), ("from '.", '\'')]
            .iter()
            .filter_map(|(pat, q)| rest.find(pat).map(|at| (at, *q)))
            .min_by_key(|(at, _)| *at);
        let Some((at, q)) = found else {
            break;
        };
        let (head, tail) = rest.split_at(at + "from \"".len());
        out.push_str(head);
        let end = tail.find(q).unwrap_or(tail.len());
        let spec = &tail[..end];
        let resolved = normalize(&dir.join(spec));
        match resolved.strip_prefix(ui) {
            Ok(r) => out.push_str(&format!("{PACKAGE_UI}{}", slash(r))),
            Err(_) => out.push_str(spec),
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

/// `to` (a `.tsx` file) from the directory `from`, as an import path.
fn relative(from: &Path, to: &Path) -> String {
    let (from, to) = (normalize(from), normalize(to));
    let a: Vec<_> = from.components().collect();
    let b: Vec<_> = to.components().collect();
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut parts: Vec<String> = vec!["..".into(); a.len() - common];
    parts.extend(
        b[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    let joined = parts.join("/");
    let joined = joined.trim_end_matches(".tsx").to_string();
    if joined.starts_with("..") {
        joined
    } else {
        format!("./{joined}")
    }
}

/// The game's own `.ts`/`.tsx` files, outside installs and outputs.
fn game_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if e.file_type().is_ok_and(|t| t.is_dir()) {
            if !matches!(
                name.as_ref(),
                "node_modules" | "cooked" | "og-out" | "dist" | ".og" | "target" | ".git"
            ) {
                game_sources(&p, out);
            }
        } else if name.ends_with(".ts") || name.ends_with(".tsx") {
            out.push(p);
        }
    }
}

/// `text` with its imports of any symbol in `exports` or `aliases` (from the
/// barrel) and of `package_path` (with or without `.tsx`) pointing at `target`.
fn rewire(
    text: &str,
    exports: &BTreeSet<String>,
    aliases: &std::collections::BTreeMap<String, String>,
    package_path: &str,
    target: &str,
) -> String {
    let mut text = text.to_string();
    for q in ['"', '\''] {
        text = text
            .replace(
                &format!("{q}{package_path}.tsx{q}"),
                &format!("{q}{target}{q}"),
            )
            .replace(&format!("{q}{package_path}{q}"), &format!("{q}{target}{q}"));
    }
    for q in ['"', '\''] {
        text = rewire_barrel(&text, exports, aliases, target, q);
    }
    text
}

/// One quote style's imports from the barrel, split so the symbols in
/// `exports` or `aliases` come from `target` (written in the same quote).
fn rewire_barrel(
    text: &str,
    exports: &BTreeSet<String>,
    aliases: &std::collections::BTreeMap<String, String>,
    target: &str,
    q: char,
) -> String {
    let mut text = text.to_string();
    let from = format!("from {q}{BARREL}{q}");
    let mut search = 0;
    while let Some(at) = text[search..].find(&from).map(|i| i + search) {
        let Some(start) = text[..at].rfind("import") else {
            break;
        };
        let end = at + from.len() + usize::from(text[at + from.len()..].starts_with(';'));
        let clause = &text[start..at];
        let (Some(open), Some(close)) = (clause.find('{'), clause.rfind('}')) else {
            search = end;
            continue;
        };
        let names: Vec<&str> = clause[open + 1..close]
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let mut moved = Vec::new();
        let mut kept = Vec::new();
        for spec in names {
            let (is_type, rest) = match spec.strip_prefix("type ") {
                Some(r) => (true, r.trim()),
                None => (false, spec),
            };
            let (imported, local) = match rest.split_once(" as ") {
                Some((i, l)) => (i.trim(), Some(l.trim())),
                None => (rest.trim(), None),
            };
            if exports.contains(imported) {
                moved.push(spec.to_string());
            } else if let Some(orig) = aliases.get(imported) {
                let prefix = if is_type { "type " } else { "" };
                let bound = local.unwrap_or(imported);
                if bound == orig {
                    moved.push(format!("{prefix}{orig}"));
                } else {
                    moved.push(format!("{prefix}{orig} as {bound}"));
                }
            } else {
                kept.push(spec);
            }
        }
        if moved.is_empty() {
            search = end;
            continue;
        }
        let own = format!(
            "{}{{ {} }} from {q}{target}{q};",
            &clause[..open],
            moved.join(", ")
        );
        let replacement = if kept.is_empty() {
            own
        } else {
            format!(
                "{}{{ {} }} {from};\n{own}",
                &clause[..open],
                kept.join(", ")
            )
        };
        text.replace_range(start..end, &replacement);
        search = start + replacement.len();
    }
    text
}

/// `og ui eject <name> [project] [--force]`, `og ui eject --list [project]`.
pub(super) fn run(args: &[&str], usage: &str) -> ExitCode {
    let (mut words, mut list, mut force) = (Vec::new(), false, false);
    for a in args {
        match *a {
            "--list" => list = true,
            "--force" => force = true,
            a if a.starts_with('-') => {
                eprintln!("og ui eject: unknown option `{a}`. {usage}");
                return ExitCode::from(2);
            }
            a => words.push(a),
        }
    }
    let (name, project) = match (list, words.as_slice()) {
        (true, []) => (None, "."),
        (true, [p]) => (None, *p),
        (false, [n]) => (Some(*n), "."),
        (false, [n, p]) => (Some(*n), *p),
        _ => {
            eprintln!("og ui eject: {usage}");
            return ExitCode::from(2);
        }
    };
    let project = Path::new(project);
    let Some(package) = project::engine_package(Some(project)) else {
        eprintln!("og ui eject: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    let ui = package.join("src/ui");
    let Some(name) = name else {
        for e in library(&ui) {
            println!("{:<28} {}", e.name, e.summary);
        }
        return ExitCode::SUCCESS;
    };
    match eject_from(&ui, name, project, force) {
        Ok(done) => {
            let shown = |p: &Path| slash(p.strip_prefix(project).unwrap_or(p));
            println!("og ui eject: '{name}' into {}", shown(&done.copy));
            if let Some(t) = &done.table {
                println!("  its strings into {}, keys under `{PREFIX}`", shown(t));
            }
            for t in &done.extra_tables {
                println!("  its strings into {}, keys under `{PREFIX}`", shown(t));
            }
            for f in &done.rewired {
                println!("  {} now imports the copy", shown(f));
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("og ui eject: {e}");
            ExitCode::FAILURE
        }
    }
}
