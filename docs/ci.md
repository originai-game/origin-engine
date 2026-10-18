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
`taiki-e/install-action@nextest` (latest) and the script installs when
absent.

| Tool | Version | Pinned by |
|---|---|---|
| Rust (rustc, rustfmt, clippy; wasm32 and x86_64-linux-gnu targets) | 1.99.1, edition 2024, `rust-version` 1.99 | `rust-toolchain.toml`, `Cargo.toml` `[workspace.package]` |
| wgpu / naga | 30.0.1 | `Cargo.toml` `[workspace.dependencies]` |
| wasm-bindgen (crate and CLI), js-sys, web-sys, wasm-bindgen-futures | 0.2.129, 0.3.106, 0.3.106, 0.4.79 | `crates/og-host-web/Cargo.toml` (exact), the CLI from `Cargo.lock` |
| binaryen (`wasm-opt`) | 133 | `BINARYEN_VERSION` in `scripts/install-tools.sh` (release archive on Linux and macOS) |
| bun | 1.4.2 | `BUN_VERSION` in `scripts/install-tools.sh` |
| TypeScript (type checking only; Bun bundles) | 7.0.2 | `packages/engine/package.json` `typecheck`, `crates/og-idl/tests/idl.rs`, the templates' `devDependencies` |
| cargo-nextest | 0.9.148 here | not pinned |
| sccache (the rustc wrapper, `.cargo/config.toml`; not used in CI) | 0.14.0 | `SCCACHE_VERSION` in `scripts/install-tools.sh` |
| cargo-zigbuild, zig (`build.sh --linux-x64`) | 0.23.4, 0.17.0 here | not pinned; outside the gate |
| Chrome for Testing (`make web-reference`, browser tests) | 150 | `CHROME_PATH`; outside the gate |

wgpu 31 is not on crates.io (2026-10-19: `cargo search wgpu` lists 30.0.1
as the newest), so wgpu and naga stay on 30.0.1; the upgrade is owed when
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
| `scripts/check.sh quick` | fmt clippy test-quick drift bun-quick | the loop while working; CI job `quick` on every PR | 19–26 s (test-quick 16–20 s for 225 tests, 256 with og-ui's and og-render's GPU tests) |
| `scripts/check.sh` (full) | fmt clippy wasm-build test wasm-opt size-gate drift bun-test determinism | before a change to rendering, animation, physics, the tick or the export is called done; CI job `gate` on every PR and push to main | `--all` 103 s (test 26 for 348 tests, wasm-opt 30, size-gate 16, bun-test 18); after an og-render change 196 s under load (wasm-build 42, test 46, wasm-opt 59); after a template-only change 41 s |

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
| web-reference (`make web-reference`; in no lane) | og-host-web's crates for wasm32, `packages/engine`, `templates/fx-showcase` | |

A change to `scripts/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`.cargo/` or `.config/` reruns every one of them. Each skip prints its
reason, the tree it rests on and its inputs; `--all` (`scripts/check.sh
--all`, `scripts/check.sh --all quick`, `scripts/check.sh --all test`) runs
every step and every test, and so does any run with `CI` set in the
environment: CI's lanes stay complete. A failed step clears its record. The
determinism step, bun-test and the non-GPU tests always run.

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

`og-tests::wasm` (og-render must compile for wasm32) takes the full lane's
`wasm-build` og-render artifact when it is newer than og-render's sources,
so it needs no second wasm32 target directory.

Before the lanes (2026-10-18, E1 tree) the single gate took 310 s warm:
`cargo test` serial 111 s, `packages/engine`'s `bun run test` 156 s of which
a release build of `og`, the addon and two wasm variants was most. Four
agents building the same crates into private `target/agent-*` directories
had grown `target/` to 143 GB.

## What runs where

| Step | Command | Lane | CI `quick` (PR) | CI `gate` (push to main, PR) | CI `perf-nightly` (03:17 UTC, manual) |
|---|---|---|---|---|---|
| `fmt` | `cargo fmt --all --check` | both | yes | yes | |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | both | yes | yes | |
| `test-quick` | `cargo nextest run --workspace --exclude og-tests` (`cargo test` without nextest); og-ui's and og-render's GPU tests left out when their inputs did not change (§Lanes) | quick | yes, on lavapipe | | |
| `test` | `cargo nextest run --workspace && cargo test --workspace --doc` (`cargo test --workspace` without nextest); gfx tests in `og-tests`, left out with og-ui's and og-render's when their inputs did not change (§Lanes). Runs after `wasm-build`, whose og-render artifact `og-tests::wasm` takes when it is newer than og-render's sources (else that test runs `cargo check --target wasm32-unknown-unknown -p og-render` itself) | full | | yes, on lavapipe | |
| `wasm-build` | `cargo build --release --target wasm32-unknown-unknown -p <crate> --lib` for `scripts/wasm-crates.txt`, plus the `publish` feature variant of crates marked `publish`; copies to `target/og-wasm/raw/` | full | | yes | |
| `wasm-opt` | `wasm-bindgen --target web --out-name origin-engine` (when the module uses it) then `wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --low-memory-unused` (rustc links wasm32 stack-first, so the low 1 KiB is only the bottom of an overflowing stack); `opt.txt` lists the shipping (publish) variants, `opt-all.txt` every one | full | | yes | |
| `size-gate` | `scripts/size-gate.sh`: brotli `-q 11` sum of `opt.txt` ≤ `wasm_core_brotli_bytes` | full | | yes | |
| `drift` | `og api check` (generated `.d.ts`, wiki, JSON schema, Rust glue vs `crates/og-api/api.ogt`), the dev `og` the test step built | both | yes | yes | |
| `bun-quick` | `bun run test` in `packages/knowledge` and `packages/create` (no native build) | quick | yes | | |
| `bun-test` | `bun run test` in each `packages/*` with a `test` script (the engine's `typecheck` also checks the six templates' `src/` against the generated declarations, `packages/engine/tsconfig.templates.json`; their playtests run under bun in `og test`); `OG_GATE=1` makes `packages/engine/scripts/build.sh` take the dev addon and `og` and the gate's wasm instead of compiling release and wasm again (below) | full | | yes | |
| `determinism` | `tools/wasm-determinism/run.sh`: the same math digest and world hash on the host and on wasm32 scalar and simd128 under node | full | | yes | needs node |
| perf | `cargo test -p og-tests --release -- --ignored perf` | `make perf` | | | yes |
| `web-reference` | `bun tests/web/reference.ts` and `--webgl` in `packages/engine`: templates/fx-showcase's five stations, then `bun tests/web/reference-e5.ts` and `--webgl` for its eight E5 stations (each a fresh run like its playtest), in headless Chrome, compared with `references/webgpu`/`webgl2` or else `metal` under og-tests' tolerance (below); path-aware (§Lanes) | `scripts/check.sh web-reference` (`make web-reference`), in no lane | | | |

`cargo nextest` runs test binaries in parallel, one process per test.
Fixtures several tests read are made once across their processes under a
file lock (og-cook's `tests/support`: the template and the rigged models,
keyed by the test executable and the sources); `.config/nextest.toml` gives
every slot to the tests that assert a wall-clock bound (and runs them last),
two slots to the determinism runs that step two worlds or renderers at once,
four at a time to the GPU-bound packages, and starts og-tests, the cooks,
the pile and the walk first. `scripts/install-tools.sh` installs it; without
it the steps fall back to `cargo test`.

CI runs on `ubuntu-latest`. The Rust toolchain comes from
`rust-toolchain.toml` (channel, rustfmt, clippy, the wasm target) through
`actions-rust-lang/setup-rust-toolchain`, which also caches `~/.cargo` and
`target/`. `scripts/install-tools.sh` installs wasm-bindgen-cli at the
version `Cargo.lock` pins, binaryen (`wasm-opt`), brotli and bun, skipping
any already present at the right version; the tools are cached under
`~/.local`. `mesa-vulkan-drivers` gives wgpu a Vulkan adapter (lavapipe) for
headless gfx tests.

On failure CI uploads `target/evidence/**` (gate summary, size report, gfx
actual and diff frames, perf reports), the wasm manifests and any
`*.diff.png` as the `evidence-<run>` artifact. The size report and summary
are uploaded on every run as `size-gate-<run>`.

## Running the gate locally

```
scripts/install-tools.sh             # once; macOS uses Homebrew for brotli
scripts/check.sh quick               # the quick lane (make quick)
scripts/check.sh                     # the full lane, stops at the first FAIL
scripts/check.sh --all               # the full lane with nothing skipped (what CI runs)
scripts/check.sh clippy test         # selected steps, in the order given
scripts/check.sh wasm-build wasm-opt size-gate   # the size gate (make wasm)
scripts/check.sh --list
scripts/check.sh web-reference       # the browser tiers (make web-reference); needs Chrome
make check                           # the same through the Makefile
```

The path-aware state is `<target>/og-gate/` (the green trees) and
`<target>/og-wasm-cache/` (wasm-opt and brotli by content); both are kept
across runs and are only ever bypassed with `--all`, never deleted to force
a step.

One gate per target directory: `scripts/check.sh` (and a release
`packages/engine/scripts/build.sh`) holds `<target>/og-gate.lock` for its
whole run (`scripts/lock.sh`: a mkdir lock with the holder's pid, taken over
when that pid no longer runs a gate); a second one prints `another gate
(pid N: …) is running on <target>, waiting` and starts when the first ends.

Each step prints `==> [n/N] step`, its own output, then `PASS`, `FAIL` or
`SKIPPED` with its duration; the script exits non-zero on the first `FAIL`
and writes the per-step lines to `target/evidence/check-summary.txt`. The
script honours `CARGO_TARGET_DIR` for CI-style isolation, but agents working
in parallel share `target/` (AGENTS.md §Build and gate discipline: private
`target/agent-*` directories compile every dependency once per agent and are
not used). It exports `OG_EVIDENCE_DIR` (default `<target>/evidence`) and
`OG_WASM_OUT` (`<target>/og-wasm`) for test harnesses and `build.sh`.

## `packages/engine/scripts/build.sh` outside the gate

Standalone (`bun run build`, `bun run test` in `packages/engine`), build.sh
does what it does under the gate whenever the gate has built in this target
directory (`<target>/og-wasm/opt-all.txt` and the dev addon and `og` exist):
the dev addon and `og`, the gate's wasm, and one line saying so. It compiles
release and its own wasm only with `OG_RELEASE=1`, with `--linux-x64`, or
when the gate has not built here; a release build holds the gate lock.
`native/build-profile` records `dev` or `release`, and `scripts/pack.sh`,
which runs `OG_RELEASE=1 build.sh` with its own arguments first, packs only
`release`: what ships is always release.

## Switches in `scripts/budgets.txt` and `scripts/wasm-crates.txt`

- `wasm-crates.txt` lists the crates the wasm step builds. `required` crates
  build in one cargo invocation and fail the gate on error; `pending` crates
  build one at a time and a failure prints `WARN` (use it while a crate is
  being ported; the step prints a `NOTE` once it builds). A crate builds a
  `.wasm` only with `crate-type = ["cdylib", …]`; until one does, the wasm
  steps report "no wasm artifact" and pass.
- `wasm_artifacts optional|required`: flip to `required` when `og-host-web`
  becomes a cdylib, so the size gate can no longer pass by measuring nothing.
- `drift_check optional|required`: `og api check` counts as available when
  `og --help` lists an `api` subcommand. While it is not, the step prints
  `SKIPPED: og api check not available yet`; with `required` the same
  situation fails the gate. Flip it in the commit that lands `og api check`.

## Browser reference tiers

gfx references cover `metal` and `vulkan-cpu`; the browser tiers come from
`packages/engine/tests/web/reference.ts` (`make web-reference`): the dev
export of `templates/fx-showcase` on a 1280×720 canvas at DPR 1 in headless
Chrome for Testing, driven one tick and one render per `frame` call as
`og run --commands` drives the native host, each station captured at its
playtest screenshot's tick and compared with
`templates/fx-showcase/references/<webgpu|webgl2>/` (committed; `metal`
when a tier has none) at og-tests' default tolerance (32/255 per channel,
0.5 % outliers, RMSE 0.01). `--record` writes the tier's references. Before
the stations it sends `call("probe", "commands", {…})` with an object
through the engine Worker and requires the probe's refusal ("probe.commands:
script must be a string, not object") from the same Worker that then runs
them. It takes about 2 s a tier after the export (Apple M4 Pro, Chrome 150)
and passes on both tiers, but it is not in the gate: CI's machine has no
Chrome for Testing (a missing Chrome is an error, never a skip).

The same runner takes another project's playtest frames: `--project
templates/archetypes/topdown --script "<the frames playtest's commands
without screenshot and quit>" --shots start@30,crit@131,dash@168,east@216`
compares (or with `--record` writes) `<project>/references/<tier>/`. The
topdown archetype's four frames matched `metal` at RMSE ≤ 0.00009 on WebGPU
and ≤ 0.00005 on WebGL2 (2026-10-18) and are committed for both tiers.

Against their own references both tiers reproduce exactly (RMSE 0 on two
clean runs). Against `metal`, measured at the commit that recorded them
(2026-10-18):

| Station | webgpu outliers | webgpu RMSE | webgl2 outliers | webgl2 RMSE |
|---|---|---|---|---|
| post | 0.000 % | 0.00107 | 0.000 % | 0.00498 |
| lens | 0.007 % | 0.00188 | 0.007 % | 0.00186 |
| fx | 0.000 % | 0.00173 | 0.013 % | 0.00272 |
| 2d | 0.000 % | 0.00228 | 0.000 % | 0.00231 |
| stylised | 0.022 % | 0.00360 | 0.022 % | 0.00360 |

The E5 stations (sky, weather, water, shaders, controller, joints, rope,
fluid) each have a playtest of their own, so `tests/web/reference-e5.ts`
reads each playtest's station and wait from `tests/<station>.og.ts`. It runs
reference.ts once per station with `--script 0.5:station:<name> --shots
<name>@<tick>`, so the browser run is the same fresh run. Against their own
references both tiers reproduce exactly. Against `metal` (2026-10-19), every
station is within RMSE 0.0010 on WebGPU and 0.0027 on WebGL2, except water
on WebGL2 at RMSE 0.0216 (0.93 % outliers). That is expected: WebGL2 has no
depth-prepass reflection march, so its water reflects the prefiltered sky
cubemap (`crates/og-render/README.md` §E5).

The runner found four defects, all fixed in og-render before the tiers
were recorded (`crates/og-render/README.md` §The overlay and §Backend
tiers): the overlay blended encoded sRGB values on the WebGPU canvas's
`Bgra8Unorm` (the HUD panel was 87,89,99 against 123,128,135; RMSE up to
0.024); on WebGL2 TAA's `textureLoad` from a depth texture had no GLSL
form, the lit skinned pipeline needed 18 of the GL backend's 16 texture
slots, and the twelve-layer local shadow map was created as a cube-map
array, which left every shadowed light black. The og-tests `overlay` gfx
scene has no browser run: og-tests' scenes load their assets from disk and
read back synchronously, so they would need a wasm harness of their own;
fx-showcase's HUD and badge go through the same overlay pass.

## Updating a budget

Budgets come from `docs/design.md` §7 and live in `scripts/budgets.txt`.
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

Commit the new references with the change, and describe in the commit what
each moved frame now shows (AGENTS.md §Validation: a rendering change carries
a frame that was read and described). Capture on the backend tier the
reference belongs to; a lavapipe frame never replaces a Metal reference.

## When the size gate fails

1. Read the table `size-gate` printed (or `target/evidence/size-gate.json`
   from the CI artifact): which wasm grew and by how much.
2. Find the growth: `cargo tree -p og-host-web --target wasm32-unknown-unknown -e normal`
   for new dependencies, and `twiggy top target/og-wasm/opt/<name>.wasm`
   (`cargo install twiggy`) for the largest functions.
3. Prefer removing the cause: drop or feature-gate the dependency, disable
   default features, move debug-only code behind the non-`publish`
   configuration, avoid generic monomorphization blow-ups.
4. Only when the growth is intended and unavoidable, update the budget as
   above. Physics was expected to push past 4 MB; measured at 49ed935 it adds
   0.69 MB brotli (3D), 0.54 MB (2D) or 1.05 MB (both) to og-host-web's
   1.10 MB (`tests/evidence/M6/physics-wasm-size.json`), so it fits. If it
   ever does not, the answer is a feature-gated physics crate, not a budget
   raise (docs/phase-1.md §5).
