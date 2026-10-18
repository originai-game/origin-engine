# Web export

`og build` turns a project into a directory any static host serves: the
player shell, the engine (wasm in a Worker on an OffscreenCanvas), the game
bundle and its cooked assets. This page is the output layout, what a host
must send, project.ogt's publish settings, the shell's behaviour, the
contract the platform's Player and Stage rely on, and how the package is
packed. Design: `docs/design.md` §2.2, §4, §6, §9.

```sh
og build .              # dev build: every probe member, no service worker
og build . --publish    # what ships: the player probe members, minified, the PWA
```

## Output layout

| Path | What it is |
|---|---|
| `index.html` | The player shell: the canvas, the boot splash, the play gate, the error card, the install hint, the badge. Carries `<meta name="og-engine" content="<engine version>">`, `og-build` (`publish`/`dev`) and `og-profile`, the inlined CSS (brand tokens and only the `genesis`, `progress` and `drift` motion rules) and the page configuration (`<script type="application/json" id="og-config">`). |
| `play.js` | The shell (`packages/engine/shell/page.ts`, bundled). |
| `game.js` | The engine Worker: the runtime and the game. |
| `audio-worklet.js` | The AudioWorklet, loaded on the first gesture. |
| `origin-engine.js`, `origin-engine_bg.wasm` | The web host. `--publish` takes `dist/web-publish` (probe trimmed to hello, tick, state, pick, entity, input, capture; `wasm-opt -Oz`). |
| `basis/` | Binomial's Basis transcoder and its licence. |
| `cooked/` | The project's cooked bundle (`og cook`, run by `og build`). |
| `icons/` | `publish.icon` as `icons/icon.png`, or the brand tile at 192 and 512 px. |
| `og-build.json` | The build manifest (below). |
| `_headers` | COOP/COEP/CORP and caching for static hosts (Netlify and Cloudflare Pages format). |
| `origingame.runtime.json` | `{"version":1,"crossOriginIsolated":true}` when `isolation` is not `off`: the platform's per-release isolation declaration (`server/src/releases/isolation.ts`). |
| `manifest.webmanifest`, `sw.js` | `--publish` only: the PWA manifest and the service worker. |

The same inputs give the same bytes wherever the directory is written: no
dates, no absolute paths, files in sorted order
(`packages/engine/tests/web-export.test.ts`).

### `og-build.json`

```json
{
  "format": 1,
  "engine": { "name": "origin-engine", "version": "0.0.1", "pinned": "0.0.1" },
  "build": "publish",
  "game": { "name": "Runner", "id": "runner", "profile": "2d" },
  "entry": "index.html",
  "worker": "game.js",
  "wasm": "origin-engine_bg.wasm",
  "probe": { "protocol": 5, "wire": 1, "build": "publish",
             "members": ["hello", "tick", "state", "pick", "entity", "input", "capture"] },
  "crossOriginIsolation": "preferred",
  "display": { "fit": "fill", "aspect": null, "orientation": "any", "dprCap": 2,
               "fullscreen": "off", "gate": "tap", "madeWith": true },
  "icons": [{ "src": "icons/icon-192.png", "sizes": "192x192", "type": "image/png", "source": "brand" },
            { "src": "icons/icon-512.png", "sizes": "512x512", "type": "image/png", "source": "brand" }],
  "pwa": { "manifest": "manifest.webmanifest", "serviceWorker": "sw.js" },
  "version": "<sha256 over every file's path and sha256>",
  "files": [{ "path": "index.html", "bytes": 12519, "sha256": "…" }],
  "total": { "files": 45, "bytes": 12477811 }
}
```

- `engine.version` is the engine that built it; `engine.pinned` is
  project.ogt's `engine`. `probe.members` is read from the copied wasm's own
  `hello`, so a publish build proves its trim.
- `crossOriginIsolation`: `required` (the game needs SharedArrayBuffer),
  `preferred` (it uses the input, request and audio rings when isolated and
  runs single-threaded otherwise; the default), `off`.
- `files` lists every file beside it (not itself) with bytes and sha256;
  `version` is their identity (the service worker's cache name is its first
  16 hex digits, computed before `sw.js` is added).

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
- `index.html`, `sw.js` and `og-build.json` should not be cached long; the
  rest may be cached forever once a host names files by content.
- Without isolation everything still works: input and probe calls cross
  `postMessage` instead of the SharedArrayBuffer rings; audio uses a
  MessagePort to the worklet.

## project.ogt `publish`

```text
publish {
  probe player            # player | all
  out "dist"
  isolation preferred     # required | preferred | off
  fit fill                # fill | letterbox
  aspect "16:9"           # letterbox: width:height (required with letterbox)
  orientation any         # any | landscape | portrait (manifest; locked in fullscreen)
  dpr_cap 2               # 0.5–4
  fullscreen off          # off | gesture
  gate tap                # tap | none
  made_with true          # the "Made with Origin Engine" badge
  icon "assets/icon.png"  # square PNG ≥ 192 px; absent: the brand tile
  short_name "Runner"     # the installed app's name
}
```

`og build` checks every value first (`crates/og-cli/src/publish.rs`); a bad
one names its line and the values it takes.

## The shell

- **Boot**: the brand symbol, white on ink `#14161b`, plays `genesis` once,
  then `progress` with the real value: bytes of the wasm and of `cooked/`
  received (streamed, counted in the Worker), never backwards. Motion never
  delays the game: a fast load skips straight to it.
- **Play gate**: the first screen is drawn behind it (the shell takes the
  game's first tick, which builds its UI, then holds the clock). The first
  tap, click, Enter or Space unlocks audio (one AudioContext), asks for
  fullscreen when `fullscreen gesture` (and locks `orientation`), takes a
  screen wake lock and starts the clock. `gate none` starts at once; audio
  then unlocks on the first gesture.
- **Layout**: `fill` covers the whole `100dvh` viewport and passes the
  safe-area insets to the UI (`og.ui` viewport); `letterbox` fits `aspect`
  inside the safe area on ink. The canvas is drawn at
  `min(devicePixelRatio, dpr_cap)`; resizes and orientation changes resize
  the render target in the Worker (`og_web_resize`) before the next frame.
- **Visibility**: a hidden page pauses the clock and suspends audio, releases
  the wake lock, and resumes all three.
- **Input**: keyboard (`KeyboardEvent.code`; keys that would scroll the page
  are kept), pointer events of every type with pointer capture (the primary
  pointer drives the engine's pointer; touch ids ride in the ring for the
  IDL change that will take them), wheel, and gamepads polled on the display
  frame as the keys of their standard mapping (A Space, B Escape, X KeyX,
  Y KeyZ, LB KeyQ, RB KeyE, Back Tab, Start Enter, d-pad and left stick
  arrows).
- **Error card** (design §6): a failed start or a crash after it (a trap, a
  failing system, an uncaught Worker error) shows the reason, the last 200
  lines of the engine's log and the Worker's console, and the world dump (a
  dev build) or the state (a publish build), with Reload and Copy report.
- **PWA** (publish builds): the service worker precaches every file the page
  fetches under `og-<version>`, drops other `og-*` caches on activation,
  serves the page from the network first (so a host's additions to it are
  current) and from the cache offline, and every other file from the cache.
  The install hint appears only when the criteria hold: not framed, not
  installed, a manifest and a service worker, and either Chromium's
  `beforeinstallprompt` (an Install button) or iOS Safari (Share, then Add to
  Home Screen); "Not now" is remembered.
- **Marks** for tests and telemetry: `performance` marks `og:ready`,
  `og:first-frame` and `og:start`.
- **URL parameters**: `webgl=1` (the WebGL2 fallback; also taken when WebGPU
  has no adapter), `replay-seed=<u32>`, `autoplay=0` (a harness drives the
  clock through the probe: no gate, no first tick, no service worker).

## The Player and Stage contract

### Player protocol 2 (`origingame/docs/player-protocol.md`)

The shell says `hello` (`substrate: "origin-engine"`) at startup and owns its
readiness: `loading` with the byte fraction, `ready` at the first rendered
frame, `fail` (`engine_start`, retryable) before it, `crash` (`EngineCrash`
and a fingerprint of the message, nothing else) after it, and one `sample`
with `renderer` (the tier) and `timings.firstFrame`. Under a Player that
answered `hello`, the shell draws no gate: it plays on `start` (or a hello
with `started: true`), pauses on `pause`, resumes on `resume`, and resumes
its AudioContext on `gesture`. It posts only to `window.parent` at its own
origin and hears only `window.parent` at its own origin.

### The og-probe wire (`origin/packages/contracts/src/probe-wire.ts`)

The shell serves the wire itself (`shell/wire.ts`) over the engine's probe,
which lives in the Worker and answers asynchronously:

- It answers `{og: "probe", v: 1, kind: "call"}` only from `window.parent`
  (else `window.opener`) at the origin in `window.__OG_STAGE_HOST__`; a page
  told no origin, or another origin, or a call of another `v`, gets nothing.
  Every post names that origin. It posts `hello` on load, when the told
  origin or the probe's availability changes, and after a `hello` call.
- Shapes: `hello` `{protocol: 5, substrate: "origin-engine"}`; `state`,
  `pause`, `resume`, `advance` `{tick, paused, timings: {frameMs}}` (the
  engine's own `paused` and `frameMs`); `entities` `{items: [{id}]}`; `bounds`
  `{items: [{parameter, label?, path?, min?, max?, step?, value?}]}`; `entity`
  `{id, params}`; `patch` `{applied: true, roundTripMs}`; `capture` `{tick,
  width, height, png}` from the engine's own readback (the canvas belongs to
  the Worker), fitted into `width`×`height` without upscaling; `record`
  `{mime, base64, durationMs}` from the canvas's capture stream.
- Errors: `refused` for a probe refusal (`false`/`null`) and parameters out
  of bounds; `unsupported` for an unknown method (including `input`, which
  the wire does not carry), no probe yet, and every member a publish build
  drops (`pause`, `resume`, `advance`, `bounds`, `patch`); `failed` for a
  fault.
- A dev (Stage) build serves every member; a publish build the 7 player
  members. `globalThis.__runtime_probe__` (protocol 5, `substrate:
  "origin-engine"`, every member a Promise) stays for in-page callers.

Proven in headless Chrome 150 by `tests/web/wire.ts` (dev and publish,
isolated and not: 120 checks), `tests/web/player.ts` and
`tests/web/export.ts`.

## Packaging

```sh
cd packages/engine
sh scripts/build.sh --linux-x64   # this machine's addon and og, the Linux x64 addon and og (cargo zigbuild, glibc 2.31), both web hosts
sh scripts/pack.sh                # dist/pack/*.tgz with sizes and sha256
bun scripts/e2b-addon.ts          # optional, billable: install the tarballs in the platform's E2B probe sandbox and og run the runner there
```

- `@origingame/engine`: the TypeScript API and runtime, both web hosts, the
  shell, the Basis transcoder, `bin/og.mjs` (runs the platform package's
  `og` with this package as `OG_ENGINE_PACKAGE`). No native code.
- `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64`:
  `origin-engine.node` and `og`, with `os`/`cpu` (and `libc: glibc`), as
  `optionalDependencies` of the main package, so npm and bun install only the
  matching one. The loader takes `OG_NATIVE_ADDON`, then a checkout's
  `native/`, then the platform package.
- Publishing (not done yet): bump the version in `package.json` (the
  optional dependencies follow it in `pack.sh`'s platform manifests and must
  be bumped with it), build on macOS arm64 with `--linux-x64`, pack, then
  `npm publish` each platform tarball before the main one, with access
  public under the `@origingame` scope.
