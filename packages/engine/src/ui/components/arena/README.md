# arena: built-in components

Character selection, the ban and pick draft, the versus reveal, the team
scoreboard, the round HUD, the lobby, ready checks, map voting and spectator
controls. They draw what the game tells them and report intents with stable
row keys. They never compute scores, kills, ranks, round outcomes, turn
order, timers, pick legality or readiness: every figure arrives from the
game, every decision is the game's. Arena owns team and match scores; the
shooter kit does not duplicate them. The draft and the round HUD compose
`HudTimer`.

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `ArenaDraft` | `arena/draft` | A ban and pick draft between two teams on a smoked cut card. |
| `ArenaLobby` | `arena/lobby` | The pre-match lobby. |
| `ArenaReady` | `arena/ready` | The ready check on a smoked cut card. |
| `ArenaRoster` | `arena/roster` | Character selection. |
| `ArenaRoundHud` | `arena/round-hud` | The round HUD along the top of a match. |
| `ArenaScoreboard` | `arena/scoreboard` | The team scoreboard over the match, bare. |
| `ArenaSpectator` | `arena/spectator` | The spectator's plate over the match. |
| `ArenaVersus` | `arena/versus` | The match reveal. |
| `ArenaVote` | `arena/vote` | The vote on a smoked cut card. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only arena/` draws every member's fixtures under every pack.
