//! The `og` tool. See docs/design.md §2.1 and docs/phase-1.md.
//!
//! One binary for the author's loop: `og run` (native host, probe, command
//! channel), `og probe` (a WebSocket client of the probe), `og dev` (watch,
//! rebuild, hot reload), `og test` (bun tests and playtests), `og polish`
//! (playtests scored against og-taste and og-feel), `og build`
//! (the play page), `og cook`, `og doctor` and `og api` (the IDL outputs).

mod api;
mod bundle;
mod cook;
mod dev;
mod doctor;
mod polish;
mod probe;
mod project;
mod publish;
mod run;
mod sfx;
mod test;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "\
og: the Origin Engine tool

usage: og <command> [options]

commands:
  run [project|entry] [--headless] [--commands \"t:action[:args];...\"] [--seed S] [--port P]
      [--ticks N] [--out DIR] [--size WxH] [--setup action[:arg]]... [--audio <out.wav>]
      [--platform-store <file.json>] [--clock <UTC ms>] [--locale <tag>] [--json]
                      cook, bundle and run the game on the native host with the probe; outputs of
                      screenshot/dump_* land in --out (default <project>/og-out)
  probe <method> [params-json] [--port P] [--out file.png] [--raw]
                      one probe call to a running `og run --port P` / `og dev` (JSON-RPC over WebSocket)
  dev [project] [--port P] [--exit-after N] [--json]
                      watch src/ and assets/, rebuild and re-cook, hot-reload through the probe,
                      report each reload against the 300 ms budget
  test [project] [--json]
                      the project's bun tests, then each tests/*.og.ts playtest headless
  polish [project] [--out DIR] [--frames N] [--json]
                      play the playtests headless and score them against og-taste and og-feel
                      (contrast, text size, HUD, transitions, UI sounds, input latency, hit
                      feedback, idle frames, frame time, missing locale keys) into DIR/report.md
                      (default <project>/og-out/polish) with frames and a fix per finding
  build [project] [--publish] [--out DIR] [--json]
                      cook and export the game to dist/ (or publish.out): the player shell,
                      game.js, the web host, cooked/, og-build.json, _headers; --publish keeps
                      only the player-facing probe members and adds the PWA (sw.js, manifest)
  cook [project] [--threads N] [--report out.md] [--json]
                      assets/ into cooked/, incrementally
  sfx [<preset> [--seed N] [--pitch P] [--length L] [--volume V] --out <file.ogt|file.wav>]
  sfx library <dir>
                      the procedural sound presets (coin, jump, hit, click, ...): list them,
                      write one as an asset `og cook` synthesises (or render it to a WAV),
                      or write the whole library
  doctor              check bun, the engine package, the native addon, the wasm, a headless
                      renderer, and (in the engine repository) the build toolchain, naming each fix
  api gen [--wiki] [--root <dir>]
                      regenerate the IDL outputs (packages/engine/{types,schema}, the hosts' glue,
                      the knowledge wiki); --wiki writes only the wiki
  api check [--root <dir>]
                      exit 1 when a generated file differs from crates/og-api/api.ogt

options:
  --version           print the og version
";

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
        ["doctor", rest @ ..] => doctor::run(rest),
        ["run", rest @ ..] => run::run(rest),
        ["probe", rest @ ..] => probe::run(rest),
        ["dev", rest @ ..] => dev::run(rest),
        ["test", rest @ ..] => test::run(rest),
        ["polish", rest @ ..] => polish::run(rest),
        ["build", rest @ ..] => bundle::run(rest),
        ["cook", rest @ ..] => cook::run(rest),
        ["api", rest @ ..] => api::run(rest),
        ["sfx", rest @ ..] => sfx::run(rest),
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

pub(crate) fn find_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(og_idl::IDL_PATH).is_file())
        .map(Path::to_path_buf)
}
