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
pushed tree, never with a force push. A head whose tree passed the whole
`full` lane with `--all` (the record `target/og-gate/green/lane.full`,
which `scripts/gate/run.py` writes only when every step ran and passed)
skips the opening quick; the recheck still runs what a rebase changed.
The queue does not span orbs: when a lander elsewhere pushes first, the
push is retried on the new main, without a gate when what landed is
disjoint from the head's inputs and after a recheck otherwise, until it
lands or the recheck fails. Conflicts confined to generated
files (`og_idl::OUTPUT_DIRS`, the UI theme) are regenerated, and so are
conflicts in the wholly generated CSS checklist, docs/css-coverage.md,
once `scripts/css-coverage.py` is merged, both sides kept where both added
whole proof or MISSING entries at one place; the recheck then runs the
steps this changed. A branch from before the checklist left docs/ui.md
has its edits to the old `css-coverage` region dropped, and the new file
regenerated in their place. Any other conflict, including one in
docs/ui.md's written text, stops for the author. `python3 scripts/test-landing.py` exercises
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
| TypeScript, tsgo (`check-conformance`'s comparison and `og check`'s fallback where tsrs has no package; no project or package depends on it) | 7.0.2 | `TSGO_VERSION` in `crates/og-cli/src/commands/check.rs` (`scripts/tsrs-diff.sh` and `.agents/setup` read it) |
| tsrs (the type checker: `og check`, the check before `og run`/`test`/`build`/`dev`, `packages/engine`'s `typecheck`, og-idl's declaration test) | 0.18.0-ts7.1.0-dev.20260929 (TypeScript 7.1.0-dev.20260930.4) | `TSRS_VERSION` and the five SRIs in `scripts/install-tools.sh`; `packages/engine/package.json` `optionalDependencies` copy it (`check-conformance` holds them equal). It moves with a tsrs release, all of these in one commit, after `scripts/tsrs-diff.sh` passes on the new binary |
| cargo-nextest | 0.9.148 here | not pinned |
| ShellCheck (CI lints `scripts/*.sh` with it; Ubuntu's 0.9 reports what 0.11 does not) | 0.11.0 | `SHELLCHECK_VERSION` in `scripts/install-tools.sh` (Homebrew's on macOS) |
| node (`determinism`'s wasm32 runs) | 24 in CI | `.github/actions/provision` |
| sccache (the compiler cache, run by og-sccache-wrap; not used in CI) | 0.19.0 | `SCCACHE_VERSION` in `scripts/install-tools.sh` |
| og-sccache-wrap (the rustc wrapper, run by `scripts/rustc-wrapper`, which `.cargo/config.toml` names) | this checkout's | `crates/og-sccache-wrap`, built and installed by `scripts/install-tools.sh` on every run |
| mold (Linux native linker) | distribution package | `.agents/setup`, `scripts/install-tools.sh`; `.cargo/config.toml` selects it |
| cargo-zigbuild, zig (`build.sh --linux-x64`, release packing's Linux x86_64 addon and og against glibc 2.31) | 0.23.4, 0.17.0 | `CARGO_ZIGBUILD_VERSION`, `ZIG_VERSION` and their sha256 in `scripts/install-tools.sh` (Linux x86_64 only); outside the gate |
| minisign (signs and checks a release's `SHA256SUMS`) | 0.11, the distribution's | `.agents/setup`; Homebrew's on macOS |
| Chrome for Testing (`make web-reference`, browser tests) | the newest agent-browser install | `CHROME`, else `~/.agent-browser/browsers/chrome-<version>` (`tests/web/cdp.ts`); `libnss3`, `libnspr4` and unprivileged user namespaces from `.agents/setup`; outside the gate |
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

### L4 milestone runner

`scripts/gpu-env.sh` holds the Vulkan/NVIDIA pins formerly kept only in
`/home/user/gpu/env.sh`. It sets no gate overrides, so it can also be sourced
for landing. The end-of-round `batch3.sh` is now
`scripts/gpu-milestone.sh`; neither setup nor landing starts it. Start it
only when the coordinator authorizes the milestone:

```sh
amp orb service start gpu-milestone --command 'bash scripts/gpu-milestone.sh'
```

The runner fetches and pins one full `origin/main` SHA, creates clean sibling
worktrees `l4-gate` and `l4-perf` if absent, and refuses to overwrite local
changes. It leaves old integrator worktrees and `/home/user/gpu` untouched.
In order: clean targets; full gate with `OG_WASM_VARIANTS=all` (rust-gpu
included); release `og` and screens for default, fantasy,
scifi and cartoon; ignored og-ui perf against `2fb1c4819`; release native
town bench twice against `6cc20b167`. GPU work uses `scripts/lock.sh`.
The dense-menu budget is 0.85 ms. Reference movements are reported for
quality review, not used as a pixel-difference landing gate.

Logs and command exit statuses live in `.og/gpu-milestone/<run>/`; review
screens and town profiles in `.amp/in/artifacts/review/<run>/`. Each run
gets its own directory. Failed stages are logged; a failed build never
runs a stale binary. Completion still needs the integrator's frame review
(no black, broken or clipped frames; never record one) and Chinese report
to the coordinator. On an idle host also check
`bold_rich_text_is_heavier_ink_never_a_box` and `scripts/test-landing.py`'s
30-second timeouts. Performance fixes follow the milestone measurement.
The supervised service stays alive after `batch-done` for log inspection.

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
| `quick` | fmt, clippy, native test build, CPU tests, GPU tests outside og-tests, generated API drift, engine TypeScript checks, non-engine package tests, the lanes wasm type check, the wasm32 check of the wasm-only crates, the CSS checklist |
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
reuse them. The run's `timings.json` records step start, outer resource
wait, command wall, log bytes and the logical bytes of known published
products. Cargo's internal waits count as command wall; product bytes
are not peak disk usage.

On the Linux L4, `scripts/check.sh full --resume RUN_ID` replays a prior
forced run with the same step selection (repeat `--only` if used). The run
ID is the timestamp-pid directory name printed next to `logs`. Bun is
reused only after a complete checked pass; browser receipts are per file.
Reuse is printed with the source run. A missing receipt or different
tree, environment, tools, device, browser or product bytes runs the test.
Prerequisites still run, and neither `--all` nor the landing's full-lane
stamp accepts reused results as a new whole-lane pass.

`wasm-bindgen` logs the wall of each bindgen and split operation. For L4
measurements, `OG_WASM_PRODUCT_CACHE=1` opts into one validated copy per
operation and variant in `target/og-wasm-products/`. It is off by default
and not used in release packing. Compare the uncached, fill and hit walls
and disk totals on the same tree and target; don't clear the warm Cargo target.

`OG_GPU_DOCUMENT_SHARDS=1` is an L4 comparison-only experiment: nextest
runs four disjoint shares of the same sorted document list through the same
reference checks, within the existing GPU group. Compare against the serial
sweep on the same tree and jobs using `rust-gpu.junit.xml` and overall wall.
The serial sweep is the default. `browser.log` reports each browser file's wall.

`OG_GATE_CARGO_UNITS=1` captures the native test and addon/CLI Cargo unit
graphs without compiling either graph an extra time. Compare features,
effective profile flags and dependencies of common units before treating
two artifacts as equivalent. For L4 profile A/B, compare the existing
incremental setting with `CARGO_PROFILE_DEV_INCREMENTAL=false` on equivalent
changed and replay inputs; collect Cargo walls, cache stats, target size and
test walls. Keep the default profile until the full wall and disk results
justify a change; do not clear the landing target merely for the experiment.

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
| `preflight` | Runs the engine TS configs and toolchain tests before native compilation, and verifies the wasm tools before a functional wasm build when selected. | quick, full or a scoped prerequisite |
| `fmt` | `cargo fmt --all --check` | quick, full |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | quick, full |
| `rust-build` | nextest builds and lists native test binaries; quick excludes og-tests. Then `cargo build -p og-host-node -p og-cli`, the dev addon and `og` every later step uses; `packages/engine/scripts/build.sh` under the gate copies them and runs no cargo. | quick, full |
| `rust-cpu`, `rust-gpu` | nextest executes the discovered tests with separate reports and bounded workers. | quick, full |
| `doc-tests` | `cargo test --workspace --doc` | full |
| `drift` | The dev `og api check` checks generated declarations, wiki, schema and Rust glue against the IDL. | quick, full |
| `native-ready` | Prepares the engine package's native addon, CLI and assets once for the selected consumers. | quick, full or a scoped prerequisite |
| `api-surface` | Compares the live native API to the IDL, in both directions, without a GPU. | quick, full or a scoped prerequisite |
| `bun-quick` | Non-engine package tests, without rebuilding native artifacts. | quick, full |
| `engine-typecheck` | Native `og check tsconfig.runtime.json` exercises checker dispatch and config handling. Preflight already checks all three configurations (API, runtime, templates). | quick, full |
| `wasm-lanes-check` | `scripts/wasm-lanes.sh check` of the non-frames `lanes` variants when their inputs change. If this run successfully built all variants with the same flags and features, that build is the proof. | quick, full |
| `wasm-check` | Check standalone default-feature `og-render` and `og-tests-web` for wasm32. With them, check og-host-web with none of og-runtime's optional modules (3D and 2D) and with each one left out, which no shipped variant builds. | quick, full |
| `wasm-build` | Build `scripts/wasm-crates.txt` modules in `wasm-test` into `target/og-wasm/raw`. `OG_WASM_VARIANTS` picks them: `dev` (the development module alone: bun-test's cross-host tests, a landing), `reference` (adds the 3D publish pair, web-reference alone), `all` (every publish and lanes variant: a run with `browser`, and `scripts/release-wasm.sh`). Tests that load a publish or lanes module skip without one. | full |
| `wasm-bindgen` | Bindgen-only modules for all variants in `target/og-wasm/dev`. `dev-all.txt` lists every module; `dev.txt` selects publish and lanes variants. | full, web-reference prerequisite |
| `web-ready` | Packages the prepared native and wasm products once for the selected web consumers. | full or a scoped prerequisite |
| `size-gate` | Brotli-11 bytes of each shipping-profile bindgen module in `dev.txt`, checked against `wasm_core_brotli_bytes` before packing. | release only |
| `bun-test` | Package tests using the dev addon and gate-produced wasm (`OG_GATE=1`), after engine typecheck, drift and surface checks. A scoped Bun landing also selects the native suites, but Bun doesn't wait for their results. | full |
| `fixtures` | `og test` for each changed runnable feature fixture and the creation skeleton; needs only the native `og`, so `--only fixtures` builds no wasm. | full |
| `browser` | Every file packages/engine's `test:browser` names, each run even when one before it failed. | full |
| `web-reference` | Headless Chrome WebGPU groups with `OG_WEB_JOBS` concurrency (default 3), compared under unchanged tolerances. | explicit |
| `check-conformance` | `scripts/tsrs-diff.sh`: compare tsrs and pinned tsgo diagnostics over packages, templates, the test fixtures and the error corpus. | explicit |
| perf | `scripts/perf.sh`: ignored `perf_*` assertions in og-tests, og-runtime and og-audio, release profile, serially under every GPU slot. An og-tests baseline holds the frame's milliseconds and, where the device has timestamp queries, each render-graph pass's GPU milliseconds (`passes_ms`): a pass over its own baseline by 15 % and 0.1 ms fails by name. | explicit |

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
dependencies, then attempts the existing bounded build prewarm; it does not
run tests or the gate.
`.agents/environment` points `CHROME` at the orb's agent-browser Chrome,
which is not claimed pixel-equivalent to the reference Chrome. Setup
reuses an existing browser; if none is present (including a fresh gcp-l4
orb), it installs agent-browser and Chrome with its Linux dependencies.
It lifts AppArmor's user-namespace restriction for
its sandbox; `.agents/resume` lifts it again on a resumed VM. Validate
changes with `shellcheck -s bash .agents/{setup,resume,environment}`,
`time .agents/setup` twice, and the tools from a new `bash -lc` shell.

## The compiler cache

sccache keeps a local disk cache on every host and no remote cache,
because remote hits were slower than local rebuilds. The checkouts on one
machine share it: `/Volumes/app/.sccache` on macrunner,
`SCCACHE_DIR=E:\sccache` on studio-win, `~/.cache/sccache` on an orb.

One sccache server per host serves every checkout. `.cargo/config.toml`'s
rustc wrapper is `scripts/rustc-wrapper`, a path in the checkout, so a
machine without `scripts/install-tools.sh` (a new orb, a plain Mac) builds
uncached with no override: it runs `og-sccache-wrap` when that and sccache
are on the PATH and rustc directly when not. On Windows, which runs only an
`.exe` as the wrapper, cargo finds `scripts/rustc-wrapper.exe`, the copy of
`og-sccache-wrap` that install-tools.sh writes there; `og-sccache-wrap`
itself also runs rustc directly when sccache is missing.
`og-sccache-wrap` (`crates/og-sccache-wrap`) starts
that server without the jobserver of the cargo that happens to need it
first, and on Unix stops a server that has stalled for `OG_SCCACHE_STALL`
seconds (default 60, 0 turns the watch off). On Windows it runs rustc
itself, uncached, for a compile whose `@argfile` would expand past the
command line limit, which sccache 0.19.0 cannot spawn (os error 206);
its source comments and
`tests/start.rs` hold the details. `RUSTC_WRAPPER=` (empty) bypasses the
wrapper and sccache. To restart a server, run `sccache --stop-server`; a
server started by hand must go without the jobserver: `env -u
CARGO_MAKEFLAGS -u MAKEFLAGS -u MFLAGS sccache --start-server`.

The wrapper also makes the release wasm reproducible on any host, so the
remote-frame archives pinned by hash rebuild to the same bytes on Linux and
macOS. Cargo's `-C metadata` takes in the host triple through every
proc-macro and build script a crate depends on, and rustc derives crate ids,
mangled names and type ids from it; `scripts/release-wasm.sh` sets
`OG_WASM_REPRODUCIBLE`, under which each wasm32 unit gets a crate id of its
own identity (package, version, crate types, `--cfg`s) and the cargo and
rustup homes, build-std's sources and the checkout are remapped. It refuses
to run with the wrapper bypassed or older than that.

## When to run what

While writing code run `cargo check -p <crate>` and that crate's tests;
`quick` is the most a worktree runs, `full` follows a batch of landings,
and `--all` is never run after every edit. After re-recording a reference
frame, rerun only that frame's lane: `--only rust-gpu` for og-tests' gfx
references, `--only bun-test` for a template's playtest frames,
`web-reference` for the browser tiers.

## Fuzzing

`crates/og-ui-fuzz` holds og-ui's fuzz targets, run with cargo-fuzz
(`cargo install cargo-fuzz`), outside the gate: `cargo fuzz run --fuzz-dir
crates/og-ui-fuzz -s none -a document`, seeded from the engine's
documents. `document` parses a document's text, as a remote `<iframe>`
delivers it, then mounts and builds it. A crash it finds is fixed at the
root and its input, minimised with `cargo fuzz tmin`, goes in
`crates/og-ui-fuzz/regressions/document/`, which the crate's test
(`cargo test -p og-ui-fuzz`, in the gate's tests) replays.

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
this target directory, and says so. It incrementally rebuilds the dev addon
and `og` before copying them, including with explicit `OG_GATE=1 --native`.
`OG_GATE=1` selects dev mode; only the scheduler's successful `rust-build`
entry in `OG_GATE_RAN` permits copying without cargo, so concurrent gate
consumers do not relink binaries other steps are running.
It compiles release and its own wasm
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
  `og test` defaults an unset switch to `hardware`: a software-only adapter
  (lavapipe, llvmpipe or SwiftShader) is reported as unavailable before
  device creation, not silently used to draw every tick on the CPU.
  `OG_RENDERER=software og test` opts into software rendering; it can be very
  slow. Real GPU adapters still render normally. Other commands keep their
  existing adapter selection unless the switch is explicitly set.
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

## The asset store

The repository's binary assets live outside Git, in one store the platform
hosts, addressed by SHA-256 (`GET /api/engine-assets/sha256/<hex>` on
origingame.ai; public objects are read anonymously, private ones and uploads
need a platform key the store lets write): the reference frames, the vfx
packs, the engine package's extensions, sheets and vendor files, the
templates' fonts, styles and look art, the test fixtures, og-tests' assets,
og.motion's weights (read by cooks through `motion.json`'s `url`), the
remote-frame archives (`frame-hosts.json`, pushed by `scripts/release.sh
publish`) and every showcase project's assets. Git keeps only the binaries a
bare cargo build compiles in, the knowledge package's previews, the install
test's sample and vendored `third_party/` data; `scripts/assets.test.ts`
fails on any other. Engines up to 0.9.0 read the frozen public R2 bucket
`origin-engine-models` instead, which stays until no supported release
names it.

A manifest that owns files a package ships sits above every directory that
is packed or copied (`packages/engine/`, `packages/vfx-packs/`,
`templates/`): `bun pm pack` applies a nested `.gitignore` even inside a
`files` entry, and a pack's archive would carry a nested `og-assets.json`.

A directory that owns pinned files keeps an `og-assets.json` beside them:
`{"version": 1, "files": [{path, sha256, size, visibility, source?, licence?}]}`,
`path` relative to that directory. It names no URL. The tool keeps a marked
block in the same directory's `.gitignore` listing the pinned paths, so a
checkout after `ensure` is clean.

```sh
bun scripts/assets.ts ensure [dir...]                    # install every pin under dir (default: the repository)
bun scripts/assets.ts verify [dir...]                    # ask the store that every pin is present at its size
bun scripts/assets.ts pin <dir> [--private] <path...>    # pin files as they are now
bun scripts/assets.ts push [dir...]                      # upload what the store lacks
bun scripts/assets.ts status [dir...]                    # list pinned files that are missing or modified
```

`ensure` reads the shared cache `$ORIGIN_ENGINE_CACHE/blobs/sha256/<ab>/<hex>`
(default `~/.cache/origin-engine`), then `OG_ASSETS_MIRROR` (a directory or
URL with the same layout), then the store, checks every file against its
pin and names each one it cannot have; a verified cache works offline.
`push` and private reads use `OG_ASSETS_KEY`, else
`OG_PLATFORM_DEVELOPMENT_KEY`; `push` refuses a file that is not its pin.
`OG_ASSETS_URL` overrides the store's origin.

`.agents/setup` and `scripts/check.sh` run `ensure` before anything reads
or compiles a pinned file. Pinned files are ignored by Git, so
`scripts/land.sh` and the release pack also run `status`: a pinned file
that is not its pin (a re-recorded reference, an edited texture) refuses a
landing and marks a pack dirty until it is pinned again and pushed, in the
same commit as its `og-assets.json`. Paths given to `pin` are relative to
its directory (`cd <dir> && bun <repo>/scripts/assets.ts pin . *.png`).

## Generated sources

The raw generated sources behind the shipped assets (Meshy models, gateway
images, training runs, recordings) are private objects in the same store;
reading them needs a key the store lets write. `tools/archive.ts` of a
showcase project puts its own, and `bun scripts/assets.ts pin <dir>
--private <path...>` and `push` pin any other.

Everything the former bucket `origin-generated-assets` held (7,293 keys,
22.9 GB, imported 2026-11-19: `meshy/`, `training/`, `kimodo/`, `scenes/`,
`gateway/`, `repro/`, `showcase-recordings/`) is in the store. Its index is
the private object `ae676abeac60964bb75f614d6830e75964a23e0ad3e15da5f83d1901be0cfaf1`
(1,437,111 bytes): `{version, bucket, imported, files: [{key, size, sha256}]}`,
so a key a README or journal cites, such as `meshy/model/<operation>/model.glb`,
resolves to its object:

```sh
curl -sH "Authorization: Bearer $OG_PLATFORM_DEVELOPMENT_KEY" \
  https://origingame.ai/api/engine-assets/sha256/ae676abeac60964bb75f614d6830e75964a23e0ad3e15da5f83d1901be0cfaf1 |
  jq -r '.files[] | select(.key == "meshy/model/<operation>/model.glb") | .sha256'
``` Ship game-ready
files (meshes simplified and compressed, maps sized for their place in the
frame) as public pins: players download them.

## Re-capturing gfx references

gfx tests in `og-tests` compare a headless render with a committed PNG and
on mismatch write `<scene>.actual.png` and `<scene>.diff.png` under
`$OG_EVIDENCE_DIR/gfx/`. When a rendering change is intended:

```
# on the GPU host, after `scripts/gate/step.sh rust-build` in the full lane
OG_GFX_CAPTURE=1 OG_UI_CAPTURE=1 scripts/gate/step.sh rust-gpu   # rewrite the references
bun scripts/assets.ts status crates/og-tests crates/og-tests-ui    # see which frames moved
git status --short -- '*/references/*'                             # and which are new
```

Reference frames are pinned files (§The asset store): each `references/`
directory's `og-assets.json` pins its frames. Pin the moved and new frames
(`cd crates/og-tests/references && bun ../../../scripts/assets.ts pin . vulkan/<scene>.png …`),
push them, and commit the `og-assets.json` with the batch.

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
