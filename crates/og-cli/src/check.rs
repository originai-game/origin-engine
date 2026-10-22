//! `og check [project|entry|tsconfig] [--json] [--checkers N] [--tool tsrs|tsgo]`:
//! type-checks game code, with the diagnostics as values.
//!
//! The checker is tsrs (the Rust port of TypeScript 7), a pinned binary run
//! as a child process, never linked (goals/tsrs/plan.md D1). It is found in
//! this order: `OG_TSRS` (a file), `node_modules/@tsrs/<platform>/tsrs`
//! beside the engine package (its `optionalDependencies`), `tsrs` on `PATH`
//! (`scripts/install-tools.sh`). Without one (Windows, Intel macOS) the
//! check falls back to tsgo, `bun x --package typescript@TSGO_VERSION tsc`,
//! and says so; both report through the same parser and the same JSON.
//!
//! The project's `tsconfig.json` is the configuration. A project without
//! one (a bare entry, a project.ogt project that never wrote one) is
//! checked with the template's options, written to
//! `.og/tsconfig.check.json` (a temporary directory for a bare entry); a
//! project whose engine package is not in its `node_modules` (one inside an
//! engine checkout, run with `OG_ENGINE_PACKAGE`) gets the same file
//! extending its own with `paths` to the engine package `og` resolved.
//!
//! The run is `-p <config> --noEmit --incremental false --pretty false
//! --checkers N` in the project directory: stateless (no `.tsbuildinfo`),
//! with N `min(4, cores)` unless `--checkers` or `OG_CHECKERS` say
//! otherwise. Exit status 0 is clean; 1 to 4 is a finished check whose
//! diagnostics are on stdout; anything else, a signal, or a non-zero
//! status with nothing parsed, is the checker failing, never "no errors".
//!
//! `og run`, `og test` and `og build` check before they bundle (`gate`),
//! and stop on a type error unless `--no-check`; `og dev` checks each
//! rebuild beside the bundle and reports without refusing the reload.

use crate::project::{self, Project};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

/// The TypeScript (tsgo) version that is the authority while tsrs is not
/// (goals/tsrs/plan.md D4), and the fallback where tsrs has no package.
/// The templates' and examples' `devDependencies.typescript` copy it;
/// `scripts/tsrs-diff.sh` reads it here and holds them equal.
pub const TSGO_VERSION: &str = "7.0.2";

/// The template's compiler options: the options of a project without a
/// tsconfig.json of its own.
const TEMPLATE_TSCONFIG: &str = include_str!("../../../templates/game/tsconfig.json");

const USAGE: &str =
    "usage: og check [project|entry|tsconfig.json] [--json] [--checkers N] [--tool tsrs|tsgo]";

pub const NO_TSRS: &str = "no tsrs: `bun install` in the project (@origingame/engine brings @tsrs/<platform>), run scripts/install-tools.sh in an engine checkout, or set OG_TSRS to the binary";

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

/// Parses `--pretty false` output:
///
/// ```text
/// <path>(<line>,<col>): error TS<code>: <message>
/// error TS<code>: <message>
///   <continuation, joined to the message above>
/// ```
pub fn parse(text: &str) -> Vec<Diagnostic> {
    let mut out: Vec<Diagnostic> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with(char::is_whitespace) {
            if let Some(last) = out.last_mut() {
                last.message.push('\n');
                last.message
                    .push_str(line.strip_prefix("  ").unwrap_or(line));
            }
            continue;
        }
        if let Some(d) = parse_head(line) {
            out.push(d);
        }
    }
    out
}

/// `error TS<code>: <message>` at the start of `s`.
fn code_and_message(s: &str) -> Option<(u32, &str)> {
    let rest = s.strip_prefix("error TS")?;
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let code = rest[..digits].parse().ok()?;
    let message = rest[digits..].strip_prefix(": ")?;
    Some((code, message))
}

fn parse_head(line: &str) -> Option<Diagnostic> {
    if let Some((code, message)) = code_and_message(line) {
        return Some(Diagnostic {
            file: None,
            line: None,
            col: None,
            code,
            message: message.to_string(),
        });
    }
    // The first `(<line>,<col>): error TS<code>: ` ends the path: a path may
    // hold parentheses, and the message comes after it.
    for (at, _) in line.match_indices('(') {
        let rest = &line[at + 1..];
        let Some((pos, tail)) = rest.split_once("): ") else {
            continue;
        };
        let Some((l, c)) = pos.split_once(',') else {
            continue;
        };
        let (Ok(l), Ok(c)) = (l.parse::<u32>(), c.parse::<u32>()) else {
            continue;
        };
        if let Some((code, message)) = code_and_message(tail) {
            return Some(Diagnostic {
                file: Some(line[..at].to_string()),
                line: Some(l),
                col: Some(c),
                code,
                message: message.to_string(),
            });
        }
    }
    None
}

/// How a checker's exit status reads (TypeScript's `ExitStatus`): `Ok`
/// with the diagnostics for a finished check, `Err` for a failure.
pub fn outcome(code: Option<i32>, stdout: &str) -> Result<Vec<Diagnostic>, String> {
    match code {
        Some(0) => Ok(parse(stdout)),
        Some(1..=4) => {
            let d = parse(stdout);
            if d.is_empty() {
                Err(format!(
                    "exited with status {} and reported nothing it could be read as",
                    code.unwrap_or_default()
                ))
            } else {
                Ok(d)
            }
        }
        Some(c) => Err(format!("exited with status {c}")),
        None => Err("was killed by a signal".into()),
    }
}

/// Which checker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tool {
    Tsrs,
    Tsgo,
}

/// A checker found and asked its version, once per og command.
#[derive(Clone, Debug)]
pub struct Checker {
    pub tool: Tool,
    /// The tsrs binary (none for tsgo, which runs through `bun x`).
    pub path: Option<PathBuf>,
    pub version: String,
}

/// The npm platform package of tsrs for this build of og.
pub fn platform() -> Option<&'static str> {
    let musl = cfg!(target_env = "musl");
    match (std::env::consts::OS, std::env::consts::ARCH, musl) {
        ("linux", "x86_64", false) => Some("linux-x64-gnu"),
        ("linux", "aarch64", false) => Some("linux-arm64-gnu"),
        ("linux", "x86_64", true) => Some("linux-x64-musl"),
        ("linux", "aarch64", true) => Some("linux-arm64-musl"),
        ("macos", "aarch64", _) => Some("darwin-arm64"),
        _ => None,
    }
}

/// A tsrs binary that was found, with the versions it reports.
#[derive(Clone, Debug)]
pub struct Tsrs {
    pub path: PathBuf,
    pub source: &'static str,
    /// The TypeScript version it ports (`7.1.0-dev.20260930.4`).
    pub version: String,
    /// Its own package version (`0.18.0-ts7.1.0-dev.20260929`).
    pub package_version: String,
}

/// Finds tsrs by the chain above; `package` is the engine package. `None`
/// when there is none; `Err` when the one found cannot be used (OG_TSRS
/// naming no file, a binary apart from its lib files, one that is no tsrs).
pub fn locate_tsrs(package: Option<&Path>) -> Result<Option<Tsrs>, String> {
    let exe = if cfg!(windows) { "tsrs.exe" } else { "tsrs" };
    let (path, source) = if let Some(p) = std::env::var_os("OG_TSRS").filter(|p| !p.is_empty()) {
        let p = PathBuf::from(p);
        if !p.is_file() {
            return Err(format!("OG_TSRS={} is not a file", p.display()));
        }
        (p, "OG_TSRS")
    } else if let Some(p) = package.and_then(|pkg| {
        let plat = platform()?;
        pkg.ancestors()
            .map(|d| d.join("node_modules/@tsrs").join(plat).join(exe))
            .find(|p| p.is_file())
    }) {
        (p, "node_modules")
    } else if let Some(p) = std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|d| d.join(exe))
            .find(|p| p.is_file())
    }) {
        (p, "PATH")
    } else {
        return Ok(None);
    };
    // tsrs reads lib.*.d.ts from its own directory (the npm build embeds
    // none): a binary copied away from them would check against nothing.
    let real = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    let dir = real.parent().unwrap_or(Path::new("."));
    if !dir.join("lib.es2022.d.ts").is_file() {
        return Err(format!(
            "{} has no lib.es2022.d.ts beside it in {}: tsrs reads its lib files from its own directory; reinstall the whole package (scripts/install-tools.sh or bun install)",
            path.display(),
            dir.display()
        ));
    }
    let out = Command::new(&path)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot start {} ({e})", path.display()))?;
    // `Version 7.1.0-dev.20260930.4 (tsrs 0.18.0-ts7.1.0-dev.20260929, microsoft/TypeScript@...)`
    let text = String::from_utf8_lossy(&out.stdout);
    let mut words = text.split_whitespace();
    let version = match (words.next(), words.next()) {
        (Some("Version"), Some(v)) if out.status.success() => v.to_string(),
        _ => {
            return Err(format!(
                "{} --version printed `{}`; not a tsrs",
                path.display(),
                text.trim()
            ));
        }
    };
    let package_version = text
        .split("(tsrs ")
        .nth(1)
        .and_then(|r| r.split([',', ')']).next())
        .unwrap_or("")
        .to_string();
    Ok(Some(Tsrs {
        path,
        source,
        version,
        package_version,
    }))
}

impl Checker {
    /// `tool` asked for, or tsrs and then tsgo; the note is why tsgo was
    /// taken when none was asked for.
    pub fn find(
        package: Option<&Path>,
        tool: Option<Tool>,
    ) -> Result<(Checker, Option<String>), String> {
        let tsgo = Checker {
            tool: Tool::Tsgo,
            path: None,
            version: TSGO_VERSION.into(),
        };
        match tool {
            Some(Tool::Tsgo) => Ok((tsgo, None)),
            Some(Tool::Tsrs) => match locate_tsrs(package)? {
                Some(t) => Ok((Checker::from(t), None)),
                None => Err(NO_TSRS.into()),
            },
            // Only a missing tsrs falls back; a broken one is an error.
            None => match locate_tsrs(package)? {
                Some(t) => Ok((Checker::from(t), None)),
                None => Ok((
                    tsgo,
                    Some(format!(
                        "{NO_TSRS}; checking with tsgo {TSGO_VERSION} through bun x (needs the network or bun's cache)"
                    )),
                )),
            },
        }
    }

    pub fn name(&self) -> &'static str {
        match self.tool {
            Tool::Tsrs => "tsrs",
            Tool::Tsgo => "tsgo",
        }
    }

    fn command(&self) -> Command {
        match &self.path {
            Some(p) => Command::new(p),
            None => {
                let mut c = Command::new("bun");
                c.args([
                    "x",
                    "--package",
                    &format!("typescript@{TSGO_VERSION}"),
                    "tsc",
                ]);
                c
            }
        }
    }

    /// Checks `target` with `checkers` threads.
    pub fn check(&self, target: &Target, checkers: usize) -> Result<Report, String> {
        let t0 = Instant::now();
        let out = self
            .command()
            .arg("-p")
            .arg(&target.config)
            .args([
                "--noEmit",
                "--incremental",
                "false",
                "--pretty",
                "false",
                "--checkers",
            ])
            .arg(checkers.max(1).to_string())
            .current_dir(&target.root)
            .stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| match self.tool {
                Tool::Tsrs => format!("cannot start tsrs ({e})"),
                Tool::Tsgo => format!("{} ({e})", project::NO_BUN),
            })?;
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let raw = String::from_utf8_lossy(&out.stdout).into_owned();
        let diagnostics = outcome(out.status.code(), &raw).map_err(|e| {
            format!(
                "{} failed: it {e}; its output:\n{}",
                self.name(),
                raw.trim_end()
            )
        })?;
        Ok(Report {
            tool: self.name(),
            version: self.version.clone(),
            config: target.display_config(),
            ms: (ms * 10.0).round() / 10.0,
            diagnostics,
            raw,
        })
    }
}

impl From<Tsrs> for Checker {
    fn from(t: Tsrs) -> Checker {
        Checker {
            tool: Tool::Tsrs,
            path: Some(t.path),
            version: t.version,
        }
    }
}

/// The checker threads: `OG_CHECKERS`, else `min(4, cores)` (the gate and
/// parallel agents already fill the machine).
pub fn default_checkers() -> usize {
    std::env::var("OG_CHECKERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n: &usize| n > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                .min(4)
        })
}

/// What is checked: a configuration, run from a directory.
#[derive(Clone, Debug)]
pub struct Target {
    pub root: PathBuf,
    pub config: PathBuf,
}

impl Target {
    fn display_config(&self) -> String {
        self.config
            .strip_prefix(&self.root)
            .unwrap_or(&self.config)
            .display()
            .to_string()
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
    pub fn of(p: &Project, package: Option<&Path>) -> Result<Target, String> {
        let root = p.root.canonicalize().unwrap_or_else(|_| p.root.clone());
        let own = root.join("tsconfig.json");
        let engine_in_node_modules = root
            .ancestors()
            .any(|d| d.join("node_modules/@origingame/engine").is_dir());
        let engine_paths = package.map(|pkg| {
            let pkg = pkg.canonicalize().unwrap_or_else(|_| pkg.to_path_buf());
            serde_json::json!({
                "@origingame/engine": [pkg.join("types/index.d.ts")],
                "@origingame/engine/playtest": [pkg.join("src/playtest.ts")],
            })
        });
        let config = if own.is_file() {
            let text =
                std::fs::read_to_string(&own).map_err(|e| format!("{}: {e}", own.display()))?;
            match engine_paths {
                Some(paths)
                    if !engine_in_node_modules && !text.contains("\"@origingame/engine\"") =>
                {
                    let mut paths = paths;
                    // Its own paths survive the override (compilerOptions
                    // merge per option), made absolute.
                    if let Some(theirs) = serde_json::from_str::<serde_json::Value>(&text)
                        .ok()
                        .and_then(|v| v["compilerOptions"]["paths"].as_object().cloned())
                    {
                        for (k, v) in theirs {
                            let abs: Vec<String> = v
                                .as_array()
                                .into_iter()
                                .flatten()
                                .filter_map(|s| s.as_str())
                                .map(|s| root.join(s).display().to_string())
                                .collect();
                            paths[k] = serde_json::json!(abs);
                        }
                    }
                    serde_json::json!({
                        "extends": own,
                        "compilerOptions": { "paths": paths },
                    })
                }
                _ => {
                    return Ok(Target { root, config: own });
                }
            }
        } else {
            let template: serde_json::Value =
                serde_json::from_str(TEMPLATE_TSCONFIG).expect("templates/game/tsconfig.json");
            let mut options = template["compilerOptions"].clone();
            if let Some(paths) = engine_paths.filter(|_| !engine_in_node_modules) {
                options["paths"] = paths;
            }
            let include: Vec<String> = if p.manifest {
                template["include"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|s| s.as_str())
                    .map(|s| root.join(s).display().to_string())
                    .collect()
            } else {
                let entry = p.entry.canonicalize().unwrap_or_else(|_| p.entry.clone());
                vec![entry.display().to_string()]
            };
            serde_json::json!({ "compilerOptions": options, "include": include })
        };
        let dir = if p.manifest {
            p.work_dir()
        } else {
            std::env::temp_dir().join(format!("og-check-{}", std::process::id()))
        };
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = dir.join("tsconfig.check.json");
        let text = format!(
            "// Written by og check (crates/og-cli/src/check.rs): the project's type-check\n// configuration with the engine package og resolved. Do not edit; it is rewritten.\n{}\n",
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
            "tool": self.tool,
            "version": self.version,
            "config": self.config,
            "ms": self.ms,
            "diagnostics": self.diagnostics.iter().map(|d| serde_json::json!({
                "file": d.file, "line": d.line, "col": d.col, "code": d.code, "message": d.message,
            })).collect::<Vec<_>>(),
        })
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

pub fn run(args: &[&str]) -> ExitCode {
    let mut path = ".".to_string();
    let mut json = false;
    let mut checkers = None;
    let mut tool = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => json = true,
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
        match Target::of(&p, package.as_deref()) {
            Ok(t) => (t, package),
            Err(e) => return fail(e),
        }
    };
    let (checker, note) = match Checker::find(package.as_deref(), tool) {
        Ok(c) => c,
        Err(e) => return fail(e),
    };
    if let Some(n) = note {
        eprintln!("og check: {n}");
    }
    let report = match checker.check(&target, checkers.unwrap_or_else(default_checkers)) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    if json {
        println!("{}", report.json());
    } else {
        print!("{}", report.raw);
        println!("{}", report.summary());
    }
    if report.ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(file: &str, line: u32, col: u32, code: u32, message: &str) -> Diagnostic {
        Diagnostic {
            file: Some(file.into()),
            line: Some(line),
            col: Some(col),
            code,
            message: message.into(),
        }
    }

    // Every input below is tsrs 0.18.0's output, copied (the Windows path
    // excepted: no tsrs for Windows, so the line is a Linux one with its
    // path written the way tsgo prints one there).

    #[test]
    fn a_located_diagnostic() {
        assert_eq!(
            parse(
                "templates/game/src/main.ts(21,98): error TS2307: Cannot find module '@origingame/engine' or its corresponding type declarations.\n"
            ),
            vec![at(
                "templates/game/src/main.ts",
                21,
                98,
                2307,
                "Cannot find module '@origingame/engine' or its corresponding type declarations."
            )]
        );
    }

    #[test]
    fn a_diagnostic_without_a_location() {
        let d = parse(
            "error TS18003: No inputs were found in config file '/tmp/tmp.aXrmlbkhaz/tsconfig.json'. Specified 'include' paths were '[\"zzz/*.ts\"]' and 'exclude' paths were '[]'.\n",
        );
        assert_eq!(d.len(), 1);
        assert_eq!(
            (d[0].file.as_deref(), d[0].line, d[0].col, d[0].code),
            (None, None, None, 18003)
        );
        assert!(d[0].message.starts_with("No inputs were found"));
    }

    #[test]
    fn continuation_lines_join_the_message_above() {
        let d = parse(concat!(
            "corpus/03-exact-optional.ts(3,25): error TS2379: Argument of type '{ name: string; commands: string; seed: undefined; check(run: PlaytestRun, t: PlaytestChecks): void; }' is not assignable to parameter of type 'PlaytestDesc' with 'exactOptionalPropertyTypes: true'. Consider adding 'undefined' to the types of the target's properties.\n",
            "  Types of property 'seed' are incompatible.\n",
            "    Type 'undefined' is not assignable to type 'number'.\n",
            "corpus/04-type-only-import.ts(2,10): error TS1484: 'World' is a type and must be imported using a type-only import when 'verbatimModuleSyntax' is enabled.\n",
        ));
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].code, 2379);
        assert!(
            d[0].message.ends_with(
                "properties.\nTypes of property 'seed' are incompatible.\n  Type 'undefined' is not assignable to type 'number'."
            ),
            "{:?}",
            d[0].message
        );
        assert_eq!((d[1].line, d[1].col, d[1].code), (Some(2), Some(10), 1484));
    }

    #[test]
    fn a_path_with_parentheses() {
        assert_eq!(
            parse(
                "sp (1)/b.ts(1,14): error TS2322: Type 'string' is not assignable to type 'number'.\n"
            ),
            vec![at(
                "sp (1)/b.ts",
                1,
                14,
                2322,
                "Type 'string' is not assignable to type 'number'."
            )]
        );
        assert_eq!(
            parse("C:\\Users\\me (work)\\game\\src\\main.ts(1,14): error TS2322: Type 'string' is not assignable to type 'number'.\n")[0]
                .file
                .as_deref(),
            Some("C:\\Users\\me (work)\\game\\src\\main.ts")
        );
    }

    #[test]
    fn a_message_holding_the_diagnostic_syntax() {
        let d = parse(
            "src/a.ts(4,7): error TS2322: Type '\"b.ts(1,2): error TS1: x\"' is not assignable to type 'number'.\n",
        );
        assert_eq!(
            d,
            vec![at(
                "src/a.ts",
                4,
                7,
                2322,
                "Type '\"b.ts(1,2): error TS1: x\"' is not assignable to type 'number'."
            )]
        );
    }

    #[test]
    fn empty_output_is_no_diagnostics_and_status_0_is_clean() {
        assert!(parse("").is_empty());
        assert_eq!(outcome(Some(0), ""), Ok(vec![]));
    }

    #[test]
    fn a_configuration_error() {
        // tsrs exits 2 on a broken tsconfig.json (TypeScript's 3 is not used).
        let out = "tsconfig.json(1,33): error TS5024: Compiler option 'strict' requires a value of type boolean.\ntsconfig.json(1,34): error TS1328: Property value can only be string literal, numeric literal, 'true', 'false', 'null', object literal or array literal.\n";
        let d = outcome(Some(2), out).unwrap();
        assert_eq!(d.iter().map(|d| d.code).collect::<Vec<_>>(), [5024, 1328]);
        let missing = outcome(
            Some(1),
            "error TS5058: The specified path does not exist: '/tmp/tmp.aXrmlbkhaz/nope.json'.\n",
        )
        .unwrap();
        assert_eq!((missing[0].file.as_deref(), missing[0].code), (None, 5058));
        assert_eq!(outcome(Some(3), out).unwrap().len(), 2);
    }

    #[test]
    fn a_failing_checker_is_never_clean() {
        assert!(
            outcome(Some(1), "").is_err(),
            "status 1 with nothing parsed"
        );
        assert!(outcome(Some(2), "thread 'main' panicked at src/checker.rs:1:1\n").is_err());
        assert!(outcome(Some(5), "").is_err(), "5: not implemented");
        assert!(outcome(Some(101), "").is_err(), "a Rust panic");
        assert!(outcome(None, "").is_err(), "a signal");
    }
}
