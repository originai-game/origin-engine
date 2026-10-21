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
never branches on where it runs. The API is in `reference/platform.md`;
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
  answered. `og.platform.now(world)` is the host's clock at the last
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
| Progress, unlocks, a best score | `storeGet(world, key)`, `storeSet(world, { key, value, revision })`; the whole world: `world.snapshot()` text through `saveSlot(world, { slot, text, meta })` (as many keys as it needs, the manifest last, one request) and `loadSlot(world, { slot, metaOnly })`, back with `world.restore` (og-game §17) | one key per independent thing; values ≤ 64 KiB, at most 64 keys and 1 MiB per player and game; pass the revision you read to refuse a stale write (`conflict`, whose `error.revision` is where the key is now: read again and merge), `create: true` to write only a key never written, no revision for "the last write wins" |
| Volume, controls, language | `loadSettings(world)`, `saveSettings(world, settings)` | the key `og.settings`, last save wins, local fallback when offline |
| A leaderboard | `leaderboardSubmit(world, { board, score })`, `leaderboardTop`, `leaderboardAround` | the board keeps the player's best; show "around me" as well as the top |
| Analytics | `telemetry(world, "round.over", { score, coins })` | one-way; a name of 1–64 letters, digits, `.` and `_` (the Player files it as `game.custom.<name>`), at most 16 scalar attributes; never personal data |
| Origin Point | `point(world, { reason, amount })` | the platform decides what it grants (today nothing under a Player: `{ granted: 0, balance: null }` until an approved point policy; show nothing that depends on it) |
| AI-generated content shown | `aiMark(world, { kind, label })` | disclosure, not permission |

`og.platform.info(world)` says what answers here (`player`, `local` or
`none`) and the methods it answers (`capabilities`): check it rather than
trying.

## 3. Rules

- Read saves on the first tick and show defaults until they answer;
  never block the title on the platform.
- Save on meaningful moments (a new best, a level done, settings closed),
  not every tick: writes are rate limited.
- Never store anything the game can recompute, and never a secret: the
  store belongs to the player, not to the game.
- The engine exposes no identity (no player id, name or account): a game
  that needs a display name asks for it (§5).

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
