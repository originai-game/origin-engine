# Platform asks

What the engine's `og.platform` (IDL module `platform`, docs/design.md §2.8)
and its tools need from the OriginGame platform and origin-agent that they
do not give today. The engine works without each one: the shell's bridge
(`packages/engine/shell/platform.ts`) maps what it can onto Player protocol 2
and answers the rest `unavailable`. Each ask names the shape the engine
would use, so the bridge's mapping shrinks to a rename when it lands. The
Player is one provider of the engine's provider contract
(`@origingame/engine/provider`, docs/design.md §2.8 Providers); `point` and
`ai.mark` are OriginGame's host extensions, not core methods.

Asks are numbered once and never renumbered or reused. Only open asks are
listed; landed ones (1–8, 12, 14, 15, 18, 27) are in git history. Nothing
here carries a credential: every method is a name and arguments, and the
Player owns the session.

## The Player's clock

**Ask 9, open.** The Player's `hello` carries `time` (UTC ms, the Player's
clock), and every answer carries `time` when the Player answered it, so
games read one clock the platform can correct rather than each browser's.
State: the platform has implemented it (the framed Player corrects its
clock from same-origin `GET /__og/time` and never moves it backwards), not
yet deployed. The engine reads hello and result time on both transports.
Until then the Worker gives og.platform the browser's clock and
`og.platform.info(world).clock` reads `browser`; with a stamped answer it
reads `player`.

## The top-level page

**Ask 28, open in production.** Player protocol 2 over same-origin HTTP
for a top-level engine page: `POST /__player/v2/<game>/<release>/{hello,call,report}`
on the game's own origin, the hand-off code redeemed from the hello's body,
the framed Player's answers and errors, and the session reports filed as
the Player files them; and Play opening a release whose entry says
`<meta name="og-player" content="frame page">` as its own top-level
document (307 to the release entry), the Stage page likewise for a Stage
build. The contract is origingame `docs/player-protocol.md` §The top-level
engine page; the shell speaks it (`docs/web-export.md` §The top-level
page). The platform still needs the game-host ingress to admit `POST` with
`Authorization` (never `Cookie`) on `/__player/v2/`, then the E2B
templates rebuilt, in that order. Until then every engine page is framed.

## The Stage overlay in the engine's document

**Ask 10, open.** In in-document mode (a Stage build with `og-player` …
`page`), the Stage overlay answers Player protocol 2 on the page's own
window: its stand-in Player (`createGameListener` in `sdk/src/stage-page.ts`)
takes `window` as the game window and posts its answers to `window` at
`location.origin`, skips the Player messages it posted itself (it hears its
own `hello`), and answers as it does framed (`hello` with `started: true`,
`gesture`, the scratch store's `result`s, `pause`/`resume`). A Reload
starts the scratch store empty unless it is kept per ticket in
`sessionStorage`. The shell speaks its side (`docs/web-export.md` §Player
protocol 2); until the overlay answers, a top-level Stage build plays on
the page's memory and draws its own gate after 300 ms. State: the
platform's Stage runs in-document against this shell; production waits on
ask 28's ingress change.

## Lanes: threaded engine pages

**Ask 11, open: not verified against production.** Keep serving a release
that ships `origingame.runtime.json` `{"version":1,"crossOriginIsolated":true}`
with COOP `same-origin` and COEP `require-corp` on its entry document and
every file under it, CORP `cross-origin` on each file, and `allow="…
cross-origin-isolated"` on the Player's iframe while a release is framed;
at the top level the 307 must land on a response carrying COOP and COEP.
Send no `worker-src` or `script-src` on the play origin that excludes
`blob:`. Without isolation the shell plays the single-threaded module with
the same results and no lanes (`docs/web-export.md` §Lanes). Nothing should
need to change; verify by publishing one lanes build and checking
`__runtime_probe__.lanes.on` on Play, framed and at the top level, on
desktop Chrome and a phone.

## The type checker in the sandbox

**Ask 13, open.** The `engine` class sandbox image keeps
`@origingame/engine`'s `optionalDependencies` when it pre-installs the
engine (no `--omit=optional`), so `node_modules/@tsrs/<platform>/tsrs` and
its `lib.*.d.ts` are in the image and `og check` never reaches the
registry; count it in the image budget (about 33 MB, 56 MB if bun also
installs the musl package on glibc). Until then `og check` falls back to
tsgo through `bun x` (the network or bun's cache), and `og doctor` reports
tsrs missing with the fix.

## Rooms

**Ask 16, open.** Rooms in Player protocol 2, listed in
`hello.capabilities` as `room.create`, `room.join`, `room.leave`,
`room.send`. Calls (each `result` only acknowledges; a refusal is `{ ok:
false, error: { code, message } }` with `code` one of `unavailable`,
`not_found`, `full`, `invalid`, `offline`): `room.create { name?, capacity? }`
(capacity 2 to 16), `room.join { room, name? }` (`room` 1–64 of letters,
digits, `.`, `_`, `-`), `room.leave {}`, `room.send { to?, channel?, data? |
bytes? }` (`to` a peer number, else every other peer; `bytes` base64;
64 KiB a game message, 16 MiB for the engine's own `og.`-channel messages).
Events, pushed as `{ og: "player", protocol: 2, kind: "room", event }` in
the order they happened: `{ type: "joined", room, peer, peers: [{ peer,
name }] }` (`peers` includes this peer), `{ type: "peer_joined", peer,
name }`, `{ type: "peer_left", peer }` (after every message that peer
sent), `{ type: "message", from, channel?, data? | bytes? }`, `{ type:
"left", reason? }`, `{ type: "error", op, code, message }`. Peer numbers are
given in join order from 0 and never reused; messages from one sender
arrive in order. The platform relays only: lockstep runs in every engine.
A Play link `?room=<id>` puts the page in that room. The shapes are og-net's
protocol (`crates/og-net/src/protocol.rs`).

State: the engine's side is done, the Play link included. The platform
side is implemented (`rooms-service/src/player-relay.ts`,
`sdk/src/player-rooms.ts`); open are its deployment, `?room=` on the Play
link and a two-Player acceptance. Until then a Player whose hello lists no
`room.join` gives roomInfo host `none`, and roomCreate/roomJoin are
refused.

## Source maps and field telemetry

**Ask 17, open.** Keep field telemetry's fields when filing `crash` and
`sample` (framed Player and the play origin's
`/__player/v2/<game>/<release>/report`), shapes in `docs/web-export.md`
§Telemetry, and never answer 400 for them (a 400 drops the whole report).
At publication, take a publish build's `sourcemaps/` out before the release
is served and store it privately under og-build.json's `sourcemaps.build`;
map a stored crash's `stack` with `mapStack` (`docs/web-export.md` §Source
maps) when showing it. Aggregate `frames` per release, `stages` and
`failures`. Accept the `end` sample by origin and the report's game and
release, not by `e.source === iframe.contentWindow`, which no longer holds
on `pagehide`. The engine sends them already; a play origin that drops
unknown fields loses nothing it had.

## Purchases, ads, achievements, identity and share

The engine owns the contract and the game-side API (IDL `platform`); the
accounts, money, ad networks and catalogue are the platform's. Every id the
Player hands back is opaque, at most 128 of letters, digits, `.`, `_`, `-`
and `:`, and scoped to this game.

**Ask 19, open.** Player protocol 2 methods, each listed in
`hello.capabilities` when the Player answers it (the shell passes them
through by name):

- **Purchases:** `purchase.products { ids: string[] }` (≤ 100; empty:
  every product) → `{ products: [{ id, kind: "consumable" | "durable" |
  "subscription", title, description, price: { amount, currency, display } }] }`;
  `purchase.buy { product, quantity }` (1–99; the Player shows its own
  sheet) → `{ purchase, product, quantity, state: "completed" | "pending" }`,
  errors `cancelled`, `not_found`, `conflict` (a durable owned);
  `purchase.entitlements {}` → `{ entitlements: [{ purchase, product, kind,
  quantity, expires }] }` (`expires` UTC ms or null);
  `purchase.consume { purchase }` → `{ purchase, product, consumed: true }`,
  `not_found` when not owned or consumed.
- **Ads:** `ad.ready { kind: "rewarded" | "interstitial", placement }` →
  `{ kind, placement, ready }`; `ad.show { kind, placement }` → `{ kind,
  placement, outcome: "completed" | "skipped" | "shown" }`, errors
  `no_fill` (retryable), `rate_limited`; the Player sends `pause` before
  the ad and `resume` after it, and answers only after `resume`.
- **Achievements:** `achievement.unlock { id }` → `{ id, percent: 100,
  unlocked: true, new }`; `achievement.progress { id, percent }` (0–100;
  never lowered) → `{ id, percent, unlocked, new }`; `achievement.list {}`
  → `{ achievements: [{ id, percent, unlocked, time }] }`.
- **Identity:** `player.get {}` → `{ player, name, guest }`;
  `friends.list { count }` (1–200) → `{ friends: [{ player, name, online,
  playing }] }`, `unauthorized` for a guest; `invite.send { to: string |
  null, message, payload }` (payload JSON ≤ 1 KiB; `to` null: a link
  through the share sheet) → `{ invite, to }`, `cancelled` when the sheet
  closed; `invite.list {}` → `{ invites: [{ invite, from: { player, name },
  payload, time, opened }] }`.
- **Share:** `share { text, title, path, image }` (text 1–280, title ≤ 80;
  the Player builds the game's Play link with `?share=<path>`) →
  `{ shared: true, target }`, `cancelled` when the sheet closed.
- **Notices:** unasked input pushed as `{ og: "player", protocol: 2, kind:
  "platform", notice: { kind, data, time? } }` with `kind` `purchase`
  (`{ purchase, product, quantity, state: "completed" | "failed" |
  "refunded" }`), `entitlements` (`{ entitlements }`) or `invite` (the
  `invite.list` entry).
- New error codes `cancelled` (not retryable) and `no_fill` (retryable). A
  verification run answers every method from a scratch fake that never
  charges.

The engine keeps only `og.PlatformGrants` (which purchases a world
delivered) and never treats it, a save or its log as proof of payment.
Until then the shell answers each method the hello does not list
`unavailable`, a published page without a Player answers every one
`unavailable`, and dev pages, `og run` and playtests answer from the local
deterministic fake (`src/platform-local.ts` `LocalFake`).

## Missing-GPU warnings and unchecked frames

**Ask 20, optional.** Set `OG_RENDERER=none` in GPU-less origin-agent
sandbox images (or `/etc/profile.d/origin-runner.sh`) so the missing-adapter
warning reads as expected; it is not a prerequisite for moving the engine
pin. Read `renderer`, `adapterError` and the unchecked counts in `og run` /
`og test` JSON and report unchecked suites apart from passed ones: a zero
exit status alone does not prove frames were checked. Without the switch
nothing breaks; the warning only says "unexpected".

## Quick play and matchmaking

**Ask 21, open.** Rooms in Player protocol 2 gain `room.quick { mode,
players, timeoutMs, name? }`, listed in `hello.capabilities`: the
platform's matchmaking queues the player by (game, `mode`, `players`);
`mode` is 1 to 32 of letters, digits, `.`, `_` and `-` (empty is
`default`), `players` 1 to 16, `timeoutMs` 0 (wait until `room.leave`) to
600000. Once `players` players wait, the platform makes a room of capacity
`players` that nobody else can join by id and pushes each of them the same
`joined` as `room.join` (peers numbered in queue order). A wait past
`timeoutMs` ends with `{ type: "error", op: "quick", code: "timeout",
message }`; `room.leave` while queued answers `{ type: "left" }`; a second
`room.quick` while queued or in a room is `invalid`. The queue may match by
more than order as long as each match is `players` players of one game and
mode. The engine's side is done (og-net's `{ op: "quick", … }`; natively
`og relay` keeps the queue). Until then a Player without `room.quick`
should answer `{ type: "error", op: "quick", code: "unavailable" }`.

## The inspector's Save on a web page

**Ask 22, open.** The Stage (the `og-probe` wire 1 at protocol 5,
`origin-agent/packages/contracts/src/probe.ts`) accepts one pushed event
from the page, `{ og: "probe", v: 1, kind: "scene_saved", scenes: [{
source, text }] }`, posted to the origin in `window.__OG_STAGE_HOST__`:
`source` is a scene's project-relative path (`assets/scenes/main.ogt`),
`text` its whole new text, at most 1 MiB a scene and 64 scenes an event.
The Stage writes each `text` over `source` in the game's project, refusing
a `source` outside `assets/` or not ending in `.ogt`, and answers nothing.
It advertises this in its `hello` reply as `capabilities: ["scene_saved"]`.
Until then the shell's save routing (`shell/wire.ts` `routeSavedScenes`)
downloads each scene as a `.ogt` file named by its source, since a web page
writes no files.

## The inspector's prefab Apply and Revert on a web page

**Ask 23, open.** Beside #22, the Stage serves the call `{ og: "probe", v:
1, kind: "call", method: "read_source", params: { source } }`, answered
`{ kind: "result", result: { source, text } }` (or `{ kind: "error", code:
"refused" }`) with the whole current text of a project file under the same
rules as #22 and at most 1 MiB. A prefab's Apply sends the prefab's file and
the instance's scene in one `scene_saved`, which the Stage writes all or
none. It advertises `capabilities: ["scene_saved", "read_source"]`. Until
then Apply and Revert on a web page say the source is not readable here and
change nothing.

## Native accessibility in Origin Desktop

**Ask 25, open.** Origin Desktop owns the OS AccessKit adapter for its
Preview window; the engine supplies the tree and consumes actions through
the shared-frame socket (docs/design.md §2.6) and never attaches to another
process's window. After each shared-frame `Hello` on a new connection,
Desktop sends `{"AccessibilitySubscribe":{"version":1}}`. Every message is a
`u32` little-endian byte length followed by JSON. Version 1 pins AccessKit
0.25.2's serde representation. Desktop deserializes the `update` field of
`{"AccessibilityUpdate":{"version":1,"generation":0,"seq":42,"update":...}}`
as `accesskit::TreeUpdate` and applies every update in socket order: the
first is a full tree, later ones changed nodes and focus, and dropping one
loses tree state. Reconnection starts a new tree. Bounds are in the shared
frame's physical pixels; Desktop applies the Preview's placement transform
and clips to it. Desktop forwards its adapter's `ActionRequest` unchanged
as `{"AccessibilityAction":{"version":1,"request":...}}`; the engine applies
the actions each node advertises on the UI tick. Until then native frames
are visual-only to the OS.

## Creation choices in origin-agent

**Ask 24, open.** The creation UI collects genre, view, look, mood, UI
style, platform, input and players before the first build (languages and
music have defaults), from the released engine's create package rather than
a platform vocabulary:

- `CREATION_CHOICES` owns the enums, and
  `creationSchema(readCatalogue().map(s => s.slug))` returns the form's
  JSON Schema 2020-12.
- `resolveCreationChoices(premise, partialChoices)` returns suggestions with
  explicit choices preserved; display and confirm the required ones.
- Send the confirmed object as `create({ dir, premise, choices })` or the
  `og new` flags with `--no-prompt`. The `look`, `mood` and `ui` go to
  `og create`, which writes the look file and project.ogt's `look`.
- Keep the resulting `project.ogt` `create` block as the decisions the build
  agent reads, with no separate editable copy that can diverge. Later
  changes edit the project's sources; `--force` is not an update API.

```ts
{
  genre: "action-platformer", view: "side-2d", look: "16bit-pixel",
  mood: "warm", ui: "pixel", platform: "phone-landscape", input: "touch",
  players: "single", lang: ["zh", "en"], music: "chiptune"
}
```

## Music and sound generation from the audio brief

**Ask 26, open.** origin-agent's music and sound generation takes its
requests from the engine's `audioBrief`, as image generation takes them from
`artBrief` (`docs/art.md` §2.2), instead of writing prompts of its own:

- **The request.** `audioBrief({ ...readAudioChoices(projectDir), premise })`
  from the released `@origingame/create-game`; before the game exists, from
  the creation form's `music`, `mood` and `genre` (ask 24). Shapes:
  `docs/art.md` §2.9.
- **Music.** Each `cues[]` entry from its `prompt`, at its `bpm`, `mode`,
  `beatsPerBar` and `seconds`, seamless at the bar line when `loop`, and,
  when `stems` is not empty, one sample-aligned file per stem
  (`music/<id>-<stem>.ogg`) beside the mix `music/<id>.ogg`, in `format`
  (Ogg Vorbis, 48 kHz stereo, −16 LUFS integrated, true peak −1 dBFS). A
  rhythm song whose beat grid drifts more than 5 ms from `bpm` over the song
  is refused.
- **Sound effects.** Each `sfx[]` entry from its `prompt`, `variants` takes
  as `audio/<id>.wav`, `audio/<id>-2.wav`, …, in `format` (WAV, 48 kHz mono,
  peak −1 dBFS), trimmed to start on the transient.
- **Writing it.** Each file at `assets/<file>` through the agent's own file
  edits, recorded in the asset log as for art; the `presets[]` stand-ins stay
  in place, and a file the maker replaced is never overwritten.

Until then `og cook`'s synthesised presets give every game its sounds.

## The engine ships no skills

**Ask 29, open.** The engine mounts no `game/skills/` and no `guides/`;
what a game agent reads of the engine is `og help`, `og api find` and
`og api show`. origin-agent's agents already point there; what still reads
the old layout is the fallback in `server/src/core/engine.ts` that resolves
`guides/` through the `skill:og-game` pack, which should be dropped.

## Picking the UI

**Ask 30, open.** origin-agent's point-and-comment asks the engine's
`pickUi(x, y)` (wire member `pick_ui`, an engine member beside protocol 5,
listed in `hello.members`) first, and `pick(x, y)` second. `pickUi`
answers the topmost UI element under the point that shows something or
takes input: `{ id, path, kind, text, rect: { x, y, width, height }, layer,
source, part, rule, rules }`, `source` the document line that built it
(`ui/shop.ogt:42`), `part` the style part it is drawn as
(`Button.base[primary][hover]`), `rules` the style rules that dress it,
each `{ selector, source }`, in the order they apply, and `rule` the most
specific. A layout container that draws nothing answers `null`. `pick`
keeps protocol 5's `PickResult`; its `source` names the scene line that
declares the entity, and outside a publish build it carries the UI element
as an extra `ui` field. The platform highlights `rect` and opens `source`,
then the rule's.

## The cursor over Origin Desktop's Preview

**Ask 31, open.** After each shared-frame `Hello` on a new connection
Origin Desktop sends `{"CursorSubscribe":{"version":1}}`; the engine
answers `{"Cursor":{"version":1,"keyword":"<css keyword>","image":<image or null>}}`
for the current shape and again whenever it changes (docs/design.md §2.6).
`keyword` is a CSS cursor keyword; `image`, when not null, is
`{"width","height","rgba","hotspot":[x,y]}` with `rgba` the base64 of
`width × height` RGBA8 pixels, rows top first, sRGB, straight alpha, at
most 128×128. Desktop shows the image with its hotspot over the Preview
(scaled by the window's scale factor), else the keyword's OS cursor. Until
then the Preview keeps Desktop's arrow.
