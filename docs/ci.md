# CI and the local gate

The gate is one script, `scripts/check.sh`, with two lanes. CI runs it
unchanged, so a green local run predicts a green CI run on the same
revision. The workflow (`.github/workflows/ci.yml`) only provisions the
machine. The rules the lanes rest on are AGENTS.md §Build and gate
discipline: one target directory, the dev profile for every native step,
tests that never run `cargo`.

## Toolchain

The one place the versions are stated. Each is pinned where the third
column says, and CI installs the same: the Rust toolchain from
`rust-toolchain.toml` (`actions-rust-lang/setup-rust-toolchain`), the rest
from `scripts/install-tools.sh`, except cargo-nextest, which CI takes from
`taiki-e/install-action` (latest) and the script installs when absent, and
node, which CI takes from `actions/setup-node`.

| Tool | Version | Pinned by |
|---|---|---|
| Rust (rustc, rustfmt, clippy, rust-src for the lanes wasm's `-Z build-std`; wasm32 and x86_64-linux-gnu targets) | 1.99.1, edition 2024, `rust-version` 1.99 | `rust-toolchain.toml`, `Cargo.toml` `[workspace.package]` |
| wgpu / naga | 30.0.1 | `Cargo.toml` `[workspace.dependencies]` |
| wasm-bindgen (crate and CLI), js-sys, web-sys, wasm-bindgen-futures | 0.2.129, 0.3.106, 0.3.106, 0.4.79 | `crates/og-host-web/Cargo.toml` (exact), the CLI from `Cargo.lock` |
| binaryen (`wasm-opt`) | 133 | `BINARYEN_VERSION` in `scripts/install-tools.sh` (release archive on Linux and macOS) |
| bun | 1.4.2 | `BUN_VERSION` in `scripts/install-tools.sh` |
| TypeScript (type checking only; Bun bundles) | 7.0.2 | `packages/engine/package.json` `typecheck`, `crates/og-idl/tests/idl.rs`, the templates' `devDependencies` |
| cargo-nextest | 0.9.148 here | not pinned |
| ShellCheck (CI lints `scripts/*.sh` with it; Ubuntu's 0.9 reports what 0.11 does not) | 0.11.0 | `SHELLCHECK_VERSION` in `scripts/install-tools.sh` (Homebrew's on macOS) |
| node (`determinism`'s wasm32 runs) | 24 in CI | `.github/actions/provision` |
| sccache (the rustc wrapper, `.cargo/config.toml`; not used in CI) | 0.14.0 | `SCCACHE_VERSION` in `scripts/install-tools.sh` |
| cargo-zigbuild, zig (`build.sh --linux-x64`) | 0.23.4, 0.17.0 here | not pinned; outside the gate |
| Chrome for Testing (`make web-reference`, browser tests) | 150 | `CHROME_PATH`; outside the gate |

wgpu 31 is not on crates.io (2026-10-19, and again 2026-10-21: `cargo
search wgpu` and `cargo search naga` list 30.0.1 as the newest), so wgpu
and naga stay on 30.0.1; the upgrade is owed when
31 ships, kept only if every committed reference frame passes without
re-recording. `cargo update` within semver had nothing left to move on
2026-10-19; the dependencies behind their latest are majors (bincode 3,
cpal 0.18, meshopt 0.7, miniz_oxide 0.9, nalgebra 0.35, syn 3 for our
derive) and tungstenite, pinned at `=0.30.0`.

The 2026-10 move (Rust 1.97.0 to 1.99.1 and edition 2024, binaryen 123 to
133, wasm-bindgen 0.2.127 to 0.2.129, TypeScript 5.9.3 to 7.0.2) changed
no committed reference frame on metal, vulkan-cpu, webgpu or webgl2 and
no determinism number. The shipping wasm (`og_host_web-publish`, after
wasm-opt) went from 9,438,293 bytes, 2,543,558 brotli, to 9,283,994 bytes,
2,518,068 brotli (-1.0 %); `tests/evidence/toolchain-2026-10.md` has each
step.

## Lanes

| Lane | Steps | When | Warm, Apple M4 Pro, another agent on the machine |
|---|---|---|---|
| `scripts/check.sh functional` | fmt clippy test drift bun-functional | after each batch of merges on main, and CI job `functional` on every PR: what a merge must pass (every test and the GPU reference frames, the generated API, the packages' tests on the native host); the web module is the last full lane's, and when a crate changed since, the tests that compare the two hosts' hashes skip (`OG_WASM_STALE`) and run in the full lane; no wasm build, size, determinism or browser tiers | 90–160 s |
| `scripts/check.sh quick` | fmt clippy test-quick drift bun-quick | the loop while working; CI job `quick` on every PR | 19–26 s (test-quick 16–20 s for 225 tests, 256 with og-ui's and og-render's GPU tests) |
| `scripts/check.sh` (full) | fmt clippy wasm-build test wasm-opt size-gate drift bun-test determinism | path-aware once per batch of merges on main, by the one session that runs gates (§When to run what); `--all` once at the end of a round, before its final commit and report; CI job `gate` on every PR and push to main | `--all` 84–101 s, steps concurrent (test 54–69 for 437 tests beside wasm-opt 54–69, then size-gate 22–25 beside bun-test 14–18), 121–135 s at a load average of 24–32; with nothing changed 30 s |

Both lanes are path-aware. A step that passes records the working tree it
passed on in `<target>/og-gate/green/<step>`: a git tree of the whole
working copy, untracked files included, written through a private index
(the real one is untouched). The next run compares that tree with the
current one (`git diff --name-only`) and skips what none of its inputs
changed:

| Skipped | When nothing changed in | Then |
|---|---|---|
| the GPU reference tests (nextest `-E 'not (package(og-tests) \| package(og-ui) \| package(og-render))'` in test and test-quick; `--exclude` under cargo test) | og-tests, og-ui, og-render and every workspace crate they build on (`cargo tree`, with dev-dependencies), `templates/ui-kit/assets` | the rest of the test step runs; the step's summary line says the GPU tests were skipped |
| wasm-build, wasm-opt, size-gate | `crates/` | the last outputs in `target/og-wasm` are reused (only when every file they list exists; a step that ran upstream in this invocation always runs the next) |
| drift | `crates/og-api` (`api.ogt`), og-idl and the crates it builds on, `crates/og-cli`, and every generated directory (`og_idl::OUTPUT_DIRS`), so a hand edit to a generated file is still caught | |
| web-reference (`make web-reference`; in no lane) | og-host-web's and og-tests-web's crates for wasm32 (og-tests-web takes og-tests' scenes, assets and references and og-ui's, whose references hold ui-kit's frames), `packages/engine`, `templates/fx-showcase`, `templates/looks`, `templates/ui-kit`, `templates/archetypes/third-person` | |

A change to `scripts/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`.cargo/` or `.config/` reruns every one of them. Each skip prints its
reason, the tree it rests on and its inputs; `--all` (`scripts/check.sh
--all`, `scripts/check.sh --all quick`, `scripts/check.sh --all test`) runs
every step and every test, and so does any run with `CI` set in the
environment: CI's lanes stay complete. A failed step clears its record.
bun-test and the non-GPU tests always run; determinism runs when
`tools/wasm-determinism` or a crate it links (og-core, og-ecs, og-physics,
og-anim, og-lanes and what they build on) changed. bun-test's slow
integration tests (packages/engine's `reload`, `reload-asset` and
`web-export`: og processes for minutes) run only under `--all` and in CI
(`OG_GATE_ALL`); a plain `bun run test` in the package runs them too.

The wasm profile depends on the lane: `--all` and CI build the shipping
`wasm` profile (`Cargo.toml`), so the size they record is the module that
ships; the path-aware lane builds `wasm-gate`, the same profile with 16
codegen units, which measured (2026-10-21, cold, Apple M4 Pro) wasm-build
293 → 205 s and wasm-opt 113 → 85 s with modules 5.3–5.9 % larger
(publish 2,947,818 → 3,106,805 B brotli, 2D 2,783,920 → 2,932,177, lanes
3,320,578 → 3,516,299), so a size gate the path-aware lane passes, the
shipping build passes by that margin. `OG_WASM_PROFILE=wasm|wasm-gate`
overrides; each profile has its own target subdirectory.

wasm-opt and size-gate also keep results by content in
`target/og-wasm-cache`: a raw module byte-identical to one already
optimised (keyed by its SHA-256, the wasm-bindgen and wasm-opt versions and
the flags) takes the earlier bindgen output and `-Oz` module, and brotli's
size of a module measured before is its recorded size. An edit under
`crates/` that does not reach the wasm (a test, a comment, native-only
code) then costs wasm-build's few seconds instead of 32 s of wasm-opt and
18 s of brotli. `--all` bypasses that cache too.

What the 2026-10 build-speed round changed (before 165–320 s for the full
lane, 83–154 s for quick; `tests/evidence/build-speed-2026-10.md` has every
measurement):

- **The dev profile has no debuginfo.** On macOS it stays in object files
  that rustc never removes, `target/debug/deps` had grown to 252,881 of
  them, and every process that opens a Metal device reads its executable's
  directory: each GPU test lost 1.4 s there. Panics still name file and
  line; `CARGO_PROFILE_DEV_DEBUG=line-tables-only` (which CI sets) gives
  backtraces with lines.
- **wgpu, wgpu-core, wgpu-hal, wgpu-types and naga are optimised in dev**
  (opt-level 3, with the physics and encoder crates), and the workspace
  crates build in 16 codegen units (an og-core edit's test build 10.4 s to
  6.6 s). `-j` below 12, nextest `--test-threads` 8 and a gpu group of 6
  measured no faster.
- **Tests:** og-cook's template and rigged models are cooked once per test
  build and read by every test process (a file lock in `tests/support`), so
  its template tests run in parallel; the walk's and the particle runs'
  second run is on a second thread; nextest starts the long poles first
  and the every-slot tests last. `cargo nextest run --workspace` went from
  112 s to 20–29 s.
- **og-api is three crates** (og-idl, og-api, og-runtime;
  `crates/og-api/README.md`): a doc-only IDL edit rebuilds nothing, a
  generator edit only og.
- **sccache** is the wrapper in `.cargo/config.toml` (`RUSTC_WRAPPER=`
  turns it off; CI does, caching `target/`), so a fresh worktree's test
  build takes about 70 s; parallel rounds reuse `scripts/worktree.sh`
  worktrees (AGENTS.md §Build and gate discipline).

The 2026-10-19 gate-speed round (`tests/evidence/gate-speed/`) took the
full lane with `--all` from 187–237 s to 84–101 s with every step and test
kept: the steps run concurrently as their inputs allow (§Running the gate
locally), and `packages/engine`'s bun test files run in parallel workers
(`bun test --parallel`: 31 s to 12 s; og polish on the topdown playtests,
7.5 s, is the longest file). One at a time (`OG_GATE_JOBS=1`) the same
gate takes 129 s.

`og-tests::wasm` (og-render must compile for wasm32) takes the full lane's
`wasm-build` og-render artifact when it is newer than og-render's sources,
so it needs no second wasm32 target directory.

Before the lanes (2026-10-18, E1 tree) the single gate took 310 s warm:
`cargo test` serial 111 s, `packages/engine`'s `bun run test` 156 s of which
a release build of `og`, the addon and two wasm variants was most. Four
agents building the same crates into private `target/agent-*` directories
had grown `target/` to 143 GB.

## What runs where

| Step | Command | Lane | CI `quick` (PR) | CI `functional` (PR) | CI `gate` (push to main, PR) | CI `perf-nightly` (03:17 UTC, manual) |
|---|---|---|---|---|---|---|
| `fmt` | `cargo fmt --all --check` | every | yes | yes | yes | |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | every | yes | yes | yes | |
| `test-quick` | `cargo nextest run --workspace --exclude og-tests` (`cargo test` without nextest); og-ui's and og-render's GPU tests left out when their inputs did not change (§Lanes) | quick | yes, on lavapipe | | | |
| `test` | `cargo nextest run --workspace && cargo test --workspace --doc` (`cargo test --workspace` without nextest); gfx tests in `og-tests`, left out with og-ui's and og-render's when their inputs did not change (§Lanes). Runs after `wasm-build`, whose og-render artifact `og-tests::wasm` takes when it is newer than og-render's sources (else that test runs `cargo check --target wasm32-unknown-unknown -p og-render` itself) | full, functional | | yes, on lavapipe | yes, on lavapipe | |
| `wasm-build` | `cargo build --profile wasm --target wasm32-unknown-unknown -p <crate> --lib` (`Cargo.toml`'s `[profile.wasm]`: release, the load-time crates at opt-level z) for `scripts/wasm-crates.txt`, plus the `publish` feature variant of crates marked `publish` in `target/wasm-<variant>/` (cargo 1.99 keeps one fingerprint per unit whatever its features, so two feature sets in one target dir recompiled og-probe, og-runtime and og-host-web on every run: 16.7 s and 12.2 s with nothing changed, 0.4 and 0.5 s apart; `tests/evidence/P2/tooling/wasm-build-relink.md`) and the lanes variant (`publish` with wasm atomics, `scripts/wasm-lanes.sh` in `target/wasm-mt/`, §The lanes wasm) of crates marked `publish lanes`; copies to `target/og-wasm/raw/` | full | | | yes | |
| `wasm-opt` | `wasm-bindgen --target web --out-name origin-engine` (`origin-engine-lanes` for a lanes variant; when the module uses it) then `scripts/wasm-opt.sh`, the one wasm-opt invocation (`-Oz --enable-bulk-memory --enable-nontrapping-float-to-int --low-memory-unused`; rustc links wasm32 stack-first, so the low 1 KiB is only the bottom of an overflowing stack), which `packages/engine/scripts/build.sh` also ships with (`--threads` adds `--enable-threads` for the lanes variant); `opt.txt` lists the shipping (publish and lanes) variants, `opt-all.txt` every one | full | | | yes | |
| `size-gate` | `scripts/size-gate.sh`: brotli `-q 11` of each file in `opt.txt` ≤ `wasm_core_brotli_bytes` (a page downloads one variant; the table also prints the sum and the largest) | full | | | yes | |
| `drift` | `og api check` (generated `.d.ts`, wiki, JSON schema, Rust glue vs `crates/og-api/api.ogt`), the dev `og` the test step built | every | yes | yes | yes | |
| `bun-quick` | `bun run test` in `packages/knowledge` and `packages/create` (no native build) | quick | yes | | | |
| `bun-functional` | `bun run test` in each `packages/*` with the last full lane's wasm (`OG_GATE=1`, as bun-test) when `target/og-wasm/opt-all.txt` exists, else only `packages/knowledge` and `packages/create` (CI's functional job builds no wasm, so it runs those two) | functional | | yes | | |
| `bun-test` | `bun run test` in each `packages/*` with a `test` script (the engine's files in parallel workers, `bun test --parallel`; the engine's `typecheck` also checks the six templates' `src/` against the generated declarations, `packages/engine/tsconfig.templates.json`; their playtests run under bun in `og test`); `OG_GATE=1` makes `packages/engine/scripts/build.sh` take the dev addon and `og` and the gate's wasm instead of compiling release and wasm again (below) | full | | | yes | |
| `determinism` | `tools/wasm-determinism/run.sh`: the same math digest and world hash on the host and on wasm32 scalar and simd128 under node, and the host release once more with lanes off and once with lanes on (failing when the lanes run did no parallel work) | full | | | yes (node from `actions/setup-node`) | |
| perf | `scripts/perf.sh`: `cargo test -p og-tests -p og-runtime --release --no-fail-fast -- --ignored perf` against this machine's baselines (og-tests: `crates/og-tests/perf/<tier>-<adapter>/`, og-runtime: extraction of 20,000 meshes, `crates/og-runtime/perf/<cpu>/`); `--measure` records instead, copies the numbers to `target/evidence/perf/` and restores the tree | `make perf` | | | | `--measure` (below) |
| `web-reference` | `bun tests/web/reference.ts` and `--webgl` in `packages/engine`: templates/fx-showcase's five stations, then `bun tests/web/reference-e5.ts` and `--webgl` for its eight E5 stations (each a fresh run like its playtest), then `bun tests/web/reference.ts --looks` and `--webgl --looks` for its looks station (37 presets in one run), then `bun tests/web/reference-gfx.ts` and `--webgl` for og-tests' gfx scenes and og-ui's four scenes, then `bun tests/web/reference-ui.ts` and `--webgl` for templates/ui-kit's 70 reference frames, then `bun tests/web/reference.ts --project templates/archetypes/third-person` and `--webgl` for Ember Vale's three frames (§Browser reference tiers), in headless Chrome, compared with `references/webgpu`/`webgl2` or else `metal` under og-tests' tolerance (below); path-aware (§Lanes) | `scripts/check.sh web-reference` (`make web-reference`), in no lane | | | | |

`cargo nextest` runs test binaries in parallel, one process per test.
Fixtures several tests read are made once across their processes under a
file lock (og-cook's `tests/support`: the template and the rigged models,
keyed by the test executable and the sources); `.config/nextest.toml` gives
every slot to the tests that assert a wall-clock bound (and runs them last),
two slots to the determinism runs that step two worlds or renderers at once,
four at a time to the GPU-bound packages, and starts og-tests, the cooks,
the pile and the walk first. `scripts/install-tools.sh` installs it; without
it the steps fall back to `cargo test`.

### CI

**There is no cloud CI during the build-out.** The GitHub Actions workflow
and its provisioning action were removed on 2026-10-21 (the owner's
decision: the shared runner was slower than the local gate, billed, and
added nothing the gate does not cover). The process is the local one: the
functional lane for every merge, `--all`, web-reference and the lavapipe
pass (or `scripts/wave-end.sh`) at a wave's end, by the one session that
merges (AGENTS.md §Build and gate discipline). What the workflow was when
it last ran green (every job in a `debian:trixie` container with `--init`
whose Lavapipe matched the og-lavapipe image, a shared provisioning
action, quick 12.5 min, functional 35 min, gate 60 min and the perf
nightly 145 min cold on a two-vCPU runner, `OG_BUN_TEST_TIMEOUT`,
`OG_TEST_TIME_SCALE` and `OG_GATE_JOBS=1` for that runner) is in the
history at 53b876f7 and 9d220790 and in docs/phase-1.md's ci-green
section; the environment variables stay honoured by the scripts. When the
repository is published, the workflow comes back from that history with
its push, pull_request and nightly triggers.

### og-tests' gfx scenes in the browser

`packages/engine/tests/web/reference-gfx.ts` renders og-tests' own gfx
scenes (`crates/og-tests/src/scenes.rs`) in headless Chrome:
`crates/og-tests-web` compiles them to a wasm-bindgen module (their glTF and
font assets embedded with `include_bytes!` on wasm32), opens each on an
OffscreenCanvas at 1280×720 on WebGPU or, with `--webgl`, on WebGL2
through wgpu's GL backend, draws its frames as the native harness does
(`og_tests::gfx::render_built`) and reads the target back, polling the
device between tasks (on WebGL2 `map_async` completes only on a poll). Each
frame is compared with `crates/og-tests/references/<webgpu|webgl2>/` when
that frame is committed, else with `metal`, under the scene's own tolerance
(`crates/og-tests/src/compare.rs`), and every frame's figures against
`metal` are kept in the report. A scene that does not apply to the tier
(`scenes::Applies`: `taa` and `shadows-local-8`, whose eight shadowed
lights are four on WebGL2, are full tier only, `msaa` the no-compute
tier's 4x path) is listed, not rendered; `taa-sharp` asks for
`Antialias::Taa` and is the WebGL2 tier's moving-camera TAA frame. `--scene a,b --time n`
times any scene it names, the shadows-aa cost scenes too (`COSTS`:
`point-shadows-cube`, `shadows-local-8-cube`, `shadows-local-8-paraboloid`,
`shadows-soft-off`, `aa-off`, `taa-motion`, built but not in `SCENES`). `--record` writes the tier's references;
`--time <n>` writes per-frame times of the perf scenes to
`crates/og-tests/perf/<tier>-chrome-<adapter>/` beside the native ones.
The perf-only city scenes (`city-street`, `city-aerial`) are timed only when
`--scene` names them. `--cpu-culling` makes WebGPU cull on the CPU
(`Culling::Cpu`; WebGL2 always does). `--perf-out <dir>` writes the timing
JSON to `<dir>/<tier>[-cpu-culling]/` instead, with the last frame's culling
counts (`tests/evidence/P3/culling/`). The native counterpart is
`og-gfx <scene> --frames <n>`, with `OG_GFX_CULLING=cpu|gpu|auto` for the
CPU path, the GPU path at any size, or the default (the GPU path from
16,384 instances). It
builds in `target/wasm-gfx` so the gate's og-render artifacts stay put.

On 2026-10-19 (Chrome for Testing 150, ANGLE Metal on the Apple M4 Pro,
`tests/evidence/P2/tooling/web-gfx-*.json`) all 47 applicable scenes pass on
each tier. WebGPU matches `metal` throughout and has no references of its
own. WebGL2 has its own for four scenes that differ from `metal` by
design: `msaa` (no other tier renders it), `showcase` and `overlay` (GPU
particles take the CPU path without compute; RMSE 0.0107 and 0.0103 against
`metal`) and `water-3d` (no screen-space reflection march, the sky cubemap
reflects instead; RMSE 0.0258). A WebGL2 reference is recorded only for a
difference the tier makes by design, never to absorb a defect. Each tier
takes 30 to 50 s after the module build.

The same module carries og-ui's four reference scenes as `og-ui/<name>`
(`sample-screen`, `text-glow`, `glass`, `widget-set`). They are built by
`crates/og-ui/tests/support/scenes.rs`, the file og-ui's native `gfx` and
`kit_gfx` tests build them with, included into og-tests-web by `#[path]`,
and judged against `crates/og-ui/references/<tier>/<name>.png`, else
`metal`. Neither browser tier has references of its own for them: they are
judged against `metal`, which they match exactly (RMSE 0) on both tiers
(2026-10-19), except `widget-set` at RMSE 0.00046, which is also what the
native metal run gives against that reference.

### og-ui's kit frames in the browser

`packages/engine/tests/web/reference-ui.ts` runs templates/ui-kit's
reference playtests in headless Chrome: `frames-desktop`, `frames-phone`,
`players`, `locales` and the twelve `skin-<pack>[-phone]` playtests, all of
them (72 frames: 23 desktop, 22 phone, 15 locale screens, 12 skin sheets of
21 cells). It bundles each playtest (`bundleGame`, as `og test` does), reads
its size, commands and screenshot times, and runs reference.ts once per
playtest with `--project templates/ui-kit --references
crates/og-ui/references --size <w>x<h> --script <the commands without
screenshot, quit and dump> --shots <file>@<round(60 t)>,…`, so each
browser run is the same fresh run as the native playtest. A skin playtest
adds `--sheet kit-skin-<pack>[-phone].png,<factor>,<cols>`: each shot is
box-shrunk (2 on desktop, 3 on a phone, as `tests/skins.ts` does) and
compared with its cell of the sheet; the gallery shot that only
proves the pack applied is not a cell. Frames are judged against
`crates/og-ui/references/<webgpu|webgl2>/`, else `metal`, at og-tests'
default tolerance (neither tier has references of its own, so every frame
and sheet is judged against `metal`), and the summary goes to
`packages/engine/dist/test-reference/ui/reference-ui-<tier>.json`, one
report per playtest beside it. `--only a,b` runs some playtests,
`--record` writes the tier's frames and sheets.

Both browser tiers are judged against `metal`'s 72 frames: per-tier
copies, recorded 2026-10-19 and reproduced exactly (RMSE 0) by both tiers,
were dropped as duplicates of `metal`, since no frame differs by the
tier's design. Against `metal` (Chrome for Testing 150, Apple M4 Pro,
2026-10-19) every frame passes on both tiers.
Eleven frames have outliers there, all at most 0.038 % (RMSE ≤ 0.0089,
`kit-desktop-settings`): the edges of the large MSDF titles ("Settings",
"Paused", "Shop", "Levels", "Toasts", the puzzle score), eleven edge
pixels on `idle` and the gallery's scrollbar thumb, whose coverage
differs by a fraction of a pixel. lavapipe, WebGPU and WebGL2
all agree there (no outliers between any two of them, RMSE ≤ 0.0019); the
difference is metal's. Every other frame is within RMSE 0.0025 of `metal`,
except the paper pack's phone sheet at 0.0040.
A tier takes 100 to 140 s (about 7 to 9 minutes while another heavy job
shares the machine), so the full set runs in `web-reference` rather than
a subset.

### The accessibility mirror in the browser

`packages/engine/tests/web/a11y.ts` (in `web-reference`, both tiers) runs
the dev export of templates/ui-kit on a 1280×720 canvas with the
reference page, one tick per `frame` call, on the settings screen, and
checks the shell's DOM mirror of `og.ui.a11y()` (src/web/a11y.ts): one
element per node in tree order, nested by parent, at each node's bounds,
with its role; the settings dialog, the section tabs, the Master slider,
the Mute switch and Done as ARIA; Chrome's own accessibility tree
(`Accessibility.getFullAXTree`) reading the switch and the slider; no tree
in the frame answers of a settled screen; a screen-reader-style
`element.click()` on Mute toggling it (dump_ui's `checked`, the mirror's
`aria-checked`, the UI's focus and the DOM focus on it); the Graphics tab
and Done clicked the same way, the "Settings saved" toast in the polite
live region; `prefers-contrast: more` (CDP's emulated media) changing the
frame and its end bringing it back; and the pause dialog arriving at full
size under `prefers-reduced-motion: reduce` where it otherwise scales in.
It writes `dist/test-a11y/out/a11y-<tier>.json`.

## Updating a budget

Budgets come from `docs/design.md` §7 and live in `scripts/budgets.txt`.
`wasm_core_brotli_bytes` is per download: a page downloads one engine
module (the 3D or the 2D publish variant by the game's profile, or the
lanes variant on an isolated page; docs/web-export.md §Variants), so each
is held to it and the size gate's figure is the largest of them
(`max_brotli_bytes` in `size-gate.json`, the `max` line of its table),
never the sum of every wasm the gate builds. The table prints each
variant's brotli size with its download and its share of the budget.
Change the number in its own commit, with the measured value from
`target/evidence/size-gate.json` and the reason in the message; a raise above
a design §7 target also changes design §7 in the same commit. The core budget
is 4 MB brotli; the design's 6 MB with physics and animation becomes a second
key (e.g. `wasm_full_brotli_bytes`) when a feature-gated build exists.

## Re-capturing gfx references

gfx tests in `og-tests` render a scene headless and compare it with a
committed PNG (per-pixel tolerance plus an RMSE threshold, references per
backend tier). On mismatch they write `<scene>.actual.png` and
`<scene>.diff.png` under `$OG_EVIDENCE_DIR/gfx/`. When a rendering change is
intended:

```
OG_CAPTURE=1 cargo test -p og-tests          # rewrite the references
git diff --stat tests/                       # see which frames moved
```

A scene with no reference on the adapter's tier is compared with
`references/metal/` and the test prints so (`gfx <scene>: no vulkan-cpu
reference …; comparing with metal's`), as every other reference harness
does (§Browser reference tiers); a scene on neither fails. So a scene
recorded on metal is held on lavapipe in CI at once, and its own
`vulkan-cpu` frame, when the tier differs by more than the tolerance, is
recorded at the wave's end (`scripts/wave-end.sh --record gfx/<scene>`).

Commit the new references with the change, and describe in the commit what
each moved frame now shows (AGENTS.md §Validation: a rendering change carries
a frame that was read and described). Capture on the backend tier the
reference belongs to; a lavapipe frame never replaces a Metal reference.

## When the size gate fails

1. Read the table `size-gate` printed (or `target/evidence/size-gate.json`
   from the CI artifact): which wasm grew and by how much.
2. Find the growth: `cargo tree -p og-host-web --target wasm32-unknown-unknown -e normal`
   for new dependencies, and `scripts/wasm-size-report.sh
   target/og-wasm/raw/<name>.wasm` for where the bytes are: the module
   optimised as it ships, then per crate and for the largest functions,
   each with its brotli alone beside its raw bytes (the optimised module in
   `target/og-wasm/opt/` keeps no names to read). Rank by brotli: generic
   copies compress to little more than one copy, so a raw table points at
   them first and they pay least (tests/evidence/P4/wasm-size/README.md).
3. Prefer removing the cause: drop or feature-gate the dependency, disable
   default features, move debug-only code behind the non-`publish`
   configuration, avoid generic monomorphization blow-ups.
4. Only when the growth is intended and unavoidable, update the budget as
   above. Physics was expected to push past 4 MB; measured at 49ed935 it adds
   0.69 MB brotli (3D), 0.54 MB (2D) or 1.05 MB (both) to og-host-web's
   1.10 MB (`tests/evidence/M6/physics-wasm-size.json`), so it fits. If it
   ever does not, the answer is a feature-gated physics crate, not a budget
   raise (docs/phase-1.md §5).
