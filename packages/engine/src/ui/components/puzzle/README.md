# puzzle: built-in components

The level path, the in-level goals and moves, the booster tray and the
level summary of a level-based puzzle game (match-three, sliding, physics).
They draw what the game tells them and report intents with stable row
keys. They never compute scores, stars, moves, which levels are open,
whether a booster applies, or what comes next. The summary composes
`StarBar`.

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `PuzzleBoosters` | `puzzle/boosters` | The booster tray. |
| `PuzzleGoals` | `puzzle/goals` | The in-level HUD. |
| `PuzzleLevelPath` | `puzzle/level-path` | A scrolling level path over the game's own map art, which the component leaves at full brightness. |
| `PuzzleSummary` | `puzzle/summary` | The end of a level. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only puzzle/` draws every member's fixtures under every pack.
