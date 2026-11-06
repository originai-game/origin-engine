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
release and wasm (owner's decision, 2026-11-06). There are no backend
overrides. Four frontend threads (`-Zthreads=4`) are enabled on every
target. Metadata embedding and the build-directory layout use Cargo's
defaults until a controlled experiment demonstrates an improvement.
Consumers use Cargo's artifact messages, not guessed executable paths.

| Native target | Dev/test backend and frontend | Linker | Verification |
|---|---|---|---|
| arm64 macOS | LLVM, `-Zthreads=4` | platform linker | LLVM controls passed on macrunner |
| x86_64 Linux | LLVM, `-Zthreads=4` | mold | Unverified in this lane |
| x86_64-pc-windows-msvc (studio-win) | LLVM, `-Zthreads=4` | rust-lld (lld-link), not link.exe | **Unverified on studio-win** |

The dated nightly includes the MSVC target, but this configuration is not
a successful Windows build report. Neither mold nor wild supports Windows.

wgpu 31 is not on crates.io (2026-10-19, 2026-10-21 and 2026-10-24: `cargo
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

## Windows x64 prerequisites

Use Git Bash on a Windows x64 host with Visual Studio 2022 Build Tools
(Desktop C++, Windows SDK and C++ ATL for x64/x86, component
`Microsoft.VisualStudio.Component.VC.ATL`), the `x86_64-pc-windows-msvc` Rust toolchain,
and the tools pinned above on PATH. Cargo uses rust-lld's lld-link frontend
with the MSVC libraries from Visual Studio; do not substitute the GNU target
for the napi addon. This nightly configuration remains unverified on Windows.
The repository's `.gitattributes` keeps source and generated files LF and
HDR images binary even with `core.autocrlf=true`.

Run `cargo check --workspace --all-targets`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo nextest run --workspace --no-fail-fast`. Windows selects DX12 and
statically links DXC on x64 MSVC, so the installed engine needs no compiler
DLL. The native binaries use the Microsoft Visual C++ x64 runtime
(MSVCP140/VCRUNTIME140); game authors need its redistributable, not Visual
Studio or Rust. The reference harness falls back to Metal; differences are reported,
not silently recorded. `crates/og-tests/references/dx12/` and `vulkan/`
(`OG_GFX_BACKEND=vulkan`) hold a frame only where that backend differs from
Metal's legitimately, each with its reason in the commit that recorded it:
nearest-sampled texels whose edges fall on pixel centres (sprites-array,
tilemap-lit), and NVIDIA's Vulkan anisotropic filter (water-ocean).
Every other retained scene is held to
Metal's. `og check` and the package's typecheck command use the pinned tsgo
through Bun because there is no Windows tsrs package.

`OG_RELEASE=1 sh packages/engine/scripts/pack.sh` builds the Windows native
package and the shared wasm package. Pack create-game in `packages/create`
with `bun scripts/pack-template.ts` then `bun pm pack --ignore-scripts` into
the same tarball directory. Run `bun tests/install/run.ts --from <dir>
--version <version> --host-only` there with Bun and Node 22. This is a
local candidate, not permission to publish. F7's evidence and remaining
registry/runner work live in `tests/evidence/release/windows-x64.md`.

## Linux renderer prerequisites

`.agents/setup` upgrades an older Mesa package rather than accepting any
installed `mesa-vulkan-drivers`. On Debian 12 it uses bookworm-backports
for that package. A warm setup skips apt when its prerequisites are met.

Debian 12's Mesa 22.3.6 grew resident memory each frame in the published
0.4.0 3D intro. Under a 4 GiB limit without swap, the kernel killed the
rendering process. Mesa 25.0.7 passed the same test at the same resolution
with a 429 MiB peak and identical captured pixels. Increasing the sandbox
memory, forcing JS collection, or waiting for each GPU frame does not
replace the driver upgrade. This does not reinstate the retired
vulkan-cpu reference lane. The [controlled comparison](../tests/evidence/P4/perf/reload-and-orb-memory.md)
records the OOM receipt, driver controls, reload timings, and setup checks.

## Lanes

UI packages have an independent lane: `scripts/check.sh kits` (also `ui`).
`kit-tests` runs the kit-local Bun suites and the optional `kit-tests` Rust
targets (`og-cook/kit-documents`, `og-ui/style-packs`, and the four historically
ignored `og-tests-cooked/style-diagnostics` checks). `ui-styles` runs the
pack-owned suites through the UI fixture project's discovery entries;
`ui-kit` runs that template's remaining playtests. `kit-web` separately runs
`templates/ui-kit/scripts/browser.ts` against an existing web build.
Engine quick, functional and full lanes do not run these suites. They keep
the generic UI layout, text, input/focus/layers, motion/drawing, document and
style grammar and common-component tests. Shared test helpers are not suites.
Template references live in `templates/ui-kit/references/<tier>`; style
references live in `templates/styles/references/<pack>/<tier>` and style
playtests in `templates/styles/tests/<pack>`. Neither belongs inside a
shipped `templates/styles/<pack>` asset root.


| Lane | Steps | When | Warm budget, Apple M4 Pro |
|---|---|---|---|
| `scripts/check.sh functional` | fmt clippy test drift bun-functional | After a batch of merges. Native tests and GPU frames; package suites remain in `kits`. Cross-host comparisons require current wasm and run in full when the cached module is stale. | 240 s |
| `scripts/check.sh quick` | fmt clippy test-quick drift bun-quick | Local edit loop. | 120 s |
| `scripts/check.sh kits` (alias `ui`) | kit-tests ui-styles ui-kit | Package-owned behavior, authored styles and template playtests; no wasm build. | 240 s |
| `scripts/check.sh` (full) | fmt clippy wasm-build test wasm-opt size-gate drift bun-test determinism | Wave-end, with `--all` to measure every step rather than reuse passing inputs. | 1200 s |

Budgets are warm-run targets, not test timeouts or permission to skip checks.
Measure on the same hardware and comparable load. A test that takes a lane
past its budget replaces an existing test rather than extending the lane.
Cold toolchain installation and compilation are reported separately. Perf
and browser-reference measurements run at wave-end, with a 1200 s budget
each. Category deletions are allowed; streams need not add tests or frames
by default. A regression defaults to an inline unit test; reserve integration
tests for cross-crate contracts, the CLI and GPU rendering.

November 6 category-cut validation on macrunner (shared load, not a
controlled speed-up): all Rust tests took 80.679 s execution, 85.05 s with
incremental compilation (1608 passed, 28 ignored); the pre-cut suite took
332.373 s execution, 394.70 s total (2787 passed, 26 ignored). Quick took
54.70 s with unchanged-input GPU/drift caching (1015 passed, 7 ignored).
Functional, full, perf and browser lanes have not been measured on this
cut; full/browser need a current optimized wasm build. The kits stream
owns its separate timings. See `tests/evidence/gate-speed/restructure.md`.

The lanes are path-aware. A step that passes records the working tree it
passed on in `<target>/og-gate/green/<step>`: a git tree of the whole
working copy, untracked files included, written through a private index
(the real one is untouched). The next run compares that tree with the
current one (`git diff --name-only`) and skips what none of its inputs
changed:

| Skipped | When nothing changed in | Then |
|---|---|---|
| each GPU package's reference tests (og-render, og-ui, og-tests, og-tests-cooked) in test and test-quick | that package's dependency closure and embedded assets; cooked fixtures additionally depend on og-cook and the template inputs they read | the rest runs; the summary names each cached package. This is input caching, never selection by host or GPU availability |
| wasm-build, wasm-opt, size-gate | `crates/` | the last outputs in `target/og-wasm` are reused (only when every file they list exists; a step that ran upstream in this invocation always runs the next) |
| drift | `crates/og-api` (`api/<module>.ogt`), og-idl and the crates it builds on, `crates/og-cli`, and every generated directory (`og_idl::OUTPUT_DIRS`), so a hand edit to a generated file is still caught | |
| check-conformance (in no lane; `scripts/check.sh check-conformance`) | `packages/engine`, `templates/`, `examples/`, `crates/og-cli/src/check.rs`, `scripts/tsrs-diff.sh`, `scripts/tsrs-corpus/`, `scripts/install-tools.sh`, `scripts/lib.sh` | |
| web-reference (`make web-reference`; in no lane) | og-host-web's and og-tests-web's crates for wasm32 (og-tests-web takes og-tests' scenes, assets and references and og-ui's, whose references hold ui-kit's frames), `packages/engine`, `templates/fx-showcase`, `templates/looks`, `templates/ui-kit`, `templates/archetypes/third-person`, `examples/brasswick`, `examples/meridian-loop` | |

A change to `scripts/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`.cargo/` or `.config/` reruns every one of them. Each skip prints its
reason, the tree it rests on and its inputs; `--all` (`scripts/check.sh
--all`, `scripts/check.sh --all quick`, `scripts/check.sh --all test`) runs
every step and every test, and so does any run with `CI` set in the
environment: CI's lanes stay complete. A failed step clears its record.
bun-test and the non-GPU tests always run; determinism runs when
`tools/wasm-determinism` or a crate it links (og-core, og-ecs, og-physics,
og-anim, og-lanes and what they build on) changed; that closure is read
with `cargo tree --locked` (`og_crate_dirs`, `scripts/lib.sh`), so a
rule never rewrites the tool's lockfile under the `--locked` build that
follows (a stale lock reads as "crates", the superset). bun-test's slow
integration tests (packages/engine's `reload`, `reload-asset` and
`web-export`: og processes for minutes) run only under `--all` and in CI
(`OG_GATE_ALL`); a plain `bun run test` in the package runs them too. A
bun test that signals a child by name uses `process.kill(pid, name)`,
never `Subprocess.kill(name)`: Bun 1.4.2 mapped the name with Linux's
numbers, and 12 is SIGUSR2 there but SIGSYS on macOS (oven-sh/bun#35296,
fixed in 1.4.3; the handoff test of `reload.test.ts` died with status
140 on the Mac, 2026-10-26, the day bun moved to 1.4.4).

The wasm profile depends on the lane: `--all` and CI build the shipping
`wasm` profile (`Cargo.toml`), so the size they record is the module that
ships; the path-aware lane builds `wasm-gate`, the same profile with 16
codegen units, which measured (2026-10-21, cold, Apple M4 Pro) wasm-build
293 → 205 s and wasm-opt 113 → 85 s with modules 5.3–5.9 % larger
(publish 2,947,818 → 3,106,805 B brotli, 2D 2,783,920 → 2,932,177, lanes
3,320,578 → 3,516,299), so a size gate the path-aware lane passes, the
shipping build passes by that margin. With a lanes module per profile
(9d220790 and owed row 71): 3D publish 3,055,713 → 3,218,005, 2D publish
2,894,396 → 3,045,414, 3D lanes 3,095,052 → 3,278,998, 2D lanes 2,928,498
→ 3,106,406 (5.3–6.1 %). `OG_WASM_PROFILE=wasm|wasm-gate`
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
(`bun test --parallel`: 31 s to 12 s). One at a time (`OG_GATE_JOBS=1`) the same
gate takes 129 s.

The gate now admits one GPU test step at a time, while independent CPU
steps can run beside it. Each GPU step retains its internal concurrency:
nextest's four-slot `gpu` group, four Bun workers, or `og test --jobs`.
The engine package uses `bun test --parallel=4` rather than a worker per
CPU. Separate step limits otherwise add together on the same adapter.
This resource exclusion does not add input dependencies to `--only`.

`og-tests::wasm` (og-render must compile for wasm32) takes the full lane's
`wasm-build` og-render artifact when it is newer than og-render's sources,
so it needs no second wasm32 target directory.

Before the lanes (2026-10-18, E1 tree) the single gate took 310 s warm:
`cargo test` serial 111 s, `packages/engine`'s `bun run test` 156 s of which
a release build of `og`, the addon and two wasm variants was most. Four
agents building the same crates into private `target/agent-*` directories
had grown `target/` to 143 GB.

### Test binaries

Each crate has at most one default integration binary, `tests`, declared
at `tests/it/main.rs` with `autotests = false`. The opt-in `kit-tests`
targets belong to the separate kits lane. The November 6 consolidation
reduces the engine's integration binaries from 37 to 23, not from 318:
most files were already modules after the October consolidation.

Single-crate checks belong inline under `#[cfg(test)]`; core, blob,
geometry and localization checks now live beside their implementation.
Integration targets cover cross-crate contracts, GPU rendering and the
CLI. nextest runs each test in its own process, including tests that
change process-wide state. Use nextest, or `cargo test -- --test-threads=1`
when running these tests directly. Filters use `package(..) & test(/^module::/)`.

The test build (`--workspace`) and the gate's dev `og` and addon
(`cargo build -p og-host-node -p og-cli`, `packages/engine/scripts/build.sh`)
share every library build. Cargo resolves features per command, so a
feature that turns on only in one graph gives that crate and every crate
above it a second build. Two had done that, and an og-core edit compiled
22 crates again for `og` after the test build had compiled them:

- syn 3: og-host-web's wasm-bindgen macros turn on `visit` and
  `visit-mut` in the test graph only, which rebuilt serde_derive and with
  it everything from og-core up. og-core-derive asks for the same
  features (`syn3`, unused in its code).
- og-physics: as a workspace member, the test build selects its
  `default`, which adds nothing but changes its hash and og-runtime's.
  og-host-node asks for og-physics' `default`.

Measured as above: after an edit's test build, `og` and the addon
recompiled 22 crates in 8.1–8.3 s after an og-core edit and 6.1 s after an
og-runtime one; now they relink og-host-node alone in 2.6–3.1 s. To check
after a dependency change, compare the two builds' artifact file names
(`--message-format=json`, `compiler-artifact` records with
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
| `wasm-build` | Build `scripts/wasm-crates.txt` variants with the `wasm` profile. Feature variants keep separate caches under `target/wasm-*`; outputs go to `target/og-wasm/raw`. | full |
| `wasm-opt` | Bindgen followed by `scripts/wasm-opt.sh`. `opt.txt` lists shipping variants; `opt-all.txt` lists every optimized module. | full |
| `size-gate` | Brotli-11 bytes of each optimized module in `opt.txt`, checked against `wasm_core_brotli_bytes`. | full |
| `bun-test` | Package tests and typechecks using the dev addon and gate-produced wasm (`OG_GATE=1`), without a second release build. | full |
| `determinism` | `tools/wasm-determinism/run.sh`: equal math digests and world hashes across native, scalar wasm, SIMD wasm and both physics profiles' threaded builds. | full |
| `kit-tests` | Package-local Bun tests and the three opt-in Rust kit targets. | kits, alias ui |
| `ui-styles` | `og test templates/ui-kit --filter tests/style-`, with `OG_UI_STYLES_JOBS` workers, default 3. Any unchecked frame fails. | kits, alias ui |
| `ui-kit` | `og test templates/ui-kit --skip tests/style-`, with `OG_UI_KIT_JOBS` workers, default 4. Any unchecked frame fails. | kits, alias ui |
| `kit-web` | `templates/ui-kit/scripts/browser.ts` against an existing web build. | explicit only |
| `web-reference` | Native and web preparation, then headless Chrome WebGPU groups with `OG_WEB_JOBS` concurrency. Compare retained scenes under unchanged tolerances. | wave-end, or explicit |
| `check-conformance` | `scripts/tsrs-diff.sh`: compare tsrs and pinned tsgo diagnostics over packages, templates, examples and the error corpus. | wave-end, or explicit |
| perf | `scripts/perf.sh`: ignored `perf_*` assertions in og-tests, og-runtime and og-audio, release profile, serially under every GPU slot. | wave-end only |

Rust test steps fall back to `cargo test` when nextest is unavailable,
serializing the GPU packages. Missing adapters fail on every host. Input
caching follows §Lanes. Bun tests have a 30-second per-test timeout through
`OG_BUN_TEST_TIMEOUT`; a real hang fails. `scripts/test-gate-guards.sh`
checks missing and partial wasm outputs for bun-functional, bun-test and
size-gate. Refresh missing wasm explicitly with `scripts/check.sh --only
wasm-opt`; the size gate always reads optimized modules.

`cargo nextest` runs test binaries in parallel, one process per test.
Fixtures several tests read are made once across their processes under a
file lock (og-cook's `tests/support`: the template and the rigged models,
keyed by the test executable and the sources); `.config/nextest.toml` gives
every slot to the tests that assert a wall-clock bound (and runs them last),
two slots to the determinism runs that step two worlds or renderers at once,
four at a time to the GPU-bound packages, and starts og-tests and
og-tests-cooked, the cooks,
the pile and the walk first. `scripts/install-tools.sh` installs it; without
it the steps fall back to `cargo test`.

### CI

There is no GitHub test CI (owner decision, 2026-11-04). The supported
gate is `scripts/check.sh`; do not subscribe to or wait for GitHub checks.
Historical workflow files and earlier timings are not a second test lane.
Every supported machine runs the same selected lane and needs a real GPU
for rendering tests. Neither a CPU adapter nor a missing renderer excuses
an unchecked reference frame.

A second workflow, `.github/workflows/release.yml` (`workflow_dispatch`
only, no secrets, nothing published), is the fallback release pack when
the Mac or the Windows host is unavailable; releases pack on those two
Amp runners (docs/release.md §3, 2026-11-01). The workflow: a `native` matrix (`macos-14` for darwin-arm64,
`windows-latest` for win32-x64) runs `OG_RELEASE=1 build.sh --native` and
`pack.sh --native`, the addon and `og` only; a `web` job in the same
trixie container as the gate, provisioned the same way plus zig 0.17.0 and
cargo-zigbuild 0.23.4, runs `scripts/release.sh pack` (the five wasm
variants, the linux-x64 addon by zigbuild, the engine package with its
knowledge/, create-game); `assemble` writes the manifest over all five
tarballs (`scripts/release.sh manifest`), runs the install test's host
half from them on the runner (no renderer: frames unchecked) and uploads
`release-<version>`. `scripts/release.sh fetch <run-id>` brings that
artifact to the machine that runs `verify` and `publish`.

The process does not change: the functional lane for every merge, `--all`
and web-reference (or `scripts/wave-end.sh`) at a wave's end, by the one
session that merges (AGENTS.md §Build and gate discipline), is what a
merge must pass, and metal is the frame authority. CI is a record
someone asks for, never a gate that runs by itself and never a
precondition of a release or a publish (docs/release.md §3).
Two intermittent failures were known when the workflow last ran (og-cook's
`cooking_twice_gives_identical_bytes`, row 80; the particle test's SIGSEGV,
row 81); a red run is read before it is re-run. Both are closed. The
SIGSEGV was the Vulkan loader of the container, not lavapipe or the
engine: libvulkan1 1.4.309 (Debian trixie; the race is in every loader
from 1.3.290 to 1.4.344, fixed upstream in 1.4.345) frees an instance's
unused drivers in its first `vkEnumeratePhysicalDevices` while another
instance's thread walks that list to name an object, so two renderers
made on two threads of one process can crash it. No test makes two
renderers at once any more; nextest runs each test in its own process,
and `cargo test` (the gate's fallback without nextest), which runs a
binary's tests on threads of one process, runs og-tests,
og-tests-cooked, og-ui and og-render after the other crates with `--test-threads=1`
(`tests/evidence/debt3/particle-sigsegv/NOTES.md`).

## Running the gate locally

```
scripts/install-tools.sh             # once; macOS uses Homebrew for brotli
scripts/check.sh quick               # the quick lane (make quick)
scripts/check.sh                     # the full lane, path-aware; stops at the first FAIL
scripts/check.sh --all               # the full lane with nothing skipped (what CI runs)
scripts/check.sh clippy test         # selected steps only
scripts/check.sh --only bun-test     # a step and the steps it reads (here wasm-build, wasm-opt)
OG_GATE_JOBS=1 scripts/check.sh      # one step at a time, in lane order
scripts/check.sh wasm-build wasm-opt size-gate   # the size gate (make wasm)
scripts/check.sh --list
scripts/check.sh web-reference       # the browser tiers (make web-reference); needs Chrome
make check                           # the same through the Makefile
```

The path-aware state is `<target>/og-gate/` (the green trees) and
`<target>/og-wasm-cache/` (wasm-opt and brotli by content); both are kept
across runs and are only ever bypassed with `--all`, never deleted to force
a step.

One gate per machine, and one per target directory: `scripts/check.sh`
holds `~/.cache/og-gate/og-gate.lock` (`OG_GATE_MACHINE_DIR` moves it;
`OG_GATE_MACHINE_LOCK=0` skips it, as CI does with one gate per runner) and
then `<target>/og-gate.lock` (a release `packages/engine/scripts/build.sh`
holds the target's) for its whole run (`scripts/lock.sh`: mkdir locks with
the holder's pid, taken over when that pid no longer runs a gate); a second
one prints `another gate (pid N: …) is running on <dir>, waiting` and starts
when the first ends. The machine lock is there because the worktrees' gates
are not independent: four at once took a 12-core machine to load 228 and
every one of them past its test timeouts, so they are faster in a queue.

The steps run concurrently, each as soon as the steps whose outputs it
reads have passed or been skipped (`step_needs` in the script): test and
wasm-opt after wasm-build (og-tests::wasm takes wasm-build's og-render
artifact), size-gate and bun-test after wasm-opt; fmt, clippy, drift,
determinism and wasm-build start at once, at most `OG_GATE_JOBS` of them
(2 by default: the native chain beside the wasm chain; 0 lifts the cap).
The gate renices itself by 10 outside CI, so the editor and the agents
beside it keep the machine. Native cargo builds lock
`target/debug` and the wasm32 ones `target/wasm` and
`target/wasm32-unknown-unknown`, so the two chains overlap, and cargo's
lock serialises what shares a directory (clippy, test, drift and the
`cargo build` in bun-test's build.sh); determinism builds into
`target/wasm-determinism`. bun-test does not wait for test. Twice in a row,
and once beside an iOS simulator and another agent's build, every
reference frame, the determinism digests and the size were as one at a
time. `OG_GATE_JOBS=N` caps how many steps run at once; 1 is the serial
order.

Each step writes its output to `<target>/evidence/gate/<step>.log`. A step
prints `--> [n/N] step started` when it starts; when it ends the gate
prints `==> [n/N] step`, the whole log, then `PASS`, `FAIL` or `SKIPPED`
with its duration, so the output never interleaves. The first `FAIL` stops
the steps still running (each is reported as `CANCELLED step (Ns): stopped
when 'x' failed; partial log …`), the gate prints `gate FAILED at 'x'` and
exits non-zero; the gate lock is released only after those steps have
stopped, and Ctrl-C stops them the same way. On success it prints `gate
PASSED (n/N steps, Ns)`. Either way the per-step lines, in lane order,
land in `target/evidence/check-summary.txt`. The
script honours `CARGO_TARGET_DIR` for CI-style isolation, but agents working
in parallel share `target/` (AGENTS.md §Build and gate discipline: private
`target/agent-*` directories compile every dependency once per agent and are
not used). It exports `OG_EVIDENCE_DIR` (default `<target>/evidence`) and
`OG_WASM_OUT` (`<target>/og-wasm`) for test harnesses and `build.sh`.

## Amp orbs

`.agents/setup` installs the Linux build prerequisites, activates
`rust-toolchain.toml`, and reuses `scripts/install-tools.sh` for the pinned
tools. It downloads a nextest binary instead of compiling the runner, fetches
both locked Cargo dependency graphs, installs `tools/bench` and
`tests/install` (the InstaBox SDK of `instabox.ts`) from their Bun
lockfiles, and caches the engine's pinned TypeScript executable. It does not
compile the engine or run the gate during startup. Amp snapshots these tools
and caches; a stale snapshot reruns the idempotent setup using what is already
installed. Build outputs still use the checkout's shared `target/`.

A repository-scoped login-shell hook loads `.agents/environment` so later
agent commands and supervised services find the tools. It also sets `CHROME`
to the orb's preinstalled agent-browser Chrome unless explicitly overridden.
That browser may be newer than the reference-capture version above; setup
does not claim pixel equivalence. No database, credentials or dev server are
needed, so `.agents/resume` does no installation or authentication.

Validate changes with `shellcheck -s bash .agents/{setup,resume,environment}`,
run `time .agents/setup` twice, and check the tools from a new `bash -lc`
shell. Setup changes reach future project orbs after landing on `main`.

## When to run what

The gate costs a round minutes only when it is run too often, and a
machine with several agents cannot carry one per agent (AGENTS.md §Build
and gate discipline: three worktree gates beside a lavapipe container took
the load average past 100 and every step past its timeouts):

- **While writing code**: `cargo check -p <crate>` and that crate's own
  tests, one at a time. An agent in a worktree runs nothing else: no lane,
  no `og test`, no reference capture. `scripts/check.sh quick` (under half
  a minute warm) is the most a worktree runs, and only when no gate runs
  elsewhere.
- **After a batch of merges on main**: `scripts/check.sh functional`, by
  the one session that merges: the tests, the GPU reference frames, the
  generated API and the packages' tests, path-aware. Merge several
  streams, then gate once; never gate per merge. The wasm build and its
  size, determinism and the browser tiers are not in it: they measure,
  and a measurement is pursued at a wave's end, not held against a merge.
- **At a wave's end**: `scripts/wave-end.sh`, by the session that runs
  gates (§The wave-end run): `scripts/check.sh --all`, then `scripts/check.sh
  web-reference`, serially.
- **Once at the end of a round**: `scripts/check.sh --all`, before the
  final commit, with its summary in the report. Never `--all` after every
  edit; never touch files or delete `target/og-gate/` to force a step.
- **After re-recording a reference frame**: rerun only that frame's lane
  with `scripts/check.sh --only <step>`: `--only test` for og-tests' gfx
  references (`OG_CAPTURE=1`, §Re-capturing gfx references; it adds
  wasm-build), `--only bun-test` for a template's playtest frames (it adds
  wasm-build and wasm-opt, skipped when `crates/` did not change),
  `scripts/check.sh web-reference` for the browser tiers. Then `--all` once
  at the end of the round as above.

## Uniform host coverage

Run the same named lane on every host. There is no Windows-only subset,
orb lane, nextest partition/archive, or wasm import/export test routing.
Platform backends still differ (Metal, DX12, Vulkan and browser WebGPU),
but missing adapters and unsupported required GPU capabilities fail tests.
`OG_RENDERER=none` is useful to test explicit simulation-only behavior;
it never turns an unchecked frame assertion into a pass.

Browser groups use separate ports and profile/output directories. Work that
writes the same project's export stays in one group. `OG_WEB_JOBS` controls
concurrency, not test selection. A serial/parallel pixel difference is a
defect, not a reason to increase tolerances.

## The wave-end run

`scripts/wave-end.sh` runs a wave's measurements one after another under
the machine's and the target's gate locks, which it holds across its
stages (the `check.sh` runs inside find them held by their parent), so no
other gate starts in between. Each stage runs even when one before it
failed; the exit status is non-zero when any did. Logs, the summary and
the table go to `target/evidence/wave-end/`.

Every host runs the same stages. The retired container-only test stages
and cross-machine build routing have been removed.

| Stage | What |
|---|---|
| `gate` | `scripts/check.sh --all` |
| `perf` | `scripts/perf.sh`, ignored cost assertions under the exclusive GPU slot lock |
| `web` | `scripts/check.sh web-reference` |
| `art` | each authored look's capture and comparison; no new frame without `--record art[/family]` |
| `conformance` | `scripts/tsrs-diff.sh` |
| `release` | `scripts/release.sh pack` then `verify`: the five packages as release tarballs in `dist/release/<version>/` with `manifest.json`, and the install test (`tests/install/run.ts`) from them on this machine and in the linux-x64 container (docs/release.md §3); publish never runs here |

`--stages gate,perf` selects stages in their defined order; `--dry-run`
prints commands without executing them. Recording takes only `art` or
`art/<family>`. Package-owned kit suites remain in `scripts/check.sh kits`.

## The lanes wasm

The lanes variant of og-host-web (docs/web-export.md §Lanes) is the
publish build compiled for wasm atomics and shared memory, which stable
Rust can only do by rebuilding std: `scripts/wasm-lanes.sh <cargo args>`
runs `cargo build --target wasm32-unknown-unknown -Z
build-std=panic_abort,std` with `RUSTC_BOOTSTRAP=1`, `RUSTFLAGS` (the
caller's, then) adding `+atomics,+bulk-memory` and the shared, imported
memory (at most 2 GiB, against the single-threaded module's 4 GiB) with
wasm-bindgen's TLS exports, and `CARGO_ENCODED_RUSTFLAGS` unset (cargo
prefers it to `RUSTFLAGS`, so one inherited from an outer cargo would drop
the lanes flags), into its own target
directory, `<target>/wasm-mt` (`scripts/wasm-lanes.sh --target-dir`
prints it), so the flags never invalidate the single-threaded build. It
needs the `rust-src` component (`rust-toolchain.toml`). The first build
compiles std and every dependency again: 9 min on the M4 Pro at a load
average around 30; afterwards a change costs about 30 s. rustc warns
`unstable feature specified for -Ctarget-feature: atomics` for every
crate; that is expected.

There is one lanes module per game profile (owed row 71,
docs/web-export.md §Variants): `scripts/wasm-crates.txt`'s `lanes` lines
build `og_host_web-lanes2d.wasm` (`publish,physics2d,profile2d,shaping`) and then
`og_host_web-lanes.wasm` (`publish,physics3d,shaping`), both in
`target/wasm-mt`, where cargo keys each unit by its features: std and
every dependency are shared, and only og-physics, og-runtime and
og-host-web's own units are built once per profile. What the second lanes
build costs, measured on the M4 Pro at a load average of 10 to 18
(2026-10-21): in the path-aware lane's `wasm-gate` profile, 39 s the
first time (after the 2D one, 63 s with std and the dependencies) and,
after an edit, 14 s for an og-physics change and 8.5 s for an og-runtime
one, the same as the 2D build's; in the shipping `wasm` profile (`--all`,
CI), 1 min 46 s the first time (the one both-profile lanes module took
3 min 20 s cold) and 64 to 83 s after an og-physics change. With nothing
changed, each relinks og-host-web, 6 s, as the publish variants do.
wasm-opt optimises the extra module beside the others (the step took 105 s
for all five modules cold in `wasm-gate`) and size-gate compresses it beside
them (25 s), so the wall clock of a full gate grows by about the one
cargo build. The determinism step's two lanes builds (§What runs where) cost 23 s
each after an og-physics change, beside the scalar build's 37 s.

wasm-bindgen writes `snippets/` (the pool's Worker script) beside
`origin-engine-lanes.js`; build.sh copies each lanes module with its
`snippets/` into its profile's directory (`dist/web-publish`,
`dist/web-publish-2d`), and `scripts/pack.sh` refuses a package without
either profile's lanes files. wasm-bindgen-rayon's helper there starts
each pool Worker with `pkg.default(module, memory)`, the positional form
the pinned wasm-bindgen's glue warns about once a Worker (eight warnings a
lanes page, examples/WEB-FINDINGS.md #8), so build.sh rewrites that one
call to the single-object form, `{ module_or_path, memory }`
(`packages/engine/scripts/rayon-worker.sh`, which stops the build when the
helper has neither form), and `tests/web-export.test.ts` checks the
shipped scripts for a positional init. In the gate's mode (`OG_GATE=1`) build.sh
exits 1 when `scripts/wasm-crates.txt` marks og-host-web `lanes` and the
gate left no `og_host_web-lanes.wasm` or no `og_host_web-lanes2d.wasm`.

A Rust panic inside a lane aborts the pool Worker it runs on. The lanes
wasm's engine thread waits for a parallel call on og-lanes' own latch, not
on rayon's, and a recorded panic rings it: the engine thread then panics
with the lane's message, the Worker traps and the page gets a structured
`panic` error naming the tick and the lane's site, instead of a Worker
asleep in `Atomics.wait` (docs/web-export.md §Errors). The browser check is
`bun tests/web/lane-panic.ts` (in `web-reference`): it injects a panic into
a lane (`og_web_lanes_fault`, the Worker's `fault` call) and needs the
crash on the page within 5 s, then the same fault single-threaded.

## `packages/engine/scripts/build.sh` outside the gate

Standalone (`bun run build`, `bun run test` in `packages/engine`), build.sh
does what it does under the gate whenever the gate has built in this target
directory (`<target>/og-wasm/opt-all.txt` and the dev addon and `og` exist):
the dev addon and `og`, the gate's wasm, and one line saying so. It compiles
release and its own wasm only with `OG_RELEASE=1`, with `--linux-x64`, or
when the gate has not built here; a release build holds the gate lock.
Its wasm-opt is `scripts/wasm-opt.sh`, the gate's own invocation, so the
publish wasm that ships is the module the size gate measures. Before
2026-10-19 build.sh left out `--low-memory-unused` and shipped 2,969,335 B
brotli against the gate's 2,943,180 B; an `OG_RELEASE=1` build.sh now
writes a `dist/web-publish/origin-engine_bg.wasm` byte-identical to the
gate's `og_host_web-publish.wasm` (10,807,952 B, 2,943,180 B brotli;
`tests/evidence/gate-speed/`).

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
- A line `variant NAME CRATE FEATURES` builds CRATE's publish variant once
  per line instead of once: `--no-default-features --features FEATURES`
  into `<target>/wasm-NAME`, raw name `<crate>-NAME.wasm`, bindgen'd as
  `origin-engine` and measured by the size gate as its own download.
  og-host-web has two, one per game profile (docs/web-export.md
  §Variants): `publish` (`publish,physics3d,shaping`, the 3D module in
  `dist/web-publish`) and `publish2d` (`publish,physics2d,profile2d,shaping`, the 2D module
  in `dist/web-publish-2d`); `build.sh` under the gate stops when a declared
  variant's wasm is missing.
- A line `lanes NAME CRATE FEATURES` does the same for the lanes variant
  (§The lanes wasm): one lanes module per line, through
  `scripts/wasm-lanes.sh`, raw name `<crate>-NAME.wasm` (NAME starts with
  `lanes`), bound as `origin-engine-lanes` and measured as its own download.
  og-host-web has two, in build order: `lanes2d` (the 2D lanes module,
  `dist/web-publish-2d`) and `lanes` (the 3D one, `dist/web-publish`), with
  their publish variant's features. Without `lanes` lines a crate marked
  `lanes` builds one lanes module with `--features publish`.
- `wasm_artifacts optional|required`: flip to `required` when `og-host-web`
  becomes a cdylib, so the size gate can no longer pass by measuring nothing.
- `drift_check optional|required`: `og api check` counts as available when
  `og --help` lists an `api` subcommand. While it is not, the step prints
  `SKIPPED: og api check not available yet`; with `required` the same
  situation fails the gate. Flip it in the commit that lands `og api check`.

## Switches in the environment

- `OG_RENDERER=none` takes the renderer away natively: og-render's
  `Renderer::new` and `with_surface` answer `NoAdapter` ("OG_RENDERER=none:
  this process runs without a renderer", `og_render::renderer_disabled`),
  as on a machine with no usable adapter. The platform's `engine` and
  `probe` sandboxes set it (docs/design.md §5: they carry no renderer), and
  the switch makes the path testable on a machine with a GPU. It is the
  only way to run without a renderer: the engine draws on a GPU only
  (docs/design.md §2.6), so a native run whose adapter request fails
  without the switch fails, naming the adapter's error, and never goes on
  as tier `none` (`attachRenderer`, packages/engine's node/index.ts; until
  2026-11-02 it went on, and a playtest passed with its frames unchecked
  and its warnings unread). Without a renderer the run is the same run: the simulation, the probe, `dump_*`,
  snapshots and the world hash are identical (rendering never feeds back
  into simulation; the native host keeps the play surface at `--size`); a
  `screenshot` step writes no PNG and logs, not as a warning, `<t>s
  screenshot: no renderer, frame unchecked (<file>)` (og-probe's
  `FRAME_UNCHECKED`); `hello.tier` is `none` and `capture` answers null.
  `og run --json` reports `tier: "none"` and the frames in `unchecked`;
  `og test` runs every playtest check outside `t.frame`, prints one
  `~ <file>: no renderer, frame unchecked` line per screenshot and the
  count in its summary, and fails with a missing-GPU error if any frame
  is unchecked. Simulation-only playtests can still pass without a GPU;
  `og doctor` reports the renderer `none`, not as a problem. Any other value, or none, leaves
  the adapters alone; the web host always has its canvas's. The gfx
  suites (og-tests, og-ui's reference frames, `og-gfx`) need a renderer and
  fail under the switch. `packages/engine/tests/no-renderer.test.ts` and
  og-runtime's `tests/it/no_renderer.rs` hold the contract; frames are verified
  where there is a renderer: the author's machine, the browser-check
  sandbox, an executor with a GPU.

## Browser reference tiers

The browser tier is WebGPU (the WebGL2 tier and its references were
removed on 2026-11-01, design §8). og-tests' gfx references cover `metal`
(the `vulkan-cpu` copies went with the CPU sandbox requirement on
2026-10-22, §The wave-end run); og-ui's 83 reference frames are `metal`'s
(the browser tier runs 76 of them) and the browser tier is judged against
`metal` (§og-ui's kit frames in the
browser);
fx-showcase's browser frames
come from `packages/engine/tests/web/reference.ts` (`make web-reference`): the dev
export of `templates/fx-showcase` on a 1280×720 canvas at DPR 1 in headless
Chrome for Testing, driven one tick and one render per `frame` call as
`og run --commands` drives the native host, each station captured at its
playtest screenshot's tick and compared with
`templates/fx-showcase/references/webgpu/` (committed; `metal`
when it has none) at og-tests' default tolerance (32/255 per channel,
0.5 % outliers, RMSE 0.01). `--record` writes the tier's references. Before
the stations it sends `call("probe", "commands", {…})` with an object
through the engine Worker and requires the probe's refusal ("probe.commands:
script must be a string, not object") from the same Worker that then runs
them. It takes about 2 s after the export (Apple M4 Pro, Chrome 150)
and passes, but it is not in the gate: CI's machine has no
Chrome for Testing (a missing Chrome is an error, never a skip).

The same runner takes another project's playtest frames: `--project
templates/archetypes/topdown --script "<the frames playtest's commands
without screenshot and quit>" --shots start@30,crit@131,dash@168,east@216`
compares (or with `--record` writes) `<project>/references/<tier>/`. What
each template commits under `references/`: `metal` alone for idle, rhythm,
roguelike, tower-defense, match3, runner, topdown, third-person and
fx-showcase (every `vulkan-cpu` copy went on 2026-10-22, and the WebGL2
beacon, water and Meridian Loop frames with the tier on 2026-11-01). The UI shell round (2026-10-21) deleted topdown's `webgpu`
and `webgl2` frames and 47 of fx-showcase's 48: each sits within the
tolerance of `metal` on its tier (topdown within RMSE 0.00009, three of
the WebGL2 four RMSE 0; fx-showcase's in
`tests/evidence/P4/ui-shell/fx-showcase-browser-vs-metal.tsv`), so they
recorded no difference the tier makes; the 36 that had gone stale when
the UI text round kerned the station HUD on `metal` failed against their
own copies, never against `metal`.

A tier's references are looked up frame by frame: a frame with no file
under `references/<tier>/` (or a tier with no directory at all) is judged
against `references/metal/`, and a frame on neither fails the run
("no reference at …; record one with --record"), never passes. A tier
gets its own reference only for a difference the tier makes by design,
never to absorb a defect.

`web-reference` also runs fx-showcase's looks station:
reference.ts with `--looks` (output in `dist/test-reference/looks`) runs
`tests/looks-script.ts`'s commands (the looks playtest's, without its
screenshots and `quit`; the playtest and the runner import the same
module, so they cannot drift) in one run and takes one shot per preset
of the catalogue at its screenshot's tick (37 shots, the last at tick
1164), each compared with `references/webgpu/looks-<slug>.png`, else
`metal`'s. The plain and look-taken-off frames are the playtest's own
check and are not taken. WebGPU has no looks references of its own
(every frame within RMSE 0.0098 of `metal`, 2026-10-21).

`web-reference` also runs Ember Vale (`templates/archetypes/third-person`):
reference.ts with `--project
templates/archetypes/third-person --out dist/test-reference/third-person`,
`tests/frames.og.ts`'s commands without its screenshots and `quit`
(`1.0:run;1.1:warp:0;1.3:key_down:KeyF;3.5:key_up:KeyF;3.6:light:1;3.7:light:2;3.8:light:3`)
and `--shots title@54,beacon@204,dawn@432` (each at its screenshot's tick,
60 t). A change to that playtest changes the step's command in
`scripts/check.sh` with it. WebGPU has no references of its own: against
`metal` (2026-10-19) title is at RMSE 0.00131, beacon 0.00475 with
0.060 % outliers, dawn 0.00067.

`web-reference` also runs two of the example games (release
row F4): Brasswick's nine frames (`examples/brasswick`: title, levels,
start, door, stomp, gate, pause, settings-es, results; the last at tick
1092) and Meridian Loop's ten (`examples/meridian-loop`: title, jump,
slide, countdown, race, ghost, pause, results, board, settings-de; the
last at tick 7272, two minutes of race). reference.ts takes them with
`--project examples/<game> --playtest tests/frames.og.ts --out
dist/test-reference/<game>`: the script and the shots are read from the
playtest itself (its commands without screenshots, `quit` and `dump_*`,
each shot at its screenshot's tick, 60 t), so a retimed playtest moves its
browser frames with it and no command in `scripts/check.sh` has to follow
it; `--print` shows what a run would play. Every frame is judged against
`metal` until the tier needs one of its own for a difference it makes.

`bun tests/web/archetype-hashes.ts` compares the three
archetypes' world hashes across hosts: each replay playtest's script runs
in Chrome and in `og run` at the page's play surface (1280×577 in the
headless window), and the two `dump_world` hashes at the same tick must be
equal. They are on WebGPU (2026-10-19,
`tests/evidence/E7/archetype-hashes-*.md`). They used to differ because the
native `og run` called `GameDesc.start` before attaching its renderer, so
`og.render.capabilities` answered 0×0 there and runner and match3 laid
their view out for 16:9 instead of the surface; `og run` now creates the
engine with `deferStart` and runs the start once the renderer is attached, as the web Worker always did.

WebGPU is judged against `metal` (2026-10-21, the UI shell branch
merged with main 376f3cd):

| Station | outliers | RMSE |
|---|---|---|
| post | 0.000 % | 0.00142 |
| lens | 0.010 % | 0.00228 |
| fx | 0.000 % | 0.00204 |
| 2d | 0.000 % | 0.00251 |
| stylised | 0.027 % | 0.00389 |

The November 6 cut retains `post` and `stylised` in the showcase smoke.
The browser reads its command timeline and screenshots from
`tests/frames.og.ts`; it no longer launches each E5 station separately.
Rendering features retain representatives in the engine's GPU suite.

The runner found a defect fixed in og-render before the tier was recorded
(`crates/og-render/README.md` §The overlay): the overlay blended encoded
sRGB values on the WebGPU canvas's `Bgra8Unorm` (the HUD panel was
87,89,99 against 123,128,135; RMSE up to 0.024).

### og-tests' gfx scenes in the browser

`packages/engine/tests/web/reference-gfx.ts` renders og-tests' own gfx
scenes (`crates/og-tests/src/scenes.rs`) in headless Chrome:
`crates/og-tests-web` compiles them to a wasm-bindgen module (their glTF and
font assets embedded with `include_bytes!` on wasm32), opens each on an
OffscreenCanvas at 1280×720 on WebGPU, draws its frames as the native
harness does (`og_tests::gfx::render_built`) and reads the target back.
Each frame is compared with `crates/og-tests/references/webgpu/` when
that frame is committed, else with `metal`, under the scene's own tolerance
(`crates/og-tests/src/compare.rs`), and every frame's figures against
`metal` are kept in the report. `--scene a,b --time n`
times any scene it names, the shadows-aa cost scenes too (`COSTS`:
`point-shadows-cube`, `shadows-local-8-cube`, `shadows-local-8-paraboloid`,
`shadows-soft-off`, `aa-off`, `taa-motion`, built but not in `SCENES`). `--record` writes the tier's references;
`--time <n>` writes per-frame times of the perf scenes to
`crates/og-tests/perf/<tier>-chrome-<adapter>/` beside the native ones.
The perf-only city scenes (`city-street`, `city-aerial`) are timed only when
`--scene` names them. `--cpu-culling` makes WebGPU cull on the CPU
(`Culling::Cpu`). `--perf-out <dir>` writes the timing
JSON to `<dir>/<tier>[-cpu-culling]/` instead, with the last frame's culling
counts (`tests/evidence/P3/culling/`). The native counterpart is
`og-gfx <scene> --frames <n>`, with `OG_GFX_CULLING=cpu|gpu|auto` for the
CPU path, the GPU path at any size, or the default (the GPU path from
16,384 instances). It
builds in `target/wasm-gfx` so the gate's og-render artifacts stay put.

On 2026-10-19 (Chrome for Testing 150, ANGLE Metal on the Apple M4 Pro,
`tests/evidence/P2/tooling/web-gfx-*.json`) all 47 applicable scenes pass on
WebGPU, which matches `metal` throughout and has no references of its
own. A WebGPU reference is recorded only for a difference the tier makes
by design, never to absorb a defect. `shadows-crypt` (wave 13, web
findings #4) has none, and the lamp's glass casts no shadow.
`shadow-flags` (owed row 87) has none either: per-instance cast and
receive flags under the sun's cascades alone. A run takes 30 to 50 s
after the module build.

The same module carries og-ui's four reference scenes as `og-ui/<name>`
(`sample-screen`, `text-glow`, `glass`, `widget-set`). They are built by
`crates/og-ui/tests/support/scenes.rs`, the file og-ui's native `gfx` and
`kit_gfx` tests build them with, included into og-tests-web by `#[path]`,
and judged against `crates/og-ui/references/<tier>/<name>.png`, else
`metal`. The browser tier has no references of its own for them: they are
judged against `metal`, which they match exactly (RMSE 0) on WebGPU
(2026-10-19), except `widget-set` at RMSE 0.00046, which is also what the
native metal run gives against that reference. `text-fields-2` rides with
them, and since release-f2 so do the four cooked-font text frames
(`og-ui/text-bidi`, `text-kerning`, `text-edit`, `text-mixed-runs`,
`crates/og-tests-cooked/tests/support/text_scenes.rs`): the module cannot cook, so
the runner first cooks `crates/og-ui/tests/text-fonts` (the fonts' import
settings) with templates/ui-kit's font files through `og cook`
(`tests/web/text-fonts.ts`, the `og` of `packages/engine/native/` or
`$OG_BIN`), serves the blobs beside the module and hands each page the
manifest and files (`og_gfx_text_manifest`, `og_gfx_text_file`), as og-ui's
native `text_gfx` (og-tests-cooked's) decodes the same cook from its bytes. They have not run
in a browser on record yet.

### og-ui's kit frames in the browser

`templates/ui-kit/scripts/browser.ts` runs templates/ui-kit's
reference playtests in headless Chrome: `frames-desktop`, `frames-phone`,
`players`, `locales` and the `style-<pack>[-phone]` playtests of fantasy's
and scifi's approved states, every frame over the pack's plain ground (the
kit's `playtest`, `tests/lib.ts`; docs/ui.md §15.4). It bundles each
playtest (`bundleGame`, as `og test` does), reads its setup, size, commands
and screenshot times, and runs reference.ts once per playtest with
`--project templates/ui-kit --references templates/ui-kit/references --size
<w>x<h> --script <the setup at tick 0, then the commands without
screenshot, quit and dump> --shots <file>@<round(60 t)>,…`, so each
browser run is the same fresh run as the native playtest. A style playtest
adds `--sheet kit-style-<pack>[-phone]-states.png,<factor>,<cols>`: each
shot is box-shrunk (2 on desktop, 3 on a phone, as `tests/styles.ts` does)
and compared with its cell of the sheet. Frames are judged against
`crates/og-ui/references/webgpu/`, else `metal`, at og-tests'
default tolerance (WebGPU has no references of its own, so every frame
and sheet is judged against `metal`), and the summary goes to
`packages/engine/dist/test-reference/ui/reference-ui-<tier>.json`, one
report per playtest beside it. `--only a,b` runs some playtests,
`--record` writes the tier's frames and sheets.

The browser tier is judged against `metal`'s 72 frames: per-tier
copies, recorded 2026-10-19 and reproduced exactly (RMSE 0),
were dropped as duplicates of `metal`, since no frame differs by the
tier's design. Against `metal` (Chrome for Testing 150, Apple M4 Pro,
2026-10-19) every frame passes.
Eleven frames have outliers there, all at most 0.038 % (RMSE ≤ 0.0089,
`kit-desktop-settings`): the edges of the large MSDF titles ("Settings",
"Paused", "Shop", "Levels", "Toasts", the puzzle score), eleven edge
pixels on `idle` and the gallery's scrollbar thumb, whose coverage
differs by a fraction of a pixel. lavapipe and WebGPU
agree there (no outliers between them, RMSE ≤ 0.0019); the
difference is metal's. Every other frame is within RMSE 0.0025 of `metal`,
except the paper pack's phone sheet at 0.0040.
A run takes 100 to 140 s (about 7 to 9 minutes while another heavy job
shares the machine), so the full set runs in `web-reference` rather than
a subset.

### The accessibility mirror in the browser

`packages/engine/tests/web/a11y.ts` (in `web-reference`) runs
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
module (the 3D or the 2D publish variant by the game's profile, or that
profile's lanes variant on an isolated page; docs/web-export.md §Variants), so each
is held to it and the size gate's figure is the largest of them
(`max_brotli_bytes` in `size-gate.json`, the `max` line of its table),
never the sum of every wasm the gate builds. The table prints each
variant's brotli size with its download and its share of the budget.
Change the number in its own commit, with the measured value from
`target/evidence/size-gate.json` and the reason in the message; a raise above
a design §7 target also changes design §7 in the same commit. The budget is
16 MB brotli per download (raised from 4 MB on 2026-11-01, the owner's
quality-first decision, design §7 and §8): one ceiling for every variant,
with no second tier. It catches accidental growth (a dependency pulled in by
mistake, monomorphisation, doc strings in a publish build) and is never a
reason to cut a feature down.

## Showcase

Every frame a stream reads, every film it records and every number it
measures (gate timings, size-gate figures, perf runs, replay results and
hashes) goes into the showcase archive, from which a web showcase of the
engine's features is built later (the owner, 2026-11-02). The files are not
in git, which is already near a gigabyte. They live in the private S3 bucket
`origin-engine-showcase` on OVHcloud Object Storage (the originai Public
Cloud project, region BHS, endpoint `https://s3.bhs.io.cloud.ovh.net`; the
owner's choice of 2026-11-02 over Cloudflare R2), under
`sha256/<2 hex>/<sha256>.<ext>`, so a file is stored once however often it
is added. The same bucket holds og.motion's weights, which `packages/engine`
ships, under `motion/<sha256>.safetensors`: `packages/engine/motion.json`
pins the file. Development builds (including `OG_GATE=1 --native`) and
`pack.sh` run `bun packages/engine/scripts/motion-model.ts ensure`: reuse a
verified local copy, copy and verify `OG_MOTION_MODEL`, or fetch the pin with
the same S3 user. A missing or stale model whose fetch fails stops the build,
rather than letting motion scenes silently draw og.anim poses. Native-only
release jobs package binaries, not these assets, and do not fetch them.
Before running scenes with a bare `cargo build` binary, run `ensure` explicitly;
Cargo does not fetch runtime assets. A fresh checkout/CI needs the S3 credentials
or a pinned local file. `bun packages/engine/scripts/motion-model.ts verify`
always reads the remote key and checks its size and SHA, even if a local copy
exists; run it when changing the pin to catch an absent or corrupt upload.
`fetch <out>` and `upload <file>` remain available. Git holds only
`showcase/manifest.jsonl`, one JSON entry per line:

| Field | What it is |
|---|---|
| `id` | `<feature>/<12 hex>`, from the file's hash, the feature and the title |
| `feature` | the roadmap row (`U6c`, `G2-3`) or stream (`ui-art`, `gate`) it shows |
| `title` | what the artifact shows, in a few words |
| `kind` | `frame`, `film`, `before-after`, `benchmark` or `concept` |
| `commit`, `date`, `thread` | the commit it shows, the day, the Amp thread that made it; a `null` thread means unknown: a pre-trailer backfill entry whose commit no single stream branch alone reaches (`add` never writes one now) |
| `key`, `sha256`, `bytes`, `type` | the object in the bucket, its hash, size and media type |
| `width`, `height`, `duration` | an image's or film's pixels, a film's seconds |
| `metrics`, `machine` | a benchmark's `{ name, value, unit }` list (a value may be a string, such as a hash) and where it ran |
| `pair`, `side` | a before-after pair's shared name and `before` or `after` |
| `source` | where the file came from (a path, an attachment URL) |
| `excluded`, `reason` | `true` and why, for withdrawn work: never shown, its bytes refused by `add` |

```
bun scripts/showcase.ts add <file> --feature U6c --title "phone HUD at 200 % text"
bun scripts/showcase.ts add before.png --feature G2-3 --title "stack before sleep" --pair "stack" --side before
bun scripts/showcase.ts add --feature gate --title "functional lane" --machine macrunner \
  --metric wall=169 s --metric tests=1500 count        # a benchmark: its metrics are stored as JSON
bun scripts/showcase.ts list [--feature U6c] [--kind film] [--json]
bun scripts/showcase.ts exclude <id|file|sha256> --reason "…"   # withdraw it (below)
bun scripts/showcase.ts verify                          # every live entry's key is in the bucket at its size
bun scripts/showcase.ts index                           # showcase/index.html from the manifest
bun scripts/showcase.ts serve                           # the index on localhost:8780, files proxied from the bucket
```

`add` defaults `--commit` to HEAD, `--date` to today (UTC) and `--thread` to
`$AMP_THREAD_ID`, else `$AMP_CURRENT_THREAD_ID`; with neither set (a
runner's shell sets no thread variable) it refuses the entry rather than
record no thread, so pass `--thread T-…` (an ID or thread URL) there. The
570 entries of the first backfill that hold `thread: null` stay unknown on
purpose: their files' commits were made without an `Amp-Thread-ID` trailer
and are reachable from several stream branches of `docs/roadmap.md` §4 (they
were on main when those branches were cut), so no thread follows from the
evidence, and a guess by author or date is worse than none. It uploads only bytes the bucket does not hold and never
repeats an entry of the same file, feature and title. It reaches the bucket
over S3 as the user `origin-engine-showcase-s3`, whose policy allows put,
get and list in this bucket alone (no delete, no other bucket), with
`OG_SHOWCASE_S3_ACCESS_KEY` and `OG_SHOWCASE_S3_SECRET_KEY`: secrets of the
origingame/origin-engine Amp project, which orbs and the runners (macrunner,
studio-win) receive (`OG_SHOWCASE_BUCKET`, `OG_SHOWCASE_S3_ENDPOINT` and
`OG_SHOWCASE_S3_REGION` override the defaults); widths and heights come from
the PNG header, else `ffprobe`. Commit the manifest lines with the change
they show; `showcase/index.html` is regenerated with them. The first backfill
(2026-11-02) took every attachment and artifact of that day's streams, the
committed `references/` frames and `tests/evidence/` images where they are
(by key; nothing moved), the perf baselines and golden replays, and
macrunner's reported gate, size and replay figures.

**Generated assets** (owner, 2026-11-04). Everything generated for the engine
and its scenes (Meshy motions and models, training libraries and checkpoints
worth keeping) is kept for later model runs in a second private bucket,
`origin-generated-assets` (same project and endpoint, versioned, encrypted).
The same user may put, get and list there, never delete; a put must send
`x-amz-server-side-encryption: AES256`. Layout (its `README.md` holds it):
`meshy/motion/<operation-id>/clip.<fbx|bvh>` (the original bytes and format) with `task.json` (prompt, Meshy task
ID, gateway media ID, bytes, sha256) under the original operation ID,
`meshy/model/<operation-id>/…`, `training/<run>/…`, `scenes/<scene>/…`.

Withdrawn work never enters the archive (the owner, 2026-11-02). `add`, and
so any backfill built on it, refuses a file or `--source` under a
`rejected/` directory, where a thread keeps a concept it was told not to
publish (ui-art's first midcore draft copied Supercell's characters). What
is withdrawn after the fact is marked with `exclude`: by id, or by content
(a file or its sha256, every entry of those bytes), as `excluded: true` with
a `reason`; content not yet in the manifest gets an excluded entry with no
`key` and nothing uploaded, so the same bytes are refused when they arrive
again by another path (a message attachment has no `rejected/` in its URL).
`index` and `serve`, and the public site after them, leave excluded entries
out.

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
`references/metal/` and the test prints so (`gfx <scene>: no <tier>
reference …; comparing with metal's`), as every other reference harness
does (§Browser reference tiers); a scene on neither fails. A tier keeps a
reference of its own only for a difference it makes by design (more than
the tolerance against `metal`); since 2026-10-22 only the browser tiers
do (the `vulkan-cpu` copies went with the CPU sandbox requirement).

Commit the new references with the change, and describe in the commit what
each moved frame now shows (AGENTS.md §Validation: a rendering change carries
a frame that was read and described). Capture on the backend tier the
reference belongs to; a frame from another tier never replaces a Metal reference.

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
   configuration, avoid generic monomorphization blow-ups. A stable sort
   in engine code is `og_core::sort::{sort, sort_by, sort_by_key}`, and a
   `BTreeMap` or `BTreeSet` built from an iterator is
   `og_core::sort::{map, set}`: the standard library's `sort_by` and
   `collect` into a B-tree monomorphise driftsort per element type and
   closure (tests/evidence/r4/size-2/NOTES.md measured 0.9 MB of them). A
   publish build carries no reflected or record field docs (og-core's and
   og-api's `publish`), so a new doc string costs a publish module nothing.
4. When the growth is intended (a feature that needs the code), update the
   budget as above rather than cutting the feature: quality comes first
   (design §7). Measured at 49ed935, physics adds 0.69 MB brotli (3D),
   0.54 MB (2D) or 1.05 MB (both) to og-host-web's 1.10 MB
   (`tests/evidence/M6/physics-wasm-size.json`).
