//! TypeScript checker discovery (`tsrs` / `tsgo`), command invocation,
//! incremental lock handling, and diagnostic parsing.

use super::*;

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
pub(super) fn tsgo_panic(stderr: &str) -> Option<(PathBuf, String)> {
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
