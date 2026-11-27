# Web export

`og build` turns a project into a directory any static host serves: the
player shell, the engine (wasm in a Worker on an OffscreenCanvas), the game
bundle and its cooked assets. This page is the output layout, what a host
must send, the shell's behaviour, the contract the platform's Player and
Stage rely on, and how the package is packed. Design: `docs/design.md`
§2.2, §4, §6, §9.

```sh
og build .              # dev build: every probe member, no service worker
og build . --publish    # what ships: the player probe members, minified, the PWA
```

## Output layout

The plain names below are a dev build's. A publish build writes the
bundles, the engine modules and the decoders under content-hashed names
only (§Content-hashed names), listed in og-build.json.

| Path | What it is |
|---|---|
| `index.html` | The player shell's page. Carries `<meta name="og-engine" content="<engine version>">`, `og-build` (`publish`/`dev`), `og-profile` and `og-player` (`frame page`: the Player transports the shell speaks, which the platform reads to open the page at the top level), and the page configuration (`<script type="application/json" id="og-config">`, data that runs nothing): no inline script or style (§Hosting). |
| `shell.css` | The shell's CSS: the brand tokens, its motions and its rules. |
| `install.js` | `--publish` only: the head's beforeinstallprompt hook, which runs before the shell's module. |
| `play.js` | The shell (`packages/engine/shell/page.ts`, bundled). |
| `game.js` | The engine Worker: the runtime and the game. |
| `audio-worklet.js` | The AudioWorklet, loaded on the first gesture. |
| `audio-decoder.js` | The audio decoder's Worker, started by the engine Worker once audio starts. |
| `game.js.map`, `play.js.map`, `audio-worklet.js.map` | A dev build's source maps, beside their bundles and linked from them (§Source maps). |
| `sourcemaps/` | A publish build's source maps of the same three bundles, unlinked and not to be served (§Source maps). |
| `origin-engine.js`, `origin-engine_bg.wasm` | The web host from wasm-bindgen: `dist/web` in a dev build, the publish variant (`dist/web-publish`, the same for every game, 2D or 3D) with `--publish` (§Variants). |
| `origin-engine-lanes.js`, `origin-engine-lanes_bg.wasm`, `snippets/` | `--publish` with `isolation` not `off`: the lanes variant and wasm-bindgen-rayon's Worker script (§Lanes). |
| `basis/` | Binomial's Basis transcoder and its licence: a dev build, and a publish build whose cooked bundle holds a texture, texture array or lightmap, or that names a content base or opens a bundle at run time (og.load.bundle). Without it the page's configuration says `noBasis`. |
| `av1/` | The AV1 fallback decoder (`og-av1.wasm`) and its Worker (`av1-worker.js`), fetched only by a browser without AV1 in WebCodecs: a build with a video, one whose code references the `video` module, or one that names a content base or opens a bundle at run time. Without it the page's configuration says `noAv1`. |
| `cooked/` | The cooked bundle, split by streaming group: the first group and one pack per streaming group, `cooked/stream/<name>.ogb` (§Streaming groups). |
| `icons/` | `publish.icon`, or the brand tile at 192 and 512 px. |
| `og-build.json` | The build manifest (below). |
| `_headers` | For static hosts (Netlify and Cloudflare Pages format): the headers of §Hosting. |
| `origingame.runtime.json` | `{"version":1,"crossOriginIsolated":true}` when `isolation` is not `off`: the platform's per-release isolation declaration. |
| `manifest.webmanifest`, `sw.js` | `--publish` only: the PWA manifest and the service worker. |

The same inputs give the same bytes wherever the directory is written: no
dates, no absolute paths, files in sorted order
(`packages/engine/tests/web-export.test.ts`).

### The manifest: `og-build.json`

```json
{
  "format": 1,
  "engine": { "name": "origin-engine", "version": "0.0.1", "pinned": "0.0.1" },
  "build": "publish",
  "game": { "name": "Runner", "id": "runner", "profile": "2d" },
  "entry": "index.html",
  "worker": "game.3f2a9c1e04b7.js",
  "wasm": "origin-engine_bg.9b0c51d2e7a4.wasm",
  "variant": "publish",
  "names": { "game.js": "game.3f2a9c1e04b7.js", "play.js": "play.81aa0f3c6d25.js", "…": "…" },
  "probe": { "protocol": 5, "wire": 1, "build": "publish",
             "members": ["hello", "tick", "state", "pick", "entity", "input", "capture"] },
  "crossOriginIsolation": "preferred",
  "assets": null,
  "content": null,
  "display": { "fit": "fill", "aspect": null, "aspectMin": null, "aspectMax": null,
               "orientation": "any", "dprCap": 2,
               "fullscreen": "off", "gate": "tap", "madeWith": true, "loading": "engine" },
  "icons": [{ "src": "icons/icon-192.png", "sizes": "192x192", "type": "image/png", "source": "brand" },
            { "src": "icons/icon-512.png", "sizes": "512x512", "type": "image/png", "source": "brand" }],
  "pwa": { "manifest": "manifest.webmanifest", "serviceWorker": "sw.js" },
  "sourcemaps": { "build": "<16 hex>", "served": false,
                  "maps": [{ "bundle": "game.js", "file": "game.3f2a9c1e04b7.js",
                             "map": "sourcemaps/game.js.map", "sha256": "…" }] },
  "player": { "protocol": 2, "transports": ["frame", "page"] },
  "version": "<sha256 over every file's path and sha256>",
  "files": [{ "path": "index.html", "bytes": 12519, "sha256": "…" }],
  "total": { "files": 45, "bytes": 12477811 },
  "stream": null
}
```

- `worker`, `wasm` and `lanes` are the served names; `names` maps each
  content-hashed file's own name to its served one (`{}` in a dev build).
- `engine.version` is the engine that built it; `engine.pinned` is
  project.ogt's `engine`. `probe` is read from the copied wasm's own
  `hello` (`build` `publish`, or `release` for a dev build).
- `variant` is `publish`, or `full` for a dev build (§Variants); the game's profile is `game.profile`.
- `assets` is `null`, or `{ "base", "origin" }` for a build whose package
  is served from an asset base (§Asset base).
- `content` is `null`, or `{ "base", "origin" }` for a build whose
  og.load.data reads catalogs from a content base (§Content base).
- `crossOriginIsolation`: `required`, `preferred` (the default) or `off`;
  without isolation a page still starts, single-threaded.
- `files` lists every file beside it (not itself) with bytes and sha256;
  `version` is the sha256 over the path and sha256 of each of them except
  `sw.js`.
- `sourcemaps.build` is the build id: the first 16 hex digits of the sha256
  over the three bundles' paths and hashes. The page's configuration
  carries it as `buildId` and a crash report as `build` (§Telemetry).
  A map's `file` is its bundle's served name. `served` is `true` for a dev
  build, `false` for a publish build.
- `player` is the Player protocol the shell speaks and its transports:
  `frame` (`postMessage` to a framing Player) and `page` (same-origin HTTP
  at the top level; §The top-level page).
- `stream` is `null` for a bundle without streaming groups, else `{
  "first": { "files", "bytes" }, "groups": [{ "name", "file", "bytes",
  "sha256", "priority", "memory", "assets" }] }`: `first` is what the boot
  fetches and the service worker precaches, and each group is its pack
  under `cooked/stream/` with the memory its assets take once loaded and
  its asset ids.
- `pruning` is `null` in a dev build. A publish build lists `omitted`
  outputs (`id`, `file`, `bytes`; an omitted id can share a file with a
  retained one, so the bytes are not savings), `removed_files` relative to
  `cooked/`, and `computed` loads (`source`, `line`, `prefix`, `reason`).
  The rules are the `assets` guide's (`og api show assets`).
- `delivery` lists runtime bundle directories, their files relative to `cooked/`
  and whether they belong to the base boot dependency closure. Cook build
  intermediates and duplicate payloads are excluded from the export. Lazy
  delivery files are excluded from boot progress and service-worker precaching.
  Streaming groups in every delivery bundle appear under `stream.groups`, with
  their full served path and pack-qualified name.

### Delivery bundles

`project.ts.packs` partitions the cook into ordinary runtime bundles. The base
manifest is `cooked/.cook.ogt`; each other manifest is
`cooked/delivery/<id>/.cook.ogt`. A manifest's `requires` entries name dependency
manifests relative to its directory, with byte size, content hash and SHA-384
integrity. Boot preloads the base dependency closure once and checks the pins.
Later `og.load.bundle` and catalog loads use the same dependency loader. A bundle
is not ready until its dependencies are ready; conflicts and cycles fail. An
open dependent prevents releasing its required bundle. Asset IDs must be unique
across open bundles, regardless of load order.

Cross-pack validation follows the cook's conservative reference index. A
declared reference to another delivered output requires a `dependsOn` path even
when its cooker embeds that output's data. Source-only inputs without delivered
outputs do not create delivery edges.

### Streaming groups

The Worker fetches the first group (the cook's manifest and every output no
group holds) before the engine starts, and a group's pack only when the
runtime admits the group (IDL module `stream`), transcoding its textures
first so the whole group goes in place at once. The transcoding is done by
other Workers (copies of the engine's, started with the first Basis texture
and kept: one more while all are busy, up to four), so that no frame waits
on it; where none starts, the engine's Worker transcodes them itself. A
chunked sound's later chunks are fetched ahead of its playheads.

## Hosting

- Serve the directory as is, under any path: every URL in it is relative.
- `.wasm` as `application/wasm`; `manifest.webmanifest` as
  `application/manifest+json`; `X-Content-Type-Options: nosniff` and
  `Cross-Origin-Resource-Policy: cross-origin` on everything.
- Isolation (`required`/`preferred`): `Cross-Origin-Opener-Policy:
  same-origin` and `Cross-Origin-Embedder-Policy: require-corp` on the page
  and its files. A page framed by another origin also needs
  `allow="cross-origin-isolated"` on the iframe. Without isolation
  everything still works, single-threaded and over `postMessage` instead
  of the SharedArrayBuffer rings.
- A Content-Security-Policy as strict as this serves the page whole, lanes
  included:

  ```text
  default-src 'self'; script-src 'self' 'wasm-unsafe-eval'
  ```

  The page has no inline script or style, and every Worker (the engine's,
  the lanes pool's, the audio and AV1 decoders', and the Workers that
  transcode streamed Basis textures, which are copies of the engine's)
  starts from a file on the page's origin, never a `blob:` URL. The Basis
  transcoder is imported as
  a module (`basis/basis_transcoder.js`), never evaluated from text, so the
  policy needs no `'unsafe-eval'`. Add to it:
  - the content base (og.load.data) and an asset base (§Asset base) to
    `connect-src`, beside `'self'`;
  - `img-src blob:` for SVG images in UI documents, which the page decodes
    from a `blob:` URL; without it they cannot be drawn.

  The lanes pool's helper starts its Workers from a `blob:` copy only when
  it is served from another origin than the page, which `og build` never
  does (scripts/rayon-worker.sh).
- `Cache-Control: no-cache` on `/`, `index.html`, `sw.js` and
  `og-build.json`; `public, max-age=31536000, immutable` on the
  content-hashed files.
- Never serve `sourcemaps/` (§Source maps).

`_headers` carries all of these.

### Asset base

A build can boot its package from another origin, a CDN, instead of the
page's directory: `og build --assets <url>`, else project.ogt `publish {
assets "<url>" }`. The URL is the directory holding `cooked/`: absolute
`https://` (`http://` only on localhost), without credentials, query or
fragment (`crates/og-cli/src/publish.rs` `asset_base`).

- **The cook** writes each output's, streaming pack's and og.motion
  model's Subresource Integrity, `integrity "sha384-…"`, in
  `cooked/.cook.ogt` beside its FNV hash.
- **The build** writes the page's configuration `assets: { base,
  manifest }` (the manifest's own integrity) and og-build.json's `assets:
  { base, origin }`, and points the service worker's `cooked/` files at the
  base. `cooked/` is still written beside the page; everything else (the
  page, the engine, the code) is served next to the page as before.
- **The platform** uploads `cooked/` to `<base>/cooked/`, every file under
  its own name, answering CORS reads from the page's origin
  (`Access-Control-Allow-Origin`, and `Cross-Origin-Resource-Policy:
  cross-origin` under isolation), and lets the page's
  Content-Security-Policy reach `origin` (`connect-src`). A release's base
  should be immutable: the page pins its manifest's hash.
- **The loader** (src/web/boot.ts) fetches the manifest against the
  configuration's integrity, then every file it or the engine fetches
  later (the first group, packs, sound and video chunks, `#mips`, font
  faces) against the manifest's; the browser refuses bytes that differ,
  and the file fails once, without retries, its load failure naming its
  integrity. A changed manifest fails the boot.
- The cook's optional top-level `preload "<file>"` entries put explicitly
  reachable font-pack faces in the boot, before GameDesc.start. The resource
  graph follows code and asset dependencies but not a font-pack index's
  optional coverage edges; computed paths conservatively include their
  prefix (or every candidate when no prefix is known). These faces count
  in boot bytes and the service-worker precache. Other faces remain lazy
  for og.i18n; the existing glyph readiness gate and drawing are unchanged.
  This is the same index and roots as publish pruning, with only font-pack
  coverage edges stopped for the boot. No size threshold silently drops
  candidates: `og-build.json`'s optional `fontPreloads` lists each path and
  raw bytes; `og build`'s text and JSON size reports also give its compressed
  bytes. A loader supplied as runtime code needs explicit
  publish.keep declarations; undeclared synchronous pack-face loads are
  refused on native as well as web, independent of residency.
  These are cooked output paths with the outputs' bytes and integrity,
  added to the existing `preloadBundle` download queue and awaited before
  engine creation. They are not the `modules` engine-code hints (§Modules):
  `world::needs` requests module names after a bundle opens or a scene
  loads, through a best-effort callback that does not wait. A missed hint
  can load code at its first call; a synchronous font load cannot wait for
  missing bytes. The two typed lists therefore remain separate; there is
  no second font fetcher, readiness gate or completion mechanism.
- Natively (`og run`, `og test`) the package is the project's `cooked/`;
  the asset base is the web export's.

`tests/web/asset-base.ts` boots a fixture's package from another origin in
Chrome, intact, with one byte changed and with its manifest changed.

### Content base

og.load.data reads a game's content catalogs (the og-api guide loading
§14) from a content base, which is not the asset base: `og build
--content <url>`, else project.ogt `publish { content "<url>" }`, under
the asset base's rules for the URL. The build writes the page's
configuration `content` and og-build.json's `content: { base, origin }`;
the Worker reads `<base>/<catalog>/<version>/manifest.json` (against the
integrity a `catalog.head` answer named, once one has) and then each file
against the integrity the manifest lists, checking SHA-384 itself so a
network or CORS failure stays `failed`. `<base>/<catalog>/head.json`
(served `no-store`) answers `catalog.head` on a page without a Player
(§Providers). The host serves the base
with CORS reads from the page's origin (and `Cross-Origin-Resource-Policy:
cross-origin` under isolation) and lets the Content-Security-Policy reach
`origin`. The service worker does not cache it: the browser's HTTP cache
keeps blobs, whose URLs are their digests. Natively the base is `og run
--content <dir>`.

### Content-hashed names

A publish build serves every file the page or the engine Worker fetches by
name as `<stem>.<the first 12 hex digits of its sha256>.<ext>`
(`game.3f2a9c1e04b7.js`): `play.js`, `shell.css`, `install.js`,
`game.js`, `audio-worklet.js`, `audio-decoder.js`, the engine glue and
modules (single-threaded and lanes), `basis/basis_transcoder.{js,wasm}` and
the AV1 fallback decoder `av1/og-av1.wasm` and its Worker
`av1/av1-worker.js` (`HASHED_FILES` in `src/node/web-build.ts`). A new build
changes the URL of what changed and nothing else. The page finds them
through its configuration's `files` (og-build.json's `names`); a host or
tool finds them through og-build.json (`worker`, `wasm`, `lanes`, `names`),
never by a plain name.

The entry points (`index.html`, `sw.js`, `og-build.json`,
`manifest.webmanifest`), the icons, `snippets/` and `cooked/` keep their
names; the service worker carries an unchanged cooked file over by its
content instead (§Incremental update). A dev build keeps every plain name.

## Lanes

Rayon runs the engine's data-parallel work on a pool of threads
(`crates/og-lanes`, whose README lists the sites and their thresholds):
one a core up to 8 natively, Workers sharing the wasm memory on the web.
Every lane writes back in input order and TS systems stay on the engine
thread, so the world hash, captures and replays are identical with lanes
on and off (`packages/engine/tests/lanes.test.ts`).

A wasm module built with atomics needs shared memory, which needs
cross-origin isolation, so there are two modules: `origin-engine_bg.wasm`
stays single-threaded, and a `--publish` build whose `isolation` is not
`off` also ships `origin-engine-lanes_bg.wasm` (built by
`scripts/wasm-lanes.sh`). A page downloads one of them. A `--publish`
build that finds the lanes files missing warns once and ships
single-threaded; under the gate (`OG_GATE=1`) `scripts/build.sh` stops
instead.

The shell (`src/web/lanes.ts` `laneChoice`) takes the lanes variant when
the build offers it, `crossOriginIsolated` is true and
`navigator.hardwareConcurrency` is above 2, with `min(cores - 1, 8)`
Workers. If the lanes module or the pool fails to start (an `error` from a
pool Worker, or `LANES_POOL_DEADLINE_MS`, 10 s), the Worker starts the
single-threaded module instead, so a page always plays. A panic in a lane
is recorded and reported as the engine's crash (§Errors) within a frame or
two. The mode is the ready message's `lanes` (`{ on, threads, why }`) and
`__runtime_probe__.lanes`. The service worker precaches the variant
`laneChoice` picks in its scope and caches the booted one once the page
posts `{ type: "og-cache", files }`. Natively, `OG_LANES` sets the pool
(`off`, `0` or `1` serial, `N` threads, unset the cores up to 8).

## Variants

One engine module per download. Every publish build ships the same
publish module, single-threaded and lanes, whatever the game's profile:
its capabilities are split modules a page fetches on first use (§Modules),
so a 2D game downloads no 3D world and a 3D game or a film can hold a 2D
minigame. The cook records the profile in cooked/.cook.ogt (`profile
"3d"` or `"2d"`, from project.ogt's `create { form }`: og-bundle's
`look::Profile`) as the project's default, which picks the physics
simulation a world starts with; it walls nothing off:

| Variant | Where | Built with (og-host-web) | Physics |
|---|---|---|---|
| publish | `dist/web-publish/origin-engine*` | `--no-default-features --features publish` | both |
| lanes | `dist/web-publish/origin-engine-lanes*`, `snippets/` | `--no-default-features --features publish`, wasm atomics (§Lanes) | both (`parallel`) |
| dev | `dist/web/origin-engine*` | default features | both, not shipped |

Every variant links rapier2d, rapier3d and the 3D world, so a world
switches `og.Physics.profile` at run time (a 2D minigame inside a 3D game). The 2D
simulation is the split module `physics2d` and the 3D one `physics3d`
(§Modules): a page fetches each on its first call.

The feature sets are scripts/wasm-crates.txt's, the gate's own build list
(scripts/lib.sh reads it); `scripts/release-dist.test.ts` holds the table's
sets equal to its `variant` and `lanes` lines for `og-host-web`.

`publish` leaves out the probe's tool members, GPU timestamp queries and
the reflected field docs. The variant is copied as `origin-engine.js` and
`origin-engine_bg.wasm` and recorded as og-build.json's `variant`
(`publish`, or `full` for a dev build). The size budget (design §7,
`scripts/budgets.txt`) holds per variant: what counts is the largest one a
player downloads, never the sum
(`scripts/size-gate.sh`, docs/ci.md).

## Modules

Each publish module, single-threaded and lanes, is split (Binaryen's `wasm-split`, pinned by
`scripts/install-tools.sh`; `packages/engine/scripts/wasm-split.ts`) into
the primary, which boots, and one module per optional capability:
`physics` with its two simulations `physics3d` (rapier3d) and `physics2d`
(rapier2d), `story` (with og.flow), `nav` (with og.ai), `motion`,
`destruction2d` (og.PixelTerrain and og.destruction's `…2d` calls) and
`destruction3d` (og.Destructible and the rest of og.destruction), `video`,
`shape` (og.procgen and og.render's modelled geometries: shape, extrude,
lathe, the polyhedra, sweep, csg, mergeGeometries), `tilemap`,
`timeline`, `net` (lockstep rooms), `shared` (og.shared), `world3d`
(terrain, scattered copies and grass, planar water, the 3D fluids and
volumetric fog's shaders, with og.terrain), `render3d` (the probe GI, the
vegetation's wind field and trample map, the crowds' impostor bake) and
`smaa` (`scripts/wasm-modules.json` assigns the functions). A world loads `physics` and the simulation of its
og.Physics.profile; switching the profile loads the other one. What a game using none of them runs stays in the primary,
so such a game loads none: every function its ticks, snapshots and
restores run (profiled by `scripts/wasm-profile.ts`), and every function
a frame, a canvas attach, a primitive geometry call or the rooms' and
og.shared's host side (the ops the Worker takes after every frame, the
room host and session provider every page names) reaches by direct calls
(`roots`, from Binaryen's call graph), whatever its name, up to the
`entries`: the engine's calls into a module that run only once the game
uses it (og.motion once og.MotionAgent is registered, a dynamic geometry's
update, the physics debug view, a playing video's stream, the terrain's
extraction once a terrain is loaded or an og.Terrain placed). og-render
makes each 3D world renderer when a frame first draws what it draws
(`renderer/resources.rs` `OnDemand`), so a frame without terrain,
scattered copies, grass, water or 3D fluids calls into none of them; its
`render3d` calls are guarded entries, made the first frame a look enables
the probe GI, wind or a motor drives the field, a pusher presses grass or
a crowd draws cards. The roots' graph does not follow the frame into
og-render (it is drawn behind og-runtime's `RenderBackend`), so a module's
patterns leave out what every frame calls there (the 2D water, the
terrains' effect group). A generated
geometry is not handed to physics until a collider names it, and a
world with nothing to simulate builds no rapier world until its first
body or collider (its snapshot carries none). A generic instance of std,
the serializers or og-lanes goes with its type arguments' module
(bincode's reader of a rapier2d body is physics2d's). `siblings` are
modules built from one source per variant (physics2d and physics3d, from
og-physics' rigid/sim.rs): the linker folds their identical bodies under
one name, so a function one sibling calls in the other goes to their
parent, `physics`. The split fails when an entry names no function any
more. A lanes module's
modules sit in `modules-lanes/` beside it. Each lanes pool Worker
instantiates the primary again on the shared memory, with the same
placeholders (`scripts/og-pool-split.js`, which `scripts/rayon-worker.sh`
wires into wasm-bindgen-rayon's Worker helper): a lane that first calls a
module loads it into its own Worker's instance (a blocking request, from
the HTTP cache once the engine Worker fetched it). The remote-frame twins
ship whole.

A split module carries its functions and the read-only data only they
read: its tables, strings and shader text
(`scripts/wasm-split-data.ts`). A publish module is linked with
`--emit-relocs` (`crates/og-host-web/build.rs`), and the relocations of
the raw module say which function and which data refer to which data
symbol; a `.rodata` symbol whose every referrer, through any chain of
data, is one module's function moves to that module, and one the primary
or two modules reach stays. The primary keeps those bytes zeroed, and
the module writes them back from an active data segment when it is
instantiated, before any of its functions runs. The bound module ships
without the relocations. The raw module's names are matched to
wasm-bindgen's by rustc-demangle (`rustfilt -h`, pinned by
`scripts/install-tools.sh`); a function it does not match counts as the
primary's, which keeps its data. `modules.json` lists each module's
`data` bytes. A lanes pool Worker that loads a module again writes the
same bytes over the shared memory, which changes nothing a thread reads.

The primary imports a placeholder for each split function
(`placeholder.<module>`); `loadWebHost`'s optional fourth argument, a
`ModuleHost` (`src/web/split.ts`), instantiates a module against the
primary's exports, and the module writes its functions into the
primary's table. A placeholder called before its module arrived loads it
synchronously (the tick waits) and calls through, so a module that loads
late changes no result: the world hash is the unsplit module's whenever
it arrives (`tests/hosts.test.ts`).

**Where they come from.** A release publishes each module once per engine
version, content-addressed and immutable, at
`https://origingame.ai/downloads/engine/<version>/wasm/<variant>/<module>.<hash12>.wasm`
(`application/wasm`, `Access-Control-Allow-Origin: *`,
`Cross-Origin-Resource-Policy: cross-origin`), listed in SHA256SUMS
(`scripts/release-dist.ts`, docs/release.md §1). `og build --publish`
records in og-build.json's `modules` the ones the game references (those
its boot scene needs, as the cook recorded them, and those whose IDL
module's members its code names), their `base` and the `origin` the
page's Content-Security-Policy must allow in `connect-src`; the Worker
fetches the referenced ones in CORS mode beside the primary during the
loading screen and puts them in place before the first tick. `og.destruction`
references `destruction2d` for its `…2d` calls and `destruction3d` for
the others; a `physics` reference, and a `destruction` one named by no
call or record, takes the project's profile. An asynchronous fetch (a
referenced module, a prefetch as content is prepared) carries the
module's SHA-256 as its Subresource Integrity (the engine's `modules.json`
for a CDN build, the copied file's for a local one), so the browser
refuses other bytes; a module a call loads synchronously and a lanes pool
Worker's own fetch are not checked.

**What content needs.** The cook records in cooked/.cook.ogt the modules
each asset needs, `modules nav physics` in its block (sorted, absent when
none): its own records' (the IDL's `split` line on a module or a record:
`RigidBody` is in `physics`, which adds the simulation of the scene's
`resource og.Physics { profile }`, else of the project's profile), its
kind's (a dialogue `story`, a video `video`, a navmesh `nav`), and those of every asset it places or plays (a
scene's sub-scenes, prefabs and models, a timeline's clips and scenes).
A top-level `modules` line is the bundle's: its boot scene's (project.ogt
`scene` through `scenes`), or every asset's when project.ogt names none.
The engine asks its host for them as content is prepared (og-runtime's
`world::needs`, the `prefetch_modules` hook): og.load.bundle as the bundle
opens; og.load.scene, og.scene.preload, og.scene.change and a sub-scene
as the scene loads. The rest load on their first call, which is the
fallback, never a different result.

`og build --publish --modules local` copies every module into the
output's `modules/` and fetches
them from beside the page, for an offline or self-hosted build; an engine
built in a checkout, which no CDN serves, does so by default. Its service
worker precaches every module of the page's variant (§The shell, PWA), so an
offline page has each one its game may call. A CDN build's precaches the
referenced ones under their CDN URL and caches the rest on their first
fetch. `og build`'s `downloads` count a build's own modules: the
referenced ones in each page's boot `engine`, the single-threaded page's
others in `later.modules`; a CDN build's are no file of it.
`modules` is null for an unsplit engine.

## Errors

A failure reaches the page as one structured host error: the Worker's
`crash` (or `error` before ready) message carries it as `error`, `Shell`
passes it to `onCrash(message, error)` and keeps it as `shell.error`, and
the page puts it on `__runtime_probe__.error` (null while the engine runs)
and on the error card:

```ts
{ kind: "panic" | "error" | "hang", message, tick, lane, location }
```

- `panic`: a Rust panic, recorded by the web host's panic hook before the
  wasm traps: its message, `file:line:column`, the tick, and the lanes site
  when it happened inside a lane (else `lane: null`).
- `error`: a failing system (its message names the tick and the system), a
  trap without a recorded panic, an uncaught error in the Worker; `tick` is
  the last tick a frame reached.
- `hang`: a visible, playing page that hears no `beat` from the Worker for
  `FRAME_DEADLINE_MS` (120 s) terminates it and reports a hang.

After any of them every pending and later probe call or `call` is refused
at once ("the engine stopped: …").

## Source maps

Every bundle `og build` makes has its source map. Its `sources` name the
game's files relative to the project (`src/main.ts`) and the engine's as
`@origingame/engine/<path>`, never a path of the building machine, with
each source's text inside; the bytes depend only on the inputs.

- **Dev build.** The maps beside their bundles, linked with
  `//# sourceMappingURL`; the error card maps its stack before drawing it.
- **Publish build.** The same maps under `sourcemaps/`, unlinked, not in
  og-build.json's `files` and `version` nor the service worker's cache. The
  platform takes `sourcemaps/` out before it serves the release and stores
  it privately under `sourcemaps.build`. A host that serves the directory
  as is must leave `sourcemaps/` out.
- **Native runs** bundle the game with its map
  (`.og/game.native.js.map`), so a native error names the `.ts` line.
- **Mapping a stored stack.** `@origingame/engine/sourcemap` has no I/O and
  no dependencies, and runs under bun, node, a browser and a Worker:

  ```ts
  import { mapStack } from "@origingame/engine/sourcemap";
  const text = mapStack(report.stack.join("\n"), { "game.js": gameMap, "play.js": playMap });
  // "at drawWater (game.js:1:48213)" → "at drawWater (src/main.ts:10:31)"
  ```

  `mapStack(stack, maps)` rewrites each frame (V8's `at f (url:l:c)`,
  JavaScriptCore's and SpiderMonkey's `f@url:l:c`) whose file has a map and
  leaves every other line as it was. Maps are looked up by the frame's file
  name (a frame in `game.<hash>.js` takes the map given as `game.js`), and
  `maps` may also be a function from a frame's location to its map.
  `SourceMap` answers `original(line, column)`, and `stackFrames(text)`
  turns an error's text into report frames. These signatures are a
  contract with the platform: a change to them is a break.

## Telemetry

Field telemetry is what the shell tells the Player about a session beyond
its lifecycle. It rides protocol 2's `crash` and `sample` on both
transports. `publish { telemetry off }` measures and sends none of it:
only `start`, `ready`, `fail`, the boot `sample` with `renderer` and
`timings`, `crash` with `name` and `fingerprint`, `heartbeat` and `end`
go.

The GPU reported is the device the engine renders with; the page never asks
for an adapter of its own. Nothing personal is sent: a browser is its
family, major version and OS family (never the user-agent string); a
device is its class and the figures any
page reads (`hardwareConcurrency`, `deviceMemory`, DPR to a quarter); a GPU
is its `vendor`, `architecture`, limits and features (never its
`description` or driver); a stack is its frames with each location cut to
the file name (never an origin, path, query or the error's message); a
load failure is a path under `cooked/` and a reason code. No identifier is
added: `session` and `device` are the transport's.

The page sends a `sample` at most once a minute while playing and when it
is hidden (at least 10 s after the last), never more than 60 a session,
with at most 20 load failures a sample and 100 a session.

| Message | Fields added (all optional to the play origin) |
|---|---|
| `crash` | `cause` (`panic`, `error`, `hang`), `tick` (the world's, or null), `lane` (a panic's lanes site), `location` (a panic's `crates/…:line:col`), `stack` (string[], at most 48 frames of 240 characters, `at f (game.js:1:2345)` or `f@game.js:1:2345`), `engine` (version), `build` (og-build.json `sourcemaps.build`), `browser { name, major, os }`, `device { class: phone \| tablet \| desktop, cores, memory, dpr, touch }`, `gpu { vendor, architecture, fallback, limits: { maxTextureDimension2D, … }, features: string[] } \| null`, `memory { linear, heapLive, heapPeak, streamUsed, streamBudget, audio, entities, gpuTotal, gpuTextures, gpuTargets, deviceLosses } \| null` (bytes; entities a count; the gpu figures ProbeMemory.gpu; deviceLosses the devices lost and recovered) |
| `sample` `phase: "boot"` (once, at the first frame) | beside `renderer` and `timings`: `stages` (the Worker's boot in ms: `wasm`, `bundle`, `create`, `attach`, `transcode`, `prime`, `settle`, `present`), `pipelines { mode, reason }` (§Pipelines), `lanes { on, threads }`, `engine`, `build`, `browser`, `device`, `gpu`, `memory`, `failures` |
| `sample` `phase: "unsupported"` (once, before `fail`, on a browser without WebGPU) | `renderer: "none"`, `reason`, `engine`, `build`, `browser`, `device`, `gpu: null`; `fail` is `engine_start` and not retryable, and the engine never starts |
| `sample` `phase: "play"` | `frames { count, ms, bins, interval, work, hitches, longHitches, max }` (`bins`: the 11 upper edges in ms, 8.4 to 250; `interval` and `work`: 12 counts each, the last above 250 ms; the engine's performance record binned: a hitch is an interval more than twice the median of the 64 before it, a long one also 250 ms or more), `memory`, `failures [{ file, reason: http_<status> \| timeout \| network }]`, `session { ms, samples }` |
| `sample` `phase: "end"` (on `pagehide`) | `memory` as last read, `failures` not yet sent, `session` |

**The platform's side** (`docs/platform-asks.md` #17): the play origin's
`report` endpoint and the framed Player keep these fields when they file
`crash` and `sample` and never answer 400 for them, store each publish
build's `sourcemaps/` privately under `sourcemaps.build`, and map a
crash's `stack` with `mapStack` when they show it.

## project.ogt `publish`

The `publish` block's keys, values and defaults are in the doc comment of
`crates/og-cli/src/publish.rs`, which checks every value before a build
and names the line of a bad one. `probe` is checked but chooses nothing: a
`--publish` build serves the player members and a dev build every member.

A build replaces its output directory whole, so it refuses one that is the
project or an ancestor of it, holds a `project.ogt`, or is not empty and
holds no `og-build.json` from an earlier build; a build that fails removes
what it wrote.

### Pipelines

How the frame's render and compute pipelines compile on the web
(`publish { pipelines }`, picked in `src/web/worker/device.ts`):

- `sync`: a pipeline is made when a pass first draws with it, and the rest
  are warmed behind the play gate after the first frame.
- `async`: the renderer's pipeline cache (og-render `gpu/deferred.rs`)
  asks the browser for every pipeline with `createRenderPipelineAsync` or
  `createComputePipelineAsync`, through wgpu's
  `Device::create_{render,compute}_pipeline_async` (the fork's port of
  gfx-rs/wgpu#10438). Readiness is the cache's and works the same
  way natively. A display frame skips a draw whose pipeline is not ready
  yet. Work whose output later frames keep (a scatter chunk's placement, a
  grass field's blades, the bakes, the compute before the frame's passes)
  waits for a later frame. A frame that waits (a scripted, recorded or
  capture frame) makes each such pipeline at once with the synchronous
  call. After a
  frame that skipped a shadow caster's draw, the static shadow cache keeps
  none of its layers. The gate shows once a frame skipped nothing
  (`og_web_frame_skipped`).
- `prewarm`: as `sync`, with only the warm-up behind the gate
  asynchronous. Opt-in.
- `auto` (the default): `async` on every GPU except Apple's, which stay
  `sync` based on the physical M4 Pro measurements below; iOS is still
  unmeasured. A cold driver cache compiles a large scene's pipelines for
  seconds each (First Light on an NVIDIA L4 under Vulkan: terrain 4.8 s,
  pbr meshlet 4.0 s, live sky 3.6 s, 2026-11), and synchronous ones in one
  GPU-process task past Chrome's 15 s watchdog lose the device.

First Light's publish build on a physical Apple M4 Pro (Safari 27.2,
2026-11) measured navigation to `og:first-frame` once per condition:

| Mode | Fresh automation session | Same-tab reload |
| --- | ---: | ---: |
| `sync` | 9.08 s | 8.62 s |
| `async` | 8.60 s | 8.71 s |

Local HTTP with `Cache-Control: no-store`, 1280 × 768 CSS pixels at DPR 2.
Fresh means a new browser automation session, not a verified cold Metal
shader cache. The reload can reuse process state but does not measure a
warm HTTP cache. Async did not win both measurements, so Apple stays on
`sync`. These are single observations, not a general performance claim;
VM timings do not set this default.

Physical verification still needed: iPhone Safari (touch, first frame,
pipelines and offline); Android Chrome on Pixel and representative
PowerVR, Adreno and Mali GPUs (first frame and pipelines); and HDR output
on an HDR display in HDR mode. A desktop browser at phone width and SDR
screenshots do not cover those checks.

The mode and why are on `__runtime_probe__.boot.pipelines`, in the
Worker's console and in the boot telemetry sample.

## The shell

- **Boot**: the brand symbol on ink `#14161b` plays `genesis` once, then
  `progress` with the bytes received, captioned by the game's stage label
  while one is reported (§Loading events). Motion never delays the game:
  progress shows after genesis or 1.2 s, and `prefers-reduced-motion`
  skips genesis.
- **Play gate**: the first screen is drawn behind it, with the platform's
  answers to the first ticks' calls already applied (`src/web/settle.ts`:
  at most 8 ticks, 300 ms before the first frame, 5 s in all). The first
  tap, click, Enter or Space unlocks audio, asks for fullscreen when
  `fullscreen gesture` and the page is not framed, takes a wake lock and
  starts the clock. `gate none` starts at once.
- **Layout**: `fill` covers the `100dvh` viewport and passes the safe-area
  insets to the UI, letterboxed past `aspect_min`/`aspect_max` (4:3 and
  32:9 without them); `letterbox` fits `aspect` inside the safe area on
  ink. The canvas is drawn at `min(devicePixelRatio, dpr_cap)`.
- **Graphics settings** (og.render.setDisplay) apply at once, except
  `powerPreference`, kept in localStorage (`og.display:<page path>`) for
  the next load.
- **Input** (`src/web/shell.ts`, `src/web/gamepad.ts`): physical keys
  (`KeyboardEvent.code`; nothing is taken while an input field has
  focus), up to five pointers, wheel, pointer lock, and the first four
  standard-mapping gamepads, which also press the keys `gamepad.ts` lists.
- **Visibility**: a hidden page pauses the clock, suspends audio and
  releases the wake lock; showing it resumes all three.
- **Error card** (design §6): a failed start (a Worker silent for
  `BOOT_DEADLINE_MS`, two minutes, included) or a crash (§Errors) shows
  the reason, the last 200 lines of the log and the world dump (dev) or
  state (publish), with Reload and Copy report.
- **PWA** (publish builds, in a secure context): the service worker
  precaches `./` and every file except `og-build.json`, `_headers`,
  `origingame.runtime.json`, licence and README files, the streaming packs,
  the engine variant the page will not take (its primary and its split
  modules) and itself, under
  `og-<scope>-<version>` (`<scope>` a hash of the registration's scope;
  `<version>` og-build.json's `version`'s first 16 hex digits), and drops
  only this scope's other caches. It serves the page from the network
  first, through navigation preload so the platform still sees a document
  load, and every other file from the cache, a pack once fetched.
- **Incremental update.** `OG_SW.hashes` names each file's content (16 hex
  digits of its sha256), and each cache keeps that index under
  `og-sw-index.json`. A new build's worker copies into its cache every
  file whose content one of this scope's older caches holds under any
  name, and fetches only the rest and the page.
- **Install hint**: shown only when not framed and not installed, with a
  manifest and a service worker; "Not now" is remembered. The shell's own
  words are Chinese when `navigator.language` is `zh*`, else English.
- **Marks**: `performance` marks `og:ready`, `og:first-frame` and
  `og:start`; `__runtime_probe__.boot` holds the Worker's boot phases.
- **WebGPU only** (design §8): a browser without WebGPU gets the
  unsupported-browser card; there is no WebGL2 fallback. The page checks
  only that `navigator.gpu` is there; the engine Worker asks for the
  adapter before it fetches the wasm, since Chrome's first
  `requestAdapter` blocks its thread while WebGPU starts (seconds on a
  busy GPU process) and on the page that froze the splash.
- **URL parameters**: `replay-seed=<u32>`, `autoplay=0` (a harness drives
  the clock through the probe: no gate, no first tick, no service worker),
  `trace` (§The og-probe wire).

### Loading events

Every exported page tells whoever hosts it how loading goes, with no
platform and no script of the host's on the game's page: a page that is
the game's own top level listens on its `window`, and a page that frames
the game (any site, any origin) hears the same as messages.

| Event | `detail` | When |
|---|---|---|
| `og:progress` | `{ value, stage, fraction, ready?, requested? }` | the boot's bar: `value` 0 to 1 over every stage, `fraction` within `stage` (`wasm`, `bundle`, `fetch`, `transcode`, `ready`, `prime`, `present`); with `present`, the first frame's pipelines `ready` of those `requested` (each only goes up) |
| `og:stage` | `{ id, fraction, label }` | the game's own stage (og.loading.stage), from GameDesc.start on |
| `og:ready` | `{ lanes, tier }` | the engine is running (before its first frame) |
| `og:first-frame` | none | the first frame is drawn |
| `og:loaded` | none | the game called og.loading.done |
| `og:failed` | `{ message }` | the engine could not start or stopped |

- A framed page posts each to `window.parent` as
  `{ source: "origin-engine", type, detail }` with target origin `*`:
  the details carry no secret, and the host checks `event.source` is the
  game's iframe and `data.source` is `origin-engine`.
- project.ogt `publish { loading engine | game | page }` picks who draws
  the loading screen (`og api show loading`): `engine` the splash;
  `game` the splash until the first frame, then the game's own frames,
  the clock running, until og.loading.done (then the gate, with
  `gate tap`); `page` no splash and the canvas hidden, the clock
  running and its frames preparing, until og.loading.done (then the
  gate, with `gate tap`), the host drawing its loader from the events
  above. `publish { loading_labels { fetch "…" … } }` names the
  splash's caption per stage. og build writes them into the page's
  configuration as `bootScreen` and `stageLabels`, and og-build.json's
  `display.loading` says which.
- Byte progress after the boot (a streaming group's pack, another
  bundle's manifest) reaches the game, not the page: og.stream.status's
  `received` and `progress`, og.scene.progress and og.load.progress
  (`loading §12`).

```js
// A site framing the game draws its own bar.
addEventListener("message", (e) => {
  if (e.source !== frame.contentWindow || e.data?.source !== "origin-engine") return;
  if (e.data.type === "og:progress") bar.value = e.data.detail.value;
  if (e.data.type === "og:loaded" || e.data.type === "og:first-frame") bar.hidden = true;
});
```

## The Player and Stage contract

### Player protocol 2 (`origingame/docs/player-protocol.md`)

The shell says `hello` (`substrate: "origin-engine"`) at startup and owns
its readiness: `loading` with the byte fraction, `ready` at the first
rendered frame (or two seconds after the boot when nothing renders), `fail`
(`engine_start`, retryable) before it, `crash` (`EngineCrash` and a
fingerprint of the message, plus §Telemetry's fields) after it, and
`sample`s (§Telemetry). A framed page waits 300 ms for a Player's `hello`
before drawing its own gate; under a Player that answered, the shell draws
no gate: it plays on `start` (or a hello with `started: true`), pauses on
`pause`, resumes on `resume`, and resumes its AudioContext on `gesture`.

Who the shell speaks protocol 2 to, in order:

1. framed: `window.parent`, posting only at its own origin and hearing only
   `window.parent` at its own origin with `og: "player", protocol: 2`;
2. top level with the platform's `window.__OG_GAME`: its play origin over
   HTTP (§The top-level page);
3. top level, no `__OG_GAME`, and told its own origin as the Stage host
   (`window.__OG_STAGE_HOST__ === location.origin`, `shell/wire.ts`
   `stageInPage`): **its own window**, where the platform's Stage overlay is
   its Player. It ignores its own echo (a `hello` that names a `substrate`
   or lists no `capabilities`) and repeats its hello every 250 ms (at most
   six) until the overlay answers. Nothing of og.platform's goes to the
   browser's storage there: what the overlay does not answer within 1.5 s,
   or does not list, the page's memory answers for that load. No service
   worker is registered there;
4. otherwise no one: the page plays alone on the browser's storage.

### Platform services over protocol 2

The Player is one provider of og.platform; the contract every provider
meets is §Providers below and docs/design.md §2.8 Providers.

`og.platform` (IDL module `platform`) reaches the Player through the shell
(`shell/platform.ts`). The engine Worker hands each tick's calls to the
page (`{ type: "platform", calls }`); the page answers them
(`platform-answer`) and the answers are applied at the start of the next
tick. A framed page (or one with the Stage overlay in its own document)
holds its first calls until the Player's `hello`, or 1.5 s.

| Shell → Player | When | Answered by |
|---|---|---|
| `call` `store.get` `{ key }` | `storeGet`, `loadSettings` | `result` `{ key, value, revision }` (`value` null, `revision` 0 for a key never written) |
| `call` `store.set` `{ key, value, revision? }` | `storeSet`, `saveSettings`: og.platform's revision N sends `revision: N`, `create` sends `revision: 0` (only a key never written), revision 0 sends none (the last write wins) | `result` `{ key, revision }`, or `conflict` with `error.revision`, the key's revision now |
| `call` `store.list` | `storeList` | `result` `{ keys: [{ key, revision }] }` |
| `call` `leaderboard.submit` `{ score, board, order }` | `leaderboardSubmit` | `result` `{ ok, rank, board, best }` (or `{ rank: null, queued: true }` offline) |
| `call` `leaderboard.top` `{ n, board }` | `leaderboardTop` | `result` `{ board, sort, top, me }` |
| `call` `leaderboard.around` `{ n, board }` | `leaderboardAround` | `result` `{ board, sort, entries: [{ rank, name, score, me }], me }` |
| `call` `point.award` `{ reason, amount }` | `point` | `result` `{ granted, balance }` (`{ granted: 0, balance: null }` while no release carries an approved point policy, origingame `docs/public-api.md` §15) |
| `call` `ai.mark` `{ kind, label, model? }` | `aiMark` | `result` `{ marked: true }` |
| `event` `game.custom.<name>` `{ attrs }` | `telemetry` (one-way; `<name>` 1–64 of letters, digits, `.` and `_`, at most 16 scalar attributes, checked by og.platform) | — |
| `call` `purchase.*`, `ad.*`, `achievement.*`, `player.get`, `friends.list`, `invite.*`, `share` with og.platform's arguments | `products`, `purchase`, `entitlements`, `consume`, `adReady`, `showAd`, `achievementUnlock`, `achievementProgress`, `achievements`, `player`, `friends`, `inviteSend`, `invites`, `share`, only where the hello lists the method (otherwise `unavailable`, never the browser's storage) | `result` as docs/platform-asks.md #19 gives it, passed through unchanged; around an ad the Player sends `pause` and `resume` and answers after `resume` |

| Player → shell | What |
|---|---|
| `platform` `{ notice: { kind, data, time? } }` | unasked input (`purchase`, `entitlements`, `invite`, or `host`: a message from the framing page, `{ name, ... }`, whose meaning the game and that page agree): forwarded to the Worker as `platform-notice`, applied at the next tick's start as og.PlatformNotice |

The answers reach og.platform renamed only where its shape differs: a
leaderboard's `sort` is og.platform's `order`, and `top` its `entries`.
Each `call` carries the shell's own `id` (`oe-<n>`), and its `result` is
matched by it; a call unanswered in 15 s fails with `timeout`. What the
shell asks follows the Player's `hello.capabilities`: a Player that lists
`save.read`/`save.get` and `save.set` but no `store.*` keeps the store in
the save's `store` object, written back with `save.set { data, revision }`
at the revision it was read at; one with `leaderboard.top` but no
`leaderboard.around` answers `around` from the top 100; a method the hello
does not list answers `unavailable`; and where the Player keeps no store
the browser's `localStorage` (`og.platform:<path>`) does, as it answers
every call unframed (`src/platform-local.ts`). Errors keep the Player's
codes, `retryable` and a conflict's `revision`.

The host's clock (`og.platform.now`, every answer's `time`) is the Worker's
`Date.now()` when the platform attaches and when an answer arrives, and
`og.platform.info(world).clock` reads `browser`. A `result` that carries a
numeric `time` (UTC ms) has it passed on as that answer's `time`, and
`clock` then reads `player` (`docs/platform-asks.md` #9).

### The top-level page (Player protocol 2 over HTTP)

The platform opens a published page whose build says `og-player` … `page`
as the top-level document on the game's own origin (origingame
`docs/player-protocol.md` §The top-level engine page). The shell
(`shell/transport.ts`) then speaks protocol 2 to its play origin itself,
when the page is not framed and carries the platform's
`window.__OG_GAME` (`{ id, releaseId, protocol: 2 }`, added to the entry at
publication).

- **Requests.** `POST /__player/v2/<id>/<releaseId>/hello`, `call` and
  `report`, same origin, `credentials: "omit"`, one at a time through one
  queue (so the page mints at most one guest), each with `Authorization:
  Bearer` once the page holds a credential. A `call` is the protocol-2
  `call` message and its answer the `result`; 15 s without an answer is
  `timeout`, a failed fetch `offline` (both retryable), any other status
  the protocol's code for it (`404` `not_found`, `429` `rate_limited`,
  `5xx` `unavailable`).
- **Identity.** The Portal's hand-off code (`#og-h=<code>`) is removed from
  the address before anything else (`history.replaceState`; the rest of the
  fragment stays the game's) and sent only in the hello's body. The answer's
  `player { token, guest }` (from the hello, or from any call or report that
  minted a guest or replaced a rejected credential) is the credential: a
  guest's is kept in memory and under `og_token_<id>`, an account's in
  memory only with the `og_portal_<id>` marker, as the framed Player keeps
  them. Before a code is redeemed both keys are removed. The credential is
  never in og.platform's storage, a URL, the error card or a report.
- **The outbox.** An unconditional `save.set` and a `leaderboard.submit`
  that fail retryably are kept under `og_outbox_<id>` (`{ save?: { data },
  scores, at }`, 7 days, the latest save, the last 20 scores in order) and
  answered `{ ok: true, queued: true }` / `{ ok: true, rank: null, queued:
  true }`; they are delivered after the next answered call and after the
  next hello. Store writes and conditional writes are never queued.
- **Reports.** Nothing before the person starts play at the shell's own
  gate (the hello says `started: false`). Then `start`, what was held
  (`ready`, `fail`, the `sample`), and after it `crash`, the game's
  telemetry as `event` `game.custom.<name>`, a `heartbeat` every 60 s once
  ready and `end` on `pagehide` (`keepalive`, with `durationMs`, `ready`
  and `failed`). Each message carries `id` (a UUID v7), `at` (epoch ms) and
  `ms` (ms since navigation start); a report carries `session`
  (`game_session_<32 hex>`, per load) and `device` (`device_<32 hex>` from
  `og_device_<id>`, 30 days). A report holds at most 20 messages and
  64 KiB; one that fails is kept (at most 200 messages) and sent with the
  next, one answered 400 is dropped, and one answered 413 is halved.
- **The brand surface** is the shell's at the top level (design §4, §9),
  with the hello's `page.backdrop` behind the boot splash and a **Back**
  that goes `history.back()` when the referrer is on the hello's
  `page.portal`, else to `page.back`.

### Providers

`@origingame/engine/provider` (`src/provider.ts`) is the host side of
og.platform as a versioned contract (`PROVIDER_VERSION`, 1; a bump is a
break): a `PlatformProvider` (`host`, `capabilities`, `handle(calls,
answer, link)`, optional `poll`; the shapes `PlatformCall`,
`PlatformAnswer`, `PlatformNotice`, `PlatformLink`) and a `RoomProvider`
(`host`, `handle(ops, deliver)`, optional `attach(deliver)`; `RoomOp` and
`RoomEvent` are og-net's protocol). The vocabulary is the IDL's:
`platform.ogt`'s list constants (`SERVICE_METHODS`, `CORE_ONLY_METHODS`,
`HOST_EXTENSIONS`, `ERROR_CODES` and `RETRYABLE_CODES`) are generated into
`src/generated/vocabulary.ts` by `og api gen` and re-exported by
`provider.ts` (`CORE_METHODS` is the core-only methods plus the service
methods). A provider answers each call once (`telemetry`
never) and anything it does not list `unavailable`;
`checkPlatformProvider(provider)` checks this in a provider author's own
tests.

The page that hosts the game names its providers, never the game:
`play({ provider: { platform?, rooms?, sessions? } })` (`shell/page.ts`),
or, for the page `og build` writes, `window.__OG_PROVIDER = { platform?,
rooms?, sessions? }` set by a script before `play.js` loads. `rooms` is a
RoomProvider or an `og relay` URL; `sessions` is a SessionProvider (`host`,
`connect({ world, reconnect })` answering `{ ok, url, token? }`) or an `og
world-server` URL. Each part takes the place of the default only where no Player
answers: `platform` that of the browser's storage, `rooms` that of no
rooms. A part that is not a provider is left out with a line in the log.
In a publish build a provider whose `host` is `local` answers the platform
services `unavailable`: a published page never fakes a purchase.

Content catalogs (og-api guide saves §11, loading §14): a provider may
name the game's content base, `content: { base() }` (one https directory
for every catalog of the game, read once when the engine starts), which
takes the place of the build's `--content`. `staticCatalog(base, { inner,
read })` is a provider over the static layout: it answers `catalog.head`
from `<base>/<catalog>/head.json` (`cache: no-store`), names `base`, and
hands every other call to `inner`. The page `og build --content` wrote
answers `catalog.head` the same way where no Player and no provider do. A
Player answers `catalog.head` where its hello lists it and may push `{
kind: "catalog", data: { catalog, version, manifest } }`; a notice is a
hint, never relied on: games poll the head. A Player's hello may name the
content base, `content: "https://…/"` (the game's catalogs' directory),
which takes the place of the build's: a framed or top-level page starts
the engine with its base to come, and og.load.data loads wait (30 s at
most) until the hello, or the wait for one, settles the platform
(`og_engine_set_content`).

`relayRooms(url, { reconnectMs })` (`src/web/relay-rooms.ts`) is the web
room provider for a self-hosted page: og-net's protocol to an `og relay`
over the browser's WebSocket, mended after a lost connection as the native
client mends it (docs/design.md §2.8). A page on https reaches only a
`wss://` relay, so `og relay` needs a TLS proxy in front of it.

og.shared's session providers (docs/design.md §2.12) name a world server
and a token for each connection; the engine Worker opens the WebSocket
itself. `staticSession({ url, token? })` is a fixed endpoint;
`tokenEndpoint(url)` POSTs `{ world, reconnect }` with the page's cookies
to the game's own endpoint, which signs a short-lived token for its player
with the secret of an `og world-server --auth hs256` (or any issuer the
server's `--auth jwks` trusts) and answers `{ url, token }`; a 401 or 403
ends the session `unauthorized`, a network failure or a 5xx is tried
again. A platform's provider is one more implementation: game code is the
same under each.

### The og-probe wire (`origin/packages/contracts/src/probe-wire.ts`)

The shell serves the wire itself (`shell/wire.ts`) over the engine's probe,
which lives in the Worker and answers asynchronously:

- It answers `{og: "probe", v: 1, kind: "call"}` only from `window.parent`
  (else `window.opener`) at the origin in `window.__OG_STAGE_HOST__`; a page
  told no origin, or another origin, or a call of another `v`, gets nothing.
  A top-level page told its own origin also answers its own window (the
  Stage overlay in the engine's document, §Player protocol 2). Every post
  names that origin. It posts `hello` on load, when the told origin or the
  probe's availability changes, and after a `hello` call.
- The shapes are the wire file's. `capture` comes from the engine's own
  readback, fitted into `width`×`height` without upscaling, and `record`
  from the canvas's capture stream. `advance` takes 0 to 3600 steps,
  `capture` a width and height within 0–4096, `record` up to 15 seconds
  and one recording at a time.
- Errors: `refused` for a probe refusal and parameters out of bounds;
  `unsupported` for an unknown method (including `input`), no probe yet,
  and every member a publish build drops (`pause`, `resume`, `advance`,
  `bounds`, `patch`); `failed` for a fault.
- A dev (Stage) build serves every member, `listen`, `memory` and `trace`
  included; a publish build the 7 player members.
  `globalThis.__runtime_probe__` (`version: 5`, every member a Promise)
  stays for in-page callers, and `?trace` records the Chrome trace from
  the boot.

## The `og` commands around the export

Their options are in `og build --help`, `og run --help` and
`og doctor --help`. What they promise beyond that:

- `og build` writes into `--out`, else `publish.out`, else
  `<project>/dist`, and prints each file's compressed size and what a
  player downloads (`downloadsOf` in `src/node/web-build.ts`): brotli -q 11
  for `--publish`, brotli -q 5 for a development build, whose -q 11 over
  the development engine takes minutes (each row's `codec`). Compressed
  sizes and probe reads are cached by sha256 in `dist/build-facts.json`
  for the engine's own modules and in `~/.cache/origingame/build-v1/`
  otherwise (`OG_BUILD_CACHE` names another directory; `off` keeps
  nothing).
- `og run --device` serves the publish build cold with its `_headers` and
  COOP/COEP/CORP, passes the play gate with a real tap, times the page and
  writes the measures and findings (no score) to `report.md` and
  `device.json`; a missing prerequisite (Chrome, safaridriver, adb) exits 1
  naming its fix (`crates/og-cli/src/device.rs`).
- `og doctor` reports no renderer as `none`, not as a problem (docs/ci.md
  §Switches in the environment).

### og dev --web

`og dev [project] --web [--port P] [--host H]` (default
`127.0.0.1:47810`; `src/node/dev-web.ts`, `src/node/dev-web-client.js`)
serves the project's development build from `og-out/dev-web/` and keeps
it live. It is the platform Stage's dev server: Origin Server runs
`og dev . --web --port $PORT --host 127.0.0.1` as a supervised service and
proxies it on its relay. It stays up through cook, bundle and type errors
and prints one line on stdout when ready:
`{"event":"ready","url":"http://127.0.0.1:<port>/","build":"<id>","error":null}`.

- **HTTP.** Every URL the page uses is relative, so it is served at `/`
  or under any prefix (a proxy's ticket path). The entry keeps
  `<meta name="og-player" content="frame page">` and the og-probe wire to
  `window.__OG_STAGE_HOST__`; every response is `no-store` and there is no
  service worker. `cooked/<path>` is the project's live cooked bundle;
  the server's own routes are under `_dev/` (nothing under `__og`).
- **`GET _dev/status`** answers
  `{ready, build, error, last?: {kind: "files"|"code"|"reload", ms, at, steps}}`.
  `build` changes only on a cold restart (the first build, a
  `project.ogt` change); `error` is the current cook, bundle or type
  error, else null.
- **`_dev/socket`** (`new URL("_dev/socket", document.baseURI)`, ws or
  wss): text JSON, each message well under 1 MiB, since file bytes are
  fetched over HTTP. Server to page: `hello {build, seq, ready, error}`
  on connect; `files {seq, files: [{path, hash}], edits, steps}` (the
  page fetches them in one `GET _dev/files?f=<path>&f=…`, each file
  framed as a u32 LE path length, the path, a u32 LE byte length and the
  bytes, and applies them in place, `devFiles`);
  `code {seq, url, hash, edits, steps}` (the game's own code, bundled
  alone as build.ts `web-code`; the page imports it and the probe's
  `reload` swaps it in, `devCode`); `reload {build}`; `error {error,
  edits}` (null: cleared). Page to server: `hello {build, seq}` on each
  (re)connect, where `seq` is the last change it applied: the same build
  gets what it missed, another build `reload`; `applied {seq, kind, ok,
  ms, steps, error?}`. The page reconnects with backoff (250 ms doubling
  to 5 s). Changes apply one at a time; those that arrive meanwhile are
  merged into one, the newest code bundle superseding older ones.
- **Timings.** `window.__og_dev_reloads__` lists each applied change with
  its steps, save to frame. Files: `saveToCookedMs` (the watch, the cook)
  + `fetchMs` (`files` and `bytes` fetched) + `applyMs` (the Worker's devFiles, `worker` its steps) +
  `frameMs`. Code: `saveToBundledMs` + `writtenToFrameMs` (`importMs`,
  `reloadMs` in the Worker, `frameMs`). Budgets: 1.5 s save to frame for
  an asset or UI edit, 300 ms from the code bundle written to the frame.
- **Style packs.** After each cold build the server cooks every built-in
  style pack the project's look does not use, once, in the background at
  the lowest priority (`og-out/dev-web-warm/`, removed after), into the
  shared cook cache (`OG_COOK_CACHE`, `~/.cache/origingame/cook-v1`). A
  later `ui "<pack>"` edit takes its fonts, art and CSS from the cache
  instead of cooking them, a few seconds the first time on a machine.
- **The Stage's dev channel.** In the same document, over
  `window.postMessage(msg, location.origin)`, each side accepting only
  `event.source === window && event.origin === location.origin`, messages
  `{og: "dev", v: 1, from: "page"|"host", type, id?, …}`: the page sends
  `hello {inspector: true, keep: false, engine}` every 250 ms, at most 20
  times, until the host answers `hello {}`; the host's `inspector
  {open}` opens or closes the inspector, and the page sends `inspector
  {open}` whenever it changes, F1 included. Version 1 has no Keep (the
  overlay's own Keep stays) and sends no `edits`.
- **Runtime tuning.** Inspector/Tune edits and prefab Revert change memory
  only, identically on native and web. Save, Apply and `scene_saved` are
  removed (docs/platform-asks.md #22/#23). Persist explicit TS literals
  through checked `og set`; edit computed/shared expressions in their source.
  Read-only source diagnostics retain their TS file/line and `read_source`.

## Packaging

```sh
cd packages/engine
sh scripts/pack.sh --linux-x64    # OG_RELEASE=1 scripts/build.sh --linux-x64, then dist/pack/*.tgz with sizes and sha256
bun ../../tests/install/instabox.ts --version <v> --from ../../dist/release/<v>   # optional, billed by the minute (docs/release.md §4 D)
```

- `@origingame/engine`: the TypeScript API and runtime, the web hosts
  (`dist/web`, `dist/web-publish`), the shell, the
  Basis transcoder, `bin/og.mjs`. No native code.
- `@origingame/engine-<platform>-<arch>`: `origin-engine.node` and `og`,
  one tarball per platform, which `scripts/release-dist.ts` puts into that
  target's archive with `@tsrs/<platform>`, `og check`'s type checker
  (pinned at `TSRS_VERSION` of `scripts/install-tools.sh`; none on
  Windows).
- How a version is released and distributed: `docs/release.md` §1–§3.
