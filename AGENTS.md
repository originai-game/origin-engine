# Origin Engine repository guidance

Current rules only; their history is in git. Design: `docs/design.md`, UI
`docs/ui.md`, gate and toolchain `docs/ci.md`, release `docs/release.md`,
status `docs/roadmap.md`. Platform contracts implemented here:
`origingame/docs/game-workflow.md`, `origin-agent/packages/contracts/src/probe.ts`
(probe, protocol 5).

## Ownership

- This repository owns the engine runtime, its TypeScript API, the cook, the
  `og` tool, the project skeleton, the UI packs (kits and style packs) and the
  generated wiki. Accounts, money, publishing, threads and the agent loop stay
  in the platform, reached through the host ABI with game-scoped tokens. No
  platform code imports engine internals; no engine code imports platform code.
- The platform consumes released versions only: the signed per-target archives
  at `https://origingame.ai/downloads/engine/<version>/` (`docs/release.md`
  §1), never npm, a git revision or a checkout. A version is tested locally
  (the install test, the platform's local stack) before a sandbox installs it.
  Development sandboxes are InstaBox (`tests/install/instabox.ts`); E2B is
  production only.
- The platform is `originai-game/origin-agent` (agent server, runner, web,
  probe contract, engine pin, `knowledge/`) and `originai-game/origingame`
  (everything else).
- Public contracts; changing one is a break (a minor version with a
  breaking entry in its changelog fragment):
  - `packages/knowledge/scripts/mount.ts`: `installed`, `mount`, `summary`.
  - `packages/knowledge/scripts/lib.ts`: `payload(roots: Roots): PayloadFile[]`,
    `defaultRoots(pkg?: string): Roots`, `PayloadFile { path; source; bytes;
    symlink }`, `Roots { pkg; template; engine }`.
  - `packages/knowledge/scripts/brief.ts`: `artBrief`, `readLook`,
    `presetLook` (re-exported by `@origingame/create-game`; `docs/art.md` §2.2).
- The engine ships no skills. What a game author needs beyond a signature
  (workflow, thresholds, pitfalls) is a guide in `crates/og-api/guides/`,
  generated into the wiki and read through `og api find` and `og api show`,
  or `og` command help. A guide never restates the API and names no
  platform tool or service.

## Principles

- **The engine supplies facts and tools; the agent decides.** No engine
  command scores, grades or refuses on taste. Checks report measurements and
  findings; thresholds belong in the guides.
- **No gameplay templates.** `og create` writes one project skeleton; the
  creation choices go into project.ogt's `create` block as constraints and
  hints. Polished templates and showcase content come after the feature work.
- **Feature checks live in fixtures** under `tests/fixtures/`, one per
  feature, a few seconds each, run only when their inputs change.
- **Fix at the root; port the state of the art.** A problem is fixed in the
  engine where it arises, never worked around in a game, kit or test. A
  missing feature starts from a search of current papers and permissive
  implementations; keep their licences under `third_party/` or in the header.
- **Size cuts never cost features.** Data only some games need is loaded or
  packed on demand, not removed. Growth from a feature is accepted within the
  size gate.
- **Frozen is not obsolete.** A frozen kit, style pack or feature is one
  whose quality is not there yet; it stays in the tree until it is rebuilt.
  A cleanup deletes only what the owner names obsolete.

## Non-negotiable contracts

- **Rust stays inside the engine.** Game code is TypeScript; a game author
  never runs `cargo`. A feature that would need game-side Rust is an engine
  feature.
- **One API, two hosts.** The TypeScript API is generated from one IDL for the
  wasm and native hosts; a member on one and not the other is a bug. Backend
  limits are capability queries. Edit `crates/og-api/api/<module>.ogt`
  (`prelude.ogt` holds the header and units, `parts.txt` the order).
- **Deterministic by construction.** Fixed tick, engine-owned seeded
  randomness, no wall clock in the tick, snapshot and restore of the whole
  world. Rendering never feeds back into simulation; presentation state is
  checkpointed beside the world, outside `world_hash` (`docs/design.md` §2.4).
- **Observable from the first frame.** The command channel, the probe,
  `dump_world`/`dump_entity`/`dump_ui` and frame capture exist in every build
  except `publish`, which keeps only the player-facing probe members.
- **Reflection is the single schema** for the inspector, Tune, the probe,
  scene text and the wiki.
- **Two text formats, one blob format.** `.ogt` text cooked into `.ogb` blobs
  with a type hash, version and cook stamp.
- **The API is for a reader who has never seen it.** Zero-value desc structs,
  no hidden state, errors as values, one documented entry per module, one
  naming convention, the three.js vocabulary where it fits.
- **wgpu as published.** Never vendored or patched; work around it in
  og-render or og-host-web.
- **Rendering never blocks the engine thread on the web.** Engine in a Worker
  with an OffscreenCanvas; the main thread owns input and the shell.
- **Brand is one ink.** Splash, loading and "made with" use the platform's
  `packages/brand` and the design tokens.
- **One design language for the engine and origin-agent.** `packages/design`
  (`tokens.json` and its document) generates og-ui's default style
  (`crates/og-ui/src/theme.rs`, drift-checked); origin-agent pins it by
  version. It binds the default style only, never a game's.

## Building and testing

- Toolchain: a pinned nightly with LLVM on every target (no Cranelift);
  versions and pins are in `docs/ci.md` §Toolchain, the one place they are
  stated. A version move changes that table and its pin together.
- **One target directory per checkout.** Never set a private
  `CARGO_TARGET_DIR` or copy the tree. Never delete a `target/`; clean one
  crate with `cargo clean -p`, and free disk with `scripts/clean-targets.sh`.
- **Lanes.** Parallel work on macrunner takes a persistent worktree with
  `scripts/lane.sh take lane-N <stream>` and frees it after landing
  (`scripts/lane.sh free lane-N`, `--list` to see them). Never switch or reset
  a lane you did not take, never `git worktree add` a fresh one.
- **Concurrent gates, bounded work.** `scripts/check.sh` takes shared admission
  and a checkout-local output lock. CPU steps take no machine work lock;
  each gate defaults to two steps with two Cargo/nextest jobs each. Rendering
  steps take FIFO GPU slots through `scripts/lock.sh`. Measurements take
  exclusive admission, drain active gates, then acquire GPU slots separately.
  Never hold all GPU slots while waiting for a child gate to acquire one.
  Source `scripts/lock.sh`; never a private copy.
- **Lanes of the gate** (`docs/ci.md` §Lanes): `quick` (fmt, clippy, logic
  tests, drift, package tests) is the loop and what a landing runs;
  `full` adds all Rust and Bun tests, retained UI-pack suites, fixtures,
  wasm builds, shipping-size checks and browser functional tests. Wave-end
  runs `full --all`, perf, browser references, conformance and release
  pack/verify. Selection uses dependency inputs; an unclassified change
  runs checks rather than skipping them. Never touch files or delete
  `target/og-gate/` to force a run; use `--all`.
- **Profiles.** `dev` for everything native in the gate, `release` for what
  ships natively, `wasm` for every wasm build. Compiling one crate in two
  profiles in one step is a defect.
- **Tests never run `cargo`.** Rust tests use `env!("CARGO_BIN_EXE_og")`; bun
  and browser tests use the gate's `og` (`tests/support/og-bin.ts`). Shared
  fixtures are made once under a file lock; what cannot run in parallel is
  declared in `.config/nextest.toml`.
- **A missing GPU is a warning:** the test counts as skipped or unchecked,
  never as passed. Tests are not split across machines.
- **Metal on macrunner is the reference platform.** Windows (DX12, Chrome,
  Edge, Firefox) compatibility runs on studio-win; Safari and Android wait.
  The web is WebGPU only. Nothing runs on lavapipe; no `references/vulkan-cpu/`.
- Generated `.d.ts` and wiki are drift-checked; never hand-edit them.
- A test is kept only when it fails on a plausible regression; a re-recorded
  frame is one the change must move, and is named in the report.
- There is no GitHub CI; the gate is `scripts/check.sh` on macrunner.

## Pitfalls

- Claim a lane only with `scripts/lane.sh take`; for an existing branch,
  take first and switch inside the claimed lane. `--list` can be seconds
  stale.
- Never kill a process you did not start: `pgrep -f "check.sh quick"`
  matches other lanes' gates. Keep `$!` and stop only that; tell the owner
  of anything else in your way. A killed script's `og test` keeps running;
  check `ps` for `target/debug/og` before re-recording into the same files.
- Never edit a shell script while a running shell is reading it.
- GPU slots: take every slot for one measured run, not a whole series, and
  call `og_gpu_acquire_all` only when the GPU is idle (it holds `gpu-all`
  while it waits). Restore anything moved aside under the lock (Chrome's
  Metal shader cache) before the slots free. Per-process GPU limits add up
  across test runners: fix contention in the scheduler, not a timeout.
- Land ready heads in stacks: regenerate once, run `quick` once, run the
  targeted Metal checks together, return only a failing head.
- `local main` can be ahead of `origin/main`: compare full object ids and
  pin a measurement worktree to the requested commit.
- The look presets are `include_str!` into og-cook: rebuild `og` (and the
  addon) before capturing after a preset edit. Regenerate a game's copied
  assets from committed sources before recording its references.
- A capture records its effective state (AA, frame history, selection)
  beside the image; equal ticks do not prove equal temporal history. A
  checker's element attribution is a claim to check against the pixels.
- Before calling an unknown option a defect, read the public parser and its
  internal callers (`og run --device` passes `--preflight` itself).
- A text change must not, as a side effect, change other fonts' cooked
  output, charset or run-time mode. Run-time glyphs (`font { runtime
  true }`), wider charsets and colour glyphs are features a font declares,
  chosen for how the text looks and what it costs; check missing-glyph
  diagnostics, frame cost and the cooked payload budget. Freeze the code
  under test for a cross-host hash run.
- Style packs are shipped asset roots: their tests and references live in
  `templates/styles/{tests,references}/<pack>`. After moving fixtures, run
  the cook-report budget and threaded-cook output tests.
- A toolchain move runs workspace Clippy with all targets, `--keep-going`
  and `-D warnings`.
- Walkers and packagers never follow directory symlinks and skip `.og/`,
  `cooked/`, `og-out/` and installs; a project must package after it ran.
- The platform development key is full-featured: when a flow fails with
  it, fix the credential contract, not the flow.

## Streams and landing

- The coordinator thread delegates streams with `create_thread`: at most eight
  author streams at once, each owning a whole feature area on one branch.
  Default mode: `external-agent:claude-code` with
  `["model:claude-opus-5-5","effort:medium"]`, project `origingame/origin-engine`,
  base commit named in the brief. Infrastructure, hard logic, backend work,
  file splits and compatibility may run in Amp `high`. A hard problem (one
  that resisted several rounds, a cost target missed widely, a look bounced by
  art, a paper port) may go to Claude Code with
  `["model:claude-fable-5-1","effort:high"]`, briefed with every attempt.
  A stopped thread is not resumed; a new one starts from `origin/main`.
- Authors work in their own lane on macrunner with the Metal GPU and run their
  own tests, scoped GPU suites and frames.
- **Landing.** A head that moves no reference frame and changes no look is
  landed by its author with `scripts/land.sh`, then posted to Slack. The command
  runs quick and serialises rebase/push separately from the gate lock; disjoint
  changes without common inputs reuse the passing gate. Overlap or shared
  inputs rerun quick with its input cache before retrying. Never push main
  directly. A head that moves frames or changes a look goes
  to the art runner, who reviews the look against commercial games and lands
  it with the same command. A failure found later goes back to its branch as
  a fix-forward commit.
- A file split moves code verbatim (only `mod`, `use` and visibility edits)
  and is reviewed with `git diff --color-moved`.
- Reports carry their numbers and frame paths; evidence is committed only when
  a test reads it. Every change a game sees, and every break with its fix,
  is recorded in the same commit as one new fragment in
  `changelog/unreleased/` (`scripts/changelog.ts`); a merge never edits
  `packages/engine/changelog.json`, which `release.sh fold` and `bump`
  build from the fragments (`docs/release.md` §3). Platform asks are drafted in the report and
  collected by the coordinator for the release.
