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
| bun | 1.4.4 | `BUN_VERSION` in `scripts/install-tools.sh`; `tests/install/Dockerfile` and `tests/install/instabox.ts` install the same for the install test |
| TypeScript, tsgo (`check-conformance`'s comparison and `og check`'s fallback where tsrs has no package; no project or package depends on it) | 7.0.2 | `TSGO_VERSION` in `crates/og-cli/src/check.rs` (`scripts/tsrs-diff.sh` and `.agents/setup` read it) |
| tsrs (the type checker: `og check`, the check before `og run`/`test`/`build`/`dev`, `packages/engine`'s `typecheck`, og-idl's declaration test) | 0.18.0-ts7.1.0-dev.20260929 (TypeScript 7.1.0-dev.20260930.4) | `TSRS_VERSION` and the five SRIs in `scripts/install-tools.sh`; `packages/engine/package.json` `optionalDependencies` copy it (`check-conformance` holds them equal). It moves only at a wave's end, with a tsrs release, all of these in one commit, after `scripts/tsrs-diff.sh` passes on the new binary |
| cargo-nextest | 0.9.148 here | not pinned |
| ShellCheck (CI lints `scripts/*.sh` with it; Ubuntu's 0.9 reports what 0.11 does not) | 0.11.0 | `SHELLCHECK_VERSION` in `scripts/install-tools.sh` (Homebrew's on macOS) |
| node (`determinism`'s wasm32 runs) | 24 in CI | `.github/actions/provision` |
| sccache (the rustc wrapper, `.cargo/config.toml`; not used in CI) | 0.14.0 | `SCCACHE_VERSION` in `scripts/install-tools.sh` |
| cargo-zigbuild, zig (`build.sh --linux-x64`) | 0.23.4, 0.17.0 here | not pinned; outside the gate |
| Chrome for Testing (`make web-reference`, browser tests) | 150 | `CHROME_PATH`; outside the gate |
| Mesa lavapipe (CPU rendering in orbs) | minimum 25.0.7 | `.agents/setup`; Debian bookworm-backports supplies Mesa and apt resolves its dependencies |

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

## Windows x64 native lane

Use Git Bash on a Windows x64 host with Visual Studio 2022 Build Tools
(Desktop C++, Windows SDK and C++ ATL for x64/x86, component
`Microsoft.VisualStudio.Component.VC.ATL`), the `x86_64-pc-windows-msvc` Rust toolchain,
and the tools pinned above on PATH. Cargo finds the MSVC linker through
Visual Studio; do not substitute the GNU target for the napi addon.
The repository's `.gitattributes` keeps source and generated files LF and
HDR images binary even with `core.autocrlf=true`.

Run `cargo check --workspace --all-targets`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo nextest run --workspace --no-fail-fast`. Windows selects DX12 and
statically links DXC on x64 MSVC, so the installed engine needs no compiler
DLL. The native binaries use the Microsoft Visual C++ x64 runtime
(MSVCP140/VCRUNTIME140); game authors need its redistributable, not Visual
Studio or Rust. The reference harness falls back to Metal; differences are reported,
not silently recorded. `og check` and the package's typecheck command use
the pinned tsgo through Bun because there is no Windows tsrs package.

`OG_RELEASE=1 sh packages/engine/scripts/pack.sh` builds the Windows native
package and the shared wasm package. Pack create-game in `packages/create`
with `bun scripts/pack-template.ts` then `bun pm pack --ignore-scripts` into
the same tarball directory. Run `bun tests/install/run.ts --from <dir>
--version <version> --host-only` there with Bun and Node 22. This is a
local candidate, not permission to publish. F7's evidence and remaining
registry/runner work live in `tests/evidence/release/windows-x64.md`.

## Orb CPU renderer

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

| Lane | Steps | When | Warm, Apple M4 Pro, another agent on the machine |
|---|---|---|---|
| `scripts/check.sh functional` | fmt clippy test drift bun-functional ui-styles | after each batch of merges on main, and CI job `functional` by hand: what a merge must pass (every test and the GPU reference frames, the generated API, the packages' tests on the native host, every style pack's reference sheets); the web module is the last full lane's, and when a crate changed since, the tests that compare the two hosts' hashes skip (`OG_WASM_STALE`) and run in the full lane; no wasm build, size, determinism or browser tiers | 90–160 s |
| `scripts/check.sh windows` | wasm-import test-windows web-reference | on studio-win beside the Mac's functional lane at the same commit, with the Mac's wasm (§Two machines) | measured in §Two machines |
| `scripts/check.sh quick` | fmt clippy test-quick drift bun-quick | the loop while working; CI job `quick` by hand | 19–26 s (test-quick 16–20 s for 225 tests, 256 with og-ui's and og-render's GPU tests) |
| `scripts/check.sh` (full) | fmt clippy wasm-build test wasm-opt size-gate drift bun-test determinism | path-aware once per batch of merges on main, by the one session that runs gates (§When to run what); `--all` once at the end of a round, before its final commit and report; CI job `gate` by hand | `--all` 84–101 s, steps concurrent (test 54–69 for 437 tests beside wasm-opt 54–69, then size-gate 22–25 beside bun-test 14–18), 121–135 s at a load average of 24–32; with nothing changed 30 s |

Both lanes are path-aware. A step that passes records the working tree it
passed on in `<target>/og-gate/green/<step>`: a git tree of the whole
working copy, untracked files included, written through a private index
(the real one is untouched). The next run compares that tree with the
current one (`git diff --name-only`) and skips what none of its inputs
changed:

| Skipped | When nothing changed in | Then |
|---|---|---|
| each GPU package's reference tests, on its own (og-render, og-ui, og-tests: nextest `-E 'not package(<pkg>)'` for each skipped one in test, test-quick and test-windows; left out of the serial GPU run under cargo test) | that package and every workspace crate it builds on (`cargo tree`, with dev-dependencies), what they embed from outside `crates/` (`third_party/ltc`; `templates/looks` and `templates/styles` when og-cook is in the closure), and for og-ui the templates its tests read (`templates/ui-kit`, `templates/styles`, `templates/game/assets/fonts`, the five archetype styles). One key per package since 2026-11-01: an og-ui, og-cook or og-api change no longer reruns og-render's GPU tests | the rest of the test step runs; the step's summary line names the packages whose GPU tests were skipped |
| wasm-build, wasm-opt, size-gate | `crates/` | the last outputs in `target/og-wasm` are reused (only when every file they list exists; a step that ran upstream in this invocation always runs the next) |
| drift | `crates/og-api` (`api.ogt`), og-idl and the crates it builds on, `crates/og-cli`, and every generated directory (`og_idl::OUTPUT_DIRS`), so a hand edit to a generated file is still caught | |
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

### Test binaries

Each crate's integration tests are one binary, `tests` (`tests/main.rs`
declares every file under `tests/` as a module; `autotests = false` and one
`[[test]]` in its `Cargo.toml`). An edit used to relink one ~80 MB
executable per test file, 199 of them after an og-core edit, and those
links were 78–85 % of the CPU of every incremental test build. Measured
2026-10-26 in an orb (16 cores, Linux, rust-lld, warm), `cargo test
--workspace --no-run` after appending a function to a crate's `lib.rs`:

| Edit | Before (231 executables) | After (58) |
|---|---|---|
| og-core | 23.0 s | 7.9 s |
| og-render | 20.1 s | 7.5 s |
| og-runtime | 12.5 s | 5.4–5.9 s |

nextest still runs every test in a process of its own, so the merge changes
no test's isolation; `cargo nextest list` gives the same 1068 integration
tests before and after. A test is now named `<module>::<test>` in the
`tests` binary: `cargo test -p og-ui --test tests gfx::` runs what `--test
gfx` ran, and nextest filters use `package(..) & test(/^module::/)`. A new
test file is a `mod` line in its crate's `tests/main.rs`.

A file stays its own binary when its tests change process-wide state that
another module's tests would see under plain `cargo test` (threads of one
process), or when they assert a wall-clock bound: og-physics' `lanes` and
`pile` (the lanes switch, grains and an injected fault), og-runtime's
`extract_cost`, og-audio's `audio` and og-tests' `render`. Such a file is a
`[[test]]` of its own in the crate's `Cargo.toml`, named in the header of
its `tests/main.rs`. Crates with one test file keep cargo's default.

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

| Step | Command | Lane | CI `quick` (manual) | CI `functional` (manual) | CI `gate` (manual) | CI `perf` (manual) |
|---|---|---|---|---|---|---|
| `fmt` | `cargo fmt --all --check` | every | yes | yes | yes | |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | every | yes | yes | yes | |
| `test-quick` | `cargo nextest run --workspace --exclude og-tests` (`cargo test` without nextest, og-ui and og-render after the rest with `--test-threads=1`); og-ui's and og-render's GPU tests left out when their inputs did not change (§Lanes) | quick | yes, on lavapipe | | | |
| `test` | `cargo nextest run --workspace && cargo test --workspace --doc` (`cargo test --workspace` without nextest, og-tests, og-ui and og-render after the rest with `--test-threads=1`); gfx tests in `og-tests`, left out with og-ui's and og-render's when their inputs did not change (§Lanes). Runs after `wasm-build`, whose og-render artifact `og-tests::wasm` takes when it is newer than og-render's sources (else that test runs `cargo check --target wasm32-unknown-unknown -p og-render` itself) | full, functional | | yes, on lavapipe | yes, on lavapipe | |
| `wasm-build` | `cargo build --profile wasm --target wasm32-unknown-unknown -p <crate> --lib` (`Cargo.toml`'s `[profile.wasm]`: release at opt-level s, the per-tick and per-frame crates at 3 and the load-time crates at z; `tests/evidence/r4/size-2/NOTES.md`) for `scripts/wasm-crates.txt`, plus the `publish` feature variant of crates marked `publish` in `target/wasm-<variant>/` (cargo 1.99 keeps one fingerprint per unit whatever its features, so two feature sets in one target dir recompiled og-probe, og-runtime and og-host-web on every run: 16.7 s and 12.2 s with nothing changed, 0.4 and 0.5 s apart; `tests/evidence/P2/tooling/wasm-build-relink.md`) and the lanes variants of crates marked `publish lanes` (one per `lanes` line: og-host-web's `lanes2d` then `lanes`, the 2D and the 3D publish features with wasm atomics, `scripts/wasm-lanes.sh` in `target/wasm-mt/`, which the two share; §The lanes wasm); copies to `target/og-wasm/raw/` | full | | | yes | |
| `wasm-opt` | `wasm-bindgen --target web --out-name origin-engine` (`origin-engine-lanes` for a lanes variant, `-lanes*`; when the module uses it) then `scripts/wasm-opt.sh`, the one wasm-opt invocation (`-Oz --enable-bulk-memory --enable-nontrapping-float-to-int --low-memory-unused`; rustc links wasm32 stack-first, so the low 1 KiB is only the bottom of an overflowing stack), which `packages/engine/scripts/build.sh` also ships with (`--threads` adds `--enable-threads` for the lanes variant); `opt.txt` lists the shipping (publish and lanes, one of each per game profile) variants, `opt-all.txt` every one | full | | | yes | |
| `size-gate` | `scripts/size-gate.sh`: brotli `-q 11` of each file in `opt.txt` ≤ `wasm_core_brotli_bytes` (a page downloads one variant; the table also prints the sum and the largest) | full | | | yes | |
| `drift` | `og api check` (generated `.d.ts`, wiki, JSON schema, Rust glue vs `crates/og-api/api.ogt`), the dev `og` the test step built | every | yes | yes | yes | |
| `bun-quick` | `bun run test` in `packages/knowledge` and `packages/create` (no native build) | quick | yes | | | |
| `bun-functional` | `bun run test` in each `packages/*` with the last full lane's wasm (`OG_GATE=1`, as bun-test) when `target/og-wasm/opt-all.txt` exists, else only `packages/knowledge` and `packages/create` (CI's functional job builds no wasm, so it runs those two) | functional | | yes | | |
| `ui-styles` | `og test templates/ui-kit --filter tests/style- --jobs $OG_UI_STYLES_JOBS` (default 3, §Two machines) with the dev `og`: every built-in style pack's screens, desktop and phone, against `crates/og-ui/references/<tier>/kit-style-<pack>[-phone].png`, else `metal`'s; without a renderer the frames are reported unchecked. Added 2026-11-01: neon's style changed in bc65bed4 without its sheets re-recorded, and only a wave end's web-reference caught it. Re-record with `OG_UI_CAPTURE=1 og test templates/ui-kit --filter tests/style-<pack>` on `metal` | functional | | yes (og-cli and what it links, `templates/ui-kit`, `templates/styles`, `crates/og-ui/references`) | | |
| `check-conformance` | `scripts/tsrs-diff.sh`: tsrs (`og check`'s checker) against tsgo `TSGO_VERSION` (its comparison), both at `--checkers 4`, on `packages/engine`'s three tsconfigs, every template (with the `paths` og check gives them here), the five examples and the error corpus `scripts/tsrs-corpus/`; a diagnostic tsgo reports and tsrs does not fails, any other difference is printed as a `diff` line into the step's log and passes; it also refuses a `typescript` devDependency in a template or example and holds the engine's `@tsrs/*` `optionalDependencies` to `TSRS_VERSION`. Reads sources only (no `step_needs`); 5 s | `scripts/wave-end.sh` (stage `conformance`), `scripts/check.sh check-conformance`; in no lane | | | | |
| `bun-test` | `bun run test` in each `packages/*` with a `test` script (the engine's files in parallel workers, `bun test --parallel`; the engine's `typecheck` also checks the six templates' `src/` against the generated declarations, `packages/engine/tsconfig.templates.json`; their playtests run under bun in `og test`); `OG_GATE=1` makes `packages/engine/scripts/build.sh` take the dev addon and `og` and the gate's wasm instead of compiling release and wasm again (below) | full | | | yes | |
| `determinism` | `tools/wasm-determinism/run.sh`: the same math digest and world hashes on the host and on wasm32 scalar and simd128 under node, the host release once more with lanes off and once with lanes on (failing when the lanes run did no parallel work), and the lanes wasm once per physics profile (`scripts/wasm-lanes.sh`, og-physics with `physics3d` or `physics2d` alone as the 3D and 2D lanes modules are built, bound by wasm-bindgen and run on one lane under node: the 2D world hash on the 2D lanes build, the 3D ones on the 3D) | full | | | yes (node from `actions/setup-node`) | |
| perf | `scripts/perf.sh`: `cargo test -p og-tests -p og-runtime --release --no-fail-fast -- --ignored perf` against this machine's baselines (og-tests: `crates/og-tests/perf/<tier>-<adapter>/`, og-runtime: extraction of 20,000 meshes, `crates/og-runtime/perf/<cpu>/`); `--measure` records instead, copies the numbers to `target/evidence/perf/` and restores the tree | `make perf` | | | | `--measure` (below) |
| `web-reference` | `OG_GATE=1 sh packages/engine/scripts/build.sh` first, so `packages/engine/native` holds this checkout's dev og and addon and `dist/web` the gate's wasm (a release build or an older gate leaves older ones there, and the tests cook every project with that og); then, in `packages/engine`, seven groups, `OG_WEB_JOBS` at a time (§Two machines): `fx` (`bun tests/web/reference.ts`: templates/fx-showcase's five stations; `bun tests/web/reference-e5.ts`: its E5 stations, each a fresh run like its playtest; `bun tests/web/reference.ts --looks`: its looks station, 37 presets in one run), `ui` (`bun tests/web/reference-ui.ts`: templates/ui-kit's reference frames; `bun tests/web/a11y.ts`: the accessibility mirror), `gfx` (`bun tests/web/reference-gfx.ts`: og-tests' gfx scenes and og-ui's), `ember-vale` (`reference.ts --project templates/archetypes/third-person`: its three frames), `brasswick` and `meridian-loop` (`reference.ts --project examples/<game> --playtest tests/frames.og.ts`: their nine and ten), `lane-panic` (`bun tests/web/lane-panic.ts`) (§Browser reference tiers), in headless Chrome on WebGPU, compared with `references/webgpu` or else `metal` under og-tests' tolerance (below); path-aware (§Lanes) | `scripts/check.sh web-reference` (`make web-reference`), windows | | | | |
| `wasm-export` | the web modules `wasm-opt` left in `target/og-wasm` (`opt/`, `bindgen/`, `opt.txt`, `opt-all.txt`) and their `stamp` (the commit, the wasm profile and the git tree of each of the wasm's inputs, `rule_wasm`), as `target/og-wasm.tar.gz`; needs `wasm-opt` (§Two machines) | `scripts/check.sh --only wasm-export` on the Mac | | | | |
| `wasm-import` | `OG_WASM_FROM` (that archive, or a directory with its contents) into `target/og-wasm`, refused unless its stamp's input trees equal this checkout's (the paths that differ are named) (§Two machines) | windows | | | | |
| `test-windows` | `cargo nextest run --workspace -E` the cfg(windows) tests (`package(og-cli) & test(/windows/)`), the GPU reference tests when their inputs changed (DX12's frames; as in `test`) and `package(<crate>)` for every crate changed since the step last passed on this machine (its own tests: the Mac ran its dependents' at the commit, and every test binary is still built); every test on a first run, a change to the build's configuration or `third_party`, or `--all` (§Two machines) | windows | | | | |

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

**GitHub Actions runs only by hand.** No push, pull request or schedule
starts a workflow; someone runs one when a shared-runner record is needed
(`gh workflow run ci.yml --ref <branch> -f lane=gate|quick|functional|perf`,
or the Actions tab). It was off during the build-out (removed 2026-10-21:
the shared runner was slower than the local gate, billed, and added
nothing the gate does not cover; the history at 53b876f7 and 9d220790 and
docs/phase-1.md's ci-green section hold what it was when it last ran
green) and came back on 2026-10-25 with the repository published, on every
push to main. That started a 60-minute gate for each of about 200 pushes
in October, half of them red or cancelled by the next push, so the owner
turned every automatic trigger off (2026-10-31: push; 2026-11-01: pull
requests and the nightly). CodeQL's default setup is off too.

`.github/workflows/ci.yml` only provisions the machine
(`.github/actions/provision`: disk, the toolchain from
`rust-toolchain.toml`, a `target/` cache per lane, Lavapipe, ALSA's
headers, node, `scripts/install-tools.sh`'s pinned tools) and runs the
scripts, so a local run and a CI run are the same gate. Every job runs in
a `debian:trixie` container with `--init`, whose Lavapipe (LLVM 19.1.7) is
the og-lavapipe image's that records the vulkan-cpu references (Ubuntu
24.04's LLVM 20.1.2 missed two of them by RMSE 0.0003 and 0.0005). The
jobs, with the times measured on the two-vCPU runner when the workflow
last ran green: `quick` (12.5 min, timeout 45) and `functional` (35 min,
timeout 90), `gate` (`scripts/check.sh` with
`CI` set so nothing is skipped and the shipping wasm profile is measured,
`OG_GATE_JOBS=1`, 60 min, timeout 240), `perf` (`scripts/perf.sh --measure`: a measurement, not a baseline check,
145 min, timeout 240). The runner's environment: `OG_GATE_MACHINE_LOCK=0`
(one gate per runner), `OG_BUN_TEST_TIMEOUT=60000` and
`OG_TEST_TIME_SCALE=3` (a render test on two vCPU of Lavapipe takes 5 to
25 s), `RUSTC_WRAPPER=` (no sccache; `target/` is cached instead),
`CARGO_PROFILE_DEV_DEBUG=line-tables-only` for backtraces. Evidence
uploads are `continue-on-error`, so the account's artifact quota never
fails a lane.

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
binary's tests on threads of one process, runs og-tests, og-ui and
og-render after the other crates with `--test-threads=1`
(`tests/evidence/debt3/particle-sigsegv/NOTES.md`).

### The vulkan-cpu tier's known divergences

Since the vulkan-cpu tier was set aside (§The wave-end run), og-tests'
gfx frames on lavapipe, the orbs' and CI's adapter, are judged against
metal's references under each scene's own tolerance
(`crates/og-tests/src/compare.rs`). Five scenes always exceeded it there
and pass on metal, and kept CI on main red with the same failures on
every run, which hid any new one (the owner's decision of 2026-10-25).
They are declared once, with the RMSE and outlier count measured on
lavapipe against metal and the date, in `VULKAN_CPU_KNOWN_DIVERGENCES`
(`crates/og-tests/src/gfx.rs`); the measurement is in
`tests/evidence/ci/known-frames/NOTES.md`. The list lives in the
harness, not as a file next to `references/`: `gfx::check` is the one
place that knows the running tier (`RenderInfo::backend_tier`) and
whether a scene fell back to metal's reference, the entries are typed and
compile-checked, and the tolerance they bend is the harness's.

`gfx::check` reads the list only when the tier is `vulkan-cpu` and the
scene is judged against metal's reference; on metal, webgpu and
dx12, and for a scene with its own vulkan-cpu reference, nothing changes,
and the tolerance itself is not widened. A listed scene past its
tolerance passes and is reported as `TOLERATED` with its numbers, as long
as it stays within 1.25 times its measured RMSE and outlier count
(`KNOWN_DIVERGENCE_MARGIN`): one that moves further fails, so a
regression in a listed scene on lavapipe is still caught there. A listed
scene now within tolerance is reported with `remove it from
VULKAN_CPU_KNOWN_DIVERGENCES`, so the list shrinks. An unlisted scene
that fails still fails. Each report is a line in
`$OG_EVIDENCE_DIR/gfx-known-divergences.txt`, which the `test` and
`test-quick` steps print after nextest (it shows no output of a passing
test), so the gate's log lists every tolerated frame. A scene joins the
list only by measurement on lavapipe against metal, never to make a red
run green; when the vulkan-cpu requirement returns with references of its
own, the list goes.

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

## Two machines

A merge's screenshots are judged on two machines: the Mac (`metal`) runs
the functional lane, and studio-win (DX12 natively, Chrome on WebGPU over
D3D12) runs what its platform adds to it at the same commit. Before
2026-11-01 one Windows round took over an hour: it built the wasm itself
(1045–1654 s), ran every test (nextest 859 s) and `og-gfx --all` (329–471 s)
and the browser tiers in Chrome and in Edge one group after another
(900–1039 s and 827–842 s). A round now runs this way:

1. **The Mac builds the wasm first**: `scripts/check.sh --only wasm-export`
   (wasm-build and wasm-opt, path-aware, then `target/og-wasm.tar.gz`).
   The wasm is the same bytes on every platform (the release packing
   relies on it too), so it is built once. The archive goes to studio-win
   (any copy: the thread file transfer, a share), and the Mac goes on with
   `scripts/check.sh functional`.
2. **studio-win starts as soon as the archive is there**, at the same
   commit: `OG_WASM_FROM=<archive> scripts/check.sh windows`. `wasm-import`
   checks the archive's stamp: the git tree of each of the wasm's inputs
   (`crates`, `third_party` and the build's configuration, the path-aware
   rule of `wasm-build`) must equal this checkout's, untracked files
   included, or the step fails and names the paths that differ. A wasm
   built from other inputs is never used; a commit that touches none of
   them (a doc, a template) takes the same wasm. studio-win builds only
   its dev og and addon (`build.sh` under `OG_GATE=1`).
3. **studio-win runs only what its platform adds**: `test-windows` (the
   cfg(windows) tests, DX12's GPU reference frames when their inputs
   changed, and the crates changed since its last pass with the crates
   that depend on them; the Mac ran the rest at the commit) and
   `web-reference` in Chrome. `og-gfx --all`, the table of every scene's
   numbers, is a wave's end's, not a round's: the GPU reference tests are
   the same scenes under the same comparison, and they fail. The wave's
   end and a release run every test (`scripts/check.sh test`, `--all`).

**Edge only before a release.** Chrome and Edge matched on all 561
compared lines of a round (2026-11-01) and share the D3D12 backend, so
during development the browser tiers on Windows run in Chrome alone.
Edge (`CHROME_PATH` naming msedge.exe, `scripts/check.sh web-reference`)
is a pre-release step, beside the wave's end (docs/release.md §3).

**The browser tiers run in groups.** web-reference's runs are seven
groups, each with the projects only it cooks and exports: two runs of
fx-showcase (its stations, its E5 stations and its looks all cook and
export it into `dist/test-reference/fx-showcase`) or of ui-kit (its frames
and the accessibility mirror) side by side would write the same
directories, so those stay in one group each. Every run serves on its own
port and starts Chrome with its own profile directory (port 0 and a
`mkdtemp` profile: `tests/web/serve.ts`, `cdp.ts`), and each group writes
its own outputs. `OG_WEB_JOBS` groups run at a time (1 is the old serial
order); each group's output is in
`target/evidence/gate/web-reference/<group>.log`, printed in order at the
end, and the step fails when any group does. A frame that differs between
a serial and a parallel run is a defect to report, never a reason to
raise a tolerance.

**Measured (2026-11-01; Mac: Apple M4 Pro, metal; studio-win: RTX 5060,
D3D12, Chrome 155).**

| Step | Machine | Before | After |
|---|---|---|---|
| The wasm | studio-win | built there, 1045–1654 s | `wasm-import` 2 s plus a 49 MB download, 6 s; the Mac's `wasm-export` 368 s when `crates/` or `scripts/` changed, else reused |
| Edge | studio-win | 827–842 s every round | before a release only |
| web-reference | Mac | 330 s, serial | 101 s at 3 groups (133 at 2, 109 at 4: ui alone runs about 100 s) |
| web-reference | studio-win | Chrome 900–1039 s | 469 s at 2 groups (844 at 1, 466 at 3: fx, about 440 s for its 37 looks stations, bounds it) |
| tests | studio-win | nextest 859 s + `og-gfx --all` 329–471 s | `test-windows`: 21 s for a change outside `crates/`; a crate change runs the GPU reference tests whose inputs it touched and the changed crates' own tests; a first run or a configuration change runs all of them (848 s) |
| ui-styles | Mac | 120–160 s | 32 s at 3 jobs (80 s at 1, 27 s at 4) |

`OG_WEB_JOBS` defaults to 3 on the Mac and 2 on Windows. In 1,565 PNGs on
the Mac and 563 on studio-win, every RMSE and outlier figure was the same
at every job count. Byte for byte, the frames that differed were lane-panic's
two crash cards (the fault is armed by the wall clock, so the tick on the
card varies from run to run, at any job count) and one terrain-grass frame
per run at one LSB on the Mac. A `test-windows` failure no longer cancels
web-reference: the windows lane stops only the steps that read a failed
step's output, so a failing round still reports the browser tiers.

## The wave-end run

`scripts/wave-end.sh` runs a wave's measurements one after another under
the machine's and the target's gate locks, which it holds across its
stages (the `check.sh` runs inside find them held by their parent), so no
other gate starts in between. Each stage runs even when one before it
failed; the exit status is non-zero when any did. Logs, the summary and
the table go to `target/evidence/wave-end/`.

**The vulkan-cpu tier is set aside (2026-10-22).** The CPU-only sandbox
requirement (a playtest's frames on Mesa lavapipe in a sandbox with no
GPU) was set aside by the owner to put features first
(docs/phase-1.md, "The CPU sandbox requirement set aside"; docs/design.md
§5 and §8). The default run is the `gate`, `web`, `conformance` and `release` stages
(`conformance`: `scripts/tsrs-diff.sh`, og check's tsrs held to tsgo;
`release`: the tarballs and the install test, docs/release.md §3); the
lavapipe stages below (`cross`, `prepare`, `gfx`, `ui`, `playtests`,
`table`) stay in the script and run only when `--stages` names them. No
`references/vulkan-cpu/` directory is kept any more (the last 25 copies
went on 2026-10-22); a tier with no directory is judged against `metal`,
as every reference harness already does. What follows about those stages
describes the script as it is, for when the requirement returns.

| Stage | What |
|---|---|
| `gate` | `scripts/check.sh --all` |
| `web` | `scripts/check.sh web-reference` |
| `release` | `scripts/release.sh pack` then `verify`: the five packages as release tarballs in `dist/release/<version>/` with `manifest.json`, and the install test (`tests/install/run.ts`) from them on this machine and in the linux-x64 container (docs/release.md §3); publish never runs here |
| `cross` | `cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.31` of `-p og-tests --bin og-gfx`, of og-ui's test binary (`--test tests`, run with the filters `gfx:: kit_gfx:: materials_gfx:: text_gfx::`) and of `-p og-host-node`, the addon copied to `packages/engine/native/origin-engine.linux-x64.node` |
| `prepare` | each template cooked (`og cook`, the gate's dev `og`) and bundled for the native host into `<template>/.og/game.native.js`, as `og test` does, on the Mac |
| `gfx` | `og-gfx --all --capture target/lava-gfx` in `og-lavapipe-bun:local` (`docker run --rm -v <checkout>:<checkout> -w <checkout>`, `OG_TEST_ASSETS=<checkout>/crates/og-tests/assets`, the og-gfx binary as `--entrypoint`; no `--platform`: the amd64 image runs under emulation, and the checkout sits at its own path because the bundles hold absolute paths) |
| `ui` | og-ui's GPU test binaries in the image, against `crates/og-ui/references/vulkan-cpu/` (else `metal`) |
| `playtests` | every playtest of the projects with vulkan-cpu frames (each `templates/`, `templates/archetypes/` or `examples/` project with `references/vulkan-cpu/`, and ui-kit, whose frames are og-ui's: 100 playtests on 2026-10-21; `--templates` names others) through `bun packages/engine/src/node/playtest-run.ts --file <playtest> --entry <template>/.og/game.native.js --root <template>`, in one container |
| `table` | `scripts/frames-vs-metal.ts`: outliers and RMSE (og-tests' rule, fx-showcase's `difference`) of the captured og-tests frames against `metal` and against the committed `vulkan-cpu`, and of every committed `references/vulkan-cpu/` frame (og-ui, the templates, the examples) against its `metal`; printed and written to `vulkan-cpu-vs-metal.tsv` |

Nothing is recorded unless `--record` names it: `<template>` or
`<template>/<playtest>` runs those playtests with `OG_FX_CAPTURE=1` and
`OG_UI_CAPTURE=1`, `gfx` or `gfx/<scene>` copies the capture into
`crates/og-tests/references/vulkan-cpu/`, `og-ui` runs og-ui's GPU tests
with `OG_UI_CAPTURE=1`. A frame recorded there is what CI's trixie
container renders (§CI): a rendering change that lands on main with its
metal frames alone leaves the vulkan-cpu one stale, and CI red, until the
wave's end records it (2026-10-21: text-bidi after a2c5e44b, stylised
after the render-style merge). `--stages cross,gfx,table` runs
some stages (in their order); `--dry-run` prints every command and runs
none.

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
  as on a machine with no usable adapter. That is the platform's `engine`
  and `probe` sandboxes (docs/design.md §5: they carry no renderer), and the
  switch makes the path testable on a machine with a GPU. Without a
  renderer the run is the same run: the simulation, the probe, `dump_*`,
  snapshots and the world hash are identical (rendering never feeds back
  into simulation; the native host keeps the play surface at `--size`); a
  `screenshot` step writes no PNG and logs, not as a warning, `<t>s
  screenshot: no renderer, frame unchecked (<file>)` (og-probe's
  `FRAME_UNCHECKED`); `hello.tier` is `none` and `capture` answers null.
  `og run --json` reports `tier: "none"` and the frames in `unchecked`;
  `og test` runs every playtest check outside `t.frame`, prints one
  `~ <file>: no renderer, frame unchecked` line per screenshot and the
  count in its summary, and exits as it would on the other checks alone;
  `og polish` reports frame time `n/a (no renderer)`; `og doctor` reports
  the renderer `none`, not as a problem. Any other value, or none, leaves
  the adapters alone; the web host always has its canvas's. The gfx
  suites (og-tests, og-ui's reference frames, `og-gfx`) need a renderer and
  fail under the switch. `packages/engine/tests/no-renderer.test.ts` and
  og-runtime's `tests/no_renderer.rs` hold the contract; frames are verified
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
their view out for 16:9 instead of the surface; `og run` and `og polish`
now create the engine with `deferStart` and run the start once the
renderer is attached, as the web Worker always did.

WebGPU is judged against `metal` (2026-10-21, the UI shell branch
merged with main 376f3cd):

| Station | outliers | RMSE |
|---|---|---|
| post | 0.000 % | 0.00142 |
| lens | 0.010 % | 0.00228 |
| fx | 0.000 % | 0.00204 |
| 2d | 0.000 % | 0.00251 |
| stylised | 0.027 % | 0.00389 |

The E5 stations (sky, weather, water, shaders, controller, joints, rope,
fluid) each have a playtest of their own, so `tests/web/reference-e5.ts`
reads each playtest's station and wait from `tests/<station>.og.ts`. It runs
reference.ts once per station with `--script 0.5:station:<name> --shots
<name>@<tick>`, so the browser run is the same fresh run. Against `metal`
(2026-10-21; the stations are now every `stationPlaytest`, 20 of them),
every station is within RMSE 0.0085 on WebGPU (fluids3d; the rest under
0.0020).

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
`crates/og-ui/tests/support/text_scenes.rs`): the module cannot cook, so
the runner first cooks `crates/og-ui/tests/text-fonts` (the fonts' import
settings) with templates/ui-kit's font files through `og cook`
(`tests/web/text-fonts.ts`, the `og` of `packages/engine/native/` or
`$OG_BIN`), serves the blobs beside the module and hands each page the
manifest and files (`og_gfx_text_manifest`, `og_gfx_text_file`), as og-ui's
native `text_gfx` decodes the same cook from its bytes. They have not run
in a browser on record yet.

### og-ui's kit frames in the browser

`packages/engine/tests/web/reference-ui.ts` runs templates/ui-kit's
reference playtests in headless Chrome: `frames-desktop`, `frames-phone`,
`players`, `locales` and the twelve `style-<pack>[-phone]` playtests, all of
them (72 frames: 23 desktop, 22 phone, 15 locale screens, 12 style sheets of
21 cells). It bundles each playtest (`bundleGame`, as `og test` does), reads
its size, commands and screenshot times, and runs reference.ts once per
playtest with `--project templates/ui-kit --references
crates/og-ui/references --size <w>x<h> --script <the commands without
screenshot, quit and dump> --shots <file>@<round(60 t)>,…`, so each
browser run is the same fresh run as the native playtest. A style playtest
adds `--sheet kit-style-<pack>[-phone].png,<factor>,<cols>`: each shot is
box-shrunk (2 on desktop, 3 on a phone, as `tests/styles.ts` does) and
compared with its cell of the sheet; the gallery shot that only
proves the pack applied is not a cell. Frames are judged against
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
