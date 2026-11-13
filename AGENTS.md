# Origin Engine repository guidance

Current rules only; history is in git. Design `docs/design.md`, UI
`docs/ui.md`, gate and toolchain `docs/ci.md`, release `docs/release.md`,
status `docs/roadmap.md`. Platform contracts implemented here:
`origingame/docs/game-workflow.md` and
`origin-agent/packages/contracts/src/probe.ts` (probe, protocol 5).

## Ownership

- This repository owns the engine runtime, its TypeScript API, the cook, the
  `og` tool, the project skeleton, the built-in UI components and style
  packs, and the generated wiki. Accounts, money, publishing, threads and the
  agent loop stay in the platform (`originai-game/origin-agent` and
  `originai-game/origingame`), reached through the host ABI with game-scoped
  tokens. Neither side imports the other's internals.
- The platform consumes released versions only: the signed archives at
  `https://origingame.ai/downloads/engine/<version>/` (`docs/release.md` §1).
  A version is tested locally before a sandbox installs it. Development
  sandboxes are InstaBox (`tests/install/instabox.ts`); E2B is production.
- Public contracts; changing one is a break (a minor version with a breaking
  changelog fragment):
  - `packages/knowledge/scripts/mount.ts`: `installed`, `mount`, `summary`.
  - `packages/knowledge/scripts/lib.ts`: `payload`, `defaultRoots`,
    `PayloadFile`, `Roots`.
  - `packages/engine/src/provider.ts` (`@origingame/engine/provider`) and
    `PROVIDER_VERSION` (`docs/design.md` §2.8 Providers).
  - `packages/knowledge/scripts/brief.ts`: `artBrief`, `readLook`,
    `presetLook` (`docs/art.md` §2.2).
- The engine ships no skills. What a game author needs beyond a signature is
  a guide in `crates/og-api/guides/` (read through `og api find` and
  `og api show`) or `og` command help. A guide never restates the API and
  names no platform tool or service.

## Principles

- **The engine supplies facts and tools; the agent decides.** No engine
  command scores or refuses on taste; thresholds belong in the guides.
- **No gameplay templates.** `og create` writes one project skeleton; the
  creation choices go into project.ogt's `create` block.
- **Fix at the root; port the state of the art.** A problem is fixed where it
  arises, never worked around in a game, component or test. A missing feature
  starts from current papers and permissive implementations; keep their
  licences under `third_party/` or in the header.
- **Size cuts never cost features.** Data only some games need is loaded or
  packed on demand.
- **Frames are judged on quality, never on their difference.** Any change
  may move frames. A frame is reviewed for what it shows (against commercial
  games: no black, broken, clipped or lost detail), never by its pixel
  difference from the old one; reports carry no diff numbers. A reference is
  only the last recorded frame, refreshed in batches.
- **Structure before size.** A refactor makes the structure clear: state
  owned by its stage, pipelines as named steps, boundaries carried by types.
  No file is split for its line count.
- **A cleanup deletes only what the owner names obsolete.** Unfinished
  features stay in the tree until they are rebuilt.

## Non-negotiable contracts

- **Rust stays inside the engine.** Game code is TypeScript; a game author
  never runs `cargo`.
- **One API, two hosts.** The TypeScript API is generated from one IDL
  (`crates/og-api/api/*.ogt`) for the wasm and native hosts; a member on one
  and not the other is a bug. Backend limits are capability queries.
  Generated `.d.ts` and wiki are drift-checked; never hand-edit them.
- **Deterministic by construction.** Fixed tick, seeded randomness, no wall
  clock in the tick, whole-world snapshot and restore. Rendering never feeds
  back into simulation (`docs/design.md` §2.4).
- **Observable from the first frame.** The command channel, the probe, the
  dumps and frame capture exist in every build except `publish`.
- **Reflection is the single schema** for the inspector, Tune, the probe,
  scene text and the wiki. `.ogt` text cooks into `.ogb` blobs.
- **The API is for a reader who has never seen it.** Zero-value desc structs,
  no hidden state, errors as values, the three.js vocabulary where it fits.
- **Structure rules** (`docs/design.md` §2.1): a runtime module is an
  optional feature with an absent stand-in; a binding is written once for
  every host; cooked formats sit below feature crates; a tier is a
  capability set (§2.6 Tiers).
- **wgpu as published**, never vendored or patched.
- **The web engine runs in a Worker** with an OffscreenCanvas; the main
  thread owns input and the shell.
- **One design language.** `packages/design/tokens.json` generates og-ui's
  default style (`crates/og-ui-core/src/theme.rs`, drift-checked), which
  origin-agent pins by version. Brand surfaces use `packages/brand`. A game's
  style pack restyles the built-in components through tokens and parts.

## Building and testing

- Toolchain pins live only in `docs/ci.md` §Toolchain.
- **One target directory per checkout.** Never set a private
  `CARGO_TARGET_DIR`, copy the tree or delete a `target/`; use
  `cargo clean -p` or `scripts/clean-targets.sh`.
- **The gate is `scripts/check.sh`** (lanes in `docs/ci.md` §Lanes); there
  is no GitHub CI. `quick` is the loop and what a landing runs; `full` adds
  every test, fixtures and wasm builds. Use `--all` to force a run, never
  touch files or delete `target/og-gate/`. GPU steps take slots through
  `scripts/lock.sh` (source it, never copy it).
- **Tests never run `cargo`.** Rust tests use `env!("CARGO_BIN_EXE_og")`, bun
  tests `tests/support/og-bin.ts`.
- **A test is kept only when it fails on a plausible regression.** A missing
  GPU makes a test skipped or unchecked, never passed.
- **One GPU host.** References are recorded on the L4 (Vulkan, `gcp-l4`
  orb; `docs/ci.md` §The GPU host, §Reference frames). Metal, DX12 and WebGPU
  are compatibility checks. macrunner and studio-win are development hosts;
  their failures are never a landing gate, and Windows-only gate failures
  are not worth a stream's time.
- A moved reference never blocks a landing; a crash, a black or blank frame
  or an unchecked frame does. Never record a black or broken frame.

## Pitfalls

- Never kill a process you did not start (`pgrep -f` matches other
  checkouts' gates); never edit a shell script a running shell is reading.
- `local main` can be ahead of `origin/main`: compare full object ids.
- Look presets are `include_str!` into og-cook: rebuild `og` before capturing
  after a preset edit.
- A text change must not change other fonts' cooked output, charset or mode.
- Walkers and packagers never follow directory symlinks and skip `.og/`,
  `cooked/`, `og-out/` and installs.
- The platform development key is full-featured: when a flow fails with it,
  fix the credential contract, not the flow.

## Streams and landing

- The coordinator delegates streams with `create_thread`, each owning a
  feature area on one branch, overlapping ones in order. Default:
  `external-agent:claude-code` with `["model:claude-opus-5-5","effort:medium"]`,
  project `origingame/origin-engine`. A hard problem may go to
  `["model:claude-fable-5-1","effort:high"]`. A stopped thread is not
  resumed.
- A stream finishes its work whole and sends it as one stack of logical
  commits (a move and a logic edit never share one). Authors run only fast
  checks: `cargo check --workspace --all-targets`, Clippy, their crates'
  tests, and any identity check the change relies on.
- **Landing** goes through `scripts/land.sh`, a merge queue; never push main
  directly. A head touching packages/engine, the IDL or og-runtime's answers
  also runs `scripts/check.sh full --only fixtures` and `--only bun-test`.
  A head that changes rendering or a look goes to the L4 integrator, which
  looks at its frames for quality (a new feature's look needs the owner's
  approval), fixes any failure at the root itself, and lands it. Every
  other head is landed by its own thread.
- A file move is its own verbatim commit, reviewed with
  `git diff --color-moved`.
- Every change a game sees is one fragment in `changelog/unreleased/` in the
  same commit (`scripts/changelog.ts`); never edit
  `packages/engine/changelog.json` (`docs/release.md` §3). Docs-only commits
  go to main directly.
- Report landings and failures to Slack `#engine-dev` with the SHA range and
  gate result.
