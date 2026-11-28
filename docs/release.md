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
| `<version>/wasm/<variant>/<module>.<hash12>.wasm` | one split engine module (docs/web-export.md §Modules): `<variant>` is the engine wasm it was split from (`publish`, `lanes`; versions before 0.9.x's world3d-on-demand also have `publish2d` and `lanes2d`); the platform's route serves any name matching `wasm/[a-z0-9-]+/[a-z0-9]+.<hash12>.wasm`, the same pattern that gates SHA256SUMS signing, served as `application/wasm` with CORS for the play origin | immutable |
| `<version>/looks/catalogue.json` | the engine package's `knowledge/looks/catalogue.json`, byte for byte (each style's `preview` stays `previews/<slug>.webp`, relative to the version) | immutable |
| `<version>/previews/<slug>.webp`, `<version>/previews/<slug>.<variant>.webp`, `<version>/previews/ui/<pack>.webp` | the creation document's previews, 960 x 540 webp, drawn by this version on the GPU host (below, The looks) | immutable |
| `<version>/creation.json` | the creation document (create-game's `creationDocument`, `packages/create/README.md` §Creation document) with every preview slot filled `{path, url, sha256, size, engine}` | immutable |
| `<version>/manifest.json` | `{version, base?, releasedAt, revision, channel: "release"\|"dev", checksums: "SHA256SUMS", signature: "SHA256SUMS.minisig", targets: {<target>: {url, sha256, size}}, vfx: {<pack>: {url, sha256, size, version, minEngine}}, looks?: {catalogue: {url, sha256, size}, previews: {<slug> \| <slug>.<variant>: {url, sha256, size, engine}}}, creation?: {url, sha256, size}}`; `engine` is the version that drew the preview; a backfill has no `looks` and `creation` | immutable |
| `<version>/SHA256SUMS`, `<version>/SHA256SUMS.minisig` | every archive's sha256 (the targets', the vfx packs' as `vfx/<pack>.tar.gz`, the split engine modules' as `wasm/<variant>/<file>`), the looks files' (`looks/catalogue.json`, `previews/...`, `creation.json`), manifest.json's and the version's installers', signed with the engine's key (below) | immutable |
| `latest/manifest.json` | the newest release (never a dev build) | no-cache |
| `<version>/install.sh`, `<version>/install.ps1` | install the `og` launcher this version was published with, which installs it; a dev build's too (§1a) | immutable |
| `install.sh`, `install.ps1` | install the `og` launcher (`packages/launcher`), the newest release's copies | no-cache |

`<version>` is a release `x.y.z` or a dev build `dev.<YYYYMMDD>.<n>`
(`channel: "dev"`, §1a); the dev builds before 0.9.0, `x.y.z-dev.n` (the
last 0.8.1-dev.100), stay installable as they are. The targets are
`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` and
`x86_64-pc-windows-msvc`, each a `.tar.gz`. `revision` is the full commit
id the version was built from; a dev build `dev.<date>.<n>` also names
`base`, the release it is based on, in manifest.json and in each archive's
`origin-engine.json` `build` record.

Shipping wasm is bindgen output with only the optional `name` custom
section removed. Instructions, imports, exports and other custom sections
stay unchanged; this does not reintroduce wasm-opt. The separate symbols
archive contains the original named modules, at `wasm-symbols/web*/` (the
split modules at `wasm-symbols/web*/modules/`), for offline stack
symbolication by function index. Pack writes
`origingame-engine-wasm-symbols-<version>.tgz` and its `.sha256` checksum.
`scripts/release.sh publish` (a release and a dev build alike) pins it as a
private object of the engine asset store (docs/ci.md §The asset store) in
`dist/release/<version>/symbols/og-assets.json`, pushes and verifies it,
and prints its sha256; it is never in the downloads endpoint, the public
manifest or the signed install sums, and `bun scripts/assets.ts ensure` on
that directory with the store key fetches it. Each version's sha256 is
recorded below (§1 Symbols). Build facts are computed after
removing names. A future public symbols endpoint requires the platform's
download filename allowlist to accept optional artifacts first.

**Optional engine data.** The installed `engine/motion.json` pins motion
weights, fetched only by cooks that use motion. The cache lives under
`ORIGIN_ENGINE_CACHE` (default `~/.cache/origin-engine`), verifies SHA256
before use, never modifies the engine install, and reports missing data
offline as an error. A verified cache works offline. A game without motion
downloads none of it.

`scripts/release.sh publish` verifies the weights in the platform's engine
asset store (docs/ci.md §The asset store) before releasing the engine whose
`motion.json` names the store's `/api/engine-assets/sha256/<hex>` URL. Engines up to 0.9.0 name the frozen
R2 bucket `origin-engine-models` instead. They are content-addressed, not target install
manifest entries. Retain the matching named wasm modules in private symbols
storage even for hosts omitted from default installs.

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

**The vfx section** of `manifest.json` names each downloadable effect
pack of the version: its archive's `url`, `sha256` and `size`, and the
`version` and `minEngine` of the pack's own `extension.json`
(`scripts/release-dist.ts`). Bundled packs are not listed.

**The looks** (docs/art.md §2.6): a version publishes the look catalogue,
the creation document and every preview the document references, so the
platform reads them from `latest/manifest.json` without installing the
package. The previews are never the checkout's files: `scripts/release.sh
previews` draws each slot of the packed create-game's `previewSlots()` with
the version's own linux-x64 og and addon on the GPU host (Colab,
`templates/looks/preview/render.ts --gpu`: a software or absent adapter
fails the step) and records the run in `looks-render.json`. A look or
variant is its preset scene; a UI pack is the components sheet's
hud page dressed in that pack (`og ui preview`, desktop). `release-dist.ts` ships them only
when that record names this version, its commit (with a clean
`templates/looks/`), the `vulkan` tier, the linux-x64 archive's og and
addon and each file's bytes, with a file for every slot and none other;
`publish` refuses a build without them. A create-game from before the
creation document gives one slot per catalogue look and no
`creation.json`.

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
…/install.ps1 | iex`: `~/.origin/bin/og`). It reads only `og-project.json`'s
`engine`, installs that version into the cache when it is missing,
and runs that version's `og` with `OG_ENGINE_PACKAGE` set; `og new`
scaffolds a game (the latest release, or `--engine <v>`), `og upgrade`
moves the pin, and `og engine install <v> | list | path | prune` manages
the cache. Nothing is installed into a game: `og` keeps one link,
`node_modules/@origingame/engine` (ignored by Git), pointing at the engine
it runs with, so TypeScript (`moduleResolution "bundler"`), the bundler and
an editor resolve `@origingame/engine` and its subpaths as a package, by
its `exports`: the engine's package.json is the one map from specifier to
file, and the templates' `tsconfig.json` names no engine path.

**Project identity.** `og-project.json` is an ordinary authored JSON file:
`{version: 1, engine, name, look?}`. These fields live only here, never in
`project.ts`. `og new` writes it; authors and tools edit it; `upgrade`
changes the pin. `check/cook/dev` only read it and never rewrite it. `look`
is a preset string or `{$ogRef: "<module>/<content-id>"}`; name and look
cannot be empty. Unknown fields, invalid types and version other than 1
are ordinary project errors. There are no size or length limits, source
lists, hashes, projection, staleness checks or source-path/symlink rules.
The cook's own dependency tracking and source sandbox remain independent.

The pin is a concrete release `x.y.z` or `dev.YYYYMMDD.n` (n positive),
not latest, a range or a download path. Historical version names remain
installable with explicit `og engine install <v>`. No `projectFormat`
field or pin-to-format mapping is added to the public release manifest.

The sole project marker is `og-project.json`, found at the target directory
or an ancestor. Without one the directory is not a project and `og new`
can create a game. Neither `project.ts` nor `project.ogt` is a marker;
`project.ogt` is ignored even beside new files. A found JSON file with
malformed content or missing fields reports an error, never latest. Missing
TS source is diagnosed only when the engine needs that source; selecting
or installing the JSON pin does not require executing or reading TS.

**The launcher's events.** An install reports its progress, and a tool
(a sandbox that installs the pinned engine on first use) reads it:
`og engine install [<v>|latest] --json` (no version in a project: its
pin) prints JSON Lines on stdout and nothing else; any other command
that has to install its engine first prints the same lines on stdout
when given `--json` (before the command's own output), on stderr when
stderr is not a terminal, and prose on a terminal. In order:

| Event | Fields |
|---|---|
| `resolve` | `version` (the concrete one, `latest` resolved), `target` |
| `download` | `bytes` so far, `total` (`null` when the server names none); every 1 % or 500 ms and at the end |
| `retry` | `attempt` (the one about to start, from 2), `max`, `reason`, `delayMs` |
| `verify` | the archive's sha256 against the signed SHA256SUMS |
| `unpack` | |
| `installed` | `version`, `path` (the dist), `cached` (true: it was installed already, nothing downloaded) |
| `error` | `code`, `message`, `retryable`; the command exits 1 |

`code` is one of `network` (no answer, or an HTTP error other than 404),
`not_found` (no such version, or no archive for this target),
`checksum`, `signature`, `disk_full`, `unpack`, `no_pin` (the project
pins no valid version), `usage`, `unsupported_target`, `internal`, and the
project errors below;
`retryable` is true for `network` and `checksum`. A network failure,
an HTTP 408, 429 or 5xx, a connection idle for 30 s
(`ORIGIN_ENGINE_IDLE_MS`) and a checksum mismatch are retried, 5 attempts
in all (`ORIGIN_ENGINE_RETRIES`), waiting 1, 2, 4 and 8 s
(`ORIGIN_ENGINE_RETRY_MS` the first). The archive downloads to
`tmp/<archive>.part` under one lock per version and resumes from it with a
Range request; a mismatching part is deleted, and nothing reaches
`engines/` before its sha256 verifies. Launcher 2 (`og
--launcher-version`) has the events; launcher 1 reports only an exit
status and prose on stderr. Launcher 3 keeps the event envelope and reads
only the new project metadata.

Project errors are non-retryable and exit 1, without install events or a
latest lookup. `project_metadata_invalid` covers malformed JSON, invalid
types, unknown fields, missing version/name, version other than 1 and
invalid look; `no_pin` covers a missing or invalid engine version.
`OG_ENGINE_VERSION` does not bypass project validation. Explicit
`og engine install <v>` is independent of the current project's files.
There are no special errors or detection for old source formats. Example:

```json
{"event":"error","code":"no_pin","message":"og-project.json: engine must name a concrete engine version","retryable":false}
```

**Symbols.** Each version's symbols archive in the asset store (private,
`<store>/api/engine-assets/sha256/<sha256>`):

| Version | sha256 | Bytes |
|---|---|---|
| dev.20261119.1 | `fff8ca45c74c16497ad720e00656990797304370ff39d41c23839000ddc1842d` | 105,445,575 (no named split modules) |

### 1a. Dev builds while a pin is being tested

A fix the platform needs to try does not wait for a release. It is pushed
to `main` and packed at that commit on the packing hosts as a release is
(§3), with `bun scripts/release-bump.ts --dev <name>` in place of `bump`;
the orb then archives, signs and publishes it on the `dev` channel with
`scripts/release.sh dev <dir> --revision <sha>` (`dev <run-id>` takes the
tarballs of a `release.yml -f dev=true` run instead). The platform then pins
that exact version. Its directory carries the previews `scripts/release.sh
previews` drew for those tarballs on the GPU host (§1 The looks), or the
publish is refused. A dev build runs no install test and no browser tier, and writes
no receipt, tag or changelog entry; `latest` never points at one.
The site's root `install.sh` stays the newest release's, whose launcher
may not know a dev build's name (launcher 1 refuses `dev.<date>.<n>`), so
a dev build carries its own installers, signed with it:
`curl -fsSL https://origingame.ai/downloads/engine/<dev>/install.sh | sh`
(`irm …/<dev>/install.ps1 | iex`) installs the launcher that installs it,
and that launcher installs every earlier version too. A
release (§3) is cut at a milestone, from whatever dev build the testing
settled on, with every gate.

**Its name** is `dev.<YYYYMMDD>.<n>`, `n` counting that UTC day's dev
builds from 1, a number of its own, unrelated to the release numbers
(`scripts/release-bump.ts --next-dev` picks the next one the downloads have
not published; `--dev <name>` writes it in the checkout, nothing of it
committed). The checkout must be at a release, the newest in
`changelog.json`: that is the build's `base`, recorded in its manifest
beside its exact `revision`, and its own changes are the changelog's
`unreleased`. The crates carry it as build metadata of the base
(`0.9.0+dev.20261119.1`, Cargo takes only semver); `og --version` prints
`og dev.20261119.1 (origin-engine dev.20261119.1, based on 0.9.0)`.

**How it orders.** A dev build stands after its base release and before
the next one, by date then `n`; its numbers never compare with a
release's, so it is never newer than a release by number. The launcher
(`og engine list`, `prune`) reads a cached build's `base` from its
`origin-engine.json`. `og new` and `og upgrade` without `--engine`/`--to`
take `latest`, a release; `og upgrade` (the engine's) moves a release pin
onto a dev build only when `--to` names it, and never from a release newer
than its base. A dev pin is compatible (§3) as its base release would be,
with the installed build's `unreleased` counting as its own.

## 2. What the packages carry

`scripts/release.sh pack` builds npm-shaped tarballs in
`dist/release/<version>/` as intermediates; `release-dist.ts` assembles
each target's archive (§1) from them.

| Package | Contents | Built by |
|---|---|---|
| `@origingame/engine` | the TypeScript API and runtime (`src/`, `types/*.d.ts`, `schema/api.schema.json`, `bindings.json`), the web hosts (`dist/web`, `dist/web-publish` with its lanes variant), the player shell, the Basis transcoder, `bin/og.mjs`, `knowledge/` (the generated wiki, the look catalogue, the look presets as og-cook flattened them, the scaffold, `mount.ts` and `validate.ts`) and `styles/` (the built-in UI styles, which og's cook copies into a game's bundle only for the style it names); no native code | `packages/engine/scripts/pack.sh` (`OG_RELEASE=1 build.sh --linux-x64` first), in the orb's `scripts/release.sh pack --target linux-x64` |
| `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64` | `origin-engine.node` (napi addon, `crates/og-host-node`) and `og` (`crates/og-cli`), release profile | `scripts/release.sh pack --target <target>`: linux-x64 in the orb by cargo zigbuild (glibc 2.31), darwin-arm64 natively on a Mac satellite |
| `@origingame/engine-win32-x64` | `origin-engine.node` (MSVC napi addon) and `og.exe`, release profile; the same API, DX12 renderer, tsgo checker fallback | `build.sh --native` then `pack.sh --native` in Git Bash on studio-win (§3) |
| `@origingame/create-game` | `bin/create.ts`, `template/` (the project structure and connected menu/navigation shell) and `film/` (the interactive-film overlay); it reads the engine package beside it: its art brief and creation vocabulary from `knowledge/scripts/{brief,creation}.ts`, the look catalogue from `knowledge/looks/catalogue.json`, built-in screen translations from `src/ui/screens/` and the font packs from `fonts/` | `packages/create` `prepack` |
| `@origingame/design` | `tokens/*.css`, the `tokens.json` generated from them and `design-language.md`, the design language the engine's default UI style is generated from and origin-agent pins | `bun pm pack` of `packages/design` in `scripts/release.sh pack` |

`packages/knowledge` is a private source package of this repository.
linux-arm64 is not shipped. `@tsrs/*` are pinned at
`TSRS_VERSION` of `scripts/install-tools.sh`, not at the engine's version.
The skeleton ships no starter art; test-only assets live under
`tests/fixtures` and `crates/og-tests/assets`, outside release archives.

## 3. Versions and the release procedure

**Version.** The minor number moves only when the owner names a new
minor (`0.10.0`, `1.0.0`); `0.9` is meant to last, and every release after
`0.9.0` is the next patch (`0.9.1`, `0.9.2`, ...). A patch may carry
breaks: an IDL member removed or renamed, a field's type or default
changed, a saved value or enum index whose meaning changed, or behaviour a
game relied on changed. Each break goes into the next patch as a breaking
entry with its `fix`, the `find` identifiers and, when the rewrite is
mechanical, a `codemod`, so that `og upgrade` rewrites what it can and
lists the rest. A patch's `kind` stays `"patch"` and its `breaking` list
says what breaks; a version is compatible exactly when that list is empty.
Dev builds are `dev.<YYYYMMDD>.<n>` (§1a). `scripts/release.sh bump` writes
every occurrence of the version and `scripts/release.sh check` holds them
equal.

A game's `og-project.json` pins its own version, and `og` reports a mismatch
with the installed package (`og doctor` as a line and in `--json` as
`pin`, `og run`/`test`/`build` as a warning); an upgrade is an explicit
edit. A pin is **compatible**, and no mismatch is reported, when both are
the same minor, the installed one is not older, and no version after the
pin up to the installed one (a dev build's `unreleased` counting as its
own) has breaking entries in the installed package's `changelog.json`.
Otherwise `og doctor` names the breaking versions crossed and points to
`og upgrade`.

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
`unreleased` into `versions` as the new dated version, its `kind` from the
numbers (`patch` in the same minor, `minor` otherwise) and its breaking
entries kept whatever the kind. Release notes are generated from
the version's entry into `dist/release/<version>/NOTES.md` by `pack`,
never written by hand.

**`og upgrade`** moves a game to the installed engine (or `--to`, a
version in its changelog): it rewrites `og-project.json`'s `engine` pin,
applies the `codemod` of every breaking entry crossed to `src/` and
`assets/` text, and lists every line where another breaking entry's `find`
identifiers appear (`manual`) and the entries with no identifiers
(`review`). It never downgrades. Its options, exit codes and JSON are in
`crates/og-cli/src/commands/upgrade.rs`.

**`og replay`** is the evidence that a version runs a game the same:
`record` writes a trace of per-tick world hashes and keyframes, and
`compare` replays it with the og at hand and names the first divergent
tick and the fields that differ. The hash covers only the types and fields
the recording saw, so a field a later version adds does not move it. Its
options are in `crates/og-cli/src/commands/replay.rs` and its shapes in
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
the root installers move only with a newer release (each version keeps its
own, §1). Every archive is at most
210 MiB, the platform's request limit.

**A release, step by step:** three hosts pack in parallel. The coordinator orb alone holds the minisign key and publish token; neither is sent to a packing host. macrunner is no longer in the release path.

1. In the orb, bump `<v>`, commit the release inputs and record `git rev-parse HEAD`. Make that exact full commit available on all three hosts. Each host checks it before starting; each pack record names it. `pack` requires a clean tree, checks that tracked inputs did not change during the build, and never archives or publishes.
2. Start these together at that commit:
   - Orb: `scripts/release.sh pack --target linux-x64`. Builds Linux with `cargo zigbuild --target x86_64-unknown-linux-gnu.2.31` (never the orb's glibc), wasm, engine, create-game and design, and release notes. `.agents/setup` installs the pinned zig, cargo-zigbuild, wasm-bindgen and Binaryen tools through `scripts/install-tools.sh`.
   - This thread's Mac satellite: `mac_sidecar start`, then `scripts/release.sh pack --target darwin-arm64`. The mirrored checkout needs Git metadata at the same commit (the mirror does not carry `.git`); seed it from a Git bundle, not from another branch. Native only; no wasm or Linux build.
   - studio-win (Git Bash, `E:\src\origin-engine`): `scripts/release.sh pack --target win32-x64`. It runs the unchanged native build/pack commands (`OG_RELEASE=1 sh packages/engine/scripts/build.sh --native` through `sh packages/engine/scripts/pack.sh --native`), never wasm.
3. Each host uploads its own intermediates directly to private staging: `scripts/release.sh stage push --version <v> --revision <full-sha> --target <target>`. Set `OG_ENGINE_STAGING_URL` to the platform's staging base and `OG_ENGINE_STAGING_TOKEN` to that host's scoped credential. studio-win receives only a staging PUT credential for its version/commit/target, not SSH access, storage credentials or the publish token.
4. In the orb, `scripts/release.sh stage pull --version <v> --revision <full-sha>` downloads all three packs, verifies their SHA-256 hashes and sizes, and rejects another commit. `archive` checks the records and bytes again. The minisign key stays in the orb for archive/sign.
5. `scripts/release.sh previews <v> --check` checks the packed preview slots. Draw the shipping previews on Colab (`colab-gpu`, docs/ci.md §The GPU host) at the same commit using this pack's Linux og/addon; return `previews/` and `looks-render.json` to the orb. Then run `scripts/release.sh archive <v>`, `scripts/release.sh verify <v>` (full install test, not `--core`, host and container), `scripts/release.sh sign <v>`, and only with explicit release approval `scripts/release.sh publish <v>` using `OG_PLATFORM_ENGINE_UPLOAD_TOKEN`. Staging is not publication and never moves `latest`.
6. Check the result as an author would: `curl -fsSL
   https://origingame.ai/downloads/engine/install.sh | sh`, `og new demo`,
   `og test .` in the new game; `latest/manifest.json` names `<v>`.

A failure in `verify` is fixed on main and packed again; nothing is
published until every platform's archive passes. CI is never a
precondition of a release. When a machine is unavailable,
`.github/workflows/release.yml` (`workflow_dispatch`) does the same pack on
GitHub-hosted runners, unsigned, and `scripts/release.sh fetch <run-id>`
brings it into the orb to archive and sign. `scripts/release.sh pack <version>`
archives a version npm already carries from its published tarballs (the
backfill).

The workflow's `dev=true` option makes an uncommitted throwaway bump and uses `--dry-run`; its dirty records cannot be published. A shippable dev version needs its version inputs committed before packing, like a release.

**Private staging contract (platform-owned).** A base URL outside `/downloads/engine/` serves authenticated `PUT` and `GET` at `<base>/<version>/<full-commit>/<target>/<filename>`. Every object is private, including records. The platform validates the version, 40-character lowercase commit, one of the three targets and a basename from that target's allowlist (`scripts/release-stage.ts`, `packFiles`), plus `pack.json`. PUT carries `X-Content-SHA256` and raw bytes (`application/octet-stream`); the server verifies the digest before committing the object. A repeated identical PUT succeeds, a different digest at an existing key returns 409, a missing GET returns 404, invalid credentials return 401/403. No redirects. The server bounds object sizes (at least 210 MiB per tgz), streams to temporary storage, atomically commits, and expires throwaway prefixes by retention policy. No listing, SSH, public URLs, publish, promotion or `latest` mutation is part of this contract.

`pack.json` is the last upload and completion marker: `{schema:1, version, revision, target, dirty, elapsedSeconds, files:{"<filename>":{sha256,size}}}`. The server validates its identity and complete file set against stored objects before accepting it. Linux owns its native tarball, engine/create-game/design tgz files, wasm-symbols tgz and `NOTES.md`; each other host owns only its native tgz. The orb's GET credential can read the exact version/commit prefix; each host's PUT credential is restricted to its own target there. These are separate from `OG_PLATFORM_ENGINE_UPLOAD_TOKEN`. The records stay local/private, never in the public manifest or checksum list. Dirty records propagate to `release.json`, so publication refuses them.

**Signing.** Darwin binaries remain ad-hoc signed (`build.sh`, `codesign -s -`). Developer ID signing and notarization are deferred; no Apple certificate or signing secrets are required. The archive checksums are minisign-signed in the orb as before. The recorded sha256s bind the actual packed bytes.

**Dry run.** Use an unused `dev.<YYYYMMDD>.<n>` only as a throwaway local version (`bun scripts/release-bump.ts --dev <v>` and `cargo update --workspace --offline` on each host), then `pack --target <target> --dry-run`. The uncommitted version bump marks these packs dirty and unpublishable. Run parallel packing, staging PUT/GET, archive and verify; never run `dev` (which publishes), `publish`, or move `latest`. A local authenticated staging stand-in checks the transport contract only, not OVH ingress/storage; repeat the round-trip on the platform endpoint before declaring that integration verified.

**The packing hosts.** Keep one target directory per checkout and do not clean it; sccache serves reusable dependencies. The toolchain is pinned in `docs/ci.md` §Toolchain.

- **Orb**: Linux native, wasm, common packages, final assembly, install verification, minisign and publish.
- **Mac satellite**: darwin-arm64 native packing only. macrunner is a development host, not a release host.
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
og new my-game && cd my-game     # the latest release, or --engine <v>; JSON identity/pin, TS configuration
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
