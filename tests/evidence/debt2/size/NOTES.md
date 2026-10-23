# Debt round 2: size (the engine's side of row 99)

Branch `debt2/size`, from `ad903b18` (origin/main, after debt2/physics-push).
Each item is one commit; each is measured against the table below on the
module that ships. Nothing here ran `scripts/check.sh` whole, `og test`,
web-reference or a release native build.

## How it was measured

- Both publish variants, built as `packages/engine` ships them: the
  shipping `wasm` profile (codegen-units 1, thin LTO, the load-time crates
  at opt-level z), `cargo build --target wasm32-unknown-unknown --profile
  wasm -p og-host-web --lib --no-default-features --features
  og-host-web/publish,og-host-web/physics3d,og-host-web/shaping` (and
  `physics2d` for the 2D host), then `scripts/wasm-size-report.sh --keep
  DIR` (wasm-bindgen 0.2.129, `scripts/wasm-opt.sh`'s flags; binaryen 133;
  `brotli -q 11`). Its first line is the size gate's number for the module.
- The per-crate table is twiggy 0.8.0's `top` (shallow bytes of every
  item) over the same module optimised with `-g` (DIR/named.wasm), each
  item charged to a crate by `scripts/wasm-size-report.ts`'s rule (the
  first crate its demangled name mentions that is not core, alloc, std or
  hashbrown): `twiggy-crates.ts` here (`TWIGGY=<path> bun
  twiggy-crates.ts DIR/named.wasm [og-render's OUT_DIR/shaders]`), joined
  with the report's "brotli alone" by `basetable.py`, compared across
  builds by `table.ts`. twiggy's shallow bytes and the report's raw bytes
  agree to 0.2 % per crate (twiggy counts each body's size prefix); the
  name section (1.95 MB) is left out, the shipping module has none.
- The registry's 0.3.0 modules the platform read were 13,024,056 B (3D)
  and 12,386,562 B (2D) raw; at `ad903b18` they are 13,045,922 B and
  12,408,297 B (+21,866 and +21,735 B: what main merged after 0.3.0).

## The table at ad903b18 (before any trim)

Size gate (brotli -q 11 of the wasm-opt'd module): **3D publish
3,532,992 B (84.2 % of 4,194,304), 2D publish 3,357,559 B (80.1 %)**;
after wasm-opt 13,045,922 B and 12,408,297 B.

| Crate | 3D twiggy B | 3D share | 3D brotli alone | 2D twiggy B | 2D share | 2D brotli alone |
|---|---:|---:|---:|---:|---:|---:|
| og_runtime | 1,783,546 | 13.67 % | 517,974 | 1,776,596 | 14.32 % | 515,581 |
| [data] | 1,517,360 | 11.63 % | 372,130 | 1,506,444 | 12.14 % | 369,664 |
| og_render | 1,040,886 | 7.98 % | 313,553 | 1,041,109 | 8.39 % | 313,448 |
| og_host_web (exports) | 886,782 | 6.80 % | 251,070 | 885,300 | 7.13 % | 250,222 |
| og_physics | 827,146 | 6.34 % | 237,945 | 645,248 | 5.20 % | 179,132 |
| rapier3d | 826,166 | 6.33 % | 268,601 | 0 | 0.00 % | 0 |
| parry3d | 773,619 | 5.93 % | 205,612 | 0 | 0.00 % | 0 |
| rapier2d | 0 | 0.00 % | 0 | 713,382 | 5.75 % | 234,089 |
| naga | 662,011 | 5.07 % | 192,820 | 661,999 | 5.34 % | 192,729 |
| parry2d | 0 | 0.00 % | 0 | 457,767 | 3.69 % | 128,330 |
| wgpu | 431,909 | 3.31 % | 145,765 | 431,909 | 3.48 % | 145,892 |
| std | 430,368 | 3.30 % | 100,506 | 427,039 | 3.44 % | 99,050 |
| wgpu_core | 374,776 | 2.87 % | 96,259 | 374,776 | 3.02 % | 96,166 |
| og_ui | 331,342 | 2.54 % | 108,316 | 331,343 | 2.67 % | 108,206 |
| og_ecs | 257,364 | 1.97 % | 68,644 | 257,326 | 2.07 % | 68,782 |
| og_anim | 252,149 | 1.93 % | 75,843 | 252,240 | 2.03 % | 75,962 |
| serde_json | 220,102 | 1.69 % | 42,736 | 220,103 | 1.77 % | 42,730 |
| wgpu_hal | 191,807 | 1.47 % | 65,594 | 190,844 | 1.54 % | 65,320 |
| og_probe | 180,985 | 1.39 % | 59,758 | 180,983 | 1.46 % | 59,682 |
| bincode | 169,239 | 1.30 % | 46,197 | 173,974 | 1.40 % | 48,486 |
| og_core | 158,957 | 1.22 % | 33,175 | 158,959 | 1.28 % | 33,234 |
| og_cook | 156,204 | 1.20 % | 45,177 | 156,058 | 1.26 % | 45,142 |
| og_nav | 150,988 | 1.16 % | 50,488 | 150,987 | 1.22 % | 50,419 |
| [unnamed] | 142,613 | 1.09 % | 38,744 | 145,930 | 1.18 % | 39,646 |
| ttf_parser | 130,565 | 1.00 % | 42,745 | 130,565 | 1.05 % | 42,788 |
| rustybuzz | 109,407 | 0.84 % | 40,404 | 109,409 | 0.88 % | 40,386 |
| og_host_web | 101,827 | 0.78 % | 25,422 | 101,825 | 0.82 % | 25,462 |
| og_audio | 94,928 | 0.73 % | 34,520 | 94,930 | 0.77 % | 34,446 |
| og_ai | 80,986 | 0.62 % | 27,433 | 80,988 | 0.65 % | 27,436 |
| js_sys | 77,939 | 0.60 % | 26,251 | 77,937 | 0.63 % | 26,262 |
| og_timeline | 72,110 | 0.55 % | 16,028 | 72,110 | 0.58 % | 16,031 |
| [wasm structure] | 62,497 | 0.48 % | 0 | 62,490 | 0.50 % | 0 |
| og_input | 48,485 | 0.37 % | 13,172 | 48,485 | 0.39 % | 13,185 |
| indexmap | 48,385 | 0.37 % | 10,371 | 46,772 | 0.38 % | 10,439 |
| og_tilemap | 47,078 | 0.36 % | 16,177 | 47,051 | 0.38 % | 16,214 |
| og_net | 40,293 | 0.31 % | 13,406 | 40,293 | 0.32 % | 13,403 |
| og_tween | 34,871 | 0.27 % | 11,717 | 34,873 | 0.28 % | 11,703 |
| og_i18n | 32,771 | 0.25 % | 12,753 | 32,771 | 0.26 % | 12,775 |
| og_lanes | 32,369 | 0.25 % | 7,891 | 29,942 | 0.24 % | 7,421 |
| glam | 29,303 | 0.22 % | 9,854 | 27,027 | 0.22 % | 9,080 |
| wgpu_types | 28,828 | 0.22 % | 9,154 | 28,828 | 0.23 % | 9,153 |
| miniz_oxide | 26,124 | 0.20 % | 11,405 | 26,124 | 0.21 % | 11,408 |
| codespan_reporting | 21,414 | 0.16 % | 8,462 | 21,420 | 0.17 % | 8,483 |
| (the rest, under 20,000 B) | 159,146 | 1.22 % | 57,754 | 153,864 | 1.24 % | 55,412 |
| **module** | **13,045,645** | 100 % | | **12,408,020** | 100 % | |

"Brotli alone" is each crate's bodies compressed by themselves
(`scripts/wasm-size-report.ts`): it ranks crates by what they cost
compressed and overstates what removing one saves.

What the table says:

- **Physics is the profile split already made**: rapier3d + parry3d +
  og-physics' 3D half are 1.78 MB raw in the 3D host; rapier2d + parry2d
  1.17 MB in the 2D one.
- **The data section (1.5 MB, 372 kB brotli alone)**: the WGSL is
  273,488 B of it (all 73 of og-render's stripped shader files, in both
  hosts; 48,486 B brotli alone), the rest is panic and assertion messages
  and their file paths (`.rodata.1`, 199,565 B, one segment), wgpu's and
  naga's error strings, the commands' help text and reflection docs.
- **The four largest functions are a fifth of the code**:
  `og_render::Renderer::render` 425,361 B (148,808 brotli alone),
  `og_physics::schedule::{closure}` 360,496 B, `og_runtime::extract::extract`
  250,518 B, `og_runtime::engine::assemble` 218,944 B: inlining has
  folded most of the renderer's and the runtime's subsystems into them,
  so a subsystem's own items (`og_render::terrain::*`, `water::*`) show only
  what was not inlined.
- **What the row named, as the table found it** (3D host, twiggy B of the
  items that are theirs alone, before inlining is undone by a cut):
  GPU timestamp queries 4,530 B (`og_render::timing::*` and the hosts'
  forwarding; the rest is inlined into `render`); the probe 180,985 B in
  og_probe plus its hooks in og_runtime (`scene_save` 34,551 B, `describe`
  and the views' id pass 44,118 B of `og_render::id_pass` of which
  `render_ids_from` 22,671 B); the BVH loader 2,925 B
  (`adopt_cooked_bvh` 1,366 B, `bvh_of` 1,559 B; `Bvh::from_parts` is
  inlined into `Bundle::model`, 19,042 B with the rest of the model's
  decoding); og-render's 3D-only subsystems in the 2D host: terrain
  111,803 B (`terrain`, `terrain_tiles`) plus og-runtime's terrain and
  terrain streaming 56,374 B, the scatter 17,352 B, and the 3D WGSL (below).

## Items

### 1. GPU timestamp queries out of publish

- **Change.** og-render gains a `publish` feature, turned on by
  og-runtime's `publish` (so both hosts' publish builds have it). Under it
  `GpuTimer` and `FrameMarks` are empty enums (`timing.rs`): no timer is
  ever made, so the query sets, the resolve and readback ring and the
  encoder's timestamp pairs are not linked; the renderer no longer asks
  the device for `TIMESTAMP_QUERY`; `gpu_timing_absent` answers "a publish
  build times no pass on the GPU", so `RenderCapabilities.gpuTimestamps`
  is false and `ProbeTimings.gpuPasses` and the trace's gpu thread are
  empty there, as on WebGL2. The CPU's pass timings stay (`state()`
  serves them in publish).
- **Measured.** 3D publish 13,045,922 → 13,033,282 B (−12,640 B),
  brotli 3,532,992 → 3,529,527 B (**−3,465 B**); 2D publish 12,408,297 →
  12,395,677 B (−12,620 B), brotli 3,357,559 → 3,353,116 B (**−4,443 B**).
  Per crate: og_render −4,098 B, wgpu −4,760 B (its query-set and
  resolve paths), og_host_web −1,716 B, wgpu_core −1,073 B, data −687 B.
- **Test.** og-render `timing::publish_tests` (run with `--features
  publish`): the publish build reports no GPU timing on an adapter with
  timestamps; the old code has no such build and reports `None` there.
  The measured bytes are the regression number.
- **Shared files.** `crates/og-render/Cargo.toml` (feature `publish`),
  `crates/og-runtime/Cargo.toml` (`publish` turns it on), og-render
  `lib.rs` (the device feature request). `crates/og-api/api.ogt`'s
  `RenderCapabilities.gpuTimestamps` doc says publish has none;
  regenerated with `og api gen` (index.d.ts, api.schema.json, wiki
  render.md). docs/design.md §3.4 (trace) and docs/web-export.md (trace)
  say the same.

### 2. The probe's non-player members compiled out of publish (describe and the views' id pass among them)

- **Why one item.** `describe` is reached from three dispatchers: the
  JSON-RPC `invoke`, the probe program (`run`) and the command channel's
  `describe` action, and `expect:entity:...:within` reads its boxes. A
  publish build refused those members at run time (`served_members`) but
  linked every one of them, so cutting describe meant gating the
  dispatchers, and with them every member publish already refused.
- **Change.**
  - og-probe `rpc.rs`: `invoke` compiles only the seven player members'
    arms under `publish` (`hello`, `tick`, `state`, `input`, `pick`,
    `entity`, `capture`); the others were already `-32601` there.
  - og-probe `probe.rs`: the command channel keeps every action that
    changes the world, because og.ai's commands and a timeline's event
    keys reach it in a shipped game (the input actions, `set`, `teleport`,
    `frame_entity`, `seed`, `pause`, `resume`, `advance`, `quit`, the
    registered commands such as `timeline:play`); the ten that only observe
    the world for a tool (`screenshot`, `listen`, `describe`, `expect`,
    `save_scene`, `dump_world`, `dump_ui`, `dump_entity`, `get`, `help`)
    are not built in and log "`<action>` is not built into a publish
    build". `register_dump` keeps nothing (so the engine modules' dumps,
    `dump_render`, `dump_desync`, `dump_lanes`, the nav and AI dumps, are
    not linked), nor do `set_listen`, `set_memory` and `set_reader`.
  - og-runtime: `SceneView::seen` (describe) answers `None` in publish and
    `pick` no longer runs the views' id passes in any build (it reads the
    main camera's pass alone; describe still runs them); the scene writer
    (`scene_save.rs`), the reloader and `get`'s derive (`atmosphere::derive`)
    are not registered or built in publish.
  - og-probe's lib carries `cfg_attr(feature = "publish", allow(dead_code,
    unused_imports))`: what only the compiled-out members read is unused
    there; the default build still warns about real dead code.
- **Measured** (after item 1). 3D publish 13,033,282 → 12,753,175 B
  (−280,107 B), brotli 3,529,527 → 3,457,520 B (**−72,007 B**); 2D
  publish 12,395,677 → 12,115,631 B (−280,046 B), brotli 3,353,116 →
  3,280,554 B (**−72,562 B**). Per crate (3D): og_runtime −309,052 B,
  og_probe −81,558 B, og_ecs −31,859 B (the world dump and reflection
  JSON), og_render −29,715 B (`render_ids_from` for views, the view
  cameras), std −20,149 B, data −15,673 B (the help text, usages,
  messages); og_host_web's exports grew +239,984 B because wasm-opt now
  inlines the smaller runtime bodies into them, which is where the
  og_runtime figure went in part.
- **Test.** og-probe `tests/publish.rs`
  `the_command_channel_runs_world_actions_and_refuses_the_tools`
  (`--features publish`): a script with `teleport`, a registered command
  and eight tool actions; the world actions run, no output is written,
  each tool action logs its refusal and a registered dump is unknown.
  It fails on the old code (the dump and describe were written as
  outputs). `only_player_facing_members_answer` and the default build's
  41 tests pass unchanged; og-runtime's lib tests (16) pass;
  `cargo check -p og-host-node --features publish` is clean.
- **Behaviour change to judge.** In a publish build, a timeline event key
  or an og.ai command naming a tool action (`dump_world`, `get`, ...) now
  logs a refusal instead of writing an output nobody reads. No game in
  the repository does that.
- **Shared files.** og-runtime `probe.rs`, `engine.rs` (the reloader's
  registration), `lib.rs` (`mod scene_save` gated); docs/design.md §3.4
  and crates/og-probe/README.md §Builds.
