# Platform asks

What the engine's `og.platform` (IDL module `platform`, docs/design.md §2.8,
E6) needs from the OriginGame platform (`origingame/origingame`, read-only
from this repository) that Player protocol 2 does not give today. The
engine works without each one: the shell's bridge
(`packages/engine/shell/platform.ts`) maps what it can onto protocol 2 and
answers the rest `unavailable`, falling back to the browser's storage for
the store. Each ask names the shape the engine would use, so the bridge's
mapping shrinks to a rename when it lands. Recorded 2026-10-19; all eight
landed in the platform's Player protocol 2 the same day (additive within
protocol 2, origingame `docs/player-protocol.md` §Versions), and the shell
asks for each where the Player's hello lists it (`docs/web-export.md`
§Platform services; E7).

| # | Ask | Why | Before (the shell's stand-in) | Landed |
|---|---|---|---|---|
| 1 | `save.get` answers `{ data, revision }` | A conditional `save.set { data, revision }` needs the revision the data was read at; protocol 2's `save.get` returns the data alone, so a write after another device's write cannot be refused (`conflict`) by the Player. | The shell keeps a revision per key inside the save and checks it itself: two tabs of one browser are caught, two devices racing between a read and a write are not. | `save.read` answers `{ data, revision }`; `save.set { data, revision }` refuses a stale write with `conflict` and `error.revision` (`save.get` still answers the data alone). The shell reads and writes the save at its revision where the Player has no keyed store. |
| 2 | A keyed store: `store.get { key }`, `store.set { key, value, revision? }`, `store.list` (per-key revision, 64 KiB per value) | Games save several independent things (settings, progress, unlocks); one blob makes every write a read-modify-write of everything and every conflict global. | The store is the save's `store` object; each `store.set` is a `save.get` plus a `save.set` of the whole save. | `store.get`, `store.set { key, value, revision? }` (revision 0: only a key never written; none: the last write wins), `store.list`; 64 KiB a value, 64 keys and 1 MiB per player and game, 30 writes a minute. The shell maps `storeSet`'s revision, `create` and last-write-wins onto it. |
| 3 | `leaderboard.around { board, n }` → `{ board, sort, entries: [{ rank, name, score }], me }` | "Around me" is the board most players see (they are rarely in the top 10). | The shell asks `leaderboard.top { n: 100 }` and cuts around `me`; a player below 100th gets the top entries. | `leaderboard.around { board, n }` → `{ board, sort, entries: [{ rank, name, score, me }], me }`. |
| 4 | `leaderboard.submit` answers the board's `best` for the player | A game shows "new best" from the answer instead of keeping its own copy. | The engine answers `best: null` under a Player (the local platform fills it). | `leaderboard.submit` answers `{ ok, rank, board, best }`. |
| 5 | `point.award { reason, amount }` → `{ granted, balance }` (Origin Point) | `og.platform.point`: the game asks, the platform decides what it grants (rate limits, caps, fraud rules stay on the platform). | `unavailable` under a Player; the local platform keeps a local balance. | `point.award { reason, amount }` → `{ granted, balance }`; every answer is `{ granted: 0, balance: null }` until a release carries an approved point policy (origingame `docs/public-api.md` §15, an owner decision for Billing and Risk). |
| 6 | `ai.mark { kind, label, model? }` → `{ marked }` | `og.platform.aiMark`: disclosure of AI-generated content the game shows, for the platform's labelling and compliance records. | `unavailable` under a Player; the local platform records the marks. | `ai.mark { kind, label, model? }` → `{ marked: true }`, recorded on the release the Player framed. |
| 7 | `hello.capabilities` lists the new methods, and a verification run answers `store.*` against a throwaway store | A game reads capabilities instead of trying calls; a verification run should exercise the save path. | The shell derives its capabilities from `save.get`/`save.set`/`leaderboard.*`. | `hello.capabilities` lists what the Player answers; a verification run answers `save.*` and `store.*` from a throwaway store. |
| 8 | Telemetry names: accept `game.custom.<name>` with `.` and `_` in `<name>` (64 characters) and up to 16 scalar attributes | `og.platform.telemetry` validates exactly this before it posts the `event`. | Posted as `event` `game.custom.<name>`; the server's validation is the platform's. | `game.custom.<name>`: 1–64 of letters, digits, `.` and `_`, at most 16 scalar attributes. |

## Open

| # | Ask | Why | Until then (the engine's stand-in) |
|---|---|---|---|
| 9 | The Player's `hello` carries `time` (UTC ms, the Player's clock), and every answer carries `time` when the Player answered it | `og.platform.now` and `PlatformResult.time` (Phase 2, core round) are the host's clock handed to the game as host input, recorded in replays like a key press; an idle game credits the time away from a store answer's `time`. The Player's clock (one the platform can correct, the same for every device of a player) should be the one games read, not each browser's. | The engine's Worker (`packages/engine/src/web/worker.ts`) gives og.platform the browser's clock (`Date.now()` when the platform attaches and when each answer arrives) and says so: `og.platform.info(world).clock` is `browser`. An answer the page hands back with its own `time` is taken as is and `clock` reads `player`, so when this lands the shell's bridge (`packages/engine/shell/platform.ts`) only passes the field on. |

Nothing here carries a credential: every method is a name and arguments,
and the Player owns the session, as protocol 2 already requires.
