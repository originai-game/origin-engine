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
| `index.html` | The player shell's page. Carries `<meta name="og-engine" content="<engine version>">`, `og-build` (`publish`/`dev`), `og-profile` and `og-player` (`frame page`: the Player transports the shell speaks, which the platform reads to open the page at the top level), the inlined CSS and the page configuration (`<script type="application/json" id="og-config">`). |
| `play.js` | The shell (`packages/engine/shell/page.ts`, bundled). |
| `game.js` | The engine Worker: the runtime and the game. |
| `audio-worklet.js` | The AudioWorklet, loaded on the first gesture. |
| `game.js.map`, `play.js.map`, `audio-worklet.js.map` | A dev build's source maps, beside their bundles and linked from them (§Source maps). |
| `sourcemaps/` | A publish build's source maps of the same three bundles, unlinked and not to be served (§Source maps). |
| `origin-engine.js`, `origin-engine_bg.wasm` | The web host from wasm-bindgen: `dist/web` in a dev build, the publish variant of the project's `profile` with `--publish` (§Variants). |
| `origin-engine-lanes.js`, `origin-engine-lanes_bg.wasm`, `snippets/` | `--publish` with `isolation` not `off`: the lanes variant of the same profile and wasm-bindgen-rayon's Worker script (§Lanes). |
| `basis/` | Binomial's Basis transcoder and its licence. |
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
  content-hashed file's own name to its served one (`{}` in a dev build).
- `engine.version` is the engine that built it; `engine.pinned` is
  project.ogt's `engine`. `probe` is read from the copied wasm's own
  `hello` (`build` `publish`, or `release` for a dev build).
- `variant` is `3d`, `2d`, or `full` for a dev build (§Variants).
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

### Streaming groups

The Worker fetches the first group (the cook's manifest and every output no
group holds) before the engine starts, and a group's pack only when the
runtime admits the group (IDL module `stream`), transcoding its textures
first so the whole group goes in place at once. A chunked sound's later
chunks are fetched ahead of its playheads.

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
- A Content-Security-Policy must allow `worker-src blob:` (or send no
  `worker-src`/`script-src`), since the lanes pool's Workers start from a
  `blob:` URL; a host that refuses it gets the single-threaded engine.
- `Cache-Control: no-cache` on `/`, `index.html`, `sw.js` and
  `og-build.json`; `public, max-age=31536000, immutable` on the
  content-hashed files.
- Never serve `sourcemaps/` (§Source maps).

`_headers` carries all of these.

### Content-hashed names

A publish build serves every file the page or the engine Worker fetches by
name as `<stem>.<the first 12 hex digits of its sha256>.<ext>`
(`game.3f2a9c1e04b7.js`): `play.js`, `game.js`, `audio-worklet.js`, the
engine glue and modules (single-threaded and lanes),
`basis/basis_transcoder.{js,wasm}` and the AV1 fallback decoder
`av1/og-av1.wasm` (`HASHED_FILES` in `src/node/web-build.ts`). A new build
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

One engine module per download. A publish build ships the variants of the
game's physics profile (project.ogt `profile`), single-threaded and lanes:

| Variant | Where | Built with (og-host-web) | Physics |
|---|---|---|---|
| 3D publish | `dist/web-publish/origin-engine*` | `--no-default-features --features publish,physics3d,shaping` | rapier3d |
| 3D lanes | `dist/web-publish/origin-engine-lanes*`, `snippets/` | the same, wasm atomics (§Lanes) | rapier3d (`parallel`) |
| 2D publish | `dist/web-publish-2d/origin-engine*` | `--no-default-features --features publish,physics2d,profile2d,shaping` | rapier2d |
| 2D lanes | `dist/web-publish-2d/origin-engine-lanes*`, `snippets/` | the same, wasm atomics | rapier2d (`parallel`) |
| dev | `dist/web/origin-engine*` | default features (both profiles) | both, not shipped |

`publish` leaves out the probe's tool members, GPU timestamp queries and
the reflected field docs. `profile2d` leaves out the 3D world
(`RenderCapabilities.world3d` is false), so `og build --publish` refuses a
`2d` project whose scenes hold `Terrain`, `TerrainScatter`, `Water`,
`WaterSurface` or `Fluid3D`, naming each by file and line. The variant is
copied as `origin-engine.js` and `origin-engine_bg.wasm` and recorded as
`variant`. The size budget (design §7, `scripts/budgets.txt`) holds per
variant: what counts is the largest one a player downloads, never the sum
(`scripts/size-gate.sh`, docs/ci.md).

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
| `sample` `phase: "play"` | `frames { count, ms, bins, interval, work, hitches, longHitches, max }` (`bins`: the 11 upper edges in ms, 8.4 to 250; `interval` and `work`: 12 counts each, the last above 250 ms; a hitch is an interval of 50 ms or more, a long one 250 ms or more), `memory`, `failures [{ file, reason: http_<status> \| timeout \| network }]`, `session { ms, samples }` |
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

How the frame's render pipelines compile on the web
(`src/web/pipelines.ts`, `publish { pipelines }`):

- `sync`: a pipeline is made when a pass first draws with it, and the rest
  are warmed behind the play gate after the first frame.
- `async`: pipelines are made with `createRenderPipelineAsync`; a pass
  whose pipeline still compiles skips its draws, and the gate shows once
  the first frame is drawn whole. Probe captures are always drawn whole.
- `prewarm`: as `sync`, with only the warm-up behind the gate
  asynchronous. Opt-in.
- `auto` (the default): `async` on a mobile GPU (`img-tec`, `arm`,
  `qualcomm`, or none named and `navigator.userAgentData.mobile`), `sync`
  otherwise, Apple GPUs included, since a mobile GPU that compiles
  synchronously holds the page for tens of seconds.

The mode and why are on `__runtime_probe__.boot.pipelines`, in the
Worker's console and in the boot telemetry sample.

## The shell

- **Boot**: the brand symbol on ink `#14161b` plays `genesis` once, then
  `progress` with the bytes received. Motion never delays the game:
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
  the engine variant the page will not take and itself, under
  `og-<scope>-<version>` (`<scope>` a hash of the registration's scope;
  `<version>` og-build.json's `version`'s first 16 hex digits), and drops
  only this scope's other caches. It serves the page from the network
  first, through navigation preload so the platform still sees a document
  load, and every other file from the cache, a pack once fetched.
- **Incremental update.** `OG_SW.hashes` names each file's content (16 hex
  digits of its sha256), and each cache keeps that index under
  `og-sw-index.json`. A new build's worker copies into its cache every
  file whose content one of this scope's older caches holds under any
  name, and fetches only the rest and the page (`tests/sw-update.test.ts`).
- **Install hint**: shown only when not framed and not installed, with a
  manifest and a service worker; "Not now" is remembered. The shell's own
  words are Chinese when `navigator.language` is `zh*`, else English.
- **Marks**: `performance` marks `og:ready`, `og:first-frame` and
  `og:start`; `__runtime_probe__.boot` holds the Worker's boot phases.
- **WebGPU only** (design §8): a browser without WebGPU gets the
  unsupported-browser card; there is no WebGL2 fallback.
- **URL parameters**: `replay-seed=<u32>`, `autoplay=0` (a harness drives
  the clock through the probe: no gate, no first tick, no service worker),
  `trace` (§The og-probe wire).

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
`RoomEvent` are og-net's protocol). The vocabulary is data:
`CORE_METHODS`, `HOST_EXTENSIONS` (`point`, `ai.mark`: OriginGame's),
`ERROR_CODES` and `RETRYABLE_CODES`, kept equal to `platform.ogt`'s by
`tests/provider.test.ts`. A provider answers each call once (`telemetry`
never) and anything it does not list `unavailable`;
`checkPlatformProvider(provider)` checks this in a provider author's own
tests.

The page that hosts the game names its providers, never the game:
`play({ provider: { platform?, rooms? } })` (`shell/page.ts`), or, for the
page `og build` writes, `window.__OG_PROVIDER = { platform?, rooms? }` set
by a script before `play.js` loads. `rooms` is a RoomProvider or an `og
relay` URL. Each part takes the place of the default only where no Player
answers: `platform` that of the browser's storage, `rooms` that of no
rooms. A part that is not a provider is left out with a line in the log.
In a publish build a provider whose `host` is `local` answers the platform
services `unavailable`: a published page never fakes a purchase.

`relayRooms(url, { reconnectMs })` (`src/web/relay-rooms.ts`) is the web
room provider for a self-hosted page: og-net's protocol to an `og relay`
over the browser's WebSocket, mended after a lost connection as the native
client mends it (docs/design.md §2.8). A page on https reaches only a
`wss://` relay, so `og relay` needs a TLS proxy in front of it.

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
  player downloads (`downloadsOf` in `src/node/web-build.ts`). Compressed
  sizes and probe reads are cached by sha256 in `dist/build-facts.json`
  for the engine's own modules and in `~/.cache/origingame/build-v1/`
  otherwise (`OG_BUILD_CACHE` names another directory; `off` keeps
  nothing).
- `og run --device` serves the publish build cold with its `_headers` and
  COOP/COEP/CORP, passes the play gate with a real tap, times the page and
  writes the measures and findings (no score) to `report.md` and
  `device.json`; the phone paths' prerequisites are in
  tests/evidence/P3/device/NOTES.md.
- `og doctor` reports no renderer as `none`, not as a problem (docs/ci.md
  §Switches in the environment).

## Packaging

```sh
cd packages/engine
sh scripts/pack.sh --linux-x64    # OG_RELEASE=1 scripts/build.sh --linux-x64, then dist/pack/*.tgz with sizes and sha256
bun ../../tests/install/instabox.ts --version <v> --from ../../dist/release/<v>   # optional, billed by the minute (docs/release.md §4 D)
```

- `@origingame/engine`: the TypeScript API and runtime, the web hosts
  (`dist/web`, `dist/web-publish`, `dist/web-publish-2d`), the shell, the
  Basis transcoder, `bin/og.mjs`. No native code.
- `@origingame/engine-<platform>-<arch>`: `origin-engine.node` and `og`,
  as `optionalDependencies` of the main package with `os`/`cpu` (and
  `libc: glibc` on Linux), so only the matching one installs; and
  `@tsrs/<platform>`, `og check`'s type checker, the same way (pinned at
  `TSRS_VERSION` of `scripts/install-tools.sh`).
- How a version is released and distributed: `docs/release.md` §1–§3.
