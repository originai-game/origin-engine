//! `og check [project|entry|tsconfig] [--json] [--checkers N] [--tool tsrs|tsgo]`:
//! type-checks game code, with the diagnostics as values.
//!
//! The checker is tsrs (the Rust port of TypeScript 7), a pinned binary run
//! as a child process, never linked: an upgrade is a new pinned binary, not
//! a rebuilt engine (docs/ci.md §Toolchain). It is found in
//! this order: `OG_TSRS` (a file), `node_modules/@tsrs/<platform>/tsrs`
//! beside the engine package (its `optionalDependencies`), `tsrs` on `PATH`
//! (`scripts/install-tools.sh`). Without one (Windows, Intel macOS) the
//! check falls back to tsgo, `bun x --package typescript@TSGO_VERSION tsc`,
//! and says so; both report through the same parser and the same JSON.
//! An install of it that bun x left incomplete (an interrupted install) is
//! an error naming the directory and the fix, found before tsgo runs
//! (`tsgo_install`) or read from tsgo's panic, never a Go stack trace.
//!
//! The project's `tsconfig.json` is the configuration. A project without
//! one (a project.ogt project that never wrote one) is checked with the
//! template's options, written to `.og/tsconfig.check.json`. The engine
//! resolves as a package either way: through the link `og` keeps in the
//! project's `node_modules` (project::LINK), by the package's `exports`
//! (`moduleResolution "bundler"`), so no configuration names an engine
//! file. A bare entry (no project.ogt) is checked alone with the same
//! options where a `node_modules` above it resolves the engine (an engine
//! checkout's own link), and refused elsewhere.
//!
//! The run is `-p <config> --noEmit --pretty false --checkers N` in the
//! project directory, with compiler-owned incremental state under `.og/check`
//! when available. A per-config file lock serializes its readers/writers;
//! `OG_SESSION=0` and the cancellable dev checks stay stateless.
//! N is `min(4, cores)` unless `--checkers` or `OG_CHECKERS` say otherwise.
//! Exit status 0 is clean; 1 to 4 is a finished check whose
//! diagnostics are on stdout; anything else, a signal, or a non-zero
//! status with nothing parsed, is the checker failing, never "no errors".
//!
//! A material shader whose mesh surface does not take one of what a scene
//! can put on it (og-render's capability table) is a warning (OG9103).
//!
//! `og run`, `og test` and `og build` check before they bundle (`gate`),
//! and stop on a type error unless `--no-check`; `og dev` checks each
//! rebuild beside the bundle and reports without refusing the reload.

use crate::project::{self, Project, typescript_path};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

mod fields;

/// The TypeScript (tsgo) version tsrs is held to
/// (`scripts/tsrs-diff.sh` reads it here) and the fallback where tsrs has
/// no package (Windows, Intel macOS).
pub const TSGO_VERSION: &str = "7.0.2";

/// The template's compiler options: the options of a project without a
/// tsconfig.json of its own.
const TEMPLATE_TSCONFIG: &str = include_str!("../../../../packages/create/template/tsconfig.json");

const USAGE: &str = "usage: og check [project|entry|tsconfig.json] [--json] [--checkers N] [--tool tsrs|tsgo] [--require-checked]";

mod lints;
mod tsgo;
mod ui;
use lints::{
    font_packs_lint, locale_fonts_lint, lockstep_lint, module_state, shader_capabilities_lint,
};
pub(crate) use tsgo::settled_tsgo_fault;
pub use tsgo::{Checker, NO_TSRS, Tool, default_checkers, locate_tsrs, platform};
use ui::{missing_tables, pack_grounds, ui_grounds_lint, unknown_classes};

/// One diagnostic, as the checker printed it.
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    /// Relative to the project directory, as printed; none for a global one.
    pub file: Option<String>,
    /// 1-based.
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub code: u32,
    /// The message, its continuation lines joined with `\n`.
    pub message: String,
}

/// What one check found.
#[derive(Clone, Debug)]
pub struct Report {
    pub tool: &'static str,
    pub version: String,
    /// The configuration checked, relative to the project directory.
    pub config: String,
    pub ms: f64,
    pub diagnostics: Vec<Diagnostic>,
    /// The checker's stdout, for the text mode (editor-clickable lines).
    pub raw: String,
}

/// What is checked: a configuration, run from a directory.
#[derive(Clone, Debug)]
pub struct Target {
    pub root: PathBuf,
    pub config: PathBuf,
}

impl Target {
    fn display_config(&self) -> String {
        typescript_path(self.config.strip_prefix(&self.root).unwrap_or(&self.config))
    }

    /// A tsconfig file, or a directory holding `tsconfig.json`, as is.
    pub fn config(path: &Path) -> Result<Target, String> {
        let config = if path.is_dir() {
            path.join("tsconfig.json")
        } else {
            path.to_path_buf()
        };
        let config = config
            .canonicalize()
            .map_err(|e| format!("{}: {e}", config.display()))?;
        Ok(Target {
            root: config.parent().unwrap_or(Path::new(".")).to_path_buf(),
            config,
        })
    }

    /// The configuration of a project (see the module doc).
    pub fn of(p: &Project) -> Result<Target, String> {
        let root = PathBuf::from(typescript_path(
            &p.root.canonicalize().unwrap_or_else(|_| p.root.clone()),
        ));
        let own = root.join("tsconfig.json");
        if own.is_file() {
            return Ok(Target { root, config: own });
        }
        // The engine resolves as a package, through the project's link
        // (project::LINK) or one in a node_modules above it (an engine
        // checkout's own). A bare entry has no link of its own.
        if !p.manifest && !root.ancestors().any(|d| d.join(project::LINK).exists()) {
            return Err(format!(
                "{} is in no project (no project.ogt above it): the engine resolves for a type check through a project's {}; check it in a project (og new), or pass --no-check",
                p.entry.display(),
                project::LINK
            ));
        }
        let template: serde_json::Value = serde_json::from_str(TEMPLATE_TSCONFIG)
            .expect("packages/create/template/tsconfig.json");
        let include: Vec<String> = if p.manifest {
            template["include"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|s| s.as_str())
                .map(|s| typescript_path(&root.join(s)))
                .collect()
        } else {
            let entry = p.entry.canonicalize().unwrap_or_else(|_| p.entry.clone());
            vec![typescript_path(&entry)]
        };
        let config = serde_json::json!({ "compilerOptions": template["compilerOptions"], "include": include });
        let dir = p.work_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = dir.join("tsconfig.check.json");
        let text = format!(
            "// Written by og check (crates/og-cli/src/check.rs): the type-check configuration\n// of a project without a tsconfig.json, the template's. Do not edit; it is rewritten.\n{}\n",
            serde_json::to_string_pretty(&config).expect("json")
        );
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        Ok(Target { root, config: path })
    }
}

impl Report {
    pub fn ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn files(&self) -> usize {
        let mut f: Vec<&str> = self
            .diagnostics
            .iter()
            .filter_map(|d| d.file.as_deref())
            .collect();
        f.sort_unstable();
        f.dedup();
        f.len()
    }

    /// The `check` event: one JSON line.
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "event": "check",
            "ok": self.ok(),
            "uncheckedCount": 0,
            "tool": self.tool,
            "version": self.version,
            "config": self.config,
            "ms": self.ms,
            "diagnostics": self.diagnostics.iter().map(|d| serde_json::json!({
                "file": d.file, "line": d.line, "col": d.col, "code": d.code, "message": d.message,
            })).collect::<Vec<_>>(),
        })
    }

    /// The short form carried by `og dev`'s reload and `og test`'s result.
    pub fn brief(&self) -> serde_json::Value {
        serde_json::json!({ "ok": self.ok(), "ms": self.ms, "errors": self.diagnostics.len() })
    }

    /// `og check: ok (tsrs, 41 ms)` or `og check: N error(s) in M file(s) (tsrs, 41 ms)`.
    pub fn summary(&self) -> String {
        let what = format!("({}, {:.0} ms)", self.tool, self.ms);
        if self.ok() {
            format!("og check: ok {what}")
        } else {
            format!(
                "og check: {} error(s) in {} file(s) {what}",
                self.diagnostics.len(),
                self.files()
            )
        }
    }
}

/// The check `og run`, `og test` and `og build` make before bundling:
/// prints the diagnostics (the `check` event with `json`) and returns the
/// report; `Err` when no checker could check.
pub fn gate(p: &Project, package: &Path, json: bool) -> Result<Report, String> {
    let (checker, note) = Checker::find(Some(package), None)?;
    if let Some(n) = note {
        eprintln!("og check: {n}");
    }
    let target = Target::of(p)?;
    let mut report = checker.check(&target, default_checkers())?;
    module_state(p, &mut report);
    unknown_classes(p, &mut report);
    pack_grounds(p, &mut report);
    missing_tables(&p.root, &project_dirs(p), &mut report);
    fields::check(p, &mut report);
    if json {
        println!("{}", report.json());
    } else {
        eprint!("{}", report.raw);
        eprintln!("{}", report.summary());
    }
    lockstep_lint(p, json, true);
    locale_fonts_lint(p, json, true);
    font_packs_lint(p, json, true);
    shader_capabilities_lint(p, json, true);
    Ok(report)
}

/// The directories whose sources a project's check reads.
fn project_dirs(p: &Project) -> Vec<PathBuf> {
    vec![
        p.entry.parent().unwrap_or(&p.root).to_path_buf(),
        p.root.join("assets/ui"),
    ]
}

/// `gate`, and `Err` on a type error as well: what stops a bundle.
pub fn require(p: &Project, package: &Path, json: bool) -> Result<Report, String> {
    let report = gate(p, package, json)?;
    if report.ok() {
        Ok(report)
    } else {
        Err(refusal(&report))
    }
}

pub fn refusal(report: &Report) -> String {
    format!(
        "{} error(s) in the check; fix them or pass --no-check",
        report.diagnostics.len()
    )
}

pub fn run(args: &[&str]) -> ExitCode {
    let mut path = ".".to_string();
    let mut json = false;
    let mut checkers = None;
    let mut tool = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => json = true,
            "--require-checked" => {} // Type checking never skips a GPU check.
            flag @ ("--checkers" | "--tool") => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("og check: {flag} needs a value. {USAGE}");
                    return ExitCode::from(2);
                };
                if flag == "--checkers" {
                    match v.parse::<usize>() {
                        Ok(n) if n > 0 => checkers = Some(n),
                        _ => {
                            eprintln!("og check: --checkers takes a number above 0. {USAGE}");
                            return ExitCode::from(2);
                        }
                    }
                } else {
                    tool = match *v {
                        "tsrs" => Some(Tool::Tsrs),
                        "tsgo" => Some(Tool::Tsgo),
                        _ => {
                            eprintln!("og check: --tool is tsrs or tsgo. {USAGE}");
                            return ExitCode::from(2);
                        }
                    };
                }
                i += 1;
            }
            a if a.starts_with('-') => {
                eprintln!("og check: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            a => path = a.to_string(),
        }
        i += 1;
    }
    let fail = |e: String| {
        eprintln!("og check: {e}");
        ExitCode::FAILURE
    };
    let path = Path::new(&path);
    // A tsconfig, or a directory with one and no project.ogt (packages/engine):
    // the configuration as is. Otherwise a project or an entry.
    let as_config = path.extension().is_some_and(|e| e == "json")
        || (path.is_dir()
            && !path.join("project.ogt").is_file()
            && path.join("tsconfig.json").is_file());
    let mut project = None;
    let (target, package) = if as_config {
        let target = match Target::config(path) {
            Ok(t) => t,
            Err(e) => return fail(e),
        };
        let package = project::engine_package(Some(&target.root));
        (target, package)
    } else {
        let p = match Project::open(path) {
            Ok(p) => p,
            Err(e) => return fail(e),
        };
        let package = project::engine_package(Some(&p.root));
        let target = match Target::of(&p) {
            Ok(t) => t,
            Err(e) => return fail(e),
        };
        project = Some(p);
        (target, package)
    };
    let (checker, note) = match Checker::find(package.as_deref(), tool) {
        Ok(c) => c,
        Err(e) => return fail(e),
    };
    if let Some(n) = note {
        eprintln!("og check: {n}");
    }
    let mut report = match checker.check(&target, checkers.unwrap_or_else(default_checkers)) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    if let Some(p) = &project {
        module_state(p, &mut report);
        unknown_classes(p, &mut report);
        pack_grounds(p, &mut report);
        missing_tables(&p.root, &project_dirs(p), &mut report);
        fields::check(p, &mut report);
    } else {
        // A configuration (packages/engine): every source under it, so an
        // engine component's bad import fails the engine's own typecheck.
        missing_tables(
            &target.root,
            std::slice::from_ref(&target.root),
            &mut report,
        );
    }
    if json {
        println!("{}", report.json());
    } else {
        print!("{}", report.raw);
        println!("{}", report.summary());
    }
    if let Some(p) = &project {
        lockstep_lint(p, json, false);
        locale_fonts_lint(p, json, false);
        font_packs_lint(p, json, false);
        shader_capabilities_lint(p, json, false);
        let game = p.root.join("assets/styles/game/pack.json");
        let dirs = [
            p.entry.parent().unwrap_or(&p.root),
            &p.root.join("assets/ui"),
        ];
        ui_grounds_lint(&p.root, &dirs, Some(&game), json);
        crate::ui_data::lint(&p.root, &dirs, json);
        crate::ui_styles::lint(&p.root, &dirs, json);
    } else {
        ui_grounds_lint(&target.root, &[&target.root], None, json);
        crate::ui_data::lint(&target.root, &[&target.root], json);
        crate::ui_styles::lint(&target.root, &[&target.root], json);
    }
    if report.ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
