# Platform asks

What the engine's `og.platform` (IDL module `platform`, docs/design.md §2.8)
and its tools need from the OriginGame platform and origin-agent that they
do not give today. The engine works without each one: the shell's bridge
(`packages/engine/shell/platform.ts`) maps what it can onto Player protocol 2
and answers the rest `unavailable`, falling back to the browser's storage for
the store. Each ask names the shape the engine would use, so the bridge's
mapping shrinks to a rename when it lands. Asks are numbered once and never
renumbered; platform asks are collected at release.

Only open asks are listed; landed ones (1–8, 12, 14, 15) are in git
history. Numbers are never reused.

## The Player's clock

| # | Ask | Why | Until then (the engine's stand-in) |
|---|---|---|---|
| 9 | The Player's `hello` carries `time` (UTC ms, the Player's clock), and every answer carries `time` when the Player answered it | `og.platform.now` and `PlatformResult.time` are the host's clock handed to the game as host input, recorded in replays like a key press; an idle game credits the time away from a store answer's `time`. The Player's clock (one the platform can correct, the same for every device of a player) should be the one games read, not each browser's. | The engine's Worker (`packages/engine/src/web/worker.ts`) gives og.platform the browser's clock (`Date.now()` when the platform attaches and when each answer arrives) and says so: `og.platform.info(world).clock` is `browser`. An answer the page hands back with its own `time` is taken as is and `clock` reads `player`, so when this lands the shell's bridge (`packages/engine/shell/platform.ts`) only passes the field on. |

State: open. The platform has implemented `time` on the protocol-2 hello
and every result (framed Player, Stage and verify mode stamp at answer
time; HTTP stamps its server's time; the framed Player corrects its clock
from same-origin `GET /__og/time` and never moves it backwards), not yet
deployed. The engine reads hello time and result time through both
transports, the shell and the Worker to `attachPlatform`; an unstamped
hello keeps the host-clock fallback. This is client consistency, not
authoritative time.

Nothing here carries a credential: every method is a name and arguments,
and the Player owns the session, as protocol 2 already requires.

## The top-level page

| # | Ask | Why | Before | State |
|---|---|---|---|---|
| 28 | (formerly the second 9) Player protocol 2 over same-origin HTTP for a top-level engine page: `POST /__player/v2/<game>/<release>/{hello,call,report}` on the game's own origin, the hand-off code redeemed from the hello's body, the framed Player's answers and errors, the session reports filed as the Player filed them; and Play (the game host's `/`, `/g/:slug`) opening a release whose entry says `<meta name="og-player" content="frame page">` as its own top-level document (307 to the release entry), the Stage page likewise for a Stage build (the creator overlay injected into the engine's document) | design §4: the shell is the play page and the brand surface; a frame costs the viewport, gestures, audio unlocking and the PWA on phones | every engine page is framed by the Player page or the Stage page | Open in production. The contract is origingame `docs/player-protocol.md` §The top-level engine page; the shell speaks it (`shell/transport.ts`, `docs/web-export.md` §The top-level page). The platform also needs the game-host ingress to admit `POST` with `Authorization` (never `Cookie`) on `/__player/v2/` (those hosts take only GET and HEAD and strip both), a production topology change the platform owns (`deploy/ovh/ingress-boundaries.conf`); production stays framed until that location is applied on the game hosts and the E2B templates are rebuilt, in that order. |

Nothing in this ask carries a credential to game code beyond what a game
on its own origin can already read: the game-scoped player credential the
page holds (design §4: the page has nothing beyond the game's own scope to
steal; the host ABI is the only route to platform APIs).

## The Stage overlay in the engine's document

| # | Ask | Why | Before | State |
|---|---|---|---|---|
| 10 | In in-document mode (`inPage`, a Stage build with `og-player` … `page`), the Stage overlay answers Player protocol 2 on the page's own window: its stand-in Player (`createGameListener` in `sdk/src/stage-page.ts`) takes `window` as the game window (today `gameWindow` is `frame?.contentWindow ?? null`, which is null in-page, so it answers nothing) and posts its answers to `window` at `location.origin`; it skips the Player messages it posted itself (a `hello` that lists `capabilities`, `result`, `start`, `gesture`, `pause`, `resume`): in one window it hears its own `hello`, which `readPlayerWireGameMessage` accepts, and would answer it forever; it answers as it does framed (`hello` with `started: true` and the scratch store's methods, `gesture` on the first trusted input, the scratch store's `result`s, Stage pause and resume as `pause`/`resume`), and may stamp each `result` with `time` (ask 9). The scratch store lives in the overlay's memory and an in-page Reload is `location.reload()`, so a Reload starts it empty: keep that (and say so on the Stage) or keep it per ticket in `sessionStorage`. | The overlay is the Player of a Stage build that is its own document: play starts from it, the creator's saves and store calls are its, and the shell keeps nothing of og.platform's on the shared Stage origin. | Without it a top-level Stage build has no Player. | The shell speaks it (`shell/player.ts`, `docs/web-export.md` §Player protocol 2): it ignores its own echo (a `hello` with `substrate` or without `capabilities`), repeats its hello every 250 ms until the overlay answers, holds og.platform until then (1.5 s) and keeps it in memory, never in `localStorage`, and registers no service worker there. Until the overlay answers, a top-level Stage build plays on the page's memory and draws its own gate after 300 ms. The platform's Stage runs in-document against this shell; production changes only with the ingress location and the E2B template rebuild. |

## Lanes: threaded engine pages

| # | Ask | Why | Before | State |
|---|---|---|---|---|
| 11 | Keep serving a release that ships `origingame.runtime.json` `{"version":1,"crossOriginIsolated":true}` (every `og build --publish` whose `isolation` is not `off`, the default `preferred` included) with COOP `same-origin` and COEP `require-corp` on its entry document and every file under it, CORP `cross-origin` on each file, and `allow="… cross-origin-isolated"` on the Player's iframe while a release is framed; at the top level (ask 9) the entry document's own COOP/COEP is what isolates it, so the 307 to the release entry must land on a response carrying them. Send no `worker-src` or `script-src` that excludes `blob:` on the play origin: wasm-bindgen-rayon starts the lanes pool's Workers from a `blob:` URL. | The lanes variant (`origin-engine-lanes_bg.wasm`, docs/web-export.md §Lanes) needs SharedArrayBuffer, so it boots only when the page is `crossOriginIsolated`; otherwise the shell plays the single-threaded module, correct but without lanes. Lanes split crowds of 128 animated characters or more, CPU culls of 2,048 objects or more and the physics step itself: rapier's parallel build runs on the lanes wasm's Worker pool through a relay from 500 bodies on two to four lanes (1,000 on five or six, 1,500 from seven), with ropes, cloths and fluids split too (crates/og-lanes/README.md); without isolation all of it runs on the engine Worker's one thread, with the same results. | Nothing to change, as read from this repository's side (platform `server/src/play/serve.ts` sends `isolationHeaders` for a declared release and a CSP of `frame-ancestors` only, `server/src/play/player.ts` adds `cross-origin-isolated` to the frame's `allow`). | Open: not verified against production. Publish one lanes build and check `__runtime_probe__.lanes.on` on Play, framed and at the top level, on desktop Chrome and a phone; a page downloads one module, a release carries both. |

## The type checker in the sandbox

| # | Ask | Why | Before | State |
|---|---|---|---|---|
| 13 | The `engine` class sandbox image keeps `@origingame/engine`'s `optionalDependencies` when it pre-installs the engine (no `--omit=optional`, no `--no-optional`), so `node_modules/@tsrs/<platform>/tsrs` and its `lib.*.d.ts` are in the image; count it in the image budget (≤ 600 MB): 32.5 MB for `@tsrs/linux-x64-gnu` unpacked, but bun 1.4.2 also installs `@tsrs/linux-x64-musl` (23.3 MB) on glibc because it ignores the package's `libc` field, so 55.9 MB unless the image prunes the musl package | `og check`, and the check `og run`, `og test`, `og build` and `og dev` make before bundling, must not reach the registry inside a sandbox; tsrs is found beside the engine package first (`crates/og-cli/src/check.rs`) | without the package `og check` falls back to tsgo through `bun x --package typescript@7.0.2 tsc` (the network or bun's cache), and `og doctor` reports tsrs missing with the fix | open |

Not an ask, for the platform's own use: `@tsrs/wasm` (the same checker as
a wasm module) returns diagnostics as JSON (`diagnostics: "json"`), the
piece an in-browser check (the Player page, an online editor) would use.
The engine does not consume it.

## Rooms

| # | Ask | Why | Before | State |
|---|---|---|---|---|
| 16 | Rooms in Player protocol 2, listed in `hello.capabilities` as `room.create`, `room.join`, `room.leave`, `room.send`. Calls (each `result` only acknowledges; a refusal is `{ ok: false, error: { code, message } }` with `code` one of `unavailable`, `not_found`, `full`, `invalid`, `offline`): `room.create { name?, capacity? }` (capacity 2 to 16), `room.join { room, name? }` (`room` 1–64 of letters, digits, `.`, `_`, `-`), `room.leave {}`, `room.send { to?, channel?, data? \| bytes? }` (`to` a peer number, else every other peer; `bytes` base64; 64 KiB a game message, 16 MiB for the engine's own `og.`-channel messages, a world snapshot for a late joiner). Events, pushed by the Player as `{ og: "player", protocol: 2, kind: "room", event }` in the order they happened, with `event` one of: `{ type: "joined", room, peer, peers: [{ peer, name }] }` (this peer is in; `peers` includes it), `{ type: "peer_joined", peer, name }`, `{ type: "peer_left", peer }` (after every message that peer sent), `{ type: "message", from, channel?, data? \| bytes? }`, `{ type: "left", reason? }` (roomLeave done, or the room was lost), `{ type: "error", op, code, message }`. Peer numbers are given in join order from 0 and never reused; messages from one sender arrive in the order sent. The platform relays (no authority, no simulation): lockstep runs in every engine. A Play link to a room (`?room=<id>`) may put the page in it at load, answered by the same `joined`. | og.platform.roomCreate/roomJoin/roomLeave/roomSend/roomPeers and og.net's lockstep (docs/design.md §2.8, §2.10): two players' engines in one room exchange inputs and world hashes; natively `og relay` carries them. | — | Open. The shell speaks it (`shell/platform.ts` `playerRooms`, `src/web/shell.ts` `RoomHost`, the Worker's `room`/`room-events` messages): with a Player whose hello lists `room.join` roomInfo's host is `player`; otherwise `none` and roomCreate/roomJoin are refused. The play page reads `?room=<id>` and the engine joins that room itself once the Player's hello lists `room.join` (the engine's side of the Play link is done; the platform's is to put `?room=` on the link). The shapes are og-net's protocol (`crates/og-net/src/protocol.rs`), the same the native relay speaks. The platform side is implemented (Rooms as its own service, `rooms-service/src/player-relay.ts`; the framed Player's client `sdk/src/player-rooms.ts`); open: deployment and a two-Player acceptance through a `?room=` Play link. |

## Source maps and field telemetry

| # | Ask | Why | Before | State |
|---|---|---|---|---|
| 17 | Keep field telemetry's fields when filing `crash` and `sample` (framed Player and the play origin's `/__player/v2/<game>/<release>/report`), shapes in `docs/web-export.md` §Telemetry; never answer 400 for them (a 400 drops the whole report). At publication, take a publish build's `sourcemaps/` out of the directory before the release is served and store it privately under og-build.json's `sourcemaps.build`; map a stored crash's `stack` with `mapStack` from `@origingame/engine/sourcemap` (a public contract, `docs/web-export.md` §Source maps) when showing it. Aggregate `frames` per release (the bins add up), `stages` and `failures`. The `end` sample is posted on `pagehide`, so by the time the Player's listener runs its `MessageEvent.source` no longer equals the iframe's `contentWindow` (seen in Chrome 150): a Player that filters messages on `e.source === iframe.contentWindow` drops every `end`; accept it by origin and the report's game and release instead. | A crash report that carries its mapped stack, tick, engine, build, browser, device, GPU and memory is one an agent can fix; frame-time and hitch distributions per session say which releases play badly on which devices. | A `crash` is `{ name, fingerprint }`; a `sample` is `renderer` and `timings`; stacks name minified bundle lines. | Open. The engine sends them (`publish { telemetry off }` opts a game out); a play origin that drops unknown fields loses nothing it had. |

## Content-hashed names in publish builds

| # | Ask | Why | Before | State |
|---|---|---|---|---|
| 18 | Find an Origin Engine publish build's files through its `og-build.json`, never by a plain file name: `worker` (the engine Worker), `wasm` and `lanes.glue`/`lanes.wasm` (the engine modules), and `names` (each file's own name to the name it is served by, `"play.js": "play.3f2a9c1e04b7.js"`; empty for a dev build) are the served names (`docs/web-export.md` §Content-hashed names). The places that name them, as read at origin-agent and origingame `main`: origin-agent `scripts/workspace-stack/scenarios/deploy.ts:149` and `stage-engine.ts:158,167` check that the entry page contains `./play.js` (check for `./${manifest.names["play.js"] ?? "play.js"}`, or `/\.\/play(\.[0-9a-f]{12})?\.js/`); `deploy.ts:150` and `stage-engine.ts:369` fetch or read `origin-engine_bg.wasm` (read `manifest.wasm`); origingame's wizard family markers (`packages/game-knowledge/production/wizard/rules.json`, `docs/wizard.md`: `origin-engine_bg.wasm`, `origin-engine.js`) detect a build by those names (detect it by `og-build.json` with `engine.name` `origin-engine`, or by `project.ogt` as now), and `tests/wizard-e2e/publish.e2e.ts:110` fetches `/origin-engine_bg.wasm` (fetch `manifest.wasm`). Keep publish builds served as a whole directory, as now: the shared engine blob (`server/src/play/serve.ts`, by sha256 and basename) works unchanged, and with content names two games on one engine still share one URL. | A new build downloads only what changed: hashed files are cached for a year (`_headers`), and the service worker carries every unchanged file over from the old build's cache. | With plain names the browser revalidated each one and the service worker downloads the whole game again on every release. | Open. Until it lands, `og build --publish` keeps the platform's checks passing. `index.html` still loads `./play.js`, which is now a one-line loader (`import "./play.<hash>.js"`, revalidated like `index.html`). The build also writes `origin-engine.js` and `origin-engine_bg.wasm` under their plain names, as copies that no page fetches and the service worker does not precache (`ALIASED_FILES` in `packages/engine/src/node/web-build.ts`). These names go only after this ask has landed, as a later breaking minor (`changelog.json`). |

## Purchases, ads, achievements, identity and share

The engine owns the contract and the game-side API (IDL `platform`,
`og api show platform`); accounts, money, ad networks and the catalogue
stay the platform's. Each call is og.platform's request and answer
(docs/design.md §2.8): the engine queues `{ request, method, args, tick }`,
the host answers `{ ok, data }` or `{ ok: false, error: { code, message,
retryable } }`, and the answer is applied at the next tick's start as host
input, kept in the engine's log and in `og replay`'s trace. No credential
crosses: the Player attaches the player's game-scoped session, and every id
it hands back (a player, a friend, a purchase, an invite) is opaque, at most
128 of letters, digits, `.`, `_`, `-` and `:`, and scoped to this game.

| # | Ask | Why | Until then (the engine's stand-in) |
|---|---|---|---|
| 19 | Player protocol 2 methods, each listed in `hello.capabilities` when the Player answers it, with these arguments and answers (the shell passes them through by name, `shell/platform.ts`). **Purchases:** `purchase.products { ids: string[] }` (≤ 100; empty: every product) → `{ products: [{ id, kind: "consumable" \| "durable" \| "subscription", title, description, price: { amount, currency, display } }] }`; `purchase.buy { product, quantity }` (1–99; the Player shows its own sheet and takes the money) → `{ purchase, product, quantity, state: "completed" \| "pending" }`, errors `cancelled`, `not_found`, `conflict` (a durable owned); `purchase.entitlements {}` → `{ entitlements: [{ purchase, product, kind, quantity, expires }] }` (owned durables, active subscriptions, unconsumed consumables; `expires` UTC ms or null); `purchase.consume { purchase }` → `{ purchase, product, consumed: true }`, `not_found` when not owned or consumed. **Ads:** `ad.ready { kind: "rewarded" \| "interstitial", placement }` → `{ kind, placement, ready }`; `ad.show { kind, placement }` → `{ kind, placement, outcome: "completed" \| "skipped" \| "shown" }`, errors `no_fill` (retryable), `rate_limited` (the platform's frequency cap); while the ad shows the Player sends `pause` before it and `resume` after it, and answers only after `resume`. **Achievements:** `achievement.unlock { id }` → `{ id, percent: 100, unlocked: true, new }`; `achievement.progress { id, percent }` (0–100; never lowered) → `{ id, percent, unlocked, new }`; `achievement.list {}` → `{ achievements: [{ id, percent, unlocked, time }] }`. **Identity:** `player.get {}` → `{ player, name, guest }`; `friends.list { count }` (1–200) → `{ friends: [{ player, name, online, playing }] }`, `unauthorized` for a guest; `invite.send { to: string \| null, message, payload }` (payload any JSON ≤ 1 KiB; `to` null: an invite link through the share sheet) → `{ invite, to }`, `cancelled` when the sheet closed; `invite.list {}` → `{ invites: [{ invite, from: { player, name }, payload, time, opened }] }` (`opened`: the invite the game was opened with). **Share:** `share { text, title, path, image }` (text 1–280, title ≤ 80; the Player builds the game's own Play link with `?share=<path>`; `image`: the frame drawn last) → `{ shared: true, target }`, `cancelled` when the sheet closed. **Notices:** unasked input pushed as `{ og: "player", protocol: 2, kind: "platform", notice: { kind, data, time? } }` with `kind` `purchase` (`{ purchase, product, quantity, state: "completed" \| "failed" \| "refunded" }`: a pending purchase settled, a refund), `entitlements` (`{ entitlements }`, changed without a call: another device, an expiry) or `invite` (the `invite.list` entry). New error codes `cancelled` (not retryable) and `no_fill` (retryable). A verification run answers every method from a scratch fake that never charges. | og.platform's `products`, `purchase`, `entitlements`, `consume`, `showAd`, `adReady`, `achievementUnlock`, `achievementProgress`, `achievements`, `player`, `friends`, `inviteSend`, `invites`, `share`, `supports` and `og.PlatformNotice`: the purchases, ads, achievements, identity and sharing commercial games need, behind the platform's accounts and payments. The engine keeps only `og.PlatformGrants` (which purchases a world delivered, so a consumable is delivered once across replays, relaunches and restores) and never treats it, a save or its log as proof of payment: a game asks for its entitlements on start. In a lockstep or rollback room each peer's answers are its own, so a system writing shared state hears none (docs/design.md §2.8). | Open. The shell passes each method through where the hello lists it and answers the rest `unavailable`; a published page without a Player answers every one `unavailable` (it never fakes a purchase); a dev page without a Player, `og run` and playtests answer from the local platform's deterministic fake (`src/platform-local.ts` `LocalFake`: `og run --platform-fake`, `PlaytestDesc.platform`), whose ads pause the game for a frame and answer after the resume. The Player's notices are forwarded to the engine (`shell/player.ts` kind `platform`). |

## Missing-GPU warnings and unchecked frames

| # | Ask | Why | Until then |
|---|---|---|---|
| 20 | **Optional:** set `OG_RENDERER=none` in GPU-less origin-agent sandbox images or `/etc/profile.d/origin-runner.sh` to make the warning expected. It is not a prerequisite for moving the engine pin. Read `renderer`, `adapterError` and unchecked counts in `og run` / `og test` JSON; report unchecked suites separately from passed suites. | An unavailable adapter warns loudly with wgpu's error and continues as renderer `none`: simulation, dumps and hashes are unchanged, screenshots are unchecked and `capture` is null. GPU tests/checks that did not run never count as passes. Exit status is zero unless an executed check failed. The switch removes only the warning's "unexpected" wording. | Existing images need no environment change. Keep the adapter warning and unchecked counts visible to the agent and player; zero exit status alone is not evidence that frames were checked. A sandbox class that gains a GPU leaves the switch unset. |

## Quick play and matchmaking

| # | Ask | Why | Until then |
|---|---|---|---|
| 21 | Rooms in Player protocol 2 gain `room.quick { mode, players, timeoutMs, name? }`, listed in `hello.capabilities` when the Player answers it: the platform's matchmaking puts the player in a queue keyed by (game, `mode`, `players`); `mode` is 1 to 32 of letters, digits, `.`, `_` and `-` (empty is `default`), `players` 1 to 16, `timeoutMs` 0 (wait until `room.leave`) to 600000. Once `players` players wait, the platform makes a room of capacity `players` that nobody else can join by id and pushes each of them the same `{ type: "joined", room, peer, peers }` as `room.join` (peers numbered in the order they queued, the room's other events as for any room). A wait past `timeoutMs` ends with `{ type: "error", op: "quick", code: "timeout", message }`; `room.leave` while queued takes the player out and answers `{ type: "left" }`; a second `room.quick` while queued or in a room is `invalid`. The queue may match by more than order (skill, region) as long as each match is `players` players of one game and mode. | og.platform.quickPlay (IDL `platform`): a game asks to be matched and hears og.RoomJoined once its room filled, then plays in lockstep as in any room (docs/design.md §2.10). The engine's side is done: the op is og-net's protocol `{ op: "quick", mode, players, timeoutMs, name }` (`crates/og-net/src/protocol.rs`), and natively `og relay` keeps the queue (`crates/og-net/src/relay.rs`, tested in `client.rs` and `packages/engine/tests/rooms.test.ts`). Matchmaking is the platform's (AGENTS.md §Ownership): the engine names no service. | On the web the shell passes `quick` to the Player's room host like `create` and `join`; a Player without `room.quick` should answer `{ type: "error", op: "quick", code: "unavailable" }`, which the game hears as og.RoomJoined with ok false and `unavailable`. Natively, quick play works through `og relay` (`og run . --room ws://127.0.0.1:47900/`, the arena's `quick:duel`). |

## The inspector's Save on a web page

| # | Ask | Why | Until then |
|---|---|---|---|
| 22 | The Stage (origin-agent's host of a dev build, the `og-probe` wire 1 at protocol 5, `origin-agent/packages/contracts/src/probe.ts`) accepts one pushed event from the page, `{ og: "probe", v: 1, kind: "scene_saved", scenes: [{ source, text }] }`, posted to the origin in `window.__OG_STAGE_HOST__` under the wire's rules (never `"*"`): `source` is a scene's project-relative path (`assets/scenes/main.ogt`, the boot scene's first), `text` its whole new text, at most 1 MiB a scene and 64 scenes an event. The Stage writes each `text` over `source` in the game's project (the sandbox's checkout, as the agent's own edits are written), refusing a `source` outside `assets/` or not ending in `.ogt`, and answers nothing on the wire; the creator sees the files changed as they see an agent's edit. Advertise it in the Stage's `hello` reply to the page as `capabilities: ["scene_saved"]`. | The inspector's Save (F1 in a development build, `docs/design.md` §3.6) is `save_scene`: natively the engine writes the scene files itself, but a web page writes no files (`shell/page.ts`, `saveScene` answers the text), so on the Stage an edit made in the panel is lost unless the page can hand the text to the host that owns the project. | A web page's Save answers the texts and writes nothing ("N lines ready; the page saves them"); the shell's save routing (`shell/wire.ts` `routeSavedScenes`) sends `scene_saved` only to a Stage whose `hello` lists the capability, and otherwise downloads each scene as a `.ogt` file named by its source (one file per scene, the browser's download) so the creator can put it back by hand. |

## Native accessibility in Origin Desktop

**Ask 25 — open.** Origin Desktop owns the OS AccessKit adapter for its
Preview window. The engine owns no window and must not attach an adapter
to another process's handle. The engine supplies the tree and consumes
actions through the shared-frame socket (docs/design.md §2.6).

After each shared-frame `Hello` on a new connection, Desktop sends
`{"AccessibilitySubscribe":{"version":1}}`. Every message is a `u32`
little-endian byte length followed by JSON. Version 1 pins AccessKit
0.25.2's serde representation. Desktop deserializes the `update` field of
`{"AccessibilityUpdate":{"version":1,"generation":0,"seq":42,"update":...}}`
as `accesskit::TreeUpdate` and applies every update in socket order.
The first update is a full tree; later updates contain changed nodes and
focus. Dropping an incremental update loses tree state, even when Desktop
skips the image for that sequence. Reconnection starts a new tree.

Bounds are in the shared frame's physical pixels. Desktop supplies the
Preview's placement transform and clips the embedded tree to the Preview.
Desktop forwards its adapter's `ActionRequest` unchanged in the `request`
field of `AccessibilityAction`, with `version: 1`. For example, in Rust:

```rust
let request = accesskit::ActionRequest {
    action: accesskit::Action::Click,
    target_tree: accesskit::TreeId::ROOT,
    target_node: accesskit::NodeId(7), // id from the received tree
    data: None,
};
let message = serde_json::json!({
    "AccessibilityAction": { "version": 1, "request": request }
});
let body = serde_json::to_vec(&message)?;
socket.write_all(&(body.len() as u32).to_le_bytes())?;
socket.write_all(&body)?;
```

The engine supports the actions advertised on each node: focus, click,
set value, increment, decrement, scroll into view, and directional page
scroll. It applies them on the UI tick, not on the socket thread. Until
this ask lands, native frames remain visual-only to the OS. The web DOM
mirror is unchanged.

## The inspector's prefab Apply and Revert on a web page

| # | Ask | Why | Until then |
|---|---|---|---|
| 23 | Beside #22's `scene_saved`, the Stage serves the page one call and takes one more pushed text. The call: `{ og: "probe", v: 1, kind: "call", method: "read_source", params: { source } }`, answered `{ kind: "result", result: { source, text } }` (or `{ kind: "error", code: "refused" }`) with the whole current text of a project file, refusing a `source` outside `assets/` or not ending in `.ogt` and answering at most 1 MiB. The pushed text: a prefab's Apply writes the prefab's file and the instance's scene, so the page sends both in one `scene_saved` (each `{ source, text }`, the prefab's `assets/prefabs/….ogt` among them), and the Stage writes every one as #22 says, all or none. Advertise the call as `capabilities: ["scene_saved", "read_source"]` in the Stage's `hello`. | The inspector's Apply (write an instance's override into its prefab, so every instance takes it) and Revert (back to the prefab's value) are `apply_prefab` and `revert_prefab`, which read the prefab's and the scene's source text and rewrite it (`docs/design.md` §3.6). Natively the engine reads the project's files; a web page has only the cooked bundle, so on the Stage Apply and Revert refuse with the file they could not read, and a creator who tunes a prefab instance on the web cannot make it the prefab's. | On a web page Apply and Revert say why they cannot run (the source is not readable here) and change nothing; Save still saves the instance's override into its scene (#22, or a download). Once the Stage reads sources, the page fetches the prefab's and the scene's texts before the edit and the result goes out as #22's `scene_saved`, or, to a Stage without `read_source`, as a download of each changed file, as #22's fallback. |

## Creation choices in origin-agent

**Ask 24, open.** The creation UI in `originai-game/origin-agent` collects
genre, view, look, mood, UI style, platform, input and players before the
first build. Languages and music have defaults. Use the released engine's
create package, not a separate platform vocabulary:

- `CREATION_CHOICES` owns the enums. `creationSchema(readCatalogue().map(s => s.slug))`
  returns the form's JSON Schema 2020-12. The look catalogue still owns its
  37 slugs. There are 14 installable UI packs in this revision, not 16;
  `cjk` and `shared` contain support assets, not styles.
- `resolveCreationChoices(premise, partialChoices)` returns suggestions
  with explicit choices preserved. Display and confirm the required
  choices. The language heuristic recognizes kana and Han text, otherwise
  defaults to English; the platform can supply a more accurate locale.
- Send the confirmed object as `create({ dir, premise, choices })` or the
  corresponding `og new` flags with `--no-prompt`. All callers get one
  empty skeleton; there are no archetypes or 2D overlays.
- Keep the resulting `project.ogt` `create` block as the decisions the
  build agent reads. Do not store an independent editable copy of those
  values that can diverge from the project.

The complete shape is:

```ts
{
  genre: "action-platformer", view: "side-2d", look: "16bit-pixel",
  mood: "warm", ui: "pixel", platform: "phone-landscape", input: "touch",
  players: "single", lang: ["zh", "en"], music: "chiptune"
}
```

Pass the confirmed `look`, `mood` and `ui` to `og create`: it writes
`assets/looks/<id>.ogt` and the top-level `look "looks/<id>.ogt"` in
`project.ogt`, so the platform no longer writes that file or patches
`og.game({})` (this replaces ask 12's source patching). The cook
resolves the preset, mood and UI pack. The entry remains `og.game({})`.
The other choices are constraints for the agent, not generated controls,
translations, cameras, multiplayer or music.

`profile` remains an optional runtime/build setting in project.ogt, omitted
by the skeleton. `create.view` records intent and does not replace it.
Languages install their script's font pack (docs/ui.md §3.4), which
og.i18n chains with no code; `og check` warns about a language with no
pack or table fonts, and og-i18n about missing tables. The lint does not
prove glyph coverage; the missing-glyph warning names what a frame lacks.

Changes after creation need the corresponding authored source/settings
update; `--force` is not an update API. `artBrief`, `readLook`, `presetLook`
and the knowledge mounting contracts keep their signatures. The mounted
payload and archive contents change. The distribution no longer contains
`archetypes/`, `create/archetypes/`, `create/template-2d/`, starter models,
sprites, scenes, audio, fonts, or gameplay playtests. `create/template/`
contains only the skeleton; `ui-sheet/` is a neutral style-control sheet,
not the old UI gallery. No platform code should read gallery screen names,
art or reference frames. `Roots.template`, `payload`, `installed`, `mount`
and `summary` keep their public signatures. See the breaking changelog.

## Music and sound generation from the audio brief

**Ask 26, open.** origin-agent's music and sound generation takes its
requests from the engine's `audioBrief`, as its image and model generation
takes them from `artBrief` (`docs/art.md` §2.2), instead of
writing prompts of its own:

- **Where the request comes from.** For a game, call
  `audioBrief({ ...readAudioChoices(projectDir), premise })` from the
  released `@origingame/create-game` (or the dist's
  `engine/knowledge/scripts/brief.ts`); before the game exists, from the
  creation form's confirmed `music`, `mood` and `genre` (ask 24). The
  premise is the one the maker wrote. Shapes: `docs/art.md` §2.9 and the
  knowledge README §The audio brief.
- **Music.** Generate each `cues[]` entry from its `prompt`, at its `bpm`,
  `mode`, `beatsPerBar` and `seconds`, honouring `loop` (seamless at the
  bar line, no fade or tail) and, when `stems` is not empty, one file per
  stem of identical length, sample-aligned, named
  `music/<id>-<stem>.ogg` beside the mix `music/<id>.ogg`. Deliver
  `format` (Ogg Vorbis, 48 kHz stereo, −16 LUFS integrated, true peak
  −1 dBFS); og cook decodes it with no converter (og-audio §5). A rhythm
  song's tempo must be exact: verify the delivered file's beat grid
  against `bpm` (a drift of more than 5 ms over the song is a refusal)
  before writing it.
- **Sound effects.** Generate each `sfx[]` entry from its `prompt`,
  `variants` takes as `audio/<id>.wav`, `audio/<id>-2.wav`, …, in
  `format` (WAV, 48 kHz mono, peak −1 dBFS), trimmed to start on the
  transient.
- **Writing it.** Put each file at `assets/<file>` in the game's project
  through the agent's own file edits, and record the model and request in
  the game's asset log as for art (`ai.mark` at publication, ask 6). Leave
  `presets[]`' `.ogt` stand-ins in place: the build agent switches each
  load once its file exists (og-audio §2). Never overwrite a file the
  maker replaced by hand.
- **Budget.** A game's full request is about 4 to 6 cues (1 to 3 minutes
  of music, with stems for the play loops) and 10 to 20 effects; the
  platform decides what it generates at creation and what on request,
  and the presets keep the game audible meanwhile.

Until then the engine's side stands alone: `og new` records the choices,
`audioBrief` lists the requests, and the presets synthesised by og cook
give every game its sounds. Nothing is generated.

## The engine measures, the agent judges

**Ask 27, open.** From the first engine version after 0.7.7, engine
commands report facts and measurements and never a taste verdict. origin-agent stops calling or describing what is
gone, and keeps the judgement in its own design (its skills, prompts and
evals):

- **`og polish` is removed**, with `@origingame/engine/node/polish`, its
  `og-out/polish/` report and its scores. Nothing replaces it inside the
  engine: review a build with `og test` (the playtests), `og ui check`
  and the frames (`og run --headless --commands`, `og frame measure`),
  judged by the agent (the engine's guides, `og api show testing`, and
  origin-agent's own skills).
- **`og ui check` exits 0 when it ran** and 1 when it could not; exit 3
  (an error finding) is gone. Its findings (overlap, text-cut,
  text-truncated, offscreen, safe-area, covered, pseudo-overflow,
  focus-reach, focus-trap, a11y-name, a11y-locale, feedback, art-slot)
  carry `rule`, `paths`, `measured`, `prop`, `fix`, `cells`, `value`,
  `unit` and `at`, and no `severity`, `limit` or `bar`. Contrast, text
  size, touch target, text scale, primary actions per layer and the
  element count are `measures` (`measure`, `paths`, `value`, `unit`,
  `detail`, `prop`, `cell`, `rect`) held to no bar; contrast is no longer
  a finding. The usual thresholds (4.5:1 and 3:1, 7:1 in high contrast;
  44 px touch targets; 12 px text, 18 px at 1080p) are the `screens`
  guide's §5 (`og api show screens`), for the agent to apply.
- **`og build --publish` no longer refuses a game in the default UI
  style**: a game may ship the default style, a built-in pack or its own.
- **`og style check --critic` is removed**, and the JSON's
  `contract.style` is `{ slug }` (no `checks`, no `intended`). The
  measurements against the game's own art contract and exit 3 off that
  contract stay. origin-agent's `critique_style` already reads the
  checks from the catalogue itself, so it is not affected.
- **`og run --device` has no scores**: device.json and `--json` lose
  `scores`; the measures and findings stay.

Callers on origin-agent `main`, as read at 73a84f7. No server or
runner code invokes `og polish`, `--critic`, branches on `og ui check`'s
or `og style check`'s exit 3, or relies on the publish refusal; what
names them is documentation, a skill and an offline eval:

- `docs/product/game.md:336` (the command table's `og polish .` row),
  §Polish at `:463`–`:477`, the skill table's og-taste and og-feel rows at
  `:601`–`:602` ("`og polish`'s taste measures", "feel measures") and
  `:611` (the pin "with `og polish` in" its command list).
- `docs/product/product-design.md:686` ("The engine's `og polish` scores
  a build against the taste and feel checklists").
- `server/prompts/create.md:43` (og-build "for the cook, run, observe,
  test, polish and ship loop").
- `knowledge/game/skills/art-director/SKILL.md:52`–`:55` (`og ui check`
  "measures contrast, text size, touch targets ... each against the
  engine's bar") and `:82`–`:84` ("a finding `og ui check` marks as a
  warning"): the bar is now the art bible's or the `screens` guide's, and there are
  no warnings, only findings and measures.
- `server/scripts/eval/art-director.ts:60`–`:65` reads
  `f.severity` and `f.bar` of each `og ui check` finding, and
  `server/evals/art/cases.json` (its `ui` documents, ten `severity` or
  `bar` fields, and case `:983`'s "just over the engine's 3:1"): render
  contrast from `measures` (`measure: "contrast"`, `value`) instead.

Until then an origin-agent pinned to 0.7.7 sees no change; one that pins
the next version and still runs `og polish` gets an unknown-command error,
and one that waits for `og ui check`'s exit 3 never sees it.

## The engine ships no skills

**Ask 29, open.** From the first engine version after 0.7.7 the engine
mounts no `game/skills/` and no `guides/`: the 25 engine skills (`og-2d`,
`og-ai`, `og-anim`, `og-audio`, `og-build`, `og-core`, `og-destruction`,
`og-feel`, `og-game`, `og-i18n`, `og-model`, `og-net`, `og-physics`,
`og-platform-sdk`, `og-render`, `og-story`, `og-taste`, `og-ui`,
`architecture-discipline`, `component-forge`, `level-texturing`,
`localization`, `recipe-craft`, `ui-craft`, `visual-acceptance`) are
deleted, and so are their `skill:<name>` pack pins. What a game agent reads
of the engine is the `og` tool: `og api find <words>` (ranked entries with
scores) and `og api show <name>` over one index of the API and the
engine's guides (`og api show start`, `testing`, `determinism`, `looks`,
`screens`, `animation`, `art-review`, `three-to-og`). The payload keeps
`game-scaffold/origin-engine-ts/`, and the package keeps
`knowledge/looks/catalogue.json` and `previews/`; `payload`,
`defaultRoots`, `installed`, `mount`, `summary`, `artBrief`, `readLook`
and `presetLook` keep their signatures.

origin-agent `main` c4d64ef already sends no game agent to an engine
skill (its instructions, create mode and own skills point at `og help` and
`og api find`/`show`). What still reads the engine's old layout:

- `server/src/core/engine.ts:3017`–`:3019`: the fallback that resolves
  `guides/` through the `skill:og-game` pack
  (`game/skills/og-game/guides/`). The guides are `og api` entries now;
  drop the lookup.
- `server/src/core/skill-pages.ts`: comments on og-game's and og-model's
  sizes (comments only).
- `scripts/knowledge/production-bundle.test.mjs:50`–`:54` keeps passing (it
  already excludes `game/skills/og-*` other than the platform's
  `og-platform`).

## Picking the UI

**Ask 30, open.** origin-agent's point-and-comment asks the engine's
`pickUi(x, y)` (wire member `pick_ui`, an engine member beside protocol 5,
listed in `hello.members`) first, and `pick(x, y)` second. `pickUi` answers
the topmost UI element under the point that shows something or takes input:
`{ id, path, kind, text, rect: { x, y, width, height }, layer, source, part,
rule, rules }`, `source` the document line that built it
(`ui/shop.ogt:42`), `part` the style part it is drawn as
(`Button.base[primary][hover]`), `rules` the style rules that dress it, each
`{ selector, source }` with its file and line, in the order they apply, and
`rule` the most specific. A layout container that draws nothing answers
`null`, so a HUD's full-screen root never hides the scene. `pick` keeps
protocol 5's `PickResult`; its `source` now names the scene line that
declares the entity (`scenes/main.ogt:12`), and outside a publish build it
carries the same UI element as an extra `ui` field. The platform highlights
the element's `rect` and opens its `source`, then the rule's. Nothing in
protocol 5 changes.
