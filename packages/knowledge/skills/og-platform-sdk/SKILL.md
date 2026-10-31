---
name: og-platform-sdk
description: The OriginGame platform from inside an Origin Engine game — og.platform's store (saves with revisions and conflicts), leaderboards, telemetry, Origin Point, AI-content marks and the settings helpers, as requests answered at a later tick; how they work in the Player, on the stage and under og run; how to test them with dump_platform and --platform-store; and what to ask the platform for when a method is missing. Use when a game saves progress or settings, shows scores, files events or asks for points (the platform's og-platform skill covers the Player protocol around it).
---

Game code never talks to the platform. It asks the engine, `og.platform.*`,
by method name and arguments; the host answers: the web shell over the
Player's protocol 2 (to the Player framing the page, or, when the platform
opens the published page itself at the top level, to its play origin over
same-origin HTTP), falling back to the browser's own storage; `og run`
from a local JSON store. No credential, URL or cookie reaches game code,
and every call works (or answers `unavailable`) everywhere, so the game
never branches on where it runs. The API is in `og api show platform`;
the platform's `og-platform` skill is the Player and readiness side.

## What works

Hollowbell (`examples/hollowbell`, written from this package alone) keeps
its run in the store (`og.platform.storeSet` at the revision it read, a
conflict read again and written back) and proves Continue across a
relaunch with a `store: "keep"` playtest; Lantern Vale
(`examples/lantern-vale`) keeps its settings through
`og.platform.saveSettings`. The relaunch worked the first time it was
asked for as §4 says. The Kestrel Stair (`examples/kestrel-stair`) keeps
its whole world at a waystone (`world.snapshot()`, about 78 KB of text, in
parts under the store's 64 KiB a key) and continues it after a
`store: "keep"` relaunch, its progress at the revision read with a
conflict merged, and a won climb on an ascending leaderboard. The store's
limits are the platform's (64 KiB a key, 64 keys and 1 MiB per player and
game), so the world goes through `og.platform.saveSlot`: its parts
(`run.a0`, `run.a1`) in whichever of the slot's two banks of keys
(`<slot>.a0..`, `<slot>.b0..`) the last save did not use, then the
manifest `run` at the revision read, so a save cut short leaves the
previous one whole and two devices saving at once get a conflict, not a
mix; one request a call. Its title reads `og.platform.loadSlot` with
`metaOnly` for Continue's place and hour, and Continue reads the parts,
checked against the manifest's length and checksum.

## 1. Requests, answered later

Every call returns a request number at once; the answer arrives at the
start of a later tick, when `og.platform` sends `og.PlatformAnswered`.

```ts
// templates/archetypes/runner's Records system.
if (time.tick === 0) {
  const asked = og.platform.storeGet(world, "best");      // the request, at once
  run.bestRequest[i] = asked.ok ? asked.value : 0;
  return;
}
const request = run.bestRequest[i]!;
if (request === 0) return;
const answer = og.platform.result(world, request);       // null while in flight
if (answer === null) return;
run.bestRequest[i] = 0;
if (answer.ok && typeof answer.data.value === "number") run.best[i] = Math.max(run.best[i]!, answer.data.value);
```

- Keep the request number in a component or resource (snapshots carry it),
  never in a closure.
- Answers are host input, like key presses: a replay records them, so the
  same answers give the same world.
- Every answer carries `time` (UTC ms): the host's clock when it was
  answered (a `saveSlot` or `loadSlot` answer, the time of the store
  answer that ended it, so a slot's load is as good a clock as a
  `storeGet`). `og.platform.now(world)` is the host's clock at the last
  answer (or when the platform attached) advanced by the ticks since, so
  it is the one wall clock a game may read: the time away is a store
  answer's `time` less the time the save wrote down
  (`templates/archetypes/idle`, `tests/away.og.ts`). `info(world).clock`
  says whose it is: `player`, `browser` (the web host's own, until the
  Player sends its time), `system` (native), `fixed` (`og run --clock`,
  playtests), `none` before the platform attaches (`now` answers 0).
- An error has a stable `code` (`offline`, `timeout`, `invalid`,
  `unauthorized`, `not_found`, `conflict`, `too_large`, `rate_limited`,
  `unavailable`, `failed`) and `retryable`. Handle `unavailable` as
  "this feature is off here" and keep playing.

## 2. What to use

| Need | Call | Notes |
|---|---|---|
| Progress, unlocks, a best score | `storeGet(world, key)`, `storeSet(world, { key, value, revision })`; the whole world: `world.snapshot()` text through `saveSlot(world, { slot, text, meta })` (as many keys as it needs, the manifest last, one request) and `loadSlot(world, { slot, metaOnly })`, back with `world.restore` (og-game §8) | one key per independent thing; values ≤ 64 KiB, at most 64 keys and 1 MiB per player and game; pass the revision you read to refuse a stale write (`conflict`, whose `error.revision` is where the key is now: read again and merge), `create: true` to write only a key never written, no revision for "the last write wins" |
| Volume, controls, language | `loadSettings(world)`, `saveSettings(world, settings)` | the key `og.settings`, last save wins, local fallback when offline |
| A leaderboard | `leaderboardSubmit(world, { board, score })`, `leaderboardTop`, `leaderboardAround` | the board keeps the player's best; show "around me" as well as the top |
| Analytics | `telemetry(world, "round.over", { score, coins })` | one-way; a name of 1–64 letters, digits, `.` and `_` (the Player files it as `game.custom.<name>`), at most 16 scalar attributes; never personal data |
| Origin Point | `point(world, { reason, amount })` | the platform decides what it grants (today nothing under a Player: `{ granted: 0, balance: null }` until an approved point policy; show nothing that depends on it) |
| AI-generated content shown | `aiMark(world, { kind, label })` | disclosure, not permission |
| Play with others in one room | `roomCreate(world, { name })`, `roomJoin(world, room)`, `roomLeave`, `roomSend(world, data, to?)`, `roomPeers`, `roomInfo`; the answers as `og.RoomJoined`, `og.RoomPeerJoined`, `og.RoomPeerLeft`, `og.RoomMessage` (read one with `roomMessage`) | natively `og relay` with `og run --room ws://127.0.0.1:P/<room>`; on the web the Player's rooms, when its hello lists `room.join` (`roomInfo(world).host` is `none` otherwise and joins are refused). A message is this peer's host input only: for a game both peers simulate, use og.net's lockstep (`og.Lockstep`, `og.PlayerInput`, og-net §1) and send nothing yourself |

A payload keeps the number it was given: an `f32` field's value is single
precision promoted to a JSON number (`f32(2.7)` sends `2.700000047683716`),
and the store neither rounds nor widens it. Compare with a tolerance in
a playtest, round only for display, or keep a value that must compare
exactly in an `f64` (og-build §1, Values in the JSON).

`og.platform.info(world)` says what answers here (`player`, `local` or
`none`) and the methods it answers (`capabilities`): check it rather than
trying.

## 3. Rules

- Read saves on the first tick and show defaults until they answer;
  never block the title on the platform. On the web the title behind the
  play gate is not tick 1's defaults: the shell takes the first tick, then
  one more tick each time the store answers what that tick asked (the
  settings, a slot's manifest, then its parts), up to 8 ticks, before the
  first frame while the answers come within 300 ms and drawn as they land
  for 5 s after (docs/web-export.md §The shell). So apply an answer the
  tick it arrives (the language, Continue's label), and apply it before
  the title is built that tick: systems run in `GameDesc.systems` order,
  so the system that reads the answers comes before (or is) the one that
  builds the title. A title built by a `Ui` system listed before an
  `Answers` system draws that tick in the defaults even though the answer
  landed, and the frame behind the gate is that tick's: the saved Spanish
  shows only after the tap. The fix is the order: list the system that
  reads every answer first (`systems: [Answers, ..., Ui, Saves]`, writes
  last), or read the answers at the top of the system that builds the
  title. A title that waits for a later tick, or a screen transition that
  only starts after the tap, still shows its defaults behind the gate.
- The frame behind the gate is a still of the opening title a few ticks
  in, held until the tap, so anything on it that moves into place over
  its first second stands part-way there: the style's panel entrance and
  its controls' stagger (about half a second), a transition, a fade the
  game plays itself. Build the screen the game opens on settled while it
  is that screen, and let every later screen enter as usual:
  `og.ui.mainMenu("Title", { ..., instant: !g.leftTitle })` with
  `g.leftTitle` set once the shown screen is another (examples/kestrel-stair
  `src/ui.ts`); `og.ui.anchor` and `og.ui.transition` take the same
  `instant`. A game's own intro on the title (a fade from black, a logo)
  starts complete on the first tick, or plays after the player's first
  press.
- Settings are a pair: a game that calls `saveSettings` also calls
  `loadSettings` on its first tick and, when it answers, applies every
  field it saves, through the same code its settings screen uses. Ask
  for the saved language with `og.i18n.setLocale` even when it matches
  the default (the host's own locale may be another). Saving alone
  passes every test that never relaunches, and every reload comes back
  in the defaults. A `store: "keep"` playtest whose relaunch reads the
  setting back proves it (§4; `templates/ui-kit/tests/settings-saved.og.ts`).
- Save on meaningful moments (a new best, a level done, settings closed),
  not every tick: writes are rate limited.
- Never store anything the game can recompute, and never a secret: the
  store belongs to the player, not to the game.
- The engine exposes no identity (no player id, name or account): a game
  that needs a display name asks for it (§5).

The settings read, as the UI kit does it (`followSettings` in
`templates/ui-kit/src/main.ts`, called by its system every tick):

```ts
if (time.tick === 0) {
  d.settingsRequest = og.platform.loadSettings(world);   // og.settings, answered later
} else if (d.settingsRequest !== 0) {
  const answer = og.platform.result(world, d.settingsRequest);
  if (answer !== null) {
    d.settingsRequest = 0;
    const saved = answer.ok ? (answer.data as { value?: unknown } | null)?.value : null;
    if (saved && typeof saved === "object") takeSettings(world, d, saved as Partial<GameSettings>, true);  // volumes, language, contrast, motion
  }
}
```

## 4. Test it

- `dump_platform[:file]` lists every call with its method, arguments, tick,
  state and answer, and what answers them: assert on it in a playtest
  (`templates/archetypes/runner/tests/records.og.ts`; a stale write read
  again and merged: `templates/archetypes/idle/tests/offline.og.ts` and
  `templates/archetypes/roguelike/tests/win.og.ts`).
- `og run --platform-store save.json` keeps the local store in a file, so
  a second run sees the first's save; without it the store is in memory
  for the run. `og run --clock <UTC ms>` fixes the host clock at tick 0
  (it then moves with the ticks); a playtest runs at 2026-01-01 00:00 UTC
  unless its `clock` says otherwise.
- A relaunch across processes: a playtest with `store: "keep"` keeps one
  local store (`<out>/.store/platform.json`, a dot name no step's output
  may take, so `dump_platform:platform.json` is safe) for its run and its `then` steps (`{ commands, clock }`,
  each a fresh `og run`), and `check` reads each step's run in
  `run.then[n]` (`templates/archetypes/idle/tests/away.og.ts`: saved at
  `clock`, launched two hours later). A command that relaunches in the
  same process stays for tests that need no second process.
- In the browser, `packages/engine/tests/web/player.ts` shows the Player
  side: a framed game's first calls wait for the Player's `hello`, then go
  as protocol 2 `call`s (`store.get`, `store.set`, `leaderboard.around`,
  `point.award`, `ai.mark`, as the hello lists them; `docs/web-export.md`
  §Platform services has the mapping).
- The same file then opens the build as a top-level page with a stand-in
  play origin: the same calls go to `/__player/v2/<game>/<release>/call`
  with the answers unchanged, so a game behaves the same framed or not
  (`docs/web-export.md` §The top-level page). Nothing in game code changes
  between the two, and nothing in it can tell them apart.

## 5. When a method is missing: ask

The engine maps what the Player's protocol 2 offers and answers the rest
`unavailable`. Every ask the engine made (the engine repository's
`docs/platform-asks.md`: the keyed store with per-key revisions,
`leaderboard.around`, the best on submit, `point.award`, `ai.mark`,
capabilities for a verification run, telemetry names) landed on
2026-10-19; an older Player answers what its hello lists. One is open:
the Player's `hello` and answers carrying its `time` (until then the web
host's answers carry the browser's clock and `info(world).clock` says
`browser`). When a game
needs something none of them gives, do not work around it with a server of
your own or by reaching the page: write the ask (the method, its arguments,
its answer, why a game needs it) for the platform owners and keep the
feature behind `info(world)` until it lands.
