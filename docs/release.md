# Release: public packages first, the local loop, the development sandbox on InstaBox, E2B last

Normative for how the engine leaves this repository and how the rest of
the work to launch is ordered. `docs/design.md` is the design, `docs/ci.md`
the gate, `docs/web-export.md` §Packaging the tarballs; this document is
the path from `main` to a version a game author, the platform's local
stack, the development sandbox and an E2B image all install the same way,
and the plan of what is still to be developed and tested, in order, with
what passes each step. The platform's side of the same plan is
`origingame/goals/origin-engine/plan.md` §四 (decision E10).

## 1. The decision (owner, 2026-10-22)

**The engine is consumed as public npm packages pinned by version, developed
and tested locally against them, then in the development sandbox, and only
then installed into E2B.** The E2B image is the last consumer of a version,
never the first place a change is tried.

**The development stage's sandbox is InstaBox, not E2B** (the owner, the
same day, made explicit here on the owner's instruction). The threads of
the development stage (the build-out, the dogfood rounds, the M7
acceptance, every pin move before launch) run in InstaBox machines
(https://dashboard.instabox.sh; a full Linux VM per thread, 4 vCPU and
8 GiB, a persistent `/data`, pause and resume, snapshots as images, an
HTTPS URL per port; the SDK `@insforge/instabox-sdk`, the API at
`api.instabox.sh` with `INSTABOX_API_KEY`). E2B stays production's
provider and the last consumer of a version (stage E); nothing of the
engine or of the development loop depends on it. What this changes on the
engine's side is small and all here: the install test runs inside an
InstaBox machine too (`tests/install/instabox.ts`, `release.sh verify
--instabox`, §3), it is the engine's check that a version runs on the
machine class the development threads use, and every script that reached
an E2B sandbox for development (`packages/engine/scripts/e2b-addon.ts`)
is gone; the two that remain (`tools/bench/e2b-run.ts`,
`crates/og-tests/scripts/capture-vulkan-cpu.ts`) are the lavapipe
measurements set aside with the CPU sandbox requirement. The platform's
side (the `instabox` provider beside `e2b`, the per-class snapshots that
install the pinned version, the development routing) is stage D and
`docs/platform-asks.md` ask 15. The account today is InstaBox's free plan
(one machine, from the `agent-browser` template or a snapshot; verified
2026-10-22, `tests/evidence/release/instabox-0.1.1.md`); more than one
development thread at a time needs the Pro plan (§6).

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
   machine, in a linux-x64 container and, `--instabox`, in an InstaBox
   machine, §3); publish; the platform's local stack (`stage-engine`,
   `deploy`, the runner's `tests/og.rs`, the published page on a real
   Portal) against the published version; the platform's InstaBox
   snapshots of the version, checked once (stage D), on which the M7
   acceptance (one Discover prototype end to end by a thread) runs;
   **then** the E2B image, which does `bun install` of the version and is
   checked once by `check-templates.ts`; then the owner enables it in the
   runtime configuration. A snapshot or template rebuild happens only
   when the pinned version moves and only after the local stack passed
   at that version.
5. **Nothing of the sandbox is special, whichever provider.** The
   snapshot or image carries bun, git, the installed packages with their
   `optionalDependencies` (the platform addon and `og`, tsrs for `og
   check`: `docs/platform-asks.md` ask 13), ffmpeg for the `probe` class,
   no renderer (design §5), no engine checkout, no Rust. `og doctor` in
   an InstaBox machine, in the E2B image and on an author's machine
   checks the same things and prints the same lines.
6. **Development on InstaBox, production on E2B.** A thread of the
   development stage runs in an InstaBox machine made from the
   platform's snapshot of the pinned version; a production thread runs
   in the E2B image of the same version. The engine never tells them
   apart: what differs is the provider the platform routes a class to.

Why. The author's loop and the platform's loop become the same loop;
every defect is found with `bun install` instead of a billed round; the
platform's test of a pin move is a version bump in one JSON file; the
knowledge, scaffold and archetypes an agent reads are the ones the
package of that version carries; the repository can be read-only to the
platform, and public when the owner decides (§6).

## 2. What the packages carry

| Package | Contents | Built by |
|---|---|---|
| `@origingame/engine` | the TypeScript API and runtime (`src/`, `types/*.d.ts`, `schema/api.schema.json`, `bindings.json`), the web hosts (`dist/web`, `dist/web-publish`, `dist/web-publish-2d`, each publish directory with its lanes variant), the player shell, the Basis transcoder, `bin/og.mjs` (runs the platform package's `og` with this package as `OG_ENGINE_PACKAGE`), and `knowledge/`: `skills/og-*` (ten), `wiki/` (generated from `api.ogt`), `looks/catalogue.json`, `scaffold/` (`origin-engine.json`, `distribution.json`, the template's text), `knowledge.json`, `scripts/mount.ts` and `validate.ts` (copied from `packages/knowledge` at pack); no native code | `packages/engine/scripts/pack.sh` (`OG_RELEASE=1 build.sh --linux-x64` first): the release workflow's `web` job (§3), or a machine's `scripts/release.sh pack` |
| `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64` | `origin-engine.node` (napi addon, `crates/og-host-node`) and `og` (`crates/og-cli`), release profile, `os`/`cpu`/`libc` so only the matching one installs, as `optionalDependencies` of the main package | linux-x64: the same (cargo zigbuild, glibc 2.31); darwin-arm64: the workflow's `macos-14` job, `pack.sh --native` |
| `@origingame/engine-win32-x64` | `origin-engine.node` (MSVC napi addon) and `og.exe`, release profile, `os: win32`, `cpu: x64`; the same API, DX12 renderer, tsgo checker fallback | the workflow's `windows-latest` job, `pack.sh --native` in Git Bash (0.3.0 to 0.6.0: by hand on the `studio-win` runner) |
| `@origingame/create-game` | `bin/create.ts` (`npm create @origingame/game` runs it: npm and bun map `create @scope/name` to `@scope/create-name`, which is why the package is `create-game`, not `create`), `template/` (`templates/game`, the 3D template), `template-2d/` (`templates/game-2d`, the 2D starter `--profile 2d` writes), `archetypes/` (the eight), `looks/` (the catalogue) | `packages/create` `prepack` |

Not shipped yet: linux-arm64 (design §12 item 5); `og.mjs` says so when the
platform package is missing. `@tsrs/*` are someone else's
packages, pinned at `TSRS_VERSION` of `scripts/install-tools.sh`, not at
the engine's version.

## 3. Versions and the release procedure

**0.7.2 released (2026-10-29 UTC).** All five npm packages are
published, and `v0.7.2` is pushed at `ccda9a97`. It is recorded as a
patch with the simulation unchanged: additive, off-by-default members
(`MaterialStyle.shadowDarkness`, `surface` and `MaterialSurface`), the
cook's texel-space painting and `conform character`, and the
heavy-ink-shadows, low-resolution-3d and surface-NaN fixes. The replay
gate passes 2/2 goldens with the tarball binaries on darwin-arm64 and
win32-x64. The tarballs passed on Mac, Linux and Windows, and so did the
registry install. See the
[release receipt](../tests/evidence/release/0.7.2.md).

**0.7.1 released (2026-10-28 UTC).** All five npm packages are
published, and `v0.7.1` is pushed at `57263610`. It is a patch with the
simulation unchanged and no IDL member changed: the dispersed Bayer
matrix for `bayer4`/`bayer8`, rubber-hose-animation and gothic-contour
without posterize, the mesh conformance's sliver faces and voxel normals,
and `set_shader_param` in place. The replay gate passes 2/2 goldens with
the tarball binaries on darwin-arm64 and win32-x64. The tarballs passed on
Mac, Linux and Windows, and so did the registry install. See the
[release receipt](../tests/evidence/release/0.7.1.md).

**0.7.0 released (2026-10-28 UTC).** All five npm packages are
published, and `v0.7.0` is pushed at `7757383a`. It is a minor release
with the simulation unchanged: the art wave (docs/art.md, A1–A7). It
adds the art contract's seven look resources, the cook's 2D and 3D
conformance, the runtime defaults, the art brief, `og style check` and the
family sample scenes. It has one breaking entry: a look that sets the art
blocks now conforms the assets. The replay gate passes 2/2 goldens with
the tarball binaries on darwin-arm64 and win32-x64. The tarballs passed on
Mac, Linux and Windows, and so did the registry install. See the
[release receipt](../tests/evidence/release/0.7.0.md).

**0.6.6 released (2026-10-28 UTC).** All five npm packages are
published, and `v0.6.6` is pushed at `71b2f032`. It is a knowledge-only
patch with the simulation unchanged and no crate changed: the craft
skills architecture-discipline, component-forge, level-texturing,
localization, recipe-craft and visual-acceptance move from origin-agent
into the knowledge package under the same names, and `payload` and
`defaultRoots` of `knowledge/scripts/lib.ts` join `installed`, `mount`
and `summary` as public contract, for origin-agent reading the knowledge
from the installed package. The replay gate passes 3/3 platforms × 2/2
goldens with the tarball binaries, and the tarballs and the registry
install passed on Mac, Linux and Windows. See the
[release receipt](../tests/evidence/release/0.6.6.md).

**0.6.5 released (2026-10-27 UTC).** All five npm packages are
published, and `v0.6.5` is pushed at `76859981`. It is a patch with the
simulation unchanged: og check names a truncated bunx TypeScript install
and its fix; the knowledge mount targets origin-agent's `knowledge/`;
og-ui glass and a view's glow chain match metal on Windows WebGL2; and
WebGL2's cold start on Windows drops from about 30 s to 7 s (background
links, and the GL tier's lit programs without rect-light code until a
scene has one). The replay gate passes 3/3 platforms × 2/2 goldens, and the
tarballs and the registry install passed on Mac, Linux and Windows. 0.6.4,
published from an orb only, was verified from the registry on Mac and
Windows afterwards. See the [release receipt](../tests/evidence/release/0.6.5.md).

**0.6.4 released (2026-10-27 UTC).** All five npm packages are
published, and `v0.6.4` is pushed at `397dbc79`. It is a patch with the
simulation unchanged and no IDL member changed: it adds `og bake
projection`, a gray level painted from views on the CPU, with xatlas
charting for levels without uvs. The replay gate passes 2/2 goldens with
both the dev and the release builds on linux-x64. The tarballs and the
registry install passed on the orb's linux-x64, on the host and in the
container; the Mac and Windows checks were not required for this patch.
See the [release receipt](../tests/evidence/release/0.6.4.md).

**0.6.3 released (2026-10-27 UTC).** All five npm packages are
published, and `v0.6.3` is pushed at `b73124bd`. It is a patch with the
simulation changed: it fixes the cross-platform yaw drift (§4 F) with a
yaw step through og-core, adds `og_core::math::quat_slerp` and `acos`, a
clippy `disallowed-methods` rule against the host's libm in the simulation
crates, and a bit-pinned test of the camera's projection. The goldens are
re-recorded, and the replay gate passes 3/3 platforms × 2/2 goldens with
both the dev and the release builds. The tarballs and the registry install
passed on Mac, Linux and Windows. See the
[release receipt](../tests/evidence/release/0.6.3.md).

**0.6.2 released (2026-10-27 UTC).** All five npm packages are
published, and `v0.6.2` is pushed at `49566391`. It is a patch that adds
`changelog.json`, `og upgrade`, `og doctor --json`, `og replay` and the
golden-replay patch gate. The tarballs and the registry install passed on
Mac, Linux and Windows. The golden replays pass on linux and darwin; on
win32, game-3d drifts at ticks 62–65, which is owed (§4 F) and fixed in
0.6.3. See the [release receipt](../tests/evidence/release/0.6.2.md).

**0.6.1 released (2026-10-27 UTC).** All five npm packages are
published, and `v0.6.1` is pushed at `418d82f8`. It is a patch release: the
five RedGPU ports are off by default, and it carries the Windows fixes
(WebGL2 on ANGLE/D3D11, the debug og-gfx stack). Both real-GPU lanes
(metal; DX12, Chrome and Edge) passed on its content. The tarballs from
`release.yml` passed verify on Mac, Linux and Windows, and so did the
registry install on all three. Shipping 3D lanes wasm is 3,260,792 B Brotli
q11 (77.74 % of 4 MiB). The owed WebGL2-on-Windows follow-ups are in the
[release receipt](../tests/evidence/release/0.6.1.md).

**0.6.0 released (2026-10-26 UTC).** All five npm packages are published;
`v0.6.0` is pushed at `34b492d0`. Metal functional and corrected Rumpus
frames passed; the exact tarballs passed Mac/Linux/Windows consumer checks,
and registry verification passed on all three. Shipping 3D lanes wasm
is 3,252,580 B Brotli q11 (77.55% of 4 MiB). The rejected water-AO candidate,
accepted pre-existing frame differences, hashes and checks are recorded in
the [release receipt](../tests/evidence/release/0.6.0.md).

**0.5.0 released (2026-10-25 UTC).** All five npm packages are published;
`v0.5.0` is pushed at `ebe7f08b`. Metal functional and Rumpus passed,
the exact tarballs passed Mac/Linux/Windows consumer checks, and registry
verification passed on Mac, Linux and Windows. Shipping 3D lanes wasm is
3,223,281 B Brotli q11 (76.85% of 4 MiB). Hashes, commands, timings and
remaining device/sandbox checks: [release receipt](../tests/evidence/release/0.5.0.md).

**Version.** `0.1.0` is the first published version. Before `1.0.0` the
minor number counts breaks and the patch number everything else (the
owner, 2026-10-27: 0.1.0 to 0.6.0 in five days took a minor for every
added field). A minor bump (`0.x.0`) is a release with a break: an IDL
member removed or renamed, a field's type or default changed, a saved
value or enum index whose meaning changed, or behaviour a game relied on
changed, so a game may need edits to upgrade and `CHANGELOG.md` says
which. A patch bump (`0.x.y`) is everything compatible: new members off
by default (a game that does not set them draws and runs exactly as
before, which the reference frames hold), variants appended to an enum,
fixes, frames and speed. Either way `CHANGELOG.md` has one line per IDL
member added, changed or removed, written at the merge that changes
`api.ogt`; a release that is compatible but large may still take a minor
when the owner names it. Every
occurrence of the version is written by `scripts/release.sh bump` and
held equal by `packages/knowledge`'s test: the three `package.json`
(`optionalDependencies` included), `scaffold/origin-engine.json`,
`templates/*/project.ogt` and `package.json`, `templates/archetypes/*`,
`examples/*`, and what `create` writes into a new project. A Game's
`project.ogt` pins its own version; `og` reports a mismatch with the
installed package (`og doctor` as a line, `og run`/`test`/`build` as a
warning: the installed package runs, the pin is the author's record of
what the game was written against, and an upgrade is an explicit edit).
A pin is **compatible** with the installed package when both are the same
minor and the installed one is not older: that is no mismatch (`og
doctor` says ok, the others say nothing), since a patch only adds and
fixes. `og doctor --json` reports it as `pin: {pinned, installed,
compatible}` (null outside a project).

**`changelog.json`** (in the engine package, schema 1) is the machine
reading of `CHANGELOG.md` the platform builds its "new engine version"
card from: `{schema, about, unreleased, versions: [{version, date, kind:
"patch"|"minor", simulation: "unchanged"|"changed", breaking: [{member,
change, fix, find?: [identifier], codemod?: {from, to}}], added: [{member,
summary}], fixed: [string]}]}`, newest first. Each merge that adds a
`CHANGELOG.md` line adds its entry under `unreleased`; `release.sh bump`
moves `unreleased` into `versions` as the new version, dated, its `kind`
from the numbers, and refuses a patch whose `unreleased` has a breaking
entry (release a minor). `simulation` says whether a game's ticks may
differ from the version before (the replay gate below holds a patch's
`unchanged`); the versions before 0.6.1 were never measured and say
`changed`.

**`og upgrade [project] [--to <v>] [--dry-run] [--json]`** moves a game
to the installed engine (or `--to`, a version in its changelog): it
rewrites `project.ogt`'s `engine` and `package.json`'s
`@origingame/engine`, applies the `codemod` of every breaking entry of
the versions crossed to `src/` and `assets/` text, and lists every line
where another breaking entry's `find` identifiers appear (`manual`, with
the change and its fix) or the entries with no identifiers (`review`).
It never downgrades and does not install (`bun install` does, then `og
check` and `og test`). Exit 0 done, 3 done with hand edits listed, 1
failed with nothing written, 2 usage. The JSON is `{status:
"done"|"manual"|"failed", project, from, to, installed, kind:
"none"|"patch"|"minor", simulation, versions, dryRun, edited, codemods:
[{version, member, file, from, to, count}], manual: [{version, member,
change, fix, file, line, text}], review: [{version, member, change,
fix}], reason?}`.

**`og replay`** is the evidence that a version runs a game the same:
`og replay record [project] --commands "…" | --commands-file <f>
[--ticks N] [--seed S] [--clock <UTC ms>] [--locale <tag>]
[--keyframe-every K] --out <trace.json> [--json]` plays the script
headless and writes the trace `{schema: 1, kind: "og-replay", engine,
project: {id, entry, bundleSha256}, seed, clock, locale, commands, ticks,
startTick, table, hashes: [016x per tick], keyframes: [{tick, save}]}`
(the hash is over the world's components and resources in `table`, the
types and fields the recording saw, sorted, so a field a later version
adds does not move it; a keyframe is an `og-save:1:` text every K ticks,
default 30). `og replay compare <trace.json> [project] [--json]` replays
it with the og at hand and prints `{same, recorded: {engine, ticks},
replayed: {engine, ticks}, sourceChanged, ticksCompared,
firstDivergentTick, keyframe: {tick, diff} | null, divergentKeyframes}`,
the diff (the first keyframe at or after the divergence) naming the
entities, components, resources and fields with the recorded and
replayed values; exit 0 the same, 3 diverged, 1 failed, 2 usage.
`sourceChanged` says the game's bundle is not the one recorded (the
engine path and path comments left out of its hash), so a divergence is
then the game's as much as the engine's.

**The patch gate.** `tests/replay/golden/` holds the golden traces
(`goldens.json`: `templates/game` and `templates/game-2d`, 300 ticks
each), and `scripts/replay-gate.ts` compares each with the og in
`packages/engine/native` (or `$OG`). `release.sh bump` runs it first: a
golden that diverges fails the bump unless `changelog.json`'s
`unreleased.simulation` is `"changed"`, and a golden whose template
changed fails as "re-record" (`--update`). A change that moves the
simulation on purpose sets `simulation: "changed"` in the same merge and
re-records the goldens, so the published entry says it.

**`scripts/release.sh`** (§4 row R3), one verb per step of the path:

```sh
scripts/release.sh bump <version>     # every occurrence of the version, CHANGELOG.md's heading
gh workflow run release.yml --ref main  # the pack, on Actions (below): five tarballs + manifest.json as the run's release-<version> artifact
scripts/release.sh fetch <run-id>     # that artifact into dist/release/<version>/
scripts/release.sh verify [--registry] [--instabox]  # the install test (below) from the tarballs, or from the registry after publishing; --instabox once more inside an InstaBox machine
scripts/release.sh publish            # npm publish, the three platform packages first, then engine, create-game; git tag v<version>; prints the install line
scripts/release.sh pack               # by hand instead of the workflow: OG_RELEASE=1 pack.sh --linux-x64 (the engine with knowledge/, this machine's and the linux-x64 platform packages), then create-game, into dist/release/<version>/ with manifest.json
scripts/release.sh manifest <dir>     # the manifest over a directory of tarballs (what pack and the workflow's assemble job end with)
```

**The pack runs on Actions since 0.6.0** (the owner's instruction,
2026-10-26, after 0.6.0's packs took 31 min on the Windows host and as
long on the Mac, each compiling the same wasm five times over): the wasm
is the same bytes on every platform, so `.github/workflows/release.yml`
(`workflow_dispatch`, no secrets, nothing published) builds it once in the
gate's Debian container together with the linux-x64 addon (cargo
zigbuild, as `pack --linux-x64` did on the Mac), the engine package and
create-game, while a `native` matrix (`macos-14` for darwin-arm64,
`windows-latest` for win32-x64) compiles only its addon and `og`
(`OG_RELEASE=1 packages/engine/scripts/build.sh --native`, then `pack.sh
--native`). An `assemble` job writes one manifest over the five tarballs
(`scripts/release.sh manifest`), runs the install test's host half from
them on the Linux runner (a smoke: no renderer there, frames unchecked)
and uploads `release-<version>`. `scripts/release.sh fetch <run-id>` puts
that artifact in `dist/release/<version>/` with its `verified` cleared,
and `verify` and `publish` go on from there as before: the manifest
names the commit the runners checked out, and `publish` holds HEAD to it
as to a local pack's. The Mac and Windows machines no longer pack; 0.5.0
and 0.6.0 were packed by hand on each (the receipts), and `pack` stays
for a machine without the workflow.

`pack` (local or the workflow's) holds the gate lock (a release build
compiles beside nothing) and refuses a `native/` that is not release;
`publish` refuses when `verify`
has not passed on these tarballs (`manifest.json` records it), when the
working tree is dirty, or when `main` is not at the commit the tarballs
were built from. It needs the owner's `npm login` (the `@origingame`
scope; 2FA) once per machine, or the project secret `NPM_TOKEN` (a
granular access token with 2FA bypass, the owner's, 2026-10-23), which
`publish` reads through a temporary `NPM_CONFIG_USERCONFIG` without
touching `~/.npmrc`. Publishing is an outward action: it runs only by
hand. **Where (the owner, 2026-10-23):** a publish runs on the
`macrunner` runner (`/Volumes/app/origin-engine`) by the Mac's thread,
or from the coordinator orb with `NPM_TOKEN` on the Mac's verified
tarballs (0.2.0 went that way after npm's browser 2FA flow failed on the
Mac), from a `main` the metal functional lane has passed; the orb then
runs `verify --registry` (its host half; the orb has no docker daemon, so
the container pass is the tarballs' one) and, when the one InstaBox
machine is free, `verify --registry --instabox` (D4).

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

**In an InstaBox machine** (`tests/install/instabox.ts`; `verify
--instabox`): the same `run.ts`, uploaded into a machine the script
creates from the `agent-browser` template (node 22, git; the script
installs bun 1.4.4 as the container's Dockerfile does, and nothing of the
engine), run there `--host-only` in the background (a foreground command
is capped at ten minutes and not streamed) with its log followed from
here, from the registry or from the tarballs (uploaded in 8 MiB parts,
the API's file limit being 10 MiB), the machine deleted after a pass and
kept after a failure to look at. It records `verified.instabox` in the
manifest. It is not a condition of `publish` (it needs the key and is
billed by the minute) but it is the check of every published version on
the machine class the development threads run on (§4 T), and the
0.1.1 run is `tests/evidence/release/instabox-0.1.1.md`: 14 s from the
registry, renderer `none`, frames unchecked, the same lines as the
container.

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
| R4 | **Written 2026-10-22** (`tests/install/run.ts`, `tests/install/Dockerfile`); its first pass is recorded in R8's evidence. The install test (`tests/install/`, §3) on this machine and in the linux-x64 container, from tarballs and `--registry`. **InstaBox added the same day** (`tests/install/instabox.ts`, `verify --instabox`): the test inside an InstaBox machine, 0.1.1 from the registry passing in `tests/evidence/release/instabox-0.1.1.md` | both runs pass on the 0.1.0 candidate; the container run and the InstaBox run show `renderer none` and `N frame(s) unchecked`, never a failure |
| R5 | **Done 2026-10-22** (the `pin` line; the rest was there). `og doctor` and `og --version` outside a checkout: the package's version, the addon's, the wasm files, tsrs, the renderer, bun; inside a checkout the toolchain lines as today | the doctor's output in the container and on this machine differ only in the renderer line |
| R6 | The template's two models (`trickster.glb`, `dropship.glb`, copied from `origingame-examples`): `dropship.glb` is MIT (the `core-business` example's `LICENSE`, named in `templates/game/LICENSES.md` now); `trickster.glb` came from `edg-king-match`, which declares no licence, so the template's `trickster` entity is an instance of the CC0 `walker.glb` since 2026-10-22 and the file left the template. `@origingame/create-game` ships nothing whose licence is not named. The engine's test copy (`crates/og-tests/assets/`, `templates/fx-showcase`) is not in any package; **the owner's word 2026-10-23: the repository's copies stay** | done: `LICENSES.md` of the template names a licence for every binary; `distribution.json` regenerated; the test copies stay in the public repository (owner, 2026-10-23) |
| R7 | **Done 2026-10-22**, the URL moved to `originai-game/origin-engine` 2026-10-23 (where the code is since GitHub's move; the old name redirects). Package metadata: `repository.url` names the repository the packages are built from; `homepage`, `bugs`; the README's install section (`bun add`, `npm create`, the three commands) | `npm pack --dry-run` lists what §2 says and nothing else; the README's lines run as written |
| R8 | **Done 2026-10-22**, 0.1.0 then 0.1.1 the same day (`tests/evidence/release/0.1.0.md`, `0.1.1.md`; 0.1.1: the `.gitignore` npm drops from tarballs, stored as `gitignore`) ( the four packages on npm, `verify --registry` passing on darwin-arm64 and in the linux-x64 container; npm's staged publishing held the darwin package back for ten minutes, see the evidence). **0.2.0 published 2026-10-23** from the coordinator orb with the owner's `NPM_TOKEN` after the Mac's browser 2FA flow failed (`0.2.0.md`): `verify --registry` passing on darwin-arm64 (Metal) and on the orb's linux-x64 (lavapipe); staged publishing took 4 to 8 minutes per package. **0.3.0 published 2026-10-23** from the coordinator orb (`0.3.0.md`), five packages: the first publish of `@origingame/engine-win32-x64` (F7, packed on `studio-win`) landed with it; `verify --registry` passing on the orb's linux-x64 (lavapipe), staged publishing 2 to 4.5 minutes. Publish 0.1.0 (owner: `npm login`, the `@origingame` scope); `release.sh verify --registry`; tag `v0.1.0` | `npm view @origingame/engine version` answers `0.1.0`; `npm create @origingame/game` in an empty directory on a machine without this checkout runs `og test`; evidence `tests/evidence/release/0.1.0.md` |

### L. The local loop on the platform (platform repository; after R8)

| # | Row | Passes when |
|---|---|---|
| L1 | **Done 2026-10-22** on the platform (pin `0.1.1`; `origin-engine.ts` installs with `bun install --os/--cpu`, both targets lay out in seconds on this Mac; `--from <tarball dir>` for a candidate). `origin-engine.json` pins `{ "package": "@origingame/engine", "version": "0.1.0" }`; `engine-toolchain.ts` runs `bun install` of the engine and create-game at that version into `ORIGIN_ENGINE_CACHE/<version>/` (`--from <dir>` takes candidate tarballs) and lays `ORIGIN_ENGINE_DIST` out of `node_modules` (`bin/og` → the platform package's `og`, `engine/` → the package, `archetypes/` → create's, `overlay/` → the mounted scaffold); no checkout, no `git archive`, no cargo, no token | `bun scripts/dev/workspace-stack/engine-toolchain.ts` on a machine with no engine checkout and no Rust produces a layout `readEngineLayout` accepts |
| L2 | **Done 2026-10-22** (`packages/game-knowledge/scripts/mount-engine.ts`; the 0.1.1 mount brings `.gitignore` back, game-knowledge 137/137). The knowledge mount from the package: `node_modules/@origingame/engine/knowledge/scripts/mount.ts <checkout> --package node_modules/@origingame/engine` writes `packages/game-knowledge/production`; `--check` in the platform's tests | `--check` is clean at the pinned version; game-knowledge's bundle tests pass |
| L3 | **In progress 2026-10-22**: the runner's `tests/og.rs` passes on the 0.1.1 layout; agent-server 442 of 443 (one staff-route test failed under load, to rerun idle); `stage-engine`, `deploy`, `engine-page.ts` pending. The local stack at the version: `stage-engine`, `deploy`, `engine-page.ts` on a real Portal (desktop and phone, plain and isolated releases), `agent-runner` `tests/og.rs`; this is the merge gate of every pin move | all four pass; `origingame/goals/origin-engine/evidence/pin-0.1.0.md` |
| L4 | **Done 2026-10-22** (no cargo or git archive under `agent-server/scripts/e2b`; the workflows check out no engine and need no token; E1's image contents written: lavapipe only in `asset`). `origin-engine.ts` (the layout builder) and `build-templates.ts`'s engine step reduced to the install of L1; `ORIGIN_ENGINE_READ_TOKEN` removed from the workflows and the plan's §四 4 | `rg cargo agent-server/scripts/e2b` finds nothing; the workflows check out no engine repository |
| L5 | **M7 acceptance, locally**: a thread on the local stack makes one Discover prototype end to end (`engine_scaffold` → `og run --commands` playtest → delivery → publish to the local Portal, played back byte for byte) | the receipt in `evidence/m7-local-0.1.x.json`; the published page passes `engine-page.ts` |

### D. The development sandbox on InstaBox (platform repository and engine; after L3, before E)

The sandbox the development stage's threads run in is an InstaBox machine
(§1). The platform reaches it as one more provider beside `e2b`, routed
per class; the engine's part is the install test inside such a machine
(R4, `tests/install/instabox.ts`) and ask 15 of `docs/platform-asks.md`,
which lists what the machine must carry. Nothing here waits on E2B.

| # | Row | Passes when |
|---|---|---|
| D1 | **Done 2026-10-23** (platform main 2c2d5c06, `goals/origin-engine/evidence/instabox-0.1.1.md`). The platform's `instabox` provider beside `e2b` (`agent-server/src/sandbox/`): create from a snapshot or template, run, files, pause and resume, delete, the port URL; the same `Sandbox` interface the runner already speaks, no engine-specific member | agent-server's sandbox tests pass on the `instabox` provider against the real account (the free plan's one machine, serially); a thread on it runs `og doctor` |
| D2 | **Done 2026-10-23** (platform main 2c2d5c06, `goals/origin-engine/evidence/instabox-0.1.1.md`). Per-class snapshots that install the pinned version (`agent-server/scripts/instabox/`: from `agent-browser`, bun 1.4.2, `bun install` of the engine and create-game with `optionalDependencies`, ffmpeg for `probe`; no renderer, no checkout, no Rust, no credential left in the image), rebuilt only when `origin-engine.json` moves; a checker that is `check-templates.ts` for InstaBox | the checker passes on the snapshot of the pinned version; its receipt in `origingame/goals/origin-engine/evidence/instabox-<version>.md` |
| D3 | **Done 2026-10-23** (platform main 2c2d5c06, `goals/origin-engine/evidence/instabox-0.1.1.md`). The development routing: `sandbox.provider.<class>` names `instabox` for the development stage's classes and `e2b` for production's, in the runtime configuration, never in code; the runner reconnects after a machine's auto-pause (thirty idle minutes on every plan) | a development thread on `engine` lands in an InstaBox machine, a production thread in E2B, with no change to the engine package |
| D4 | **Done for 0.1.1** (`tests/evidence/release/instabox-0.1.1.md`) **and 0.2.0** (`0.2.0.md`: 20 s on a fresh machine from the registry, deleted after) **and 0.3.0** (`0.3.0.md`: 107 s on a fresh machine from the registry, the first version with the win32-x64 package on npm, deleted after). The install test in an InstaBox machine at every published version (`release.sh verify --registry --instabox`) | `verified.instabox` in the manifest of every version from 0.1.1 on |
| D5 | **M7 acceptance in the sandbox**: L5's prototype, made by a thread in an InstaBox machine from D2's snapshot | the receipt in `evidence/m7-instabox-0.1.x.json`; the published page passes `engine-page.ts` |

### F. The features still owed (engine repository; beside R and L)

The owner's rule of 2026-10-22 (`docs/phase-1.md`, "Features before
optimisation") stands: feature streams and dogfood rounds only, the
optimisation rows (§P) after them.

| # | Row | Passes when |
|---|---|---|
| F1 | **All three merged and Metal-verified** (`kit-words` a61fd4f4, `platformer-2d` c9845b1e, `dogfood-7` 7aa1b801). The missing `PARTICLES_ARMED` runtime export was fixed in ff952176. Meridian Loop's eleven moved frames were inspected, recorded in d07963ef and reproduced at RMSE 0; the HUD, air-yaw retune and later captures explain the changes. Brasswick's eight suites passed with all nine frames unchanged at RMSE 0. Rumpus Isle's eleven Metal references landed in e9b69719 after inspection, all nine suites passed (239 checks, frames RMSE 0), and the functional lane passed in 91 s (`docs/phase-1.md`, F1 sections; two presentation debts recorded in the game's FINDINGS) | each merged through the functional lane; Meridian Loop's ten suites, Brasswick's eight and Rumpus Isle's nine green on Metal |
| F2 | **Closed 2026-10-24.** Row 58's four cooked-font scenes pass on WebGPU and WebGL2 against Metal. Row 63's `_loop` and playback range landed in 1ef30b48; row 59's kill buffer and visual bidi movement in 284b32c2. The visibility-track optimisation remains deferred | each feature row closed in `docs/phase-1.md`; browser acceptance in the round 3 closeout |
| F3 | **Audit closed 2026-10-24.** Every non-optional phase 2 and 3 feature has an implementation and historical checks. Tune is the reflected probe contract, not an engine UI; splats remain optional and unplanned. The full §7 acceptance is still open: real phones, current reference-scene and Chrome lanes measurements, capture latency and executor image size | `docs/phase-1.md` §F3 coverage audit and terrain acceptance maps each §11 item to its source and evidence; it does not claim the unmeasured budgets passed |
| F4 | **Closed 2026-10-24.** Brasswick's nine and Meridian Loop's ten frames pass on both browser tiers. Published-page checks pass 21/21 and 22/22 respectively on each tier; device checks pass 27/27 for each game. Meridian's WebGL2 references account for its MSAA/bloom difference, not a renderer defect | `tests/evidence/r3/meridian-webgl2/NOTES.md` and `docs/phase-1.md` round 3 closeout; `published.ts` stays a manual check |
| F5 | wgpu and naga 31 when published (row 35), kept only if every reference frame passes unchanged. **Still upstream-blocked 2026-10-24:** `cargo search` lists 30.0.1 for both. Since 2026-10-26 wgpu-hal 30.0.1 is carried in `third_party/wgpu-hal` (the root `[patch.crates-io]`; every other wgpu crate is the crates.io one) with one GLES-only patch, its own commit: WebGL2 GPU pass timing through `EXT_disjoint_timer_query_webgl2` (`third_party/wgpu-hal/ORIGIN.md`, `tests/evidence/debt3/wgpu-hal-vendor/NOTES.md`; a second patch, the queue skipping GL calls that set what is already set, was reverted at the merge for want of a Mac gain, row 101). At 31, 31's crates.io wgpu-hal replaces the copy as one unchanged commit and the patch is carried over as its own commit or dropped where 31 has an equivalent (ORIGIN.md §Moving wgpu); the copy and its patch line go when no patch is left. Either way `tests/web/gpu-timing.ts` passes | a wave end at the new version with every frame unmoved and the WebGL2 GPU timing kept, or the row stays open with the date |
| F6 | Each wave's end: `scripts/wave-end.sh` (`gate`, `web`, `conformance`, `release`), summary in `docs/phase-1.md` | the wave's section records the four stages |
| F9 | **The art wave** (`docs/art.md`, owner 2026-10-28): a game's style as a contract the assets meet (the look's art resources, the cook's conformance, the runtime's sprite pieces, `og style check`), previews and honest layers in the catalogue, and the platform's style step at game creation; released as 0.7.0 | `docs/art.md` §6 |
| F7 | **Started 2026-10-22 on the `studio-win` runner**; **native MSVC candidate verified 2026-10-23**, branch `windows-x64` (the coordinator merges). Release `@origingame/engine-win32-x64` carries `og.exe` and `origin-engine.node`, with embedded DXC. Workspace check/clippy pass; nextest runs every crate: 939 passed, five Metal-reference failures, 12 skipped. All 84 DX12 gfx scenes render; six exceed Metal tolerance, no new tier references committed. The 0.2.0 packed-tarball install test passes in 87.1 s (3D starter, runner, 2D starter, publish wasm, knowledge mount/check). Doctor reports `renderer dx12` and the explicit `tsgo 7.0.2 through bun x` fallback. Windows paths, LF/HDR checkout, process fixtures and mount permissions are portable; one TypeScript API. Evidence and exact frame differences: `tests/evidence/release/windows-x64.md`; device row `windows` below. No cloud path changed | **Merged 2026-10-23**; the six DX12 frame differences reviewed by the coordinator (`windows-x64.md` §The coordinator's review: star, texel-edge and caustic dither, nothing moved; no DX12 reference tier). The win32-x64 package's first publish landed with 0.3.0 (2026-10-23, packed on `studio-win` from d8bcf2d2, its tarball install test 115.3 s on DX12; `tests/evidence/release/0.3.0.md`). Still owed: the install test from the registry on Windows and one engine thread on the platform runner's Windows host; the Mac's registry run passed (76.9 s, Metal, `0.3.0.md`) |
| F8 | **The depth wave** (`docs/depth.md`, started 2026-10-22): nine streams on branches `depth/<stream>`, each an orb thread, merged serially by the coordinator on `macrunner`: S1 runtime geometry, lines, points, debug draw and mesh raycast; S2 paths and the timeline asset; S3 the video producer (`og render`, `type video`); S4 the room ABI and deterministic lockstep with desync diagnosis; S5 the semantic frame, `expect`, trace, scene write-back and `cook --report`; S6 mesh LOD, reflection probes and SSR; S7 baked lightmaps, irradiance volumes, volumetric fog, area lights and cookies; S8 render targets, secondary cameras, layers, full custom shaders and custom post passes; S9 2D skeletal animation and the math layer | each stream merged through the functional lane with its Metal references recorded and its round in `docs/phase-1.md`; `docs/depth.md` §10's order; the wave's end measured once |

**Fixed in 0.6.3 (2026-10-27, `tests/evidence/release/0.6.3.md`):
the replay gate passes 3/3 platforms × 2/2 goldens. Found 2026-10-27
(0.6.2's replay gate): cross-platform yaw drift.** The golden replays (`scripts/replay-gate.ts`) pass 2/2 on
linux-x64 and darwin-arm64. On win32-x64, game-2d passes; game-3d's hashes
differ at ticks 62–65 only, and all its keyframe saves are byte-identical
to Linux's. At tick 62 the hash is Linux `810661cd9c33c9d3` and Windows
`dc03b963…`. The only field that differs is the guard's
`og.Transform.quaternion`, by 1 to 16 ulp, until the turn snaps to its
target at tick 66. The cause is og-nav's faceMovement turn
(`crates/og-nav/src/system.rs`), which calls glam's `Quat::slerp`, and that
uses the host's `sinf` (the MSVC CRT versus glibc), against og-core
math.rs's rule. 0.6.1 had it too, unseen: `og replay` is the first tool
that measures it. 0.6.2 shipped with it by the owner's decision. The fix
is 0.6.3: the turn as a yaw step through og-core, an `og_core::math::quat_slerp`,
clippy `disallowed-methods` in the simulation crates, simulation
"changed" and the goldens re-recorded. 0.6.3 publishes only when all 3
platforms pass both goldens.

**F7 complete, 2026-10-24:** the Windows registry install test now passes
(117.0 s, DX12, published 0.3.0 packages only). The real-engine runner
tests pass 2/2 and runner library tests 62/62 after two local platform
fixes: selecting `og.exe` and waiting for the stopped service's leader.
`tests/evidence/release/0.3.0.md` records commands and limitations. After
the authorized Identity and Agent Service fixes, a fresh registered
Windows runner completed scaffold/start/probe/capture/edit/reload/stop
and pushed its branch. A fresh clone verified all eight LFS objects.
Registrations, tokens, processes and tunnels were cleaned up; test
repositories were archived. `tests/evidence/release/0.4.0.md` records the
real-thread receipt separately from the 0.4.0 package checks.

### T. The test matrix (what runs where, when; standing)

| Layer | What | Where | When |
|---|---|---|---|
| Crates | fmt, clippy, unit and logic tests, gfx references on `metal`, drift of the generated `.d.ts`, wiki and skills | `scripts/check.sh quick` / `functional` | every merge to `main` (the coordinator; `docs/ci.md`) |
| Packages | `packages/engine` bun tests on the dev addon, typecheck through tsrs, create's and knowledge's tests | `functional` (bun-functional) | every merge |
| Templates, archetypes, examples | `og test`, `og polish`, their reference frames | `functional` (bun-test) | every merge |
| Wasm | build, wasm-opt, size (a number to read), determinism across hosts | `check.sh --all` | every wave end |
| Browser | both tiers' reference frames, the kit, the a11y mirror, the published page under a script | `check.sh web-reference`, `og run --device chrome`: Chrome on macrunner, and Chrome and Edge on studio-win (WebGPU on D3D12 and WebGL2 on ANGLE/D3D11) | every wave end and before every publish; by hand for a rendering change |
| Release | pack, the install test from tarballs on darwin-arm64 and linux-x64 | `wave-end.sh release` / `release.sh verify` | every wave end; before every publish |
| Registry | the install test from npm | `release.sh verify --registry` | after every publish |
| Platform, local | `stage-engine`, `deploy`, `engine-page.ts`, runner `tests/og.rs`, the knowledge mount `--check` | the platform's local stack | every pin move (L3) |
| Devices | iPhone first frame and frame times (Safari, WebGPU); a mid-range Android's 10,000-entity tick is deferred by the compatibility baseline (design §8, the owner, 2026-10-27) | `og run --device iphone` on the owner's devices | once before launch, then per minor version |
| windows | Native MSVC workspace check, clippy, nextest of every crate; `og-gfx --all` DX12 frames against Metal; web-reference in Chrome and Edge (WebGPU and WebGL2); install test from packed tarballs, doctor renderer and checker lines | `studio-win`, Windows x64, `tests/evidence/release/windows-x64.md`; no new reference tier without coordinator approval | after every batch of merges (the coordinator starts it beside macrunner's metal lane), and on the release commit before every publish |
| InstaBox (development) | the install test inside a machine (`release.sh verify --registry --instabox`); the platform's snapshot checker (D2) | an InstaBox machine (billed by the minute; the free plan's one machine, kept at one for the verification stage, §6 item 6) | the install test after every publish; the checker once per pin move, after L3 |
| Runners (the owner's machines, Amp runners) | `macrunner`: the functional lane on `metal`, every reference frame's recording, the wave's end (`--all`, web-reference) in the measure lane; `studio-win`: the Windows x64 build, its tests, the install test with the `dx12` tier (F7); a coordinator orb (InstaBox stage, no GPU) merges and runs the functional lane with the GPU frames set aside. **Every runner's checkout is kept at the latest `origin/main`** (the owner, 2026-10-23): a runner thread fast-forwards `main` before each job and merges `origin/main` into its stream branch before each push; a publish runs from the Mac (§3) | `/Volumes/app/origin-engine` on the Mac, `C:\Users\win\src\origin-engine` on the Windows host | the metal lane after every batch of merges the orb cannot frame-check; Windows per F7, then every publish |
| E2B (production only) | `check-templates.ts` on the image that installs the version | E2B (billed) | once per pin move, after D; never for development |
| Production | `verify-production.sh`, `state.ts` without drift, the Player telemetry of the first releases | the platform's release process | at launch and each ship |

### E. E2B and production (platform and owner; last)

E2B is production's provider only (§1). No row of R, L, D or F waits on
one of these; they run once per pin move after D, as the last consumer of
the version.

| # | Row | Passes when |
|---|---|---|
| E1 | The `engine`, `probe` and `asset` images install the pinned version (`bun install` of the engine and create-game with `optionalDependencies`, in the image, no registry at run time), carry no renderer, no layout, no checkout; `/opt/origin-engine` is the installed project directory; the `engine` image ≤ 600 MB | `check-templates.ts` passes on the image built from the version; the size in its receipt |
| E2 | The owner enables the four templates in the runtime configuration and ships agent-server (`release.sh ship agent-server --development`; the prerequisites of the plan's §四 4 minus the token) | `bun deploy/ovh/state.ts` shows no drift but HELD; a production thread on the `engine` class |
| E3 | The templates built on 2026-10-20 from the `d51c8ae` layout are not enabled; they are superseded by E1 | the runtime configuration never names them |
| E4 | Production: the game-host ingress for `/__player/v2/` (applied 2026-10-20), `verify-production.sh`, the first engine release's Player telemetry read | the receipt in the platform's evidence |

### P. Deferred until the features are complete (engine; after F)

Owed rows 14 and 44 (the publish wasm under 65 %, the budget's meaning
the owner's), 51 and 52 (the three.js comparison and the lanes physics in
Chrome; row 51 is judged on the complex scene, `tools/bench/complex`, since
2026-10-25: ≥ 1.5× three.js on WebGPU, WebGL2 and the reference scene
reported, `tests/evidence/r4/bench-complex/NOTES.md`), 56 (quantised clips resident), 88
(the web first-frame settle cost; the native late-arrival test passed on
2026-10-24 at `195cd595`), 89 and 90 (perf-games'
idle-machine tables and follow-ups; branch `perf-2`). Each is taken up as
the product needs it, measured at a wave's end, and never holds a merge
or a release.

Row 99 remains open, but several proposed trims already landed. The
debt2 size stream (`c8d7c2b3`, `tests/evidence/debt2/size/NOTES.md`) removed
GPU timestamp queries and tool-only probe code from publish and excluded
3D world subsystems from the 2D renderer. Round 3's profile-quality stream
also removed the 2D runtime's terrain implementation. The round 3
closeout's gate recorded a largest download of 3,787,011 B brotli (90 % of
4 MiB; a plain `scripts/check.sh`, so the `wasm-gate` profile, which runs
5 to 6 % above the shipping one, docs/ci.md §Lanes). Branch `r4/size-2`
(`tests/evidence/r4/size-2/NOTES.md`, from 1bb341e5) took the new approach:
one non-generic stable sort and B-tree build for the engine
(`og_core::sort`, the largest monomorphisation family), no reflected or
record field docs in publish, one bincode encoder for the physics save, and
the `wasm` profile at opt-level s with the per-frame crates at 3. Shipping
profile, brotli: 3D publish 3,561,039 → 3,283,141 B (78.3 %), 3D lanes
3,591,631 → 3,320,422 B (79.2 %, the largest), 2D publish 3,291,397 →
3,034,243 B (72.3 %), 2D lanes 3,327,994 → 3,072,212 B (73.2 %): every
variant under 80 %. Still open until the coordinator's gate, its
determinism matrix, the Metal frames and Chrome's frame times (og-runtime
at s is the one per-frame crate whose cost was not measured).

The platform already routes standalone CAS wasm through a content hash
and basename in `server/src/play/serve.ts`. Its current test uses a PNG,
not two engine games in a browser. Relative redirects on distinct game
origins do not establish one shared absolute URL, and `.br` and `.gz`
paths remain release-scoped. Cross-game wasm download reuse therefore
still needs implementation or evidence, as the platform audit below
records. Neither the size target nor that audit authorizes deployment.

### Platform audit, 2026-10-24

Read-only audit of platform `main` at `ffd54d999`, with a clean worktree.
These are source and archived-receipt findings, not fresh test runs or
claims about deployed production state. They supersede the older version
and pending labels in L and D above.

| Item | Current evidence | Remaining work |
|---|---|---|
| L1, L3 | `agent-server/scripts/e2b/origin-engine.json` pins 0.3.0. `goals/origin-engine/evidence/pin-0.3.0.md` records runner 2/2, Stage 12/12, real Portal desktop/phone-viewport checks in plain and isolated modes | Agent-server 450/451, with a staff-route failure also on unchanged main. The `deploy` scenario still calls retired `search`/`execute deploy_create` and `deploy_update`; migrate the acceptance to the current Wizard `publish` contract rather than restore the retired tools |
| L5, D5 | No `m7-local-*` or `m7-instabox-*` receipt found. `workspace:stack deploy` uses fixtures and uploads no bytes | One engine thread must make, publish and read back a prototype through a real local Portal, then repeat on InstaBox |
| D2, D3 | Provider and four 0.1.1 snapshots have real receipts in `instabox-0.1.1.md`; `agent-server/scripts/instabox/dev.env` still names those snapshots | Build/check the current pin's snapshots with approval. Natural 30-minute idle pause was not observed; provider-driven pause/resume was. D4's 0.3.0 package install is not a platform snapshot or M7 acceptance |
| Rooms, ask 16 | `room.ticket` and `?room=` propagation exist | Create/join/leave/send relay and an event transport are absent. The engine channel's 16 MiB bound differs from the HTTP call's 320 KiB limit; implement explicit semantics and two-player acceptance |
| Player clock, ask 9 | In-page Stage stamps results through `player-core.ts`'s optional clock | Hello, framed Player, HTTP responses and the wire readers need one contract and success/error coverage |
| Creation style, ask 12 | `og-taste` catalogue and authoring guidance are mounted | No executable style-step/blind-critic chain found. The current Wizard adapts and deploys an existing game; the creation entry point needs a product decision before adding a style-changing step there |
| Shared wasm, row 99 | `server/src/play/serve.ts` has a CAS shared route, covered by `serve.test.ts` with a PNG | Verify wasm bytes and actual cache reuse across two game origins, including compressed exports and browser cache partitioning |
| Production | Platform workflows install the npm pin without an engine checkout or cargo | No current production image/runtime-config verification in this audit. Image build, deployment and activation require separate authorization |

After the audit, platform commits `30b23ba51` and `ff518833c` implemented
the Player clock wire stamps and tests locally; `0cb3f8601` adds
server-offset calibration and `db9ba6680` records its convergence limit.
The engine consumes hello time at `b7fcbacb`. The coordinator reviewed
the calibration and reran the SDK suite, 33/33. Stage and verify remain
on their page clocks, and an already-stamped ahead clock holds until
server time catches up or the next load uses the stored offset.
`docs/platform-asks.md` ask 9 records these limits. Windows fixes and
its process-cleanup test were integrated into platform main through
`2f0258332`. Nothing in this round is pushed or deployed.

Platform `ff6eb749a` subsequently migrated the local deploy scenario to
the `publish` tool and a stand-in Wizard. Its real runner uploaded 44
files (30.4 MB) of engine publish output to the fixture Portal; all ten
steps and 49 assertions passed. Removing `source_checkpoint` made the
publication fail. This closes the retired-tool failure in that scenario,
not L5/M7: adaptation, models, artwork, cover requirements and readiness
are not exercised, and the Portal and Identity remain fixtures.

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
2. ~~Whether the repository becomes public~~ (MIT since 2026-10-17;
   **decided 2026-10-23: public**. The packages' `repository.url` names
   `originai-game/origin-engine` (R7); the visibility change is made on
   GitHub by the owner's account (the orb's token cannot); CI on GitHub
   restored with it, 2026-10-25, row 83).
3. ~~The two models of the template (R6)~~ (2026-10-23: the test copies
   stay; the template ships the CC0 `walker.glb`).
4. The wasm budget's meaning (row 44), `topdown` in Discover, third-party
   Godot Web exports, the WebGL2 fallback's lifetime, a third native tier
   (rows 39): unchanged from the platform's plan §四.
5. The devices for the T matrix's device row (row 40).
6. The InstaBox plan. The account (zhangfan0220@gmail.com) is on the free
   plan on 2026-10-22: one machine at a time, created from the
   `agent-browser` template or a snapshot only, 4 vCPU and 8 GiB,
   auto-paused after thirty idle minutes. One development thread at a
   time fits; the dogfood rounds and the M7 acceptance beside each other
   (D3, D5), and the per-class snapshots (D2: up to five snapshots, one
   per class and version), need the Pro plan. **Decided 2026-10-22 (the
   owner): one machine stays for the whole verification stage**, while
   the engine's features are not all working and the tests are not fully
   implemented; D1–D3 are designed for one machine used serially, and no
   upgrade is planned. The owner's own runners carry the rest of the
   verification (the T matrix's Runners row): `macrunner`
   (`/Volumes/app/origin-engine`, the metal gate and every reference
   frame) and `studio-win` (Windows x64, F7).
7. Whether E2B stays production's provider once D passes, or InstaBox
   takes production too. Today's answer (§1) is E2B for production, and
   stage E is written for it; the engine is indifferent.
