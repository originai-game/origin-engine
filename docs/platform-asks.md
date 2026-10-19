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

Nothing here carries a credential: every method is a name and arguments,
and the Player owns the session, as protocol 2 already requires.

## The top-level page (player-page, 2026-10-19)

| # | Ask | Why | Before | State |
|---|---|---|---|---|
| 9 | Player protocol 2 over same-origin HTTP for a top-level engine page: `POST /__player/v2/<game>/<release>/{hello,call,report}` on the game's own origin, the hand-off code redeemed from the hello's body, the framed Player's answers and errors, the session reports filed as the Player filed them; and Play (the game host's `/`, `/g/:slug`) opening a release whose entry says `<meta name="og-player" content="frame page">` as its own top-level document (307 to the release entry), the Stage page likewise for a Stage build (the creator overlay injected into the engine's document) | design §4: the shell is the play page and the brand surface; a frame costs the viewport, gestures, audio unlocking and the PWA on phones | every engine page is framed by the Player page or the Stage page | The contract is origingame `docs/player-protocol.md` §The top-level engine page (platform commit `4f5d40850`). The shell speaks it (`shell/transport.ts`, `docs/web-export.md` §The top-level page). The platform side is built on its branch `player-page`; on the platform it also needs the game-host ingress to admit `POST` with `Authorization` (never `Cookie`) on `/__player/v2/` (today those hosts take only GET and HEAD and strip both), which is a production topology change the platform owns. Until the platform's pin moves past this round, every engine release stays framed (the pinned `cf3cb0b` shell has no `og-player` meta). |

Nothing in this ask carries a credential to game code beyond what a game
on its own origin can already read: the game-scoped player credential the
page holds (design §4: the page has nothing beyond the game's own scope to
steal; the host ABI is the only route to platform APIs).
