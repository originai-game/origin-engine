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
//! `og ui eject` copies a built-in TSX component, screen or control into
//! the game and points the game's imports at the copy (`ui/eject.rs`).
//!
//! Without `--screen`, every screen the project declares: project.ogt's
//! `ui { screen "<name>" "<commands>" ... }` entries and its `screens { title
//! "<commands>" play ... results ... }` block (og style check-project's); without
//! those, ui-run.ts takes the values the game's show command enumerates in
//! its usage (`<a|b|c>`, the kit's `show:`), else the screens the
//! playtests take screenshots of, else the start screen.

use crate::project::{self, Project};
use std::path::Path;
use std::process::{Command, ExitCode};

mod eject;
pub mod extensions;
mod literals;

const USAGE: &str = "usage: og ui add <materials|directory> [project]\n       og ui list [project] [--json]\n       og ui literals [path...] [--json]\n       og ui eject <component|screens/<name>|controls/<name>> [project] [--force]\n       og ui eject --list [project]\n       og ui check|preview [project] [--screen <name>[=<commands>]]... [--show <template>] [--styles a,b] [--style-cmd <template>] [--breakpoints desktop,phonePortrait,phoneLandscape,tablet] [--locales game,pseudo,<cjk>,<rtl>] [--scales 100,150] [--high-contrast] [--settle S] [--film <commands> --at t,...] [--out DIR] [--json]";

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
    if let ["add", rest @ ..] = args {
        return extensions::add(rest, "materials");
    }
    if let ["list", rest @ ..] = args {
        return extensions::list(rest, None);
    }
    if let ["literals", rest @ ..] = args {
        return literals::run(rest);
    }
    if let ["eject", rest @ ..] = args {
        return eject::run(rest, USAGE);
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
