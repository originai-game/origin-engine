# Platform asks

What the engine's `og.platform` (IDL module `platform`, docs/design.md §2.8,
E6) needs from the OriginGame platform (`origingame/origingame`, read-only
from this repository) that Player protocol 2 does not give today. The
engine works without each one: the shell's bridge
(`packages/engine/shell/platform.ts`) maps what it can onto protocol 2 and
answers the rest `unavailable`, falling back to the browser's storage for
the store. Each ask names the shape the engine would use, so the bridge's
mapping shrinks to a rename when it lands. Recorded 2026-10-19.

| # | Ask | Why | Today |
|---|---|---|---|
| 1 | `save.get` answers `{ data, revision }` | A conditional `save.set { data, revision }` needs the revision the data was read at; protocol 2's `save.get` returns the data alone, so a write after another device's write cannot be refused (`conflict`) by the Player. | The shell keeps a revision per key inside the save and checks it itself: two tabs of one browser are caught, two devices racing between a read and a write are not. |
| 2 | A keyed store: `store.get { key }`, `store.set { key, value, revision? }`, `store.list` (per-key revision, 64 KiB per value) | Games save several independent things (settings, progress, unlocks); one blob makes every write a read-modify-write of everything and every conflict global. | The store is the save's `store` object; each `store.set` is a `save.get` plus a `save.set` of the whole save. |
| 3 | `leaderboard.around { board, n }` → `{ board, sort, entries: [{ rank, name, score }], me }` | "Around me" is the board most players see (they are rarely in the top 10). | The shell asks `leaderboard.top { n: 100 }` and cuts around `me`; a player below 100th gets the top entries. |
| 4 | `leaderboard.submit` answers the board's `best` for the player | A game shows "new best" from the answer instead of keeping its own copy. | The engine answers `best: null` under a Player (the local platform fills it). |
| 5 | `point.award { reason, amount }` → `{ granted, balance }` (Origin Point) | `og.platform.point`: the game asks, the platform decides what it grants (rate limits, caps, fraud rules stay on the platform). | `unavailable` under a Player; the local platform keeps a local balance. |
| 6 | `ai.mark { kind, label, model? }` → `{ marked }` | `og.platform.aiMark`: disclosure of AI-generated content the game shows, for the platform's labelling and compliance records. | `unavailable` under a Player; the local platform records the marks. |
| 7 | `hello.capabilities` lists the new methods, and a verification run answers `store.*` against a throwaway store | A game reads capabilities instead of trying calls; a verification run should exercise the save path. | The shell derives its capabilities from `save.get`/`save.set`/`leaderboard.*`. |
| 8 | Telemetry names: accept `game.custom.<name>` with `.` and `_` in `<name>` (64 characters) and up to 16 scalar attributes | `og.platform.telemetry` validates exactly this before it posts the `event`. | Posted as `event` `game.custom.<name>`; the server's validation is the platform's. |

Nothing here carries a credential: every method is a name and arguments,
and the Player owns the session, as protocol 2 already requires.
