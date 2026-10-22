# Release: public packages first, the local loop, E2B last

Normative for how the engine leaves this repository and how the rest of
the work to launch is ordered. `docs/design.md` is the design, `docs/ci.md`
the gate, `docs/web-export.md` §Packaging the tarballs; this document is
the path from `main` to a version a game author, the platform's local
stack and an E2B image all install the same way, and the plan of what is
still to be developed and tested, in order, with what passes each step.
The platform's side of the same plan is `origingame/goals/origin-engine/
plan.md` §四 (decision E10).

## 1. The decision (owner, 2026-10-22)

**The engine is consumed as public npm packages pinned by version, developed
and tested locally against them, and only then installed into E2B.** The
E2B image is the last consumer of a version, never the first place a change
is tried.

What this replaces. Since the cutover the platform has consumed the engine
through a *layout*: `agent-server/scripts/e2b/origin-engine.ts` exports a
pinned git revision of this (private) repository with a read token, builds
it with the full Rust toolchain (release addon by `cargo zigbuild`, the
wasm twice, the lanes variant with `-Z build-std`, wasm-opt), lays the
result out as `/opt/origin-engine`, hashes it into the E2B template names
and uploads it (`build-templates.ts`, billed, the owner's step); the
platform's local stack (`engine-toolchain.ts`) builds the same layout from
the same revision to run at all; the runner reads the layout
(`ORIGIN_ENGINE_DIST`). Every engine change reaches a running thread only
through a pin move, a layout build, a template rebuild and a runtime-config
change, so a defect found there costs a full round and a day, and what an
E2B thread runs (`/opt/origin-engine/engine`) is not what an author outside
the platform installs (`bun add @origingame/engine`). AGENTS.md §Ownership
already said the platform consumes published packages pinned by version;
the layout was the interim that never became the rule. Now it does.

What holds from this commit:

1. **Four packages, one version.** `@origingame/engine` (with the
   agents' knowledge inside it as `knowledge/`, the owner's call of
   2026-10-22: one version, one install, 424 KB of text),
   `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64` and
   `@origingame/create-game` are published to the npm registry, public, MIT,
   at the same version, bumped together (§3). `packages/knowledge` stays a
   private source package of this repository.
2. **Three ways to run the engine, and no fourth.** (a) The registry:
   `bun add @origingame/engine@<v>`, `npm create @origingame/game@<v>`,
   which is what an author, the platform's local stack and an E2B image
   use. (b) Candidate tarballs from `scripts/release.sh pack`, installed
   with `bun add <tgz>`, for testing a version before it is published.
   (c) An engine checkout for engine developers only:
   `OG_ENGINE_PACKAGE=packages/engine` with the `og` cargo built. No
   platform script exports a git revision, runs `cargo`, or needs a token
   to this repository (`ORIGIN_ENGINE_READ_TOKEN` is withdrawn; see
   `docs/platform-asks.md` §14).
3. **The pin is a version string.** The platform's
   `agent-server/scripts/e2b/origin-engine.json` names a package version,
   not a SHA; `project.ogt`'s `engine "<v>"` pins the same thing per Game
   (decision E9); `og` compares the two and says so (§4 row R1).
4. **Local first, in this order, for every version.** The gate on `main`;
   `scripts/release.sh pack` and `verify` (the install test on this
   machine and in a linux-x64 container, §3); publish; the platform's
   local stack (`stage-engine`, `deploy`, the runner's `tests/og.rs`, the
   published page on a real Portal) against the published version; the
   M7 acceptance (one Discover prototype end to end by a thread) on the
   local stack; **then** the E2B image, which does `bun install` of the
   version and is checked once by `check-templates.ts`; then the owner
   enables it in the runtime configuration. A template rebuild happens
   only when the pinned version moves and only after the local stack
   passed at that version.
5. **Nothing of the sandbox is special.** The image carries bun, git, the
   installed packages with their `optionalDependencies` (the platform
   addon and `og`, tsrs for `og check`: `docs/platform-asks.md` ask 13),
   ffmpeg for the `probe` class, no renderer (design §5), no engine
   checkout, no Rust. `og doctor` in the image and on an author's machine
   checks the same things and prints the same lines.

Why. The author's loop and the platform's loop become the same loop;
every defect is found with `bun install` instead of a billed round; the
platform's test of a pin move is a version bump in one JSON file; the
knowledge, scaffold and archetypes an agent reads are the ones the
package of that version carries; the repository can be read-only to the
platform, and public when the owner decides (§6).

## 2. What the packages carry

| Package | Contents | Built by |
|---|---|---|
| `@origingame/engine` | the TypeScript API and runtime (`src/`, `types/*.d.ts`, `schema/api.schema.json`, `bindings.json`), the web hosts (`dist/web`, `dist/web-publish`, `dist/web-publish-2d`, each publish directory with its lanes variant), the player shell, the Basis transcoder, `bin/og.mjs` (runs the platform package's `og` with this package as `OG_ENGINE_PACKAGE`), and `knowledge/`: `skills/og-*` (ten), `wiki/` (generated from `api.ogt`), `looks/catalogue.json`, `scaffold/` (`origin-engine.json`, `distribution.json`, the template's text), `knowledge.json`, `scripts/mount.ts` and `validate.ts` (copied from `packages/knowledge` at pack); no native code | `packages/engine/scripts/pack.sh` (`OG_RELEASE=1 build.sh --linux-x64` first) |
| `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64` | `origin-engine.node` (napi addon, `crates/og-host-node`) and `og` (`crates/og-cli`), release profile, `os`/`cpu`/`libc` so only the matching one installs, as `optionalDependencies` of the main package | the same |
| `@origingame/create-game` | `bin/create.ts` (`npm create @origingame/game` runs it: npm and bun map `create @scope/name` to `@scope/create-name`, which is why the package is `create-game`, not `create`), `template/` (`templates/game`), `archetypes/` (the eight), `looks/` (the catalogue) | `packages/create` `prepack` |

Not shipped: Windows, linux-arm64 (design §12 item 5; `og.mjs` says so
when the platform package is missing). `@tsrs/*` are someone else's
packages, pinned at `TSRS_VERSION` of `scripts/install-tools.sh`, not at
the engine's version.

## 3. Versions and the release procedure

**Version.** `0.1.0` is the first published version. Before `1.0.0` a
minor bump carries API additions or changes (recorded in `CHANGELOG.md`,
one line per IDL member added, changed or removed, written at the merge
that changes `api.ogt`), a patch bump carries fixes and frames. Every
occurrence of the version is written by `scripts/release.sh bump` and
held equal by `packages/knowledge`'s test: the three `package.json`
(`optionalDependencies` included), `scaffold/origin-engine.json`,
`templates/*/project.ogt` and `package.json`, `templates/archetypes/*`,
`examples/*`, and what `create` writes into a new project. A Game's
`project.ogt` pins its own version; `og` reports a mismatch with the
installed package (`og doctor` as a line, `og run`/`test`/`build` as a
warning: the installed package runs, the pin is the author's record of
what the game was written against, and an upgrade is an explicit edit).

**`scripts/release.sh`** (to be written, §4 row R3), four verbs, each one
step of the path, all local:

```sh
scripts/release.sh bump <version>     # every occurrence of the version, CHANGELOG.md's heading
scripts/release.sh pack               # OG_RELEASE=1 pack.sh --linux-x64 (the engine with knowledge/, the platform packages), then create-game: dist/release/<version>/*.tgz + manifest.json (sizes, sha256)
scripts/release.sh verify [--registry] # the install test (below) from the tarballs, or from the registry after publishing
scripts/release.sh publish            # npm publish, platform packages first, then engine, create-game; git tag v<version>; prints the install line
```

`pack` holds the gate lock (a release build compiles beside nothing) and
refuses a `native/` that is not release; `publish` refuses when `verify`
has not passed on these tarballs (`manifest.json` records it), when the
working tree is dirty, or when `main` is not at the commit the tarballs
were built from. It needs the owner's `npm login` (the `@origingame`
scope; 2FA) once per machine. Publishing is an outward action: it runs
only by hand.

**The install test** (`tests/install/`, bun; §4 row R4) is what `verify`
runs, on this machine (darwin-arm64) and in a `node:22-bookworm-slim`
container with bun (linux-x64, the executor's platform), from the tarballs
or the registry:

1. an empty directory; `bun add` the engine (and, from tarballs, the
   platform package and the create-game tarball) at the version;
2. `bunx create-game demo --name Demo --no-prompt`, then `bun install` in
   it, with no `OG_ENGINE_PACKAGE`, `OG_NATIVE_ADDON` or `ORIGIN_OG_BIN`
   in the environment;
3. `bunx og --version` prints the version; `og doctor` passes (bun, the
   package, the addon, both wasm files, tsrs beside the package; the
   renderer line is `none` in the container and not a failure);
4. `og check`, `og test` (the template's two playtests; frames unchecked
   in the container, checked on this machine), `og build --publish`
   (`og-build.json` names the version, the publish wasm's sha256 equals
   the tarball's);
5. `--archetype runner` once more through steps 2 to 4;
6. the knowledge inside the installed engine (`knowledge/validate.ts`), and
   `knowledge/scripts/mount.ts --package <the engine package> --check`
   against a stand-in platform tree.

It is also a stage of the wave's end (`scripts/wave-end.sh release`:
`pack` and `verify`, the tarballs it leaves are the ones `publish`
ships), so the gate never compiles release (`docs/ci.md` §One profile per
lane) and a wave ends with a candidate ready.

## 4. The plan: what remains, in order

Everything still to develop and test before the engine is live, as the
platform's plan §四 and `docs/phase-1.md`'s owed table count it on
2026-10-22. Launch means: the platform's `engine` class runs threads on a
published version in production, new Games are created on it
(`engine_scaffold` from `@origingame/create-game`'s archetypes), and one
Discover prototype has been made end to end by a thread, played through
`og run --commands`, delivered and published. The stages run in this
order; inside a stage the rows run in parallel where their lanes allow
(AGENTS.md §Build and gate discipline). Each row's evidence is filed under
`tests/evidence/release/` (engine) or `origingame/goals/origin-engine/
evidence/` (platform).

### R. The release pipeline (engine repository; first)

| # | Row | Passes when |
|---|---|---|
| R1 | **Done 2026-10-22** (`scripts/release-bump.ts`, `og` pin notice and doctor line). One version everywhere: `release.sh bump`; `create` writes it into `project.ogt` and `package.json`; the knowledge test holds every occurrence equal (`validate.test.ts` already names `origin-engine.json` and the template's `project.ogt`); `og` compares `project.ogt`'s `engine` with the installed package's version (`doctor` line, run/test/build warning) | `bump 0.1.0` leaves `git grep 0.0.1` empty outside history; `og doctor` in a project pinned at another version prints the mismatch and its fix |
| R2 | **Done 2026-10-22**, then folded (the owner, the same day): the knowledge ships inside `@origingame/engine` as `knowledge/` (pack.sh copies `packages/knowledge`'s files there); `mount.ts --package <installed engine dir>` mounts it; `packages/knowledge` stays private | `bun pm pack` of the package installs in an empty directory and `bunx` its `validate.ts` passes; `mount.ts --package` on a stand-in platform tree gives the same bytes as `--commit` of the same source |
| R3 | **Done 2026-10-22.** `scripts/release.sh` (`bump`, `pack`, `verify`, `publish`) and the wave-end `release` stage; `pack.sh` packs create and knowledge too; `manifest.json` with sizes and sha256; `CHANGELOG.md` with 0.1.0's entry (the IDL as it stands) | `wave-end.sh release` leaves `dist/release/0.1.0/` with four tarballs and a manifest; `publish` refuses a dirty tree, an unverified manifest, a wrong `main` |
| R4 | **Written 2026-10-22** (`tests/install/run.ts`, `tests/install/Dockerfile`); its first pass is recorded in R8's evidence. The install test (`tests/install/`, §3) on this machine and in the linux-x64 container, from tarballs and `--registry` | both runs pass on the 0.1.0 candidate; the container run shows `renderer none` and `N frame(s) unchecked`, never a failure |
| R5 | **Done 2026-10-22** (the `pin` line; the rest was there). `og doctor` and `og --version` outside a checkout: the package's version, the addon's, the wasm files, tsrs, the renderer, bun; inside a checkout the toolchain lines as today | the doctor's output in the container and on this machine differ only in the renderer line |
| R6 | The template's two models (`trickster.glb`, `dropship.glb`, copied from `origingame-examples`): `dropship.glb` is MIT (the `core-business` example's `LICENSE`, named in `templates/game/LICENSES.md` now); `trickster.glb` came from `edg-king-match`, which declares no licence, so the template's `trickster` entity is an instance of the CC0 `walker.glb` since 2026-10-22 and the file left the template. `@origingame/create-game` ships nothing whose licence is not named. The engine's test copy (`crates/og-tests/assets/`, `templates/fx-showcase`) is not in any package; whether it stays in a public repository is the owner's (§6) | done 2026-10-22 for the packages: `LICENSES.md` of the template names a licence for every binary; `distribution.json` regenerated; the owner's word still owed for the repository's copies |
| R7 | **Done 2026-10-22.** Package metadata: `repository.url` names the repository the packages are built from (today `origingame/origin-engine`, which is not where the code is); `homepage`, `bugs`; the README's install section (`bun add`, `npm create`, the three commands) | `npm pack --dry-run` lists what §2 says and nothing else; the README's lines run as written |
| R8 | **Publish 0.1.0** (owner: `npm login`, the `@origingame` scope); `release.sh verify --registry`; tag `v0.1.0` | `npm view @origingame/engine version` answers `0.1.0`; `npm create @origingame/game` in an empty directory on a machine without this checkout runs `og test`; evidence `tests/evidence/release/0.1.0.md` |

### L. The local loop on the platform (platform repository; after R8)

| # | Row | Passes when |
|---|---|---|
| L1 | `origin-engine.json` pins `{ "package": "@origingame/engine", "version": "0.1.0" }`; `engine-toolchain.ts` runs `bun install` of the engine and create-game at that version into `ORIGIN_ENGINE_CACHE/<version>/` (`--from <dir>` takes candidate tarballs) and lays `ORIGIN_ENGINE_DIST` out of `node_modules` (`bin/og` → the platform package's `og`, `engine/` → the package, `archetypes/` → create's, `overlay/` → the mounted scaffold); no checkout, no `git archive`, no cargo, no token | `bun scripts/dev/workspace-stack/engine-toolchain.ts` on a machine with no engine checkout and no Rust produces a layout `readEngineLayout` accepts |
| L2 | The knowledge mount from the package: `node_modules/@origingame/engine/knowledge/scripts/mount.ts <checkout> --package node_modules/@origingame/engine` writes `packages/game-knowledge/production`; `--check` in the platform's tests | `--check` is clean at the pinned version; game-knowledge's bundle tests pass |
| L3 | The local stack at the version: `stage-engine`, `deploy`, `engine-page.ts` on a real Portal (desktop and phone, plain and isolated releases), `agent-runner` `tests/og.rs`; this is the merge gate of every pin move | all four pass; `origingame/goals/origin-engine/evidence/pin-0.1.0.md` |
| L4 | `origin-engine.ts` (the layout builder) and `build-templates.ts`'s engine step reduced to the install of L1; `ORIGIN_ENGINE_READ_TOKEN` removed from the workflows and the plan's §四 4 | `rg cargo agent-server/scripts/e2b` finds nothing; the workflows check out no engine repository |
| L5 | **M7 acceptance, locally**: a thread on the local stack makes one Discover prototype end to end (`engine_scaffold` → `og run --commands` playtest → delivery → publish to the local Portal, played back byte for byte) | the receipt in `evidence/m7-local-0.1.x.json`; the published page passes `engine-page.ts` |

### F. The features still owed (engine repository; beside R and L)

The owner's rule of 2026-10-22 (`docs/phase-1.md`, "Features before
optimisation") stands: feature streams and dogfood rounds only, the
optimisation rows (§P) after them.

| # | Row | Passes when |
|---|---|---|
| F1 | The three streams stopped on 2026-10-22 land: `kit-words` (the kit HUDs' words, Meridian Loop on the new fields), `platformer-2d` (Brasswick's runtime findings), `dogfood-7` (the local-multiplayer party game) | each merged through the functional lane; Meridian Loop's `ghost`, `lap`, `rerun`, `locales`, `frames` playtests green on `main` |
| F2 | Owed feature rows: 58 (og-ui's cooked-font text frames in the browser tiers), 63's remainder (`.vox` `_loop` and the playback range), 59's remainder (the emacs kill buffer, visual caret movement across a bidi edge) | each row closed in the owed table with its commit |
| F3 | Design §11 phase 2 and 3 checked item by item against the waves that landed them (Tune is the probe's `set` and bounds over the reflected schema, read by the platform's Stage; splats optional and not planned); what is missing becomes a row here | the table in `docs/phase-1.md` names, for each §11 item, the round that landed it or this row's number |
| F4 | Browser frames for Brasswick and Meridian Loop on both tiers; the published-page harness on the new games | `web-reference` passes with them; `og run --device chrome` plans for both |
| F5 | wgpu and naga 31 when published (row 35), kept only if every reference frame passes unchanged | a wave end at the new version with every frame unmoved, or the row stays open with the date |
| F6 | Each wave's end: `scripts/wave-end.sh` (`gate`, `web`, `conformance`, `release`), summary in `docs/phase-1.md` | the wave's section records the four stages |

### T. The test matrix (what runs where, when; standing)

| Layer | What | Where | When |
|---|---|---|---|
| Crates | fmt, clippy, unit and logic tests, gfx references on `metal`, drift of the generated `.d.ts`, wiki and skills | `scripts/check.sh quick` / `functional` | every merge to `main` (the coordinator; `docs/ci.md`) |
| Packages | `packages/engine` bun tests on the dev addon, typecheck through tsrs, create's and knowledge's tests | `functional` (bun-functional) | every merge |
| Templates, archetypes, examples | `og test`, `og polish`, their reference frames | `functional` (bun-test) | every merge |
| Wasm | build, wasm-opt, size (a number to read), determinism across hosts | `check.sh --all` | every wave end |
| Browser | both tiers' reference frames, the kit, the a11y mirror, the published page under a script | `check.sh web-reference`, `og run --device chrome` | every wave end; by hand for a rendering change |
| Release | pack, the install test from tarballs on darwin-arm64 and linux-x64 | `wave-end.sh release` / `release.sh verify` | every wave end; before every publish |
| Registry | the install test from npm | `release.sh verify --registry` | after every publish |
| Platform, local | `stage-engine`, `deploy`, `engine-page.ts`, runner `tests/og.rs`, the knowledge mount `--check` | the platform's local stack | every pin move (L3) |
| Devices | iPhone first frame and frame times (Safari, WebGPU), a mid-range Android's 10,000-entity tick (design §7; owed row 40) | `og run --device iphone`, `--device android` on the owner's devices | once before launch, then per minor version |
| E2B | `check-templates.ts` on the image that installs the version | E2B (billed) | once per pin move, after L3 |
| Production | `verify-production.sh`, `state.ts` without drift, the Player telemetry of the first releases | the platform's release process | at launch and each ship |

### E. E2B and production (platform and owner; last)

| # | Row | Passes when |
|---|---|---|
| E1 | The `engine`, `probe` and `asset` images install the pinned version (`bun install` of the engine and create-game with `optionalDependencies`, in the image, no registry at run time), carry no renderer, no layout, no checkout; `/opt/origin-engine` is the installed project directory; the `engine` image ≤ 600 MB | `check-templates.ts` passes on the image built from the version; the size in its receipt |
| E2 | The owner enables the four templates in the runtime configuration and ships agent-server (`release.sh ship agent-server --development`; the prerequisites of the plan's §四 4 minus the token) | `bun deploy/ovh/state.ts` shows no drift but HELD; a production thread on the `engine` class |
| E3 | The templates built on 2026-10-20 from the `d51c8ae` layout are not enabled; they are superseded by E1 | the runtime configuration never names them |
| E4 | Production: the game-host ingress for `/__player/v2/` (applied 2026-10-20), `verify-production.sh`, the first engine release's Player telemetry read | the receipt in the platform's evidence |

### P. Deferred until the features are complete (engine; after F)

Owed rows 14 and 44 (the publish wasm under 65 %, the budget's meaning
the owner's), 51 and 52 (the three.js comparison and the lanes physics in
Chrome), 56 (quantised clips resident), 57 (the cull's lane grain), 88
(a terrain streaming test whose tiles arrive late), 89 and 90 (perf-games'
idle-machine tables and follow-ups; branch `perf-2`). Each is taken up as
the product needs it, measured at a wave's end, and never holds a merge
or a release.

## 5. The author's and the agent's loop, after this

```sh
npm create @origingame/game my-game -- --premise "a lamplit clockwork platformer"   # or --archetype runner
cd my-game && bun install          # @origingame/engine@<v>, this platform's og and addon, tsrs
bunx og doctor                     # the package, the addon, the wasm, tsrs, the renderer
bunx og dev .                      # watch, rebuild, re-cook, hot reload through the probe
bunx og test . && bunx og polish . # the playtests; taste and feel scores
bunx og build . --publish          # the play page: dist/ with og-build.json naming the version
```

Inside the platform a thread does the same through the runner's `og.rs`
adapter against the installed package of the pinned version, with the
knowledge of that version mounted; the Wizard publishes `dist/` as
`engine=origin-engine`. An engine developer alone uses the checkout
(`OG_ENGINE_PACKAGE=packages/engine`, `target/debug/og`), as every test in
this repository does.

## 6. Open with the owner

1. ~~The npm account and the `@origingame` scope~~ (2026-10-22: `doufunao123`, owner of the `origingame` organisation, logged in; npm asks a one-time password per publish, answered by the person).
2. Whether the repository becomes public (MIT since 2026-10-17). A public
   package built from a private repository is allowed and the packages'
   `repository.url` must then be honest (R7); a public repository restores
   CI on GitHub (owed row 83).
3. The two models of the template (R6): leave to redistribute, or
   replaced.
4. The wasm budget's meaning (row 44), `topdown` in Discover, third-party
   Godot Web exports, the WebGL2 fallback's lifetime, a third native tier
   (rows 39): unchanged from the platform's plan §四.
5. The devices for the T matrix's device row (row 40).
