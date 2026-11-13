# The local gate

The gate is one script, `scripts/check.sh`; its GPU steps run on the GPU
host (§The GPU host), and there is no GitHub CI. Its standing rules are in
AGENTS.md §Building and testing; this document holds the operational
detail.

## Landing: a merge queue

Frames are judged on quality, never on their difference from a reference
(AGENTS.md §Principles): any change may move frames, and a moved reference
does not block a landing. The engine's own reference suites (og-tests,
og-tests-ui, the font packs, og-cook's compressed frames) report a frame
outside its tolerance as `REFERENCE MISMATCH` with its actual and diff
PNGs, and the gate prints every report after rust-gpu; they still fail a
crash, a missing reference, a frame of another size, a one-colour frame
where the reference is not (`og_tests::compare::broken`) and an unchecked
run. Fixture playtests are game projects: their `expect:frame:...:rmse`
stays a failure, as in any game. A reference is the last recorded frame; the GPU
integrator refreshes moved ones in batches. Inspect every new or
re-recorded frame for what it shows before committing it, including phone
layouts and non-default states. Populate screen fixtures with their supplied content and
art; keep missing-art cases as explicit placeholder tests. A reference that
records clipping, incorrect values or broken layout is a defect, even when
the comparison passes.

`scripts/land.sh` (AGENTS.md §Streams and landing) runs quick outside the
FIFO landing lock, which is per Git directory, and rechecks at most once
under it, with `OG_LANDING_RECHECK=1` giving the recheck priority over
other gates and measurements. The lock holder lands the waiting heads
(`og-landing/batch/`) as one stack; when it fails, bisection pushes the
passing prefix and hands the first failing head back with a marker in
`og-landing/failed/`. Main is pushed only after quick passed on exactly the
pushed tree, never with a force push. Conflicts confined to generated
files (`og_idl::OUTPUT_DIRS`, the UI theme) are regenerated; any other
conflict stops for the author. `python3 scripts/test-landing.py` exercises
concurrent landings against a disposable bare remote.

## Toolchain

The one place the versions are stated. Each is pinned where the third
column says: the Rust toolchain in `rust-toolchain.toml`, the rest
installed by `scripts/install-tools.sh` (cargo-nextest when absent).

| Tool | Version | Pinned by |
|---|---|---|
| Rust (rustc, rustfmt, clippy, rust-src; wasm32-unknown-unknown, wasm32-wasip1, x86_64-unknown-linux-gnu and x86_64-pc-windows-msvc targets) | nightly-2026-11-05 (1.101.0-nightly), edition 2024, `rust-version` 1.99 | `rust-toolchain.toml`, `Cargo.toml` `[workspace.package]` |
| wgpu / naga | 30.0.1 | `Cargo.toml` `[workspace.dependencies]` |
| AccessKit (native shared-frame accessibility schema) | 0.25.2 | `crates/og-ui/Cargo.toml` (exact; serde enabled, no OS adapter in the engine) |
| Parley / parley_engine, HarfRust, Skrifa (UI text) | 0.12.0 / 0.12.0, 0.12.0, 0.44.0 | `crates/og-ui/Cargo.toml` (exact compatible stack; default features off, `libm`, Parley `complex-scripts`, no system font discovery) |
| write-fonts (cook-only SFNT shaping tables) | 0.55.0 | `crates/og-cook/Cargo.toml` (exact, default features off; no runtime outlines in layout-only fonts) |
| wasm-bindgen (crate and CLI), js-sys, web-sys, wasm-bindgen-futures | 0.2.129, 0.3.106, 0.3.106, 0.4.79 | `crates/og-host-web/Cargo.toml` (exact), the CLI from `Cargo.lock` |
| rav1d (the AV1 decoder: native streams, og-cook's posters, the web fallback; BSD-2-Clause) | 1.1.0, 8-bit only, no assembly | `crates/og-video/Cargo.toml` |
| bun | 1.4.4 | `BUN_VERSION` in `scripts/install-tools.sh`; `tests/install/Dockerfile` and `tests/install/instabox.ts` install the same for the install test |
| bun-types (`bun:test`'s declarations for playtest files, vendored; moves with bun) | 1.4.4 | the first line of `packages/engine/vendor/bun-types/test.d.ts` (its header lists the local patches); `scripts/toolchain.test.ts` in bun-quick fails when it, this row and `BUN_VERSION` differ |
| TypeScript, tsgo (`check-conformance`'s comparison and `og check`'s fallback where tsrs has no package; no project or package depends on it) | 7.0.2 | `TSGO_VERSION` in `crates/og-cli/src/check.rs` (`scripts/tsrs-diff.sh` and `.agents/setup` read it) |
| tsrs (the type checker: `og check`, the check before `og run`/`test`/`build`/`dev`, `packages/engine`'s `typecheck`, og-idl's declaration test) | 0.18.0-ts7.1.0-dev.20260929 (TypeScript 7.1.0-dev.20260930.4) | `TSRS_VERSION` and the five SRIs in `scripts/install-tools.sh`; `packages/engine/package.json` `optionalDependencies` copy it (`check-conformance` holds them equal). It moves with a tsrs release, all of these in one commit, after `scripts/tsrs-diff.sh` passes on the new binary |
| cargo-nextest | 0.9.148 here | not pinned |
| ShellCheck (CI lints `scripts/*.sh` with it; Ubuntu's 0.9 reports what 0.11 does not) | 0.11.0 | `SHELLCHECK_VERSION` in `scripts/install-tools.sh` (Homebrew's on macOS) |
| node (`determinism`'s wasm32 runs) | 24 in CI | `.github/actions/provision` |
| sccache (the compiler cache, run by og-sccache-wrap; not used in CI) | 0.19.0 | `SCCACHE_VERSION` in `scripts/install-tools.sh` |
| og-sccache-wrap (the rustc wrapper `.cargo/config.toml` names) | this checkout's | `crates/og-sccache-wrap`, built and installed by `scripts/install-tools.sh` on every run |
| mold (Linux native linker) | distribution package | `.agents/setup`, `scripts/install-tools.sh`; `.cargo/config.toml` selects it |
| cargo-zigbuild, zig (`build.sh --linux-x64`) | 0.23.4, 0.17.0 here | not pinned; outside the gate |
| Chrome for Testing (`make web-reference`, browser tests) | 150 | `CHROME_PATH`; outside the gate |
| Mesa lavapipe (CPU rendering in orbs) | minimum 25.0.7 | `.agents/setup`; Debian bookworm-backports supplies Mesa and apt resolves its dependencies |

Every target and profile uses LLVM with four frontend threads
(`-Zthreads=4`); there are no backend overrides. Metadata embedding and the
build-directory layout use Cargo's defaults, and consumers read Cargo's
artifact messages, not guessed executable paths.

| Native target | Dev/test backend and frontend | Linker | Verification |
|---|---|---|---|
| arm64 macOS | LLVM, `-Zthreads=4` | platform linker | verified on macrunner |
| x86_64 Linux | LLVM, `-Zthreads=4` | mold | unverified |
| x86_64-pc-windows-msvc (studio-win) | LLVM, `-Zthreads=4` | rust-lld (lld-link), not link.exe | verified on studio-win (RTX 5060, DX12 and Vulkan) |

wgpu and naga stay on 30.0.1 until 31 is on crates.io; the move is kept
when its frames look right.

## Windows x64 prerequisites

Git Bash on Windows x64 with Visual Studio 2022 Build Tools (Desktop C++,
Windows SDK and C++ ATL for x64/x86, component
`Microsoft.VisualStudio.Component.VC.ATL`), the `x86_64-pc-windows-msvc`
Rust toolchain and the pinned tools on PATH. Cargo links with rust-lld's
lld-link and the MSVC libraries; do not substitute the GNU target for the
napi addon. `.gitattributes` keeps source and generated files LF and HDR
images binary even with `core.autocrlf=true`.

Windows selects DX12 and statically links DXC; game authors need only the
Microsoft Visual C++ x64 redistributable (MSVCP140/VCRUNTIME140).
`crates/og-tests/references/dx12/` holds a frame only where DX12
legitimately differs (nearest-sampled texels whose edges fall on pixel
centres), with the reason in the commit that recorded it; every other frame
is judged against `references/vulkan/` (§Reference frames). `og check` and
the package's typecheck use the pinned tsgo, since tsrs has no Windows
package. The Windows release package is built as docs/release.md §3 says.

To run the gate on studio-win, also put Python 3.12 with numpy and Pillow
(as `python3`), cargo-nextest and Bun on Git Bash's PATH, and point
`CHROME` at Chrome for Testing. Native GPU steps use DX12 on the NVIDIA
driver (`OG_GFX_BACKEND=vulkan` for Vulkan), never WARP.

## The GPU host

The GPU host is an NVIDIA L4 (proprietary Vulkan driver) behind the
personal orb provider `gcp-l4`: one GCP VM running one exclusive orb at a
time, directly on the VM. Pin
`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/nvidia_icd.json` so llvmpipe is
never picked. The owner starts and stops the VM; a stopped orb keeps its
disk, not its processes. One thread, the GPU integrator, holds that orb
and runs the GPU steps for the heads queued to it, in order (AGENTS.md
§Streams and landing). The compatibility hosts are listed in AGENTS.md
§Building and testing.

## Reference frames

One reference set serves every real GPU backend. New frames are recorded
on the GPU host only (`RECORDING_TIER`, `vulkan`), so a frame never
flip-flops between hosts; `references/<tier>/` holds a frame for another
backend only where it differs by design. A frame is judged against its own
tier's reference when one is committed, else the recording tier's, under
one rule, `og_tests::compare::tolerance_for` (png.ts `toleranceFor` and
reference-gfx.ts `limitFor` in the browser):

- against a reference recorded on the frame's own backend, or on a
  software adapter, the scene's `tolerance` (by default a channel off by
  more than 32/255 is an outlier; fail above 0.5 % outliers or an RGB RMSE
  above 0.01), so the recording platform keeps catching subtle regressions;
- against another backend's reference, that widened by the scene's
  `cross_tolerance`, `Tolerance::CROSS_PLATFORM` by default: channel 32/255,
  at most 1 % outliers, RMSE 0.024, and no RGB channel's absolute mean signed
  difference above 0.012, which is what keeps a
  dimmer light or a colour shift from hiding inside the wider RMSE.

`CROSS_PLATFORM` allows about twice the measured spread between backends
(mostly rasterisation ties along edges); measured 2026-11-10, it still
rejected a 15 % dimmer sun, a lost shadow, text moved by 3 px and a wrong
text colour. A scene whose spread is wider by nature names its own
`cross_tolerance` in `scenes.rs` with the measurement beside it; never
widen one to absorb a defect.

## Linux renderer prerequisites

`.agents/setup` upgrades `mesa-vulkan-drivers` below 25.0.7 (on Debian 12
from bookworm-backports), because Debian 12's Mesa 22.3.6 leaks memory
every frame. No reference lane runs on lavapipe.

## Lanes

There are two lanes. `quick` is the landing gate. `full` adds the checks
that require wasm, the retained UI packs, fixtures and browser execution.
The default is `full`; `--all` bypasses passing-input snapshots, not immutable
wasm products keyed by content. Neither lane partitions tests across machines.

| Lane | Checks |
|---|---|
| `quick` | fmt, clippy, native test build, CPU tests, GPU tests outside og-tests, generated API drift, non-engine package tests, the lanes wasm type check |
| `full` | quick plus og-tests, doctests, engine and extension Bun tests, bindgen-only wasm variants, fixtures, the browser tests of packages/engine's `test:browser` |

`OG_GATE_SKIP` (comma-separated step names) drops those steps from a run.
Its standing use is `OG_GATE_SKIP=rust-gpu scripts/land.sh` for a head
that does not change rendering (AGENTS.md §Streams and landing). It is never
a default and is read only from the environment; an unknown name fails the
run. A skipped step prints `SKIPPED <step> (OG_GATE_SKIP)` and the final
line says `NOT RUN (OG_GATE_SKIP)`, so a skip never reads as a pass; steps
that depend on it still run.

`scripts/check.sh` owns admission, `scripts/gate/run.py` schedules the
dependency DAG, and `step.sh` takes GPU slots only for rendering steps.
Native test binaries are built and listed once; CPU and GPU executions
reuse them.

Selection uses Cargo's resolved dependency graph, Bun's import graph and
fixture input manifests; passing snapshots are git trees of the working
copy under `target/og-gate/green`. A test binary's closure is its own
package's workspace crates, every path under `crates/` its Rust sources
name as `../og-x/...` or `crates/og-x/...`, and, for og-cli alone,
og-host-node. A Rust test outside og-cli that runs `og`, or one that reads
another crate's `.rs` file without naming its path that way, must be added
to `RUNS` in `scripts/gate/inputs.py`. An unchecked GPU run never creates
a passing record.

The profiles are as AGENTS.md §Building and testing names them;
`OG_WASM_PROFILE` selects the wasm one. Every web build ships
wasm-bindgen's output directly, and every wasm size check uses
shipping-profile bytes. No gate lane runs the size gate: `bash
scripts/release-wasm.sh` does, and `release.sh pack` calls it. `dev.txt`
lists the shipping publish and lanes variants the size gate measures,
despite its name.

### Test binaries

Each crate has at most one default integration binary, `tests`, declared
at `tests/it/main.rs` with `autotests = false`. Single-crate checks live
inline under `#[cfg(test)]`; integration targets cover cross-crate
contracts, GPU rendering and the CLI. nextest runs each test in its own
process; when running tests directly use `cargo test --
--test-threads=1`. Filters use `package(..) & test(/^module::/)`.

The test build (`--workspace`) and the gate's dev `og` and addon (`cargo
build -p og-host-node -p og-cli`, in `rust-build`) share every library
build. Cargo resolves features per command, so a feature that turns on in
only one graph builds that crate and every crate above it twice. Two are
aligned on purpose:

- syn 3: og-host-web's wasm-bindgen macros turn on `visit` and `visit-mut`
  in the test graph, so og-core-derive asks for the same features (`syn3`,
  unused in its code).
- og-physics: as a workspace member the test build selects its `default`,
  so og-host-node asks for og-physics' `default`.

To check after a dependency change, compare the two builds' artifact file
names (`--message-format=json`, `compiler-artifact` records with
`profile.test` false): a library both graphs build under two hashes is a
split, and the lowest such crate is its cause.

## What runs where

| Step | Command and contract | Lane |
|---|---|---|
| `fmt` | `cargo fmt --all --check` | quick, full |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | quick, full |
| `rust-build` | nextest builds and lists native test binaries; quick excludes og-tests. Then `cargo build -p og-host-node -p og-cli`, the dev addon and `og` every later step uses; `packages/engine/scripts/build.sh` under the gate copies them and runs no cargo. | quick, full |
| `rust-cpu`, `rust-gpu` | nextest executes the discovered tests with separate reports and bounded workers. | quick, full |
| `doc-tests` | `cargo test --workspace --doc` | full |
| `drift` | The dev `og api check` checks generated declarations, wiki, schema and Rust glue against the IDL. | quick, full |
| `bun-quick` | Non-engine package tests, without rebuilding native artifacts. | quick, full |
| `wasm-lanes-check` | `scripts/wasm-lanes.sh check` of each `lanes` variant in `scripts/wasm-crates.txt` whenever og-render, og-host-web or a crate they depend on changes, because under atomics wgpu's web types are not Send. | quick, full |
| `wasm-build` | Build `scripts/wasm-crates.txt` modules in `wasm-test` into `target/og-wasm/raw`. `OG_WASM_VARIANTS` picks them: `dev` (the development module alone: bun-test's cross-host tests, a landing), `reference` (adds the 3D publish pair, web-reference alone), `all` (every publish and lanes variant: a run with `browser`, and `scripts/release-wasm.sh`). Tests that load a publish or lanes module skip without one. | full |
| `wasm-bindgen` | Bindgen-only modules for all variants in `target/og-wasm/dev`. `dev-all.txt` lists every module; `dev.txt` selects publish and lanes variants. | full, web-reference prerequisite |
| `size-gate` | Brotli-11 bytes of each shipping-profile bindgen module in `dev.txt`, checked against `wasm_core_brotli_bytes` before packing. | release only |
| `bun-test` | Package tests and typechecks using the dev addon and gate-produced wasm (`OG_GATE=1`). | full |
| `fixtures` | `og test` for each changed runnable feature fixture and the creation skeleton; needs only the native `og`, so `--only fixtures` builds no wasm. | full |
| `browser` | Every file packages/engine's `test:browser` names, each run even when one before it failed. | full |
| `web-reference` | Headless Chrome WebGPU groups with `OG_WEB_JOBS` concurrency (default 3), compared under unchanged tolerances. | explicit |
| `check-conformance` | `scripts/tsrs-diff.sh`: compare tsrs and pinned tsgo diagnostics over packages, templates, the test fixtures and the error corpus. | explicit |
| perf | `scripts/perf.sh`: ignored `perf_*` assertions in og-tests, og-runtime and og-audio, release profile, serially under every GPU slot. | explicit |

The gate requires nextest. Bun tests have a 30-second per-test timeout
(`OG_BUN_TEST_TIMEOUT`). Refresh missing functional wasm with
`scripts/check.sh --only wasm-bindgen`. Never subscribe to or wait on
GitHub checks; `.github/workflows/release.yml` is only the fallback release
pack (docs/release.md §3).

## Running the gate locally

```
scripts/install-tools.sh             # once; macOS uses Homebrew for brotli
scripts/check.sh quick               # the quick lane (make quick)
scripts/check.sh full                # the full lane, dependency-selected
scripts/check.sh --all               # the full lane with nothing skipped
scripts/check.sh quick --only rust-cpu # selected steps and prerequisites
OG_GATE_JOBS=1 scripts/check.sh      # one step at a time, in lane order
bash scripts/release-wasm.sh         # release bindgen wasm and its shipping-size check
scripts/check.sh web-reference       # the browser tiers (make web-reference); needs Chrome
scripts/check.sh --list
```

The path-aware state is `<target>/og-gate/`, and `<target>/og-wasm-cache/`
keeps compressed-size results; never delete either to force a step.

`check.sh` and `land.sh` keep scratch in a private
`<checkout>/.og/tmp/run.*` directory (at least 2 GiB free) and remove only
their own on exit. A forced kill can leave scratch behind: check processes
and open files before removing it, and never sweep another gate's live
directory.

Admission, the per-checkout target lock and the FIFO GPU slots
(`OG_GPU_SLOTS`, default 6) follow AGENTS.md §Building and testing and are
implemented in `scripts/lock.sh`. A measurement draining admission yields
to a waiting landing recheck at once and to every waiting gate after
`OG_MEASURE_DRAIN` seconds (default 120). `OG_GATE_JOBS` bounds concurrent
steps (default 2) and `OG_GATE_THREADS` the Cargo and nextest jobs per step.

Each run keeps its logs and reports under
`<target>/evidence/gate/runs/<timestamp>-<pid>/` (`OG_EVIDENCE_DIR`
defaults to `<target>/evidence`). Reports from a cancelled run are not
evidence of a pass.

## Amp orbs

`.agents/setup` installs the prerequisites and pinned tools and fetches the
dependencies; it does not compile the engine or run the gate.
`.agents/environment` points `CHROME` at the orb's agent-browser Chrome,
which is not claimed pixel-equivalent to the reference Chrome. Validate
changes with `shellcheck -s bash .agents/{setup,resume,environment}`,
`time .agents/setup` twice, and the tools from a new `bash -lc` shell.

## The compiler cache

sccache keeps a local disk cache on every host and no remote cache,
because remote hits were slower than local rebuilds. The checkouts on one
machine share it: `/Volumes/app/.sccache` on macrunner,
`SCCACHE_DIR=E:\sccache` on studio-win, `~/.cache/sccache` on an orb.

One sccache server per host serves every checkout. `.cargo/config.toml`'s
rustc wrapper is `og-sccache-wrap` (`crates/og-sccache-wrap`), which starts
that server without the jobserver of the cargo that happens to need it
first, and on Unix stops a server that has stalled for `OG_SCCACHE_STALL`
seconds (default 60, 0 turns the watch off); its source comments and
`tests/start.rs` hold the details. `RUSTC_WRAPPER=` (empty) bypasses the
wrapper and sccache. To restart a server, run `sccache --stop-server`; a
server started by hand must go without the jobserver: `env -u
CARGO_MAKEFLAGS -u MAKEFLAGS -u MFLAGS sccache --start-server`.

## When to run what

While writing code run `cargo check -p <crate>` and that crate's tests;
`quick` is the most a worktree runs, `full` follows a batch of landings,
and `--all` is never run after every edit. After re-recording a reference
frame, rerun only that frame's lane: `--only rust-gpu` for og-tests' gfx
references, `--only bun-test` for a template's playtest frames,
`web-reference` for the browser tiers.

## Uniform host coverage

Every host runs the same named lane, with no host-only subset or test
routing. A missing adapter warns with wgpu's error and the skipped counts;
other initialization errors and failed checks still fail. Rust GPU tests
declare `#[ignore = "requires GPU"]` in an adapter-aware `tests/it` binary
(`tests/gpu.rs`), which discovery enables when an adapter exists; perf ones
use `requires GPU; perf`. Bun uses `gpuTest` from
`packages/engine/tests/support/gpu.ts` or `test.skipIf(...)`; never return
early from a test body to skip. Never force GPU tests with nextest's
`--run-ignored`, which would count a runtime skip as a pass; use `cargo
test -- --ignored`. A serial/parallel pixel difference between browser
groups is a defect, not a reason to raise tolerances.

## The lanes wasm

The lanes variant of og-host-web (docs/web-export.md §Lanes) is the
publish build compiled for wasm atomics and shared memory with a rebuilt
std, one module per game profile. `scripts/wasm-lanes.sh <cargo args>`
builds it into `<target>/wasm-mt`, so its flags never invalidate the
single-threaded build; it needs the `rust-src` component, and rustc's
`unstable feature specified for -Ctarget-feature: atomics` warning is
expected. build.sh copies each lanes module with its `snippets/` into its
profile's directory; under the gate it exits 1 when either is missing, and
`scripts/pack.sh` refuses a package without them. A Rust panic inside a
lane reaches the page as a structured `panic` error (docs/web-export.md
§Errors), which `bun tests/web/lane-panic.ts` checks.

## `packages/engine/scripts/build.sh` outside the gate

Standalone (`bun run build`, `bun run test` in `packages/engine`), build.sh
reuses the gate's dev addon, `og` and wasm whenever the gate has built in
this target directory, and says so. It compiles release and its own wasm
only with `OG_RELEASE=1`, with `--linux-x64`, or when the gate has not
built here; a release build holds the gate lock, and `OG_RELEASE=1` runs
the shipping-size check. `native/build-profile` records `dev` or
`release`, and `scripts/pack.sh` packs only `release`.

## Switches in `scripts/budgets.txt` and `scripts/wasm-crates.txt`

Each file's comments describe its lines. `wasm-crates.txt` marks a crate
`required` (fails the gate) or `pending` (warns while it is ported); its
`variant` and `lanes` lines each build one download the size gate measures
on its own (docs/web-export.md §Variants). `budgets.txt`'s
`wasm_artifacts required` fails the size gate when no wasm was built, and
`drift_check required` fails the gate when `og api check` is missing.

## Switches in the environment

- `OG_RENDERER=none` makes native og-render answer `NoAdapter`, as on a
  machine with no usable adapter, so the no-renderer path is testable
  beside a GPU; the platform's `engine` and `probe` sandboxes set it
  (docs/design.md §5). Without a renderer the simulation, probe, `dump_*`,
  snapshots and world hash are identical, screenshots are reported
  unchecked, and `--require-checked` makes unchecked checks exit 1.
  `packages/engine/tests/no-renderer.test.ts` and og-runtime's
  `tests/no_renderer.rs` hold the contract.
- `OG_GATE_JOBS`, `OG_GATE_THREADS` (§Running the gate locally),
  `OG_WASM_PROFILE` (§Lanes), `OG_BUN_TEST_TIMEOUT` and `OG_WEB_JOBS`
  (§What runs where).

## Browser reference tiers

The browser tier is WebGPU in headless Chrome for Testing. A frame with no
file under `references/<tier>/` is judged against `references/vulkan/`
under §Reference frames' rule, and a frame on neither fails the run, never
passes. A tier gets its own reference only for a difference it makes by
design; `--record` writes the tier's references. A missing Chrome is an
error, never a skip.

`web-reference` runs four groups (`WEB_GROUPS` in
`scripts/gate/commands.sh`): **ui** (§The accessibility mirror in the
browser), **gfx** (§og-tests' gfx scenes in the browser),
**vehicle-terrain** (`tests/fixtures/vehicle-terrain`, its script and shots
read from the playtest itself, so a retimed playtest moves its browser
frames with it) and **lane-panic** (§The lanes wasm).

### og-tests' gfx scenes in the browser

`packages/engine/tests/web/reference-gfx.ts` draws og-tests' and og-ui's
reference scenes and the cooked-font text frames through the wasm module
`crates/og-tests-web`, as the native harness does, and compares each with
its `references/webgpu/` frame, else `vulkan`. It builds only
`og-tests-web`, in `wasm-test` under `target/wasm-gfx` (the `wasm` profile
for `--time` and `--perf-out`); its options are in the script.

### The accessibility mirror in the browser

`packages/engine/tests/web/a11y.ts` runs the dev export of
`tests/fixtures/a11y` and checks the shell's DOM mirror of `og.ui.a11y()`
(src/web/a11y.ts) against the UI and Chrome's own accessibility tree:
structure, bounds, roles, screen-reader-style clicks, the live region,
`prefers-contrast` and `prefers-reduced-motion`. It writes
`dist/test-a11y/out/a11y-<tier>.json`.

## Updating a budget

Budgets come from `docs/design.md` §7 and live in `scripts/budgets.txt`.
`wasm_core_brotli_bytes` is per download: a page downloads one engine
module, so each variant is held to it and the size gate's figure is the
largest (`max_brotli_bytes` in `size-gate.json`), never the sum. The
budget is 16 MB brotli per download, one ceiling for every variant. It
catches accidental growth and is never a reason to cut a feature. Change
the number in its own commit, with the measured value from
`target/evidence/size-gate.json` and the reason in the message; a raise
above a design §7 target changes design §7 in the same commit.

## The model bucket

og.motion's weights, which `packages/engine` ships, are not in git.
`packages/engine/motion.json` pins the file (`file`, `sha256`, `bytes`,
`key`, `url`). Every machine reads it anonymously from `url`, the public
Cloudflare R2 bucket `origin-engine-models`, so no credentials are needed
to fetch. The private S3 bucket `origin-engine-showcase` on OVHcloud
Object Storage (endpoint `https://s3.bhs.io.cloud.ovh.net`, region `bhs`)
keeps the same file under `motion/<sha256>.safetensors` as the fallback
and the upload target; its credentials are `OG_SHOWCASE_S3_ACCESS_KEY` and
`OG_SHOWCASE_S3_SECRET_KEY`. A new pin is put in both: `upload` for OVH,
then `wrangler r2 object put origin-engine-models/<key> --file <file>
--remote`. `bun packages/engine/scripts/motion-model.ts ensure` installs
the pin; development builds and `pack.sh` run it and stop when it fails,
while a bare `cargo build` does not.

A second private bucket in the same project, `origin-generated-assets`
(versioned, encrypted; puts send `x-amz-server-side-encryption: AES256`),
keeps generated source assets for later model runs; its layout is in its
`README.md`.

## Re-capturing gfx references

gfx tests in `og-tests` compare a headless render with a committed PNG and
on mismatch write `<scene>.actual.png` and `<scene>.diff.png` under
`$OG_EVIDENCE_DIR/gfx/`. When a rendering change is intended:

```
# on the GPU host, after `scripts/gate/step.sh rust-build` in the full lane
OG_GFX_CAPTURE=1 OG_UI_CAPTURE=1 scripts/gate/step.sh rust-gpu   # rewrite the references
git diff --stat -- '*/references/vulkan/*'                         # see which frames moved
```

`OG_GFX_CAPTURE` takes `1` or a comma list of og-tests scenes;
`OG_UI_CAPTURE=1` rewrites og-ui's frames in
`crates/og-tests-ui/references`. Record twice and compare the two sets byte
for byte before committing, so no frame flips on the host. Commit the new
references in a batch and say what each moved frame now shows, not by how
much it moved.
A frame from another tier never replaces a recording-tier reference.
A scene with no reference on the adapter's tier is compared with
`references/vulkan/` under the cross-platform tolerance (§Reference
frames), and the test says so. Metal keeps no set of its own: it is judged
against `references/vulkan/` like every other tier. A project whose check
names `references/metal/` itself (`tests/fixtures/ui-css-pack`) keeps
those frames until the L4 records them.

## When the size gate fails

1. Read the table `size-gate` printed (or `target/evidence/size-gate.json`):
   which wasm grew and by how much.
2. Find the growth: `cargo tree -p og-host-web --target
   wasm32-unknown-unknown -e normal` for new dependencies, and
   `scripts/wasm-size-report.sh target/og-wasm/raw/<name>.wasm` for where
   the bytes are, per crate and for the largest functions. Rank by brotli:
   generic copies compress to little more than one copy.
3. Prefer removing the cause: drop or feature-gate the dependency, disable
   default features, move debug-only code behind the non-`publish`
   configuration, avoid monomorphisation blow-ups. A stable sort in engine
   code is `og_core::sort::{sort, sort_by, sort_by_key}`, and a `BTreeMap`
   or `BTreeSet` built from an iterator is `og_core::sort::{map, set}`,
   because the standard library's versions monomorphise driftsort per
   element type and closure. A publish build carries no reflected field
   docs, so a new doc string costs a publish module nothing.
4. When the growth is intended, update the budget (§Updating a budget)
   rather than cutting the feature (design §7).
