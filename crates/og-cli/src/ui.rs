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
//! Without `--screen`, every screen the project declares: project.ogt's
//! `ui { screen "<name>" "<commands>" ... }` entries and its `screens { title
//! "<commands>" play ... results ... }` block (og style check's); without
//! those, ui-run.ts takes the values the game's show command enumerates in
//! its usage (`<a|b|c>`, the kit's `show:`), else the screens the
//! playtests take screenshots of, else the start screen.

use crate::project::{self, Project};
use std::path::Path;
use std::process::{Command, ExitCode, ExitStatus};

const USAGE: &str = "usage: og ui slots [project] [--json]\n       og ui check|preview [project] [--screen <name>[=<commands>]]... [--show <template>] [--styles a,b] [--style-cmd <template>] [--breakpoints desktop,phonePortrait,phoneLandscape,tablet] [--locales game,pseudo,<cjk>,<rtl>] [--scales 100,200] [--high-contrast] [--settle S] [--film <commands> --at t,...] [--out DIR] [--json]";

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
    use super::declared_screens;

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
