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
- Public contracts with the platform; a change to one is coordinated with
  the platform and recorded as a breaking changelog fragment (`member`,
  `change`, `fix`), never kept compatible:
  - `packages/knowledge/scripts/mount.ts`: `installed`, `mount`, `summary`.
  - `packages/knowledge/scripts/lib.ts`: `payload`, `defaultRoots`,
    `PayloadFile`, `Roots`.
  - `packages/engine/src/provider.ts` (`@origingame/engine/provider`) and
    `PROVIDER_VERSION` (`docs/design.md` §2.8 Providers).
  - `packages/knowledge/scripts/brief.ts`: `artBrief`, `readLook`,
    `presetLook` (`docs/art.md` §2.2).
  - A version's `manifest.json` at `/downloads/engine/<version>/` and
    `latest/` (`docs/release.md` §1).
  - The engine package's `knowledge/looks/catalogue.json`,
    `knowledge/presets/`, `knowledge/scaffold/` (`origin-engine.json`,
    `distribution.json`, `template/`) and `knowledge/wiki/` paths
    (`docs/release.md` §2, `packages/knowledge/knowledge.json`).
  - Scaffold `og-project.json` (`version: 1`, concrete `engine`, `name`, optional
    preset or content-ref `look`) and `project.ts` (`defineProject`), plus the
    checked JSON from `og project --json`; the platform never evaluates TS.
  - `og test --json`, `og probe`, `og build --json` and where outputs land
    (`dist/` and `og-out/`) (testing guide §9); `og new --look --premise
    --no-prompt` (start guide §1); the launcher's install events
    (`docs/release.md` §1 The launcher's events).
- The engine ships no skills. What a game author needs beyond a signature is
  a guide in `crates/og-api/guides/` (read through `og api find` and
  `og api show`) or `og` command help. A guide never restates the API and
  names no platform tool or service. A guide may carry recipes: short
  examples that compose the API for one mechanism, each type-checked and
  run by the gate; a recipe is never a game.

## Principles

- **The engine supplies facts and tools; the agent decides.** No engine
  command scores or refuses on taste; thresholds belong in the guides. A
  shipped system carries out a rule the game wrote into its components and
  reports what it did; it never picks a value, ranks a play by taste or
  decides an outcome the game did not state.
- **No gameplay templates; mechanisms by name.** `og create` writes one
  project structure with a connected menu/navigation shell configured by
  project.ts's `create` choices (platform, input, languages and look/UI
  pack). Interactive films use the same shell over the engine's FlowScreen.
  The shell supplies no gameplay, levels, rules or balance. The engine
  ships mechanisms, never games: primitives report facts
  (a hit, an overlap, a candidate) and the small systems in `og.systems`
  carry out the rules their components state (a window, a cooldown, a
  stack limit). A game lists each system it uses by name, sets every number
  (a zero value does nothing) and may copy a system's source to change it.
  Nothing the engine ships holds a level, a rule set, a balance or a
  genre's composition.
- **Fix at the root; port the state of the art.** A problem is fixed where it
  arises, never worked around in a game, component or test. A missing feature
  starts from current papers and permissive implementations; keep their
  licences under `third_party/` or in the header.
- **One implementation, no compatibility.** The game API, components,
  defaults and save formats change whenever a cleaner design needs it; old
  games and old saves are not supported. A replacement deletes what it
  replaces in the same stack: no second path, legacy field, alias, shim,
  save or schema migration, or version fallback. A change a game sees is a
  breaking changelog fragment saying what to write instead.
- **Size and speed cuts never cost features.** Data only some games need
  is loaded or packed on demand; an optimisation never removes what a frame
  shows. A cost worth cutting goes behind a quality tier or a setting the
  game chooses.
- **A value whose doc records its reason changes with a new reason.** A
  constant or default whose comment cites a measurement, a paper or a fixed
  bug changes only with a new measurement or source, and its doc changes in
  the same commit.
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
  scene records and the wiki. Author configuration is `project.ts`, with ordinary
  `og-project.json` as the sole project marker and name/engine/look authority.
  Content TS evaluates to schema-checked IR and cooks into `.ogb`; it never
  serializes through OGT. Scene/prefab/scatter consume captured native TS data;
  Inspector/Tune changes memory only, while `og set` edits checked TS literals.
  Looks consume checked TS and native binary values, including embedded presets.
  The foundation's scanned Yarn catalogue remains until its domain cutover;
  remove its author parser with that cutover. OGT then remains internal text
  (IDL, dumps and probe transport), not another author path.
- **The API is for a reader who has never seen it.** Zero-value desc structs,
  no hidden state, errors as values, the three.js vocabulary where it fits.
- **Structure rules** (`docs/design.md` §2.1): a runtime module is built
  into every engine and split out of the web module, loaded on its first
  call (`wasm-modules.json`); a binding is written once for every host;
  cooked formats sit below feature crates; a tier is a capability set
  (§2.6 Tiers).
- **wgpu is our fork, wgpu-flash.** It is ours and built for speed
  (`https://github.com/originai-game/wgpu-flash`, pinned at a commit in
  `Cargo.toml`'s `[patch.crates-io]`, whose comment lists its patches). It
  is based on a wgpu release, and we submit nothing upstream. Its crates
  keep the names wgpu and naga, which `[patch.crates-io]` needs. wgpu is
  never vendored into this tree.
- **The web engine runs in a Worker** with an OffscreenCanvas; the main
  thread owns input and the shell.
- **One design language.** `packages/design/tokens/*.css` generates the
  default style pack (`crates/og-ui-next/default/pack.json`) and
  `packages/design/tokens.json`, both by og-ui-next and drift-checked
  (`scripts/gen-pack.sh`, `scripts/gen-tokens.sh`); origin-agent pins the
  design package by version. Brand surfaces use `packages/brand`. A game's
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
- **Tests are few and guard contracts.** A test exists only for one of: a
  public contract (the list above, the IDL and its drift); determinism
  (fixed tick, seeded randomness, snapshot and restore, replay); a crash or
  validation failure on a host (shader and pipeline validation, wasm and
  native parity); a repository rule check (drift, literals, changelog,
  structure); or one end-to-end path per public subsystem. Nothing else gets
  a test: not a private helper, a call order, a format, a default or
  constant, a near-duplicate case, or a look.
- **A fix brings no test by default.** Add one only when the bug broke one
  of the kinds above and can plausibly come back; otherwise the fix, its
  commit message and the evidence in the report are enough. "It fails
  without the fix" is not a reason to keep a test. Extend an existing test
  before writing a new one, and never append to a shared inline
  `mod tests`.
- **Quality is never a test.** Looks, layout and feel are judged by eye on
  captures; no pixel, number or snapshot test stands in for them, and green
  tests never mean the work is done.
- **Deleting a test is ordinary work.** A test outside the kinds above, or
  whose regression nobody can name, is deleted with the helpers and
  fixtures only it used. A cleanup only lowers the test count.
- **GPU results.** A missing
  GPU makes a test skipped or unchecked, never passed. Lavapipe is neither
  a standard nor a check: a GPU result from lavapipe is unchecked, and no
  frame, timing or landing is judged on it. SwiftShader (headless Chrome's
  CPU WebGPU) is a functional check only, that the web path runs; web
  frames and timings are judged on Chrome on the L4.
- **One GPU host: Colab.** GPU results and references come from a Colab
  NVIDIA GPU (L4 by default, Vulkan) through the workspace plugin
  `colab-gpu` (`docs/ci.md` §The GPU host, §Reference frames); the `gcp-l4`
  orb is retired. Metal, DX12 and WebGPU are compatibility checks against
  that set; Metal keeps no set of its own (a `references/metal/` frame
  exists only until Colab records it).
  macrunner and studio-win are development and packing hosts
  (`docs/release.md` §3); their failures are never a landing gate, and
  Windows-only gate failures are not worth a stream's time.
- A moved reference never blocks a landing: the engine's reference suites
  report it with its actual and diff PNGs (`docs/ci.md` §Landing); a crash,
  a black or blank frame or an unchecked frame fails. Never record a black
  or broken frame.

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
  feature area on one branch, overlapping ones in order. Every stream
  thread runs Amp's own mode with model `openai/gpt-6.1-sol` and the `fast`
  feature, project `origingame/origin-engine`; no other model. A stopped
  thread is not resumed.
- A stream finishes its work whole and sends it as one stack of logical
  commits (a move and a logic edit never share one). Authors run only fast
  checks: fmt, `cargo check --workspace --all-targets`, Clippy on the
  touched crates, those crates' tests, and `og api gen` then `og api check`
  last. Verification never runs for hours: no full gate, fixtures or L4 run
  for an ordinary head.
- **Landing** is fast: the coordinator lands a head on the author's checks
  plus engine typecheck and the changelog check, as a fast-forward of
  origin/main. The full gate, fixtures and `bun-test` run periodically on
  main, never as a per-head wait; a failure there gets a fix head.
  A head that changes rendering or a look has its frames captured on Colab
  and looked at for quality before it lands (a new feature's look needs
  the owner's approval). Every other head is landed by its own thread.
- A file move is its own verbatim commit, reviewed with
  `git diff --color-moved`.
- Every change a game sees is one fragment in `changelog/unreleased/` in the
  same commit (`scripts/changelog.ts`); never edit
  `packages/engine/changelog.json` (`docs/release.md` §3). Docs-only commits
  go to main directly.
- Report landings and failures to Slack `#engine-dev` with the SHA range and
  gate result.
