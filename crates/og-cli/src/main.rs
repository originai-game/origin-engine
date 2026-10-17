//! The `og` tool. See docs/design.md §2.1 and docs/phase-1.md.
//!
//! Phase 1 ships `og --version`, `og doctor`, `og api gen`, `og api check`
//! and `og run --headless` (M2: load, advance, hash). Every other planned
//! subcommand answers that it is not there yet and exits non-zero, so no
//! script mistakes it for success.

mod doctor;
mod run;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "\
og: the Origin Engine tool

usage: og <command> [options]

commands:
  doctor              check the toolchain against the repository's pins, naming each fix
  api gen             regenerate packages/engine/{types,schema} from crates/og-api/api.ogt
  api check           exit 1 when the generated files differ from crates/og-api/api.ogt
                      (the .d.ts, binding table, schema, host traits, both hosts' glue)
  run --headless <project> [--ticks N] [--seed S] [--setup action[:arg]]... [--json]
                      load the game on the native host, advance N ticks, print the world hash

options:
  --version           print the og version
  --root <dir>        for `api`: the repository root (default: found from the current directory)

planned: build (M2), run --commands (M5), cook (M4), probe (M5), test (M5)
";

/// Planned subcommands and the milestone of docs/phase-1.md that brings them.
const PLANNED: &[(&str, &str)] = &[
    ("build", "M2"),
    ("cook", "M4"),
    ("probe", "M5"),
    ("test", "M5"),
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["--version"] | ["-V"] | ["version"] => {
            println!("og {VERSION} (origin-engine {VERSION})");
            ExitCode::SUCCESS
        }
        ["--help"] | ["-h"] | ["help"] => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        ["doctor"] => doctor::run(),
        ["run", rest @ ..] => run::run(rest),
        ["api", "gen", rest @ ..] => match root_option(rest) {
            Ok(root) => api_gen(&root),
            Err(code) => code,
        },
        ["api", "check", rest @ ..] => match root_option(rest) {
            Ok(root) => api_check(&root),
            Err(code) => code,
        },
        [cmd, ..] if PLANNED.iter().any(|(c, _)| c == cmd) => {
            let milestone = PLANNED
                .iter()
                .find(|(c, _)| c == cmd)
                .map(|p| p.1)
                .unwrap_or("");
            eprintln!(
                "og {cmd} is not here yet; it lands with milestone {milestone} of docs/phase-1.md. Nothing was done."
            );
            ExitCode::from(2)
        }
        [] => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
        _ => {
            eprintln!(
                "og: unknown command `{}`; run `og --help` for the list.",
                args.join(" ")
            );
            ExitCode::from(2)
        }
    }
}

/// `--root <dir>`, or the nearest ancestor of the current directory that holds the IDL.
fn root_option(rest: &[&str]) -> Result<PathBuf, ExitCode> {
    match rest {
        [] => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            find_root(&cwd).ok_or_else(|| {
                eprintln!(
                    "og api: no {} above {}; run inside the engine repository or pass --root <dir>.",
                    og_api::IDL_PATH,
                    cwd.display()
                );
                ExitCode::from(2)
            })
        }
        ["--root", dir] => Ok(PathBuf::from(dir)),
        _ => {
            eprintln!(
                "og api: unexpected arguments `{}`; the only option is --root <dir>.",
                rest.join(" ")
            );
            Err(ExitCode::from(2))
        }
    }
}

pub(crate) fn find_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(og_api::IDL_PATH).is_file())
        .map(Path::to_path_buf)
}

fn api_gen(root: &Path) -> ExitCode {
    let api = match og_api::read(&root.join(og_api::IDL_PATH)) {
        Ok(api) => api,
        Err(e) => {
            eprint!("og api gen: {e}");
            return ExitCode::FAILURE;
        }
    };
    let files = og_api::generate(&api);
    if let Err(e) = og_api::write(root, &files) {
        eprintln!("og api gen: cannot write under {}: {e}", root.display());
        return ExitCode::FAILURE;
    }
    for f in &files {
        println!("wrote {}", f.path);
    }
    ExitCode::SUCCESS
}

fn api_check(root: &Path) -> ExitCode {
    match og_api::check(root) {
        Err(e) => {
            eprint!("og api check: {e}");
            ExitCode::FAILURE
        }
        Ok(drift) if drift.is_empty() => {
            println!(
                "og api check: the generated files match {}.",
                og_api::IDL_PATH
            );
            ExitCode::SUCCESS
        }
        Ok(drift) => {
            eprintln!(
                "og api check: {} file(s) differ from what {} generates:",
                drift.len(),
                og_api::IDL_PATH
            );
            for d in &drift {
                eprintln!("  {d}");
            }
            ExitCode::FAILURE
        }
    }
}
