//! `og ui check` and `og ui preview` (docs/ui.md §16.4, §16.5): a project's
//! screens laid out over styles × breakpoints × locales × text scale on
//! the native host (packages/engine/src/node/ui-run.ts), each cell driven
//! by a command script. `check` reports every rule of ui-check.ts with the
//! element paths, the measured value and the prop to change, into
//! DIR/check.json and check.txt (default `<project>/og-out/ui-check`), and
//! exits 3 on an error finding; `preview` writes a contact sheet per screen
//! and style and index.json (default `<project>/og-out/ui-preview`), or with
//! `--film` a filmstrip of a motion at chosen times.
//!
//! `og ui slots [project] [--json]` (docs/ui.md §13.3) lists every art
//! slot of the game, read from its documents without running it: each
//! `art` element's path, kind, aspect, focal point, safe rectangle and fit,
//! whether its art is set or it still draws the style's placeholder, and
//! what its spec gets wrong with the fix (exit 3 on one). `--json` prints
//! them as one array, what the art pipeline reads.
//!
//! `og ui eject <screen> [project] [--force]` (docs/ui.md §11) copies one
//! of the library's screen documents (the engine package's
//! `src/ui/screens/<screen>.ogt`, one or more for every screen type of
//! docs/ui.md Appendix B) into the game's `assets/ui/<screen>.ogt`, where
//! `og.load.document(world, "ui/<screen>.ogt")` loads it and the game edits
//! it as its own file. It refuses to overwrite one already there without
//! `--force`; an unknown name exits 1 with the closest names. `og ui eject
//! --list [project]` prints every screen with its Appendix B row, read from
//! the table of that directory's README.md, marking those that leave a slot
//! for a pattern still to come. The package is the one og run uses
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
use std::path::Path;
use std::process::{Command, ExitCode, ExitStatus};

const USAGE: &str = "usage: og ui slots [project] [--json]\n       og ui eject <screen> [project] [--force]\n       og ui eject --list [project]\n       og ui check|preview [project] [--screen <name>[=<commands>]]... [--show <template>] [--styles a,b] [--style-cmd <template>] [--breakpoints desktop,phonePortrait,phoneLandscape,tablet] [--locales game,pseudo,<cjk>,<rtl>] [--scales 100,200] [--high-contrast] [--settle S] [--film <commands> --at t,...] [--out DIR] [--json]";

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
    if let ["slots", rest @ ..] = args {
        return slots(rest);
    }
    if let ["eject", rest @ ..] = args {
        return eject(rest);
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
            "--json" | "--high-contrast" => forward.push(rest[i]),
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
        Ok(s) if s.success() => ExitCode::SUCCESS,
        // 3: findings (check), as og style check's off-contract exit.
        Ok(s) if s.code() == Some(3) => ExitCode::from(3),
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("og ui {mode}: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Runs ui-run.ts's `check` or `preview` into `out` on the cooked project's
/// native bundle; without a `--screen` in `forward`, over every screen the
/// project declares (og polish runs the check this way, over the same set).
pub fn spawn(
    mode: &str,
    p: &Project,
    package: &Path,
    bundle: &Path,
    out: &Path,
    forward: &[&str],
) -> Result<ExitStatus, String> {
    let given = forward.contains(&"--screen");
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
    cmd.status()
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
    collect_ogt(&assets, &mut files);
    files.sort();
    let mut rows = Vec::new();
    let mut wrong = 0;
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        let ty = text.lines().next().unwrap_or("");
        if ty != "type screen" && ty != "type component" {
            continue;
        }
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
        let out: Vec<serde_json::Value> = rows
            .iter()
            .map(|(id, s, problems)| {
                let mut v = og_ui::doc::build::slot_json(&s.spec);
                if let Some(o) = v.as_object_mut() {
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

/// Where the library's screen documents sit in the engine package.
const SCREENS_DIR: &str = "src/ui/screens";

/// One library screen: its name (the file's stem), its Appendix B row, and
/// whether it leaves a slot for a pattern still to come.
#[derive(Debug, PartialEq)]
struct LibScreen {
    name: String,
    row: String,
    pending: bool,
}

/// The library's screens in the order of the README's table
/// (`| <Appendix B row> | `a`, `b` | <the pending pattern of `a`> |`).
fn lib_screens(dir: &Path) -> Result<Vec<LibScreen>, String> {
    let readme = dir.join("README.md");
    let text = std::fs::read_to_string(&readme)
        .map_err(|e| format!("{}: {e} (reinstall @origingame/engine)", readme.display()))?;
    let mut out = Vec::new();
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
        let pending: Vec<&str> = cells
            .get(3)
            .map_or("", |c| c)
            .split('`')
            .skip(1)
            .step_by(2)
            .collect();
        for d in docs.split(',') {
            let name = d.trim().trim_matches('`').to_string();
            out.push(LibScreen {
                pending: pending.contains(&name.as_str()),
                name,
                row: (*row).to_string(),
            });
        }
    }
    Ok(out)
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

/// Copies the library's `name` from `dir` into `project`'s
/// `assets/ui/<name>.ogt`; answers the path written.
fn eject_from(
    dir: &Path,
    name: &str,
    project: &Path,
    force: bool,
) -> Result<std::path::PathBuf, String> {
    let screens = lib_screens(dir)?;
    if !screens.iter().any(|s| s.name == name) {
        let near = closest(name, &screens);
        return Err(if near.is_empty() {
            format!("no library screen '{name}' (og ui eject --list names them)")
        } else {
            format!(
                "no library screen '{name}'; the closest: {} (og ui eject --list names them all)",
                near.join(", ")
            )
        });
    }
    if !project.join("project.ogt").is_file() {
        return Err(format!(
            "{}: no project.ogt; run it in a game's directory or pass the project",
            project.display()
        ));
    }
    let source = dir.join(format!("{name}.ogt"));
    let bytes = std::fs::read(&source).map_err(|e| format!("{}: {e}", source.display()))?;
    let target = project.join("assets/ui").join(format!("{name}.ogt"));
    if target.exists() && !force {
        return Err(format!(
            "assets/ui/{name}.ogt already exists; nothing written (--force overwrites it)"
        ));
    }
    let parent = target.parent().expect("assets/ui");
    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    std::fs::write(&target, bytes).map_err(|e| format!("{}: {e}", target.display()))?;
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
    let dir = package.join(SCREENS_DIR);
    let Some(name) = name else {
        return match lib_screens(&dir) {
            Ok(screens) => {
                for s in &screens {
                    let pending = if s.pending { "  (pattern pending)" } else { "" };
                    println!("{:<20} {}{pending}", s.name, s.row);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("og ui eject: {e}");
                ExitCode::FAILURE
            }
        };
    };
    match eject_from(&dir, name, project, force) {
        Ok(_) => {
            println!(
                "og ui eject: the library's '{name}' into assets/ui/{name}.ogt\n\
                 load it once with og.load.document(world, \"ui/{name}.ogt\") and push it with og.ui.push(world, \"{name}\", vm);\n\
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

fn collect_ogt(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_ogt(&p, out);
        } else if p.extension().is_some_and(|x| x == "ogt") {
            out.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SCREENS_DIR, declared_screens, eject_from, lib_screens};
    use std::path::{Path, PathBuf};

    /// The checkout's library (what the engine package ships).
    fn library() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/engine")
            .join(SCREENS_DIR)
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
        let got = eject_from(&library(), "inventory", &dir, false).unwrap();
        assert_eq!(got, dir.join("assets/ui/inventory.ogt"));
        assert_eq!(
            std::fs::read(&got).unwrap(),
            std::fs::read(library().join("inventory.ogt")).unwrap()
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn eject_refuses_to_overwrite_without_force() {
        let dir = game("refuses");
        let target = dir.join("assets/ui/pause.ogt");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "the game's own").unwrap();
        let e = eject_from(&library(), "pause", &dir, false).unwrap_err();
        assert!(e.contains("already exists") && e.contains("--force"), "{e}");
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "the game's own");
        eject_from(&library(), "pause", &dir, true).unwrap();
        assert_eq!(
            std::fs::read(&target).unwrap(),
            std::fs::read(library().join("pause.ogt")).unwrap()
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn eject_list_names_every_document_with_its_appendix_b_row() {
        let screens = lib_screens(&library()).unwrap();
        let mut files: Vec<String> = std::fs::read_dir(library())
            .unwrap()
            .filter_map(|e| {
                let f = e.unwrap().file_name().into_string().unwrap();
                f.strip_suffix(".ogt").map(str::to_string)
            })
            .collect();
        files.sort();
        let mut names: Vec<String> = screens.iter().map(|s| s.name.clone()).collect();
        names.sort();
        assert_eq!(names, files);
        let pause = screens.iter().find(|s| s.name == "pause").unwrap();
        assert_eq!(pause.row, "Pause and system menu");
        assert!(!pause.pending);
        let garage = screens.iter().find(|s| s.name == "garage").unwrap();
        assert_eq!(garage.row, "Garage, tuning, livery editor");
        assert!(garage.pending);
        let backlog = screens.iter().find(|s| s.name == "backlog").unwrap();
        assert!(
            !backlog.pending,
            "the row's gallery and flowchart are pending, not its backlog"
        );
    }

    #[test]
    fn eject_of_an_unknown_name_answers_the_closest() {
        let dir = game("unknown");
        let e = eject_from(&library(), "inventroy", &dir, false).unwrap_err();
        assert!(
            e.contains("no library screen 'inventroy'") && e.contains("inventory"),
            "{e}"
        );
        let e = eject_from(&library(), "map", &dir, false).unwrap_err();
        assert!(e.contains("world-map") && e.contains("saga-map"), "{e}");
        let e = eject_from(&library(), "shop", &dir, false).unwrap_err();
        assert!(e.contains("the closest: vendor"), "{e}");
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
