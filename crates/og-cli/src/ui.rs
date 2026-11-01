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
//! Without `--screen`, every screen the project declares: project.ogt's
//! `ui { screen "<name>" "<commands>" ... }` entries and its `screens { title
//! "<commands>" play ... results ... }` block (og style check's); without
//! those, ui-run.ts takes the values the game's show command enumerates in
//! its usage (`<a|b|c>`, the kit's `show:`), else the screens the
//! playtests take screenshots of, else the start screen.

use crate::project::{self, Project};
use std::path::Path;
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: og ui check|preview [project] [--screen <name>[=<commands>]]... [--show <template>] [--styles a,b] [--style-cmd <template>] [--breakpoints desktop,phonePortrait,phoneLandscape,tablet] [--locales game,pseudo,<cjk>,<rtl>] [--scales 100,200] [--high-contrast] [--settle S] [--film <commands> --at t,...] [--out DIR] [--json]";

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
    let given = forward.contains(&"--screen");
    let declared = if given {
        Vec::new()
    } else {
        match declared_screens(&p.root) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("og ui {mode}: {e}");
                return ExitCode::FAILURE;
            }
        }
    };
    let mut playtests = Vec::new();
    if !given && declared.is_empty() {
        crate::test::files(&p.root.join("tests"), ".og.ts", &mut playtests);
    }
    let mut cmd: Command = project::bun(&package, "src/node/ui-run.ts");
    cmd.arg(mode)
        .arg("--entry")
        .arg(&bundle)
        .arg("--root")
        .arg(&p.root)
        .arg("--out")
        .arg(&out)
        .args(&forward);
    for (name, commands) in &declared {
        cmd.arg("--screen").arg(format!("{name}={commands}"));
    }
    for t in &playtests {
        cmd.arg("--playtest").arg(t);
    }
    match cmd.status() {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        // 3: findings (check), as og style check's off-contract exit.
        Ok(s) if s.code() == Some(3) => ExitCode::from(3),
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("og ui {mode}: {} ({e})", project::NO_BUN);
            ExitCode::FAILURE
        }
    }
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
    let mut out: Vec<(String, String)> = Vec::new();
    if let Some(ui) = body.get("ui").and_then(|e| e.block.as_ref()) {
        for e in ui.entries.iter().filter(|e| e.key == "screen") {
            let name = e.string(0).map_err(|_| {
                format!(
                    "project.ogt:{}: ui's screen takes a name and its commands: screen \"hud\" \"0:show:play\"",
                    e.line
                )
            })?;
            let commands = e.values.get(1).and_then(|v| v.as_str()).unwrap_or("");
            out.push((name.to_string(), commands.to_string()));
        }
    }
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
