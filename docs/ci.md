# The local gate

The gate is one script, `scripts/check.sh`, run on macrunner; there is no
GitHub CI. The rules it rests on are in AGENTS.md §Building and testing: one
target directory, the dev profile for every native step, tests that never
run `cargo`.

## Landing without gate starvation

Authors land non-visual heads with `scripts/land.sh`; the art runner uses the
same command after approving visual heads. Start with a clean, committed
branch. The command rebases onto fetched main and runs `check.sh quick`.
It then takes the repository-wide landing lock in the common Git directory,
separate from the machine gate lock, fetches main again, and pushes.

If main advanced while quick ran, the command compares both sides from the
tested base. Disjoint file changes reuse the pass only when neither side
touches common build inputs or generated/drift-checked files. Otherwise it
rebases, releases the landing lock, and runs quick again; existing per-step
input records skip unaffected work. No gate runs under the landing lock.
Push rejection caused by another main advance retries the same comparison;
other push failures stop. A rebase conflict stops for the author to resolve.
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
| binaryen (`wasm-opt`) | 133 | `BINARYEN_VERSION` in `scripts/install-tools.sh` (release archive on Linux and macOS) |
| bun | 1.4.4 | `BUN_VERSION` in `scripts/install-tools.sh`; `tests/install/Dockerfile` and `tests/install/instabox.ts` install the same for the install test |
| TypeScript, tsgo (`check-conformance`'s comparison and `og check`'s fallback where tsrs has no package; no project or package depends on it) | 7.0.2 | `TSGO_VERSION` in `crates/og-cli/src/check.rs` (`scripts/tsrs-diff.sh` and `.agents/setup` read it) |
| tsrs (the type checker: `og check`, the check before `og run`/`test`/`build`/`dev`, `packages/engine`'s `typecheck`, og-idl's declaration test) | 0.18.0-ts7.1.0-dev.20260929 (TypeScript 7.1.0-dev.20260930.4) | `TSRS_VERSION` and the five SRIs in `scripts/install-tools.sh`; `packages/engine/package.json` `optionalDependencies` copy it (`check-conformance` holds them equal). It moves only at a wave's end, with a tsrs release, all of these in one commit, after `scripts/tsrs-diff.sh` passes on the new binary |
| cargo-nextest | 0.9.148 here | not pinned |
| ShellCheck (CI lints `scripts/*.sh` with it; Ubuntu's 0.9 reports what 0.11 does not) | 0.11.0 | `SHELLCHECK_VERSION` in `scripts/install-tools.sh` (Homebrew's on macOS) |
| node (`determinism`'s wasm32 runs) | 24 in CI | `.github/actions/provision` |
| sccache (the rustc wrapper, `.cargo/config.toml`; not used in CI) | 0.14.0 | `SCCACHE_VERSION` in `scripts/install-tools.sh` |
| mold (Linux native linker) | distribution package | `.agents/setup`, `scripts/install-tools.sh`; `.cargo/config.toml` selects it |
| cargo-zigbuild, zig (`build.sh --linux-x64`) | 0.23.4, 0.17.0 here | not pinned; outside the gate |
| Chrome for Testing (`make web-reference`, browser tests) | 150 | `CHROME_PATH`; outside the gate |
| Mesa lavapipe (CPU rendering in orbs) | minimum 25.0.7 | `.agents/setup`; Debian bookworm-backports supplies Mesa and apt resolves its dependencies |

Every target and profile uses LLVM, including dev, test, build scripts,
release and wasm; there are no backend overrides. Four frontend threads
(`-Zthreads=4`) are enabled on every target. Metadata embedding and the
build-directory layout use Cargo's defaults. Consumers use Cargo's
artifact messages, not guessed executable paths.

| Native target | Dev/test backend and frontend | Linker | Verification |
|---|---|---|---|
| arm64 macOS | LLVM, `-Zthreads=4` | platform linker | verified on macrunner |
| x86_64 Linux | LLVM, `-Zthreads=4` | mold | unverified |
| x86_64-pc-windows-msvc (studio-win) | LLVM, `-Zthreads=4` | rust-lld (lld-link), not link.exe | **unverified on studio-win** |

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
reference harness falls back to Metal's frames. `crates/og-tests/references/dx12/`
and `vulkan/` (`OG_GFX_BACKEND=vulkan`) hold a frame only where that
backend legitimately differs from Metal (nearest-sampled texels whose edges
fall on pixel centres; NVIDIA's Vulkan anisotropic filter), with the reason
in the commit that recorded it. `og check` and the package's typecheck use
the pinned tsgo through Bun, since tsrs has no Windows package. The
Windows release package is built with `OG_RELEASE=1 sh
packages/engine/scripts/build.sh --native && sh
packages/engine/scripts/pack.sh --native` (docs/release.md §3).

## Linux renderer prerequisites

`.agents/setup` upgrades `mesa-vulkan-drivers` below 25.0.7 (on Debian 12
from bookworm-backports): Debian 12's Mesa 22.3.6 grows resident memory
every frame and is killed under a 4 GiB limit. A warm setup skips apt when
its prerequisites are met. No reference lane runs on lavapipe.

## Lanes

| Lane | Steps | When | Warm budget, Apple M4 Pro |
|---|---|---|---|
| `scripts/check.sh quick` | fmt clippy test-quick drift bun-quick | the edit loop, and what a landing runs | 120 s |
| `scripts/check.sh functional` | fmt clippy test drift bun-functional fixtures | after a batch of landings: native tests and GPU frames, no wasm build (og-tests' wasm32 check of og-render is left out; cross-host comparisons skip on a stale wasm) | 240 s |
| `scripts/check.sh kits` (alias `ui`) | kit-tests ui-styles ui-kit | package-owned UI suites, authored styles, template playtests; no wasm build | 240 s |
| `scripts/check.sh` (full) | fmt clippy wasm-build test wasm-opt size-gate drift bun-test fixtures | a wave's end, with `--all` (`scripts/wave-end.sh`) | 1200 s |

Budgets are warm-run targets, not timeouts or permission to skip checks.
Measure on the same hardware under comparable load; cold installation and
compilation are reported separately. A test that takes a lane past its
budget replaces an existing test. Perf and browser-reference runs are
wave-end measurements with a 1200 s budget each. A regression defaults to
an inline unit test; integration tests are for cross-crate contracts, the
CLI and GPU rendering.

The kits lane: `kit-tests` runs the Bun suites under
`packages/engine/extensions` and the opt-in `kit-tests` Rust targets
(`og-cook/kit-documents`, `og-ui/style-packs`,
`og-tests-cooked/style-diagnostics`); `ui-styles` runs
`templates/ui-kit`'s `tests/style-` playtests, `ui-kit` its other
playtests. `kit-web` (explicit only) runs `templates/ui-kit/scripts/browser.ts`.
The engine lanes keep the generic UI tests (layout, text, input, focus,
layers, motion, drawing, documents, style grammar, common components).
Template references live in `templates/ui-kit/references/<tier>`; style
references in `templates/styles/references/<pack>/<tier>` and style
playtests in `templates/styles/tests/<pack>`, never inside a shipped
`templates/styles/<pack>` asset root.

**Path-aware steps.** A step that passes records the working tree it
passed on in `<target>/og-gate/green/<step>`: a git tree of the whole
working copy, untracked files included, written through a private index.
The next run compares that tree with the current one and skips a step none
of whose inputs changed:

| Skipped | When nothing changed in | Then |
|---|---|---|
| each GPU package's reference tests (og-render, og-ui, og-tests, og-tests-cooked) in test and test-quick | that package's dependency closure (with dev-dependencies) and what it embeds (`third_party/ltc`; with og-cook, `templates/looks` and `templates/styles`); for og-ui and og-tests-cooked also the templates their tests read | the rest runs; the summary names each cached package. Input caching, never selection by host or GPU availability |
| wasm-build, wasm-opt, size-gate | `crates/`, `third_party/` | the last outputs in `target/og-wasm` are reused (only when every file they list exists; a step that ran upstream in this invocation always runs the next) |
| drift | `crates/og-api`, og-idl and the crates it builds on, `crates/og-cli`, and every generated directory (`og_idl::OUTPUT_DIRS`), so a hand edit to a generated file is still caught | |
| ui-styles, ui-kit | og-cli's and og-host-node's closure, `packages/engine`, `templates/ui-kit`, `templates/styles` | |
| fixtures, per fixture (`tests/fixtures/<name>`) | the fixture's directory and the paths in its `inputs` file, og's and og-host-node's crate closure, `packages/engine`, `templates/looks`, `templates/styles` | each fixture whose inputs are unchanged since it last passed is skipped |
| check-conformance (in no lane) | `packages/engine`, `templates/`, `tests/fixtures/`, `crates/og-cli/src/check.rs`, `scripts/tsrs-diff.sh`, `scripts/tsrs-corpus/`, `scripts/install-tools.sh`, `scripts/lib.sh` | |
| web-reference (in no lane) | og-host-web's and og-tests-web's crates for wasm32, `crates/og-cook`, `packages/engine`, `templates/fx-showcase`, `templates/looks`, `templates/ui-kit`, `templates/styles`, `templates/archetypes/third-person`, `tests/fixtures/vehicle-terrain` | |

A change to `scripts/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`.cargo/`, `.config/` or `tests/gpu.rs` reruns every one of them. Each skip
prints its reason, the tree it rests on and its inputs. `--all` runs every
step and every test, and so does `CI` set in the environment. A failed step
clears its record. bun-test and the non-GPU tests always run. bun-test's
slow integration tests (packages/engine's `reload`, `reload-asset` and
`web-export`) run only under `--all` (`OG_GATE_ALL`); a plain `bun run
test` in the package runs them too. A bun test that signals a child by
name uses `process.kill(pid, name)`, never `Subprocess.kill(name)`.

**The wasm profile.** `--all` builds the shipping `wasm` profile
(`Cargo.toml`), so the size it records is the module that ships; the
path-aware lane builds `wasm-gate`, the same profile with 16 codegen units,
faster to build and with modules about 5–6 % larger, so a size the
path-aware lane passes, the shipping build passes. `OG_WASM_PROFILE=wasm|wasm-gate`
overrides; each profile has its own target subdirectory.

wasm-opt and size-gate also keep results by content in
`target/og-wasm-cache`: a raw module byte-identical to one already
optimised (keyed by its SHA-256, the wasm-bindgen and wasm-opt versions and
the flags) reuses the earlier bindgen output and `-Oz` module, and a module
measured before reuses its brotli size. `--all` bypasses that cache.

Outside `--all`, the development `og_host_web` module skips wasm-opt when
the build includes its publish variant. Development already loads the
unoptimised bindgen output. Publish and lanes variants still run wasm-opt,
as does every module under `--all`. A build without publish variants keeps
the development module optimised because the size gate measures it instead.
The content cache distinguishes these modes.

**Build settings the gate relies on.** The dev profile has no debuginfo
(on macOS it stays in object files every Metal process reads;
`CARGO_PROFILE_DEV_DEBUG=line-tables-only` gives backtraces with lines).
wgpu, wgpu-core, wgpu-hal, wgpu-types, naga and the physics and encoder
crates are optimised in dev (opt-level 3), and workspace crates build in
16 codegen units. og-api is three crates (og-idl, og-api, og-runtime;
`crates/og-api/README.md`), so a doc-only IDL edit rebuilds nothing.
sccache is the wrapper in `.cargo/config.toml` (`RUSTC_WRAPPER=` turns it
off). `og-tests::wasm` takes the full lane's `wasm-build` og-render
artifact when it is newer than og-render's sources, so it needs no second
wasm32 target directory.

**One GPU step at a time.** The gate admits one GPU step (test,
test-quick, bun-test, bun-functional, fixtures, kit-tests, ui-styles, ui-kit,
kit-web, web-reference) at a time, while CPU steps run beside it. Each GPU
step keeps its own concurrency (nextest's four-slot `gpu` group, Bun
workers, `og test --jobs`), which must not add up across steps. This
exclusion adds no input dependencies to `--only`.

### Test binaries

Each crate has at most one default integration binary, `tests`, declared
at `tests/it/main.rs` with `autotests = false`. The opt-in `kit-tests`
targets belong to the kits lane. Single-crate checks live inline under
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
| `fmt` | `cargo fmt --all --check` | quick, functional, full |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | quick, functional, full |
| `test-quick` | `cargo nextest run --workspace --exclude og-tests` | quick |
| `test` | `cargo nextest run --workspace` and `cargo test --workspace --doc` | functional, full |
| `drift` | The dev `og api check` checks generated declarations, wiki, schema and Rust glue against the IDL. | quick, functional, full |
| `bun-quick` | Knowledge, create and design package tests, without rebuilding native artifacts. | quick |
| `bun-functional` | Package tests with the last full lane's wasm. Missing or partial wasm outputs fail, never shrink the suite. | functional |
| `wasm-build` | Build `scripts/wasm-crates.txt` variants. Feature variants keep separate caches under `target/wasm-*`; outputs go to `target/og-wasm/raw`. | full |
| `wasm-opt` | Bindgen followed by `scripts/wasm-opt.sh`, except the development module outside `--all` when publish variants exist. `opt.txt` lists optimized size-gate inputs; `opt-all.txt` lists every processed module. | full |
| `size-gate` | Brotli-11 bytes of each optimized module in `opt.txt`, checked against `wasm_core_brotli_bytes`. | full |
| `bun-test` | Package tests and typechecks using the dev addon and gate-produced wasm (`OG_GATE=1`), without a second release build. | full |
| `kit-tests` | Package-local Bun tests and the three opt-in Rust kit targets. | kits, alias ui |
| `ui-styles` | `og test templates/ui-kit --filter tests/style-`, with `OG_UI_STYLES_JOBS` workers, default 3. Any unchecked frame fails. | kits, alias ui |
| `ui-kit` | `og test templates/ui-kit --skip tests/style-`, with `OG_UI_KIT_JOBS` workers, default 4. Any unchecked frame fails. | kits, alias ui |
| `kit-web` | `templates/ui-kit/scripts/browser.ts` against an existing web build. | explicit only |
| `web-reference` | Native and web preparation, then headless Chrome WebGPU groups with `OG_WEB_JOBS` concurrency (default 3). Compare retained scenes under unchanged tolerances. | wave-end, or explicit |
| `check-conformance` | `scripts/tsrs-diff.sh`: compare tsrs and pinned tsgo diagnostics over packages, templates, the test fixtures and the error corpus. | wave-end, or explicit |
| perf | `scripts/perf.sh`: ignored `perf_*` assertions in og-tests, og-runtime and og-audio, release profile, serially under every GPU slot. | wave-end only |

Rust test steps fall back to `cargo test` when nextest is unavailable
(§CI). Missing adapters warn on every host; GPU tests are skipped, not
passed. Bun tests have a 30-second per-test timeout through
`OG_BUN_TEST_TIMEOUT`; a real hang fails. `scripts/test-gate-guards.sh`
checks missing and partial wasm outputs for bun-functional, bun-test and
size-gate. Refresh missing wasm with `scripts/check.sh --only wasm-opt`;
the size gate always reads optimized modules.

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

Without nextest, `cargo test` runs a binary's tests on threads of one
process, and two renderers made on two threads of one process can crash
the Vulkan loader (libvulkan1 1.3.290 to 1.4.344 frees an instance's
unused drivers in its first `vkEnumeratePhysicalDevices` while another
thread walks that list). So the fallback runs og-tests, og-tests-cooked,
og-ui and og-render after the other crates with `--test-threads=1`. No
test makes two renderers at once.

`.github/workflows/release.yml` (`workflow_dispatch` only, no secrets,
nothing published) is the fallback release pack when the Mac or the
Windows host is unavailable (docs/release.md §3).

## Running the gate locally

```
scripts/install-tools.sh             # once; macOS uses Homebrew for brotli
scripts/check.sh quick               # the quick lane (make quick)
scripts/check.sh functional          # the functional lane
scripts/check.sh kits                # the UI package lane
scripts/check.sh                     # the full lane, path-aware; stops at the first FAIL
scripts/check.sh --all               # the full lane with nothing skipped
scripts/check.sh clippy test         # selected steps only
scripts/check.sh --only bun-test     # a step and the steps it reads (here wasm-build, wasm-opt)
OG_GATE_JOBS=1 scripts/check.sh      # one step at a time, in lane order
scripts/check.sh wasm-build wasm-opt size-gate   # the size gate (make wasm)
scripts/check.sh --list
scripts/check.sh web-reference       # the browser tiers (make web-reference); needs Chrome
make check                           # the same through the Makefile
```

The path-aware state is `<target>/og-gate/` (the green trees) and
`<target>/og-wasm-cache/`; both are kept across runs and only bypassed
with `--all`, never deleted to force a step.

One gate per machine, and one per target directory: `scripts/check.sh`
holds `~/.cache/og-gate/og-gate.lock` (`OG_GATE_MACHINE_DIR` moves it;
`OG_GATE_MACHINE_LOCK=0` skips it) and then `<target>/og-gate.lock` (a
release `packages/engine/scripts/build.sh` holds the target's) for its
whole run (`scripts/lock.sh`: mkdir locks with the holder's pid, taken
over when that pid no longer runs a gate). Waiters receive increasing
tickets and print their queue position. Only the oldest live ticket can
acquire the lock. Single-slot and all-slot GPU requests share one FIFO,
so neither can bypass an earlier request. Dead waiters lose their tickets;
dead holders release their locks. `python3 scripts/test-locks.py` exercises
three concurrent waiters, exclusive GPU admission, and dead-owner recovery.
The machine's versioned lock implementation applies to newly started callers
even in older lanes. Already waiting shells must be restarted by their owners
to adopt a new implementation; live holders are never interrupted.

The steps run concurrently, each as soon as the steps whose outputs it
reads have passed or been skipped (`step_needs` in the script): test and
wasm-opt after wasm-build (not in the functional lane), size-gate and
bun-test after wasm-opt, ui-kit after ui-styles. At most `OG_GATE_JOBS`
run at once (2 by default: the native chain beside the wasm chain; 0 lifts
the cap). The gate renices itself by 10. Native cargo builds lock
`target/debug` and the wasm32 ones `target/wasm` and
`target/wasm32-unknown-unknown`, so the two chains overlap.

Each run retains its logs under `<target>/evidence/gate/runs/<UTC-time>-<pid>/`.
`run.txt` records the commit, working tree, selected steps, force-all flag,
and start/end timestamps. Each step writes `<step>.log`, `<step>.time`
(wall seconds), and `<step>.rc` (exit status). A step
prints `--> [n/N] step started` when it starts; when it ends the gate
prints `==> [n/N] step`, the whole log, then `PASS`, `FAIL` or `SKIPPED`
with its duration. The first `FAIL` stops the steps still running (each
reported as `CANCELLED step (Ns): stopped when 'x' failed; partial log …`),
the gate prints `gate FAILED at 'x'` and exits non-zero; the lock is
released only after those steps have stopped, and Ctrl-C stops them the
same way. On success it prints `gate PASSED (n/N steps, Ns)`. The per-step
lines land in `target/evidence/check-summary.txt` and the run's `summary.txt`.
Nextest writes `<step>.junit.xml`: testcase times are execution seconds,
and their sum per binary is not wall time because tests run concurrently.
Configured exclusions are in the log, not JUnit's skipped count. Bun writes
`bun-<package>.timings.json`, with milliseconds per test file. Each run starts
with fresh timing files so telemetry does not change test scheduling. Missing
reports on a failed or cancelled run are not zero-duration passes. The script honours
`CARGO_TARGET_DIR` and exports `OG_EVIDENCE_DIR` (default
`<target>/evidence`) and `OG_WASM_OUT` (`<target>/og-wasm`) for test
harnesses and `build.sh`.

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

## When to run what

- **While writing code**: `cargo check -p <crate>` and that crate's own
  tests. `scripts/check.sh quick` is the most a worktree runs, and what a
  landing runs.
- **After a batch of landings**: `scripts/check.sh functional`. The wasm
  build and its size, determinism and the browser tiers are measurements
  of a wave's end, never held against a landing.
- **At a wave's end**: `scripts/wave-end.sh` (§The wave-end run). Never
  `--all` after every edit; never touch files or delete `target/og-gate/`
  to force a step.
- **After re-recording a reference frame**: rerun only that frame's lane:
  `scripts/check.sh --only test` for og-tests' gfx references (§Re-capturing
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
wave-end perf`. Bun uses `gpuTest` from `packages/engine/tests/support/gpu.ts`,
or `test.skipIf(gpuUnavailable || otherPrerequisiteMissing)`; never return
early from a test body to skip. The gate forwards nextest's
adapter-discovery warnings and does not cache an unchecked GPU run as
green. Do not force unavailable GPU tests with `--run-ignored all/only`
(rejected: nextest would call a runtime skip a pass); use `cargo test --
--ignored` for explicit ignored-test runs, as the perf lane does.

Browser groups use separate ports and profile/output directories; work
that writes the same project's export stays in one group. A serial/parallel
pixel difference is a defect, not a reason to raise tolerances.

## The wave-end run

`scripts/wave-end.sh` runs a wave's measurements one after another under
the machine's and the target's gate locks, held across its stages. Each
stage runs even when one before it failed; the exit status is non-zero
when any did. Logs, the summary and the table go to
`target/evidence/wave-end/`.

| Stage | What |
|---|---|
| `gate` | `scripts/check.sh --all` |
| `determinism` | `tools/wasm-determinism/run.sh`: math, physics, animation and text digests across native, scalar/SIMD wasm and both lanes profiles |
| `perf` | `scripts/perf.sh`, ignored cost assertions under the exclusive GPU slot lock |
| `web` | `scripts/check.sh web-reference` |
| `art` | each authored look's capture and comparison; no new frame without `--record art[/family]` |
| `conformance` | `scripts/tsrs-diff.sh` |
| `release` | `scripts/release.sh pack` then `verify` (docs/release.md §3); publish never runs here |

`--stages gate,perf` selects stages in their defined order; `--dry-run`
prints commands without running them.

The separate determinism workspace runs only in this stage, not in any
`check.sh` lane. Bun's cross-host tests still compare native and shipped-wasm
world hashes in the full gate. This is temporary until those tests cover all
of the standalone digests using the engine builds already present.

## The lanes wasm

The lanes variant of og-host-web (docs/web-export.md §Lanes) is the
publish build compiled for wasm atomics and shared memory, which needs a
rebuilt std: `scripts/wasm-lanes.sh <cargo args>` runs `cargo build
--target wasm32-unknown-unknown -Z build-std=panic_abort,std` with
`RUSTC_BOOTSTRAP=1`, `RUSTFLAGS` adding `+atomics,+bulk-memory` and the
shared, imported memory (at most 2 GiB, against the single-threaded
module's 4 GiB) with wasm-bindgen's TLS exports, and
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
this target directory (`<target>/og-wasm/opt-all.txt` and the dev addon
and `og` exist): the dev addon and `og`, the gate's wasm, and one line
saying so. It compiles release and its own wasm only with `OG_RELEASE=1`,
with `--linux-x64`, or when the gate has not built here; a release build
holds the gate lock. Its wasm-opt is `scripts/wasm-opt.sh`, so the publish
wasm an `OG_RELEASE=1` build ships is byte-identical to the module the
size gate measures. `native/build-profile` records `dev` or `release`;
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
  `t.frame`, prints one `~ <file>: no renderer, frame unchecked` line per
  screenshot and counts unchecked suites separately from passes; exit
  status is zero unless an executed check failed. `og doctor` reports the
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
defect. The default tolerance is og-tests' (`crates/og-tests/src/compare.rs`):
a channel off by more than 32/255 is an outlier; fail above 0.5 % outliers
or an RGB RMSE above 0.01. `--record` writes the tier's references.

`web-reference` runs seven groups (`WEB_GROUPS` in `scripts/check.sh`):

- **fx**: `packages/engine/tests/web/reference.ts --playtest
  tests/frames.og.ts` on `templates/fx-showcase`'s dev export at 1280×720,
  DPR 1, one tick and one render per `frame` call as `og run --commands`
  drives the native host. It takes the playtest's representative frames
  (`post`, `stylised`) at their screenshots' ticks and compares them with
  `templates/fx-showcase/references/webgpu/`, else `metal`. Before them it
  sends `call("probe", "commands", {…})` with an object and requires the
  probe's refusal ("probe.commands: script must be a string, not object")
  from the same Worker.
- **ui**: the accessibility mirror (§The accessibility mirror in the
  browser).
- **gfx**: og-tests' and og-ui's scenes (§og-tests' gfx scenes in the
  browser).
- **ember-vale** (`templates/archetypes/third-person`, its
  `tests/frames.og.ts`) and **vehicle-terrain** (`tests/fixtures/vehicle-terrain`,
  its `tests/drive.og.ts`): reference.ts with `--project <dir> --playtest
  <file> --out dist/test-reference/<name>`: the script and shots are read from the
  playtest itself (its commands without screenshots, `quit` and `dump_*`,
  each shot at its screenshot's tick, 60 t), so a retimed playtest moves
  its browser frames with it; `--print` shows what a run would play.
- **lane-panic**: `bun tests/web/lane-panic.ts` (§The lanes wasm).

A missing Chrome is an error, never a skip. `bun
tests/web/archetype-hashes.ts` (not in `web-reference`) compares the
archetypes' world hashes across hosts: each replay playtest's script runs
in Chrome and in `og run` at the page's play surface, and the two
`dump_world` hashes at the same tick must be equal (`og run` creates the
engine with `deferStart` and starts once the renderer is attached, as the
web Worker does).

### og-tests' gfx scenes in the browser

`packages/engine/tests/web/reference-gfx.ts` renders og-tests' gfx scenes
(`crates/og-tests/src/scenes.rs`) in headless Chrome: `crates/og-tests-web`
compiles them to a wasm-bindgen module (glTF and font assets embedded with
`include_bytes!` on wasm32), opens each on an OffscreenCanvas at 1280×720
on WebGPU, draws its frames as the native harness does
(`og_tests::gfx::render_built`) and reads the target back. Each frame is
compared with `crates/og-tests/references/webgpu/`, else `metal`, under
the scene's own tolerance, and every frame's figures against `metal` are
kept in the report. It builds in `target/wasm-gfx`, so the gate's
og-render artifacts stay put.

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
`crates/og-ui/tests/text-fonts` with templates/ui-kit's font files through
`og cook` (`tests/web/text-fonts.ts`, the `og` of `packages/engine/native/`
or `$OG_BIN`), serves the blobs beside the module and hands each page the
manifest and files (`og_gfx_text_manifest`, `og_gfx_text_file`), as the
native `text_gfx` test decodes the same cook.

### og-ui's kit frames in the browser

`templates/ui-kit/scripts/browser.ts` (`kit-web`) runs templates/ui-kit's
reference playtests in headless Chrome: `frames-desktop`, `frames-phone`,
`players` and the `style-<pack>[-phone]` playtests of the packs whose
approved states the browser tier judges (fantasy and scifi), every frame
over the pack's plain ground (docs/ui.md §15.4). It bundles each playtest
as `og test` does, reads its setup, size, commands and screenshot times,
and runs reference.ts once per playtest with `--project templates/ui-kit
--references templates/ui-kit/references --size <w>x<h> --script <the
setup at tick 0, then the commands without screenshot, quit and dump>
--shots <file>@<round(60 t)>,…`, so each browser run is the same fresh run
as the native playtest. A style playtest adds `--sheet
kit-style-<pack>[-phone]-states.png,<factor>,<cols>`: each shot is
box-shrunk (2 on desktop, 3 on a phone) and compared with its cell of the
sheet. The summary goes to
`packages/engine/dist/test-reference/ui/reference-ui-<tier>.json`, one
report per playtest beside it. `--only a,b` runs some playtests.

### The accessibility mirror in the browser

`packages/engine/tests/web/a11y.ts` runs the dev export of templates/ui-kit
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

og.motion's weights, which `packages/engine` ships, are not in git. They
live in the private S3 bucket `origin-engine-showcase` on OVHcloud Object
Storage (endpoint `https://s3.bhs.io.cloud.ovh.net`, region `bhs`) under
`motion/<sha256>.safetensors`; `packages/engine/motion.json` pins the file
(`file`, `sha256`, `bytes`, `key`). The credentials are
`OG_SHOWCASE_S3_ACCESS_KEY` and `OG_SHOWCASE_S3_SECRET_KEY` (project
secrets that orbs and the runners receive); `OG_SHOWCASE_BUCKET`,
`OG_SHOWCASE_S3_ENDPOINT` and `OG_SHOWCASE_S3_REGION` override the
bucket, endpoint and region.

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
`references/metal/` and the test says so (`gfx <scene>: no <tier>
reference …; comparing with metal's`); a scene on neither fails. Commit
the new references with the change and describe what each moved frame now
shows (AGENTS.md §Building and testing). Capture on the tier the reference belongs
to; a frame from another tier never replaces a Metal reference.

## When the size gate fails

1. Read the table `size-gate` printed (or `target/evidence/size-gate.json`):
   which wasm grew and by how much.
2. Find the growth: `cargo tree -p og-host-web --target
   wasm32-unknown-unknown -e normal` for new dependencies, and
   `scripts/wasm-size-report.sh target/og-wasm/raw/<name>.wasm` for where
   the bytes are: the module optimised as it ships, then per crate and for
   the largest functions, each with its brotli beside its raw bytes (the
   optimised module in `target/og-wasm/opt/` keeps no names). Rank by
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
