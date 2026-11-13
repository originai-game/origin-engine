//! `og ui check` and `og ui preview` (docs/ui.md §6, §6): a project's
//! screens laid out over styles × breakpoints × locales × text scale on
//! the native host (packages/engine/src/node/ui-run.ts), each cell driven
//! by a command script. `check` reports every rule of ui-check.ts with the
//! element paths, what was found and the prop that sets it, and every
//! measure (contrast, text size, touch target, text scale, primary actions,
//! element count) with no bar, into DIR/check.json and check.txt (default
//! `<project>/og-out/ui-check`); it exits 0 when it ran, whatever it found,
//! and 1 when it could not run; `preview` writes a contact sheet per screen
//! and style and index.json (default `<project>/og-out/ui-preview`), or with
//! `--film` a filmstrip of a motion at chosen times.
//!
//! `og ui slots [project] [--json]` (docs/ui.md §3.3) lists every art
//! slot of the game, read from its documents without running it: each
//! `art` element's path, kind, aspect, focal point, safe rectangle and fit,
//! whether its art is set or it still draws the style's placeholder, and
//! what its spec gets wrong with the fix (exit 3 on one). `--json` prints
//! them as one array, what the art pipeline reads.
//!
//! `og ui migrate <file|dir>... [--check]` (docs/ui.md §3.2, §3)
//! rewrites every version 2 style as version 3 (selector blocks, custom properties), line
//! for line with its comments, after checking the two read the same;
//! `--check` writes nothing and exits 1 when one still needs it.
//!
//! `og ui eject <screen> [project] [--force]` (docs/ui.md §2) copies one
//! of the library's screen or component documents (the engine package's
//! `src/ui/screens/<screen>.svelte`, one or more for every screen type of
//! docs/ui.md §2, and `src/ui/components/<component>.svelte`) into the game's `assets/ui/<screen>.svelte`, where
//! `og.load.document(world, "ui/<screen>.svelte")` loads it and the game edits
//! it as its own file. It refuses to overwrite one already there without
//! `--force`; an unknown name exits 1 with the closest names. `og ui eject
//! --list [project]` prints every screen with its Appendix B row, read from
//! the table of that directory's README.md. The package is the one og run uses
//! (`OG_ENGINE_PACKAGE`, the project's installed @origingame/engine, or a
//! checkout's packages/engine).
//!
//! Without `--screen`, every screen the project declares: project.ogt's
//! `ui { screen "<name>" "<commands>" ... }` entries and its `screens { title
//! "<commands>" play ... results ... }` block (og style check's); without
//! those, ui-run.ts takes the values the game's show command enumerates in
//! its usage (`<a|b|c>`, the kit's `show:`), else the screens the
//! playtests take screenshots of, else the start screen.

use crate::project::{self, Project};
use og_ui::prelude::*;
use std::path::Path;
use std::process::{Command, ExitCode};

pub mod extensions;
mod literals;
mod review;
mod sheet;

const USAGE: &str = "usage: og ui add <materials|directory> [project]\n       og ui list [project] [--json]\n       og ui slots [project] [--json]\n       og ui types [project]\n       og ui literals [path...] [--json]\n       og ui eject <screen|component> [project] [--force]\n       og ui eject --list [project]\n       og ui migrate <file|dir>... [--check]\n       og ui review <group|sheet-directory> [project] [--style <pack>]... [--states a,b] [--grid N] [--out DIR] [--json]\n       og ui sheet <pack|group|sheet-directory> [project] [--out DIR] [--json]\n       og ui check|preview [project] [--screen <name>[=<commands>]]... [--show <template>] [--styles a,b] [--style-cmd <template>] [--breakpoints desktop,phonePortrait,phoneLandscape,tablet] [--locales game,pseudo,<cjk>,<rtl>] [--scales 100,150] [--high-contrast] [--settle S] [--film <commands> --at t,...] [--out DIR] [--json]";

/// Options forwarded to ui-run.ts with their value.
const VALUED: [&str; 10] = [
    "--screen",
    "--show",
    "--styles",
    "--style-cmd",
    "--breakpoints",
    "--locales",
    "--scales",
    "--settle",
    "--film",
    "--at",
];

pub fn run(args: &[&str]) -> ExitCode {
    if let ["sheet", rest @ ..] = args {
        return sheet::run(rest);
    }
    if let ["review", rest @ ..] = args {
        return review::run(rest);
    }
    if let ["add", rest @ ..] = args {
        return extensions::add(rest, "materials");
    }
    if let ["list", rest @ ..] = args {
        return extensions::list(rest, None);
    }
    if let ["literals", rest @ ..] = args {
        return literals::run(rest);
    }
    if let ["slots", rest @ ..] = args {
        return slots(rest);
    }
    if let ["types", rest @ ..] = args {
        let project = match rest {
            [] => ".",
            [p] if !p.starts_with('-') => p,
            _ => {
                eprintln!("og ui types: {USAGE}");
                return ExitCode::from(2);
            }
        };
        let p = match Project::open(Path::new(project)) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("og ui types: {e}");
                return ExitCode::FAILURE;
            }
        };
        return match write_types(&p.root) {
            Ok(typed) => {
                for u in &typed.untyped {
                    eprintln!("og ui types: {}", u.line());
                }
                println!(
                    "og ui types: {} document(s) typed in {TYPES_DIR}/",
                    typed.documents
                );
                if typed.untyped.is_empty() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(e) => {
                eprintln!("og ui types: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if let ["eject", rest @ ..] = args {
        return eject(rest);
    }
    if let ["migrate", rest @ ..] = args {
        return migrate(rest);
    }
    let (mode, rest) = match args {
        [m @ ("check" | "preview"), rest @ ..] => (*m, rest),
        _ => {
            eprintln!("og ui: {USAGE}");
            return ExitCode::from(2);
        }
    };
    let mut project = ".";
    let mut out: Option<&str> = None;
    let mut forward: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "--json" | "--high-contrast" | "--require-checked" => forward.push(rest[i]),
            "--out" if i + 1 < rest.len() => {
                out = Some(rest[i + 1]);
                i += 1;
            }
            a if VALUED.contains(&a) && i + 1 < rest.len() => {
                if a == "--film" && mode != "preview" {
                    eprintln!("og ui check: --film is preview's. {USAGE}");
                    return ExitCode::from(2);
                }
                forward.extend([a, rest[i + 1]]);
                i += 1;
            }
            a if a.starts_with('-') => {
                eprintln!("og ui {mode}: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => project = a,
        }
        i += 1;
    }
    let p = match Project::open(Path::new(project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og ui {mode}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(package) = project::engine_package(Some(&p.root)) else {
        eprintln!("og ui {mode}: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    if let Some(n) = project::pin_notice(&p, &package) {
        eprintln!("og ui {mode}: {n}");
    }
    if p.has_assets()
        && let Err(e) = crate::cook::cook(&p.root, 0, false)
    {
        eprintln!("og ui {mode}: {e}");
        return ExitCode::FAILURE;
    }
    let bundle = match project::bundle_native(&p, &package) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("og ui {mode}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let dir = if mode == "check" {
        "ui-check"
    } else {
        "ui-preview"
    };
    let out = out.map_or_else(
        || p.root.join("og-out").join(dir),
        |o| {
            let o = Path::new(o);
            if o.is_absolute() {
                o.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(o)
            }
        },
    );
    match spawn(mode, &p, &package, &bundle, &out, &forward) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("og ui {mode}: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Runs ui-run.ts's `check` or `preview` into `out` on the cooked project's
/// native bundle; without a `--screen` in `forward`, over every screen the
/// project declares.
pub fn spawn(
    mode: &str,
    p: &Project,
    package: &Path,
    bundle: &Path,
    out: &Path,
    forward: &[&str],
) -> Result<ExitCode, String> {
    let given = forward.contains(&"--screen") || forward.contains(&"--baseline");
    let declared = if given {
        Vec::new()
    } else {
        declared_screens(&p.root)?
    };
    let mut playtests = Vec::new();
    if !given && declared.is_empty() {
        crate::test::files(&p.root.join("tests"), ".og.ts", &mut playtests);
    }
    let mut cmd: Command = project::bun(package, "src/node/ui-run.ts");
    cmd.arg(mode)
        .arg("--entry")
        .arg(bundle)
        .arg("--root")
        .arg(&p.root)
        .arg("--out")
        .arg(out)
        .args(forward);
    for (name, commands) in &declared {
        cmd.arg("--screen").arg(format!("{name}={commands}"));
    }
    for t in &playtests {
        cmd.arg("--playtest").arg(t);
    }
    if let Some(code) = crate::session::try_run(&p.root, package, &cmd)? {
        return Ok(code);
    }
    cmd.status()
        .map(project::exit_of)
        .map_err(|e| format!("{} ({e})", project::NO_BUN))
}

/// The screens project.ogt declares, in order: `ui { screen "<name>"
/// "<commands>" }` entries, then `screens { title|play|results "<commands>" }`.
fn declared_screens(root: &Path) -> Result<Vec<(String, String)>, String> {
    let path = root.join("project.ogt");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(Vec::new());
    };
    let body = og_core::ogt::parse(&text)
        .map_err(|e| format!("{}: {}", path.display(), e.reason()))?
        .body;
    let mut out: Vec<(String, String)> = og_cook::cook::ui_screens::screens(&body)
        .map_err(|e| format!("project.ogt:{e}"))?
        .into_iter()
        .map(|s| (s.name, s.commands))
        .collect();
    if let Some(screens) = body.get("screens").and_then(|e| e.block.as_ref()) {
        for e in &screens.entries {
            if let Ok(commands) = e.string(0)
                && !out.iter().any(|(n, _)| n == &e.key)
            {
                out.push((e.key.clone(), commands.to_string()));
            }
        }
    }
    Ok(out)
}

/// `og ui slots [project] [--json]`.
fn slots(args: &[&str]) -> ExitCode {
    let mut project = ".";
    let mut json = false;
    for a in args {
        match *a {
            "--json" => json = true,
            a if a.starts_with('-') => {
                eprintln!("og ui slots: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => project = a,
        }
    }
    let p = match Project::open(Path::new(project)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("og ui slots: {e}");
            return ExitCode::FAILURE;
        }
    };
    let assets = p.root.join("assets");
    let mut files = Vec::new();
    collect_files(&assets, "svelte", &mut files);
    files.sort();
    let mut rows = Vec::new();
    let mut wrong = 0;
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        let id = f
            .strip_prefix(&assets)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        let doc = match og_ui::UiDocument::parse(&id, &text) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("og ui slots: {}", e.join("\n"));
                continue;
            }
        };
        for s in og_ui::slot::document_slots(&doc) {
            let problems = s.spec.validate();
            wrong += usize::from(!problems.is_empty());
            rows.push((id.clone(), s, problems));
        }
    }
    if json {
        // A set is every slot drawn with one look; its reference is the
        // first member whose art is set, the image a generator matches.
        let set = |name: &str| {
            let members: Vec<&str> = rows
                .iter()
                .filter(|(_, s, _)| s.spec.set == name)
                .map(|(_, s, _)| s.path.as_str())
                .collect();
            let reference = rows
                .iter()
                .find(|(_, s, _)| s.spec.set == name && s.textured)
                .map(|(_, s, _)| s.path.as_str());
            serde_json::json!({ "name": name, "members": members, "reference": reference })
        };
        let out: Vec<serde_json::Value> = rows
            .iter()
            .map(|(id, s, problems)| {
                let mut v = og_ui::doc::build::slot_json(&s.spec);
                if let Some(o) = v.as_object_mut() {
                    o.insert("set".into(), set(&s.spec.set));
                    o.insert("path".into(), serde_json::json!(s.path));
                    o.insert(
                        "source".into(),
                        serde_json::json!(format!("{id}:{}", s.line)),
                    );
                    o.insert(
                        "art".into(),
                        serde_json::json!(if s.textured { "set" } else { "placeholder" }),
                    );
                    o.insert("problems".into(), serde_json::json!(problems));
                }
                v
            })
            .collect();
        println!("{}", serde_json::Value::Array(out));
    } else if rows.is_empty() {
        println!("og ui slots: no art slots in {}", assets.display());
    } else {
        for (id, s, problems) in &rows {
            let aspect = match &s.spec.aspect {
                og_ui::slot::SlotAspect::One(a) => a.to_string(),
                og_ui::slot::SlotAspect::AnyOf(v) => v
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join("|"),
                og_ui::slot::SlotAspect::Range(lo, hi) => format!("{lo}..{hi}"),
                og_ui::slot::SlotAspect::Free => "free".into(),
            };
            let safe = s.spec.safe.map_or("-".to_string(), |r| {
                format!("{} {} {} {}", r[0], r[1], r[2], r[3])
            });
            println!(
                "{:<32} {:<16} {:<6} focal {} {}  safe {:<15} {:<9} {:<11} {id}:{}",
                s.path,
                s.spec.kind.name(),
                aspect,
                s.spec.focal[0],
                s.spec.focal[1],
                safe,
                format!("{:?}", s.spec.fit).to_lowercase(),
                if s.textured { "art" } else { "placeholder" },
                s.line
            );
            for e in problems {
                println!("    {e}");
            }
        }
    }
    if wrong > 0 {
        ExitCode::from(3)
    } else {
        ExitCode::SUCCESS
    }
}

/// Where the library's screen and component documents sit in the engine
/// package, in the order eject searches and lists them.
const LIBRARY_DIRS: [&str; 2] = ["src/ui/screens", "src/ui/components"];

/// One library document: its name (the file's stem), its README row and
/// the directory it sits in.
#[derive(Debug, PartialEq)]
struct LibScreen {
    name: String,
    row: String,
    dir: std::path::PathBuf,
}

/// The library's documents in the order of each directory's README table
/// (`| <purpose row> | `a`, `b` |`).
fn lib_screens(dirs: &[std::path::PathBuf]) -> Result<Vec<LibScreen>, String> {
    let mut out = Vec::new();
    for dir in dirs {
        lib_table(dir, &mut out)?;
    }
    Ok(out)
}

/// One directory's README table, appended to `out`.
fn lib_table(dir: &Path, out: &mut Vec<LibScreen>) -> Result<(), String> {
    let readme = dir.join("README.md");
    let text = std::fs::read_to_string(&readme)
        .map_err(|e| format!("{}: {e} (reinstall @origingame/engine)", readme.display()))?;
    for line in text
        .lines()
        .skip_while(|l| !l.starts_with("|---"))
        .skip(1)
        .take_while(|l| l.starts_with('|'))
    {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let (Some(row), Some(docs)) = (cells.get(1), cells.get(2)) else {
            continue;
        };
        for d in docs.split(',') {
            let name = d.trim().trim_matches('`').to_string();
            out.push(LibScreen {
                name,
                row: (*row).to_string(),
                dir: dir.to_path_buf(),
            });
        }
    }
    Ok(())
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

/// The screens nearest `name`, at most three: those whose name contains it
/// or it contains, then those a few edits from their name or from a word of
/// their Appendix B row (`shop` finds `vendor`, the shop in the world).
fn closest<'a>(name: &str, screens: &'a [LibScreen]) -> Vec<&'a str> {
    // How far `name` is from `b`, when it is near enough to suggest.
    let near = |b: &str| {
        let d = if b.contains(name) || name.contains(b) {
            0
        } else {
            distance(name, b)
        };
        (d <= (b.len().max(name.len()) / 3).max(1)).then_some(d)
    };
    let mut scored: Vec<(usize, &str)> = screens
        .iter()
        .filter_map(|s| {
            let row = s
                .row
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2)
                .filter_map(|w| near(w).map(|d| d + 1))
                .min();
            let d = match (near(&s.name), row) {
                (Some(a), Some(b)) => a.min(b),
                (a, b) => a.or(b)?,
            };
            Some((d, s.name.as_str()))
        })
        .collect();
    scored.sort();
    scored.into_iter().take(3).map(|(_, n)| n).collect()
}

/// Copies the library's `name` from `dirs` into `project`'s
/// `assets/ui/<name>.svelte`, with its declared strings; answers the path written.
fn eject_from(
    dirs: &[std::path::PathBuf],
    name: &str,
    project: &Path,
    force: bool,
) -> Result<std::path::PathBuf, String> {
    let screens = lib_screens(dirs)?;
    let Some(dir) = screens
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.dir.as_path())
    else {
        let near = closest(name, &screens);
        return Err(if near.is_empty() {
            format!("no library screen '{name}' (og ui eject --list names them)")
        } else {
            format!(
                "no library screen '{name}'; the closest: {} (og ui eject --list names them all)",
                near.join(", ")
            )
        });
    };
    if !project.join("project.ogt").is_file() {
        return Err(format!(
            "{}: no project.ogt; run it in a game's directory or pass the project",
            project.display()
        ));
    }
    let id = format!("{name}.svelte");
    let source = dir.join(&id);
    let text =
        std::fs::read_to_string(&source).map_err(|e| format!("{}: {e}", source.display()))?;
    let document = og_ui::UiDocument::parse(&id, &text).map_err(|e| e.join("\n"))?;
    let destination = project.join("assets/ui");
    let target = destination.join(&id);
    let mut files = vec![(target.clone(), text.into_bytes())];
    for strings in &document.strings {
        let source = dir.join(strings);
        files.push((
            destination.join(strings),
            std::fs::read(&source).map_err(|e| format!("{}: {e}", source.display()))?,
        ));
    }
    for (path, _) in &files {
        if path.exists() && !force {
            return Err(format!(
                "{} already exists; nothing written (--force overwrites it)",
                path.display()
            ));
        }
    }
    for (path, bytes) in files {
        std::fs::create_dir_all(path.parent().expect("assets/ui"))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(target)
}

/// `og ui eject <screen> [project] [--force]`, `og ui eject --list [project]`.
fn eject(args: &[&str]) -> ExitCode {
    let (mut words, mut list, mut force) = (Vec::new(), false, false);
    for a in args {
        match *a {
            "--list" => list = true,
            "--force" => force = true,
            a if a.starts_with('-') => {
                eprintln!("og ui eject: unknown option `{a}`. {USAGE}");
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
            eprintln!("og ui eject: {USAGE}");
            return ExitCode::from(2);
        }
    };
    let project = Path::new(project);
    let Some(package) = project::engine_package(Some(project)) else {
        eprintln!("og ui eject: {}", project::NO_ENGINE);
        return ExitCode::FAILURE;
    };
    let dirs = LIBRARY_DIRS.map(|d| package.join(d));
    let Some(name) = name else {
        return match lib_screens(&dirs) {
            Ok(screens) => {
                for s in &screens {
                    println!("{:<20} {}", s.name, s.row);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("og ui eject: {e}");
                ExitCode::FAILURE
            }
        };
    };
    match eject_from(&dirs, name, project, force) {
        Ok(_) if dirs[1].join(format!("{name}.svelte")).is_file() => {
            println!(
                "og ui eject: the library's '{name}' component into assets/ui/{name}.svelte\n\
                 it replaces the built-in one of the name its <meta name=\"component\"> declares, and loads with every screen that uses it;\n\
                 its header names its props, actions and fixture"
            );
            ExitCode::SUCCESS
        }
        Ok(_) => {
            println!(
                "og ui eject: the library's '{name}' into assets/ui/{name}.svelte\n\
                 it replaces the built-in screen of its name: og.load.document(world, \"ui/{name}.svelte\") (or the built-in id, \"@ui/screens/{name}.svelte\") loads it, then mount its named stores;\n\
                 its header names the viewmodel's props, its events and what fills each slot.\n\
                 preview it with og run --commands \"0.5:ui:push:{name}:default\", and add screen \"{name}\" \"<commands>\" to project.ogt's ui block so og ui check covers it"
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("og ui eject: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `og ui migrate <file|dir>... [--check]`.
fn migrate(args: &[&str]) -> ExitCode {
    let check = args.contains(&"--check");
    let paths: Vec<&str> = args.iter().copied().filter(|a| *a != "--check").collect();
    if paths.is_empty() || paths.iter().any(|a| a.starts_with('-')) {
        eprintln!("og ui migrate: {USAGE}");
        return ExitCode::from(2);
    }
    let mut files = Vec::new();
    for p in &paths {
        let p = Path::new(p);
        if p.is_dir() {
            collect_files(p, "ogt", &mut files);
        } else {
            files.push(p.to_path_buf());
        }
    }
    files.sort();
    let (mut pending, mut failed) = (0, false);
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            eprintln!("og ui migrate: {}: cannot read", f.display());
            failed = true;
            continue;
        };
        let Ok(head) = og_core::ogt::Document::parse_header(&text) else {
            continue;
        };
        if head.ty != "style" || head.version != 2 {
            continue;
        }
        let source = f.display().to_string();
        let migrated: Result<String, Vec<String>> = Err(vec![format!(
            "{source}: style.ogt has been removed; use a directory of CSS files and assets. CSS migration tooling is pending phase C."
        )]);
        match migrated {
            Ok(_) if check => {
                println!("og ui migrate: {source} needs migrating");
                pending += 1;
            }
            Ok(new) => match std::fs::write(f, new) {
                Ok(()) => println!("og ui migrate: {source}"),
                Err(e) => {
                    eprintln!("og ui migrate: {source}: {e}");
                    failed = true;
                }
            },
            Err(e) => {
                for e in e {
                    eprintln!("og ui migrate: {e}");
                }
                failed = true;
            }
        }
    }
    if failed || pending > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Where `og ui types` writes, under the project: one `<name>.d.ts` per
/// document, a `/` in its name a `.`.
pub const TYPES_DIR: &str = ".og/types/ui";

/// What [`write_types`] did: how many documents it typed, and those it
/// could not type because they do not parse.
pub struct Typed {
    pub documents: usize,
    pub untyped: Vec<Untyped>,
}

/// A document that does not parse: the cook refuses it with the same error.
pub struct Untyped {
    /// The document, relative to the project (`assets/ui/shop.svelte`).
    pub file: String,
    /// Where its first error is, 1-based; none for one without a place.
    pub at: Option<(u32, u32)>,
    pub message: String,
}

impl Untyped {
    /// `assets/ui/shop.svelte(3,1): error OG9102: <message>`, as a checker prints a diagnostic.
    pub fn line(&self) -> String {
        let (line, col) = self.at.unwrap_or((1, 1));
        format!(
            "{}({line},{col}): error OG{UNTYPED_CODE}: {}",
            self.file, self.message
        )
    }
}

/// The code of a document that does not parse in `og check`'s diagnostics.
pub const UNTYPED_CODE: u32 = 9102;

/// The stylesheets a document's `@import`s name, read from `assets/` as
/// the cook reads them. One that cannot be read is left out: the parse
/// then names the `@import` that lacks it.
fn document_imports(assets: &Path, id: &str, text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut pending: Vec<(String, String)> = og_ui::UiDocument::imports(text)
        .into_iter()
        .map(|path| (id.to_string(), path))
        .collect();
    while let Some((from, path)) = pending.pop() {
        let rel = og_ui::doc::at_rules::resolve(&from, &path);
        if out.iter().any(|(p, _)| *p == rel) || rel.split('/').any(|c| c == ".." || c.is_empty()) {
            continue;
        }
        let Ok(css) = std::fs::read_to_string(assets.join(&rel)) else {
            continue;
        };
        pending.extend(
            og_ui::UiDocument::imports(&css)
                .into_iter()
                .map(|path| (rel.clone(), path)),
        );
        out.push((rel, css));
    }
    out
}

/// Write the `.d.ts` of every screen and component document under the
/// project's `assets/` (og_ui::doc::types), leaving a file as it is when
/// it reads the same and removing those whose document is gone. A
/// document that does not parse is not typed and is answered in
/// [`Typed::untyped`] with its first error, which is the cook's as well:
/// `og check` and `og ui types` fail on it.
pub fn write_types(root: &Path) -> Result<Typed, String> {
    let assets = root.join("assets");
    let mut files = Vec::new();
    collect_files(&assets, "svelte", &mut files);
    files.sort();
    let mut docs = Vec::new();
    let mut untyped = Vec::new();
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        let id = f
            .strip_prefix(&assets)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        let imports = document_imports(&assets, &id, &text);
        match og_ui::UiDocument::parse_with_imports(&id, &text, imports) {
            Ok(d) => docs.push(d),
            Err(e) => {
                let first = e.first().map(String::as_str).unwrap_or("it does not parse");
                // An error reads `<id>:<line>:<col>: <message>`.
                let placed = first.strip_prefix(&format!("{id}:")).and_then(|rest| {
                    let (line, rest) = rest.split_once(':')?;
                    let (col, message) = rest.split_once(": ")?;
                    Some(((line.parse().ok()?, col.parse().ok()?), message))
                });
                untyped.push(Untyped {
                    file: format!("assets/{id}"),
                    at: placed.map(|(at, _)| at),
                    message: placed.map_or(first, |(_, message)| message).to_string(),
                });
            }
        }
    }
    // The built-in screens the game's code names by id and does not
    // replace are typed as its own, as the cook bundles them.
    let mut builtin: Vec<og_ui::UiDocument> = Vec::new();
    if let Ok(index) = og_cook::cook::ui_components::index() {
        for path in og_cook::cook::assets::game_typescript(root).unwrap_or_default() {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(refs) = og_cook::cook::ts_refs::parse(&path, &text) else {
                continue;
            };
            for l in refs.literals {
                if let Some(c) = index.values().find(|c| c.id == l.value)
                    && c.doc.kind == og_ui::doc::DocKind::Screen
                    && !docs.iter().chain(&builtin).any(|d| d.name == c.doc.name)
                {
                    builtin.push(c.doc.clone());
                }
            }
        }
    }
    docs.extend(builtin);
    let dir = root.join(TYPES_DIR);
    if docs.is_empty() && !dir.is_dir() {
        return Ok(Typed {
            documents: 0,
            untyped,
        });
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let components: std::collections::BTreeMap<&str, &og_ui::UiDocument> = docs
        .iter()
        .filter(|d| d.kind == og_ui::doc::DocKind::Component)
        .map(|d| (d.name.as_str(), d))
        .collect();
    // A name the game does not declare is the engine's built-in component,
    // as the cook resolves it.
    let lookup = |n: &str| {
        components.get(n).copied().or_else(|| {
            let index = og_cook::cook::ui_components::index().ok()?;
            index.get(n).map(|c| &c.doc)
        })
    };
    let mut keep = std::collections::BTreeSet::new();
    for d in &docs {
        let file = format!("{}.d.ts", d.name.replace('/', "."));
        let text = og_ui::doc::types::document_dts(d, &lookup);
        let path = dir.join(&file);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        keep.insert(file);
    }
    for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.ends_with(".d.ts") && !keep.contains(&name) {
            let _ = std::fs::remove_file(e.path());
        }
    }
    Ok(Typed {
        documents: docs.len(),
        untyped,
    })
}

fn collect_files(dir: &Path, extension: &str, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_files(&p, extension, out);
        } else if p.extension().is_some_and(|x| x == extension) {
            out.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LIBRARY_DIRS, declared_screens, eject_from, lib_screens};
    use std::path::{Path, PathBuf};

    /// The checkout's library (what the engine package ships).
    fn library() -> [PathBuf; 2] {
        LIBRARY_DIRS.map(|d| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../packages/engine")
                .join(d)
        })
    }

    /// An empty game in a fresh temporary directory.
    fn game(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("og-ui-eject-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("project.ogt"),
            "type project\nversion 1\n---\nname \"G\"\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn eject_copies_the_library_document_into_assets_ui() {
        let dir = game("copies");
        let got = eject_from(&library(), "pause", &dir, false).unwrap();
        assert_eq!(got, dir.join("assets/ui/pause.svelte"));
        assert_eq!(
            std::fs::read(&got).unwrap(),
            std::fs::read(library()[0].join("pause.svelte")).unwrap()
        );
        assert_eq!(
            std::fs::read(dir.join("assets/ui/pause.en.ogt")).unwrap(),
            std::fs::read(library()[0].join("pause.en.ogt")).unwrap()
        );
        let got = eject_from(&library(), "ability-bar", &dir, false).unwrap();
        assert_eq!(
            std::fs::read(&got).unwrap(),
            std::fs::read(library()[1].join("ability-bar.svelte")).unwrap()
        );
        assert!(dir.join("assets/ui/ability-bar.en.ogt").is_file());
        // A group's document keeps its directory, beside its group's strings.
        let got = eject_from(&library(), "rpg/backpack", &dir, false).unwrap();
        assert_eq!(got, dir.join("assets/ui/rpg/backpack.svelte"));
        assert!(dir.join("assets/ui/rpg/rpg.en.ogt").is_file());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn eject_refuses_to_overwrite_without_force() {
        let dir = game("refuses");
        let target = dir.join("assets/ui/pause.svelte");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "the game's own").unwrap();
        let e = eject_from(&library(), "pause", &dir, false).unwrap_err();
        assert!(e.contains("already exists") && e.contains("--force"), "{e}");
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "the game's own");
        eject_from(&library(), "pause", &dir, true).unwrap();
        assert_eq!(
            std::fs::read(&target).unwrap(),
            std::fs::read(library()[0].join("pause.svelte")).unwrap()
        );
        std::fs::remove_file(&target).unwrap();
        std::fs::write(dir.join("assets/ui/pause.en.ogt"), "edited messages").unwrap();
        assert!(eject_from(&library(), "pause", &dir, false).is_err());
        assert!(
            !target.exists(),
            "a companion collision must not partially eject a screen"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("assets/ui/pause.en.ogt")).unwrap(),
            "edited messages"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn eject_list_names_every_document_with_its_appendix_b_row() {
        let screens = lib_screens(&library()).unwrap();
        // A group's documents (`rpg/backpack`) are named by their path.
        let mut files: Vec<String> = Vec::new();
        for dir in library() {
            let mut found = Vec::new();
            super::collect_files(&dir, "svelte", &mut found);
            files.extend(found.iter().map(|f| {
                f.strip_prefix(&dir)
                    .unwrap()
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/")
            }));
        }
        files.sort();
        let mut names: Vec<String> = screens.iter().map(|s| s.name.clone()).collect();
        names.sort();
        assert_eq!(names, files);
        let pause = screens.iter().find(|s| s.name == "pause").unwrap();
        assert_eq!(pause.row, "Pause and system menu");
        let rows = |row: &str| -> Vec<&str> {
            screens
                .iter()
                .filter(|s| s.row == row)
                .map(|s| s.name.as_str())
                .collect()
        };
        assert_eq!(rows("Save and load with thumbnails"), ["save-load"]);
        assert_eq!(
            rows("Boot, splash, title, main menu, mode select"),
            ["splash", "title", "main-menu", "mode-select"]
        );
    }

    #[test]
    fn eject_of_an_unknown_name_answers_the_closest() {
        let dir = game("unknown");
        let e = eject_from(&library(), "setings", &dir, false).unwrap_err();
        assert!(
            e.contains("no library screen 'setings'") && e.contains("settings"),
            "{e}"
        );
        let e = eject_from(&library(), "load", &dir, false).unwrap_err();
        assert!(e.contains("save-load") && e.contains("loading"), "{e}");
        let e = eject_from(&library(), "onboarding", &dir, false).unwrap_err();
        assert!(e.contains("the closest: tutorial"), "{e}");
        let e = eject_from(&library(), "zzzzzzzzzzzz", &dir, false).unwrap_err();
        assert!(e.contains("og ui eject --list"), "{e}");
        assert!(!dir.join("assets").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_project_declares_its_screens_in_ui_and_in_screens() {
        let dir = std::env::temp_dir().join(format!("og-ui-screens-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("project.ogt"),
            "type project\nversion 1\n---\nname \"G\"\nui {\n  screen \"hud\" \"0:key:Enter\"\n  screen \"shop\" \"0:shop\"\n}\nscreens {\n  title \"0:wait\"\n  play \"0:key:Enter\"\n}\n",
        )
        .unwrap();
        let got = declared_screens(&dir).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        let names: Vec<&str> = got.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["hud", "shop", "title", "play"]);
        assert_eq!(got[0].1, "0:key:Enter");
    }
}
