//! The `og` tool. See docs/design.md §2.1.
//!
//! One binary for the author's loop: `og run` (native host, probe, command
//! channel; `--device` times the publish build in a real browser), `og probe` (a WebSocket client of the probe), `og dev` (watch,
//! rebuild, hot reload), `og check` (type-check with tsrs), `og test` (bun tests and playtests),
//! `og compare` (a model review's silhouette IoU, ΔE00 and sheet), `og
//! migrate three` (a three.js file rewritten towards og, the rest listed),
//! `og build` (the play page), `og cook`, `og doctor` and `og api` (the IDL outputs).

mod api;
mod api_lookup;
mod bake_lightmaps;
mod bake_probes;
mod bake_projection;
mod bundle;
mod check;
mod compare;
mod contact_sheet;
mod cook;
mod dev;
mod device;
mod doctor;
mod doctor_project;
mod frame;
mod i18n;
mod lockstep_lint;
mod migrate;
mod prefab;
mod probe;
mod project;
mod publish;
mod relay;
mod render;
mod replay;
mod run;
mod serve_relay;
mod session;
mod sfx;
mod style;
mod style_art;
mod style_check;
mod test;
mod trace;
mod ui;
mod upgrade;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "\
og: the Origin Engine tool

usage: og <command> [options]

commands:
  session [project]   keep a worktree's native host warm until idle or interrupted
  run [project|entry] [--headless] [--commands \"t:action[:args];...\"] [--seed S] [--port P]
      [--ticks N] [--out DIR] [--size WxH] [--setup action[:arg]]... [--audio <out.wav>]
      [--platform-store <file.json>] [--platform-fake <file.json>] [--clock <UTC ms>] [--locale <tag>] [--room <ws url>]
      [--json] [--no-check] [--require-checked]
                      cook, type-check, bundle and run the game on the native host with the probe;
                      outputs of screenshot/dump_* land in --out (default <project>/og-out)
                      --require-checked exits 1 when GPU checks remain unchecked (default: warning)
  run --device chrome|safari|android[:serial]|iphone[:udid] [project] [--seconds N] [--port P] [--out DIR]
      [--size WxH] [--headed] [--chrome <path>] [--cert <pem> --key <pem>] [--host <ip>]
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
  check [project|entry|tsconfig.json] [--json] [--checkers N] [--tool tsrs|tsgo] [--require-checked]
                      type-check the game code with tsrs (tsgo where tsrs has no package);
                      --json prints the diagnostics as one `check` event; og run, og test and
                      og build check before bundling (--no-check skips it), og dev on each save
  test [project] [--filter <path>] [--jobs N] [--json] [--no-check] [--require-checked]
                      the project's bun tests, then each tests/*.og.ts playtest headless
                      --filter selects suite files by a project-relative path substring;
                      --jobs N runs up to N playtests at once, each one's output whole, in order
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
  prefab check [project]
                      flatten every scene and prefab as the cook does: cycles, name clashes and
                      orphaned overrides (a node, component or field the prefab no longer has);
                      exits 1 while any exists (a publish cook refuses an orphaned override)
  prefab show <prefab> [--scene <id> --instance <name>] [--project <dir>]
                      the flattened tree, each field with the text its value comes from
  prefab extract <scene> <entity> <prefab> [--project <dir>]
                      an authored entity becomes a new prefab, its block an instance of it
  i18n extract [project] [--json] [--write] [--check]
                      every og.i18n key the dialogues and quests show (type dialogue, type
                      quest), with the arguments they pass and where, against each string
                      table: per locale, the keys it lacks and the arguments nothing passes;
                      --write adds the keys the base table lacks to it (assets/strings/en.ogt
                      when there is none); --check exits 1 while any table is incomplete
  sfx [<preset> [--seed N] [--pitch P] [--length L] [--volume V] --out <file.ogt|file.wav>]
  sfx library <dir>
                      the procedural sound presets (coin, jump, hit, click, ...): list them,
                      write one as an asset `og cook` synthesises (or render it to a WAV),
                      or write the whole library
  style list          the built-in UI styles' slugs, each with its signature
  style eject <slug> [<name>] [--project <dir>] [--force]
                      write the built-in style <slug> (style.ogt, atlas, icons, fonts and their
                      import settings, licences) into the game's assets/styles/<name>/ (default
                      <slug>) to change at will, refusing to overwrite without --force; the look
                      names it as ui \"styles/<name>/style.ogt\"
  style upgrade <style.ogt> [--out <file>] [--check]
                      write a version 1 UI style as version 2 (its values kept, its part lines as
                      dressings, the new tokens derived from its palette); --check only fails on v1
  style pack <dir> [--out <dir>]
                      pack a UI style's art drawn by hand (<part>.9.png nine-patches, other PNGs)
                      into atlas.png and the style file's atlas, slice and ornament lines
  style check [project] [--title <commands>] [--play <commands>] [--results <commands>] [--size WxH]
      [--no-frames] [--json]
                      cook, then measure every cooked asset against the art contract of the look
                      project.ogt names (palette hit rate, colours, pixel grid, texel density,
                      faces against the budget, flat normals, the line) and list every opt-out;
                      capture the title, play and results screens (project.ogt's screens block or
                      the flags) and measure their colours, palette and edges; exit 0 within the
                      game's own contract or with no look, 3 off it
  ui sheet <pack|kit> [project] [--out DIR] [--json]
                      render fixed baseline pages on the pack's own ground at desktop and phone,
                      100% and 150%; frames, UI dumps and sheets for review, never a gate step
  ui check [project] [--screen <name>[=<commands>]]... [--show <template>] [--styles a,b]
      [--style-cmd <template>] [--breakpoints desktop,phonePortrait,phoneLandscape,tablet]
      [--locales game,pseudo,<cjk>,<rtl>] [--scales 100,150] [--high-contrast] [--settle S]
      [--out DIR] [--json] [--require-checked]
                      lay each screen (its commands, or show:<name>; without --screen every screen
                      project.ogt's ui { screen \"<name>\" \"<commands>\" } and screens blocks declare,
                      else the values the game's show usage lists, else the playtests' screenshots)
                      out per style, breakpoint and
                      locale (the game's, a pseudo-locale 40 % longer, CJK, RTL) and report facts:
                      findings (undeclared overlap, cut text, the safe area and the shell's corner,
                      pad reach, names, feedback) with the paths, what was found and the prop, and
                      measures with no bar (contrast on pixels, text size, touch targets, text
                      scale, primary actions, element count) (DIR/check.json, check.txt; default
                      <project>/og-out/ui-check); exit 0 when it ran, 1 when it could not
  ui preview [project] [the same options] [--film <commands> --at t,t,...]
                      a contact sheet per screen and style (a row per breakpoint, a column per
                      locale) and index.json of every cell, or a filmstrip of a motion at the
                      chosen times after the screen settles (default <project>/og-out/ui-preview)
  ui eject <screen> [project] [--force] | ui eject --list [project]
                      copy one of the library's screen documents (the engine package's
                      src/ui/screens/, one for every screen type of docs/ui.md Appendix B) into
                      assets/ui/<screen>.ogt to edit as the game's own; refuses to overwrite one
                      there without --force; --list prints each with its Appendix B row
  frame measure <png> [--region x,y,w,h] [--against x,y,w,h] [--units px|fraction]
      [--metric contrast,highpass,mean,ramp] [--rows N] [--json] [--require-checked]
                      a reviewer's numbers off a captured frame, the same on every machine: WCAG
                      contrast (the ink, the region's darkest or brightest 5 %, against the median
                      ground of --against or the region), high-pass energy (mean |grey - its σ 2
                      blur|, with its ratio to --against), mean colour, and the grey per band of
                      --rows rows with the largest step (visual-acceptance §5, §6)
  relay [--port P] [--host <ip>]
                      a local WebSocket relay for rooms: `og run --room ws://127.0.0.1:P/<room>`
                      processes join <room> and play together (og.net's lockstep)
  doctor [--json] [--require-checked]
                      check bun, the engine package, the native addon, the wasm, a headless
                      renderer, and (in the engine repository) the build toolchain, naming each fix;
                      --json adds the pin: {pinned, installed, compatible} (same minor, installed >= pinned)
  upgrade [project] [--to <version>] [--dry-run] [--json]
                      move project.ogt's engine pin and package.json's dependency to the installed
                      engine (or --to), apply the changelog's codemods for the versions crossed and
                      list what needs a hand edit; exit 0 done, 3 hand edits listed, 1 failed
  api find <words> [--kind fn|record|type|member|const|value|column] [--limit N]
                      the API reference entries matching every word (name, module, first
                      sentence), best first, one line each
  api show <name>     one API entry in full: signature, fields, doc (og.CharacterController,
                      og.physics.raycast, BODY_STATIC; og. may be left out)
  api gen [--wiki] [--root <dir>]
                      regenerate the IDL outputs (packages/engine/{types,schema}, the hosts' glue,
                      the knowledge wiki); --wiki writes only the wiki
  api check [--root <dir>]
                      exit 1 when a generated file differs from crates/og-api/api/
  bake projection <init|plan|prepare|project|holes|export> --run <dir> [options]
                      texture a level from views an image model paints (CPU, native): the
                      caller paints each prepared view; `og bake projection` lists the steps
  bake probes [project] [--commands \"t:action[:args];...\"] [--at S] [--size WxH]
                      render the baked og.ReflectionProbe captures on the native host into
                      cooked/probes/ (loaded instead of drawn on first use)

options:
  --version           print the og version
";

/// `og <command> [<sub>] … --help`: that command's entries of [`USAGE`]
/// (narrowed to `<sub>`'s when it names one), or `None` for a command
/// [`USAGE`] does not list. Every command answers its help from this one
/// text, on stdout with success, as `og --help` does.
fn command_help(args: &[&str]) -> Option<String> {
    let (cmd, rest) = args.split_first()?;
    // The entries: a line indented two spaces starts one, deeper lines go on.
    let mut entries: Vec<Vec<&str>> = Vec::new();
    let mut open = false;
    for line in USAGE.lines() {
        let entry = line.starts_with("  ") && !line.starts_with("   ");
        if entry {
            entries.push(vec![line]);
            open = true;
        } else if open && line.starts_with("   ") {
            entries.last_mut().expect("opened").push(line);
        } else {
            open = false;
        }
    }
    let word = |e: &Vec<&str>, i: usize| e[0].split_whitespace().nth(i).map(str::to_string);
    let mut mine: Vec<&Vec<&str>> = entries
        .iter()
        .filter(|e| word(e, 0).as_deref() == Some(cmd))
        .collect();
    if let Some(sub) = rest.first().filter(|a| !a.starts_with('-'))
        && mine.iter().any(|e| word(e, 1).as_deref() == Some(sub))
    {
        mine.retain(|e| word(e, 1).as_deref() == Some(sub));
    }
    (!mine.is_empty()).then(|| {
        let mut out = String::from("usage: og <command> [options]\n\n");
        for e in mine {
            for line in e {
                out += line;
                out.push('\n');
            }
        }
        out
    })
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if args.len() > 1
        && args[1..].iter().any(|a| *a == "--help" || *a == "-h")
        && let Some(help) = command_help(&args)
    {
        print!("{help}");
        return ExitCode::SUCCESS;
    }
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
        ["session", rest @ ..] => session::run(rest),
        ["replay", rest @ ..] => replay::run(rest),
        ["render", rest @ ..] => render::run(rest),
        ["compare", rest @ ..] => compare::run(rest),
        ["migrate", rest @ ..] => migrate::run(rest),
        ["relay", rest @ ..] => serve_relay::run(rest),
        ["probe", rest @ ..] => probe::run(rest),
        ["dev", rest @ ..] => dev::run(rest),
        ["check", rest @ ..] => check::run(rest),
        ["test", rest @ ..] => test::run(rest),
        ["build", rest @ ..] => bundle::run(rest),
        ["cook", rest @ ..] => cook::run(rest),
        ["bake", "projection", rest @ ..] => bake_projection::run(rest),
        ["bake", "probes", rest @ ..] => bake_probes::run(rest),
        ["bake", rest @ ..] => bake_lightmaps::run(rest),
        ["api", rest @ ..] => api::run(rest),
        ["prefab", rest @ ..] => prefab::run(rest),
        ["sfx", rest @ ..] => sfx::run(rest),
        ["i18n", rest @ ..] => i18n::run(rest),
        ["style", rest @ ..] => style::run(rest),
        ["ui", rest @ ..] => ui::run(rest),
        ["frame", rest @ ..] => frame::run(rest),
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
        .find(|dir| dir.join(og_idl::IDL_PATH).is_dir())
        .map(Path::to_path_buf)
}
