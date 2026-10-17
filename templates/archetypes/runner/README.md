# Rooftop Run — the runner archetype (2D)

A courier runs the night roofs on her own; the player decides every jump. A
tap is a hop that clears a crate, a hold is a full jump that clears a fence,
and coins hang where the arcs go. The run speeds up until a crash ends it;
the score is the distance plus the coins.

屋顶快跑：信使在夜晚的屋顶上自己奔跑，每一次起跳由你决定。点按小跳越过木箱，按住高跳越过路障，金币挂在跳跃的弧线上。速度越来越快，直到撞上障碍。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests. It
replaces the platform's Godot Discover runner (Skyline Post) and keeps its
feel (one thumb, tap for a hop, hold for a jump, pace that ramps), not its
code, art or route.

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.5:key_down:Space;0.55:key_up:Space;3:screenshot:play.png;3.1:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og build .                   # dist/: the play page for the browser
```

## Rules

- **Title.** The city scrolls slowly; the first press (Space, ArrowUp, W, a
  click or a touch) starts the run. That press does not also jump.
- **Running.** The courier stays at x = −5 and the world scrolls left at the
  run speed, which starts at `Tuning.speed` and ramps by `Tuning.speedRamp`
  up to `Tuning.speedMax`. Every `Tuning.spawnInterval` seconds (×0.85–1.25
  from the dice) a pattern enters from the right: a crate, a crate under an
  arc of coins, a line of coins, and after 60 m a fence or a fence under a
  high coin, and after 150 m two crates in a row.
- **Jumping.** A press takes off at `Tuning.jumpSpeed`; holding keeps the
  rise, releasing cuts the upward speed to `Tuning.jumpCut` of itself. A
  release before 6 ticks still rises 6 ticks: the shortest hop (1.1 m)
  clears a crate (0.7 m), only a full jump (1.9 m) clears a fence (1.55 m).
  A press up to 6 ticks before landing is kept and jumps on landing.
- **Score.** `floor(distance) + coins × Tuning.coinValue`. Touching a crate or
  a fence ends the round; the best score is kept for the session.
- **Game over.** The card holds for 30 ticks, then a press starts the next
  round at once.

Everything is deterministic from the project seed (`project.ogt`, or
`og run --seed`): the patterns come from `og.Rng`, the clock is the fixed
tick, and the arithmetic is plain `+ − × ÷` (the coins' turn is a triangle
wave of the tick), so the same script gives the same world hash on the
native and the web host.

## The world

Every drawn thing is an entity with `og.Transform` (position; scale is the
sprite's size), `og.Mesh` (a quad and a material from
`assets/models/sprites.glb`), and the game's three components below. The
roof tiles, buildings and clouds wrap; the crates, fences and coins are
pools (4, 3 and 16) parked at x = −1000 until a pattern puts them in play.

| Component | On | What it holds |
|---|---|---|
| `Actor` | every row | `kind` (sky, moon, cloud, building, ground, runner, crate, fence, coin), `active`, the hit box `w`×`h`, `parallax` (the share of the run speed a row scrolls at), `wrap` (the span scenery jumps right by), `vy`, and up to five looks (materials) |
| `Run` | every row; **the runner's is the authority** | `phase` (0 title, 1 running, 2 over), `phaseTicks`, `speed`, `distance`, `coins`, `score`, `best`, `rounds`, `spawnIn`, and the jump's `grounded`, `buffer`, `hold`, `rising` |
| `Tuning` | every row; **the runner's is the authority** | the tunables below |
| `Hud` | the runner only | the font handle and the visible area the HUD lays out in |

Why `Run` and `Tuning` sit on every row: a system sees exactly one query,
the AND of what it declares, and inside a system the engine reaches no other
entity and no game resource yet. So the systems that move obstacles and end
the round must find the run's state on the same rows as the obstacles. The
other rows' copies keep their defaults and nothing reads them. When the
engine lets a system read another entity or a resource, `Run` and `Tuning`
become a resource or the runner's alone and the systems keep their shape.

## Systems (in tick order)

| System | Reads | Writes | Job |
|---|---|---|---|
| `Director` | `og.Input`, `Tuning` | `Run`, `Actor`, `og.Transform`, `og.Mesh` | phases; the speed ramp, distance and score; a new round (pools retired) |
| `Jump` | `og.Input`, `Tuning` | `Run`, `Actor`, `og.Transform` | buffered take-off, the hold and the cut, gravity, landing |
| `Scroll` | `Run` | `Actor`, `og.Transform`, `og.Mesh` | everything left at speed × parallax; scenery wraps over 30 m; pooled rows retire past x = −13 |
| `Spawn` | `Tuning` | `og.Rng`, `Run`, `Actor`, `og.Transform`, `og.Mesh` | the next pattern at x = 13 from the dice |
| `Collide` | `og.Transform`, `Tuning` | `Run`, `Actor`, `og.Mesh` | coins collected, crashes, the best score |
| `Animate` | `Run`, `Actor` | `og.Mesh`, `og.Transform` | the runner's pose (idle, two run frames, jump, crash); the coins' turn |
| `Hud` | `Run`, `Hud`, `Tuning` | — | the score line, the title card and the game-over card (`og.render.text`) |

Spawning and retiring at fixed x (beyond the widest view) keeps the play
surface's shape out of the simulation: a phone and a desktop see the same
obstacles at the same ticks.

## Tunables (`Tuning`, on the runner)

| Field | Default | Range | Unit | What it changes |
|---|---|---|---|---|
| `speed` | 6 | 3–12 | m/s | the pace a round starts at |
| `speedRamp` | 0.12 | 0–1 | m/s² | how fast the pace rises (the cap after 50 s) |
| `speedMax` | 12 | 4–20 | m/s | the top pace |
| `gravity` | 36 | 10–80 | m/s² | how heavy jumps feel; a full jump peaks at jumpSpeed² / (2 gravity) |
| `jumpSpeed` | 12 | 6–20 | m/s | take-off speed |
| `jumpCut` | 0.5 | 0.1–1 | × | what a release keeps of the rise: the hop's height |
| `spawnInterval` | 1.3 | 0.5–60 | s | the gap between patterns |
| `coinValue` | 10 | 0–100 | points | what a coin adds |

They are written in `assets/scenes/main.ogt` (the runner's `Tuning` block)
and live with `set:runner.Tuning.<field>=<value>` from the command channel
or Tune. The hop/jump numbers were measured against the obstacle heights:
with these defaults a hop clears a crate if it leaves 7–17 ticks before the
crate arrives, a full jump clears a fence if it leaves 13–24 ticks before.

## What a Maker changes first

1. **Feel**: `jumpSpeed`, `gravity` and `jumpCut` in the scene's `Tuning`.
   Keep the hop below the fence and above the crate (`tests/jump.og.ts`
   checks exactly that).
2. **Pace**: `speed`, `speedRamp`, `speedMax`, `spawnInterval`.
3. **What comes**: `placePattern` and the `top` gate in `Spawn`
   (`src/systems.ts`): add a pattern, change when fences appear.
4. **Look**: the shapes and colours in `tools/make-assets.py`, then
   `python3 tools/make-assets.py && og cook .`. A new sprite goes at the end
   of `SPRITES` there and in `src/art.ts`.
5. **Words**: the HUD strings in `Hud`. A CJK character must be in `CHARS`
   of `tools/make-assets.py` (it writes `assets/fonts/ui.ttf.ogt`).

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `place:<crate\|fence\|coin>:<x>[:<y>]` | puts a pooled obstacle or coin in play at x (y defaults to its resting height) |
| core actions | `key_down:Space`/`key_up:Space` (or `touch_start`/`touch_end`) jump; `set:runner.Tuning.<field>=<v>` tunes; `dump_entity:runner` shows `Run` |

`facts` (the probe's `state()`) reports `phase`, `score`, `coins`,
`distance`, `speed`, `best` and `rounds`.

## Playtests (`og test .`)

| File | Proves |
|---|---|
| `tests/flow.og.ts` | title → a press starts → an untouched crate ends the round → the card holds → a press starts round two from zero, best kept; every system matches its rows |
| `tests/coins.og.ts` | a coin in reach is collected and adds `coinValue`; one hung at 3.2 m passes over a standing runner |
| `tests/jump.og.ts` | a full jump clears a crate and a fence; a hop clears a crate and crashes into a fence |
| `tests/tune.og.ts` | `set` writes `Tuning`: the start speed and the jump height follow |

Each one fails when the behaviour it names is broken (checked by breaking
coin collection, the hop's minimum hold and the card's hold).

## Assets

`tools/make-assets.py` regenerates every asset byte for byte (sprites,
sounds, the font subset); `LICENSES.md` lists them. The sounds are cooked
but not played yet: the engine's audio module is not in the API this
archetype was built on.

## Known limits (engine gaps, not design)

- No sprite or immediate-mode quad API: sprites are glTF quads with
  emissive, alpha-masked materials, and colour is lit out (no environment
  light, a dark sun, tone mapping off).
- No UI module yet: the title and game-over cards are world-space text, and
  there is no Start button for `touch_move_to`.
- Systems cannot read other entities or game resources (hence `Run` and
  `Tuning` on every row), cannot spawn (hence the pools) and cannot move the
  camera (hence the fixed camera and the scrolling world).
- The view's size is read once in `start`; on the web the renderer is not
  attached yet then, so the layout assumes 16:9 and a portrait phone sees a
  narrow slice of the roofs.
