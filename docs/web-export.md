# Web export

`og build` turns a project into a directory any static host serves: the
player shell, the engine (wasm in a Worker on an OffscreenCanvas), the game
bundle and its cooked assets. This page is the output layout, what a host
must send, project.ogt's publish settings, the shell's behaviour, the
contract the platform's Player and Stage rely on, the `og` commands around
it, and how the package is packed. Design: `docs/design.md` §2.2, §4, §6,
§9.

```sh
og build .              # dev build: every probe member, no service worker
og build . --publish    # what ships: the player probe members, minified, the PWA
```

## Output layout

The plain names below are a dev build's. A publish build writes the
bundles, the engine modules and the decoders under content-hashed names
only (`play.<hash>.js`; §Content-hashed names), listed in og-build.json.

| Path | What it is |
|---|---|
| `index.html` | The player shell: the canvas, the boot splash (with the cover's backdrop on a platform page), the play gate, the error card, the install hint, Back, the badge. Carries `<meta name="og-engine" content="<engine version>">`, `og-build` (`publish`/`dev`), `og-profile` and `og-player` (`frame page`: the Player transports the shell speaks, what the platform reads to open the page at the top level), the inlined CSS (brand tokens and only the `genesis`, `progress` and `drift` motion rules) and the page configuration (`<script type="application/json" id="og-config">`). |
| `play.js` | The shell (`packages/engine/shell/page.ts`, bundled). |
| `game.js` | The engine Worker: the runtime and the game. |
| `game.js.map`, `play.js.map`, `audio-worklet.js.map` | A dev build's source maps, beside their bundles and linked from them (§Source maps). |
| `sourcemaps/` | A publish build's source maps of the same three bundles, unlinked: not in `files`, `version` or the service worker's cache, and not to be served (§Source maps). |
| `audio-worklet.js` | The AudioWorklet, loaded on the first gesture. |
| `origin-engine.js`, `origin-engine_bg.wasm` | The web host, shipped directly from wasm-bindgen. A dev build takes `dist/web` (every member, both physics profiles). `--publish` takes the publish variant of the project's `profile` (§Variants): `dist/web-publish` for `3d`, `dist/web-publish-2d` for `2d`, each with only its profile's physics (probe trimmed to hello, tick, state, pick, entity, input, capture); og-build.json's `variant` names it. |
| `origin-engine-lanes.js`, `origin-engine-lanes_bg.wasm`, `snippets/` | `--publish` with `isolation` not `off`: the lanes variant of the same profile (§Lanes, §Variants), built with wasm atomics and shared memory, from the same directory as `origin-engine_bg.wasm`, and wasm-bindgen-rayon's Worker script. |
| `basis/` | Binomial's Basis transcoder and its licence. |
| `cooked/` | The cooked bundle (`og cook`, run by `og build`), split by streaming group: the first group (everything no scene's `stream` block holds, the `#proxy` stand-ins included) and one pack per streaming group, `cooked/stream/<name>.ogb`. A grouped output's own file is left out; the engine reads it from its pack. |
| `icons/` | `publish.icon` as `icons/icon.png`, or the brand tile at 192 and 512 px. |
| `og-build.json` | The build manifest (below). |
| `_headers` | For static hosts (Netlify and Cloudflare Pages format): `Cross-Origin-Resource-Policy: cross-origin` and `X-Content-Type-Options: nosniff` on everything, COOP `same-origin` and COEP `require-corp` unless `isolation off`, `Cache-Control: no-cache` on `/`, `index.html`, `sw.js` and `og-build.json`, immutable caching on the content-hashed files, and the `.wasm` and `manifest.webmanifest` content types. |
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
  "variant": "2d",
  "names": { "game.js": "game.3f2a9c1e04b7.js", "play.js": "play.81aa0f3c6d25.js", "…": "…" },
  "probe": { "protocol": 5, "wire": 1, "build": "publish",
             "members": ["hello", "tick", "state", "pick", "entity", "input", "capture"] },
  "crossOriginIsolation": "preferred",
  "display": { "fit": "fill", "aspect": null, "aspectMin": null, "aspectMax": null,
               "orientation": "any", "dprCap": 2,
               "fullscreen": "off", "gate": "tap", "madeWith": true },
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
  content-hashed file's own name to its served one (§Content-hashed names;
  `{}` in a dev build). A source map's `file` is its bundle's served name;
  `mapStack` matches a frame in `game.<hash>.js` to the map given as
  `game.js`.
- `engine.version` is the engine that built it; `engine.pinned` is
  project.ogt's `engine`. `probe.members` is read from the copied wasm's own
  `hello`, so a publish build proves its trim. `probe.build` is what that
  `hello` reports: `publish` for `--publish`, `release` for a dev build (the
  top-level `build` is what says `dev`).
- `variant` is the engine module `wasm` is (§Variants): `3d` or `2d` (the
  publish variant of `game.profile`), or `full` for a dev build. The lanes
  module, when there is one, is the same profile's.
- `crossOriginIsolation`: `required` (the game needs SharedArrayBuffer; a
  page served without isolation still starts, single-threaded, with a
  warning in the log), `preferred` (the default: it uses the input, request
  and audio rings when isolated and runs single-threaded otherwise), `off`.
- `files` lists every file beside it (not itself) with bytes and sha256;
  `version` is the sha256 over the path and sha256 of each of them except
  `sw.js`, so it is the same before and after `sw.js` is written.
- `sourcemaps` names each bundle's map (`game.js`, `play.js`,
  `audio-worklet.js`) and `build`, the build id: the first 16 hex digits of
  the sha256 over the three bundles' paths and hashes. The page's
  configuration carries it as `buildId`, and a crash report names it as
  `build` (§Telemetry), so the platform finds the maps of the bundles that
  threw. `served` is `true` for a dev build, `false` for a publish build.
- `player` is the Player protocol the shell speaks and its transports, in
  every build: `frame` (protocol 2 over `postMessage` to a framing Player)
  and `page` (protocol 2 over same-origin HTTP when the platform opens the
  page at the top level; §The top-level page). `<meta name="og-player">`
  says the same.
- `stream` is `null` for a bundle without streaming groups, else `{
  "first": { "files", "bytes" }, "groups": [{ "name", "file", "bytes",
  "sha256", "priority", "memory", "assets" }] }`. `first` is what the boot
  fetches and the service worker precaches (the cooked manifest included; a
  chunked sound is its first chunk there). Each group is its pack under
  `cooked/stream/`, with the memory its assets take once loaded (what
  `og.stream`'s budget counts) and its asset ids. The packs are in `files`.
- `pruning` is `null` in a development build. A publish build lists `omitted`
  outputs (`id`, `file`, `bytes`), `removed_files` relative to `cooked/`, and
  `computed` loads (`source`, `line`, `prefix`, `reason`). An omitted id can
  share a file with a retained id, so omitted bytes must not be summed as savings.
  Roots are project asset references, placed scenes, TypeScript literals, and
  `publish { keep "path/**" }` patterns. A computed load retains its static
  prefix, or everything without a prefix. A needed streaming pack remains whole.
  Pruning changes only the export, never source assets or development cooked files.
  `cooked/.assets.ogt` and `.og/publish-assets.json` are local metadata, never shipped.

### Streaming groups

The Worker fetches the first group before the engine starts: the manifest,
every output no group holds, and their Basis textures transcoded. A
streaming group's pack is fetched only when the runtime admits the group
(its trigger wants it and it fits the budget, IDL module `stream`), and its
Basis textures are transcoded before the pack reaches the engine
(`og_web_pack_ktx2`), so the whole group goes in place at once. Until then
its models draw their proxies, its textures are 1x1 placeholders and its
sounds play as silence of their own length. A sound cooked in chunks is
fetched by neither: its first chunk comes with its group (the boot, or the
pack) and the engine asks for each other chunk ahead of its playheads,
never before its group is resident or after it unloads. The boot's
progress (`expect.cooked`) counts only the first group.
`packages/engine/tests/asset-stream.test.ts` checks the split, and
`packages/engine/tests/web/stream.ts` plays it in Chrome.

## Hosting

- Serve the directory as is, under any path: every URL in it is relative.
- `.wasm` as `application/wasm`; `manifest.webmanifest` as
  `application/manifest+json`.
- Isolation (`required`/`preferred`): `Cross-Origin-Opener-Policy:
  same-origin` and `Cross-Origin-Embedder-Policy: require-corp` on the page
  and its files. The build is self-contained, so COEP refuses nothing. A
  page framed by another origin also needs `allow="cross-origin-isolated"`
  on the iframe and `Cross-Origin-Resource-Policy: cross-origin` (in
  `_headers`).
- Without isolation everything still works: input and probe calls cross
  `postMessage` instead of the SharedArrayBuffer rings, audio uses a
  MessagePort to the worklet, and the engine runs single-threaded (§Lanes).
- The lanes pool's Workers start from a `blob:` URL (wasm-bindgen-rayon), so
  a host that sends a Content-Security-Policy must allow `worker-src blob:`
  (or send no `worker-src`/`script-src`); one that refuses it gets the
  single-threaded engine after the pool's deadline at most (§Lanes).
- `index.html`, `sw.js` and `og-build.json` should not be cached long. A
  publish build's content-hashed files may be cached forever
  (`public, max-age=31536000, immutable`, in `_headers`).

### Content-hashed names

A publish build serves every file the page or the engine Worker fetches by
name under a name that carries its content: `<stem>.<the first 12 hex
digits of its sha256>.<ext>` (`game.3f2a9c1e04b7.js`). This covers
`play.js`, `game.js`, `audio-worklet.js`, `origin-engine.js`,
`origin-engine_bg.wasm`, the lanes glue and module,
`basis/basis_transcoder.{js,wasm}` and the AV1 fallback decoder
`av1/og-av1.wasm` (fetched only where WebCodecs cannot decode AV1; the
service worker caches it lazily) (`HASHED_FILES` in
`src/node/web-build.ts`). A new build changes the URL of what changed and
nothing else. The page finds them through its configuration's `files` (the
same map as og-build.json's `names`), and `index.html` loads
`./play.<hash>.js` directly. A publish build writes these files under their
hashed names only: a host or tool finds them through og-build.json
(`worker`, `wasm`, `lanes`, `names`), never by a plain name.

Some names stay as they are: the entry points (`index.html`, `sw.js`,
`og-build.json`, `manifest.webmanifest`), the icons, `snippets/` (the lanes
pool's helper, which imports the glue by the URL it is given) and
`cooked/`, whose files the cook's manifest names; the service worker
carries an unchanged cooked file over by its content instead (§Incremental
update). A dev build keeps every plain name, because `og dev`, the tests
and its linked source maps read them.

## Lanes

Rayon runs the engine's data-parallel work on a pool of threads
(`crates/og-lanes`): one a core up to 8 natively (the addon and `og`),
Workers sharing the wasm memory on the web. `og_lanes::map`, `for_each_mut`
and `for_each_indexed` keep the input order; each caller names its site and
the report counts the parallel calls and items per site:

- `anim`: the animation graph's evaluation and skinning palettes per
  character (og-anim, 64 skeletons a task).
- `extract`: extraction's per-row decoding, natively only (og-runtime, 512
  rows a task; og.Mesh and og.Transform are read from their columns). The
  wasm decodes serially, since a row's decode there is allocations behind
  one allocator lock.
- `cull`: frustum culling per view (og-render, 1024 bounds a task from
  131,072 bounds, serial below). The recording stays on the engine thread.
- `physics`: the whole step, natively and in the lanes wasm, through
  `og_lanes::install_from`. rapier's `parallel` build (bit for bit the
  sequential build under enhanced-determinism; the single-threaded wasm
  keeps the sequential build) splits broad phase, narrow phase and solver
  from `og_physics::pool_bodies` bodies for the pool's thread count (500 on
  two to four, 1,000 on five or six, 1,500 from seven); component gathering
  by row ranges, the contact graph by edge ranges with its touching pairs
  sorted by `og_lanes::sort`, ropes, cloths and fluid sets side by side, and
  the fluids' neighbour search and collisions by particle ranges
  (`og_physics::Grains`). Every rapier call stays in body order; a fluid's
  density, viscosity, relaxation and mixing passes stay serial.
  `og.physics.stats` reports each phase's time and whether it went to the
  pool (on the web timed with each Worker's `performance.now`). Physics
  queries outside the step (raycasts, shape casts, overlaps) run through
  `og_lanes::serial`, on the engine thread inside its one-thread pool, so
  parry's own `rayon::join` never reaches a Worker.

A call splits only from two tasks' worth of items, since waking the pool
costs more than small work. TS systems keep their deterministic order on
the engine thread. Every lane writes back in input order, so the world
hash, captures and replays are identical with lanes on and off
(`packages/engine/tests/lanes.test.ts` and the native/wasm hash tests).

**One wasm or two.** Two. A wasm module built with atomics needs shared
memory, which needs cross-origin isolation, and without isolation it cannot
be instantiated at all. So `origin-engine_bg.wasm` stays single-threaded
(the module every page can load), and a `--publish` build whose `isolation`
is not `off` also ships `origin-engine-lanes_bg.wasm`. A page downloads one
of them, so the size budget holds per download (§Variants). Both are one
per physics profile. The lanes build needs nightly-only flags
(`scripts/wasm-lanes.sh`: `RUSTC_BOOTSTRAP=1`, `-Z build-std` with
`rust-src`, its flags appended to the caller's `RUSTFLAGS`, an inherited
`CARGO_ENCODED_RUSTFLAGS` unset; docs/ci.md). Its shared memory has a
maximum of 4 GiB (65,536 wasm pages), matching the single-threaded module's
wasm32 ceiling where the browser allows. The browser reserves that address
space up front and commits memory as it grows. wasm-bindgen generates the
shared-memory initializer from the module's declared maximum; the host
loader does not impose a separate cap. A `--publish` build
that would ship lanes but finds the lanes files missing from its profile's
directory says so in one warning naming the file and ships
single-threaded; `scripts/build.sh` under the gate (`OG_GATE=1`) stops
instead when `scripts/wasm-crates.txt` marks og-host-web `lanes` and either
profile's lanes wasm is missing.

**The choice at boot.** The shell (`src/web/lanes.ts` `laneChoice`) takes
the lanes variant when the build offers it, `crossOriginIsolated` is true
and `navigator.hardwareConcurrency` is above 2, with `min(cores - 1, 8)`
Workers beside the engine Worker. The engine Worker starts the pool
(`initThreadPool`) and tells the engine (`og_lanes_ready`); if the lanes
module or the pool fails to start, it starts the single-threaded module
instead, so a page always plays. wasm-bindgen-rayon waits for each pool
Worker with no timeout and no error handler, so `src/web/host.ts`
`startPool` races the pool against `LANES_POOL_DEADLINE_MS` (10 s) and
against an `error` event from any Worker the pool creates; on either it
terminates the pool's Workers and discards the lanes glue before the engine
is told. That covers a lanes glue or wasm that does not load, a pool Worker
the page may not create (a CSP without `worker-src blob:`) and one that
never reports ready (only the deadline catches that).

A panic inside a lane aborts its pool Worker, so the lane never ends and
rayon's own wait would hold the engine Worker in `Atomics.wait` forever.
So in the lanes wasm the engine thread never waits on rayon: it hands each
parallel call to a relay (one pool Worker that runs what the engine thread
sends it, started by `og_lanes_ready`) and waits on og-lanes' own latch,
which the call's end or a recorded panic rings (checked every 250 ms
besides). Once the relay runs, the engine thread sits in its own one-thread
pool, so rayon reached from it outside og-lanes (a library's own
`rayon::join`) runs inline, and a panic in work the engine thread runs
itself traps at once. The panic hook records the lane's panic (message,
location, tick, site) and rings the latch; the engine thread then panics
with that line, traps, and the Worker reports the crash with the lane's
report (§Errors) within a frame or two
(`packages/engine/tests/web/lane-panic.ts`). Anything else that stops a
playing Worker's frames is caught by the shell's frame deadline (§Errors,
`hang`).

The API is the same either way. The mode is reported: the ready message's
`lanes` (`{ on, threads, why }`, `why` saying why it is off), a log line
(`lanes: on, 7 Workers` or `lanes: off (…)`), the page's
`__runtime_probe__.lanes`, and the Worker's `call("lanes")` (`og_web_lanes`:
threads, pool, and how many parallel calls and items ran).

**Service worker.** `OG_SW` lists both variants (`single`, `lanes`); the
worker precaches the one `laneChoice` picks in its own scope (its
`crossOriginIsolated`, since `sw.js` is served with the build's COEP, and
its `hardwareConcurrency`) and serves the other from the network into the
same cache the first time it is asked, so a wrong guess costs one download,
never a failed boot. Once the engine is ready the page posts `{ type:
"og-cache", files }` (the booted variant's glue and wasm) to the active
worker, which caches whatever of that variant it lacks (`snippets/` for
lanes), so the variant the page actually runs plays offline.

**Locally.** `og dev` and `og run` run the native host, whose lanes are a
rayon pool (`OG_LANES`: `off`, `0` or `1` serial, `N` threads, unset the
cores up to 8). A browser page of a build is isolated when its server sends
the build's `_headers` (`tests/web/serve.ts` does, as the platform's play
origin does for a release with `origingame.runtime.json`); `og run
--device` also forces COOP `same-origin`, COEP `require-corp` and CORP
`cross-origin` on every response.

## Variants

One engine module per download. A publish build ships the variants of the
game's physics profile (project.ogt `profile`), single-threaded and lanes,
so a page never downloads the other profile's physics:

| Variant | Where | Built with (og-host-web) | Physics |
|---|---|---|---|
| 3D publish | `dist/web-publish/origin-engine*` | `--no-default-features --features publish,physics3d,shaping` | rapier3d |
| 3D lanes | `dist/web-publish/origin-engine-lanes*`, `snippets/` | the same, wasm atomics (§Lanes) | rapier3d (`parallel`) |
| 2D publish | `dist/web-publish-2d/origin-engine*` | `--no-default-features --features publish,physics2d,profile2d,shaping` | rapier2d |
| 2D lanes | `dist/web-publish-2d/origin-engine-lanes*`, `snippets/` | the same, wasm atomics | rapier2d (`parallel`) |
| dev | `dist/web/origin-engine*` | default features (both profiles) | both, not shipped |

What `publish` leaves out of every variant, beside the probe's tool
members: GPU timestamp queries and the reflected and record field docs
(og-core's and og-api's `publish`: a schema keeps its names, labels, units,
ranges, defaults and variants).

Otherwise every variant is the same (animation, UI, audio, navigation) but
for two switches: og-physics' `physics2d`/`physics3d`, and `profile2d` in
the 2D modules. og-render's `profile2d` leaves out the 3D world's renderer
(terrain, planar water, the 3D fluids, scattered copies and volumetric fog:
their shaders and per-frame work; `RenderCapabilities.world3d` is false),
and og-runtime's leaves out terrain and tile streaming, so every
`og.terrain` call and a `SHAPE_HEIGHTFIELD` collider answer a refusal
naming `world3d`. The rest of the renderer, lit meshes included, is in
both. So that nothing ships that would not draw, `og build --publish` (and
`og run --device`) refuses a `2d` project whose scenes hold `Terrain`,
`TerrainScatter`, `Water`, `WaterSurface` or `Fluid3D`, naming each by
scene file and line, before anything is cooked (og-cli's
`publish::refuse_3d_world`); a 2D dev build and a 3D project keep them. A
world of the other profile is refused when it is created ("og.Physics.profile
2: this engine was built without the physics2d feature"), on the lanes
modules too (`packages/engine/tests/hosts.test.ts`, which also steps each
profile's scene on its lanes module against the native host's world).

`og build --publish` copies the variant of `profile` as `origin-engine.js`
and `origin-engine_bg.wasm` (the shell and the service worker see one
module either way) and records it as `variant`; a `2d` project whose
package has no `dist/web-publish-2d` fails the build with the fix rather
than shipping the 3D module. Its lanes files come from the same directory
under the names `origin-engine-lanes.js`, `origin-engine-lanes_bg.wasm`,
`snippets/`.

The size budget (design §7, `scripts/budgets.txt`) is per download: each
variant is held to it, and what counts is the largest one a player
downloads, never the sum (`scripts/size-gate.sh` prints each and the
largest; docs/ci.md §Updating a budget). The gate builds all four
(`scripts/wasm-crates.txt` `variant` and `lanes` lines, docs/ci.md
§Switches).

## Errors

A failure reaches the page as a structured host error, the same object
everywhere: the Worker's `crash` (or `error` before ready) message carries
it as `error`, `Shell` passes it to `onCrash(message, error)` and keeps it
as `shell.error`, the page puts it on `__runtime_probe__.error` (null while
the engine runs) and on the error card:

```ts
{ kind: "panic" | "error" | "hang", message, tick, lane, location }
```

- `panic`: a Rust panic. The web host's panic hook records the first one
  (og-lanes `record_panic`: its message, `file:line:column`, the tick the
  world was running, and the lanes site when it happened inside a lane,
  else `lane: null`) before the wasm traps, and the Worker reads it back
  (`og_web_panic`). The card's line is `panic at tick 412 in a lane (cull):
  <message> (<location>)`.
- `error`: a failing system (its message names the tick and the system), a
  trap without a recorded panic, an uncaught error in the Worker; `tick` is
  the last tick a frame reached.
- `hang`: the frame deadline. A playing Worker sends the shell a `beat`
  about once a second; a visible, playing page that hears none for
  `FRAME_DEADLINE_MS` (120 s) terminates the Worker and reports a hang. A
  hidden page, a paused clock and a page that was itself blocked do not
  count.

After any of them every pending and later probe call or `call` is refused
at once ("the engine stopped: …"), so nothing on the page waits for an
answer that will not come.

## Source maps

Every bundle `og build` makes has its source map (`bundleGame`,
`packages/engine/src/node/build.ts`). Its `sources` name the game's files
relative to the project (`src/main.ts`), the engine's as
`@origingame/engine/<path>`, never a path of the machine that built it;
each source's text is inside (`sourcesContent`), except the generated
Worker entry (`og:worker`). The bytes depend only on the inputs.

- **Dev build.** `game.js.map`, `play.js.map` and `audio-worklet.js.map`
  beside their bundles, each linked with `//# sourceMappingURL`. Devtools
  show the TypeScript, and the error card maps its stack before drawing it.
- **Publish build.** The same maps under `sourcemaps/`, minified bundles
  that do not link them, not in og-build.json's `files` and `version` nor
  the service worker's cache. The platform takes `sourcemaps/` out before it
  uploads the release to the game's origin and stores it privately under
  `sourcemaps.build`. A host that serves the directory as is must leave
  `sourcemaps/` out.
- **Native runs.** `og run`, `og dev`, `og test`, playtests, `og ui` bundle
  the game with its linked map beside it (`.og/game.native.js.map`), so
  every error a native run prints names the `.ts` file, line and column. A
  hot-reloaded bundle carries its own map.
- **Mapping a stored stack.** `@origingame/engine/sourcemap` has no I/O and
  no dependencies, and runs under bun, node, a browser and a Worker:

  ```ts
  import { mapStack } from "@origingame/engine/sourcemap";
  // report.stack: the crash report's frames; the maps the release stored under sourcemaps/.
  const text = mapStack(report.stack.join("\n"), { "game.js": gameMap, "play.js": playMap });
  // "at drawWater (game.js:1:48213)" → "at drawWater (src/main.ts:10:31)"
  ```

  `mapStack(stack, maps)` rewrites each frame (V8's `at f (url:l:c)`,
  JavaScriptCore's and SpiderMonkey's `f@url:l:c`) whose file has a map and
  leaves every other line as it was. Maps are looked up by the frame's file
  name, so the serving origin does not matter; `maps` may also be a
  function from a frame's location to its map. `SourceMap` answers
  `original(line, column)` for one position, and `stackFrames(text)` turns
  an error's text into report frames, as the shell does. These signatures
  are a contract with the platform: a change to them is a break.

## Telemetry

Field telemetry is what the shell tells the Player about a session beyond
its lifecycle. It rides protocol 2's `crash` and `sample` on both
transports: `postMessage` to a framed Player, and the top-level page's
`report` (with its batching: 20 messages and 64 KiB per report, 200 unsent
at most). `publish { telemetry off }` (the page configuration's
`telemetry`) measures and sends none of it: the Worker counts no frames,
the page reads no adapter or memory, and only `start`, `ready`, `fail`, the
boot `sample` with `renderer` and `timings`, `crash` with `name` and
`fingerprint`, `heartbeat` and `end` go.

**The GPU is the engine's own device.** The page never asks for an adapter
of its own (a second adapter may be another GPU and costs every load).
With telemetry on, the Worker reads the device it renders with after
attaching the canvas and sends it with `ready`: its limits and features
from the wasm (`og_web_gpu`: the WebGPU names of what the device was
created with, and whether the adapter is a fallback one) and the adapter's
`vendor` and `architecture` from `GPUDevice.adapterInfo` of the canvas
context's configured device (`getConfiguration()`). Where a browser has
neither, `vendor` and `architecture` are empty.

**A lost device is made again on the same canvas** (iOS Safari after the
background or memory pressure, a GPU reset, `loseDevice` in tests). The
first frame that finds it lost asks for a new adapter and device on the
same OffscreenCanvas (frames draw nothing until it lands; the simulation
goes on); the next frame uploads the whole scene again from the engine's
CPU copy. When the new device takes other texture encodings, each Basis
texture goes back to the Worker's transcoder (`og_web_set_transcoder`) and
draws as the streaming placeholder until the new encoding lands.

**Nothing personal.** A browser is its family, major version and OS family
(never the user-agent string); a device is its class and the figures any
page reads (`hardwareConcurrency`, `deviceMemory`, DPR to a quarter); a GPU
is the adapter's `vendor` and `architecture` and the limits and features
below (never its `description` or driver); a stack is its frames with each
location cut to the file name (never an origin, path, query or the error's
message, of which only the fingerprint leaves); a load failure is a path
under `cooked/` and a reason code. No identifier is added: `session` and
`device` are the transport's (`shell/telemetry.ts`).

**Throttled.** The Worker counts each played frame into fixed bins
(`src/web/frames.ts`: no allocation, no message per frame); the page sends
the counts in a `sample` at most once a minute while playing and when the
page is hidden (at least 10 s after the last), never more than 60 a
session. At most 20 load failures go per sample and 100 a session.

| Message | Fields added (all optional to the play origin) |
|---|---|
| `crash` | `cause` (`panic`, `error`, `hang`), `tick` (the world's, or null), `lane` (a panic's lanes site), `location` (a panic's `crates/…:line:col`), `stack` (string[], at most 48 frames of 240 characters, `at f (game.js:1:2345)` or `f@game.js:1:2345`), `engine` (version), `build` (og-build.json `sourcemaps.build`), `browser { name, major, os }`, `device { class: phone \| tablet \| desktop, cores, memory, dpr, touch }`, `gpu { vendor, architecture, fallback, limits: { maxTextureDimension2D, … }, features: string[] } \| null`, `memory { linear, heapLive, heapPeak, streamUsed, streamBudget, audio, entities, gpuTotal, gpuTextures, gpuTargets, deviceLosses } \| null` (bytes; entities a count; the gpu figures ProbeMemory.gpu; deviceLosses the devices lost and recovered) |
| `sample` `phase: "boot"` (once, at the first frame) | beside `renderer` and `timings`: `stages` (the Worker's boot in ms: `wasm`, `bundle`, `create`, `attach`, `transcode`, `prime`, `settle`, `present`), `pipelines { mode, reason }` (§Pipelines), `lanes { on, threads }`, `engine`, `build`, `browser`, `device`, `gpu`, `memory`, `failures` |
| `sample` `phase: "unsupported"` (once, before `fail`, on a browser without WebGPU: no `navigator.gpu`, or no adapter) | `renderer: "none"`, `reason` (`webGpuMissing`'s words), `engine`, `build`, `browser`, `device`, `gpu: null`; the page shows the card that says why, `fail` is `engine_start` and not retryable, and the engine never starts (`tests/web/unsupported.ts`) |
| `sample` `phase: "play"` | `frames { count, ms, bins, interval, work, hitches, longHitches, max }` (`bins`: the 11 upper edges in ms, 8.4 to 250; `interval` and `work`: 12 counts each, the last above 250 ms; a hitch is an interval of 50 ms or more, a long one 250 ms or more), `memory`, `failures [{ file, reason: http_<status> \| timeout \| network }]`, `session { ms, samples }` |
| `sample` `phase: "end"` (on `pagehide`) | `memory` as last read, `failures` not yet sent, `session` |

`build` names the maps that resolve a stack's frames (§Source maps). The
session's start and end are the transport's `start` and `end`
(`durationMs`, `ready`, `failed`).

**The platform's side** (`docs/platform-asks.md` #17): the play origin's
`report` endpoint and the framed Player keep these fields when they file
`crash` and `sample`, and must not answer 400 for a message that carries
them (a report answered 400 is dropped whole). It stores each publish
build's `sourcemaps/` privately under `sourcemaps.build` and maps a crash's
`stack` with `mapStack` when it shows the report.

## project.ogt `publish`

```text
publish {
  probe player            # player | all (checked; the probe set follows --publish, see below)
  out "dist"
  isolation preferred     # required | preferred | off
  fit fill                # fill | letterbox
  aspect "16:9"           # letterbox: width:height (required with letterbox)
  aspect_min "16:10"      # fill: the narrowest shape it fills (4:3 gets bars above and below)
  aspect_max "21:9"       # fill: the widest shape it fills (32:9 gets bars beside it)
  orientation any         # any | landscape | portrait (manifest; locked in fullscreen)
  dpr_cap 2               # 0.5–4
  fullscreen off          # off | gesture
  gate tap                # tap | none
  made_with true          # the "Made with Origin Engine" badge
  icon "assets/icon.png"  # square PNG ≥ 192 px; absent: the brand tile
  short_name "Runner"     # the installed app's name
  color_textures etc1s    # etc1s | uastc: this export's colour textures (default: cook's)
  telemetry on            # on | off: field telemetry (§Telemetry)
  pipelines auto          # auto | sync | async | prewarm (§Pipelines)
}
```

`og build` checks every value first (`crates/og-cli/src/publish.rs`); a bad
one, an unknown key, `fit letterbox` without `aspect`, `aspect_min` wider
than `aspect_max`, or either with `fit letterbox` names its line and the
values it takes. Every key is optional; the defaults are the values shown,
with no aspect, icon or short name. `probe` is checked but chooses nothing:
a `--publish` build serves the player members and a dev build every member.
`color_textures` overrides project.ogt's `cook { color_textures }` (default
etc1s) for `og build` and `og run --device`, so a game that cooks UASTC
natively can ship ETC1S on the web.

A build replaces its output directory (`out`, or `--out`) whole, so it
refuses one that is the project or an ancestor of it, holds a
`project.ogt`, or is not empty and holds no `og-build.json` from an earlier
build; a build that fails after replacing it removes what it wrote.

### Pipelines

How the frame's render pipelines compile on the web (`src/web/pipelines.ts`):

- `sync`: each render pipeline is made with `createRenderPipeline` when a
  pass first draws with it (og-render `deferred.rs`), so the first frame
  compiles only what it draws. After the first frame the Worker warms the
  rest behind the play gate (`og_web_warm`, in idle time), with an empty
  submit after each so the GPU process compiles it then, at most three in
  flight (`WARM_AHEAD`), until none is left or the tap starts play; a
  pipeline the warm-up did not reach is made by the frame that first draws
  with it.
- `async`: the frame's pipelines are asked for with
  `createRenderPipelineAsync`. A pass whose pipeline still compiles skips
  its draws, the first frame is drawn again until none does, and only then
  does the gate show. The renderer opens its frame span around the passes
  it redraws every frame (og-render `FrameSpan`, `FramePass::redrawn`);
  passes whose output later frames keep (shadow maps, captures, IBL, mips)
  make theirs at once. A harness's `frame` call and a probe `capture` or
  `sweep` are always drawn whole.
- `prewarm`: the frames make their pipelines synchronously as `sync` does,
  and only the warm-up behind the gate after the first frame
  (`warmBehindGate`, `pipelines.warming`) is asynchronous, at most six
  compiling at once. A frame that meets a pipeline still compiling makes it
  at once and skips no draw. The warm-up before the first frame stays
  synchronous. Opt-in.
- `auto` (the default) picks when the Worker meets the adapter (`choose`):
  `async` when `GPUAdapterInfo.vendor` (or its `architecture`) names a
  mobile GPU (`img-tec`, `arm`, `qualcomm`: PowerVR, Mali, Adreno), `sync`
  for any other vendor. A browser that names none (Firefox) is `async` when
  `navigator.userAgentData.mobile` says so, else `sync`. Apple GPUs stay
  `sync`, an iPhone's included, until Safari on iOS is measured. The reason:
  a mobile GPU process that compiles synchronously holds the page without a
  frame for tens of seconds, while on a desktop Dawn's asynchronous compile
  makes a cold boot several times later.

A mode written in project.ogt is kept on every GPU. The mode run and why
(`auto: a mobile GPU (img-tec d-series)`, `publish { pipelines sync }`) are
`mode` and `reason` on `__runtime_probe__.boot.pipelines` (which also
counts the pipelines made each way, at the first frame and when the warm-up
ends or the tap stops it), the Worker's console (`[og] pipelines async: …`)
and the boot telemetry sample's `pipelines` (§Telemetry).

## The shell

- **Boot**: the brand symbol, white on ink `#14161b`, plays `genesis` once,
  then `progress` with the real value: bytes of the wasm and of `cooked/`
  received (streamed, counted in the Worker), never backwards. Motion never
  delays the game: progress shows after genesis or after 1.2 s, whichever
  comes first, a fast load skips straight to the game, and
  `prefers-reduced-motion` skips genesis.
- **Play gate**: the first screen is drawn behind it (the shell takes the
  game's first tick, which builds its UI, then holds the clock). The
  platform's answers to what that tick asked are applied behind the gate
  too (`src/web/settle.ts`): the Worker waits for the page's answers to the
  first tick's calls and takes one tick to apply them, then again for the
  calls that tick made, until a tick leaves nothing to apply, at most 8
  ticks (`SETTLE_TICKS`). It waits up to 300 ms (`SETTLE_FIRST_MS`) before
  the first frame, so a store that answers quickly is in the first frame;
  later answers are applied and drawn as they land, up to 5 s after the
  first tick (`SETTLE_MS`). So a returning player's title is in the saved
  language with Continue on behind the gate (`tests/settle.test.ts`). The
  frame is a still a few ticks in, so a game builds the screen it opens on
  settled (`data-og-instant`). These are the game's own ticks; the gate's tap ends
  the settling. The first tap, click, Enter or Space unlocks audio (one
  AudioContext), asks for fullscreen when `fullscreen gesture` and the page
  is not framed (and locks `orientation`), takes a screen wake lock and
  starts the clock. `gate none` starts at once; audio then unlocks on the
  first gesture.
- **Layout**: `fill` covers the whole `100dvh` viewport and passes the
  safe-area insets to the UI (`og.ui` viewport); `letterbox` fits `aspect`
  inside the safe area on ink. A `fill` game with `aspect_min`/`aspect_max`
  fills every shape between them and is letterboxed at the bound past it
  (`layoutCanvas`); without them it fills 4:3 to 32:9 and the camera widens
  its view (Hor+: the vertical field of view is held), while og-ui keeps
  a HUD document placed at `env(-og-hud-inset-*)` in a centred band no
  wider than `UiDesc.hudMaxAspect` (21:9 by default) and never stretches
  a widget. The canvas is drawn at
  `min(devicePixelRatio, dpr_cap)`; resizes, orientation changes and a
  change of devicePixelRatio (`watchPixelRatio`) resize the render target
  in the Worker (`og_web_resize`) and og-ui's scale before the next frame.
- **Graphics settings** (og.render.setDisplay): the preset, render scale,
  frame cap and auto quality apply at once. Vsync is always on in a browser
  (`display().vsync`; a frame cap under the refresh rate paces below it).
  `fullscreen` enters fullscreen on the next press when the browser needs a
  gesture, leaves it at once, and follows the player's own Escape
  (`fullscreenchange`). `powerPreference` (`highPerformance`, `lowPower`)
  picks the adapter, which the browser chooses only when the device is
  made: the shell keeps it in localStorage (`og.display:<page path>`) and
  passes it at the next load; until then `display().restartNeeded` is true
  and the page says "Applies after a restart".
- **Pointer lock** (`src/web/pointer-lock.ts`, og.PointerLock and
  og.input.setMouse): a wanted lock is requested on the next press on the
  canvas, with `unadjustedMovement` (raw input) unless `MouseDesc.raw` is
  off. A browser that rejects the option (NotSupportedError) is asked again
  plain in the same gesture, and one that ignores it gets the plain lock;
  both set `og.input.mouse().rawUnsupported`. Escape and focus loss release
  the lock and re-arm it; a refused request re-arms too, an unanswered one
  is asked again by a press after a second, and a lock granted after the
  game stopped wanting it is released at once: no sequence leaves a hidden
  cursor the game cannot use.
- **Keyboard layout** (`src/web/keyboard-layout.ts`): bindings stay on
  physical keys (`KeyboardEvent.code`); what each key prints reaches
  og.input.setKeyboardLayout, so glyphs and prompts read Z Q S D on AZERTY.
  Where `navigator.keyboard.getLayoutMap()` exists it is read at load, on
  focus and on a layout change; elsewhere each key is learnt from its first
  unmodified press, showing the US label until then.
- **Visibility**: a hidden page (and `pagehide`) pauses the clock, suspends
  audio and releases the wake lock; showing it resumes all three.
- **Input** (`src/web/shell.ts`, `src/web/gamepad.ts`): keyboard
  (`KeyboardEvent.code`; keys that would scroll the page or move focus are
  kept from it; nothing is taken while an input field has focus); pointer
  events of every type with pointer capture, up to five held at once: each
  pressed pointer takes the lowest free engine id, 0 (the primary pointer
  `og.Input` follows) while none is held, then 1 to 4, and `og.Pointers`
  reports each with its position and press point; a hovering mouse moves
  pointer 0; wheel. Gamepads are polled on the display frame while one is
  connected: the first four standard-mapping pads reach `og.Gamepad` whole
  whenever they change, and every pad also presses the keys of its
  standard mapping (A Space, B Escape, X KeyX, Y KeyZ, LB KeyQ, RB KeyE,
  Back Tab, Start Enter, d-pad and the left stick past 0.5 the arrows);
  og-ui counts a pad intent and the key it maps to once.
- **Error card** (design §6): a failed start (the Worker's `error`, a
  Worker that fails to load or says nothing for `BOOT_DEADLINE_MS`, two
  minutes, during the boot, an engine that stops before its first frame)
  or a crash after it (§Errors) shows the reason, the last 200 lines of the
  engine's log, the Worker's console and the page's errors, and the world
  dump (a dev build) or the state (a publish build), with Reload and Copy
  report; the brand symbol switches to `drift`.
- **PWA** (publish builds, in a secure context): the service worker
  precaches `./` and every file of the build except `og-build.json`,
  `_headers`, `origingame.runtime.json`, licence and README files, the
  streaming packs, the engine variant the page will not take (§Lanes) and
  itself, under `og-<scope>-<version>` (`<scope>` a hash of the
  registration's scope, so builds side by side on one origin keep their own
  caches; `<version>` og-build.json's `version`'s first 16 hex digits; a
  failed install keeps the old build). On activation it drops only this
  scope's other caches (and an older unscoped `og-<version>` cache that
  holds this scope's `./`). It serves the page from the network first and
  from the cache offline, and every other file from the cache. The network
  page is the browser's own navigation request (navigation preload, enabled
  on activation), because a worker's own `fetch` of a navigation leaves as
  `Sec-Fetch-Dest: empty` and the platform gives its derived entry (with
  `window.__OG_GAME`) only to a document or frame load; the worker fetches
  the page itself only without navigation preload. A pack is not
  precached: it comes from the cache, or from the network into the same
  cache the first time the engine fetches it, so a visited place also
  plays offline.
- **Incremental update.** `OG_SW.hashes` names each file's content (the
  first 16 hex digits of its sha256), and each cache keeps that index under
  `og-sw-index.json`. A new build's worker copies into its cache every
  file, precached or lazy, whose content one of this scope's older caches
  holds under any name, so a visited pack and an unchanged engine are not
  fetched again. It fetches only the rest, and the page (`./`,
  `index.html`) every time. A release that changes game code downloads the
  game bundle, the cook manifest and the outputs that changed
  (`tests/sw-update.test.ts`).
- **Install hint**: shown only when not framed, not installed, with a
  manifest and a service worker, and either Chromium's
  `beforeinstallprompt` (an Install button calling the event's `prompt()`)
  or iOS Safari (Share, then Add to Home Screen); "Not now" is remembered.
  Chromium fires the event once, often before the engine has booted, so a
  script in `index.html`'s head keeps it for the shell.
- **Language**: the shell's own words (loading, the gate, the error card,
  the install hint) are Chinese when `navigator.language` is `zh*`, else
  English.
- **Marks** for tests and telemetry: `performance` marks `og:ready`,
  `og:first-frame` and `og:start`; where the Worker's boot went is
  `__runtime_probe__.boot` (src/web/worker.ts `BootPhases`: `wasm`,
  `bundle`, `create`, `attach`, `transcode`, `prime`, `settle`, `present`,
  ms each), sent with the first frame.
- **WebGPU only** (design §8): a browser without `navigator.gpu`, or whose
  adapter or device cannot be had, gets the shell's unsupported-browser
  state instead of an engine; there is no WebGL2 fallback.
- **URL parameters**: `replay-seed=<u32>`, `autoplay=0` (a harness drives
  the clock through the probe: no gate, no first tick, no service worker),
  `trace` (§The og-probe wire).

## The Player and Stage contract

### Player protocol 2 (`origingame/docs/player-protocol.md`)

The shell says `hello` (`substrate: "origin-engine"`) at startup and owns
its readiness: `loading` with the byte fraction, `ready` at the first
rendered frame (or two seconds after the boot when nothing renders), `fail`
(`engine_start`, retryable) before it, `crash` (`EngineCrash` and a
fingerprint of the message, plus §Telemetry's fields when on) after it, and
`sample`s: the boot's at the first frame, with `renderer` (the tier),
`timings.firstFrame` and `timings.boot`, and the frames played
(§Telemetry). A framed page (and the Stage overlay's own document, below)
waits 300 ms for a Player's `hello` before drawing its own gate; under a
Player that answered, the shell draws no gate: it plays on `start` (or a
hello with `started: true`), pauses on `pause`, resumes on `resume`, and
resumes its AudioContext on `gesture`. Framed, it posts only to
`window.parent` at its own origin and hears only `window.parent` at its own
origin with `og: "player", protocol: 2`.

Who the shell speaks protocol 2 to, in order:

1. framed: `window.parent` (above);
2. top level with the platform's `window.__OG_GAME`: its play origin over
   HTTP (§The top-level page);
3. top level, no `__OG_GAME`, and told its own origin as the Stage host
   (`window.__OG_STAGE_HOST__ === location.origin`, decided as the og-probe
   wire decides it, `shell/wire.ts` `stageInPage`): **its own window**.
   That is a Stage build opened as its own document with the platform's
   Stage overlay injected, and the overlay is its Player. The shell posts
   to `window` at its own origin and hears only `window` at its own origin;
   a `hello` that names a `substrate` or lists no `capabilities` (the
   shell's own echo) is not taken for the Player's. The hello repeats every
   250 ms until the overlay answers (at most six). The overlay's `hello`
   with `started: true` starts play (no gate of the shell's), its `pause`,
   `resume` and `gesture` count, and og.platform's calls go to it as below.
   On the Stage nothing of og.platform's goes to the browser's storage:
   where the overlay answers nothing (no hello within 1.5 s, or a method it
   does not list), the page's memory does, for that load. No service worker
   is registered there;
4. otherwise no one: the page plays alone on the browser's storage.

### Platform services over protocol 2

`og.platform` (IDL module `platform`, docs/design.md §2.8) reaches the
Player through the same channel (`shell/platform.ts`). The engine Worker
hands each tick's calls to the page (`{ type: "platform", calls }`); the
page answers them (`platform-answer`) and the answers are applied at the
start of the next tick (behind the play gate, §The shell). A framed page
(or one with the Stage overlay in its own document) holds its first calls
until the Player's `hello`, or 1.5 s, so they never reach the wrong store.

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
`save.read`/`save.get` and `save.set` but no `store.*` (an older Player)
keeps the store in the save's `store` object, read with `save.read` and
written back with `save.set { data, revision }` at the revision it was
read at; one with `leaderboard.top` but no `leaderboard.around` answers
`around` from the top 100; a method the hello does not list answers
`unavailable`; and where the Player keeps no store (none listed, or
`unavailable`) the browser's `localStorage` (`og.platform:<path>`) does.
Unframed, every call is answered from there (`src/platform-local.ts`, as on
the native host). Errors keep the Player's codes, `retryable` and a
conflict's `revision`. The Stage overlay in the page's own document is
answered the same way, with the page's memory instead of `localStorage`.
Tests: `tests/platform-player.test.ts` (the bridge against a stand-in
Player, an older one, and none), `tests/platform-stage.test.ts` (the
own-window path), `tests/web/player.ts` (in Chrome, framed and as a Stage
build, at desktop and phone sizes).

The host's clock (`og.platform.now`, every answer's `time`) is the Worker's
`Date.now()` when the platform attaches and when an answer arrives, between
ticks, and `og.platform.info(world).clock` reads `browser`. A `result` that
carries a numeric `time` (UTC ms), from any transport, has it passed on as
that answer's `time` (a store kept in the save takes the time of the call
that settled it: `save.set`'s, else `save.read`'s), and `clock` then reads
`player` (`docs/platform-asks.md` #9).

### The top-level page (Player protocol 2 over HTTP)

The platform opens a published page whose build says `og-player` … `page`
as the top-level document on the game's own origin (origingame
`docs/player-protocol.md` §The top-level engine page): no Player page
frames it. The shell (`shell/transport.ts`) then speaks protocol 2 to its
play origin itself, when the page is not framed and carries the platform's
`window.__OG_GAME` (`{ id, releaseId, protocol: 2 }`, added to the entry at
publication). A framed page keeps the `postMessage` path, and a top-level
page without `__OG_GAME` plays alone on the browser's storage.

- **Requests.** `POST /__player/v2/<id>/<releaseId>/hello`, `call` and
  `report`, same origin, `credentials: "omit"`, one at a time through one
  queue (so the page mints at most one guest), each with `Authorization:
  Bearer` once the page holds a credential. A `call` is the protocol-2
  `call` message and its answer the `result`; 15 s without an answer is
  `timeout`, a failed fetch `offline` (both retryable), any other status
  the protocol's code for it (`404` `not_found`, `429` `rate_limited`,
  `5xx` `unavailable`). og.platform's mapping is the framed one, from the
  HTTP hello's `capabilities`.
- **Identity.** The Portal's hand-off code (`#og-h=<code>`) is removed from
  the address before anything else (`history.replaceState`; the rest of the
  fragment stays the game's) and sent only in the hello's body. The answer's
  `player { token, guest }` (from the hello, or from any call or report that
  minted a guest or replaced a rejected credential) is the credential: a
  guest's is kept in memory and under `og_token_<id>`, an account's in
  memory only with the `og_portal_<id>` marker, the framed Player's keys and
  rule (a later open without a code resumes no account). Before a code is
  redeemed both keys are removed. The credential is never in og.platform's
  storage, a URL, the error card or a report.
- **The outbox.** An unconditional `save.set` and a `leaderboard.submit`
  that fail retryably are kept under `og_outbox_<id>` (`{ save?: { data },
  scores, at }`, 7 days, the latest save, the last 20 scores in order) and
  answered `{ ok: true, queued: true }` / `{ ok: true, rank: null, queued:
  true }`; they are delivered after the next answered call and after the
  next hello. Store writes and conditional writes are never queued.
- **Reports.** Nothing before the person starts play at the shell's own
  gate (the hello says `started: false`: the page owns its gate). Then
  `start`, what was held (`ready`, `fail`, the `sample`), and after it
  `crash`, the game's telemetry as `event` `game.custom.<name>`, a
  `heartbeat` every 60 s once ready and `end` on `pagehide` (`keepalive`,
  with `durationMs`, `ready` and `failed`). Each message is a protocol-2
  message with `id` (a UUID v7), `at` (epoch ms) and `ms` (ms since
  navigation start); a report carries `session` (`game_session_<32 hex>`,
  per load) and `device` (`device_<32 hex>` from `og_device_<id>`, 30 days,
  the Player's). A report holds at most 20 messages and 64 KiB (UTF-8;
  `end` sends the newest that fit). A report that fails is kept (at most
  200 messages) and sent with the next; one answered 400 is dropped, and
  one answered 413 is halved, a single message that still does not fit
  dropped.
- **The brand surface** is the shell's at the top level (design §4, §9):
  the boot splash on ink with the cover's 32 px derivative upscaled and
  blurred behind it (the hello's `page.backdrop`, `crossorigin="anonymous"`
  on an isolated page), the play gate, `100dvh` and the safe-area insets,
  the PWA install hint, the badge, and **Back** (top left inside the safe
  area; full on the boot splash, the gate and the error card, a small
  handle while playing): `history.back()` when the referrer is on the
  hello's `page.portal` and there is history, else `page.back`, the game's
  page on the site. There is no purchase hand-off: protocol 2 has no
  commerce store.

Tests: `tests/platform-page.test.ts` (the transport against a stand-in play
origin), `tests/web-shell-crash.test.ts` (a crash report's fields end to
end), `tests/sourcemap.test.ts`, and `tests/web/player.ts` (the runner's
publish build at the top level in Chrome against a stand-in play origin).

### The og-probe wire (`origin/packages/contracts/src/probe-wire.ts`)

The shell serves the wire itself (`shell/wire.ts`) over the engine's probe,
which lives in the Worker and answers asynchronously:

- It answers `{og: "probe", v: 1, kind: "call"}` only from `window.parent`
  (else `window.opener`) at the origin in `window.__OG_STAGE_HOST__`; a page
  told no origin, or another origin, or a call of another `v`, gets nothing.
  A top-level page told its own origin also answers its own window (and
  speaks Player protocol 2 to it, §Player protocol 2): the Stage overlay
  drawn in the engine's document, which posts its calls to `window`. Every
  post names that origin. It posts `hello` on load, when the told origin or
  the probe's availability changes, and after a `hello` call.
- Shapes: `hello` `{protocol: 5, substrate: "origin-engine"}` (both `null`
  before the engine is ready); `state`, `pause`, `resume`, `advance`
  `{tick, paused, timings: {frameMs}}`; `entities` `{items: [{id,
  label?}]}`; `bounds` `{items: [{parameter, label?, path?, min?, max?,
  step?, value?}]}`; `entity` `{id, params}`; `patch` `{applied: true,
  roundTripMs}`; `capture` `{tick, width, height, png}` from the engine's
  own readback, fitted into `width`×`height` without upscaling; `record`
  `{mime, base64, durationMs}` from the canvas's capture stream.
- Bounds: `advance` takes 0 to 3600 steps, `capture` a width and height
  within 0–4096, `record` up to 15 seconds and one recording at a time.
- Errors: `refused` for a probe refusal (`false`/`null`) and parameters out
  of bounds; `unsupported` for an unknown method (including `input`, which
  the wire does not carry), no probe yet, and every member a publish build
  drops (`pause`, `resume`, `advance`, `bounds`, `patch`); `failed` for a
  fault.
- A dev (Stage) build serves every member; a publish build the 7 player
  members. `globalThis.__runtime_probe__` (`version: 5`, `substrate:
  "origin-engine"`, every member a Promise) stays for in-page callers.
- `listen` (dev builds): the last seconds (at most 2) of what the engine
  Worker's mixer rendered, `{tick, rate, frames, peak, rms, wav}` with a
  base64 16-bit stereo WAV; the command channel's
  `listen[:path][:seconds]` writes the same as a `wav` output; before the
  page's audio is attached it answers `null`.
- `memory` (dev builds, both hosts; IDL `EngineProbe.memory`,
  `ProbeMemory`, bytes): `host`, `linear` (the wasm linear memory, which
  the browser never takes back; null natively), `resident` (the process's,
  natively; null on the web), `heap` (`live`, `peak`, `allocations`: the
  counting global allocator, og-core's `alloc`), `world.entities`, `stream`
  (`used`, `peak`) and `audio` (the sound bank's samples). Every build's
  page answers the same from the engine Worker
  (`__runtime_probe__.memory()`, `og_web_memory`), a publish page included,
  whose probe has no `memory`.
- `trace`: the run's Chrome trace, the document `og run --trace` writes
  natively (`src/trace.ts`), buffered in the engine Worker, which takes the
  engine's spans (`og_web_trace_take`) after every frame and places them on
  the page's clock. `?trace` records from the boot;
  `__runtime_probe__.trace("start" | "stop" | "take")` begins, ends or
  reads it; `saveTrace(name)` downloads it. The GPU's passes are there
  where the adapter has `timestamp-query` (`RenderCapabilities.gpuTimestamps`;
  never in a publish module), a frame or two after the frame
  (`tests/web-trace.test.ts`).

`bun run test:browser` in `packages/engine` checks the wire in headless
Chrome (`tests/web/wire.ts`, dev and publish, isolated and not),
`tests/web/player.ts` and `tests/web/export.ts`; these need a Chrome and
are not in the gate. `tests/web-export.test.ts` (layout, manifest,
determinism, icons) is.

## The `og` commands around the export

- `og build [project] [--publish] [--out DIR] [--json]` cooks the project,
  checks project.ogt (above) and writes the directory into `--out`, else
  `publish.out`, else `<project>/dist`. It prints each file's bytes and
  brotli `-q 11` size (gzip `-9` without the brotli CLI), then what a player
  downloads (`downloadsOf` in `src/node/web-build.ts`): the boot of the
  lanes page and of the single-threaded page (one engine module each, the
  shell, the Basis transcoder and the cooked first group without the later
  chunks of a chunked sound) and what comes later (the streaming packs, the
  sound chunks, the audio worklet, the service worker and icons); the
  table's `total` is every file and no download. Then the probe build and
  members read from the copied wasm, and the manifest's version; `--json`
  prints one JSON line instead (with `downloads`). The web host comes from
  `packages/engine/dist/web` or `dist/web-publish`; a checkout without them
  is told to run `bun run build` in `packages/engine`.
- `og run` (the native host under bun) supervises the game process: when it
  dies with a failure, `og run` says at once how (its exit status, or the
  signal), whether its probe had been serving (`--port`), and its last
  stderr lines, and exits with the process's code (128 + the signal).
- `og run --device chrome|safari|android[:serial]|iphone[:udid] [project]
  [--seconds N] [--port P] [--out DIR] [--size WxH] [--headed]
  [--chrome <path>] [--cert <pem> --key <pem>] [--host <ip>] [--json]`
  exports the publish build into `<project>/.og/device`, serves it with its
  `_headers` and COOP/COEP/CORP on 127.0.0.1 (on 0.0.0.0 only for the
  iPhone, which reaches it over the LAN), opens it in the target, passes the
  play gate with a real tap and times the page. Every load is cold:
  responses are no-store, `sw.js` is answered 404, and on a DevTools target
  (`chrome`, `android`) the origin's storage is cleared first. `chrome` is
  this machine's Chrome over the DevTools protocol (`--chrome`, `CHROME`,
  `CHROME_PATH`, the agent-browser install, /Applications, PATH; headless
  unless `--headed`); `safari` is this Mac's Safari over WebDriver
  (safaridriver; `sudo safaridriver --enable` once), its viewport `--size`;
  `android` is Chrome on a phone over adb (`adb reverse` puts the page at
  the phone's own localhost, a secure context, and `adb forward` reaches
  its DevTools); `iphone` (or `ios`) is Safari over safaridriver at the LAN
  URL, a secure context only over HTTPS (`--cert`/`--key`, from mkcert;
  refused for `chrome` and `android`). The LAN address is a private (RFC
  1918) one on an Ethernet or Wi-Fi interface before tunnels, bridges and
  VM interfaces, never link-local; `--host <ip>` names it instead, and the
  address used is printed. SIGINT and SIGTERM undo the run's adb forwards,
  Chrome profile, WebDriver session and server, then exit 130 or 143. It
  reads `og:first-frame`, `__runtime_probe__.hello()`'s tier and, after a
  second of play, `__og_timings__` over `--seconds` (default 5): the
  Worker's display interval and engine work per played frame as count,
  mean, p50, p95, p99 and max. `report.md` and `device.json` (default
  `<project>/og-out/device`) have the measures (no score), findings with a
  fix each (a first frame over the design §7 budget of 2.0 s, 1.5 s on the
  iPhone, missed frames, work over 16.7 ms, no isolation, console errors),
  the device (user agent, platform, tier, WebGPU adapter, DPR, viewport,
  canvas, cores, memory, isolation) and the headers served. A missing
  Chrome, adb or safaridriver is told in one line before the build; the
  phone paths' prerequisites are in tests/evidence/P3/device/NOTES.md.
- The publish build has no command channel, so its scripted check runs
  through what it keeps: `bun tests/web/published.ts --project <dir>
  [--no-build] [--no-isolation]` in `packages/engine` publishes the project,
  serves it as the platform opens a release (`tests/web/play-origin.ts`:
  the top-level page with `window.__OG_GAME`, a stand-in play origin
  answering Player protocol 2 over HTTP with a store, the build's
  `_headers`), and plays a script of the playtests' syntax in headless
  Chrome: input steps (`key_down`, `key_up`, `type`, `touch_move_to` and
  `move_to_text` found in the accessibility mirror, `touch_move`,
  `touch_start`, `touch_end`, `scroll`) as real input events, `screenshot`
  as the probe's `capture` compared with the project's references (its
  tier's, else metal's) under og-tests' rule, `state` and `mirror` as the
  probe's `state()` and the mirror's nodes and focus, and relaunches (a
  playtest's `then`) as reloads under the service worker; a command-channel
  step is refused, never skipped. The engine Worker's display clock is the
  harness's (its `requestAnimationFrame` and `performance.now`, replaced
  through the DevTools protocol before its first line runs), so every step
  lands on its tick and every frame is the tick it names; nothing of the
  build changes. It checks that the game boots, its title is in the mirror
  behind the gate, a real click opens the gate, the frames match, the
  plan's checks hold, and the page, its Workers and Chrome (with its
  accessibility tree on) log no warning or error. Its built-in plan is
  the former test sample's; `--script "<steps>" --title
  <node> [--then "<steps>"] [--references <dir> --prefix <p>]` runs
  another, and `--playtest <file>` a playtest's own commands (its `then` as
  reloads; dump and game-command steps are refused, and its `check` is not
  run). Run by hand, one Chrome at a time; no gate lane runs it.
- `og doctor` (no arguments) checks what the author's loop needs: bun (node
  is optional when bun is present), the engine package, its native addon,
  both web host wasm files, and whether a 64×64 frame renders headless
  (Metal or Vulkan). No renderer is reported as `none`, not as a problem:
  the platform's `engine` and `probe` sandboxes carry none, and there `og
  run`, `og test` and the probe run the same game with each screenshot a
  frame unchecked (docs/ci.md §Switches in the environment). Inside the
  engine repository it also compares rustc, cargo and wasm-bindgen with
  `rust-toolchain.toml` and `Cargo.lock`; outside, those lines say
  `skipped`. Each problem prints one line naming its fix, and the exit code
  is non-zero when there is any.

## Packaging

```sh
cd packages/engine
sh scripts/pack.sh --linux-x64    # OG_RELEASE=1 scripts/build.sh --linux-x64 (this machine's addon and og, the Linux x64 addon and og through cargo zigbuild against glibc 2.31, both web hosts), then dist/pack/*.tgz with sizes and sha256
bun ../../tests/install/instabox.ts --version <v> --from ../../dist/release/<v>   # optional, billed by the minute: the install test inside an InstaBox machine (docs/release.md §4 D; scripts/release.sh verify --instabox)
```

- `@origingame/engine`: the TypeScript API and runtime, the web hosts
  (`dist/web`, `dist/web-publish`, `dist/web-publish-2d`), the shell, the
  Basis transcoder, `bin/og.mjs` (runs the platform package's `og` with this
  package as `OG_ENGINE_PACKAGE`). No native code.
- `@origingame/engine-<platform>-<arch>`: `origin-engine.node` and `og`,
  with `os`/`cpu` (and `libc: glibc` on Linux), as `optionalDependencies`
  of the main package, so only the matching one installs. The loader takes
  `OG_NATIVE_ADDON`, then a checkout's `native/`, then the platform
  package.
- `@tsrs/darwin-arm64`, `@tsrs/linux-{x64,arm64}-{gnu,musl}`: tsrs, `og
  check`'s type checker, with its `lib.*.d.ts`, the same way (pinned at
  `TSRS_VERSION` of `scripts/install-tools.sh`, not at the engine's
  version), so a project's install brings the checker and `og check` finds
  it beside the engine package with no registry call at check time.
- What the packages carry and how a version is released and distributed
  (the signed archives): `docs/release.md` §1–§3.
