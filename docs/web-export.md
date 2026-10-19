# Web export

`og build` turns a project into a directory any static host serves: the
player shell, the engine (wasm in a Worker on an OffscreenCanvas), the game
bundle and its cooked assets. This page is the output layout, what a host
must send, project.ogt's publish settings, the shell's behaviour, the
contract the platform's Player and Stage rely on, the `og` commands
around it, and how the package is packed. Design: `docs/design.md` §2.2,
§4, §6, §9.

```sh
og build .              # dev build: every probe member, no service worker
og build . --publish    # what ships: the player probe members, minified, the PWA
```

## Output layout

| Path | What it is |
|---|---|
| `index.html` | The player shell: the canvas, the boot splash (with the cover's backdrop on a platform page), the play gate, the error card, the install hint, Back, the badge. Carries `<meta name="og-engine" content="<engine version>">`, `og-build` (`publish`/`dev`), `og-profile` and `og-player` (`frame page`: the Player transports the shell speaks, what the platform reads to open the page at the top level), the inlined CSS (brand tokens and only the `genesis`, `progress` and `drift` motion rules) and the page configuration (`<script type="application/json" id="og-config">`). |
| `play.js` | The shell (`packages/engine/shell/page.ts`, bundled). |
| `game.js` | The engine Worker: the runtime and the game. |
| `audio-worklet.js` | The AudioWorklet, loaded on the first gesture. |
| `origin-engine.js`, `origin-engine_bg.wasm` | The web host. `--publish` takes `dist/web-publish` (probe trimmed to hello, tick, state, pick, entity, input, capture; `wasm-opt -Oz`). |
| `basis/` | Binomial's Basis transcoder and its licence. |
| `cooked/` | The project's cooked bundle (`og cook`, run by `og build`), split by streaming group: the first group (everything no scene's `stream` block holds, the `#proxy` stand-ins included) and one pack per streaming group, `cooked/stream/<name>.ogb`. A grouped output's own file is left out, because the engine reads it from its pack. |
| `icons/` | `publish.icon` as `icons/icon.png`, or the brand tile at 192 and 512 px. |
| `og-build.json` | The build manifest (below). |
| `_headers` | For static hosts (Netlify and Cloudflare Pages format): `Cross-Origin-Resource-Policy: cross-origin` and `X-Content-Type-Options: nosniff` on everything, COOP `same-origin` and COEP `require-corp` unless `isolation off`, `Cache-Control: no-cache` on `/`, `index.html`, `sw.js` and `og-build.json`, and the `.wasm` and `manifest.webmanifest` content types. |
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
  "player": { "protocol": 2, "transports": ["frame", "page"] },
  "version": "<sha256 over every file's path and sha256>",
  "files": [{ "path": "index.html", "bytes": 12519, "sha256": "…" }],
  "total": { "files": 45, "bytes": 12477811 },
  "stream": null
}
```

- `engine.version` is the engine that built it; `engine.pinned` is
  project.ogt's `engine`. `probe.members` is read from the copied wasm's own
  `hello`, so a publish build proves its trim.
- `crossOriginIsolation`: `required` (the game needs SharedArrayBuffer; a
  page served without isolation still starts, single-threaded, with a
  warning in the log),
  `preferred` (it uses the input, request and audio rings when isolated and
  runs single-threaded otherwise; the default), `off`.
- `files` lists every file beside it (not itself) with bytes and sha256;
  `version` is the sha256 over the path and sha256 of each of them except
  `sw.js`, so it is the same before and after `sw.js` is written; the
  service worker's cache name is `og-` and its first 16 hex digits.
- `player` is the Player protocol the shell speaks and its transports, in
  dev and publish builds alike: `frame` (protocol 2 over `postMessage` to a
  framing Player) and `page` (protocol 2 over same-origin HTTP when the
  platform opens the page itself at the top level; §The top-level page).
  `index.html`'s `<meta name="og-player">` says the same.
- `probe.build` is what the copied wasm's `hello` reports: `publish` for a
  `--publish` build, `release` for a dev build (the web host is a release
  wasm either way; the top-level `build` is what says `dev`).
- `stream` is `null` for a bundle without streaming groups. Otherwise it is
  `{ "first": { "files", "bytes" }, "groups": [{ "name", "file", "bytes",
  "sha256", "priority", "memory", "assets" }] }`. `first` is what the boot
  fetches and the service worker precaches (the cooked manifest included).
  Each group is its pack under `cooked/stream/`, with the memory its assets
  take once loaded (what `og.stream`'s budget counts) and its asset ids.
  The packs are in `files` too.

### Streaming groups

The Worker fetches the first group before the engine starts: the manifest,
every output no group holds, and their Basis textures transcoded. A
streaming group's pack is fetched only when the runtime admits the group
(its trigger wants it and it fits the budget, IDL module `stream`). Before
the pack is handed to the engine, the Worker transcodes its Basis textures
to the encodings the renderer takes, as for the first group
(`og_web_pack_ktx2`), so the whole group goes in place at once. Until then
its models draw their proxies, its textures are 1x1 placeholders and its
sounds play as silence of their own length. The boot's progress (`expect.cooked`) counts
only the first group. `packages/engine/tests/asset-stream.test.ts` checks
the split, and `packages/engine/tests/web/stream.ts` plays it in Chrome: no
pack at boot, each pack fetched once when its group is wanted, the
textures transcoded, and the visited groups loading offline.

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
  probe player            # player | all (checked; the probe set follows --publish, see below)
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
one, an unknown key, or `fit letterbox` without `aspect` names its line and
the values it takes. Every key is optional; the defaults are the values
shown, with no aspect, icon or short name. `probe` is read and checked but
does not choose anything today: a `--publish` build always serves the
player members and a dev build every member, whatever it says.

## The shell

- **Boot**: the brand symbol, white on ink `#14161b`, plays `genesis` once,
  then `progress` with the real value: bytes of the wasm and of `cooked/`
  received (streamed, counted in the Worker), never backwards. Motion never
  delays the game: progress shows after genesis or after 1.2 s, whichever
  comes first, a fast load skips straight to the game, and
  `prefers-reduced-motion` skips genesis.
- **Play gate**: the first screen is drawn behind it (the shell takes the
  game's first tick, which builds its UI, then holds the clock). The first
  tap, click, Enter or Space unlocks audio (one AudioContext), asks for
  fullscreen when `fullscreen gesture` and the page is not framed (and locks
  `orientation`), takes a screen wake lock and starts the clock. `gate none`
  starts at once; audio then unlocks on the first gesture.
- **Layout**: `fill` covers the whole `100dvh` viewport and passes the
  safe-area insets to the UI (`og.ui` viewport); `letterbox` fits `aspect`
  inside the safe area on ink. The canvas is drawn at
  `min(devicePixelRatio, dpr_cap)`; resizes and orientation changes resize
  the render target in the Worker (`og_web_resize`) before the next frame.
- **Visibility**: a hidden page (and `pagehide`) pauses the clock and
  suspends audio, releases the wake lock, and resumes all three.
- **Input** (`src/web/shell.ts`, `src/web/gamepad.ts`): keyboard
  (`KeyboardEvent.code`; keys that would scroll the page or move focus are
  kept from it; nothing is taken while an input field has focus); pointer
  events of every type (mouse, pen, touch) with pointer capture, up to five
  held at once: each pressed pointer takes the lowest free engine id, 0
  (the primary pointer `og.Input` follows) while none is held, then 1 to 4,
  and `og.Pointers` reports each with its position and press point, so a
  stick and a button can be held together; a hovering mouse moves pointer
  0; wheel. Gamepads are polled on the display frame while one is
  connected: the first four standard-mapping pads reach `og.Gamepad` whole
  (buttons, both sticks, both triggers) whenever they change, and, for
  games that only read keys, every pad also presses the keys of its
  standard mapping (A Space, B Escape, X KeyX, Y KeyZ, LB KeyQ, RB KeyE,
  Back Tab, Start Enter, d-pad and the left stick past 0.5 the arrows);
  og-ui counts a pad intent and the key it maps to once.
- **Error card** (design §6): a failed start or a crash after it (a trap, a
  failing system, an uncaught Worker error) shows the reason, the last 200
  lines of the engine's log, the Worker's console and the page's errors,
  and the world dump (a dev build) or the state (a publish build), with
  Reload and Copy report; the brand symbol switches to `drift`.
- **PWA** (publish builds, in a secure context): the service worker
  precaches `./` and every file of the build except `og-build.json`,
  `_headers`, `origingame.runtime.json`, licence and README files, the
  streaming groups' packs and itself under `og-<version>` (a failed install
  keeps the old build), drops the scope's other `og-*` caches on activation,
  serves the page from the network first (so a host's additions to it are
  current) and from the cache offline, and every other file from the cache.
  The network page is the browser's own navigation request (navigation
  preload, enabled when the worker activates): a worker's own
  `fetch(event.request)` of a navigation leaves as `Sec-Fetch-Dest: empty`,
  and a host that answers documents differently (the platform gives its
  derived entry, with `window.__OG_GAME`, only to a document or frame load)
  would hand a revisit its stored bytes; the worker fetches the page itself
  only where the browser has no navigation preload.
  A pack is not precached. It comes from the cache, or from the network into
  the same cache the first time the engine fetches it, so a place the player
  has already visited also plays offline.
  The install hint appears only when the criteria hold: not framed, not
  installed, a manifest and a service worker, and either Chromium's
  `beforeinstallprompt` (an Install button) or iOS Safari (Share, then Add to
  Home Screen); "Not now" is remembered.
- **Language**: the shell's own words (loading, the gate, the error card,
  the install hint) are Chinese when `navigator.language` is `zh*`, else
  English.
- **Marks** for tests and telemetry: `performance` marks `og:ready`,
  `og:first-frame` and `og:start`.
- **URL parameters**: `webgl=1` (the WebGL2 fallback; also taken when WebGPU
  has no adapter), `replay-seed=<u32>`, `autoplay=0` (a harness drives the
  clock through the probe: no gate, no first tick, no service worker).

## The Player and Stage contract

### Player protocol 2 (`origingame/docs/player-protocol.md`)

The shell says `hello` (`substrate: "origin-engine"`) at startup and owns its
readiness: `loading` with the byte fraction, `ready` at the first rendered
frame (or two seconds after the boot when nothing renders), `fail`
(`engine_start`, retryable) before it, `crash` (`EngineCrash` and a
fingerprint of the message, nothing else) after it, and one `sample` with
`renderer` (the tier) and `timings.firstFrame` and `timings.boot`. A framed
page (and the Stage overlay's own document, below) waits 300 ms for a
Player's `hello` before drawing its own gate; under
a Player that answered, the shell draws no gate: it plays on `start` (or a
hello with `started: true`), pauses on `pause`, resumes on `resume`, and
resumes its AudioContext on `gesture`. It posts only to `window.parent` at
its own origin and hears only `window.parent` at its own origin with
`og: "player", protocol: 2`.

Who the shell speaks protocol 2 to, in order:

1. framed: `window.parent` (above);
2. top level with the platform's `window.__OG_GAME`: its play origin over
   HTTP (§The top-level page);
3. top level, no `__OG_GAME`, and told its own origin as the Stage host
   (`window.__OG_STAGE_HOST__ === location.origin`, decided exactly as the
   og-probe wire decides it, `shell/wire.ts` `stageInPage`): **its own
   window**. That is a Stage build opened as its own document with the
   platform's Stage overlay injected into it, and the overlay is its
   Player. The shell posts to `window` at its own origin and hears only
   `window` at its own origin; since every post in one window reaches
   every listener, a `hello` that names a `substrate` or lists no
   `capabilities` (the shell's own) is not taken for the Player's. The
   hello repeats every 250 ms until the overlay answers (at most six), in
   case the overlay's listener comes up after the shell. The overlay's
   answers are the Player's: its `hello` with `started: true` starts play
   (no gate of the shell's), its `pause`, `resume` and `gesture` count, and
   og.platform's calls go to it as below. On the Stage nothing of
   og.platform's goes to the browser's storage: where the overlay answers
   nothing (no hello within 1.5 s, or a method it does not list), the
   page's memory does, for that load. No service worker is registered
   there (the shared Stage origin installs none);
4. otherwise no one: the page sends and hears nothing and plays alone on
   the browser's storage.

### Platform services over protocol 2 (E6)

`og.platform` (IDL module `platform`, docs/design.md §2.8) reaches the
Player through the same channel. The engine Worker hands each tick's calls
to the page (`{ type: "platform", calls }`); the page answers them
(`platform-answer`) and the answers are applied at the start of the next
tick. A framed page (or one with the Stage overlay in its own document)
holds its first calls (the tick behind the gate) until the Player's
`hello`, or 1.5 s, so they never reach the wrong store. The
messages the shell now sends and hears:

| Shell → Player | When | Answered by |
|---|---|---|
| `call` `store.get` `{ key }` | `storeGet`, `loadSettings` | `result` `{ key, value, revision }` (`value` null, `revision` 0 for a key never written) |
| `call` `store.set` `{ key, value, revision? }` | `storeSet`, `saveSettings`: og.platform's revision N sends `revision: N`, `create` sends `revision: 0` (only a key never written), revision 0 sends none (the last write wins) | `result` `{ key, revision }`, or `conflict` with `error.revision`, the key's revision now |
| `call` `store.list` | `storeList` | `result` `{ keys: [{ key, revision }] }` |
| `call` `leaderboard.submit` `{ score, board, order }` | `leaderboardSubmit` | `result` `{ ok, rank, board, best }` (or `{ rank: null, queued: true }` offline) |
| `call` `leaderboard.top` `{ n, board }` | `leaderboardTop` | `result` `{ board, sort, top, me }` |
| `call` `leaderboard.around` `{ n, board }` | `leaderboardAround` | `result` `{ board, sort, entries: [{ rank, name, score, me }], me }` |
| `call` `point.award` `{ reason, amount }` | `point` | `result` `{ granted, balance }` (today `{ granted: 0, balance: null }`: no release carries an approved point policy, origingame `docs/public-api.md` §15) |
| `call` `ai.mark` `{ kind, label, model? }` | `aiMark` | `result` `{ marked: true }` |
| `event` `game.custom.<name>` `{ attrs }` | `telemetry` (one-way; `<name>` 1–64 of letters, digits, `.` and `_`, at most 16 scalar attributes, checked by og.platform) | — |

The answers reach og.platform renamed only where its shape differs: a
leaderboard's `sort` is og.platform's `order`, and `top` its `entries`.
Each `call` carries the shell's own `id` (`oe-<n>`), and its `result` is
matched by it; a call unanswered in 15 s fails with `timeout`. What the
shell asks follows the Player's `hello.capabilities`: a Player that lists
`save.read`/`save.get` and `save.set` but no `store.*` (released before
2026-10-19) keeps the store in the save's `store` object, read with
`save.read` and written back with `save.set { data, revision }` at the
revision it was read at; one with `leaderboard.top` but no
`leaderboard.around` answers `around` from the top 100; a method the hello
does not list answers `unavailable`; and where the Player keeps no store
(none listed, or `unavailable`) the browser's `localStorage`
(`og.platform:<path>`) does (a verification run lists `store.*` from a
throwaway store of its own). Unframed, every call is answered from there
(the native host's local platform, `src/platform-local.ts`). Errors keep
the Player's codes, `retryable` and a conflict's `revision`. The Stage
overlay in the page's own document is answered the same way (from its
hello's capabilities), with the page's memory instead of `localStorage`
behind it.
`tests/platform-player.test.ts` runs the bridge against a stand-in Player
that answers the protocol's calls (and an older one, and none);
`tests/web/player.ts` checks the `store.get` round trip in Chrome (the
runner's stored best becomes its best). `tests/platform-stage.test.ts`
runs the own-window path against a stand-in overlay answering in the same
window (one hello, never the shell's echo; the store and its `time`;
storage untouched; foreign windows and origins ignored; the precedence),
and `tests/web/player.ts` opens the runner's publish build as a Stage
build with an in-document stand-in overlay in Chrome, at desktop and
phone sizes.

The host's clock (`og.platform.now`, every answer's `time`, Phase 2 core
round) is the Worker's `Date.now()` when the platform attaches and when an
answer arrives, between ticks, and `og.platform.info(world).clock` reads
`browser`: protocol 2's `hello` and `result` carry no time today
(`docs/platform-asks.md` #9). A `result` that carries a numeric `time`
(UTC ms) has it passed on by the bridge, from the framed Player, the HTTP
transport and the Stage overlay alike, as that answer's `time` (a store
kept in the save takes the time of the call that settled it: `save.set`'s,
else `save.read`'s), and the Worker takes it as the Player's (`clock` then
reads `player`).

### The top-level page (Player protocol 2 over HTTP)

The platform opens a published page whose build says `og-player` … `page`
as the top-level document on the game's own origin (origingame
`docs/player-protocol.md` §The top-level engine page): no Player page
frames it. The shell (`shell/transport.ts`) then speaks protocol 2 to its
play origin itself, when the page is not framed and carries the
platform's `window.__OG_GAME` (`{ id, releaseId, protocol: 2 }`, added to
the entry at publication). A framed page keeps the `postMessage` path
above, and a top-level page without `__OG_GAME` (any other static host)
plays alone on the browser's storage, as before.

- **Requests.** `POST /__player/v2/<id>/<releaseId>/hello`, `call` and
  `report`, same origin, `credentials: "omit"`, one at a time through one
  queue (so the page mints at most one guest), each with
  `Authorization: Bearer` once the page holds a credential. A `call` is the
  protocol-2 `call` message and its answer the `result`; 15 s without an
  answer is `timeout`, a failed fetch `offline` (both retryable), any other
  status the protocol's code for it (`404` `not_found`, `429`
  `rate_limited`, `5xx` `unavailable`). og.platform's mapping is the
  framed one, from the HTTP hello's `capabilities`.
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
  per load) and `device` (`device_<32 hex>` from `og_device_<id>`, 30
  days, the Player's). A report that fails is kept (at most 200 messages)
  and sent with the next.
- **The brand surface** is the shell's at the top level (design §4, §9):
  the boot splash on ink with the cover's 32 px derivative upscaled and
  blurred behind it (the hello's `page.backdrop`, `crossorigin="anonymous"`
  on an isolated page), the play gate (one gesture: audio unlock,
  fullscreen when `fullscreen gesture`, the orientation lock, wake lock),
  `100dvh` and the safe-area insets, the PWA install hint, the badge, and
  **Back** (top left inside the safe area; full on the boot splash, the gate
  and the error card, a small handle while playing): `history.back()` when
  the referrer is on the hello's `page.portal` and there is history, else
  `page.back`, the game's page on the site. There is no purchase hand-off:
  protocol 2 has no commerce store.

`tests/platform-page.test.ts` runs the transport against a stand-in play
origin (the code stripped and sent only in the body; an account kept out of
storage and a stored one never resumed without a code; one guest for two
concurrent first calls; a rejected credential replaced; a conflict's
revision; offline scores queued then delivered in order; a timeout;
nothing reported before start, UUID v7 ids, one session and device, the
end with keepalive; og.platform's store, telemetry and scores through it).
`tests/web/player.ts` opens the runner's publish build at the top level in
Chrome against a stand-in play origin, at desktop and phone sizes.

### The og-probe wire (`origin/packages/contracts/src/probe-wire.ts`)

The shell serves the wire itself (`shell/wire.ts`) over the engine's probe,
which lives in the Worker and answers asynchronously:

- It answers `{og: "probe", v: 1, kind: "call"}` only from `window.parent`
  (else `window.opener`) at the origin in `window.__OG_STAGE_HOST__`; a page
  told no origin, or another origin, or a call of another `v`, gets nothing.
  A top-level page told its own origin also answers its own window, replying
  to it (and speaks Player protocol 2 to it, §Player protocol 2): that is the
  platform's Stage overlay drawn in the engine's document
  (a Stage build with `og-player` … `page` is opened as its own top-level
  document with the overlay injected), which posts its calls to `window`.
  Every post names that origin. It posts `hello` on load, when the told
  origin or the probe's availability changes, and after a `hello` call.
- Shapes: `hello` `{protocol: 5, substrate: "origin-engine"}` (both `null`
  before the engine is ready); `state`, `pause`, `resume`, `advance`
  `{tick, paused, timings: {frameMs}}` (the engine's own `paused` and
  `frameMs`); `entities` `{items: [{id, label?}]}`; `bounds` `{items:
  [{parameter, label?, path?, min?, max?, step?, value?}]}`; `entity` `{id,
  params}`; `patch` `{applied: true, roundTripMs}`; `capture` `{tick, width,
  height, png}` from the engine's own readback (the canvas belongs to the
  Worker), fitted into `width`×`height` without upscaling; `record` `{mime,
  base64, durationMs}` from the canvas's capture stream.
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
- `listen` (E6, dev builds): the last seconds (at most 2) of what the
  engine Worker's mixer rendered, `{tick, rate, frames, peak, rms, wav}`
  with a base64 16-bit stereo WAV; the command channel's
  `listen[:path][:seconds]` writes the same as a `wav` output; before the
  page's audio is attached it answers `null`.

Proven in headless Chrome 150 by `bun run test:browser` in
`packages/engine`: `tests/web/wire.ts` (dev and publish, isolated and not:
120 checks), `tests/web/player.ts` (the framed Player, then the top-level page at
desktop and phone sizes) and `tests/web/export.ts` (40;
transcripts in `tests/evidence/E1/web-*`). These need a Chrome and are not
in the gate; `tests/web-export.test.ts` (layout, manifest, determinism,
icons) is.

## The `og` commands around the export

- `og build [project] [--publish] [--out DIR] [--json]` cooks the project,
  checks project.ogt (above) and writes the directory into `--out`, else
  `publish.out`, else `<project>/dist`; it prints each file's bytes and
  brotli `-q 11` size (gzip `-9` without the brotli CLI), the probe build
  and members read from the copied wasm, and the manifest's version;
  `--json` prints one JSON line instead. The web host comes from
  `packages/engine/dist/web` or `dist/web-publish`; a checkout without them
  is told to run `bun run build` in `packages/engine`.
- `og run` (the native host under bun) supervises the game process: when
  it dies with a failure, `og run` says at once how (its exit status, or
  the signal), whether its probe had been serving (`--port`: it stops
  answering), and its last stderr lines, and exits with the process's code
  (128 + the signal).
- `og run --device chrome|android[:serial]|iphone[:udid] [project]
  [--seconds N] [--port P] [--out DIR] [--size WxH] [--headed] [--webgl]
  [--chrome <path>] [--cert <pem> --key <pem>] [--json]` exports the
  publish build into `<project>/.og/device`, serves it on 0.0.0.0 with its
  `_headers` and COOP/COEP/CORP (no-store, so every load is cold), prints
  the LAN URL, opens it in the target, passes the play gate with a real
  tap and times the page. `chrome` is this machine's Chrome over the
  DevTools protocol (`--chrome`, `CHROME`, `CHROME_PATH`, the agent-browser
  install, /Applications, PATH; headless unless `--headed`); `android` is
  Chrome on a phone over adb (`adb reverse` puts the page at the phone's own
  localhost, a secure context, and `adb forward` reaches its DevTools);
  `iphone` (or `ios`) is Safari over safaridriver at the LAN URL, which is
  a secure context only over HTTPS (`--cert`/`--key`, from mkcert). It
  reads `og:first-frame` (navigation to the first frame behind the gate),
  `__runtime_probe__.hello()`'s tier and, after a second of play,
  `__og_timings__` over `--seconds` (default 5): the Worker's display
  interval and engine work per played frame as count, mean, p50, p95, p99
  and max. `report.md` and `device.json` (default `<project>/og-out/device`)
  have og polish's shape: a first-frame score against the §7 budget (2.0 s,
  1.5 s on the iPhone), a frame-time score (p95 interval, 17 ms: 100,
  33.3 ms: 0), findings with a fix each (over budget, missed frames, work
  over 16.7 ms, no isolation, console errors), the device (user agent,
  platform, tier, WebGPU adapter, WebGL renderer, DPR, viewport, canvas,
  cores, memory, isolation) and the headers served. A missing Chrome, adb
  or safaridriver is told in one line before the build; the phone paths'
  prerequisites are in tests/evidence/P3/device/NOTES.md.
- `og doctor` (no arguments) checks what the author's loop needs: bun
  (node is optional when bun is present), the engine package, its native
  addon, both web host wasm files, and a 64×64 frame rendered headless
  (Metal, Vulkan or lavapipe: the executor path that needs no sandbox).
  Inside the engine repository it also compares rustc, cargo and
  wasm-bindgen with `rust-toolchain.toml` and `Cargo.lock`; outside, those
  lines say `skipped`. Each problem prints one line naming its fix, and the
  exit code is non-zero when there is any.

## Packaging

```sh
cd packages/engine
sh scripts/pack.sh --linux-x64    # OG_RELEASE=1 scripts/build.sh --linux-x64 (this machine's addon and og, the Linux x64 addon and og through cargo zigbuild against glibc 2.31, both web hosts), then dist/pack/*.tgz with sizes and sha256
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
  be bumped with it), `sh scripts/pack.sh --linux-x64` on macOS arm64 (a
  release build: `pack.sh` refuses a dev `native/`), then
  `npm publish` each platform tarball before the main one, with access
  public under the `@origingame` scope.
