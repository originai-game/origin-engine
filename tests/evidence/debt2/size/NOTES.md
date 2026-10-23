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

### 3. `og cook --report` warns past a share of the model: `bvhShare`, 30 %

- **The share, read from the models.** Every model the repository cooks
  (the templates and examples with a 3D model; the examples cooked from a
  copy with their missing LFS sources' import settings removed), its blob
  and its cooked BVH (`bvhBytes`):

| project | model | LOD 0 triangles | model blob B | BVH B | BVH share of the blob | asset cooked B |
|---|---|---:|---:|---:|---:|---:|
| fx-after | models/tree.glb | 3,160 | 53,758 | 26,486 | 49.3 % | 53,758 |
| templates/fx-showcase | models/tree.glb | 3,160 | 53,758 | 26,486 | 49.3 % | 53,758 |
| examples/rumpus-isle | models/island.glb | 1,920 | 31,296 | 14,404 | 46.0 % | 31,296 |
| ru-after | models/island.glb | 1,920 | 31,296 | 14,404 | 46.0 % | 31,296 |
| game-after | models/dropship.glb | 16,512 | 280,686 | 108,384 | 38.6 % | 280,686 |
| templates/game | models/dropship.glb | 16,512 | 280,686 | 108,384 | 38.6 % | 280,686 |
| examples/lantern-vale | models/slime.glb | 800 | 11,662 | 4,340 | 37.2 % | 15,397 |
| examples/lantern-vale | models/tree.vox | 1,448 | 25,980 | 8,920 | 34.3 % | 25,980 |
| examples/meridian-loop | models/ground.glb | 512 | 6,101 | 2,042 | 33.5 % | 7,931 |
| me-after | models/ground.glb | 512 | 6,101 | 2,042 | 33.5 % | 7,931 |
| fx-after | models/trickster.glb | 18,641 | 560,433 | 172,765 | 30.8 % | 1,098,746 |
| templates/fx-showcase | models/trickster.glb | 18,641 | 560,433 | 172,765 | 30.8 % | 1,098,746 |
| examples/meridian-loop | models/track.glb | 6,674 | 182,701 | 55,190 | 30.2 % | 194,143 |
| me-after | models/track.glb | 6,674 | 182,701 | 55,190 | 30.2 % | 194,143 |
| examples/rumpus-isle | models/palm.glb | 240 | 5,881 | 1,705 | 29.0 % | 5,881 |
| ru-after | models/palm.glb | 240 | 5,881 | 1,705 | 29.0 % | 5,881 |
| examples/meridian-loop | models/scenery.glb | 4,478 | 127,343 | 33,846 | 26.6 % | 134,168 |
| me-after | models/scenery.glb | 4,478 | 127,343 | 33,846 | 26.6 % | 134,168 |
| examples/rumpus-isle | models/berry.glb | 264 | 5,760 | 1,528 | 26.5 % | 5,760 |
| ru-after | models/berry.glb | 264 | 5,760 | 1,528 | 26.5 % | 5,760 |
| examples/kestrel-stair | models/stair.glb | 1,196 | 30,701 | 7,605 | 24.8 % | 59,493 |
| examples/lantern-vale | models/rock.vox | 254 | 5,767 | 1,374 | 23.8 % | 5,767 |
| examples/rumpus-isle | models/islet.glb | 192 | 5,342 | 1,269 | 23.8 % | 5,342 |
| ru-after | models/islet.glb | 192 | 5,342 | 1,269 | 23.8 % | 5,342 |
| examples/kestrel-stair | models/imp.glb | 360 | 11,204 | 2,540 | 22.7 % | 15,479 |
| examples/lantern-vale | models/lantern.vox | 188 | 4,660 | 1,046 | 22.4 % | 4,660 |
| examples/kestrel-stair | models/courier.glb | 312 | 10,175 | 2,214 | 21.8 % | 76,907 |
| fx-after | models/strider.glb | 252 | 7,965 | 1,621 | 20.4 % | 28,466 |
| templates/archetypes/third-person | models/strider.glb | 252 | 7,965 | 1,621 | 20.4 % | 28,466 |
| templates/fx-showcase | models/strider.glb | 252 | 7,965 | 1,621 | 20.4 % | 28,466 |
| fx-after | models/stout.glb | 252 | 7,865 | 1,585 | 20.2 % | 12,065 |
| templates/archetypes/third-person | models/stout.glb | 252 | 7,865 | 1,585 | 20.2 % | 12,065 |
| templates/fx-showcase | models/stout.glb | 252 | 7,865 | 1,585 | 20.2 % | 12,065 |
| examples/kestrel-stair | models/warden.glb | 264 | 8,558 | 1,718 | 20.1 % | 12,795 |
| examples/lantern-vale | models/hero.vox | 132 | 3,790 | 718 | 18.9 % | 3,790 |
| fx-after | models/puppet2d.glb | 180 | 5,989 | 1,010 | 16.9 % | 14,592 |
| templates/fx-showcase | models/puppet2d.glb | 180 | 5,989 | 1,010 | 16.9 % | 14,592 |
| examples/meridian-loop | models/ghost.glb | 124 | 5,228 | 834 | 16.0 % | 5,228 |
| me-after | models/ghost.glb | 124 | 5,228 | 834 | 16.0 % | 5,228 |
| fx-after | models/walker.glb | 108 | 4,422 | 672 | 15.2 % | 13,473 |
| game-after | models/walker.glb | 108 | 4,422 | 672 | 15.2 % | 13,473 |
| templates/fx-showcase | models/walker.glb | 108 | 4,422 | 672 | 15.2 % | 13,473 |
| templates/game | models/walker.glb | 108 | 4,422 | 672 | 15.2 % | 13,473 |
| examples/meridian-loop | models/wheel.glb | 144 | 4,524 | 687 | 15.2 % | 4,524 |
| me-after | models/wheel.glb | 144 | 4,524 | 687 | 15.2 % | 4,524 |
| examples/meridian-loop | models/ghost-wheel.glb | 144 | 4,540 | 687 | 15.1 % | 4,540 |
| me-after | models/ghost-wheel.glb | 144 | 4,540 | 687 | 15.1 % | 4,540 |
| examples/meridian-loop | models/car.glb | 196 | 8,416 | 1,089 | 12.9 % | 8,416 |
| me-after | models/car.glb | 196 | 8,416 | 1,089 | 12.9 % | 8,416 |
| examples/rumpus-isle | models/tile.glb | 72 | 3,177 | 383 | 12.1 % | 3,177 |
| ru-after | models/tile.glb | 72 | 3,177 | 383 | 12.1 % | 3,177 |
| fx-after | models/window-room.glb | 96 | 3,556 | 416 | 11.7 % | 3,556 |
| templates/fx-showcase | models/window-room.glb | 96 | 3,556 | 416 | 11.7 % | 3,556 |
| examples/rumpus-isle | models/podium.glb | 108 | 3,811 | 424 | 11.1 % | 3,811 |
| ru-after | models/podium.glb | 108 | 3,811 | 424 | 11.1 % | 3,811 |
| examples/lantern-vale | models/pine.vox | 288 | 24,572 | 1,915 | 7.8 % | 24,572 |
| examples/lantern-vale | models/ruins.vox | 162 | 13,611 | 979 | 7.2 % | 13,611 |
| fx-after | models/cornell.glb | 30 | 3,245 | 211 | 6.5 % | 3,245 |
| templates/fx-showcase | models/cornell.glb | 30 | 3,245 | 211 | 6.5 % | 3,245 |
| examples/kestrel-stair | anims/humanoid.glb | 0 | 2,528 | 16 | 0.6 % | 56,462 |
| fx-after | anims/humanoid-clips.glb | 0 | 2,544 | 16 | 0.6 % | 15,120 |
| templates/archetypes/third-person | anims/humanoid-clips.glb | 0 | 2,544 | 16 | 0.6 % | 15,120 |
| templates/fx-showcase | anims/humanoid-clips.glb | 0 | 2,544 | 16 | 0.6 % | 15,120 |

  Small models sit at 6 to 25 % of their blob and cost a few kB; the ones
  that cost a download (the dropship 108,384 B, the trickster 172,765 B,
  meridian's track 55,190 B) are 30 to 49 %. A share alone would warn on
  rumpus's 31 kB island (46 %, 14 kB of BVH), so the warning needs both:
  over **30 %** of the blob and at least **16 KiB** (`BVH_WARN_BYTES`).
  Four models warn: dropship (38.6 %), fx-showcase's tree (49.3 %) and
  trickster (30.8 %), meridian-loop's track (30.2 %). The BVH stays on by
  default, as decided.
- **Change.** og-cook `cook::stats`: `budgets.bvhShare` (default 30, a
  project moves it in `budgets { }`), the model entry gains `bytes` and
  `bvhShare` (percent, a tenth), `asset_warnings` and each asset's
  `warnings` with `totals.warned`; `og cook --report` prints each warning
  ("og cook: warning: models/dropship.glb: its raycast BVH is 108384 B,
  38.6 % of the model's 280686 B (warned past 30 %); `model { raycast
  false }` ..."); `og doctor --project` reports it as a `bvh-share`
  warning (it does not fail the doctor: it is not a budget flag).
- **Test.** og-cook `stats::tests::a_model_warns_when_its_bvh_passes_the_share_and_the_floor`
  (the dropship's numbers warn at 38.6 %, 30.0 % and under do not, the
  floor holds a small model, a project's share moves it); it does not
  compile on the old code. Read by hand: `og cook templates/game --report`
  prints the dropship's warning, `og doctor --project templates/game
  --json` has it as a `bvh-share` finding.
- **Shared files.** og-cook `cook/stats.rs` (the report), og-cli `cook.rs`
  and `doctor_project.rs`; docs/design.md §3.4 (doctor) and the og-game
  skill §21.

### 4. The 2D host without the 3D world's renderer (`profile2d`): the 3D WGSL and the subsystems that draw it

The row's "3D WGSL sources in the 2D host" and its first per-profile
render trim are one change: the text is only droppable where the code
that compiles it never runs, so the subsystems go with it.

- **Which subsystems.** Those no 2D game of the repository uses and that
  draw only in a 3D world: terrain (`terrain.rs`, `terrain_tiles` draws),
  planar water (`og.Water`; `og.Water2D`'s strips stay), the 3D fluids,
  the scattered copies and volumetric fog (already off for an orthographic
  camera). The lit mesh path (pbr, physical, shadows, area lights, IBL,
  lightmaps, GPU culling) stays: a 2D game may draw meshes, and it is
  most of the 3D WGSL (the 3D-only text I listed at first, 156,617 B,
  is mostly that path).
- **Change.** og-render feature `profile2d` (og-runtime's and
  og-host-web's `profile2d` turn it on): `WORLD_3D` is false; the five
  subsystems' own WGSL files come through `wgsl3d!`, empty there; the
  frame takes no water (`scene_water`), no volumetric fog, no 3D fluids,
  no terrain draws or bounds and no scatter batches from the scene, and
  `warm` skips their pipelines; so their per-frame code is dead and their
  shader text unreferenced. Every one of their pipelines was already
  deferred (made on first use), so none is made from an empty module.
  `RenderCapabilities.world3d` (new, api.ogt; `og api gen` run: the .d.ts,
  the schema and the wiki, whose pages moved by one IDL line) says which.
  The 2D publish and lanes variants are built with it
  (`scripts/wasm-crates.txt`, `packages/engine/scripts/build.sh`).
- **Measured** (2D publish, after item 2): 12,115,631 → 11,971,544 B
  (−144,087 B), brotli 3,280,554 → 3,241,126 B (**−39,428 B**). Per crate:
  og_render −106,523 B, data −48,763 B (12 WGSL files absent: 45,129 B of
  shader text; 61 of 73 remain), og_runtime +10,749 B and og_cook
  +4,853 B (inlining moved). The 3D publish module is unchanged in kind
  (12,753,430 B, brotli 3,456,521 B: +255 B raw, −999 B brotli, inlining
  noise from the constant branches).
- **Test.** og-render `tests/profile2d.rs` (`--features profile2d`, on a
  GPU: lavapipe here): a lit ball with planar water and volumetric fog
  renders the same pixels as the ball alone. The same test without the
  feature fails ("the water and the fog drew"), which is what it guards.
- **Behaviour to judge.** A 2D game that puts `og.Water`, a terrain,
  scattered copies or 3D fluids in its world draws them in `og dev` and the
  native host (both have every subsystem) and not in its publish build.
  No 2D project in the repository does; `RenderCapabilities.world3d` says
  so at run time, but nothing warns at `og build --publish`. A check there
  (the cooked scenes' components against the profile) would be the
  platform-facing guard; it is not in this branch.
- **Shared files.** og-render `lib.rs` (`wgsl3d!`, `WORLD_3D`, the frame's
  gates, `warm`), `Cargo.toml` features of og-render, og-runtime and
  og-host-web, `scripts/wasm-crates.txt`, `packages/engine/scripts/build.sh`,
  og-runtime `api.rs` (the capability), docs/web-export.md §Variants,
  docs/ci.md (the variants' feature lists).

## The table after the trims (the branch's head)

Size gate numbers on the shipping `wasm` profile, measured after each
item on the same machine and toolchain:

| Step | 3D after wasm-opt | 3D brotli | 2D after wasm-opt | 2D brotli |
|---|---:|---:|---:|---:|
| ad903b18 (before) | 13,045,922 | 3,532,992 (84.2 %) | 12,408,297 | 3,357,559 (80.1 %) |
| 1. no GPU timestamp queries | 13,033,282 | 3,529,527 (−3,465) | 12,395,677 | 3,353,116 (−4,443) |
| 2. probe tool members compiled out | 12,753,175 | 3,457,520 (−72,007) | 12,115,631 | 3,280,554 (−72,562) |
| 3. (cook report only) | | | | |
| 4. 2D host without the 3D world | 12,753,430 | 3,456,521 (−999) | 11,971,544 | 3,241,126 (−39,428) |
| **total** | **−292,492** | **−76,471 (82.4 %)** | **−436,753** | **−116,433 (77.3 %)** |

Per crate, before and after (twiggy shallow bytes):

| Crate | 3D ad903b1 | 3D after |
|---|---:|---:|
| og_runtime | 1,783,546 | 1,474,495 (−309,051) |
| [data] | 1,517,360 | 1,501,008 (−16,352) |
| og_render | 1,040,886 | 1,007,074 (−33,812) |
| og_host_web (exports) | 886,782 | 1,127,012 (+240,230) |
| og_physics | 827,146 | 827,138 (−8) |
| rapier3d | 826,166 | 826,166 |
| parry3d | 773,619 | 773,619 |
| naga | 662,011 | 662,061 (+50) |
| wgpu | 431,909 | 427,149 (−4,760) |
| std | 430,368 | 410,219 (−20,149) |
| wgpu_core | 374,776 | 373,703 (−1,073) |
| og_ui | 331,342 | 331,637 (+295) |
| og_ecs | 257,364 | 225,502 (−31,862) |
| og_anim | 252,149 | 252,149 |
| serde_json | 220,102 | 194,199 (−25,903) |
| wgpu_hal | 191,807 | 191,816 (+9) |
| og_probe | 180,985 | 99,427 (−81,558) |
| bincode | 169,239 | 169,236 (−3) |
| og_core | 158,957 | 158,549 (−408) |
| og_cook | 156,204 | 152,591 (−3,613) |
| og_nav | 150,988 | 150,988 |
| [unnamed] | 142,613 | 140,862 (−1,751) |
| ttf_parser | 130,565 | 130,565 |
| rustybuzz | 109,407 | 109,407 |
| og_host_web | 101,827 | 100,109 (−1,718) |
| og_audio | 94,928 | 94,928 |
| og_ai | 80,986 | 80,986 |
| js_sys | 77,939 | 77,956 (+17) |
| og_timeline | 72,110 | 72,110 |
| [wasm structure] | 62,497 | 62,082 (−415) |
| og_input | 48,485 | 48,485 |
| indexmap | 48,385 | 48,385 |
| og_tilemap | 47,078 | 47,078 |
| og_net | 40,293 | 40,293 |
| og_tween | 34,871 | 34,871 |
| og_i18n | 32,771 | 32,899 (+128) |
| og_lanes | 32,369 | 32,369 |
| glam | 29,303 | 29,303 |
| wgpu_types | 28,828 | 28,828 |
| miniz_oxide | 26,124 | 26,040 (−84) |
| codespan_reporting | 21,414 | 21,587 (+173) |
| (the rest, under 20,000 B) | 159,146 | 158,272 (−874) |
| **module** | **13,045,645** | **12,753,153** (−292,492) |

| Crate | 2D ad903b1 | 2D after |
|---|---:|---:|
| og_runtime | 1,776,596 | 1,478,334 (−298,262) |
| [data] | 1,506,444 | 1,441,356 (−65,088) |
| og_render | 1,041,109 | 900,746 (−140,363) |
| og_host_web (exports) | 885,300 | 1,126,282 (+240,982) |
| rapier2d | 713,382 | 713,382 |
| naga | 661,999 | 662,048 (+49) |
| og_physics | 645,248 | 645,240 (−8) |
| parry2d | 457,767 | 457,767 |
| wgpu | 431,909 | 424,617 (−7,292) |
| std | 427,039 | 404,612 (−22,427) |
| wgpu_core | 374,776 | 373,719 (−1,057) |
| og_ui | 331,343 | 331,639 (+296) |
| og_ecs | 257,326 | 225,468 (−31,858) |
| og_anim | 252,240 | 252,150 (−90) |
| serde_json | 220,103 | 194,199 (−25,904) |
| wgpu_hal | 190,844 | 190,853 (+9) |
| og_probe | 180,983 | 99,426 (−81,557) |
| bincode | 173,974 | 173,971 (−3) |
| og_core | 158,959 | 158,551 (−408) |
| og_cook | 156,058 | 157,313 (+1,255) |
| og_nav | 150,987 | 150,974 (−13) |
| [unnamed] | 145,930 | 144,183 (−1,747) |
| ttf_parser | 130,565 | 130,565 |
| rustybuzz | 109,409 | 109,409 |
| og_host_web | 101,825 | 99,862 (−1,963) |
| og_audio | 94,930 | 94,930 |
| og_ai | 80,988 | 80,988 |
| js_sys | 77,937 | 78,978 (+1,041) |
| og_timeline | 72,110 | 72,110 |
| [wasm structure] | 62,490 | 62,033 (−457) |
| og_input | 48,485 | 48,485 |
| og_tilemap | 47,051 | 47,078 (+27) |
| indexmap | 46,772 | 46,772 |
| og_net | 40,293 | 40,293 |
| og_tween | 34,873 | 34,873 |
| og_i18n | 32,771 | 32,899 (+128) |
| og_lanes | 29,942 | 29,924 (−18) |
| wgpu_types | 28,828 | 28,828 |
| glam | 27,027 | 26,282 (−745) |
| miniz_oxide | 26,124 | 26,040 (−84) |
| codespan_reporting | 21,420 | 21,593 (+173) |
| (the rest, under 20,000 B) | 153,864 | 152,495 (−1,369) |
| **module** | **12,408,020** | **11,971,267** (−436,753) |

The og_host_web (exports) growth is inlining, not new code: with the
probe's dispatchers smaller, wasm-opt folds runtime bodies (most of what
og_runtime lost) into the export shims.

## Left out, and why

- **The BVH loader where no model raycasts.** The table puts it at about
  3 kB raw (`adopt_cooked_bvh` 1,366 B, `BvhNodeRow` 264 B, the
  `bvh_nodes`/`bvh_tris` decoding inlined into `Bundle::model`, 19,046 B
  with the rest of the model's decoding), around 1 kB brotli. Cutting
  it needs a build that knows no model raycasts, i.e. a per-game engine
  module, and row 99's first step is the opposite: one content-addressed
  engine wasm per version shared by every game. Without the loader a game
  that raycasts would build each BVH on its first raycast (the dropship's
  16,512 triangles), the stall the cooked BVH exists to remove. The cook
  side carries the choice instead (item 3: the warning and `raycast
  false`, which saves the game's own download, 108 kB for the dropship).
- **og-runtime's terrain in the 2D host.** `og_runtime::terrain` and
  `terrain_stream` are 56,374 B raw in the 2D module and still built:
  they load heightfields for `og.terrain.sample` and the physics
  heightfield too, so cutting them means a capability-reported refusal in
  the terrain module's API; the renderer's half went in item 4. The next
  per-profile step, with the 3D lit path's own share measured first.
- **The lanes modules.** Not measured per crate here; they carry the same
  features (the 2D lanes module has `profile2d`) and the size gate run
  below gives their numbers.
- **The panic and assertion strings** (`.rodata.1`, 199,565 B, the
  largest data segment) and wgpu's and naga's error strings: not in the
  row; `panic = "abort"` with `panic_immediate_abort`-style trimming
  would be a toolchain decision (nightly `-Z build-std` features), worth
  a measurement in the next size round.

## For the coordinator to decide

1. **Item 2's command channel in publish:** a timeline event key or an
   og.ai command naming a tool action (`dump_world`, `get`, `help`, ...)
   now logs a refusal in a publish build instead of writing an output no
   one reads. No game here does that.
2. **Item 4's dev/publish divergence for 2D games:** og.Water, terrain,
   scattered copies and 3D fluids draw in `og dev` and natively but not in
   a 2D publish build. Whether `og build --publish` should refuse a 2D
   project whose scenes hold them (a check in og-cli) is open.
3. **`bvhShare` 30 % with a 16 KiB floor** (item 3) warns on four models
   of the repository (dropship, fx-showcase's tree and trickster,
   meridian-loop's track); whether any of them should say
   `raycast false` is each game's call (fx-showcase's geometry station
   raycasts the terrain, not the models).
4. Reference frames: none should move (publish-only and 2D-publish-only
   changes; the frame's code under the default features is unchanged but
   for `pick` no longer running the views' id passes, which never fed the
   main camera's answer).
