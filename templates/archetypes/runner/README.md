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

## What it demonstrates, and the skills it exercises

A one-button 2D game end to end: og.PlatformerController with its designer
units in a `Tuning` resource and its CharacterEvents driving the take-off
stretch and landing squash (og-game §13; og-feel §5, §12), a scrolling world
of flipbooks and parallax tile layers (og-game §8; og-feel §9–§11), coins
with floating text, hit stop, shake and kick on a crash (og-feel §3, §4,
§7), every action heard through one `Voices` system (og-audio §1), the best
score and a leaderboard through og.platform's requests answered later
(og-platform-sdk §1–§4), and playtests for jumps, tuning, records and a
replay to one hash (og-build §1, §3). `og polish .` (2026-10-19): feel 99
(taste n/a: the HUD is screen-space text, not og.ui).


## Rules

- **Title.** The city scrolls slowly; the first press (Space, ArrowUp, W, a
  click or a touch) starts the run. That press does not also jump.
- **Running.** The courier stays at x = −5 and the world scrolls left at the
  run speed, which starts at `Tuning.speed` and ramps by `Tuning.speedRamp`
  up to `Tuning.speedMax`. Every `Tuning.spawnInterval` seconds (×0.85–1.25
  from the dice) a pattern enters from the right: a crate, a crate under an
  arc of coins, a line of coins, and after 60 m a fence or a fence under a
  high coin, and after 150 m two crates in a row.
- **Jumping.** The courier's body is an `og.PlatformerController` that
  never runs (the world scrolls past it) on a static roof collider: a press
  jumps to `Tuning.jumpHeight` in `Tuning.timeToApex`, hanging a little at
  the top (`apexGravity`) and coming down a little faster (`fallGravity`);
  releasing while rising multiplies gravity by `jumpCut`. A two-tick tap is
  the shortest hop (1.30 m) and clears a crate (0.7 m); only a full jump
  (2.12 m) clears a fence (1.55 m). A press up to 0.1 s before landing jumps
  on landing (the controller's buffer), and one 0.08 s after running off an
  edge still jumps (its coyote time).
- **Score.** `floor(distance) + coins × Tuning.coinValue`. Touching a crate or
  a fence ends the round; the best score is kept for the session.
- **Game over.** The card holds for 30 ticks, then a press starts the next
  round at once.

Everything is deterministic from the project seed (`project.ogt`, or
`og run --seed`): the patterns come from `og.Rng`, the clock is the fixed
tick, and the feel (tweens, flipbooks, hit stop, shake, floating text,
afterimages) is engine world state, so the same script gives the same world
hash on the native and the web host (`tests/replay.og.ts` runs it twice).

## The world

A night city in pixel art: every sheet is drawn texel by texel by
`tools/make-assets.py`, imported as `kind pixel` (lossless, sampled
nearest) and outlined in the brand ink `#14161b` so the silhouettes read
on the dark sky. Layers, back to front:

| Layer | What | Drawn by |
|---|---|---|
| −40 | the sky's colour bands | `og.Sprite` over `sprites/sky.png` |
| −38, −36 | the moon, three clouds (drifting at 0.06 of the speed) | `og.Sprite` + `og.Flipbook` (`sprites/props.ogt`) |
| −30 | the far skyline, antenna lights blinking | `og.Tilemap` of `maps/far.ogt`, parallax 0.15 |
| −20 | the near buildings, warm windows (one goes on and off), a neon sign, a water tank | `og.Tilemap` of `maps/near.ogt`, parallax 0.45 |
| −10 | the roof the courier runs on | `og.Tilemap` of `maps/roofs.ogt`, parallax 1 |
| 0, 1 | crates and fences; coins (a 6-frame turn) and dust puffs | `og.Sprite` + `og.Flipbook` |
| 2 | the courier (idle, run, jump, fall, crash) | `og.Sprite` + `og.Flipbook` (`sprites/courier.ogt`) + `og.Afterimage` |
| 10, 100 | the HUD; "+10" | `og.render.text`; `og.render.floatingText` |

The three city strips are text tilemaps (`.ogt` rows of legend
characters: the script writes them, a person can edit them by hand) over one
tileset (`tiles/city.ogt`, with its animated tiles) and draw with `repeat
x`; the camera stays still, so each strip's entity scrolls left at the run
speed times its parallax and jumps right by its own width, seamlessly. The
crates, fences, coins and dust are pools (4, 3, 16 and 4) parked at
x = −1000 until a pattern or a footfall puts them in play.

| Component | On | What it holds |
|---|---|---|
| `Actor` | every row | `kind` (sky, moon, cloud, city, dust, runner, crate, fence, coin), `active` (collides), the hit box `w`×`h`, `parallax` (the share of the run speed a row scrolls at), `wrap` (the span scenery jumps right by), `vy` |
| `Run` | every row; **the runner's is the authority** | `phase` (0 title, 1 running, 2 over), `phaseTicks`, `speed`, `distance`, `coins`, `score`, `best`, `rounds`, `spawnIn`, and the body's `grounded` (copied from the controller), `startHeld` (the starting press is not a jump) and `kick` (a speed Director or Collide gave the body) |
| `Tuning` | every row; **the runner's is the authority** | the tunables below |
| `Hud` | the runner only | the font handle and the visible area the HUD lays out in |

The runner's position is its feet (the sheet's pivot is `0.5 0`), so a
squash keeps them on the roof; its hit box is centred 0.8125 m above them.

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
| `Director` | `og.Input`, `Tuning` | `Run`, `Actor`, `og.Transform`, `og.Sprite` | phases; the speed ramp, distance and score; a new round (pools retired, the courier stood up); a press during the card's hold answered by the hold tick |
| `Jump` | `og.Input`, `Tuning`, `og.CharacterEvent` | `Run`, `Actor`, `og.PlatformerController` | the controller's jump held while the press is, its tunables from `Tuning`; its take-off (a stretch and the sound) and landing (a squash) events |
| `Scroll` | `Run` | `Actor`, `og.Transform` | everything left at speed × parallax; scenery wraps by its span; pooled rows retire past x = −13 |
| `Spawn` | `Tuning` | `og.Rng`, `Run`, `Actor`, `og.Transform`, `og.Sprite` | the next pattern at x = 13 from the dice |
| `Collide` | `og.Transform`, `Tuning` | `Run`, `Actor` | coins collected ("+10" and the coin's pop), crashes (the crash's feel), the best score |
| `Dust` | `og.FlipbookEvent`, `Run` | `Actor`, `og.Transform` | a dust puff at the feet on each `step` event of the run clip |
| `Animate` | `Run`, `Actor`, `Tuning` | `og.Flipbook`, `og.Afterimage` | the courier's clip from the phase and the body, the run clip's pace from the speed, the afterimage near top speed |
| `Hud` | `Run`, `Hud`, `Tuning` | — | the score line, the title card and the game-over card (`og.render.text`) |

Motion reads `Time.scaledDt`, so a hit stop freezes the world while the
camera still shakes. Spawning and retiring at fixed x (beyond the widest
view) keeps the play surface's shape out of the simulation: a phone and a
desktop see the same obstacles at the same ticks.

## Feel (E4)

| Moment | What happens | Where |
|---|---|---|
| running | an 8-frame run at 14 fps, faster with the speed (×0.8–1.6); `step` events on the footfalls kick up dust | `sprites/courier.ogt`, `Animate`, `Dust` |
| take-off | stretch (0.78, 1.28), then the jump frame | `og.tween.squash` in `Jump` |
| landing | squash (1.3, 0.72), recovering with `backOut` | `og.tween.squash` in `Jump` |
| a coin | "+10" pops and rises; the coin grows, lifts and fades | `og.render.floatingText`, `og.tween.parallel` in `Collide` |
| a crash | 8 ticks of hit stop, shake 0.5, a kick of the view, the courier knocked back 0.9 m and up, flashing red, the crash clip played once | `og.time.hitStop`, `og.camera.shake`/`kick`, tweens in `Collide` |
| top speed | four mint ghosts trail the courier | `og.Afterimage` in `Animate` |

The numbers are `FEEL` in `src/systems.ts`. None of them changes a rule:
the playtests below hold with any of them.

## Tunables (`Tuning`, on the runner)

| Field | Default | Range | Unit | What it changes |
|---|---|---|---|---|
| `speed` | 6 | 3–12 | m/s | the pace a round starts at |
| `speedRamp` | 0.12 | 0–1 | m/s² | how fast the pace rises (the cap after 50 s) |
| `speedMax` | 12 | 4–20 | m/s | the top pace (and where the afterimage shows) |
| `jumpHeight` | 2 | 0.5–6 | m | a held jump's height |
| `timeToApex` | 0.333 | 0.15–1 | s | how long it takes to peak (with the height: gravity and take-off speed) |
| `jumpCut` | 1.8 | 1–6 | × | gravity while rising with the press released: the hop's height |
| `apexGravity` | 0.75 | 0.2–1 | × | gravity near the top of a held jump: the hang |
| `fallGravity` | 1.15 | 1–3 | × | gravity on the way down: the landing's snap |
| `spawnInterval` | 1.3 | 0.5–60 | s | the gap between patterns |
| `coinValue` | 10 | 0–100 | points | what a coin adds (and the floating text shows) |

They are written in `assets/scenes/main.ogt` (the runner's `Tuning` block)
and live with `set:runner.Tuning.<field>=<value>` from the command channel
or Tune. The hop and jump heights above were measured with `get:runner.og.Transform.position`
through a tap and a hold; `tests/jump.og.ts` holds them against the obstacles.

## What a Maker changes first

1. **Feel**: `jumpHeight`, `timeToApex`, `jumpCut`, `apexGravity` and `fallGravity` in the scene's `Tuning`.
   Keep the hop below the fence and above the crate (`tests/jump.og.ts`
   checks exactly that). The juice is `FEEL` in `src/systems.ts`.
2. **Pace**: `speed`, `speedRamp`, `speedMax`, `spawnInterval`.
3. **What comes**: `placePattern` and the `top` gate in `Spawn`
   (`src/systems.ts`): add a pattern, change when fences appear.
4. **Look**: the pixel art and the city in `tools/make-assets.py`
   (`courier`, `run_frame`, `tile`, `far_rows`, `near_rows`), then
   `python3 tools/make-assets.py && og cook .`. A new courier clip goes in
   `COURIER_OGT` there and `COURIER` in `src/art.ts` (which checks the
   order). Re-record the reference frames afterwards (below).
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
| `tests/frames.og.ts` | the title, a coin's pickup ("+10", the pop, dust) and the crash inside its hit stop (red flash, shake) match `references/<tier>/frames-*.png` under og-tests' tolerance (`tests/frames.ts`) |
| `tests/replay.og.ts` | the same script of jumps, coins, a crash and a second round, run twice in fresh processes, ends on one world hash |

Each one fails when the behaviour it names is broken (checked by breaking
coin collection, the hop's minimum hold, the card's hold, the coin's
floating text, the crash flash through the hit stop and the determinism of
the dust). After a deliberate change to the look, re-record the frames:
`OG_FX_CAPTURE=1 og test .` (a tier without its own references compares
with `metal` and says so).

## Assets

`tools/make-assets.py` regenerates every asset byte for byte (sheets,
tileset, maps, sounds, the font subset); `LICENSES.md` lists them. The
sounds play through one `og.AudioSource` voice each: Director (a round
starts, and a quiet tick, og sfx's `click` preset in `assets/audio/hold.ogt`,
for a press while the game-over card holds), Jump and Collide (a coin, a
crash) set bits of the `Sounds` resource and Voices plays each once
(`og polish` found the archetype silent, no input answered by a sound;
tests/sounds.og.ts holds it, and tests/flow.og.ts the held press's tick).

## Known limits

- Systems cannot read other entities (hence `Run` and `Tuning` on every
  row; a game resource they declare, `Sounds`, they can) and cannot spawn (hence the pools, the dust's
  included); the camera stays still and the world scrolls.
- The view's size is read once in `start`; on the web the renderer is not
  attached yet then, so the layout assumes 16:9 and a portrait phone sees a
  narrow slice of the roofs.
