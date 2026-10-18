# Tile Swap — the match3 archetype

Swap two neighbouring gems to make a line of three or more of one colour.
Lines clear, the gems above fall, new gems drop in, and any line the fall
makes clears again as a cascade worth more. Reach the target score before
the moves run out.

三消：交换相邻的两块方块，连成三个或更多同色即可消除；上方方块落下、新方块补入，下落时连成的线会再次消除（连锁得分更高）。在步数用完前达到目标分数即胜利。

A complete Origin Engine project: `og run .`, `og test .`, `og build .`.
It replaces the platform's Godot Discover match3 (Lantern Swap) and keeps
its feel (one thumb, a swap that makes nothing is free, cascades as the
reward, a move budget instead of a clock), not its code, art or lanterns.

It is also the engine's 2D and game-feel showcase for a puzzle game:

- **Sprites and flipbooks.** Every gem is an `og.Sprite` playing an
  `og.Flipbook` clip from one sheet (`assets/sprites/gems.ogt`): an idle
  loop that holds 2.4 s and then glints, each gem seeked to its own moment,
  and a `once` burst when it clears.
- **Nine-slice and a tilemap.** The board's backing is a nine-sliced
  `og.Sprite` panel; the checker under the gems is a hand-written 7 × 7
  tilemap (`assets/sprites/board.ogt` over the `cells.ogt` tileset).
- **Tweens drive the phases.** A swap slides with `cubicInOut` (a refused
  one goes there and back as one yoyo), cleared gems squash to nothing and
  fade, gems fall with `bounceOut`, column by column, and squash as they
  land; new rounds pop in with `backOut`. Each phase waits for its tween
  group's `og.TweenCompleted` (by tag), not for a tick count.
- **Feel.** Every cleared group's score floats up from it
  (`og.render.floatingText`, a gold crit on a cascade), the gems flash
  before they burst, four or more in a group punch the camera in, a cascade
  shakes it by its depth, a third cascade holds time for 3 ticks
  (`og.time.hitStop`), and the selection frame breathes on a looping tween.

```sh
og run . --headless --commands "0.5:touch_move:640,700;0.5:touch_start;0.55:touch_end;1:screenshot:board.png;1.1:quit"
og test .
og build .
```

## Rules

- **Title.** The first tap (or Space/Enter) deals a round: `Tuning.moves`
  moves, score 0. That tap does not also select.
- **Swapping.** Tap a tile, then a neighbour; or press a tile and drag at
  least 0.45 of a cell towards a neighbour. Tapping the selected tile again
  deselects it; tapping a tile that is not a neighbour selects it instead.
- **Lines.** After a swap, every horizontal or vertical run of three or more
  of one colour is marked. No line: the tiles swap back and no move is
  spent. Lines: one move is spent and the marked tiles fold away over
  `Tuning.clearTicks`.
- **Score.** Each cleared tile scores `Tuning.tileScore × chain`, where the
  chain is 1 for the swap's own lines and rises by one for every cascade.
- **Fall and refill.** Survivors fall to the bottom of their columns at
  `Tuning.fallSpeed` cells/s; the cleared tiles come back above the board
  with colours from the dice and fall in. Lines the fall makes clear again.
- **End.** When the board settles: score ≥ `Tuning.target` wins; no moves
  left loses; a board where no swap makes a line is dealt again (colours
  only, `Match.shuffles` counts it). The end card holds 40 ticks, then a tap
  deals the next round.

Deals never contain a ready line and always contain a move. Every colour
draw comes from `og.Rng`, so a seed (`project.ogt`, `og run --seed`) gives
the same deals, refills and reshuffles on every host.

## The world

| Row | Count | Name | Drawn as |
|---|---|---|---|
| backdrop | 1 | `backdrop` | a soft lift of light behind the board (layer −30); around it the camera's background is the brand ink `#14161b` |
| board | 1 | `board` (from `assets/scenes/main.ogt`) | the nine-sliced panel (layer −20) |
| tiles | 49 | `tile-0` … `tile-48` (pool rows: a tile's cell is `Piece.col/row`, not its name) | gems: `og.Sprite` + `og.Flipbook` on the gem sheet (layer 0) |
| selection frame | 1 | `cursor` | a rounded frame (layer 1) |

Those 52 rows have `og.Transform`, `og.Sprite`, `og.Flipbook` (sheet 0, so
unused, except on the gems) and:

| Component | On | What it holds |
|---|---|---|
| `Piece` | every row | `kind`, a tile's `col`/`row` (0,0 is the bottom left), `colour` (−1 empty), `marked` |
| `Match` | every row; **the board's is the authority** | `phase` (title, idle, swap, swapBack, clear, fall, won, lost), `phaseTicks`, `score`, `best`, `movesLeft`, `chain`, `maxChain`, `cleared`, `lastCleared`, `rounds`, `shuffles`, `selected`, `swapA`/`swapB`, the drag (`dragFrom`, `pressX/Y`), the view (`viewHeight`, `viewCentreY`), the pointer mapping Pick works out each tick (`pxPerM`, `screenX0/Y0`), `wait` (the tag of the tween group the phase waits for) and `font` (for the floating scores) |
| `Tuning` | every row; **the board's is the authority** | the tunables below |
| `Hud` | the board only | the font handle and the view's top |

One more entity, `cells`, carries only the `og.Tilemap` of the checker
(layer −10), placed by its top-left corner at the board's; no system sees
it.

`Match` and `Tuning` ride on every row for the same reason as in the
runner: a system sees one query and cannot reach another entity or a game
resource yet, and the swap, clear and fall need the round's state together
with the tiles.

`src/board.ts` is pure logic over a colour grid (lines, "is there a move",
the deal, the board text format); `src/layout.ts` is the geometry. Neither
touches the world, so systems, commands and playtests share them.

## Systems (in tick order)

| System | Reads | Writes | Job |
|---|---|---|---|
| `Flow` | `og.Input`, `Tuning` | `og.Rng`, `Match`, `Piece`, `og.Transform` | the title and end cards; a tap deals a round, the gems popping in on staggered tweens |
| `Pick` | `og.Input`, `Piece` | `Match` | pointer → cell; tap-tap and drag start a swap; a `swap` command's request |
| `Resolve` | `Tuning`, `og.TweenCompleted` | `og.Rng`, `Match`, `Piece`, `og.Transform` | starts the swap (or the refused yoyo), clear and fall tween groups and waits for each; marks, scores, collapses and refills; floating scores, flash, punch, shake and hit stop; cascade, win, lose, reshuffle |
| `Animate` | `Match`, `Piece`, `Tuning` | `og.Sprite`, `og.Flipbook`, `og.Transform` | each gem's clip (idle or burst), visibility and fade-in at the top edge, the selected gem's pulse, the frame's cell; gems hidden on the title and dimmed under the end cards |
| `Hud` | `Match`, `Hud`, `Tuning` | — | score, moves, target, best; cascade callout; title, win and lose cards |

## Tunables (`Tuning`, on the board)

| Field | Default | Range | Unit | What it changes |
|---|---|---|---|---|
| `moves` | 20 | 1–60 | moves | the budget of a round |
| `target` | 1500 | 10–10000 | points | the score that wins |
| `colours` | 5 | 3–6 | colours | fewer colours, more lines and cascades |
| `tileScore` | 10 | 1–100 | points | per cleared tile, times the chain |
| `swapTicks` | 8 | 1–30 | ticks | the swap tween's length (a refused swap takes it twice) |
| `clearTicks` | 12 | 1–40 | ticks | the clear tween's length (squash and fade) |
| `fallSpeed` | 14 | 2–60 | cells/s | a fall tween lasts 0.12 s plus its drop at this speed |

Written in `assets/scenes/main.ogt` (the board's `Tuning` block), live with
`set:board.Tuning.<field>=<value>` or Tune.

## What a Maker changes first

1. **Difficulty**: `moves`, `target` and `colours` in the scene's `Tuning`.
2. **Scoring**: the score line in `Resolve`'s clear step (a bonus for lines
   of four or five, for example).
3. **The opening**: deal an authored first board with the `deal` command's
   text format from `start`, so the first move teaches.
4. **Look**: the gem cuts and colours, the glint and the burst frames in
   `tools/make-assets.py` (six cuts, so colour is never the only cue), the
   idle hold in the sheet it writes, then
   `python3 tools/make-assets.py && og cook .`.
5. **Feel**: the eases, durations and staggers in `Resolve` (`startClear`,
   `collapse`), the punch, shake and hit-stop thresholds in `startClear`.
6. **Size**: `SIZE` in `src/layout.ts` (the pools follow; regenerate the
   tilemap in `tools/make-assets.py`'s `board_map`).

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `deal:<rows>` | lays out the board: 7 lines of 7 colour digits joined by `/`, the top row first; needs an idle board |
| `swap:<col>,<row>:<col>,<row>` | swaps two neighbouring cells on the next tick, as two taps would |

`facts` reports `phase`, `score`, `movesLeft`, `cleared`, `maxChain`,
`best`, `rounds` and the pointer mapping. A cell's CSS pixel is
`(x0 + x·pxPerM, y0 − y·pxPerM)` with `(x, y)` from `cellCentre` in
`src/layout.ts`; at 1280×720 that is 60 px per metre around (640, 390).

## Playtests (`og test .`)

| File | Proves |
|---|---|
| `tests/swap.og.ts` | on an authored board, real taps: a swap that makes nothing goes back for free; the swap that makes a line clears it, spends a move, scores, and the fall cascades (chain 2, both lines scored); every system matches its rows |
| `tests/drag.og.ts` | a drag shorter than half a cell does nothing; past it, the swap happens and clears |
| `tests/endings.og.ts` | target 30: the first line wins; one move and an unreachable target: that move loses; a tap after the card deals the next round with the tuned moves; best kept |
| `tests/frames.og.ts` | at 1280×720 the dealt board, the swap's line mid-clear (burst, squash, `+30`) and the cascade mid-clear (gold crit `+60`, shake) match `references/<tier>/` (og-tests' tolerance, `tests/frames.ts`); away from the board the frame is the brand ink |
| `tests/replay.og.ts` | `runs: 2`: a refused swap and a cascading swap, stopped mid-fall, end on the same world hash in two fresh processes |

Each one fails when its behaviour is broken (checked by charging refused
swaps, removing cascades, dropping the drag threshold, shrinking the gems
for the frames and a wall-clock fall delay for the replay). Record a tier's
reference frames with `OG_FX_CAPTURE=1 og test .` (`references/metal/` is
recorded; other tiers compare with it until they have their own).

## Assets

`python3 tools/make-assets.py` writes them all, byte for byte
(`LICENSES.md`): `assets/sprites/gems.png` + `gems.ogt` (six rows of eight
80 px frames: idle, three glint frames, four burst frames; clip `2c` is
colour `c`'s idle loop, `2c + 1` its burst), `panel.png` (96 px, 32 px
nine-slice insets), `backdrop.png`, `ring.png`, `cells.png` +
`cells.ogt` + `board.ogt` (the tileset and the map, `kind pixel`), the
sounds and the font subset. Then `og cook .`.

## Known limits (engine gaps, not design)

- No audio playback yet: the sounds are cooked, not played.
- Systems cannot read other entities, game resources or the camera, so
  `Match` and `Tuning` ride on every row, and `Pick` maps the pointer from
  `og.render.capabilities` (device pixels) assuming a device pixel ratio of
  1: on a high-DPR phone taps land at the wrong cell until a system can
  read the surface in CSS pixels.
- The camera is set once in `start`; on the web the renderer is not
  attached then, so the view assumes 16:9 and a portrait phone clips the
  board's sides.
