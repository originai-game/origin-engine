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
the versions already there stay as they are. The pin is a version string.
A version is tested locally first (the install test, then the platform's
local stack), then in the development sandbox (an InstaBox machine), and
installed into E2B, production's provider, last.

**What a version is.** Under `/downloads/engine/`:

| Path | What | Cache |
|---|---|---|
| `<version>/origin-engine-<version>-<target>.tar.gz` | one target's dist, its root directory `origin-engine-<version>-<target>/` | immutable |
| `<version>/vfx/<pack>.tar.gz` | one downloadable effect pack, its root directory `<pack>/` (below) | immutable |
| `<version>/manifest.json` | `{version, releasedAt, revision, channel: "release"\|"dev", checksums: "SHA256SUMS", signature: "SHA256SUMS.minisig", targets: {<target>: {url, sha256, size}}, vfx, ui, fonts, looks}`; the four pack sections are the pack catalogue (below) | immutable |
| `<version>/previews/<kind>/<name>.webp` | one preview per catalogued pack, `<kind>` one of `vfx`, `ui`, `fonts`, `looks`, at most 960×540 | immutable |
| `<version>/SHA256SUMS`, `<version>/SHA256SUMS.minisig` | every archive's sha256 (the targets', the vfx packs' as `vfx/<pack>.tar.gz`), every preview's, manifest.json's and a release's installers', signed with the engine's key (below) | immutable |
| `latest/manifest.json` | the newest release (never a dev build) | no-cache |
| `install.sh`, `install.ps1` | install the `og` launcher (`packages/launcher`) | no-cache |

`<version>` is `x.y.z` or a dev build `x.y.z-dev.n` (`channel: "dev"`).
The targets are `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` and
`x86_64-pc-windows-msvc`, each a `.tar.gz`. `revision` is the commit the
version was built from.

**The dist** is one archive's root, assembled by `scripts/release-dist.ts`
and run unpacked, with no npm and no `bun
install`: `origin-engine.json` with the `build` record, `bin/og`,
`engine/` (the engine package with its native addon; this is
`OG_ENGINE_PACKAGE`), `create/` (the skeleton `og new` writes), `design/`,
and the tsrs checker (none on Windows, where `og check` uses tsgo). The
addon and tsrs keep the node platform names (`linux-x64`), because every
version's loader looks for those; `build.target` is that platform and
`build.triple` the archive's target.

**The downloadable effect packs** (docs/vfx-packs.md §5): every
`packages/vfx-packs/<pack>/` with an `extension.json` is archived
reproducibly as `vfx/<pack>.tar.gz`, listed in SHA256SUMS so the one
signature covers it, and never put in a target archive. `og vfx add
<pack>` installs one after verifying the signed SHA256SUMS as the launcher
does. The few bundled packs ship inside `engine/`.

**The pack catalogue** (origin-agent's Engine page reads it, and nothing
else of the engine tree). `manifest.json` lists every pack the version
offers, bundled or downloadable, in four sections keyed by pack name, one
shape for every kind:

```ts
{ version, minEngine, title, description, tags: string[], size,
  bundled: boolean, url?, sha256?, preview }
```

The sections are `vfx` (effect packs; a downloadable one has `url` and
`sha256`), `ui` (style packs and kits, with `kind: "style" | "kit"`),
`fonts` and `looks` (`knowledge/looks/catalogue.json`). `size` is the
installed bytes and `preview` the absolute https URL of
`<version>/previews/<kind>/<name>.webp`. The other fields come from each
pack's own manifest, never from a second list, and `release-dist.ts`
refuses a pack without them or without a preview. A frozen pack
(AGENTS.md) is listed with the tag `frozen`.

**The release key** is a minisign (Ed25519) pair:

```text
untrusted comment: minisign public key F6CC11AC12997D89
RWSJfZkSrBHM9kOnWQNC0WoGXEFI+9NMCj8+Wy6WJDkYl9pgPGkoCoxt
```

The secret key is the project secret `OG_ENGINE_MINISIGN_KEY` of
`origingame/origin-engine`; signing writes it to a 0600 temporary file and
removes it after, and it is never a file in this repository. The public
key is pinned by the launcher (`packages/launcher/og.ts`), by `og vfx add`
(`packages/engine/src/node/vfx-pack.ts`; a test checks both against this
document), by the platform's upload endpoint and mirror, and by
origin-agent's runner. A new key is a new launcher and a note to both.

**Publishing** is an upload to the platform (`scripts/release.sh
publish`, §3), which verifies the signature and every sha256, serves the
files and refuses to overwrite a version. There is no GitHub release and
no npm publish.

**The cache** every consumer shares is `$ORIGIN_ENGINE_CACHE` (default
`~/.cache/origin-engine`), a version at `engines/<version>/<target>/`
(the archive's root). An install verifies `SHA256SUMS` with the pinned
key and the archive against it, unpacks into a temporary directory and
renames it into place: a version is complete or absent, and nothing
writes into it after. Its files keep the archive's modes and are not made
read-only, because released tools copy files out of the dist with their
modes.

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

A fix the platform needs to try does not wait for a release. It is pushed
to `main`, built by `gh workflow run release.yml --ref main -f dev=true`
(version `<next patch>-dev.<run number>`, nothing of it committed), and
archived, signed and published on the `dev` channel at the run's commit by
`scripts/release.sh dev <run-id|dir>`. The platform then pins that exact
version. A dev build runs no install test and no browser tier, and writes
no receipt, tag or changelog entry; `latest` never points at one. A
release (§3) is cut at a milestone, from whatever dev build the testing
settled on, with every gate.

## 2. What the packages carry

`scripts/release.sh pack` builds npm-shaped tarballs in
`dist/release/<version>/` as intermediates; `release-dist.ts` assembles
each target's archive (§1) from them.

| Package | Contents | Built by |
|---|---|---|
| `@origingame/engine` | the TypeScript API and runtime (`src/`, `types/*.d.ts`, `schema/api.schema.json`, `bindings.json`), the web hosts (`dist/web`, `dist/web-publish`, `dist/web-publish-2d`, each publish directory with its lanes variant), the player shell, the Basis transcoder, `bin/og.mjs`, `knowledge/` (the generated wiki, the look catalogue, the scaffold, `mount.ts` and `validate.ts`) and `styles/` (the built-in UI styles, which og's cook copies into a game's bundle only for the style it names); no native code | `packages/engine/scripts/pack.sh` (`OG_RELEASE=1 build.sh --linux-x64` first), in `scripts/release.sh pack` on macrunner |
| `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64` | `origin-engine.node` (napi addon, `crates/og-host-node`) and `og` (`crates/og-cli`), release profile | both in the Mac's `scripts/release.sh pack`: linux-x64 by cargo zigbuild (glibc 2.31), darwin-arm64 natively |
| `@origingame/engine-win32-x64` | `origin-engine.node` (MSVC napi addon) and `og.exe`, release profile; the same API, DX12 renderer, tsgo checker fallback | `build.sh --native` then `pack.sh --native` in Git Bash on studio-win (§3) |
| `@origingame/create-game` | `bin/create.ts`, `template/` (one empty skeleton), `looks/` (the catalogue), presets, previews, styles and locales | `packages/create` `prepack` |
| `@origingame/design` | `tokens.json` and `design-language.md`, the design language the engine's default UI style is generated from and origin-agent pins | `bun pm pack` of `packages/design` in `scripts/release.sh pack` |

`packages/knowledge` is a private source package of this repository.
linux-arm64 is not shipped (design §12 item 5). `@tsrs/*` are pinned at
`TSRS_VERSION` of `scripts/install-tools.sh`, not at the engine's version.
The skeleton ships no starter art; test-only assets live under
`tests/fixtures` and `crates/og-tests/assets`, outside release archives.

## 3. Versions and the release procedure

**Version.** Before `1.0.0` the minor number counts breaks and the patch
number everything else. A minor bump (`0.x.0`) is a release with a break:
an IDL member removed or renamed, a field's type or default changed, a
saved value or enum index whose meaning changed, or behaviour a game
relied on changed. A patch bump (`0.x.y`) is everything compatible: new
members off by default (a game that does not set them draws and runs
exactly as before), variants appended to an enum, fixes, frames and speed.
A compatible but large release may still take a minor when the owner
names it. `scripts/release.sh bump` writes every occurrence of the version
and `scripts/release.sh check` holds them equal.

A game's `project.ogt` pins its own version, and `og` reports a mismatch
with the installed package (`og doctor` as a line and in `--json` as
`pin`, `og run`/`test`/`build` as a warning); an upgrade is an explicit
edit. A pin is **compatible**, and no mismatch is reported, when both are
the same minor and the installed one is not older.

**`changelog.json`** (in the engine package, schema 1) is the engine's one
changelog: the machine contract the platform builds its "new engine
version" card from, the source of `og upgrade`'s codemods and of each
release's notes. Its shape is `{schema, about, unreleased, versions:
[{version, date, kind: "patch"|"minor", simulation: "unchanged"|"changed",
breaking: [{member, change, fix, find?: [identifier], codemod?: {from,
to}}], added: [{member, summary}], fixed: [{member, summary} | string]}]}`,
newest first. It has one entry per IDL member added, changed or removed.

**Merges only record** (AGENTS.md §Streams and landing): a merge adds one
fragment, `changelog/unreleased/<branch>-<topic>.json`, in the shapes
above, including `simulation: "changed"` when it moves simulation on
purpose (`scripts/changelog.ts` has the details; the quick gate
shape-checks them). `scripts/release.sh fold` folds the fragments into
`unreleased`, refusing a malformed one; the coordinator runs it between
releases and commits the result. `release.sh bump` folds first, moves
`unreleased` into `versions` as the new dated version, and refuses a patch
whose `unreleased` has a breaking entry. Release notes are generated from
the version's entry into `dist/release/<version>/NOTES.md` by `pack`,
never written by hand.

**`og upgrade`** moves a game to the installed engine (or `--to`, a
version in its changelog): it rewrites `project.ogt`'s `engine` pin,
applies the `codemod` of every breaking entry crossed to `src/` and
`assets/` text, and lists every line where another breaking entry's `find`
identifiers appear (`manual`) and the entries with no identifiers
(`review`). It never downgrades. Its options, exit codes and JSON are in
`crates/og-cli/src/upgrade.rs`.

**`og replay`** is the evidence that a version runs a game the same:
`record` writes a trace of per-tick world hashes and keyframes, and
`compare` replays it with the og at hand and names the first divergent
tick and the fields that differ. The hash covers only the types and fields
the recording saw, so a field a later version adds does not move it. Its
options are in `crates/og-cli/src/replay.rs` and its shapes in
`packages/engine/src/node/replay.ts`.

**`scripts/release.sh`** has one verb per step; its header lists them.
`pack` holds the gate lock and refuses a `native/` that is not release.
`publish` is an outward action and runs only by hand, never from a gate.
It refuses a dirty pack, a missing target, an unsigned `SHA256SUMS` or
one that does not verify with §1's key, archives the install test has not
passed on this machine and in the container (a dev build excepted), and
for the checkout's version a HEAD whose packaged inputs moved since the
pack. The platform verifies the signature and every sha256 again, checks
the manifest and refuses to overwrite a published version; `latest/` and
the installers move only with a newer release. Every archive is at most
210 MiB, the platform's request limit.

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
   archive`, `scripts/release.sh verify` (the full install test, not
   `--core`, host and container), then `scripts/release.sh sign` in a
   thread that has the key if this one has not, and `scripts/release.sh
   publish` with `OG_PLATFORM_ENGINE_UPLOAD_TOKEN`.
4. Check the result as an author would: `curl -fsSL
   https://origingame.ai/downloads/engine/install.sh | sh`, `og new demo`,
   `og test .` in the new game; `latest/manifest.json` names `<v>`.

A failure in `verify` is fixed on main and packed again; nothing is
uploaded until every platform's archive passes. CI is never a
precondition of a release. When a machine is unavailable,
`.github/workflows/release.yml` (`workflow_dispatch`) does the same pack on
GitHub-hosted runners, unsigned, and `scripts/release.sh fetch <run-id>`
brings it in to archive and sign here. `scripts/release.sh pack <version>`
archives a version npm already carries from its published tarballs (the
backfill).

**The packing hosts.** Each keeps a checkout only for packing, with a
target directory that is never cleaned, so a pack reuses every compiled
dependency; sccache serves the rest. The toolchain is the pinned one
(`docs/ci.md` §Toolchain). Two packs of the same commit give
byte-identical tarballs.

- **Mac** (macrunner): the worktree `/Volumes/app/origin-engine-release`
  (branch `release`, tracking origin/main; `git pull` it to the release
  commit) with its own `target/`. `pack` takes only that target's lock, so
  it never waits on a gate in another checkout; run it under `nice -n 10`.
- **Windows** (studio-win): `E:\src\origin-engine` on a Dev Drive with
  `SCCACHE_DIR=E:\sccache`, excluded from Defender. Git Bash, PowerShell
  7, Rust (MSVC) and bun are on the PATH of the `win` account the Amp
  runner runs as; restart the runner after changing its environment.

**The install test** (`tests/install/run.ts`, whose header lists its
steps) is what `verify` runs, on this machine and in a linux-x64
container, from the archives served to the launcher as a channel laid out
like `/downloads/engine/`: what an author does on a fresh machine, with no
npm and no `bun install`. `--core` (a backfilled version) runs the steps
every version from 0.5.0 has. `verify --instabox`
(`tests/install/instabox.ts`) runs it once more inside an InstaBox machine
of the class the development threads use; it is not a condition of
`publish` (it needs `INSTABOX_API_KEY` and is billed by the minute), but it
is the check of every published version on that machine class (§4 T).

## 4. What remains before launch

Launch means: the platform's `engine` class runs threads on a released
version in production, new Games are created on it, and one Discover
prototype has been made end to end by a thread, played through `og run
--commands`, delivered and published. Engine feature work and its order
are in `docs/roadmap.md`; this section keeps the release-side stages.

### L. The local loop on the platform (platform repository)

The platform installs the pinned version (no checkout, no `cargo`, no
token to this repository) and mounts that version's knowledge (L2:
`engine/knowledge/scripts/mount.ts --package <installed engine dir>`,
`--check` in the platform's tests). Open: **L5**, the M7 acceptance
locally: a thread on the local stack makes one Discover prototype end to
end (scaffold, `og run --commands` playtest, delivery, publish to a real
local Portal, played back byte for byte).

### D. The development sandbox on InstaBox (platform repository and engine)

The development stage's threads run in an InstaBox machine, a provider
beside `e2b` routed per class in the platform's runtime configuration; the
engine never tells them apart. The engine's part is the install test
inside such a machine and ask 15 of `docs/platform-asks.md`, which lists
what the machine must carry. Open: per-class snapshots of the current pin,
and **D5**, L5's prototype made by a thread in an InstaBox machine.

### T. The test matrix

| Layer | What | Where | When |
|---|---|---|---|
| Crates, packages | fmt, clippy, unit and logic tests, drift, package tests | `scripts/check.sh quick` | every landing |
| GPU frames | gfx references on the GPU host, fixture and package frames | `scripts/check.sh full` on the GPU host | after a batch of landings |
| Wasm | bindgen build and determinism across hosts; shipping-profile build and size check | `scripts/check.sh --all`; `scripts/release-wasm.sh` (also called by pack) | before every release |
| Browser | the WebGPU reference frames, the a11y mirror | `scripts/check.sh web-reference` | before every release |
| Release | pack, the install test from the archives on darwin-arm64 and linux-x64 | `release.sh pack` / `verify` | before every publish |
| Windows | native MSVC check, clippy, nextest, DX12 frames under the cross-platform tolerance, the install test from the packed tarball | studio-win | on the release commit |
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

Optimisations, such as one content-addressed engine wasm URL shared by
every game of a version, are taken up as the product needs them and never
hold a merge or a release.

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
   time, 4 vCPU and 8 GiB), and one machine stays for the whole
   verification stage, used serially. More than one development thread at
   once, or one snapshot per class and version, needs the Pro plan.
7. Whether E2B stays production's provider once D passes, or InstaBox
   takes production too. Today it is E2B; the engine is indifferent.
