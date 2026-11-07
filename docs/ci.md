# The local gate

The gate is one script, `scripts/check.sh`, run on macrunner or on
studio-win under Git Bash; there is no GitHub CI. The rules it rests on are in AGENTS.md §Building and testing: one
target directory, the dev profile for every native step, tests that never
run `cargo`.

## Landing: a merge queue

Authors land their heads with `scripts/land.sh`. Start with a clean,
committed branch. The command rebases onto fetched main and runs
`check.sh quick` outside any landing lock, so several landers' gates
overlap. It then joins the landing queue: one FIFO lock in the common Git
directory, separate from the machine gate lock, held through the push.
Only the lander at the head of the queue moves main.

If main advanced while quick ran, the command compares both sides from the
tested base. Disjoint file changes reuse the pass only when neither side
touches common build inputs or generated/drift-checked files. Otherwise it
rebases and runs quick once more at the head of the queue; per-step input
records skip unaffected work. Main cannot move during that recheck, so a
landing gates at most twice. Waiting landers queue behind it instead of
racing it. A push rejected because a writer outside the queue moved main
stops with a message to run the command again.
Conflicts confined to the IDL's generated directories
(`og_idl::OUTPUT_DIRS`, including declarations and wiki) or the generated UI
theme are regenerated from the rebased sources. Any other conflict stops the
rebase untouched for the author. Regeneration builds the IDL generator without
the hosts, so conflicted generated Rust cannot block it. Each conflict stop is
handled separately, and the completed rebase is regenerated again: later commits
can merge cleanly yet leave stale derived text. That final refresh is amended
into the branch tip before quick runs. Generation failure stops landing.
No force push is used. Rerunning after success is a no-op.

`python3 scripts/test-landing.py` exercises concurrent landings against a
disposable local bare remote, including overlap, shared inputs and failures.

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
| TypeScript, tsgo (`check-conformance`'s comparison and `og check`'s fallback where tsrs has no package; no project or package depends on it) | 7.0.2 | `TSGO_VERSION` in `crates/og-cli/src/check.rs` (`scripts/tsrs-diff.sh` and `.agents/setup` read it) |
| tsrs (the type checker: `og check`, the check before `og run`/`test`/`build`/`dev`, `packages/engine`'s `typecheck`, og-idl's declaration test) | 0.18.0-ts7.1.0-dev.20260929 (TypeScript 7.1.0-dev.20260930.4) | `TSRS_VERSION` and the five SRIs in `scripts/install-tools.sh`; `packages/engine/package.json` `optionalDependencies` copy it (`check-conformance` holds them equal). It moves with a tsrs release, all of these in one commit, after `scripts/tsrs-diff.sh` passes on the new binary |
| cargo-nextest | 0.9.148 here | not pinned |
| ShellCheck (CI lints `scripts/*.sh` with it; Ubuntu's 0.9 reports what 0.11 does not) | 0.11.0 | `SHELLCHECK_VERSION` in `scripts/install-tools.sh` (Homebrew's on macOS) |
| node (`determinism`'s wasm32 runs) | 24 in CI | `.github/actions/provision` |
| sccache (the rustc wrapper, `.cargo/config.toml`; not used in CI) | 0.19.0 | `SCCACHE_VERSION` in `scripts/install-tools.sh` |
| mold (Linux native linker) | distribution package | `.agents/setup`, `scripts/install-tools.sh`; `.cargo/config.toml` selects it |
| cargo-zigbuild, zig (`build.sh --linux-x64`) | 0.23.4, 0.17.0 here | not pinned; outside the gate |
| Chrome for Testing (`make web-reference`, browser tests) | 150 | `CHROME_PATH`; outside the gate |
| Mesa lavapipe (CPU rendering in orbs) | minimum 25.0.7 | `.agents/setup`; Debian bookworm-backports supplies Mesa and apt resolves its dependencies |

Every target and profile uses LLVM, including dev, test, build scripts,
release, wasm-test and wasm; there are no backend overrides. Four frontend threads
(`-Zthreads=4`) are enabled on every target. Metadata embedding and the
build-directory layout use Cargo's defaults. Consumers use Cargo's
artifact messages, not guessed executable paths.

| Native target | Dev/test backend and frontend | Linker | Verification |
|---|---|---|---|
| arm64 macOS | LLVM, `-Zthreads=4` | platform linker | verified on macrunner |
| x86_64 Linux | LLVM, `-Zthreads=4` | mold | unverified |
| x86_64-pc-windows-msvc (studio-win) | LLVM, `-Zthreads=4` | rust-lld (lld-link), not link.exe | verified on studio-win (RTX 5060, DX12 and Vulkan) |

wgpu and naga stay on 30.0.1 until 31 is on crates.io; the move is kept
only if every committed reference frame passes without re-recording.

## Windows x64 prerequisites

Git Bash on Windows x64 with Visual Studio 2022 Build Tools (Desktop C++,
Windows SDK and C++ ATL for x64/x86, component
`Microsoft.VisualStudio.Component.VC.ATL`), the `x86_64-pc-windows-msvc`
Rust toolchain and the pinned tools on PATH. Cargo links with rust-lld's
lld-link and the MSVC libraries; do not substitute the GNU target for the
napi addon. `.gitattributes` keeps source and generated files LF and HDR
images binary even with `core.autocrlf=true`.

Windows selects DX12 and statically links DXC, so the installed engine
needs no compiler DLL; game authors need the Microsoft Visual C++ x64
redistributable (MSVCP140/VCRUNTIME140), not Visual Studio or Rust. The
reference harness falls back to Metal's frames under the cross-platform
tolerance (§Reference frames). `crates/og-tests/references/dx12/`
and `vulkan/` (`OG_GFX_BACKEND=vulkan`) hold a frame only where that
backend legitimately differs from Metal (nearest-sampled texels whose edges
fall on pixel centres; NVIDIA's Vulkan anisotropic filter), with the reason
in the commit that recorded it. `og check` and the package's typecheck use
the pinned tsgo through Bun, since tsrs has no Windows package. The
Windows release package is built with `OG_RELEASE=1 sh
packages/engine/scripts/build.sh --native && sh
packages/engine/scripts/pack.sh --native` (docs/release.md §3).

To run the gate on studio-win, also put on Git Bash's PATH: Python 3.12
with numpy and Pillow, reachable as `python3` (a `python3.exe` copy of
`python.exe`); cargo-nextest; and Bun. Point `CHROME` at Chrome for
Testing (`chrome-win64/chrome.exe`). The scripts set `PYTHONUTF8=1` and
take `C:/` paths as absolute. `scripts/lock.sh` identifies a holder by its
start time from `ps -o lstart`, or from `/proc/<pid>/stat` where MSYS's
`ps` has no `lstart`. Native GPU steps use DX12 on the NVIDIA driver
(`OG_GFX_BACKEND=vulkan` for Vulkan), never WARP. web-reference's motion
groups need the pinned og.motion weights (§The model bucket: the
`OG_SHOWCASE_S3_*` credentials or a local copy with the pinned sha256).

## Reference frames

One reference set serves every real GPU backend. New frames are recorded
on `metal` (macrunner) only, so a frame never flip-flops between hosts'
renderings (§Re-capturing gfx references); `references/<tier>/` holds a
frame for another backend only where it differs by design (§Windows x64
prerequisites). A frame is judged against its own tier's reference when
one is committed, else Metal's, under one rule,
`og_tests::compare::tolerance_for` (png.ts `toleranceFor` and
reference-gfx.ts `limitFor` in the browser):

- against a reference recorded on the frame's own backend, or on a
  software adapter, the scene's `tolerance` (by default a channel off by
  more than 32/255 is an outlier; fail above 0.5 % outliers or an RGB RMSE
  above 0.01), so the recording platform keeps catching subtle regressions;
- against another backend's reference, that widened by the scene's
  `cross_tolerance`, `Tolerance::CROSS_PLATFORM` by default: RMSE 0.016, and
  no RGB channel's mean signed difference above 0.008, which is what keeps a
  dimmer light or a colour shift from hiding inside the wider RMSE.

`CROSS_PLATFORM` is set from the measured spread with a margin (2026-11-08,
RTX 5060 on DX12, Vulkan and Chrome 150 on D3D12, against Metal): RMSE at
most 0.0102, outliers at most 0.08 %, channel bias at most 0.0057, almost
all of it rasterisation and threshold ties along edges.
`og-tests` `the_cross_platform_tolerance_still_catches_small_regressions`
proves that it still fails a 10 % dimmer sun, a lost shadow, text moved by
3 px and a wrong text colour. A scene whose spread is wider by nature
names its own `cross_tolerance` in `scenes.rs` with the measurement beside
it (visual-garment, stylise-print, sprites-array, tilemap-lit); never widen
one to absorb a defect.

## Linux renderer prerequisites

`.agents/setup` upgrades `mesa-vulkan-drivers` below 25.0.7 (on Debian 12
from bookworm-backports): Debian 12's Mesa 22.3.6 grows resident memory
every frame and is killed under a 4 GiB limit. A warm setup skips apt when
its prerequisites are met. No reference lane runs on lavapipe.

## Lanes

There are two lanes. `quick` is the landing gate. `full` adds the checks
that require wasm, the retained UI packs, fixtures and browser execution.
The default is `full`; `--all` bypasses passing-input snapshots, not immutable
wasm products keyed by content. Neither lane partitions tests across machines.

| Lane | Checks |
|---|---|
| `quick` | fmt, clippy, native test build, CPU tests, GPU tests outside og-tests, generated API drift, non-engine package tests |
| `full` | quick plus og-tests, doctests, opt-in kit Rust targets, engine and extension Bun tests, bindgen-only wasm variants, fixtures, browser wire/player/export/unsupported/motion-tick tests |

`scripts/check.sh` owns admission and checkout ownership.
`scripts/gate/run.py` schedules the dependency DAG. `step.sh` acquires GPU
slots only for rendering steps and dispatches commands. Native build and
execution are separate: nextest lists built binaries; pinned-nightly
libtest JSON lists their tests, source files and GPU requirements without
opening an adapter. CPU and GPU executions reuse those binaries.

Selection uses Cargo's resolved dependency graph, Bun's import graph
(including package self-imports), and fixture input manifests. Passing
snapshots are git trees of the working copy, including untracked files,
under `target/og-gate/green`. Changes whose consumers are not proven,
including runtime resources, native inputs, generated files and dynamic
loads, run checks rather than skipping them. Rust source selection remains
conservative at the binary's crate closure where shared helper usage is
unknown. Bun records each test file separately. A failure invalidates its
record; an unchecked GPU run never creates a passing record.

The Cargo profiles are `dev` (native tests), `release` (native shipping),
`wasm-test` (functional and browser wasm), and `wasm` (shipping wasm and
explicit performance runs). `wasm-test` uses opt-level 1, no LTO, 16 codegen
units, incremental compilation and no debug info. Shipping package overrides
remain in `wasm`. Every wasm size check uses shipping-profile bytes, not an
upper-bound development profile. Normal wasm variants and lanes variants
build concurrently because their Cargo outputs are separate. The shared
sccache wrapper caches compilation, including build-std; each checkout
retains its own target directory. Variants sharing a target build serially.

Every web build ships wasm-bindgen's output directly, including publish,
lanes and development variants. No gate lane runs the size gate.
An explicit web-reference run builds the development host and the 3D publish
and lanes modules its groups load, not the unused 2D publish pair. The gfx
group separately builds only `og-tests-web`.
`bash scripts/release-wasm.sh` prepares release wasm and checks its shipping
size. `release.sh pack` calls it through the package build. `dev.txt` lists
the shipping publish and lanes variants measured by the size gate, despite
the directory's historical `dev` name. They use the `wasm` profile for releases.

The deleted gameplay templates, gallery, art-samples, standalone
wasm-determinism tool and replay-gate have no gate stage. Retained package
and extension tests run in `full`; the old functional and kits lanes no
longer exist.

### Test binaries

Each crate has at most one default integration binary, `tests`, declared
at `tests/it/main.rs` with `autotests = false`. The opt-in `kit-tests`
targets belong to the full lane. Single-crate checks live inline under
`#[cfg(test)]`; integration targets cover cross-crate contracts, GPU
rendering and the CLI. nextest runs each test in its own process,
including tests that change process-wide state; use nextest, or `cargo
test -- --test-threads=1` when running them directly. Filters use
`package(..) & test(/^module::/)`.

The test build (`--workspace`) and the gate's dev `og` and addon (`cargo
build -p og-host-node -p og-cli`, `packages/engine/scripts/build.sh`)
share every library build. Cargo resolves features per command, so a
feature that turns on in only one graph gives that crate and every crate
above it a second build. Two are aligned on purpose:

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
| `rust-build` | nextest builds and lists native test binaries; quick excludes og-tests, full includes retained kit features. | quick, full |
| `rust-cpu`, `rust-gpu` | nextest executes the discovered tests with separate reports and bounded workers. | quick, full |
| `doc-tests` | `cargo test --workspace --doc` | full |
| `drift` | The dev `og api check` checks generated declarations, wiki, schema and Rust glue against the IDL. | quick, full |
| `bun-quick` | Non-engine package tests, without rebuilding native artifacts. | quick, full |
| `wasm-build` | Build `scripts/wasm-crates.txt` variants in `wasm-test` for the cross-host, publish and lanes tests. Normal variants share the wasm32 target; lanes variants share `target/wasm-mt`. Outputs go to `target/og-wasm/raw`. | full |
| `wasm-bindgen` | Bindgen-only modules for all variants in `target/og-wasm/dev`, with glue in `bindgen`. `dev-all.txt` lists every module; `dev.txt` selects publish and lanes variants. | full, web-reference prerequisite |
| `size-gate` | Brotli-11 bytes of each shipping-profile bindgen module in `dev.txt`, checked against `wasm_core_brotli_bytes` before packing. | release only |
| `bun-test` | Package tests and typechecks using the dev addon and gate-produced wasm (`OG_GATE=1`), without a second release build. | full |
| `fixtures` | `og test` for each changed runnable feature fixture and the creation skeleton. | full |
| `browser` | Browser wire, player, export, device, unsupported-device and og.motion tick tests (`test:browser`). | full |
| `web-reference` | Native and web preparation, then headless Chrome WebGPU groups with `OG_WEB_JOBS` concurrency (default 3). Compare retained scenes under unchanged tolerances. | explicit |
| `check-conformance` | `scripts/tsrs-diff.sh`: compare tsrs and pinned tsgo diagnostics over packages, templates, the test fixtures and the error corpus. | explicit |
| perf | `scripts/perf.sh`: ignored `perf_*` assertions in og-tests, og-runtime and og-audio, release profile, serially under every GPU slot. | explicit |

The gate requires nextest. Missing adapters warn on every host; GPU tests
are skipped, not passed. Bun tests have a 30-second per-test timeout through
`OG_BUN_TEST_TIMEOUT`; a real hang fails.
Refresh missing functional wasm with `scripts/check.sh --only wasm-bindgen`.
Run release optimization and the size gate with `bash scripts/release-wasm.sh`.

Fixtures several test processes read are made once under a file lock
(og-cook's `tests/support`: the template and the rigged models, keyed by
the test executable and the sources). `.config/nextest.toml` gives every
slot to the tests that assert a wall-clock bound (and runs them last), two
slots to the determinism runs that step two worlds or renderers at once,
four at a time to the GPU-bound packages, and starts og-tests,
og-tests-cooked, the cooks, the pile and the walk first.

### CI

There is no GitHub test CI: the gate is `scripts/check.sh`; never
subscribe to or wait on GitHub checks. Every machine runs the same lane
and needs a real GPU for rendering tests; an unchecked frame is never
evidence of a rendering pass.

nextest isolates tests in separate processes. Direct `cargo test` runs
tests on threads of one process; use `--test-threads=1` for renderer
tests to avoid simultaneous adapter creation in one process.

`.github/workflows/release.yml` (`workflow_dispatch` only, no secrets,
nothing published) is the fallback release pack when the Mac or the
Windows host is unavailable (docs/release.md §3).

## Running the gate locally

```
scripts/install-tools.sh             # once; macOS uses Homebrew for brotli
scripts/check.sh quick               # the quick lane (make quick)
scripts/check.sh full                # the full lane, dependency-selected
scripts/check.sh --all               # the full lane with nothing skipped
scripts/check.sh quick --only rust-cpu # selected steps and prerequisites
scripts/check.sh --only bun-test     # includes wasm-build and wasm-bindgen
OG_GATE_JOBS=1 scripts/check.sh      # one step at a time, in lane order
bash scripts/release-wasm.sh         # release bindgen wasm and its shipping-size check
scripts/check.sh --list
scripts/check.sh web-reference       # the browser tiers (make web-reference); needs Chrome
make check                           # the same through the Makefile
```

The path-aware state is `<target>/og-gate/` (the green trees).
`<target>/og-wasm-cache/` retains compressed-size results. Never delete
either to force a step; `--all` bypasses passing-input snapshots.

Bindgen outputs stay in each checkout's `target/og-wasm/`. There is no
machine-wide wasm-output store or post-bindgen optimization pass. sccache
still shares compiled dependencies between lanes (§The compiler cache).

Each checkout holds its own target lock. Gates briefly register shared
admission and then execute concurrently, without a machine-wide CPU lock.
Exclusive admission closes the entrance and drains active gates for perf.
It is separate from GPU ownership: nothing holds GPU slots while waiting
for a child gate. Rendering steps acquire a FIFO slot;
single-slot and all-slot requests share the queue. Owners are identified
by PID and process birth, not command names. Inherited ownership lets a
child reuse its parent's resources. The versioned implementation in
`~/.cache/og-gate/gpu-lock.sh` applies to newly started callers.

`OG_GATE_JOBS` bounds concurrent steps (default 2), and `OG_GATE_THREADS`
bounds Cargo and nextest jobs per step. Unset, `scripts/check.sh` derives
it at start: the performance cores (`nproc` off macOS) divided by the live
gates in admission times `OG_GATE_JOBS`, at least 2, so a lone gate on an
idle machine takes most of the cores. Both must be positive.
The DAG starts a step only after its prerequisites complete. CPU tests
hold no GPU slot; rendering tests reuse the same compiled binaries.

Each run retains `<target>/evidence/gate/runs/<timestamp>-<pid>/run.json`,
step logs, nextest JUnit reports and Bun per-file timing JSON. The summary
reports PASS, FAIL or SKIPPED and wall time per step. Failure or cancellation
stops child process groups before releasing ownership. Reports from a
cancelled run are not evidence of a pass. `OG_EVIDENCE_DIR` defaults to
`<target>/evidence`; `OG_WASM_OUT` points at `<target>/og-wasm`.

## Amp orbs

`.agents/setup` installs the Linux build prerequisites, activates
`rust-toolchain.toml` and reuses `scripts/install-tools.sh` for the pinned
tools. It downloads a nextest binary, fetches both locked Cargo dependency
graphs, installs `tools/bench` and `tests/install` from their Bun
lockfiles and caches the pinned TypeScript executable. It does not compile
the engine or run the gate; a stale snapshot reruns the idempotent setup.
A repository-scoped login-shell hook loads `.agents/environment` and sets
`CHROME` to the orb's agent-browser Chrome unless overridden (not claimed
pixel-equivalent to the reference Chrome). `.agents/resume` does no
installation or authentication. Validate changes with `shellcheck -s bash
.agents/{setup,resume,environment}`, `time .agents/setup` twice, and the
tools from a new `bash -lc` shell.

## The compiler cache

sccache keeps a local disk cache on every host (the owner, 2026-11-07): a
remote cache in OVH BHS measured 302.7 s for a cleaned `og-cli` rebuild on
macrunner against 38.9 s from the local disk (about 3.7 s per remote hit),
so there is none. The persistent lanes on one machine share its cache: on
macrunner `/Volumes/app/.sccache`, on studio-win `SCCACHE_DIR=E:\sccache`,
on an orb `~/.cache/sccache`.

## When to run what

- **While writing code**: `cargo check -p <crate>` and that crate's own
  tests. `scripts/check.sh quick` is the most a worktree runs, and what a
  landing runs.
- **After a batch of landings**: `scripts/check.sh full`. This includes
  bindgen-only wasm, cross-host determinism and browser contracts, never
  shipping-size measurements.
- **Before packing a release**: `bash scripts/release-wasm.sh` builds the
  shipping-profile variants, runs bindgen and checks their brotli size.
  `scripts/release.sh pack` runs this preparation automatically.
- Never `--all` after every edit; never touch files or delete
  `target/og-gate/` to force a step.
- **After re-recording a reference frame**: rerun only that frame's lane:
  `scripts/check.sh --only rust-gpu` for og-tests' gfx references (§Re-capturing
  gfx references), `--only bun-test` for a template's playtest frames,
  `scripts/check.sh web-reference` for the browser tiers.

## Uniform host coverage

Every host runs the same named lane: no host-only subset, nextest
partition or archive, or test routing. A missing adapter produces a loud
warning with wgpu's error and skipped/unchecked counts, not a failing exit
status; other initialization errors and failed checks still fail.
`OG_RENDERER=none` only makes the missing adapter expected.

Rust GPU tests declare `#[ignore = "requires GPU"]` in an adapter-aware
`tests/it` binary (`tests/gpu.rs`); discovery enables them when an adapter
exists. GPU tests that also need explicit perf selection use `requires GPU;
perf`. Bun uses `gpuTest` from `packages/engine/tests/support/gpu.ts`,
or `test.skipIf(gpuUnavailable || otherPrerequisiteMissing)`; never return
early from a test body to skip. The gate forwards nextest's
adapter-discovery warnings and does not cache an unchecked GPU run as
green. Do not force unavailable GPU tests with `--run-ignored all/only`
(rejected: nextest would call a runtime skip a pass); use `cargo test --
--ignored` for explicit ignored-test runs, as the perf lane does.

Browser groups use separate ports and profile/output directories; work
that writes the same project's export stays in one group. A serial/parallel
pixel difference is a defect, not a reason to raise tolerances.

The full lane's Bun cross-host tests compare native and wasm math, physics,
animation and text digests using the development engine modules. Publish
modules expose no debug digest command; they are compared through world
snapshots. The lanes tests use real workers and assert parallel execution.
There is no separate determinism workspace or determinism stage.

## The lanes wasm

The lanes variant of og-host-web (docs/web-export.md §Lanes) is the
publish build compiled for wasm atomics and shared memory, which needs a
rebuilt std: `scripts/wasm-lanes.sh <cargo args>` runs `cargo build
--target wasm32-unknown-unknown -Z build-std=panic_abort,std` with
`RUSTC_BOOTSTRAP=1`, `RUSTFLAGS` adding `+atomics,+bulk-memory` and the
shared, imported memory (at most 4 GiB, the same wasm32 ceiling as the
single-threaded module) with wasm-bindgen's TLS exports, and
`CARGO_ENCODED_RUSTFLAGS` unset (cargo prefers it to `RUSTFLAGS`). It
builds into `<target>/wasm-mt` (`scripts/wasm-lanes.sh --target-dir`
prints it), so its flags never invalidate the single-threaded build, and
needs the `rust-src` component. rustc's `unstable feature specified for
-Ctarget-feature: atomics` warning on every crate is expected.

There is one lanes module per game profile (docs/web-export.md
§Variants): `scripts/wasm-crates.txt`'s `lanes` lines build
`og_host_web-lanes2d.wasm` and `og_host_web-lanes.wasm` in
`target/wasm-mt`, sharing std and every dependency.

wasm-bindgen writes `snippets/` (the pool's Worker script) beside
`origin-engine-lanes.js`; build.sh copies each lanes module with its
`snippets/` into its profile's directory (`dist/web-publish`,
`dist/web-publish-2d`), and `scripts/pack.sh` refuses a package without
either profile's lanes files. build.sh rewrites wasm-bindgen-rayon's
positional `pkg.default(module, memory)` call to the single-object form
`{ module_or_path, memory }` (`packages/engine/scripts/rayon-worker.sh`,
which stops the build when the helper has neither form), and
`tests/web-export.test.ts` checks the shipped scripts for a positional
init. Under the gate (`OG_GATE=1`) build.sh exits 1 when
`scripts/wasm-crates.txt` marks og-host-web `lanes` and either lanes
module is missing.

A Rust panic inside a lane aborts the pool Worker it runs on. The engine
thread waits on og-lanes' own latch, which a recorded panic rings: the
engine thread then panics with the lane's message and the page gets a
structured `panic` error naming the tick and the lane's site
(docs/web-export.md §Errors). `bun tests/web/lane-panic.ts` (in
`web-reference`) injects a panic into a lane (`og_web_lanes_fault`) and
needs the crash on the page within 5 s, then the same fault
single-threaded.

## `packages/engine/scripts/build.sh` outside the gate

Standalone (`bun run build`, `bun run test` in `packages/engine`),
build.sh does what it does under the gate whenever the gate has built in
this target directory (`<target>/og-wasm/dev-all.txt` and the dev addon
and `og` exist): the dev addon and `og`, the gate's wasm, and one line
saying so. It compiles release and its own wasm only with `OG_RELEASE=1`,
with `--linux-x64`, or when the gate has not built here; a release build
holds the gate lock. `OG_RELEASE=1` calls `scripts/release-wasm.sh`, including
the shipping-size check before packing. The publish and lanes modules it
copies from `target/og-wasm/dev/` are byte-identical to those the size gate
measures. `native/build-profile` records `dev` or `release`;
`scripts/pack.sh` runs `OG_RELEASE=1 build.sh` first and packs only
`release`.

## Switches in `scripts/budgets.txt` and `scripts/wasm-crates.txt`

- `wasm-crates.txt` lists the crates the wasm step builds. `required`
  crates build in one cargo invocation and fail the gate on error;
  `pending` crates build one at a time and a failure prints `WARN` (while
  a crate is being ported; the step prints a `NOTE` once it builds). A
  crate builds a `.wasm` only with `crate-type = ["cdylib", …]`.
- A line `variant NAME CRATE FEATURES` builds CRATE's publish variant
  once per line: `--no-default-features --features FEATURES` into
  `<target>/wasm-NAME`, raw name `<crate>-NAME.wasm`, bindgen'd as
  `origin-engine` and measured by the size gate as its own download.
  og-host-web has two, one per game profile (docs/web-export.md
  §Variants): `publish` (`publish,physics3d,shaping`, `dist/web-publish`)
  and `publish2d` (`publish,physics2d,profile2d,shaping`,
  `dist/web-publish-2d`); `build.sh` under the gate stops when a declared
  variant's wasm is missing.
- A line `lanes NAME CRATE FEATURES` does the same for the lanes variant
  (§The lanes wasm), through `scripts/wasm-lanes.sh`, raw name
  `<crate>-NAME.wasm` (NAME starts with `lanes`), bound as
  `origin-engine-lanes` and measured as its own download. og-host-web has
  two, in build order: `lanes2d` and `lanes`, with their publish variant's
  features. Without `lanes` lines a crate marked `lanes` builds one lanes
  module with `--features publish`.
- `wasm_artifacts optional|required`: with `required` the size gate fails
  when the wasm build produced no artifact.
- `drift_check optional|required`: with `optional` a missing `og api
  check` prints `SKIPPED: og api check not available yet`; with `required`
  it fails the gate.

## Switches in the environment

- `OG_RENDERER=none` takes the renderer away natively: og-render's
  `Renderer::new` and `with_surface` answer `NoAdapter` ("OG_RENDERER=none:
  this process runs without a renderer", `og_render::renderer_disabled`),
  as on a machine with no usable adapter. The platform's `engine` and
  `probe` sandboxes set it (docs/design.md §5). It is optional: an
  unavailable adapter warns with wgpu's error and continues as tier `none`
  (`attachRenderer`, packages/engine's node/index.ts); the switch only
  removes the warning's "unexpected" wording. Without a renderer the run is
  the same run: the simulation, the probe, `dump_*`, snapshots and the
  world hash are identical (the native host keeps the play surface at
  `--size`); a `screenshot` step writes no PNG and logs `<t>s screenshot:
  no renderer, frame unchecked (<file>)` (og-probe's `FRAME_UNCHECKED`);
  `hello.tier` is `none` and `capture` answers null. `og run --json`
  reports `tier: "none"`, `renderer: "none"`, `adapterError`, the frames in
  `unchecked` and `uncheckedCount`; `og test` runs every check outside
  `run.frame`, prints one `~ <file>: no renderer, frame unchecked` line per
  screenshot and counts unchecked suites separately from passes; exit
  status is zero unless an executed check failed. `--require-checked` on
  `og run`, `og test`, `og check`, `og ui check` (also preview), `og frame
  measure`, and `og doctor` opts into exit 1 when checks remain unchecked.
  Their JSON summaries use `uncheckedCount`, including zero when no check
  was skipped. The `og test` aggregate retains `unchecked` as a deprecated
  alias; per-playtest `unchecked` remains a list of files. `og doctor` reports the
  renderer `none`, not as a problem. Any other value leaves the adapters
  alone; the web host always has its canvas's. The gfx suites register
  their GPU tests as skipped under the switch.
  `packages/engine/tests/no-renderer.test.ts` and og-runtime's
  `tests/no_renderer.rs` hold the contract.
- `OG_GATE_JOBS`, `OG_GATE_MACHINE_LOCK`, `OG_GATE_MACHINE_DIR`
  (§Running the gate locally), `OG_WASM_PROFILE` (§Lanes),
  `OG_BUN_TEST_TIMEOUT`, `OG_UI_STYLES_JOBS`, `OG_UI_KIT_JOBS`,
  `OG_WEB_JOBS` (§What runs where).

## Browser reference tiers

The browser tier is WebGPU in headless Chrome for Testing. Reference
frames are recorded on `metal`; a tier's references are looked up frame by
frame: a frame with no file under `references/<tier>/` is judged against
`references/metal/`, and a frame on neither fails the run ("no reference
at …; record one with --record"), never passes. A tier gets its own
reference only for a difference it makes by design, never to absorb a
defect. Tolerances follow §Reference frames: the tight one against the
tier's own frame, the cross-platform one against `metal`'s. `--record`
writes the tier's references.

`web-reference` runs four groups (`WEB_GROUPS` in `scripts/gate/commands.sh`):
- **ui**: the accessibility mirror (§The accessibility mirror in the
  browser).
- **gfx**: og-tests' and og-ui's scenes (§og-tests' gfx scenes in the
  browser).
- **vehicle-terrain** (`tests/fixtures/vehicle-terrain`,
  its `tests/drive.og.ts`): reference.ts with `--project <dir> --playtest
  <file> --out dist/test-reference/<name>`: the script and shots are read from the
  playtest itself (its commands without screenshots, `quit` and `dump_*`,
  each shot at its screenshot's tick, 60 t), so a retimed playtest moves
  its browser frames with it; `--print` shows what a run would play.
- **lane-panic**: `bun tests/web/lane-panic.ts` (§The lanes wasm).

A missing Chrome is an error, never a skip.

### og-tests' gfx scenes in the browser

`packages/engine/tests/web/reference-gfx.ts` renders og-tests' gfx scenes
(`crates/og-tests/src/scenes.rs`) in headless Chrome: `crates/og-tests-web`
compiles them to a wasm-bindgen module (glTF and font assets embedded with
`include_bytes!` on wasm32), opens each on an OffscreenCanvas at 1280×720
on WebGPU, draws its frames as the native harness does
(`og_tests::gfx::render_built`) and reads the target back. Each frame is
compared with `crates/og-tests/references/webgpu/`, else `metal`, under
§Reference frames' rule, and every frame's figures against `metal` are
kept in the report. It builds only `og-tests-web`, in `wasm-test`, under
`target/wasm-gfx`, so the gate's og-render artifacts stay put. Explicit
`--time` or `--perf-out` runs use the shipping `wasm` profile instead.

Options: `--scene a,b` selects scenes, including the cost scenes built but
not in `SCENES` (`COSTS`: `point-shadows-cube`, `shadows-local-8-cube`,
`shadows-local-8-paraboloid`, `shadows-soft-off`, `aa-off`, `taa-motion`)
and the perf-only city scenes (`city-street`, `city-aerial`); `--time <n>`
writes per-frame times of the perf scenes to
`crates/og-tests/perf/<tier>-chrome-<adapter>/`; `--cpu-culling` makes
WebGPU cull on the CPU (`Culling::Cpu`); `--perf-out <dir>` writes the
timing JSON to `<dir>/<tier>[-cpu-culling]/` with the last frame's culling
counts. The native counterpart is `og-gfx <scene> --frames <n>`, with
`OG_GFX_CULLING=cpu|gpu|auto` (auto: the GPU path from 16,384 instances).

The same module carries og-ui's reference scenes as `og-ui/<name>`
(`sample-screen`, `text-glow`, `glass`, `widget-set`, `text-fields-2`),
built by `crates/og-ui/tests/support/scenes.rs` (included into
og-tests-web by `#[path]`) and judged against
`crates/og-ui/references/<tier>/<name>.png`, else `metal`; and the four
cooked-font text frames (`og-ui/text-bidi`, `text-kerning`, `text-edit`,
`text-mixed-runs`, `crates/og-tests-cooked/tests/support/text_scenes.rs`).
The module cannot cook, so the runner first cooks
`crates/og-ui/tests/text-fonts` with `tests/fixtures/ui-text/assets/fonts` through
`og cook` (`tests/web/text-fonts.ts`, the `og` of `packages/engine/native/`
or `$OG_BIN`), serves the blobs beside the module and hands each page the
manifest and files (`og_gfx_text_manifest`, `og_gfx_text_file`), as the
native `text_gfx` test decodes the same cook.

### The accessibility mirror in the browser

`packages/engine/tests/web/a11y.ts` runs the dev export of its `tests/fixtures/a11y`
on a 1280×720 canvas, one tick per `frame` call, on the settings screen,
and checks the shell's DOM mirror of `og.ui.a11y()` (src/web/a11y.ts): one
element per node in tree order, nested by parent, at each node's bounds,
with its role; the settings dialog, the section tabs, the Master slider,
the Mute switch and Done as ARIA; Chrome's own accessibility tree
(`Accessibility.getFullAXTree`) reading the switch and the slider; no tree
in the frame answers of a settled screen; a screen-reader-style
`element.click()` on Mute toggling it (dump_ui's `checked`, the mirror's
`aria-checked`, the UI's focus and the DOM focus); the Graphics tab and
Done clicked the same way, the "Settings saved" toast in the polite live
region; `prefers-contrast: more` changing the frame and its end bringing
it back; and the pause dialog arriving at full size under
`prefers-reduced-motion: reduce`. It writes
`dist/test-a11y/out/a11y-<tier>.json`.

## Updating a budget

Budgets come from `docs/design.md` §7 and live in `scripts/budgets.txt`.
`wasm_core_brotli_bytes` is per download: a page downloads one engine
module (the 3D or 2D publish variant by the game's profile, or that
profile's lanes variant on an isolated page; docs/web-export.md
§Variants), so each is held to it and the size gate's figure is the
largest (`max_brotli_bytes` in `size-gate.json`, the `max` line of its
table), never the sum. The table prints each variant's brotli size and its
share of the budget. The budget is 16 MB brotli per download, one ceiling
for every variant. It catches accidental growth (a dependency pulled in by
mistake, monomorphisation, doc strings in a publish build) and is never a
reason to cut a feature. Change the number in its own commit, with the
measured value from `target/evidence/size-gate.json` and the reason in the
message; a raise above a design §7 target changes design §7 in the same
commit.

## The model bucket

og.motion's weights, which `packages/engine` ships, are not in git.
`packages/engine/motion.json` pins the file (`file`, `sha256`, `bytes`,
`key`, `url`). Every machine reads it anonymously from `url`, the public
Cloudflare R2 bucket `origin-engine-models` (it holds only pinned model
files and is not listable), so no credentials are needed to fetch. The
private S3 bucket `origin-engine-showcase` on OVHcloud Object Storage
(endpoint `https://s3.bhs.io.cloud.ovh.net`, region `bhs`) keeps the same
file under `motion/<sha256>.safetensors`, as the fallback and the upload
target; its credentials are `OG_SHOWCASE_S3_ACCESS_KEY` and
`OG_SHOWCASE_S3_SECRET_KEY`, and `OG_SHOWCASE_BUCKET`,
`OG_SHOWCASE_S3_ENDPOINT` and `OG_SHOWCASE_S3_REGION` override them. A new
pin is put in both: `upload` for OVH, then `wrangler r2 object put
origin-engine-models/<key> --file <file> --remote`.

`bun packages/engine/scripts/motion-model.ts`:

- `ensure`: install the pin at `packages/engine/<file>`: copy and verify
  `OG_MOTION_MODEL` when set, else reuse a local copy whose size and
  sha256 match, else fetch it. Development builds (including `OG_GATE=1
  --native`) and `pack.sh` run it; a missing or stale model whose fetch
  fails stops the build, rather than letting motion scenes draw og.anim
  poses. Native-only release jobs do not fetch it. A bare `cargo build`
  does not: run `ensure` before running scenes with it.
- `verify`: read the remote key and check its size and sha256, even with a
  local copy; run it when changing the pin.
- `fetch <out>`: download the pin to `<out>`, checked against its sha256.
- `upload <file>`: put `<file>` in the bucket, refused unless it is the
  pinned file; a key already there at the pinned size is left.

A second private bucket in the same project, `origin-generated-assets`
(versioned, encrypted; puts send `x-amz-server-side-encryption: AES256`),
keeps generated source assets for later model runs; its layout is in its
`README.md`.

## Re-capturing gfx references

gfx tests in `og-tests` render a scene headless and compare it with a
committed PNG (per-pixel tolerance plus an RMSE threshold). On mismatch
they write `<scene>.actual.png` and `<scene>.diff.png` under
`$OG_EVIDENCE_DIR/gfx/`. When a rendering change is intended:

```
OG_CAPTURE=1 cargo test -p og-tests          # rewrite the references
git diff --stat tests/                       # see which frames moved
```

A scene with no reference on the adapter's tier is compared with
`references/metal/` under the cross-platform tolerance (§Reference frames)
and the test says so (`gfx <scene>: no <tier> reference …; comparing with
metal's`); a scene on neither fails. Commit the new references with the
change and describe what each moved frame now shows (AGENTS.md §Building
and testing). Record on macrunner (Metal); a frame from another tier never
replaces a Metal reference. A change that moves a frame can be checked on
studio-win first, but its references are re-recorded on macrunner.

## When the size gate fails

1. Read the table `size-gate` printed (or `target/evidence/size-gate.json`):
   which wasm grew and by how much.
2. Find the growth: `cargo tree -p og-host-web --target
   wasm32-unknown-unknown -e normal` for new dependencies, and
   `scripts/wasm-size-report.sh target/og-wasm/raw/<name>.wasm` for where
   the bytes are: the bindgen module as it ships, then per crate and for
   the largest functions, each with its brotli beside its raw bytes.
   Bindgen retains function names. Rank by
   brotli: generic copies compress to little more than one copy.
3. Prefer removing the cause: drop or feature-gate the dependency, disable
   default features, move debug-only code behind the non-`publish`
   configuration, avoid monomorphisation blow-ups. A stable sort in engine
   code is `og_core::sort::{sort, sort_by, sort_by_key}`, and a `BTreeMap`
   or `BTreeSet` built from an iterator is `og_core::sort::{map, set}`: the
   standard library's `sort_by` and `collect` into a B-tree monomorphise
   driftsort per element type and closure. A publish build carries no
   reflected or record field docs (og-core's and og-api's `publish`), so a
   new doc string costs a publish module nothing.
4. When the growth is intended, update the budget (§Updating a budget)
   rather than cutting the feature (design §7).
