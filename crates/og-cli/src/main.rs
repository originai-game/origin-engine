//! The `og` tool. See docs/design.md §2.1 and docs/phase-1.md.
//!
//! One binary for the author's loop: `og run` (native host, probe, command
//! channel; `--device` times the publish build in a real browser), `og probe` (a WebSocket client of the probe), `og dev` (watch,
//! rebuild, hot reload), `og check` (type-check with tsrs), `og test` (bun tests and playtests), `og polish`
//! (playtests scored against og-taste and og-feel), `og compare` (a model
//! review's silhouette IoU, ΔE00 and sheet), `og migrate three` (a three.js
//! file rewritten towards og, the rest listed), `og build`
//! (the play page), `og cook`, `og doctor` and `og api` (the IDL outputs).

mod api;
mod bake_lightmaps;
mod bake_probes;
mod bundle;
mod check;
mod compare;
mod contact_sheet;
mod cook;
mod dev;
mod device;
mod doctor;
mod doctor_project;
mod migrate;
mod polish;
mod probe;
mod project;
mod publish;
mod relay;
mod render;
mod replay;
mod run;
mod serve_relay;
mod sfx;
mod style;
mod test;
mod trace;
mod upgrade;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "\
og: the Origin Engine tool

usage: og <command> [options]

commands:
  run [project|entry] [--headless] [--commands \"t:action[:args];...\"] [--seed S] [--port P]
      [--ticks N] [--out DIR] [--size WxH] [--setup action[:arg]]... [--audio <out.wav>]
      [--platform-store <file.json>] [--clock <UTC ms>] [--locale <tag>] [--room <ws url>]
      [--json] [--no-check]
                      cook, type-check, bundle and run the game on the native host with the probe;
                      outputs of screenshot/dump_* land in --out (default <project>/og-out)
  run --device chrome|android[:serial]|iphone[:udid] [project] [--seconds N] [--port P] [--out DIR]
      [--size WxH] [--headed] [--webgl] [--chrome <path>] [--cert <pem> --key <pem>] [--host <ip>]
      [--json]
                      play the publish build in a real browser (served with COOP/COEP on 127.0.0.1,
                      or for the iPhone on 0.0.0.0 with the LAN URL printed; --host names the
                      address), pass the play gate and time the first frame and the frames;
                      report.md and device.json in --out (default <project>/og-out/device)
  replay record [project] --commands \"t:action[:args];...\" | --commands-file <f> [--ticks N] [--seed S]
      [--clock <UTC ms>] [--locale <tag>] [--keyframe-every K] --out <trace.json> [--json]
  replay compare <trace.json> [project] [--json]
                      record the world's portable hash after every tick of a scripted headless run
                      (and its save every K ticks, default 30), or replay a trace with this engine
                      and report the first divergent tick with the keyframe's diff; compare exits
                      0 the same, 3 diverged
  render [project] --commands \"t:action[:args];...\" | --timeline <name> [--fps F] [--from S] [--to S]
      [--size WxH] [--out DIR|out.mp4|out.webm] [--frames png|jpg] [--audio] [--contact-sheet COLS]
      [--jobs N] [--seed S] [--json] [--no-check]
                      render the script to frames, a video (ffmpeg: H.264 or VP9, the capture
                      mixer's audio with --audio), frames.json (tick, time, sha256 per frame) and a
                      contact sheet; the fps divides the tick rate; --jobs splits the range
  compare <reference.png> <render.png> [--mask <mask.png>] [--align] [--key <ΔE00>] [--sheet <out.png>]
      [--json]
                      a model review's numbers: silhouette IoU and ΔE00 per mask region (or over the
                      overlap) of a captured frame against a reference image, and one sheet PNG of
                      the reference beside the render beside their silhouettes (og-model skill)
  migrate three <file.ts> [--out <file.ts> | --write] [--json]
                      rewrite a three.js file towards og where the mapping is mechanical (geometries,
                      materials, meshes, lights, the scene graph, transforms, the camera, texture
                      loads) and list each line that is not, with the three-to-og guide's section
  probe <method> [params-json] [--port P] [--out file.png] [--raw]
                      one probe call to a running `og run --port P` / `og dev` (JSON-RPC over WebSocket)
  dev [project] [--port P] [--exit-after N] [--json]
                      watch src/ and assets/, rebuild and re-cook, hot-reload through the probe,
                      report each reload against the 300 ms budget and type-check each src/ change
                      beside the rebuild (reported, never refusing the reload)
  check [project|entry|tsconfig.json] [--json] [--checkers N] [--tool tsrs|tsgo]
                      type-check the game code with tsrs (tsgo where tsrs has no package);
                      --json prints the diagnostics as one `check` event; og run, og test and
                      og build check before bundling (--no-check skips it), og dev on each save
  test [project] [--filter <path>] [--json] [--no-check]
                      the project's bun tests, then each tests/*.og.ts playtest headless
                      --filter selects suite files by a project-relative path substring
  polish [project] [--out DIR] [--frames N] [--json]
                      play the playtests headless and score them against og-taste and og-feel
                      (contrast, text size, HUD, transitions, UI sounds, input latency, hit
                      feedback, idle frames, frame time, missing locale keys) into DIR/report.md
                      (default <project>/og-out/polish) with frames and a fix per finding
  build [project] [--publish] [--out DIR] [--json] [--no-check]
                      cook, type-check and export the game to dist/ (or publish.out): the player shell,
                      game.js, the web host, cooked/, og-build.json, _headers; --publish keeps
                      only the player-facing probe members and adds the PWA (sw.js, manifest)
  cook [project] [--threads N] [--report [out.md]] [--json] [--watch [--exit-after N]]
                      assets/ into cooked/, incrementally; --report prints the ten slowest
                      assets by their last cook; --watch cooks each change, timing it
  bake lightmaps [project] [--samples N] [--bounces N] [--texels-per-metre N] [--sky R,G,B] [--scene <id>]
                      trace the light of the scenes with static models or LightVolumes into
                      <scene>.lightmap.ogt and .ktx2 beside them (CPU, native), then cook
  sfx [<preset> [--seed N] [--pitch P] [--length L] [--volume V] --out <file.ogt|file.wav>]
  sfx library <dir>
                      the procedural sound presets (coin, jump, hit, click, ...): list them,
                      write one as an asset `og cook` synthesises (or render it to a WAV),
                      or write the whole library
  style pack <dir> [--out <dir>]
                      pack a UI style's art drawn by hand (<part>.9.png nine-patches, other PNGs)
                      into atlas.png and the style file's atlas, slice and ornament lines
  relay [--port P] [--host <ip>]
                      a local WebSocket relay for rooms: `og run --room ws://127.0.0.1:P/<room>`
                      processes join <room> and play together (og.net's lockstep)
  doctor [--json]     check bun, the engine package, the native addon, the wasm, a headless
                      renderer, and (in the engine repository) the build toolchain, naming each fix;
                      --json adds the pin: {pinned, installed, compatible} (same minor, installed >= pinned)
  upgrade [project] [--to <version>] [--dry-run] [--json]
                      move project.ogt's engine pin and package.json's dependency to the installed
                      engine (or --to), apply the changelog's codemods for the versions crossed and
                      list what needs a hand edit; exit 0 done, 3 hand edits listed, 1 failed
  api gen [--wiki] [--root <dir>]
                      regenerate the IDL outputs (packages/engine/{types,schema}, the hosts' glue,
                      the knowledge wiki); --wiki writes only the wiki
  api check [--root <dir>]
                      exit 1 when a generated file differs from crates/og-api/api.ogt
  bake probes [project] [--commands \"t:action[:args];...\"] [--at S] [--size WxH]
                      render the baked og.ReflectionProbe captures on the native host into
                      cooked/probes/ (loaded instead of drawn on first use)

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
        ["upgrade", rest @ ..] => upgrade::run(rest),
        ["run", rest @ ..] => run::run(rest),
        ["replay", rest @ ..] => replay::run(rest),
        ["render", rest @ ..] => render::run(rest),
        ["compare", rest @ ..] => compare::run(rest),
        ["migrate", rest @ ..] => migrate::run(rest),
        ["relay", rest @ ..] => serve_relay::run(rest),
        ["probe", rest @ ..] => probe::run(rest),
        ["dev", rest @ ..] => dev::run(rest),
        ["check", rest @ ..] => check::run(rest),
        ["test", rest @ ..] => test::run(rest),
        ["polish", rest @ ..] => polish::run(rest),
        ["build", rest @ ..] => bundle::run(rest),
        ["cook", rest @ ..] => cook::run(rest),
        ["bake", "probes", rest @ ..] => bake_probes::run(rest),
        ["bake", rest @ ..] => bake_lightmaps::run(rest),
        ["api", rest @ ..] => api::run(rest),
        ["sfx", rest @ ..] => sfx::run(rest),
        ["style", rest @ ..] => style::run(rest),
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
