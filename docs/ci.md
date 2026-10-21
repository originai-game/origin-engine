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

## What runs where

| Step | Command | Lane | CI `quick` (PR) | CI `functional` (PR) | CI `gate` (push to main, PR) | CI `perf-nightly` (03:17 UTC, manual) |
|---|---|---|---|---|---|---|
| `fmt` | `cargo fmt --all --check` | every | yes | yes | yes | |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | every | yes | yes | yes | |
| `test-quick` | `cargo nextest run --workspace --exclude og-tests` (`cargo test` without nextest); og-ui's and og-render's GPU tests left out when their inputs did not change (§Lanes) | quick | yes, on lavapipe | | | |
| `test` | `cargo nextest run --workspace && cargo test --workspace --doc` (`cargo test --workspace` without nextest); gfx tests in `og-tests`, left out with og-ui's and og-render's when their inputs did not change (§Lanes). Runs after `wasm-build`, whose og-render artifact `og-tests::wasm` takes when it is newer than og-render's sources (else that test runs `cargo check --target wasm32-unknown-unknown -p og-render` itself) | full, functional | | yes, on lavapipe | yes, on lavapipe | |
| `wasm-build` | `cargo build --profile wasm --target wasm32-unknown-unknown -p <crate> --lib` (`Cargo.toml`'s `[profile.wasm]`: release, the load-time crates at opt-level z) for `scripts/wasm-crates.txt`, plus the `publish` feature variant of crates marked `publish` in `target/wasm-<variant>/` (cargo 1.99 keeps one fingerprint per unit whatever its features, so two feature sets in one target dir recompiled og-probe, og-runtime and og-host-web on every run: 16.7 s and 12.2 s with nothing changed, 0.4 and 0.5 s apart; `tests/evidence/P2/tooling/wasm-build-relink.md`) and the lanes variants of crates marked `publish lanes` (one per `lanes` line: og-host-web's `lanes2d` then `lanes`, the 2D and the 3D publish features with wasm atomics, `scripts/wasm-lanes.sh` in `target/wasm-mt/`, which the two share; §The lanes wasm); copies to `target/og-wasm/raw/` | full | | | yes | |
| `wasm-opt` | `wasm-bindgen --target web --out-name origin-engine` (`origin-engine-lanes` for a lanes variant, `-lanes*`; when the module uses it) then `scripts/wasm-opt.sh`, the one wasm-opt invocation (`-Oz --enable-bulk-memory --enable-nontrapping-float-to-int --low-memory-unused`; rustc links wasm32 stack-first, so the low 1 KiB is only the bottom of an overflowing stack), which `packages/engine/scripts/build.sh` also ships with (`--threads` adds `--enable-threads` for the lanes variant); `opt.txt` lists the shipping (publish and lanes, one of each per game profile) variants, `opt-all.txt` every one | full | | | yes | |
| `size-gate` | `scripts/size-gate.sh`: brotli `-q 11` of each file in `opt.txt` ≤ `wasm_core_brotli_bytes` (a page downloads one variant; the table also prints the sum and the largest) | full | | | yes | |
| `drift` | `og api check` (generated `.d.ts`, wiki, JSON schema, Rust glue vs `crates/og-api/api.ogt`), the dev `og` the test step built | every | yes | yes | yes | |
| `bun-quick` | `bun run test` in `packages/knowledge` and `packages/create` (no native build) | quick | yes | | | |
| `bun-functional` | `bun run test` in each `packages/*` with the last full lane's wasm (`OG_GATE=1`, as bun-test) when `target/og-wasm/opt-all.txt` exists, else only `packages/knowledge` and `packages/create` (CI's functional job builds no wasm, so it runs those two) | functional | | yes | | |
| `bun-test` | `bun run test` in each `packages/*` with a `test` script (the engine's files in parallel workers, `bun test --parallel`; the engine's `typecheck` also checks the six templates' `src/` against the generated declarations, `packages/engine/tsconfig.templates.json`; their playtests run under bun in `og test`); `OG_GATE=1` makes `packages/engine/scripts/build.sh` take the dev addon and `og` and the gate's wasm instead of compiling release and wasm again (below) | full | | | yes | |
| `determinism` | `tools/wasm-determinism/run.sh`: the same math digest and world hashes on the host and on wasm32 scalar and simd128 under node, the host release once more with lanes off and once with lanes on (failing when the lanes run did no parallel work), and the lanes wasm once per physics profile (`scripts/wasm-lanes.sh`, og-physics with `physics3d` or `physics2d` alone as the 3D and 2D lanes modules are built, bound by wasm-bindgen and run on one lane under node: the 2D world hash on the 2D lanes build, the 3D ones on the 3D) | full | | | yes (node from `actions/setup-node`) | |
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
both locked Cargo dependency graphs, installs `tools/bench` from its Bun
lockfile, and caches the engine's pinned TypeScript executable. It does not
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
  gates (§The wave-end run): `scripts/check.sh --all`, `scripts/check.sh
  web-reference` and the vulkan-cpu frames on lavapipe, serially, with the
  table of every vulkan-cpu frame against metal.
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

## The wave-end run

`scripts/wave-end.sh` runs a wave's measurements one after another under
the machine's and the target's gate locks, which it holds across its
stages (the `check.sh` runs inside find them held by their parent), so no
other gate starts in between. Each stage runs even when one before it
failed; the exit status is non-zero when any did. Logs, the summary and
the table go to `target/evidence/wave-end/`.

| Stage | What |
|---|---|
| `gate` | `scripts/check.sh --all` |
| `web` | `scripts/check.sh web-reference` |
| `cross` | `cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.31` of `-p og-tests --bin og-gfx`, of og-ui's GPU test binaries (`--test gfx --test kit_gfx --test materials_gfx --test text_gfx`) and of `-p og-host-node`, the addon copied to `packages/engine/native/origin-engine.linux-x64.node` |
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
build `og_host_web-lanes2d.wasm` (`publish,physics2d,shaping`) and then
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
  `dist/web-publish`) and `publish2d` (`publish,physics2d,shaping`, the 2D module
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

## Browser reference tiers

og-tests' gfx references cover `metal` and `vulkan-cpu` (and `webgl2` for
four scenes, §og-tests' gfx scenes in the browser); og-ui's 83 reference
frames cover `metal` and `vulkan-cpu` (the browser tiers run 76 of them), and the browser tiers are judged
against `metal` (§og-ui's kit frames in the browser);
fx-showcase's browser tiers
come from `packages/engine/tests/web/reference.ts` (`make web-reference`): the dev
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
compares (or with `--record` writes) `<project>/references/<tier>/`. What
each template commits under `references/`: `metal` alone for idle, rhythm,
roguelike and tower-defense; `metal` and `vulkan-cpu` for match3, runner
and topdown; `metal` and a WebGL2 `beacon` for third-person (below);
`metal`, `vulkan-cpu` and one WebGL2 frame (`water`, below) for
fx-showcase. The UI shell round (2026-10-21) deleted topdown's `webgpu`
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
gets its own reference only for a difference the tier makes by design
(§og-tests' gfx scenes in the browser).

`web-reference` also runs fx-showcase's looks station on both tiers:
reference.ts with `--looks` (output in `dist/test-reference/looks`) runs
`tests/looks-script.ts`'s commands (the looks playtest's, without its
screenshots and `quit`; the playtest and the runner import the same
module, so they cannot drift) in one run and takes one shot per preset
of the catalogue at its screenshot's tick (37 shots, the last at tick
1164), each compared with `references/<tier>/looks-<slug>.png`, else
`metal`'s. The plain and look-taken-off frames are the playtest's own
check and are not taken. WebGPU has no looks references of its own
(every frame within RMSE 0.0098 of `metal`, 2026-10-21). WebGL2 keeps
two by design, `paper-craft` and `voxel-worlds` (RMSE 0.0187 and 0.0113
against `metal` at cf5f2b9): both turn on `PostEffects.ao` with the
default method, which is GTAO on the full tier and SSAO on WebGL2
(og-render's README, "Ambient occlusion by tier"); `sculpted-clay` uses
it too but stays within tolerance (0.0062) and is judged against
`metal`. The other 34 are judged against `metal` on WebGL2 since
cf5f2b9 and 1a9037a: before them, TAA's history drifted sideways on the
GL backend with a still camera (every edge about a pixel off, which the
looks' posterize, ink and outlines turned into whole moved lines: 30
frames at RMSE 0.010 to 0.046, present since the station joined the
tiers). The cause (1a9037a): wgpu's GL backend binds no sampler to a
depth texture read with `textureLoad`, GLSL ES treats the texture, left
on linear filtering, as incomplete and reads 0, so every pixel
reprojected from the near plane; the resolve now samples the depth
through a nearest sampler, decides a still camera on the CPU (cf5f2b9)
and reads each pixel's own history texel. `taa-sharp`, the sliding
camera, applies on every tier since 1a9037a and is WebGL2's
moving-camera TAA frame: RMSE 0.00028 against `metal` (WebGPU 0.00024).

`web-reference` also runs Ember Vale (`templates/archetypes/third-person`)
on both tiers: reference.ts with `--project
templates/archetypes/third-person --out dist/test-reference/third-person`,
`tests/frames.og.ts`'s commands without its screenshots and `quit`
(`1.0:run;1.1:warp:0;1.3:key_down:KeyF;3.5:key_up:KeyF;3.6:light:1;3.7:light:2;3.8:light:3`)
and `--shots title@54,beacon@204,dawn@432` (each at its screenshot's tick,
60 t). A change to that playtest changes the step's command in
`scripts/check.sh` with it. WebGPU has no references of its own: against
`metal` (2026-10-19) title is at RMSE 0.00131, beacon 0.00475 with
0.060 % outliers, dawn 0.00067. On WebGL2 title (0.00428) and dawn
(0.00180) are judged against `metal` too; the beacon keeps a WebGL2
reference by design. Since the shadows-aa round (2026-10-21) the CPU
particle path bounces off the scene's colliders (og-runtime hands it the
og.Collider boxes and the terrain's height field), so WebGL2's ember
sparks bounce on the vale and the plinth as the full tier's do off the
depth buffer: against main's re-recorded `metal` beacon the WebGL2 frame
went from RMSE 0.0185 with 0.919 % outliers (sparks falling through) to
0.0165 with 0.809 %. It stays over 0.01 by design: WebGL2's 4× MSAA
against the full tier's TAA (RMSE 0.0130 on its own in the showcase
round's per-spark read; docs/phase-1.md), and sparks that bounce off the
colliders' exact surfaces rather than off the depth buffer's (with its
thickness band), so their paths after the first bounce differ. The WebGL2
beacon was re-recorded (and reproduced at RMSE 0); webgpu's beacon is at
RMSE 0.00475 against `metal`.

`bun tests/web/archetype-hashes.ts [--webgl]` compares the three
archetypes' world hashes across hosts: each replay playtest's script runs
in Chrome and in `og run` at the page's play surface (1280×577 in the
headless window), and the two `dump_world` hashes at the same tick must be
equal. They are on WebGPU and WebGL2 (2026-10-19,
`tests/evidence/E7/archetype-hashes-*.md`). They used to differ because the
native `og run` called `GameDesc.start` before attaching its renderer, so
`og.render.capabilities` answered 0×0 there and runner and match3 laid
their view out for 16:9 instead of the surface; `og run` and `og polish`
now create the engine with `deferStart` and run the start once the
renderer is attached, as the web Worker always did.

Both tiers are judged against `metal` (2026-10-21, the UI shell branch
merged with main 376f3cd):

| Station | webgpu outliers | webgpu RMSE | webgl2 outliers | webgl2 RMSE |
|---|---|---|---|---|
| post | 0.000 % | 0.00142 | 0.000 % | 0.00506 |
| lens | 0.010 % | 0.00228 | 0.010 % | 0.00227 |
| fx | 0.000 % | 0.00204 | 0.013 % | 0.00288 |
| 2d | 0.000 % | 0.00251 | 0.000 % | 0.00254 |
| stylised | 0.027 % | 0.00389 | 0.027 % | 0.00389 |

The E5 stations (sky, weather, water, shaders, controller, joints, rope,
fluid) each have a playtest of their own, so `tests/web/reference-e5.ts`
reads each playtest's station and wait from `tests/<station>.og.ts`. It runs
reference.ts once per station with `--script 0.5:station:<name> --shots
<name>@<tick>`, so the browser run is the same fresh run. Against `metal`
(2026-10-21; the stations are now every `stationPlaytest`, 20 of them),
every station is within RMSE 0.0085 on WebGPU (fluids3d; the rest under
0.0020) and 0.0085 on WebGL2 (fluids3d, terrain 0.0078, environment
0.0078; the rest under 0.0056), except water on WebGL2 at RMSE 0.0216
(0.93 % outliers), which keeps its WebGL2 reference. That is expected: WebGL2 has no
depth-prepass reflection march, so its water reflects the prefiltered sky
cubemap (`crates/og-render/README.md` §E5).

The runner found four defects, all fixed in og-render before the tiers
were recorded (`crates/og-render/README.md` §The overlay and §Backend
tiers): the overlay blended encoded sRGB values on the WebGPU canvas's
`Bgra8Unorm` (the HUD panel was 87,89,99 against 123,128,135; RMSE up to
0.024); on WebGL2 TAA's `textureLoad` from a depth texture had no GLSL
form, the lit skinned pipeline needed 18 of the GL backend's 16 texture
slots, and the twelve-layer local shadow map was created as a cube-map
array, which left every shadowed light black. 
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
difference the tier makes by design, never to absorb a defect.
`shadows-crypt` (wave 13, web findings #4) has none: its shadowed point
light is two paraboloid hemispheres on WebGL2 and six cube faces on
`metal`, which store the same faces since the paraboloids' winding was
mirrored (the paraboloid frame within RMSE 0.0036 of the cube one
natively), and the lamp's glass casts no shadow on either.
`shadow-flags` (owed row 87) has none either: per-instance cast and
receive flags under the sun's cascades alone. Each tier
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
module (the 3D or the 2D publish variant by the game's profile, or that
profile's lanes variant on an isolated page; docs/web-export.md §Variants), so each
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
