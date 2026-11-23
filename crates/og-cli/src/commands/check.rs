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

const USAGE: &str =
    "usage: og check [project|entry|tsconfig.json] [--json] [--checkers N] [--tool tsrs|tsgo]";

/// The lib files of tsgo's platform package at TSGO_VERSION, beside
/// `lib.d.ts` (`lib.<name>.d.ts`). Its package.json lists only `lib`, so
/// the names are pinned here with the version; a missing default lib is no
/// error to tsgo, it reads as `Cannot find global type 'Array'` in the game.
const TSGO_LIBS: &str = "decorators decorators.legacy dom.asynciterable dom dom.iterable \
    es2015.collection es2015.core es2015 es2015.generator es2015.iterable es2015.promise \
    es2015.proxy es2015.reflect es2015.symbol es2015.symbol.wellknown es2016.array.include \
    es2016 es2016.full es2016.intl es2017.arraybuffer es2017 es2017.date es2017.full \
    es2017.intl es2017.object es2017.sharedmemory es2017.string es2017.typedarrays \
    es2018.asyncgenerator es2018.asynciterable es2018 es2018.full es2018.intl es2018.promise \
    es2018.regexp es2019.array es2019 es2019.full es2019.intl es2019.object es2019.string \
    es2019.symbol es2020.bigint es2020 es2020.date es2020.full es2020.intl es2020.number \
    es2020.promise es2020.sharedmemory es2020.string es2020.symbol.wellknown es2021 \
    es2021.full es2021.intl es2021.promise es2021.string es2021.weakref es2022.array es2022 \
    es2022.error es2022.full es2022.intl es2022.object es2022.regexp es2022.string \
    es2023.array es2023.collection es2023 es2023.full es2023.intl es2024.arraybuffer \
    es2024.collection es2024 es2024.full es2024.object es2024.promise es2024.regexp \
    es2024.sharedmemory es2024.string es2025.collection es2025 es2025.float16 es2025.full \
    es2025.intl es2025.iterator es2025.promise es2025.regexp es5 es6 esnext.array \
    esnext.collection esnext esnext.date esnext.decorators esnext.disposable esnext.error \
    esnext.full esnext.intl esnext.sharedmemory esnext.temporal esnext.typedarrays \
    scripthost webworker.asynciterable webworker webworker.importscripts webworker.iterable";

/// The directory where `bun x` installed `typescript@TSGO_VERSION`, if it
/// has (bun 1.4, `cache_dir` in src/runtime/cli/bunx_command.rs): on
/// Windows `%TEMP%\bunx-<user id>-typescript@V`.
#[cfg(windows)]
pub fn tsgo_install() -> Option<PathBuf> {
    let tmp = std::env::temp_dir();
    let suffix = format!("-typescript@{TSGO_VERSION}");
    std::fs::read_dir(&tmp)
        .ok()?
        .flatten()
        .map(|e| tmp.join(e.file_name()))
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("bunx-") && n.ends_with(&suffix))
                && p.is_dir()
        })
}

/// Elsewhere `<base>/.bunx-<uid>/typescript@V`, the base being the first
/// that holds a `.bunx-<uid>` of the install cache (BUN_INSTALL_CACHE_DIR,
/// `$BUN_INSTALL/install/cache`, `$XDG_CACHE_HOME` or `$HOME` +
/// `/.bun/install/cache`) and the temporary directory. A cache directory
/// set in bunfig.toml is not read: tsgo's own panic still names it.
#[cfg(unix)]
pub fn tsgo_install() -> Option<PathBuf> {
    let var = |k| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let cache = var("BUN_INSTALL_CACHE_DIR")
        .or_else(|| var("BUN_INSTALL").map(|d| d.join("install/cache")))
        .or_else(|| {
            var("XDG_CACHE_HOME")
                .or_else(|| var("HOME"))
                .map(|d| d.join(".bun/install/cache"))
        });
    // SAFETY: getuid has no preconditions and cannot fail.
    let root = format!(".bunx-{}", unsafe { libc::getuid() });
    cache
        .into_iter()
        .chain([std::env::temp_dir()])
        .map(|base| base.join(&root))
        .find(|r| r.is_dir())
        .map(|r| r.join(format!("typescript@{TSGO_VERSION}")))
        .filter(|t| t.is_dir())
}

/// The tsgo install's fault once no other `og` is installing it: `bun x`
/// writes the tree file by file, so an `og check` beside another one's
/// first run (the tests, an agent checking while it builds, on a cold
/// cache) saw a half-written tree and called it broken (studio-win,
/// 2026-11-05: 5 og-cli tests at once). A tree that is missing or looks
/// broken is looked at again under a lock in the temporary directory, the
/// install made there by whichever `og` holds it first (`bun x … --version`,
/// its output dropped, when `install`; `og doctor` only waits); a tree
/// still broken after that is broken.
pub(crate) fn settled_tsgo_fault(install: bool) -> Option<(PathBuf, String)> {
    let fault = || tsgo_install().and_then(|t| tsgo_install_fault(&t).map(|f| (t, f)));
    if tsgo_install().is_some() && fault().is_none() {
        return None;
    }
    let lock =
        std::fs::File::create(std::env::temp_dir().join(format!("og-tsgo-{TSGO_VERSION}.lock")));
    let _held = lock.as_ref().ok().filter(|f| f.lock().is_ok());
    if install && tsgo_install().is_none() {
        let _ = Command::new("bun")
            .args([
                "x",
                "--package",
                &format!("typescript@{TSGO_VERSION}"),
                "tsc",
                "--version",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    fault()
}

/// What is wrong with the tsgo install in `tree`: the files it needs that
/// are missing or empty, `None` when there are none.
pub fn tsgo_install_fault(tree: &Path) -> Option<String> {
    let os = match std::env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        os => os,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        arch => arch,
    };
    let modules = tree.join("node_modules");
    let lib = modules.join(format!("@typescript/typescript-{os}-{arch}/lib"));
    let exe = if cfg!(windows) { "tsc.exe" } else { "tsc" };
    let needed: Vec<PathBuf> = [
        modules.join("typescript/package.json"),
        modules.join("typescript/lib/tsc.js"),
        lib.join(exe),
        lib.join("lib.d.ts"),
    ]
    .into_iter()
    .chain(
        TSGO_LIBS
            .split_whitespace()
            .map(|n| lib.join(format!("lib.{n}.d.ts"))),
    )
    .collect();
    let bad: Vec<String> = needed
        .iter()
        .filter(|p| std::fs::metadata(p).map_or(true, |m| m.len() == 0))
        .map(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    if bad.is_empty() {
        return None;
    }
    let shown = bad.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
    let more = match bad.len() {
        n if n > 3 => format!(" and {} more", n - 3),
        _ => String::new(),
    };
    Some(format!(
        "{} of its {} files missing or empty ({shown}{more})",
        bad.len(),
        needed.len()
    ))
}

/// The error of a broken tsgo install, with the fix.
pub fn broken_tsgo(tree: &Path, fault: &str) -> String {
    format!(
        "the TypeScript checker (tsgo {TSGO_VERSION}) that bun x installed in {} is incomplete, as an interrupted install leaves it: {fault}. Delete that directory, or clear bun's cache with `bun pm cache rm`, then rerun",
        tree.display()
    )
}

/// The install a tsgo panic on stderr blames: `panic: bundled:
/// <tree>/node_modules/@typescript/typescript-<platform>/lib/lib.d.ts does
/// not exist; this executable may be misplaced`, then a Go stack trace.
/// The tree and the panic's own words.
fn tsgo_panic(stderr: &str) -> Option<(PathBuf, String)> {
    let line = stderr.lines().find(|l| l.starts_with("panic: "))?;
    let missing = line
        .strip_prefix("panic: bundled: ")?
        .split(" does not exist")
        .next()?;
    let tree = Path::new(missing)
        .ancestors()
        .find(|a| a.file_name().is_some_and(|n| n == "node_modules"))?
        .parent()?;
    Some((
        tree.to_path_buf(),
        format!("tsgo stopped: {missing} does not exist"),
    ))
}

pub const NO_TSRS: &str = "no tsrs: run og through its launcher (the engine's dist carries node_modules/@tsrs/<platform>), run scripts/install-tools.sh in an engine checkout, or set OG_TSRS to the binary";

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
    if platform().is_none() && std::env::var_os("OG_TSRS").is_none() {
        return Ok(None);
    }
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
            "{} has no lib.es2022.d.ts beside it in {}: tsrs reads its lib files from its own directory; reinstall the whole package (scripts/install-tools.sh, or the engine's dist: og engine install)",
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
        // tsgo, unless the install bun x would run it from is broken.
        let tsgo = |note: Option<String>| {
            if let Some((tree, fault)) = settled_tsgo_fault(true) {
                return Err(broken_tsgo(&tree, &fault));
            }
            let checker = Checker {
                tool: Tool::Tsgo,
                path: None,
                version: TSGO_VERSION.into(),
            };
            Ok((checker, note))
        };
        match tool {
            Some(Tool::Tsgo) => tsgo(None),
            None if platform().is_none() && std::env::var_os("OG_TSRS").is_none() => {
                tsgo(Some(format!(
                    "no tsrs package for this platform; checking with tsgo {TSGO_VERSION} through bun x (needs the network or bun's cache)"
                )))
            }
            Some(Tool::Tsrs) => match locate_tsrs(package)? {
                Some(t) => Ok((Checker::from(t), None)),
                None => Err(NO_TSRS.into()),
            },
            // Only a missing tsrs falls back; a broken one is an error.
            None => match locate_tsrs(package)? {
                Some(t) => Ok((Checker::from(t), None)),
                None => tsgo(Some(format!(
                    "{NO_TSRS}; checking with tsgo {TSGO_VERSION} through bun x (needs the network or bun's cache)"
                ))),
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

    /// The check of `target` with `checkers` threads, to spawn (stdout
    /// piped; stderr passed through, tsgo's read for its panic first).
    pub fn command_for(
        &self,
        target: &Target,
        checkers: usize,
        build_info: Option<&Path>,
    ) -> Command {
        let mut c = self.command();
        c.arg("-p")
            .arg(typescript_path(&target.config))
            .args([
                "--noEmit",
                "--incremental",
                if build_info.is_some() {
                    "true"
                } else {
                    "false"
                },
                "--pretty",
                "false",
                "--checkers",
            ])
            .arg(checkers.max(1).to_string())
            .current_dir(&target.root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(match self.tool {
                Tool::Tsrs => Stdio::inherit(),
                Tool::Tsgo => Stdio::piped(),
            });
        if let Some(path) = build_info {
            c.arg("--tsBuildInfoFile").arg(path);
        }
        c
    }

    /// Why the checker could not start.
    pub fn spawn_error(&self, e: std::io::Error) -> String {
        match self.tool {
            Tool::Tsrs => format!("cannot start tsrs ({e})"),
            Tool::Tsgo => format!("{} ({e})", project::NO_BUN),
        }
    }

    /// The report of a check that ended with `out` after `ms`.
    pub fn report(
        &self,
        target: &Target,
        out: std::process::Output,
        ms: f64,
    ) -> Result<Report, String> {
        // A tsgo whose install lost its lib files panics with a Go stack
        // trace: the install and the fix instead.
        let stderr = String::from_utf8_lossy(&out.stderr);
        if let Some((tree, words)) = tsgo_panic(&stderr) {
            let fault = tsgo_install_fault(&tree).unwrap_or(words);
            return Err(broken_tsgo(&tree, &fault));
        }
        eprint!("{stderr}");
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

    /// Checks `target` with `checkers` threads.
    pub fn check(&self, target: &Target, checkers: usize) -> Result<Report, String> {
        let t0 = Instant::now();
        let incremental = self.incremental(target);
        let mut command = self.command_for(
            target,
            checkers,
            incremental.as_ref().map(|(_, path)| path.as_path()),
        );
        if let Some((lock, _)) = &incremental {
            // A killed CLI cannot unlock state while its checker still writes.
            command.stdin(Stdio::from(lock.try_clone().map_err(|e| e.to_string())?));
        }
        let out = command.output().map_err(|e| self.spawn_error(e))?;
        self.report(target, out, t0.elapsed().as_secs_f64() * 1000.0)
    }

    /// The compiler owns dependency discovery and invalidation, including
    /// missing imports, extended configs and declarations outside the project.
    /// A cache is optional: read-only and bare temporary targets still check.
    fn incremental(&self, target: &Target) -> Option<(std::fs::File, PathBuf)> {
        use std::hash::{Hash, Hasher};
        if std::env::var("OG_SESSION").as_deref() == Ok("0")
            || !target.config.starts_with(&target.root)
        {
            return None;
        }
        let mut hash = std::hash::DefaultHasher::new();
        target.config.hash(&mut hash);
        let directory = target
            .root
            .join(".og/check")
            .join(format!("{:016x}", hash.finish()));
        std::fs::create_dir_all(&directory).ok()?;
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("state.lock"))
            .ok()?;
        lock.lock().ok()?;
        let binary = self.path.as_ref().and_then(|path| {
            let path = path.canonicalize().ok()?;
            let metadata = std::fs::metadata(&path).ok()?;
            Some((path, metadata.len(), metadata.modified().ok()?))
        });
        let identity = format!("{} {} {binary:?}", self.name(), self.version);
        let info = directory.join("state.tsbuildinfo");
        let identity_path = directory.join("identity");
        if std::fs::read_to_string(&identity_path).ok().as_deref() != Some(&identity) {
            if info.exists() {
                std::fs::remove_file(&info).ok()?;
            }
            std::fs::write(identity_path, identity).ok()?;
        }
        Some((lock, info))
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
    shader_capabilities_lint(p, json, true);
    Ok(report)
}

fn locale_fonts_lint(p: &Project, json: bool, to_stderr: bool) {
    let Some(doc) = std::fs::read_to_string(p.root.join("project.ogt"))
        .ok()
        .and_then(|text| og_core::ogt::parse(&text).ok())
    else {
        return;
    };
    let Some(lang) = doc
        .body
        .get("create")
        .and_then(|e| e.block.as_ref())
        .and_then(|b| b.get("lang"))
    else {
        return;
    };
    let mut missing: Vec<_> = lang
        .values
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|tag| {
            // Scripts a style's face does not draw: each needs a pack or
            // a table's own fonts.
            matches!(
                og_i18n::locale::language(tag).as_str(),
                "zh" | "ja"
                    | "ko"
                    | "ar"
                    | "fa"
                    | "ur"
                    | "he"
                    | "yi"
                    | "hi"
                    | "mr"
                    | "ne"
                    | "th"
                    | "ru"
                    | "uk"
                    | "bg"
                    | "sr"
                    | "el"
            )
        })
        .collect();
    if missing.is_empty() {
        return;
    }
    let assets = p.root.join("assets");
    let mut available = Vec::new();
    let mut dirs = vec![assets.clone()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(ty) = entry.file_type() else { continue };
            let path = entry.path();
            if ty.is_dir() {
                dirs.push(path);
            } else if ty.is_file()
                && path.file_name().is_some_and(|n| n == "extension.json")
                && let Some(m) = std::fs::read(&path)
                    .ok()
                    .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
                    .filter(|m| m["kind"] == "font")
            {
                // An installed font pack (docs/ui.md §3.4): og.i18n chains it.
                let dir = path.parent().unwrap_or(&assets);
                let faces = m["faces"].as_array().cloned().unwrap_or_default();
                if !faces.is_empty()
                    && faces
                        .iter()
                        .all(|f| f.as_str().is_some_and(|f| dir.join(f).is_file()))
                {
                    available.extend(
                        m["languages"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(|l| l.as_str().map(og_i18n::locale::canonical)),
                    );
                }
            } else if ty.is_file()
                && path.extension().is_some_and(|e| e == "ogt")
                && let Some(table) = std::fs::read_to_string(path)
                    .ok()
                    .and_then(|text| og_i18n::StringTable::parse(&text).ok())
                    .filter(|t| {
                        !t.fonts.is_empty() && t.fonts.iter().all(|f| assets.join(f).is_file())
                    })
            {
                available.push(table.locale);
            }
        }
    }
    missing.retain(|tag| og_i18n::negotiate(&[tag], &available).is_none());
    if missing.is_empty() {
        return;
    }
    let message = format!(
        "{}: no font pack and no locale table with a font chain is installed for it. Install the pack with `og font add <pack>` (`og font list` names them; og.i18n chains an installed pack with no code), or declare a table's own fonts in its fonts line. Verify rendered glyphs.",
        missing.join(", ")
    );
    let text = if json {
        serde_json::json!({"event": "locale-fonts", "locales": missing, "message": message})
            .to_string()
    } else {
        format!("project.ogt: warning: {message}")
    };
    if to_stderr && !json {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

/// The determinism lint (`lockstep_lint`) over a project's sources, beside
/// the entry, for every game: printed as warnings
/// (`<file>(<line>,<col>): warning OG<code>:`) or, with `json`, as one
/// `lockstep-lint` event (the name it had when only lockstep games were
/// linted; `lockstep` says whether this one is). Never fails a check.
fn lockstep_lint(p: &Project, json: bool, to_stderr: bool) {
    let dir = p.entry.parent().unwrap_or(&p.root);
    let (lockstep, found) = crate::lockstep_lint::lint(&p.root, dir);
    if found.is_empty() {
        return;
    }
    let text = if json {
        serde_json::json!({
            "event": "lockstep-lint",
            "lockstep": lockstep,
            "warnings": found.iter().map(|(file, f)| serde_json::json!({
                "file": file, "line": f.line, "col": f.col, "code": f.code, "message": f.message,
            })).collect::<Vec<_>>(),
        })
        .to_string()
    } else {
        let mut t: Vec<String> = found
            .iter()
            .map(|(file, f)| {
                format!(
                    "{file}({},{}): warning OG{}: {}",
                    f.line, f.col, f.code, f.message
                )
            })
            .collect();
        t.push(if lockstep {
            format!(
                "og check: {} determinism warning(s): a peer that runs these parts from the room, and a replay or a save differs on the other host (`og api show determinism`, §1 and §3); `// og-ok` on a line that only draws",
                found.len()
            )
        } else {
            format!(
                "og check: {} determinism warning(s): where a system that changes the world runs these, a replay, a restored save or a playtest's end hash differs (`og api show determinism`, §1); `// og-ok` on a line that only draws",
                found.len()
            )
        });
        t.join("\n")
    };
    if to_stderr && !json {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

/// The grounds and inks the TSX under `dirs` writes by hand
/// (`ui_grounds`), against the default pack under the game's own: printed
/// as warnings (`<file>(<line>,<col>): warning OG9108:`) or, with `json`,
/// as one `ui-grounds` event. Never fails a check.
fn ui_grounds_lint(root: &Path, dirs: &[&Path], game: Option<&Path>, json: bool) {
    use og_ui_next::pack::{Layer, Pack, Scheme};
    let mut layers = vec![Layer::default_pack()];
    if let Some(layer) = game
        .and_then(|g| std::fs::read_to_string(g).ok())
        .and_then(|t| Layer::parse(&t).ok())
    {
        layers.push(layer);
    }
    // The game's theme recipe (`styles/game/theme.json`): its layer, and its
    // art slots as the grounds a child's ink is checked on (OG9107). One
    // that does not compile is the cook's error.
    let theme = game
        .map(|g| g.with_file_name(og_cook::cook::ui_themes::RECIPE))
        .and_then(|r| std::fs::read_to_string(&r).ok())
        .and_then(|t| og_cook::cook::ui_themes::compile(&t, "styles/game/theme.json").ok());
    if let Some(layer) = theme
        .as_ref()
        .and_then(|t| t.layer(Default::default()).ok())
    {
        layers.push(layer);
    }
    let art = theme
        .as_ref()
        .and_then(|t| crate::ui_grounds::ArtGrounds::of(t).ok());
    let pack = Pack::new(&layers);
    let colors = crate::ui_grounds::colors(&Scheme::ALL.map(|s| pack.theme_json(s)));
    // The pack's own scheme first: a colour written by hand is a role's
    // value in the scheme the UI was drawn in.
    let schemes = match pack.scheme {
        Some(Scheme::Light) => [Scheme::Light, Scheme::Dark],
        _ => [Scheme::Dark, Scheme::Light],
    };
    let scale = crate::ui_values::Scale::new(&schemes.map(|s| pack.theme_json(s)));
    let found = crate::ui_grounds::lint(root, dirs, &colors, &scale, art.as_ref());
    if found.is_empty() {
        return;
    }
    if json {
        let event = serde_json::json!({
            "event": "ui-grounds",
            "warnings": found.iter().map(|(file, f)| serde_json::json!({
                "file": file, "line": f.line, "col": f.col, "code": f.code, "class": f.class, "message": f.message,
                "fix": f.fix.as_ref().map(|(classes, _)| classes), "exact": f.fix.as_ref().map(|(_, exact)| exact),
            })).collect::<Vec<_>>(),
        });
        println!("{event}");
        return;
    }
    for (file, f) in &found {
        println!(
            "{file}({},{}): warning OG{}: {}",
            f.line, f.col, f.code, f.message
        );
    }
    let count = |code: u32| {
        let of: Vec<&str> = found
            .iter()
            .filter(|(_, f)| f.code == code)
            .map(|(file, _)| file.as_str())
            .collect();
        let mut files = of.clone();
        files.dedup();
        (of.len(), files.len())
    };
    let (grounds, in_files) = count(crate::ui_grounds::CODE);
    if grounds > 0 {
        println!(
            "og check: {grounds} ground or ink warning(s) in {in_files} file(s): text sits on a pack slot or an opaque theme ground, in that ground's ink (crates/og-ui-next/CONVENTIONS.md §Theme and packs, Grounds and inks)"
        );
    }
    let (values, in_files) = count(crate::ui_values::CODE);
    if values > 0 {
        println!(
            "og check: {values} hand-written value warning(s) in {in_files} file(s): a size, a colour, a shadow or a strength is a theme step or role, never a bracket, a literal or an opacity modifier (crates/og-ui-next/CONVENTIONS.md §Theme and packs, Steps and roles)"
        );
    }
}

/// `og check`'s code for a material shader whose surface does not take
/// some of what a scene can ask of it (`shader_capabilities_lint`).
pub const SHADER_CAPABILITY_CODE: u32 = 9103;

/// Every material shader under `assets/` (`.ogt` of `type shader`) whose
/// mesh surface does not take a capability a game sees, read from its lit
/// module as the renderer builds it (`og_render_data::reach::material_shader_lacks`,
/// the check of og-render's capability table): printed as warnings, or
/// with `json` as one `shader-capabilities` event. Never fails a check; a
/// shader the cook would refuse is the cook's to report.
fn shader_capabilities_lint(p: &Project, json: bool, to_stderr: bool) {
    let assets = p.root.join("assets");
    let mut found = Vec::new();
    let mut dirs = vec![assets.clone()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let Ok(ty) = entry.file_type() else { continue };
            let path = entry.path();
            if ty.is_dir() {
                if !path.file_name().is_some_and(|n| {
                    n == ".og" || n == "cooked" || n == "og-out" || n == "node_modules"
                }) {
                    dirs.push(path);
                }
                continue;
            }
            if !ty.is_file() || path.extension().is_none_or(|e| e != "ogt") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if og_core::ogt::parse(&text)
                .ok()
                .is_none_or(|d| d.ty != og_cook::shader::SHADER_TYPE)
            {
                continue;
            }
            let file = path
                .strip_prefix(&p.root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            let mut read = |rel: &str| {
                if rel.split('/').any(|c| c == ".." || c.is_empty()) {
                    return Err(format!("'{rel}' is not a path under assets/"));
                }
                std::fs::read_to_string(assets.join(rel)).map_err(|e| e.to_string())
            };
            let Ok(shader) = og_cook::shader::parse_shader(&name, &text, &mut read) else {
                continue;
            };
            let Ok(lacks) = og_render_data::reach::material_shader_lacks(&shader) else {
                continue;
            };
            if !lacks.is_empty() {
                found.push((file, shader.mode, lacks));
            }
        }
    }
    if found.is_empty() {
        return;
    }
    let mode = |m: og_render_data::ShaderMode| match m {
        og_render_data::ShaderMode::Hooks => "hooks",
        og_render_data::ShaderMode::Full { lit: true } => "full lit",
        og_render_data::ShaderMode::Full { lit: false } => "full unlit",
        og_render_data::ShaderMode::Sprite => "sprite",
    };
    let message = |m: og_render_data::ShaderMode, lacks: &[og_render_data::Capability]| {
        let names: Vec<_> = lacks.iter().map(|c| c.name()).collect();
        format!(
            "a mesh drawn with this shader (mode {}) does not take: {}; in a scene that has them, they leave its surface as it is (`og api show render`, §6)",
            mode(m),
            names.join(", ")
        )
    };
    let text = if json {
        serde_json::json!({
            "event": "shader-capabilities",
            "warnings": found.iter().map(|(file, m, lacks)| serde_json::json!({
                "file": file,
                "code": SHADER_CAPABILITY_CODE,
                "mode": mode(*m),
                "lacks": lacks.iter().map(|c| c.name()).collect::<Vec<_>>(),
                "message": message(*m, lacks),
            })).collect::<Vec<_>>(),
        })
        .to_string()
    } else {
        found
            .iter()
            .map(|(file, m, lacks)| {
                format!(
                    "{file}(1,1): warning OG{SHADER_CAPABILITY_CODE}: {}",
                    message(*m, lacks)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    if to_stderr && !json {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

/// The module-scope state check (`module_state`) over the game's sources
/// beside the entry: each finding an error of the report, as a type error
/// is, so `og check` fails and `og run`, `og test` and `og build` stop.
fn module_state(p: &Project, report: &mut Report) {
    let dir = p.entry.parent().unwrap_or(&p.root);
    for (file, f) in crate::module_state::check(&p.root, dir) {
        let message = f.message();
        report.raw.push_str(&format!(
            "{file}({},{}): error OG{}: {message}\n",
            f.line,
            f.col,
            crate::module_state::CODE
        ));
        report.diagnostics.push(Diagnostic {
            file: Some(file),
            line: Some(f.line),
            col: Some(f.col),
            code: crate::module_state::CODE,
            message,
        });
    }
}

fn class_resolver(pack: &og_ui_next::pack::Pack) -> Result<og_ui_next::classes::Classes, String> {
    use og_ui_next::{
        classes::{Classes, nine_slice},
        pack::Scheme,
    };
    let (mut classes, _) = Classes::new(Some(&pack.theme_json(Scheme::Dark)))?;
    classes.set_variants(pack.variants().clone());
    classes.set_images(
        [Scheme::Light, Scheme::Dark]
            .into_iter()
            .flat_map(|s| {
                pack.images(s)
                    .iter()
                    .map(|(n, i)| (n.clone(), nine_slice(i, 0)))
            })
            .collect(),
    );
    Ok(classes)
}

/// Check against the default pack under the game's own pack, using the
/// same class resolver as runtime rather than a parallel parser contract.
fn unknown_classes(p: &Project, report: &mut Report) {
    // The game's code and its UI components, where the template's
    // tsconfig reads TSX.
    let mut files = Vec::new();
    crate::lockstep_lint::sources(p.entry.parent().unwrap_or(&p.root), &mut files);
    crate::lockstep_lint::sources(&p.root.join("assets/ui"), &mut files);
    files.retain(|f| f.extension().is_some_and(|e| e == "tsx" || e == "jsx"));
    files.sort();
    files.dedup();
    if files.is_empty() {
        return;
    }
    use og_ui_next::pack::{Layer, Pack};
    let mut layers = vec![Layer::default_pack()];
    let game = p.root.join("assets/styles/game/pack.json");
    if let Ok(text) = std::fs::read_to_string(&game) {
        match Layer::parse(&text) {
            Ok(layer) => layers.push(layer),
            // The cook refuses the same file and says why.
            Err(_) => return,
        }
    }
    let pack = Pack::new(&layers);
    let Ok(mut classes) = class_resolver(&pack) else {
        return;
    };
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let file = typescript_path(path.strip_prefix(&p.root).unwrap_or(&path));
        for u in
            nsh_jsx::unknown_classes_with(&path.to_string_lossy(), &text, |s| classes.unknown(s))
        {
            let message = format!("unknown class `{}`", u.class);
            report.raw.push_str(&format!(
                "{file}({},{}): error OG{UNKNOWN_CLASS}: {message}\n",
                u.line, u.column
            ));
            report.diagnostics.push(Diagnostic {
                file: Some(file.clone()),
                line: Some(u.line as u32),
                col: Some(u.column as u32),
                code: UNKNOWN_CLASS,
                message,
            });
        }
    }
}

/// Each of the game's style packs (`assets/styles/<name>/pack.json`) whose
/// variants paint the ground under the text without its ink
/// (`og_ui_next::pack::grounds_without_ink`), laid over the default pack.
fn pack_grounds(p: &Project, report: &mut Report) {
    use og_ui_next::pack::{Layer, grounds_without_ink};
    let Ok(dirs) = std::fs::read_dir(p.root.join("assets/styles")) else {
        return;
    };
    let mut packs: Vec<_> = dirs
        .flatten()
        .map(|d| d.path().join("pack.json"))
        .filter(|f| f.is_file())
        .collect();
    packs.sort();
    for path in packs {
        // One that does not parse is the cook's error.
        let Some(layer) = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| Layer::parse(&t).ok())
        else {
            continue;
        };
        let file = typescript_path(path.strip_prefix(&p.root).unwrap_or(&path));
        for message in grounds_without_ink(&[Layer::default_pack()], &layer) {
            report
                .raw
                .push_str(&format!("{file}: error OG{PACK_GROUND_INK}: {message}\n"));
            report.diagnostics.push(Diagnostic {
                file: Some(file.clone()),
                line: None,
                col: None,
                code: PACK_GROUND_INK,
                message,
            });
        }
    }
}

/// The diagnostic code of a pack variant painting a ground without its ink.
const PACK_GROUND_INK: u32 = 9107;

/// The diagnostic code of an unknown class in a game's TSX.
const UNKNOWN_CLASS: u32 = 9105;

/// The diagnostic code of a string table a source imports that is not there.
const MISSING_TABLE: u32 = 9106;

/// The `.ogt` files a source imports for their effect (`import "./x.en.ogt"`),
/// each with its 1-based line and column.
fn ogt_imports(text: &str) -> Vec<(u32, u32, &str)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        // A statement, not an import a comment or a string quotes.
        let Some(stmt) = line.trim_start().strip_prefix("import") else {
            continue;
        };
        let rest = stmt.trim_start();
        let Some(quote) = rest.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            continue;
        };
        let body = &rest[1..];
        let Some(end) = body.find(quote) else {
            continue;
        };
        let spec = &body[..end];
        if spec.ends_with(".ogt") {
            let col = line.len() - rest.len() + 1;
            out.push((i as u32 + 1, col as u32 + 1, spec));
        }
    }
    out
}

/// Each `.ogt` a TS or TSX source under `dirs` imports beside itself that does
/// not exist: an error naming the importer. The cook fails the whole asset
/// that bundles it (every component's table with it), so a bad import stops
/// at its author's check instead.
fn missing_tables(root: &Path, dirs: &[PathBuf], report: &mut Report) {
    let mut files = Vec::new();
    for d in dirs {
        crate::lockstep_lint::sources(d, &mut files);
    }
    files.sort();
    files.dedup();
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let dir = path.parent().unwrap_or(root);
        for (line, col, spec) in ogt_imports(&text) {
            if !spec.starts_with('.') || dir.join(spec).is_file() {
                continue;
            }
            let file = typescript_path(path.strip_prefix(root).unwrap_or(&path));
            let message = format!(
                "`{spec}` is not beside this file: a component imports only a string table in its own directory"
            );
            report.raw.push_str(&format!(
                "{file}({line},{col}): error OG{MISSING_TABLE}: {message}\n"
            ));
            report.diagnostics.push(Diagnostic {
                file: Some(file),
                line: Some(line),
                col: Some(col),
                code: MISSING_TABLE,
                message,
            });
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_check_uses_the_runtime_contract_without_accepting_typos() {
        use og_ui_next::pack::{Layer, Pack};
        let pack = Pack::new(&[Layer::default_pack()]);
        let mut classes = class_resolver(&pack).unwrap();
        let found = nsh_jsx::unknown_classes_with(
            "a.tsx",
            r#"<View class="scheme-dark scheme-light aspect-[3.2] data-active:text-white aspect-3.2 scheme-dakr" />"#,
            |s| classes.unknown(s),
        );
        assert_eq!(
            found.iter().map(|f| f.class.as_str()).collect::<Vec<_>>(),
            ["aspect-3.2", "scheme-dakr"]
        );
    }

    /// The engine's own TSX components (packages/engine/src/ui) use only
    /// classes the default pack's theme knows, as a game's must
    /// ([`unknown_classes`]): one that styles nothing fails here, where
    /// no game's check reaches it.
    #[test]
    fn the_engines_components_use_only_known_classes() {
        use og_ui_next::pack::{Layer, Pack};
        let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
        let pack = Pack::new(&[Layer::default_pack()]);
        let mut classes = class_resolver(&pack).unwrap();
        let mut files = Vec::new();
        crate::lockstep_lint::sources(&ui, &mut files);
        files.retain(|f| f.extension().is_some_and(|e| e == "tsx"));
        assert!(files.len() > 20, "the components are there");
        let mut unknown = Vec::new();
        for path in files {
            let text = std::fs::read_to_string(&path).unwrap();
            for u in nsh_jsx::unknown_classes_with(&path.to_string_lossy(), &text, |s| {
                classes.unknown(s)
            }) {
                let file = path
                    .strip_prefix(&ui)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                unknown.push(format!("{file}:{}: {}", u.line, u.class));
            }
        }
        assert!(
            unknown.is_empty(),
            "unknown classes:\n{}",
            unknown.join("\n")
        );
    }

    /// No engine component draws its own border, ring or outline on an
    /// element a nine-sliced frame draws (`nine-*`): the frame is the
    /// element's edge, as CSS `border-image` is, so a rectangle round it
    /// is a second edge that misses the art's shape (a cut or rounded
    /// corner). Its room for content is the frame's own inset.
    #[test]
    fn no_engine_component_edges_a_framed_element_twice() {
        use og_ui_next::classes::Env;
        use og_ui_next::pack::{Layer, Pack};
        let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
        let pack = Pack::new(&[Layer::default_pack()]);
        let mut classes = class_resolver(&pack).unwrap();
        let env = Env {
            window: [390.0, 844.0],
            dark: true,
            rtl: false,
            touch: true,
            states: 0,
        };
        let mut files = Vec::new();
        crate::lockstep_lint::sources(&ui, &mut files);
        files.retain(|f| f.extension().is_some_and(|e| e == "tsx"));
        // The edges `s` draws besides its nine-* frame, if it draws one.
        let mut edges = |s: &str| {
            let r = classes.record(s, &env).skin;
            if r.image.slice == [0.0; 4] {
                return Vec::new();
            }
            let mut edges = Vec::new();
            if r.border.iter().any(|&w| w > 0.0) {
                edges.push("border");
            }
            let ring = r.shadows[..r.shadow_count as usize]
                .iter()
                .any(|sh| !sh.inset && sh.blur == 0.0 && sh.spread > 0.0);
            if ring {
                edges.push("ring");
            }
            if r.outline.width > 0.0 {
                edges.push("outline");
            }
            if edges.is_empty() {
                Vec::new()
            } else {
                vec![edges.join(" ")]
            }
        };
        let mut twice = Vec::new();
        // The pack slots a component puts on a framed element
        // (`packClass("Row", "frame", "selected")` beside `nine-*`).
        let mut framed_slots = std::collections::BTreeSet::new();
        for path in files {
            let text = std::fs::read_to_string(&path).unwrap();
            for line in text.lines().filter(|l| l.contains("nine-")) {
                for call in line.split("packClass(").skip(1) {
                    let args: Vec<_> = call
                        .split(')')
                        .next()
                        .unwrap_or("")
                        .split(',')
                        .map(|a| a.trim().trim_matches('"'))
                        .collect();
                    if args
                        .iter()
                        .all(|a| !a.is_empty() && a.chars().all(|c| c.is_ascii_alphanumeric()))
                    {
                        framed_slots.insert(args.join("."));
                    }
                }
            }
            for u in nsh_jsx::unknown_classes_with(&path.to_string_lossy(), &text, &mut edges) {
                let file = path
                    .strip_prefix(&ui)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                twice.push(format!("{file}:{}: {}", u.line, u.class));
            }
        }
        // A slot carries the look a theme gives it, so it is held to the
        // same rule over a frame: the slots components frame, and any
        // slot that draws a frame itself.
        assert!(
            framed_slots.contains("Row.frame.selected"),
            "slots are found"
        );
        for (key, value) in pack.variants() {
            let framed = framed_slots.contains(key) || value.contains("nine-");
            for e in edges(&format!("nine-plate-cut {value}")) {
                if framed {
                    twice.push(format!("pack slot {key}: {e}"));
                }
            }
        }
        assert!(
            twice.is_empty(),
            "a nine-* frame with its own edge as well ({}):\n{}",
            twice.len(),
            twice.join("\n")
        );
    }

    /// No engine component sets an ink on a theme's art that the art does
    /// not carry (OG9107 on a child): under a theme of every material, the
    /// text inside an art slot reads in the slot's solved ink, or in one
    /// that holds 4.5:1 on that art too.
    #[test]
    fn no_engine_component_inks_over_a_themes_art() {
        let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/engine/src/ui");
        let materials = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../templates/themes/dimensions/material");
        let mut low = std::collections::BTreeSet::new();
        let mut names: Vec<String> = std::fs::read_dir(&materials)
            .unwrap()
            .map(|e| {
                e.unwrap()
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        for m in names {
            for source in ["#d9a531", "#5cc8ff", "#ff5c9a"] {
                let recipe = format!(
                    r##"{{ "material": "{m}", "shape": "soft", "color": {{ "source": "{source}", "neutral": "#4c5643" }} }}"##
                );
                let theme = og_cook::cook::ui_themes::compile(&recipe, "test").unwrap();
                let art = crate::ui_grounds::ArtGrounds::of(&theme).unwrap();
                let colors = crate::ui_grounds::colors(
                    &art.colors
                        .clone()
                        .map(|c| serde_json::json!({ "colors": c }).to_string()),
                );
                let scale = crate::ui_values::Scale::new(&[]);
                for (file, f) in crate::ui_grounds::lint(&ui, &[&ui], &colors, &scale, Some(&art)) {
                    if f.code == crate::ui_grounds::ART_INK {
                        low.insert(format!(
                            "{file}:{}: {} ({m} {source}: {})",
                            f.line,
                            f.class,
                            f.message.split(" (under").next().unwrap_or("")
                        ));
                    }
                }
            }
        }
        // one line per class, its first material
        let mut seen = std::collections::BTreeSet::new();
        let low: Vec<_> = low
            .into_iter()
            .filter(|l| seen.insert(l.split(" (").next().unwrap().to_owned()))
            .collect();
        assert!(
            low.is_empty(),
            "{} inks on art:\n{}",
            low.len(),
            low.join("\n")
        );
    }

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
    fn an_ogt_import_names_its_table() {
        let text = "import { View } from \"@origingame/engine/ui\";\nimport \"./world.en.ogt\";\n  import './hud.ogt';\n// the `import \"./x.en.ogt\"` above\nconst s = \"import\";\n";
        let found: Vec<_> = ogt_imports(text)
            .into_iter()
            .map(|(l, _, s)| (l, s))
            .collect();
        assert_eq!(found, vec![(2, "./world.en.ogt"), (3, "./hud.ogt")]);
    }

    #[test]
    fn a_table_not_beside_its_importer_is_an_error() {
        let dir = std::env::temp_dir().join(format!("og-check-ogt-{}", std::process::id()));
        let hud = dir.join("hud");
        std::fs::create_dir_all(&hud).unwrap();
        std::fs::write(hud.join("bar.en.ogt"), "").unwrap();
        std::fs::write(hud.join("bar.tsx"), "import \"./bar.en.ogt\";\n").unwrap();
        std::fs::write(hud.join("damage.tsx"), "import \"./world.en.ogt\";\n").unwrap();
        let mut report = Report {
            tool: "tsrs",
            version: String::new(),
            config: String::new(),
            ms: 0.0,
            diagnostics: Vec::new(),
            raw: String::new(),
        };
        missing_tables(&dir, std::slice::from_ref(&dir), &mut report);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(report.diagnostics.len(), 1);
        let d = &report.diagnostics[0];
        assert_eq!(
            (d.file.as_deref(), d.line, d.code),
            (Some("hud/damage.tsx"), Some(1), MISSING_TABLE)
        );
        assert!(d.message.contains("./world.en.ogt"));
    }

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
