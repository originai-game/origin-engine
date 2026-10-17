# M6 games: the runner and match3 archetypes, test run

Engine: the code of 49ed935 (unchanged at 5cca9c1), exported with `git archive` and
built with `packages/engine/scripts/build.sh` (CARGO_TARGET_DIR=target/agent-games),
because the shared `packages/engine/native` addon was mid-rebuild by the Tools
agent (its IDL had 19 symbols the addon lacked). `og` = target/agent-games/debug/og
with `OG_ENGINE_PACKAGE=<export>/packages/engine`. macOS, M4 Pro, Metal.

## og test

```
$ og test templates/archetypes/runner
PASS coins (7 checks, 264 ms): a coin in reach is collected and scores; one out of reach is not
PASS flow (25 checks, 267 ms): title, run, crash on a crate, game over, run again
PASS jump (6 checks, 351 ms): a full jump clears a crate and a fence; a hop clears a crate, not a fence
PASS tune (7 checks, 206 ms): set writes Tuning: start speed and gravity change the run
og test: 4 suite(s) passed
$ og test templates/archetypes/match3
PASS drag (8 checks, 179 ms): a drag onto a neighbour swaps like two taps
PASS endings (9 checks, 274 ms): reaching the target wins, running out of moves loses, a press deals again
PASS swap (23 checks, 246 ms): a swap that makes a line clears, scores and cascades; one that makes nothing is free
og test: 3 suite(s) passed
```

TypeScript: `tsc --noEmit` (typescript 5.9.3, the archetype's tsconfig with
`@origingame/engine` mapped to `packages/engine/types/index.d.ts`): clean for both.

## Tests checked against deliberate defects (each restored afterwards)

| Defect | Caught by |
|---|---|
| runner: coins never collected | coins (5 failures) |
| runner: MIN_HOLD 60 (every tap a full jump) | jump: "the hop did not clear fence 2" |
| runner: no game-over hold | flow: "a press during the card's hold does not restart" |
| match3: a refused swap costs a move | swap: "the refused swap cost no move" |
| match3: no cascades after the fall | swap: chain 1, 3 tiles, score 30 |
| match3: drag swaps without the half-cell threshold | drag (4 failures), swap |

## Frames (read)

- `games-runner-title.png` (tick 18): night skyline, banded dusk sky, moon right, the
  courier idle at x=-5 on the teal roof; "ROOFTOP RUN" gold, 屋顶快跑, the start prompts.
- `games-runner-play.png` (tick 120): HUD SCORE/COINS left, BEST right, the hop/jump hint;
  the courier mid-stride, scenery scrolled.
- `games-runner-jump.png` (tick 114 of tests/jump): the courier in the jump pose above a
  crate approaching from the right.
- `games-runner-over.png` (tick 240): GAME OVER card in coral, score and best, the
  restart prompt; the courier in the crash pose against the crate.
- `games-match3-title.png`: TILE SWAP, 三消, an empty 7x7 board of cells, the start
  prompt and the rules line.
- `games-match3-play.png`: the authored board (CASCADE_BOARD) dealt, score 0, 20 moves.
- `games-match3-clearing.png`: the three bottom-left tiles of the line shrinking
  mid-clear, MOVES 19.
- `games-match3-won.png` / `games-match3-lost.png`: YOU WIN (gold, 胜利) and
  OUT OF MOVES (coral, 失败) cards over the emptied board, score 90 best 90.
- `games-runner-browser.png` / `games-match3-browser.png` (web, tick 600): the same
  scenes from the WebGPU host in Chrome; runner in round two after a crash
  (score 12, best 81), match3 after one cleared line (score 30, moves 19).
