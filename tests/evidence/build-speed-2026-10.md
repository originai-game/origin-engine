# Build speed 2026-10

The inner-loop round of 2026-10-19 (edd9736..HEAD on main), measured on the
Apple M4 Pro (12 cores, 24 GB) while another agent ran platform validation
and an iOS simulator on the same machine: the load average is given with
each number, and it moved between 7 and 35 during the round, so a number is
indicative within about 20 %. Every "before" is at edd9736, every "after" at
the commit that finished the piece (or HEAD).

## Where the time went (before)

- **Not the compiler.** A warm content edit rebuilt the test build in
  3.5–15.8 s and clippy in 3–14 s. The quick lane after an og-render edit
  was 154 s, of which test-quick 134 s; the full lane 287 s, of which the
  test step 177 s, wasm-opt 31 s, size-gate 17 s, bun-test 32 s.
- **The tests.** `cargo nextest run --workspace`: 112–114 s for 348 tests,
  317 s of test time. og-tests alone was 167 s of it behind the 4-slot gpu
  group and started at 68 s (alphabetical order). og-cook's template tests
  ran one at a time (45 s) and each cooked the template again (a `OnceLock`
  is per process, nextest runs one process per test). The 500-body pile
  held every slot for 10 s though it asserts no wall-clock bound.
- **macOS and the target directory.** `target/debug/deps` held 252,881
  object files: dev debuginfo stays in `.o` files on macOS
  (split-debuginfo unpacked) and rustc never removes an incremental unit's
  old ones. Every process that opens a Metal device reads its executable's
  directory (`MTLCopyAllDevices` → IOSurface → `CFBundle` reading the main
  bundle's resources): `bloom_matches_its_reference` took 1.7 s from
  `target/debug/deps` and 0.3 s from a directory of its own. That was
  most of og-tests' 167 s. The first run of newly linked binaries also
  waits for the system's executable scan (12 s for `cargo nextest list`
  after a relink, 0.6 s after); adding the terminal under System Settings ›
  Privacy & Security › Developer Tools removes it, a machine setting this
  round did not change.
- **The hub.** `og api gen` rewrote every output even when unchanged, and
  the generated Rust repeated the IDL's prose, so a doc-only IDL edit
  rebuilt og-api, og-host-web and og; a generator edit (og-api's `gen/`)
  rebuilt the runtime and both hosts, the release wasm included (wasm-build
  37 s).
- **Fresh worktree.** A fresh `git worktree add` built the dev test build in
  67 s (sccache 74.6 % hits), not 10–15 min: sccache was already the wrapper
  (`~/.cargo/config.toml`). Its 51 misses (rapier2d/3d, parry, nalgebra,
  naga, wgpu-core: 17 s, 16 s, 12 s, …) are evictions from a full 10 GiB
  cache, not path-dependent keys: after `cargo clean -p rapier3d` in either
  checkout the rebuild is a hit.

## Touch-one-file rebuilds

`cargo test --workspace --no-run` (what both test steps build) then
`cargo clippy --workspace --all-targets -- -D warnings`, warm. "touch"
changes only the modification time; "edit" appends a public function.
Seconds, crates compiled in parentheses, load average after.

| Change | Before: test build / clippy | After: test build / clippy |
|---|---|---|
| og-core, touch | 21.4 (16) / 13.6, load 24 | 9.0 (18) / 5.0, load 12 |
| og-core, edit | 15.8 (16) / 13.8, load 15 | 10.0 (18) / 5.0, load 17 |
| og-ecs, touch | 11.4 (12) / 9.8, load 22 | 7.0 (13) / 4.3, load 15 |
| og-ecs, edit | 15.2 (12) / 13.2, load 11 | 7.3 (13) / 3.6, load 21 |
| og-render, touch | 9.2 (7) / 6.0, load 20 | 6.4 (7) / 3.1, load 15 |
| og-render, edit | 10.4 (7) / 8.3, load 11 | 6.5 (7) / 2.9, load 22 |
| og-ui, touch | 6.2 (5) / 5.4, load 19 | 4.6 (5) / 2.0, load 15 |
| og-ui, edit | 8.2 (5) / 7.0, load 14 | 4.1 (5) / 2.3, load 24 |
| runtime (og-api `src/runtime`, now og-runtime), touch | 3.2 (3) / 2.4 | 1.2 (2) / 1.2 |
| runtime, edit | 3.5 (3) / 3.1, load 18 | 2.1 (2) / 1.3, load 22 |
| `api.ogt` doc-only edit, `og api gen` then build | 0.6 + 2.9 (3: og-api, og-host-web, og) / 2.7 | 0.7 + 0.3 (0) / 0.5 |
| generator (og-api `gen/`, now og-idl), edit: wasm-build | 37 s (og-api, og-host-web release, two variants) | 3 s |
| generator, edit: test build / clippy | not measured | 1.3 (og-idl, og) / 0.6 |
| a template's TypeScript (`templates/game/src/main.ts`) | 0.2 (0) / 0.3; the full lane then ran everything: 287 s | 0.3 (0) / 0.3; full lane 41 s (below) |

"After" adds the 16 codegen units of 8a6baf5: measured alone, an og-core
edit 10.4 s to 6.6 s, og-ecs 7.0 s to 4.9 s, og-render 7.5 s to 4.3 s
(load 22–35); incremental off made og-render 7.5 s to 9.5 s and was not
kept.

## Test time

`cargo nextest run --workspace`, warm, binaries already scanned:

| | Before (edd9736) | After (HEAD) |
|---|---|---|
| wall | 112.1–113.9 s, load 9 | 29.0–29.3 s with fresh shared cooks (load 9–21), 20.3 s in the final `--all` gate (load 8–13) |
| test time (sum) | 317 s | 139–222 s |
| og-tests (92 tests) | 167 s, started at 68 s | 58 s, started at 0 |

The ten slowest tests before, in the before run and in the final `--all`
gate's run (where the long ones start together and share the machine), and
alone:

| Test | Before | After, in the run | After, alone | What changed |
|---|---|---|---|---|
| og-cook::animation skins_and_morphs_follow_the_cooked_vertices | 25.9 | 0.8 | 0.4–3.5 | grid lookup of each cooked vertex's source (same vertex as the full scan for all 20,839, checked once); the shared cook |
| og-cook::template cooking_twice_gives_identical_bytes | 13.9 | 18.0 | 9.5 | the shared cook replaces its own first cook; the one-thread cook it asserts against stays |
| og-cook::animation markers_come_from_extras_and_import_settings | 11.8 | 10.8 | 4.5 | none (its two cooks have their own settings); parallel with the rest |
| og-cook::animation cooked_clips_resample_to_the_gltf_sampler_within_1e_5 | 11.8 | 6.3 | 2.5 | the shared cook for its default-settings half |
| og-anim::walk a_3600_tick_walk_… | 10.6 | 13.3 (two slots) | 6.5 | its second run on a second thread |
| og-physics::pile the_pile_is_deterministic_… | 10.4 (every slot) | 17.8 (two slots) | 10.4 | two slots instead of all twelve |
| og-cook::template a_gltf_edit_recooks_exactly_its_outputs | 8.6 | 0.7 | 0.3 | copies the shared cook |
| og-tests::effects a_600_tick_gpu_particle_run_is_deterministic | 8.9 | 6.3 | 2.8 | second renderer on a second thread; wgpu at opt-level 3 |
| og-tests::effects a_600_tick_cpu_particle_run_is_deterministic | 8.7 | 6.3 | 2.7 | the same |
| og-cook::template cooked_models_render_the_gltf_reference_scene | 5.9 | 2.9 | 1.2 | the shared cook; the Metal device no longer reads 252,881 entries |

Every GPU test gained the same 1.4 s from the empty `deps` directory
(`bloom_matches_its_reference` 1.88 s to 0.35 s); wgpu, wgpu-core,
wgpu-hal, wgpu-types and naga at opt-level 3 took five og-tests tests from
13.4 s to 10.4 s. Tried and not kept: the image/png/deflate stack at
opt-level 3 (2.18 s against 2.23 s for four gfx tests), objc2 at opt-level 3
without its debug encoding checks (no change), a gpu group of 6 (32.3 s
against 29.3 s), og-tests started before the cook tests (no better under the
load of the time).

## Parallelism under load

| Setting | Measured | Kept |
|---|---|---|
| `cargo test --no-run -j 12 / 8 / 6` after an og-core edit | 10.2 / 9.4 / 9.6 s, then 8.9 / 9.5 / 9.2 s (load 9–28) | no: the default |
| `nextest --test-threads 12 / 8` | 27.7 / 27.5 s, then 27.9 / 29.1 s (load 16–19) | no: the default |
| codegen-units 16 for the workspace crates | above | yes |
| `incremental = false` | slower | no |

## Fresh worktree with sccache

`cargo test --workspace --no-run --timings` in a new worktree after a warm
main build, sccache 0.14.0 (`.cargo/config.toml` now names it; it was
already the wrapper through `~/.cargo/config.toml`):

| | Before | After (`scripts/worktree.sh`) |
|---|---|---|
| wall | 67.0 s, load 14–19 | 74.0 s, load 11–17 |
| sccache | 150 hits, 51 misses (Rust 69 %) | 151 hits, 50 misses (Rust 70 %) |
| units, sum of unit times | 342, 396 s | 346, 425 s |
| longest units | rapier2d 17.2, rapier3d 16.3, parry3d 12.1, nalgebra 10.7 s | rapier3d 27.7, rapier2d 23.6, naga 15.4, nalgebra 13.1 s |

The misses are evictions (the cache is at its 10 GiB limit); the
workspace's own crates are incremental in dev and never cached. Raising the
limit (`SCCACHE_CACHE_SIZE`, with `SCCACHE_DIR` on the 1.8 TB volume, since
the system volume had 45 GB free) is a machine setting shared with other
projects and was left to the owner.

## Gate

| Run | Before | After |
|---|---|---|
| quick lane after an og-render change | 154 s (test-quick 134 s), load 17 | 19 s (test-quick 16 s, 256 tests with og-ui's and og-render's GPU tests) |
| full lane after an og-render change | 287 s: fmt 1, clippy 10, wasm-build 7, test 177, wasm-opt 31, size-gate 17, drift 5, bun-test 32, determinism 7 (load 11–14) | 196 s: fmt 1, clippy 4, wasm-build 42, test 46, wasm-opt 59, size-gate 18, drift skipped, bun-test 25, determinism 1 (load 11–20) |
| full lane after a template-only change | as any change: 287 s | 41 s: wasm-build, wasm-opt, size-gate, drift and the GPU tests skipped; test 19, bun-test 19 |
| `scripts/check.sh --all`, end of the round | 165–320 s | 103 s: fmt 0, clippy 1, wasm-build 7, test 26 (348 tests, 20.3 s), wasm-opt 30, size-gate 16, drift 1, bun-test 18, determinism 2 (load 8–13) |
| wasm-opt + size-gate on a byte-identical raw module | 32 + 18 s | 0 + 0 s (target/og-wasm-cache) |

The skip demonstration (HEAD, `templates/game/src/main.ts` with one
comment line appended after a green `--all`):

```
path-aware: working tree c4fbd901ed26; a step none of whose inputs changed since it last passed is skipped (--all runs everything)
SKIPPED wasm-build (reusing target/og-wasm/raw): no input changed since it passed on tree 2a4f8f1fd5c8 (inputs: crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo .config scripts; --all runs it)
SKIPPED the GPU reference tests (package(og-tests) | package(og-ui) | package(og-render)): no input changed since it passed on tree 2a4f8f1fd5c8 (inputs: crates/og-core crates/og-core-derive crates/og-ecs crates/og-input crates/og-probe crates/og-render crates/og-tests crates/og-tilemap crates/og-ui templates/ui-kit/assets Cargo.toml Cargo.lock rust-toolchain.toml .cargo .config scripts; --all runs it)
SKIPPED wasm-opt (reusing target/og-wasm/opt): no input changed since it passed on tree 2a4f8f1fd5c8 (…)
SKIPPED size-gate (the wasm it measured is unchanged): no input changed since it passed on tree 2a4f8f1fd5c8 (…)
SKIPPED drift: no input changed since it passed on tree 2a4f8f1fd5c8 (inputs: crates/og-api crates/og-core crates/og-core-derive crates/og-idl crates/og-cli packages/engine/types packages/engine/schema packages/engine/src/generated crates/og-host-web/src/generated crates/og-host-node/src/generated packages/knowledge/wiki …)
gate PASSED (9/9 steps, 41s)
PASS fmt (1s)
PASS clippy (1s)
SKIPPED wasm-build (0s): no input changed since it passed on tree 2a4f8f1fd5c8
PASS test (19s): the GPU reference tests skipped, no input changed since it passed on tree 2a4f8f1fd5c8
SKIPPED wasm-opt (0s): no input changed since it passed on tree 2a4f8f1fd5c8
SKIPPED size-gate (0s): no input changed since it passed on tree 2a4f8f1fd5c8
SKIPPED drift (0s): no input changed since it passed on tree 2a4f8f1fd5c8
PASS bun-test (19s)
PASS determinism (1s)
```

## Frames and sizes

No reference frame was re-recorded. The gate's og-tests references on
metal, `scripts/check.sh web-reference` (fx-showcase on webgpu and webgl2,
RMSE 0 against their own references) and `og test` on templates/game,
ui-kit (17 suites), runner, match3, topdown and fx-showcase pass at HEAD.
The publish wasm after wasm-opt is 9,284,871 bytes, 2,518,249 brotli
(+877 and +181 against edd9736, 0.007 %; they appeared with the og-api
split, which moved the runtime into its own crate, and their cause was not
traced further).
