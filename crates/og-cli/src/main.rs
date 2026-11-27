//! The `og` tool. See docs/design.md §2.1.
//!
//! One binary for the author's loop: `og run` (native host, probe, command
//! channel; `--device` times the publish build in a real browser), `og probe` (a WebSocket client of the probe), `og dev` (watch,
//! rebuild, hot reload), `og check` (type-check with tsrs), `og test` (bun tests and playtests),
//! `og compare` (a model review's silhouette IoU, ΔE00 and sheet, and FLIP), `og
//! migrate three` (a three.js file rewritten towards og, the rest listed),
//! `og build` (the play page), `og cook`, `og doctor` and `og api` (the IDL outputs).

mod commands;
use commands::{
    api, assets, bake_lightmaps, bake_probes, bake_projection, bundle, cache, check, compare, cook,
    cost, create, dev, doctor, explore, export, flow, fmt, frame, i18n, materials, mcp, migrate,
    model, prefab, probe, render, replay, run, script, serve_relay, session, sfx, sprite, style,
    test, ui, upgrade, vfx_import, vfx_preview, video, world_server,
};
mod api_lookup;
mod author_types;
mod bundle_cache;
mod contact_sheet;
mod device;
mod doctor_project;
mod keyframes;
mod lockstep_lint;
mod module_state;
mod project;
mod publish;
mod relay;
mod style_check;
mod trace;
mod ui_api_docs;
mod ui_components;
mod ui_data;
mod ui_grounds;
mod ui_styles;
mod ui_values;
mod vfx_catalogue;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
og: the Origin Engine tool

usage: og <command> [options]

commands:
  create [dir] [--name \"Name\"] [--id slug] [--premise \"one line\"] [--force] [--no-prompt]
      [--form F] [--genre G] [--view V] [--look L] [--variant V] [--ui U] [--platform P]
      [--input I] [--players P] [--lang en,zh] [--music M]
                      write one empty game project and its creation choices; `og new` is an alias;
                      never prompts with --no-prompt or without a terminal; --look is a catalogue
                      style with a preset (an unknown one exits 1, listing them);
                      --genre interactive-film adds a flow (og api show flow);
                      the start guide §1
  new [dir] [options]  alias for og create
  script <file.ts> [--root DIR] [--params JSON] [--seed N] [--input name=path] [--out FILE] [--json]
                      execute a deterministic cook generator; no network, arbitrary files, processes or clock
  project [project] --json
                      read checked TS bootstrap and normalized host startup data, without executing game/content code
  session [project]   keep a worktree's native host warm until idle or interrupted
  run [project|entry] [--headless] [--commands \"t:action[:args];...\" | --commands-file <f>] [--seed S] [--port P]
      [--ticks N] [--out DIR] [--size WxH] [--setup action[:arg]]... [--audio null|device|<out.wav>] [--content <dir>]
      [--platform-store <file.json>] [--platform-fake <file.json>] [--clock <UTC ms>] [--locale <tag>]
      [--trace <out.json>] [--perf <dir> [--budget <line>=<ms>]...] [--gpu-serial] [--profile <out.cpuprofile|out.md>]
      [--room <ws url>] [--shared <ws or https url> [--shared-token-file <f>] [--shared-cert-hash <hex>]]
      [--capture-only] [--forks N] [--restart-at S [--restart-keep Type.field,...]]
      [--json] [--no-check] [--require-checked]
                      cook, type-check, bundle and run the game on the native host with the probe;
                      outputs of screenshot/dump_* land in --out (default <project>/og-out)
                      --commands-file: UTF-8, one step per line or separated by semicolons;
                      blank lines and full-line # comments are ignored (also render/replay record)
                      --require-checked exits 1 when GPU checks remain unchecked (default: warning)
  run --device chrome|safari|android[:serial]|iphone[:udid] [project] [--seconds N] [--port P] [--out DIR]
      [--size WxH] [--headed] [--chrome <path>] [--cert <pem> --key <pem>] [--host <ip>]
      [--json]
                      play the publish build in a real browser (served with COOP/COEP on 127.0.0.1,
                      or for the iPhone on 0.0.0.0 with the LAN URL printed; --host names the
                      address), pass the play gate and time the first frame and the frames;
                      report.md and device.json in --out (default <project>/og-out/device)
  replay record [project] --commands \"t:action[:args];...\" | --commands-file <f> [--ticks N] [--seed S]
      [--clock <UTC ms>] [--locale <tag>] [--platform-fake <file.json> | --platform-journal <journal.json>]
      [--keyframe-every K] [--loads <journal.json>] [--content <dir>] --out <trace.json> [--json]
                      --commands-file: one step per line or semicolons, blank lines and full-line # comments
  replay compare <trace.json> [project] [--content <dir>] [--json]
                      record the world's portable hash after every tick of a scripted headless run
                      (and its save every K ticks, default 30), or replay a trace with this engine
                      and report the first divergent tick with the keyframe's diff; compare exits
                      0 the same, 3 diverged
  explore [project] [--seconds S] [--ticks N] [--memory MB] [--jobs N] [--seed S] [--horizon T]
      [--out DIR] [--web auto|on|off] [--first] [--no-shrink] [--json]
                      search the game's states from snapshots (Go-Explore: an archive of cells from
                      reflection, returns by restore, bursts of its actions and intents) for invariant
                      violations, systems that throw and the sometimes/reachable/each sites it reaches;
                      each finding is shrunk to a minimal input script, replayed to the same world
                      hash in a fresh og run and on the web host, and written as a repro and a bun:test
                      playtest beside report.json in --out (default <project>/og-out/explore); defaults
                      60 s, 200000 ticks, 256 MB of snapshots, 1 job, horizon 3600 ticks; --first
                      stops at the first violation or error and reports only those; the explore
                      guide: og api show explore
  render [project] --commands \"t:action[:args];...\" | --commands-file <f> | --timeline <name> [--fps F] [--from S] [--to S]
      [--size WxH] [--out DIR|out.mp4|out.webm] [--frames png|jpg] [--audio] [--contact-sheet COLS]
      [--keyframes [--keyframe-change F] [--keyframe-gap S]] [--jobs N] [--seed S] [--json] [--no-check]
                      render the script to frames, a video (ffmpeg: H.264 or VP9, the capture
                      mixer's audio with --audio), frames.json (tick, time, sha256 per frame) and a
                      contact sheet; the fps divides the tick rate; --jobs splits the range;
                      --commands-file: one step per line or semicolons, blank lines and full-line # comments
                      --keyframes adds the frames where something happened (an input, a step, a
                      screen change, an assertion or expectation, a mean FLIP of F, default 0.02,
                      from the last key frame at least S seconds, default 0.5, after it) as
                      keyframes/<index>.png at 1280, keyframes.png and timeline.json (the captures
                      guide)
  model <command> <program.json | mesh> [options]
                      og-shape's model and mesh tools: build, export, measure, preview, import,
                      repair, decimate, unwrap, place, collide, compress, process, rig
                      (`og model help`)
  compare <reference.png> <render.png> [--mask <mask.png>] [--align] [--key <ΔE00>] [--sheet <out.png>]
      [--flip <out.png>] [--ppd P] [--regions N] [--json]
                      a model review's numbers: silhouette IoU and ΔE00 per mask region (or over the
                      overlap) of a captured frame against a reference image, and one sheet PNG of
                      the reference beside the render beside their silhouettes (the art-review guide);
                      NVIDIA's FLIP of the two (mean, percentiles, max), its heat map (default
                      <render>.flip.png) and its worst regions, each reference | render | heat
                      enlarged in <heat>-<n>.png (the captures guide)
  cost [project] --off \"<og run options>\" [--on \"<og run options>\"] [--runs N] [--ticks A..B] [--json]
      [-- <og run options for both>]
                      what a feature costs the real frame: og run --trace with it on and off, at
                      least 4 runs a side alternating, the median GPU frame span of each side with
                      its range and the load, and their difference (the render guide)
  migrate three <file.ts> [--out <file.ts> | --write] [--json]
                      rewrite a three.js file towards og where the mapping is mechanical (geometries,
                      materials, meshes, lights, the scene graph, transforms, the camera, texture
                      loads) and list each line that is not, with the three-to-og guide's section
  probe <method> [params-json] [--port P] [--out file.png] [--timeout S] [--raw]
                      one probe call to a running `og run --port P` / `og dev` (JSON-RPC over WebSocket):
                      the result as JSON on stdout, exit 0 (a refusal, false or null, is a result); an
                      error on stderr, exit 1; a capture/marks PNG written to --out (default capture.png),
                      capture_views/sweep frames beside it as <stem>-<view>.png, listen WAV to --out
                      (default listen.wav); --timeout sets socket timeout in seconds (default 30);
                      --raw prints the whole answer frame (the testing guide §9)
  mcp                 serve the og tool over MCP on stdio: probe, run_logs, run, test, check, cook,
                      build, create, explore, replay, ui, style, model, assets, prefab, doctor,
                      render, compare, frame, cost, vfx, sfx, sprite, materials, i18n, flow, fmt,
                      font, api_find, api_show and api_list, with the same JSON as the commands
  dev [project] [--port P] [--exit-after N] [--json] [--engine]
                      watch src/ and assets/, rebuild and re-cook, hot-reload through the probe,
                      report each reload against the 300 ms budget and type-check each src/ change
                      beside the rebuild (reported, never refusing the reload)
  dev [project] --web [--port P] [--host H]
                      serve the development web build on http://H:P/ (default 127.0.0.1:47810;
                      the inspector, Tune, Pause and Step, the probe) with every edit pushed to
                      the open page: assets and UI applied in place, code swapped with the world
                      kept, a reload when project.ts or og-project.json changes (the testing guide §1)
  check [project|entry|tsconfig.json] [--json] [--checkers N] [--tool tsrs|tsgo] [--require-checked]
                      type-check the game code with tsrs (tsgo where tsrs has no package);
                      --json prints NDJSON (`check` plus optional `ui-check`/`style-check` events);
                      og run, og test and og build check before bundling (--no-check skips it), og dev on each save
  test [project] [--filter <path>] [--skip <path>] [--affected <rev>] [--jobs N] [--json] [--no-check] [--require-checked]
                      the project's bun tests, then each tests/*.og.ts playtest file (bun:test,
                      its tests awaiting runPlaytest) under bun test against the built game
                      --filter selects suite files by a project-relative path substring;
                      --skip omits suite files matching a substring (repeatable);
                      --affected <git-revision> selects changed dependencies (unknown inputs run all);
                      --jobs N runs up to N playtest files at once, each one's output whole, in order
                      --json: stdout is JSON Lines only, the check, playtest and test events (the
                      testing guide §9); the prose goes to stderr
  build [project] [--publish] [--out DIR] [--assets URL] [--content URL] [--modules cdn|local] [--json] [--no-check]
                      cook, type-check and export the game to dist/ (or publish.out): the player shell,
                      game.js, the web host, cooked/, og-build.json, _headers; --publish keeps
                      only the player-facing probe members and adds the PWA (sw.js, manifest);
                      --json: the check and build events as JSON Lines (the testing guide §9)
  cook [project] [--threads N] [--report [out.md]] [--json] [--watch [--exit-after N]]
                      assets/ into cooked/, incrementally; --report prints the ten slowest
                      assets by their last cook; --watch cooks each change, timing it
  export gltf <asset> [project] -o <file.glb> [--lod N]
                      cook the project and write the cooked model <asset> (a canonical content ID
                      for authored parts/plants/programs, or an imported mesh ID) as a glTF binary: one
                      level of detail (0, the full mesh, by default), its materials with PNG
                      textures, its instances as nodes, its skins, clips and morph targets
  assets [--kind KIND] [--group GROUP] [--unused] [--refs] [--by] [--dupes [--near N]] [--json] [text]
                      query cooked metadata; --refs shows dependencies, --by their users;
                      --near uses caller-chosen Hamming distance 0..64; --project DIR selects a project
  assets mv <from> <to> [--project DIR] [--json]
                      move a source and its import settings, rewrite static OGT/TS asset references
  fmt [path…] [--check] [--json]
                      rewrite .ogt files in the canonical layout, comments kept: a file, a directory,
                      or a project's assets/ (the default, here); entries keep their
                      order; --check writes nothing, lists what would change and exits 1 if any does
  cache gc [--max-bytes N] [--json]
                      trim the shared cook action/content cache (default cap 10 GiB)
  bake lightmaps [project] [--samples N] [--bounces N] [--texels-per-metre N] [--sky R,G,B] [--scene <id>]
                      trace the light of the scenes with static models or LightVolumes into
                      content/lightmaps/ TS and assets/lightmaps/ KTX2 (CPU, native), then cook
  prefab check [project]
                      flatten every scene and prefab as the cook does: cycles, name clashes and
                      orphaned overrides (a node, component or field the prefab no longer has);
                      exits 1 while any exists (a publish cook refuses an orphaned override)
  prefab show <prefab> [--scene <id> --instance <name>] [--project <dir>]
                      the flattened tree, each field with the text its value comes from
  prefab extract <scene> <entity> <prefab> [--project <dir>]
                      an authored entity becomes a new prefab, its block an instance of it
  i18n extract [project] [--json] [--write] [--check]
                      every og.i18n key the dialogues, quests and tables show (.yarn,
                      type quest, file.table), with the arguments they pass and where,
                      against each string table: per locale, the keys it lacks and the arguments nothing passes;
                      --write adds the keys the base table lacks to it (content/strings/en.ts
                      when there is none); --check exits 1 while any table is incomplete
  i18n tables [project] [--builtins | --index] --json
                      checked locale metadata, canonical IDs and ICU sources from all modules,
                      or the installed engine's UI tables; --index lists only project IDs/paths
                      without evaluating content; never executes game code
  sfx [<preset> [--seed N] [--pitch P] [--length L] [--volume V] --out <file.ts|file.wav>]
  sfx library <dir>
                      the procedural sound presets (coin, jump, hit, click, ...): list them,
                      write one as an asset `og cook` synthesises (or render it to a WAV),
                      or write the whole library
  materials brief <library.ogt> [--json]
  materials make <library.ogt> [--json]
                      a material library's maps from images an image model draws: list the
                      images its briefed materials ask for (art brief kind, metres, where each
                      goes), or make each map periodic (a facade module across only) and its
                      authored normal, packed roughness and cavity maps, reporting seams (the
                      districts guide)
  sprite inspect <file.gif|.webp|.png|content/sheet.ts> [--key auto|off|#rrggbb] [--tolerance T]
      [--holes] [--out <frames.png>] [--json]
                      measure an animated sprite source or a flipbook sheet built from them:
                      frames, delays, each frame's drawn bounds and centroid and their drift,
                      the loop seam against the frame-to-frame steps, the alpha edge fringe
                      (numbers only; a flat border is keyed before measuring unless --key off);
                      the texels near the key still drawn (pockets the border does not reach;
                      --holes keys them too, as a sheet's key.holes);
                      --out writes the measured frames side by side (a sheet: its packed texture)
  font add <pack|directory> [project]
                      a font pack (one per script: latin, cyrillic-greek, arabic, hebrew,
                      devanagari, thai, japanese, korean, chinese-traditional,
                      chinese-simplified; and emoji, colour emoji painted when first
                      shown) into assets/fonts/<pack>; og.i18n puts the active
                      locale's pack in the font chain, the others load on demand
  font list [project] the bundled and installed font packs, with their languages
                      (a pack of no language: its scripts)
  vfx add <pack|directory> [project] [--any-engine]
                      copy an effect pack (`kind vfx`: file.vfx and file.vfx2d effects,
                      emitters, effect materials and their textures) into the game's
                      assets/vfx/<pack>/ and content/vfx/<pack>/ to use and change: a directory, a pack bundled with
                      the engine (`og ui list` names them), else one of this engine's
                      downloadable packs (its release's manifest.json `vfx` names them),
                      fetched from <downloads>/<version>/vfx/<pack>.tar.gz and refused unless
                      the release's SHA256SUMS verifies with the engine's key and lists it
                      (downloads: $ORIGIN_ENGINE_DOWNLOADS, default
                      https://origingame.ai/downloads/engine); --any-engine installs a pack
                      whose minEngine is newer than this engine
  vfx import <file.efkproj|file.efkefc> <out-dir> [--root <assets-dir>] [--hit]
                      convert an Effekseer effect to the engine's own as text to edit:
                      <out-dir>/<name>.ogt (type vfx), <name>/ (emitters, effect materials,
                      meshes) and <out-dir>/textures/ (copies of its textures, shared by the
                      effects imported there; asset ids from --root, default the
                      nearest assets/ above <out-dir>), listing what did not convert; an
                      Effekseer file in assets/ is converted the same way by og cook. A
                      one-shot effect gets an impact where the most starts at once; --hit
                      adds a light and a camera shake there (hits, explosions)
  vfx preview <effect.ogt>... [--project DIR] [--out DIR] [--size WxH] [--stage dusk|day]
              [--project-look | --look asset.ogt] [--scene asset.ogt]
              [--position x,y,z] [--camera px,py,pz,tx,ty,tz,fov]
              [--quality low|medium|high] [--no-cost | --cost-only | --together] [--keep] [--json]
                      play each effect (type vfx, or canonical file.vfx2d ID in a 2D project) alone on a
                      dark stage at dusk (--stage day: a grey one under an overcast day, where
                      smoke, dust, debris and marks read), or the selected project look/scene,
                      and report it: six frames tiled into
                      <out>/<effect>.png (default out: og-out/vfx-preview), the camera placed
                      by the box the effect was measured to reach; its length, impact time,
                      measured size and anchor, draw calls, particle slots, the engine's
                      warnings that name it and its peak GPU
                      milliseconds over an empty frame (--no-cost skips the timed run,
                      --cost-only adds it to an earlier preview in --out), also written to
                      <out>/<effect>.json; --together plays them all at once on a grid and
                      reports the frame's and each pass's GPU milliseconds (together.json)
  style list [project] [--json]
                      list bundled and installed style directories
  style add <directory> [project]
                      install a theme.json or pack.json style under assets/styles/<name>
  style init <name> [project] [--from <theme>]
                      copy a theme recipe into assets/styles/<name>/theme.json (default: minimal);
                      never overwrites an existing style
  style validate <directory>
                      compile a theme recipe, parse its pack and report grounds without paired inks
  style check-project [project] [--title <commands>] [--play <commands>] [--results <commands>] [--size WxH]
      [--no-frames] [--json]
                      cook, then measure every cooked asset against the art contract of the look
                      og-project.json names (palette hit rate, colours, pixel grid, texel density,
                      faces against the budget, flat normals, the line) and list every opt-out;
                      capture the title, play and results screens (project.ts's screens map or
                      the flags) and measure their colours, palette and edges; exit 0 within the
                      game's own contract or with no look, 3 off it
  ui list [project] [--json]
                      list bundled and installed UI style packs, effect packs and materials
  ui add <materials|directory> [project]
                      install UI materials or a UI pack directory into assets/ui
  ui check [project] [--screen <name>[=<commands>]]... [--show <template>] [--styles a,b]
      [--style-cmd <template>] [--breakpoints desktop,phonePortrait,phoneLandscape,tablet]
      [--locales game,pseudo,<cjk>,<rtl>] [--scales 100,150] [--high-contrast] [--settle S]
      [--out DIR] [--json] [--require-checked]
                      lay each screen (its commands, or show:<name>; without --screen every screen
                      project.ts's screens map declares,
                      else the values the game's show usage lists, else the playtests' screenshots)
                      out per style, breakpoint and
                      locale (the game's, a pseudo-locale 40 % longer, CJK, RTL) and report facts:
                      findings (overlap, cut text, the safe area and the shell's corner,
                      pad reach, names, feedback) with the paths, what was found and the prop, and
                      measures with no bar (contrast on pixels, text size, touch targets, text
                      scale, primary actions, element count) (DIR/check.json, check.txt; default
                      <project>/og-out/ui-check); exit 0 when it ran, 1 when it could not
  ui preview [project] [the same options] [--film <commands> --at t,t,...]
                      a contact sheet per screen and style (a row per breakpoint, a column per
                      locale) and index.json of every cell, or a filmstrip of a motion at the
                      chosen times after the screen settles (default <project>/og-out/ui-preview)
  ui literals [path...] [--json]
                      the style values a document writes itself instead of taking from the
                      style: colours (#rrggbb, { r, g, b }), corner radii and font families, each
                      with the token to use; facts only, exit 0 when it ran (default: the
                      project's assets/ui); `style-literal-ok` on a line skips it
  ui eject <name> [project] [--force] | ui eject --list [project]
                      copy a built-in TSX component (hud/stat-bar), screen (screens/title) or
                      control (controls/button) into src/ui/<name>.tsx as the game's own: its
                      strings into content/ui/<name>.<locale>.strings.ts under game. keys, its imports of other
                      built-ins kept, and the game's imports of it pointed at the copy; refuses to
                      overwrite without --force; --list prints each with its doc comment's line
  frame measure <png> [--region x,y,w,h] [--against x,y,w,h] [--units px|fraction]
      [--metric contrast,highpass,mean,ramp] [--rows N] [--json] [--require-checked]
                      a reviewer's numbers off a captured frame, the same on every machine: WCAG
                      contrast (the ink, the region's darkest or brightest 5 %, against the median
                      ground of --against or the region), high-pass energy (mean |grey - its σ 2
                      blur|, with its ratio to --against), mean colour, and the grey per band of
                      --rows rows with the largest step (the art-review guide)
  relay [--port P] [--host <ip>]
                      a local WebSocket relay for rooms: `og run --room ws://127.0.0.1:P/<room>`
                      processes join <room> and play together (og.net's lockstep); behind a TLS
                      proxy it is reached as `og run --room wss://<host>/<room>`
  world-server [--listen ADDR] [--wt-listen ADDR --cert F --key F] [--config F]
      [--auth hs256|jwks|none --dev] [--secret-file F] [--jwks URL|FILE] [--audience GAME] [--metrics ADDR] [--store F]
                      the shared-world server (og.shared): presence and avatars near each client,
                      and shared objects (kept across restarts with --store), standalone on any
                      host; tokens checked offline (hs256, jwks; none only with
                      --dev); GET /healthz, Prometheus --metrics, SIGTERM drains. `world-server
                      load` runs a bot swarm against one and measures bytes, sight and tick time
                      (--gate holds it to the design's budgets; --latency and --loss shape it);
                      `world-server token` signs an HS256 token for a self-hosted server;
                      `world-server cert` makes a self-signed WebTransport certificate and prints
                      the hash a page passes
  doctor [--project <dir>] [--json] [--require-checked]
                      check bun, the engine package, the native addon, the wasm, a headless
                      renderer, and (in the engine repository) the build toolchain, naming each fix;
                      --project validates a game project's layout and pin;
                      --json adds the pin: {pinned, installed, compatible} (same minor, installed >= pinned, and
                      no version after the pin up to the installed one has breaking entries)
  upgrade [project] [--to <version>] [--dry-run] [--json]
                      move og-project.json's engine pin and package.json's dependency to the installed
                      engine (or --to), apply the changelog's codemods for the versions crossed and
                      list what needs a hand edit; exit 0 done, 3 hand edits listed, 1 failed
  api find <words> [--kind module|fn|record|type|member|const|value|column|guide|file] [--limit N] [--json]
                      the API reference and guide entries for the words (a name, a purpose, a
                      typo), ranked by BM25F over name, summary, members and body fused with
                      static word embeddings, one line each; --json adds the scores
  api show <name> [--all] [--guide] [--json]
                      one API entry in full: signature, fields, doc (og.CharacterController,
                      og.physics.castRay, MAX_RAYCAST_HITS, Button, \"saves §2\"; og. may be left out);
                      a module shows its overview and its core members, --all every member; --guide shows
                      the guide named like a module (physics, input, camera, audio, render); large guides
                      print a section table unless --all is passed
  api list [--kind module|guide|file] [--json]
                      list every API module and guide with one-line summaries and section counts
  api gen [--wiki] [--root <dir>]
                      regenerate the IDL outputs (packages/engine/{types,schema}, the hosts' glue,
                      the knowledge wiki); --wiki writes only the wiki
  api check [--root <dir>] [--budget]
                      exit 1 when a generated file differs from crates/og-api/api/, or more
                      descriptions are over their budget (600 characters for modules, functions
                      and types, 300 for fields and the rest) than its backlog; --budget lists them
  bake projection <init|plan|prepare|project|holes|export> --run <dir> [options]
                      texture a level from views an image model paints (CPU, native): the
                      caller paints each prepared view; `og bake projection` lists the steps
  bake probes [project] [--commands \"t:action[:args];...\"] [--at S] [--size WxH]
                      render the baked og.ReflectionProbe captures on the native host into
                      cooked/probes/ (loaded instead of drawn on first use)
  flow list|routes [project] [--json]
                      a project's flows (og.flow) from its sources: list gives each node's segment
                      (footage, live or both; duration; window or hold), the footage and scenes it
                      names (missing, placeholder) and where each choice leads; routes gives every
                      route from the start to an ending as the steps that take it, and the endings
                      none reaches (a step marked ? passes a logic node's condition)
  video edges <clip> [--out DIR]
                      write <stem>.first.png and <stem>.last.png; clip is .mp4, .webm, .mov or
                      a file.video TS declaration (source and maxSeconds); requires ffmpeg (OG_FFMPEG)
  video study <clip> [--out DIR] [--interval S] [--at 2,5,8] [--window S] [--cut-change F]
                      study AI footage or a recording with ffmpeg (OG_FFMPEG); clip as for edges;
                      output defaults to <clip-stem>.study/ beside the clip: study.json,
                      frames/<time>s-pts<PTS>-f<index>.png (native size), contact-sheet.png
                      all selected PNGs are streamed; the sheet shows at most 24 evenly spaced
                      frames including first and last; JSON sheet_frames lists their source indices
                      --interval S (default 0.25): fixed samples from 0, strictly before the end;
                      --at adds exact requested seconds without changing interval statistics;
                      both select the displayed frame (greatest PTS <= time), not a future frame
                      times are relative to the first decoded PTS; JSON also keeps integer source
                      PTS, time_base, every decoded frame, packet-end duration and nominal fps;
                      timestamps describe the file, not rendering throughput before encoding
                      --cut-change F (0..1, default 0.35): candidates where adjacent decoded
                      frames' FFmpeg scene score > F; JSON keeps all scores, no quality verdict
                      palettes: up to 8 Oklab colors with opaque pixel fractions, whole clip and per cut
                      segment; interval thumbnails plus segment starts, long side <=192,
                      equal-weight pixels (not duration weighted); extra --at frames do not bias it
                      changes: mean FLIP of adjacent interval samples fitted to 192 (scaled PPD)
                      --window S (default 3): every full window at interval steps, mean and median
                      of changes ending in (start,end]; even median averages the middle two;
                      empty statistics are null; windows stay chronological, never ranked

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
            let v = project::own_version();
            if project::is_dev(v) {
                println!(
                    "og {v} (origin-engine {v}, based on {})",
                    project::own_release()
                );
            } else {
                println!("og {v} (origin-engine {v})");
            }
            ExitCode::SUCCESS
        }
        ["--help"] | ["-h"] | ["help"] => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        ["doctor", rest @ ..] => doctor::run(rest),
        ["create" | "new", rest @ ..] => create::run(rest),
        ["upgrade", rest @ ..] => upgrade::run(rest),
        ["run", rest @ ..] => run::run(rest),
        ["session", rest @ ..] => session::run(rest),
        ["replay", rest @ ..] => replay::run(rest),
        ["explore", rest @ ..] => explore::run(rest),
        ["render", rest @ ..] => render::run(rest),
        ["compare", rest @ ..] => compare::run(rest),
        ["model", rest @ ..] => model::run(rest),
        ["cost", rest @ ..] => cost::run(rest),
        ["migrate", rest @ ..] => migrate::run(rest),
        ["relay", rest @ ..] => serve_relay::run(rest),
        ["world-server", rest @ ..] => world_server::run(rest),
        ["probe", rest @ ..] => probe::run(rest),
        ["mcp", rest @ ..] => mcp::run(rest),
        ["dev", rest @ ..] => dev::run(rest),
        ["check", rest @ ..] => check::run(rest),
        ["test", rest @ ..] => test::run(rest),
        ["build", rest @ ..] => bundle::run(rest),
        ["cook", rest @ ..] => cook::run(rest),
        ["script", rest @ ..] => script::run(rest),
        ["project", rest @ ..] => project::run(rest),
        ["export", rest @ ..] => export::run(rest),
        ["assets", rest @ ..] => assets::run(rest),
        ["fmt", rest @ ..] => fmt::run(rest),
        ["cache", rest @ ..] => cache::run(rest),
        ["bake", "projection", rest @ ..] => bake_projection::run(rest),
        ["bake", "probes", rest @ ..] => bake_probes::run(rest),
        ["bake", rest @ ..] => bake_lightmaps::run(rest),
        ["api", rest @ ..] => api::run(rest),
        ["prefab", rest @ ..] => prefab::run(rest),
        ["sfx", rest @ ..] => sfx::run(rest),
        ["materials", rest @ ..] => materials::run(rest),
        ["sprite", rest @ ..] => sprite::run(rest),
        ["video", rest @ ..] => video::run(rest),
        ["flow", rest @ ..] => flow::run(rest),
        ["i18n", rest @ ..] => i18n::run(rest),
        ["style", rest @ ..] => style::run(rest),
        ["ui", rest @ ..] => ui::run(rest),
        ["vfx", "add", rest @ ..] => ui::extensions::add(rest, "vfx"),
        ["font", "add", rest @ ..] => ui::extensions::add(rest, "font"),
        ["font", "list", rest @ ..] => ui::extensions::list_fonts(rest),
        ["vfx", "import", rest @ ..] => vfx_import::run(rest),
        ["vfx", "preview", rest @ ..] => vfx_preview::run(rest),
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
