# Release: signed archives at /downloads/engine

Normative for how the engine leaves this repository. `docs/design.md` is
the design, `docs/ci.md` the gate, `docs/web-export.md` §Packaging the
tarballs; this document is the path from `main` to a version that a game
author, the platform's local stack, the development sandbox and an E2B
image all install the same way. Open work is in `docs/roadmap.md`.

## 1. The engine releases itself

Every version is released as signed archives at
`https://origingame.ai/downloads/engine/<version>/`, one per target, and
that is what an author's `og`, the platform's runner, Origin Desktop, the
development sandbox and an E2B image install. Nothing is published to npm;
the versions already there (0.1.0 to 0.7.7, and 0.7.8-dev.17) stay as
they are. The pin is a version string. A version is tested locally first
(the install test, then the platform's local stack), then in the
development sandbox (an InstaBox machine), and installed into E2B,
production's provider, last. The contract is shared with the platform (the
download route, the upload endpoint, the mirror) and origin-agent (the
runner, the server, Desktop).

**What a version is.** Under `/downloads/engine/`:

| Path | What | Cache |
|---|---|---|
| `<version>/origin-engine-<version>-<target>.tar.gz` | one target's dist, its root directory `origin-engine-<version>-<target>/` | immutable |
| `<version>/manifest.json` | `{version, releasedAt, revision, channel: "release"\|"dev", checksums: "SHA256SUMS", signature: "SHA256SUMS.minisig", targets: {<target>: {url, sha256, size}}}` | immutable |
| `<version>/SHA256SUMS`, `<version>/SHA256SUMS.minisig` | every archive's sha256, signed with the engine's key (below) | immutable |
| `latest/manifest.json` | the newest release (never a dev build) | no-cache |
| `install.sh`, `install.ps1` | install the `og` launcher (`packages/launcher`) | no-cache |

`<version>` is `x.y.z` or a dev build `x.y.z-dev.n` (`channel: "dev"`).
The targets are `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` and
`x86_64-pc-windows-msvc`, each a `.tar.gz` (Windows 10 and later carry
`tar.exe`). `revision` is the commit the version was built from.

**The dist** (one archive's root; `scripts/release-dist.ts` assembles it,
and it runs unpacked, with no npm and no `bun install`):

- `origin-engine.json`: the scaffold overlay's `origin-engine.json` with
  `build {package, version, revision, target, triple, og, addon, wasm,
  wasmPublish, wasmPublish2d, wasmLanes}` (the sha256 of each binary);
- `overlay/`: the knowledge's `game-scaffold/origin-engine-ts`;
- `files/<path>`: the binaries `distribution.json` names, sha256-checked;
- `bin/og` (`bin/og.exe` on Windows);
- `engine/`: what `@origingame/engine` carried (src, types, schema,
  shell, `dist/web*`, `knowledge/`, `styles/`, motion weights) with
  `native/origin-engine.<platform>.node`; this is `OG_ENGINE_PACKAGE`;
- `create/`: one empty project skeleton and `bin/create.ts` (`og new` runs it;
  `bin/brief.ts` is the art brief the platform calls);
- `design/`: `tokens.json` and `design-language.md` (from 0.7.8);
- `node_modules/@tsrs/<platform>/`: the type checker `og check` finds
  beside the engine (none on Windows, where og check uses tsgo).

The addon and tsrs keep the node platform names
(`origin-engine.linux-x64.node`, `@tsrs/linux-x64-gnu`), because every
version's loader (`src/node`, `og check`) looks for those; `build.target`
is that platform (`linux-x64`, what origin-agent's layout reader uses) and
`build.triple` the archive's target.

**The release key** is a minisign (Ed25519) pair:

```text
untrusted comment: minisign public key F6CC11AC12997D89
RWSJfZkSrBHM9kOnWQNC0WoGXEFI+9NMCj8+Wy6WJDkYl9pgPGkoCoxt
```

The secret key is the project secret `OG_ENGINE_MINISIGN_KEY` of
`origingame/origin-engine` (the unencrypted key file's content); signing
writes it to a 0600 temporary file for `minisign` and removes it after,
and it is never a file in this repository. The public key is pinned by
the launcher (`packages/launcher/og.ts`, whose installers refuse to build
with another), by the platform's upload endpoint and mirror, and by
origin-agent's runner. A new key is a new launcher and a note to both.

**Publishing** is an upload to the platform (`scripts/release.sh
publish`, §3), which verifies the signature and every sha256, serves the
files and refuses to overwrite a version. There is no GitHub release and
no npm publish.

**The cache** every consumer shares is `$ORIGIN_ENGINE_CACHE` (default
`~/.cache/origin-engine`), a version at `engines/<version>/<target>/`
(the archive's root). An install downloads `SHA256SUMS` and its
signature, verifies it with the pinned key, downloads the archive and
checks its sha256 against the signed list, unpacks into a temporary
directory and renames it into place: a version is complete or absent, and
nothing writes into it after. Its files keep the archive's modes and are
not made read-only: every released version's tools copy files out of the
dist and keep their modes (create's template files, `og build`'s publish
wasm, `snippets/` and `basis/`), so read-only files would make a new
game's `project.ogt` unwritable and a second `og build` fail.

**The launcher.** An author installs `og` once (`curl -fsSL
https://origingame.ai/downloads/engine/install.sh | sh`, or `irm
…/install.ps1 | iex`: `~/.origin/bin/og`). It reads `project.ogt`'s
`engine "<v>"`, installs that version into the cache when it is missing,
and runs that version's `og` with `OG_ENGINE_PACKAGE` set; `og new`
scaffolds a game (the latest release, or `--engine <v>`), `og upgrade`
moves the pin, and `og engine install <v> | list | path | prune` manages
the cache. A game has no `node_modules`: `og` keeps `.og/engine` (ignored
by Git) pointing at the engine it runs with, and the templates'
`tsconfig.json` maps `@origingame/engine` there for editors.

### 1a. Dev builds while a pin is being tested

A fix the platform needs to try does not wait for a release. It is
pushed to `main`, built by `gh workflow run release.yml --ref main -f
dev=true` (the checkouts carry `<next patch>-dev.<run number>`, e.g.
`0.7.8-dev.12` after 0.7.7; nothing of it is committed), and archived,
signed and published on the `dev` channel at the run's commit by
`scripts/release.sh dev <run-id|dir>`. The platform then pins that exact
version. A dev build runs no install test and no browser tier, and writes
no receipt, tag or changelog entry (its changes stay under `unreleased`);
`latest` never points at one. A release (§3) is cut at a milestone, from
whatever dev build the testing settled on, with every gate.

## 2. What the packages carry

`scripts/release.sh pack` builds npm-shaped tarballs in
`dist/release/<version>/` as intermediates; `release-dist.ts` assembles
each target's archive (§1) from them.

| Package | Contents | Built by |
|---|---|---|
| `@origingame/engine` | the TypeScript API and runtime (`src/`, `types/*.d.ts`, `schema/api.schema.json`, `bindings.json`), the web hosts (`dist/web`, `dist/web-publish`, `dist/web-publish-2d`, each publish directory with its lanes variant), the player shell, the Basis transcoder, `bin/og.mjs`, and `knowledge/`: `wiki/` (generated from `api/<module>.ogt` and the guides beside it), `looks/catalogue.json`, `scaffold/` (`origin-engine.json`, `distribution.json`, the template's text), `knowledge.json`, `scripts/mount.ts` and `validate.ts` (copied from `packages/knowledge` at pack), and `styles/`: the built-in UI styles (`templates/styles/<pack>/`'s `style.ogt`, atlases at 1×, 2× and 3×, icons, fonts, licences; not `art/`), which og's cook reads from disk and copies into a game's bundle only for the style it names; no native code | `packages/engine/scripts/pack.sh` (`OG_RELEASE=1 build.sh --linux-x64` first), in `scripts/release.sh pack` on macrunner |
| `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64` | `origin-engine.node` (napi addon, `crates/og-host-node`) and `og` (`crates/og-cli`), release profile | both in the Mac's `scripts/release.sh pack`: linux-x64 by cargo zigbuild (glibc 2.31), darwin-arm64 natively |
| `@origingame/engine-win32-x64` | `origin-engine.node` (MSVC napi addon) and `og.exe`, release profile; the same API, DX12 renderer, tsgo checker fallback | `build.sh --native` then `pack.sh --native` in Git Bash on studio-win (§3) |
| `@origingame/create-game` | `bin/create.ts`, `template/` (one empty skeleton), `looks/` (the catalogue), presets, previews, styles and locales | `packages/create` `prepack` |
| `@origingame/design` | `tokens.json` and `design-language.md`, the design language the engine's default UI style is generated from and origin-agent pins | `bun pm pack` of `packages/design` in `scripts/release.sh pack` |

`packages/knowledge` is a private source package of this repository. Not
shipped: linux-arm64 (design §12 item 5). `@tsrs/*` are someone else's
packages, pinned at `TSRS_VERSION` of `scripts/install-tools.sh`, not at
the engine's version.

## 3. Versions and the release procedure

**Version.** Before `1.0.0` the minor number counts breaks and the patch
number everything else. A minor bump (`0.x.0`) is a release with a break:
an IDL member removed or renamed, a field's type or default changed, a
saved value or enum index whose meaning changed, or behaviour a game
relied on changed, so a game may need edits to upgrade and the changelog
says which. A patch bump (`0.x.y`) is everything compatible: new members
off by default (a game that does not set them draws and runs exactly as
before, which the reference frames hold), variants appended to an enum,
fixes, frames and speed. Either way the changelog has one entry per IDL
member added, changed or removed, recorded by the merge that changes it
(below); a compatible but large release may still take a minor when the
owner names it. Every occurrence of the version is written by
`scripts/release.sh bump` and held equal by `packages/knowledge`'s test
(`scripts/release.sh check`): the four `package.json`
(`optionalDependencies` included), `scaffold/origin-engine.json`,
`packages/create/template`, `packages/engine/sheets`, `tests/fixtures/*`,
and what `create` writes into a new project.

A Game's `project.ogt` pins its own version; `og` reports a mismatch with
the installed package (`og doctor` as a line, `og run`/`test`/`build` as a
warning: the installed package runs, the pin is the author's record, and
an upgrade is an explicit edit). A pin is **compatible** with the
installed package when both are the same minor and the installed one is
not older: no mismatch (`og doctor` says ok, the others say nothing).
`og doctor --json` reports it as `pin: {pinned, installed, compatible}`
(null outside a project).

**`changelog.json`** (in the engine package, schema 1) is the engine's one
changelog: the machine contract the platform builds its "new engine
version" card from, the source of `og upgrade`'s codemods and of each
release's notes. Its shape is `{schema, about, unreleased, versions:
[{version, date, kind: "patch"|"minor", simulation: "unchanged"|"changed",
breaking: [{member, change, fix, find?: [identifier], codemod?: {from,
to}}], added: [{member, summary}], fixed: [{member, summary} | string]}]}`,
newest first.

**Merges only record.** A merge never edits `changelog.json` (one shared
file conflicted on almost every concurrent landing). A merge that adds,
changes or removes an IDL member, breaks something a game relied on, or
fixes something a game sees adds one new file in the same commit:
`changelog/unreleased/<branch>-<topic>.json`, with any of `simulation:
"changed"`, `breaking`, `added` and `fixed` in the shapes above
(`scripts/changelog.ts` has the details). Its own name keeps two branches
from ever touching the same lines. The quick gate shape-checks every
fragment and `changelog.json` (`scripts/changelog.test.ts`).

**Folded periodically.** `scripts/release.sh fold` folds every fragment into
`unreleased` in name order (a `changed` simulation wins, the lists append)
and deletes them, refusing and writing nothing when one is malformed; the
coordinator runs it between releases and commits the result. `release.sh
bump` folds first, then moves `unreleased` into `versions` as the new
version, dated, its `kind` from the numbers, and refuses a patch whose
`unreleased` has a breaking entry; a dev build (`release-bump.ts --dev`)
folds in its own checkout, so its package carries what the merges recorded.
`simulation` says whether a game's ticks may differ from the version before
(the replay gate below holds a patch's `unchanged`); the versions before
0.6.1 were never measured and say `changed`.

**Release notes** are generated, never written by hand: `release.sh pack`
writes `dist/release/<version>/NOTES.md` from the version's entry (`bun
scripts/changelog.ts notes <version>`; with no version, the notes so far).
The prose changelog up to 0.7.7 is frozen in `docs/history/CHANGELOG.md`.

**`og upgrade [project] [--to <v>] [--dry-run] [--json]`** moves a game to
the installed engine (or `--to`, a version in its changelog): it rewrites
`project.ogt`'s `engine` (the one pin; a game has no `package.json`
dependency on the engine), applies the `codemod` of every breaking entry
of the versions crossed to `src/` and `assets/` text, and lists every line
where another breaking entry's `find` identifiers appear (`manual`, with
the change and its fix) or the entries with no identifiers (`review`). It
never downgrades; the launcher's `og upgrade` installs the target version
and runs this with it. Exit 0 done, 3 done with hand edits listed, 1
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

**Simulation checks.** Core replay, save/restore and native/wasm hash tests
remain in the engine suites. The template-specific golden replay gate is
removed with the gameplay templates. A change that moves simulation on
purpose still sets `simulation: "changed"` in the same merge.

**`scripts/release.sh`**, one verb per step:

```sh
scripts/release.sh check              # every occurrence of the version equals packages/engine's
scripts/release.sh fold               # changelog/unreleased/'s fragments into changelog.json's unreleased (any time; commit it)
scripts/release.sh bump <version>     # fold, then every occurrence of the version and changelog.json's unreleased into versions
scripts/release.sh pack               # on the Mac: OG_RELEASE=1 pack.sh --linux-x64, create-game, design as tarballs (build intermediates), NOTES.md, then archive
# on studio-win, Git Bash, the same commit: OG_RELEASE=1 sh packages/engine/scripts/build.sh --native && sh packages/engine/scripts/pack.sh --native
#   then copy packages/engine/dist/pack/origingame-engine-win32-x64-<v>.tgz into the Mac's dist/release/<v>/
scripts/release.sh archive            # scripts/release-dist.ts: an archive per target, manifest.json, a release's install.sh/install.ps1, SHA256SUMS, signed when OG_ENGINE_MINISIGN_KEY is set
scripts/release.sh pack <version>     # a version npm carries (0.5.0 and later, 0.7.8-dev.17): its published tarballs, integrity-checked, archived (the backfill)
scripts/release.sh verify [<v>] [--core] [--instabox]  # the install test (below) from the archives, host and linux-x64 container; --core for a backfill
scripts/release.sh sign [<v>]         # sign again over archives in place (a thread that has OG_ENGINE_MINISIGN_KEY); the install test's passes stand
scripts/release.sh publish [<v>]      # the platform's upload: PUT SHA256SUMS, SHA256SUMS.minisig, every listed file, then POST publish (OG_PLATFORM_ENGINE_UPLOAD_TOKEN)
scripts/release.sh dev <run-id|dir>   # a dev build (§1a): archived, signed and published on the dev channel; no verify
gh workflow run release.yml --ref main  # a fallback when a machine is unavailable: the same tarballs on GitHub-hosted runners
scripts/release.sh fetch <run-id>     # that run's tarballs into dist/release/<version>/, archived here
```

`pack` holds the gate lock (a release build compiles beside nothing) and
refuses a `native/` that is not release. `publish` is an outward action
and runs only by hand, never from a gate or the wave's end. It refuses a
dirty pack, a missing target, an unsigned `SHA256SUMS` or one that does
not verify with §1's key, archives the install test has not passed on
this machine and in the container (a dev build excepted), and for the
checkout's version a HEAD whose packaged inputs moved since the pack. A
version already published with the same `SHA256SUMS` is reported and
left. The platform verifies the signature and every sha256 again, checks
the manifest (version, channel, each target's url and sha256) and refuses
to overwrite a published version; `latest/` and the installers move only
with a newer release. Every archive is at most 210 MiB (the platform's
request limit; `archive` refuses a larger one).

**A release, step by step:**

1. On the Mac (macrunner, the release worktree at the release commit,
   clean tree): `scripts/release.sh bump <v>`, commit, then `nice -n 10
   scripts/release.sh pack`. It packs the darwin-arm64 and linux-x64
   tarballs, create-game and design into `dist/release/<v>/`, archives
   those two targets and ends by naming the missing win32-x64 tarball.
2. On studio-win (Git Bash, `E:\src\origin-engine` at the same commit):
   `OG_RELEASE=1 sh packages/engine/scripts/build.sh --native && sh
   packages/engine/scripts/pack.sh --native`. The Windows thread sends
   `packages/engine/dist/pack/origingame-engine-win32-x64-<v>.tgz` to the
   Mac thread by Amp's thread file transfer. It never compiles the wasm.
3. On the Mac: copy it into `dist/release/<v>/`, then `scripts/release.sh
   archive` (all three archives, `manifest.json`, `install.sh`,
   `install.ps1`, `SHA256SUMS`; signed when `OG_ENGINE_MINISIGN_KEY` is
   set), `scripts/release.sh verify` (the full install test, not
   `--core`, host and container), then `scripts/release.sh sign` in a
   thread that has the key if this one has not, and `scripts/release.sh
   publish` with `OG_PLATFORM_ENGINE_UPLOAD_TOKEN`.
4. Check the result as an author would: `curl -fsSL
   https://origingame.ai/downloads/engine/install.sh | sh`, `og new demo`,
   `og test .` in the new game; `latest/manifest.json` names `<v>`.

A failure in step 1 or 3's `verify` is fixed on main and packed again;
nothing is uploaded until both platforms' archives pass. The pack runs on
the Amp runners; CI is never a precondition of a release.
`.github/workflows/release.yml` (`workflow_dispatch`, GitHub-hosted
`macos-14`, `windows-latest` and a Linux job in the gate's Debian
container, then `assemble`) does the same pack when a machine is
unavailable, archived unsigned; `scripts/release.sh fetch <run-id>` brings
its artifact in with `verified` cleared, to archive and sign here.

**The packing hosts.** Each keeps a checkout only for packing, with a
target directory that is never cleaned, so a pack reuses every compiled
dependency; sccache serves the rest. The toolchain is the pinned one
(`docs/ci.md` §Toolchain).

- **Mac** (macrunner): the worktree `/Volumes/app/origin-engine-release`
  (branch `release`, tracking origin/main; `git pull` it to the release
  commit), its own `target/`, and the shared sccache in
  `/Volumes/app/.sccache`. `pack` takes only that target's lock
  (`scripts/lock.sh`), so it never waits on a gate in another checkout;
  run it under `nice -n 10`.
- **Windows** (studio-win): `E:\src\origin-engine` on a Dev Drive (ReFS,
  trusted, Defender performance mode), `D:\DevDrive\dev.vhdx` attached at
  boot by the SYSTEM task "Attach Dev Drive E". sccache with
  `SCCACHE_DIR=E:\sccache`. Defender excludes the checkout, `.cargo`,
  `.rustup`, the sccache and bun caches, and cargo, rustc, link and
  sccache. Git Bash, PowerShell 7, Rust (MSVC) and bun are on the PATH of
  the `win` account the Amp runner runs as; restart the runner after
  changing its environment.

The thin-LTO, codegen-units=1 links of `og` and `og-host-node` and
wasm-opt are not cacheable and dominate a warm pack. Two packs of the same
commit give byte-identical tarballs.

**The install test** (`tests/install/`, bun) is what `verify` runs, on
this machine (darwin-arm64) and in a `node:22-bookworm-slim` container
with bun (linux-x64, the executor's platform), from the archives in
`dist/release/<v>/`, served to the launcher as a channel laid out like
`/downloads/engine/`; no npm and no `bun install` anywhere:

1. the launcher's `install.sh` into an empty `ORIGIN_HOME`; `og engine
   install <v>` verifies `SHA256SUMS.minisig` and the archive's sha256 and
   unpacks it into an empty cache; the build record and the layout;
2. `og new demo --engine <v>`: the pin, no `node_modules`, `.og/engine`
   linked to the dist's `engine/`;
3. `og --version` prints the version; `og doctor` passes (the renderer
   line is `none` in the container and not a failure);
4. `og check`, `og test`, `og build --publish` (`og-build.json` names the
   version, the publish wasm is the dist's byte for byte), and `og ui
   check`, `og style check --sizes`, `og frame measure`;
5. a non-default genre/player selection through `og test`;
6. `--view side-2d` writes the same skeleton and passes check/test;
7. the knowledge in the dist's `engine/knowledge/`: `validate.ts`, and
   `mount.ts --package` and `--check` against a stand-in platform tree;
8. an MP3 sound cooks and plays; 9. a built-in UI style from `engine/styles/`.

`--core` (a backfilled version) runs 1 to 5 and 7's validate: what every
version from 0.5.0 has. The install test is also the wave end's `release`
stage (`scripts/wave-end.sh release`: `pack` and `verify`), so the gate
never compiles release (AGENTS.md §Building and testing, profiles) and a
wave ends with a candidate ready.

**In an InstaBox machine** (`tests/install/instabox.ts`; `verify
--instabox`): the same `run.ts`, uploaded into a machine the script
creates from the `agent-browser` template (node 22, git; the script
installs bun 1.4.4 as the container's Dockerfile does, and nothing of the
engine), run there `--host-only` in the background (a foreground command
is capped at ten minutes and not streamed) with its log followed from
here, from the archives (uploaded in 8 MiB parts, the API's file limit
being 10 MiB), the machine deleted after a pass and kept after a failure
to look at. `INSTABOX_API_KEY` is read by the SDK and never reaches the
machine. It records `verified.instabox` in the manifest. It is not a
condition of `publish` (it needs the key and is billed by the minute),
but it is the check of every published version on the machine class the
development threads run on (§4 T).

## 4. What remains before launch

Launch means: the platform's `engine` class runs threads on a released
version in production, new Games are created on it, and one Discover
prototype has been made end to end by a thread, played through `og run
--commands`, delivered and published. Engine feature work and its order
are in `docs/roadmap.md`; this section keeps the release-side stages.

### R. The release pipeline (engine repository)

Done: one version everywhere (`release.sh bump`/`check`, `og`'s pin
notice and doctor line), the knowledge inside the engine package, the
release script and the wave-end `release` stage, the install test (host,
container, InstaBox), `og doctor` outside a checkout, and the archive
channel of §1. The skeleton ships no starter art. Retained test-only assets
live under `tests/fixtures` and `crates/og-tests/assets`, outside release
archives; fixture licence manifests record their provenance.

### L. The local loop on the platform (platform repository)

The platform installs the pinned version (no checkout, no `cargo`, no
token to this repository) and mounts that version's knowledge (L2:
`engine/knowledge/scripts/mount.ts --package <installed engine dir>`,
`--check` in the platform's tests). Open: **L5**, the M7 acceptance
locally: a thread on the local stack makes one Discover prototype end to
end (scaffold, `og run --commands` playtest, delivery, publish to a real
local Portal, played back byte for byte).

### D. The development sandbox on InstaBox (platform repository and engine)

The development stage's threads run in an InstaBox machine
(dashboard.instabox.sh), a provider beside `e2b` routed per class in the
platform's runtime configuration; the engine never tells them apart. The
engine's part is the install test inside such a machine
(`tests/install/instabox.ts`, `release.sh verify --instabox`) and ask 15
of `docs/platform-asks.md`, which lists what the machine must carry. Open:
per-class snapshots of the current pin, and **D5**, L5's prototype made by
a thread in an InstaBox machine.

### F. The features still owed (engine repository)

Feature work and its order: `docs/roadmap.md`.

### T. The test matrix

| Layer | What | Where | When |
|---|---|---|---|
| Crates, packages | fmt, clippy, unit and logic tests, drift, package tests | `scripts/check.sh quick` | every landing |
| GPU frames | gfx references on `metal`, template and package frames | `scripts/check.sh functional`, `kits` | after a batch of landings |
| Wasm | build, wasm-opt, size, determinism across hosts | `scripts/check.sh --all` | every wave end |
| Browser | the WebGPU reference frames, the kit, the a11y mirror | `scripts/check.sh web-reference` | every wave end and before every release |
| Release | pack, the install test from the archives on darwin-arm64 and linux-x64 | `wave-end.sh release` / `release.sh verify` | every wave end; before every publish |
| Windows | native MSVC check, clippy, nextest, DX12 frames against Metal, the install test from the packed tarball | studio-win | on the release commit |
| InstaBox (development) | the install test inside a machine; the platform's snapshot checker | an InstaBox machine (billed by the minute; one machine, §6 item 6) | after every publish; the checker once per pin move |
| Platform, local | the local stack, the runner's `tests/og.rs`, the knowledge mount `--check` | the platform's local stack | every pin move |
| E2B (production only) | `check-templates.ts` on the image that installs the version | E2B (billed) | once per pin move, after D; never for development |
| Devices | iPhone and Android first frame and frame times | the owner's devices | once before launch, then per minor version |

### E. E2B and production (platform and owner; last)

E2B is production's provider only, the last consumer of a version, once
per pin move after D. Open: the `engine`, `probe` and `asset` images
install the pinned version and carry no renderer, no checkout and no Rust
(`engine` image ≤ 600 MB, checked by `check-templates.ts`); the owner
enables them in the runtime configuration; production's first engine
release is verified (`verify-production.sh`, the Player telemetry).

### P. Deferred until the features are complete

Optimisation rows are taken up as the product needs them, measured at a
wave's end, and never hold a merge or a release. Among them: the platform
serves one content-addressed engine wasm URL shared by every game of a
version, so a browser downloads the engine once across games. The
platform's CAS route exists (`server/src/play/serve.ts`); cross-game reuse
of the wasm bytes across game origins, including compressed exports and
browser cache partitioning, still needs implementation or evidence.

## 5. The author's loop

```sh
curl -fsSL https://origingame.ai/downloads/engine/install.sh | sh   # the og launcher, once
og new my-game && cd my-game     # the latest release, or --engine <v>; choices go in project.ogt
og doctor                        # the engine, the addon, the wasm, tsrs, the renderer
og dev .                         # watch, rebuild, re-cook, hot reload through the probe
og test . && og ui check .       # the playtests; the screens measured
og build . --publish             # the play page: dist/ with og-build.json naming the version
```

Inside the platform a thread does the same through the runner's adapter
against the installed engine of the pinned version, with that version's
knowledge mounted. An engine developer alone uses the checkout
(`OG_ENGINE_PACKAGE=packages/engine`, `target/debug/og`), as every test in
this repository does.

## 6. Open with the owner

4. `topdown` in Discover, third-party Godot Web exports, a third native
   tier.
5. The devices for the T matrix's device row.
6. The InstaBox plan: the account is on the free plan (one machine at a
   time, created from the `agent-browser` template or a snapshot, 4 vCPU
   and 8 GiB, auto-paused after thirty idle minutes), and one machine
   stays for the whole verification stage, used serially. More than one
   development thread at once, or one snapshot per class and version,
   needs the Pro plan.
7. Whether E2B stays production's provider once D passes, or InstaBox
   takes production too. Today it is E2B; the engine is indifferent.
