# The 2D starter

What `og new` writes for a 2D game (`--profile 2d`, or
a premise that says 2D, platformer, side-scroller, metroidvania or pixel
art): the smallest whole 2D Origin Engine project. A courier runs and jumps
the rooftops of one level, takes coins, jumps up through one-way ledges and
lands on them, loses a life in the pit and clears the level at the far
wall; Escape (or the HUD's Pause) opens the pause menu. Beside the spawn a
lamp keeper, a 2D skeletal rig, breathes on a ramp with its feet planted
and walks along it when sent (`keeper:<x>`): it turns over a few ticks,
steps along the plank, lets both feet land and turns back.

Made with the top-down-2d view (`og new --view top-down-2d`, src/creation.ts's
`view`), the same game is played from above in a courtyard
(`src/courtyard.ts`): the courier walks in any direction as an
og.CharacterController in its top-down mode, facing down, up or to the
side; Dash (Space or Left Shift, seat 1 Enter or Right Shift, pad A, the
touch Dash) sends it a burst along its way with ghosts behind it, a sound
and a cooldown that flashes it when ready. A canal crosses the yard:
walking in costs a life and stands the courier on the bank it left, a dash
carries it over. Coins lie about the yard, the gate beside the lamp keeper
clears it, and the HUD, the pause and the two-seat framing are the
rooftops' (`src/run.ts`).

## Rules

| | Side-on (side-2d, the default) | From above (top-down-2d) |
|---|---|---|
| Move | the arrows, A and D, a pad, the touch d-pad | WASD, the arrows, a stick or d-pad, the touch stick, in eight directions |
| Button | Jump (held for the full height; Down drops through a ledge) | Dash: 2.1 m in 9 ticks, then 32 ticks of cooldown |
| Coins | 7 on the rooftops | 8 about the courtyard |
| Hazard | the pit: a life, back at the spawn | the canal: a life, back on the bank |
| Win | the far wall | the gate |
| Loss | the last life starts the level again, every coin back | the same |

Lives 3, shared by the seats; Escape, a pad's Start or the HUD's Pause
opens the pause screen for everyone. Two seats play on one device (WASD
and the left keys against the arrows and the right keys, pad 1 against
pad 2, two touch columns on a phone) or over a room in lockstep.

```sh
og run .     # headless with the probe; og dev . reloads on save
og test .    # tests/smoke.og.ts, tests/keeper.og.ts, tests/seats.og.ts
og build .   # the play page
```

## What it demonstrates, and the skills it exercises

| Part | Where | Skill |
|---|---|---|
| The project in the 2D profile: an orthographic camera, sprites, 2D physics | `project.ogt`, `start` in `src/main.ts` | og-game §2, §8 |
| A hand-written tilemap over a tileset: a parallax skyline, roofs that collide, ledges on their own collider layer (`colliderLayers 2`) that the controller makes one-way | `assets/maps/level.ogt`, `assets/tiles/city.ogt` | og-game §8 (the grammar) |
| A flipbook sheet, its clips numbered in file order and checked against the code | `assets/sprites/courier.ogt`, `CLIP`, `checkClips` | og-game §8 |
| A 2D skeletal rig: bones, slots, region attachments, a weighted-mesh cape the idle clip deforms, two-bone IK legs planted on a slope (og.IkTarget2D) and stepping along it under a walk clip with Bézier keys (turning over 21 ticks, as fast as the feet can follow, no foot faster than the gait's swing, settling before idle), its clip events (breaths, footfalls) read as og.Skeleton2DEvent, its lantern an og.Light2D under og.Lighting2D (layers 1 and 2) | `assets/rigs/keeper.ogt`, `spawnKeeper`, `Walk`, `Plant`, `tests/keeper.og.ts` | og-game §8 |
| og.PlatformerController: run, jump (held for the full height), coyote time, the jump buffer, one-way ledges, Down to drop | `Control`, the hero in `start` | og-game §13; og-feel §12 |
| The map's objects: the spawn, the coins, the goal | `start`, `Pickups` | og-game §8 |
| Input on the press's tick: keys, a pad, the HUD's touch controls on a phone | `Control`, `Hud` | og-feel §1; og-ui §5 |
| og.ui's HUD components (the lives as `og.ui.statBar` pips, coins and score as `og.ui.counter`, Pause, a d-pad and Jump on a phone) and the game's own pause screen, ejected with `og ui eject pause` and edited, every word from the string table | `Hud`, `assets/ui/pause.ogt`, `assets/strings/en.ogt` | docs/ui.md §8, §11; og-i18n §1 |
| A camera that follows with a dead zone and look-ahead | og.Follow2D on the hero | og-feel §8 |
| The frame's art direction as data: a look extending the 16bit-pixel preset | `assets/looks/origin-game-2d.ogt` (a created game's is `<id>.ogt`) | og-taste §1, §12 |
| A playtest that asserts on dumps, never on the code | `tests/smoke.og.ts` | og-build §3, §4 |
| The view as a creation choice: one module per view over shared state, HUD and voices | `src/main.ts` (`TOP_DOWN`), `src/courtyard.ts`, `src/run.ts` | og-game §2 |
| og.CharacterController top-down (gravityScale 0): eight-way walk sliding along the walls, a facing from the stronger axis | `Control` in `src/courtyard.ts` | og-game §13; og-feel §12 |
| A dash with game feel: og.Afterimage ghosts in each seat's scarf colour, a cooldown that flashes the courier when ready, its sound; the canal's splash with a hit stop, a shake, a flash and a flicker | `Control`, `Yard` | og-feel §3, §4, §10 |
| A top-down tilemap: ground, a colliding walls layer, lanterns drawn as their own layer, the map's objects (spawn, coins, `hazard`, goal, keeper, lamps) | `assets/maps/courtyard.ogt`, `assets/tiles/courtyard.ogt` | og-game §8 |
| A three-quarter flipbook: idle, walk and dash facing down, up and to the side (flipX faces left), checked against the code | `assets/sprites/courier-top.ogt`, `checkClips` in `src/courtyard.ts` | og-feel §9 |
| The keeper from above: the same rig breathing with its feet's IK planted, a body the couriers walk round, its lantern and the lamp posts as og.Light2D | `spawnKeeper`, `lamp` in `src/courtyard.ts` | og-game §8 |
| Seats read through og.PlayerInput alone, checked from keys, pads and touch in either view | `tests/seats.og.ts` | og-net |

## Components, resources and systems

| Name | View | What it is |
|---|---|---|
| `Game` (resource) | both | coins, lives, won, paused, each seat's courier position, the sounds asked for (`src/run.ts`) |
| `Coin`, `Voice` | both | a coin taken once; a sound's voice |
| `Hero`, `View`, `Plant`, `KeeperWalk` | side-2d | a courier's spawn and buttons; the camera's target; the keeper's feet and walk |
| `Walker` | top-down-2d | a courier's spawn, facing, dash and cooldown, the last bank it stood on, its stumble |
| `Yard` (resource) | top-down-2d | the courtyard's tunables and the canal's and the gate's rectangles |
| `Seats`, `Control`, `Frame`, `Pickups`, `Hud`, `Voices` | both (`Control`, `Pickups` per view) | input, movement, two-seat framing, pickups and the goal, the HUD and pause, sounds |
| `Animate`, `Walk`, `Plant` | side-2d | the courier's clips; the keeper's walk and feet |

## Tunables

Side-on: og.PlatformerController's fields on the hero (og-feel §12).
From above, `Yard` (Tune, or `set:Yard.dashSpeed=16`):

| Field | Default | What |
|---|---|---|
| `speed` | 5 m/s | the walk |
| `dashSpeed`, `dashTicks` | 14 m/s, 9 | the dash: 2.1 m, twice the canal's width |
| `dashCooldown` | 32 ticks | after the dash, before the next |
| `splashShake` | 0.35 | the camera's trauma when a courier falls in |

## Commands

`bot:<n>` (scripted seats), `warp:<x>:<y>` (the courier's feet there),
`viewport:<desktop|phone>` (lay the UI out as a portrait phone, its touch
controls showing) in both views; `keeper:<x>` side-on.

## Playtests

| File | Side-on | From above |
|---|---|---|
| `tests/smoke.og.ts` | two coins, up through the ledge, pause and resume, the pit costs a life | pause and resume, two coins, the canal costs a life, a dash crosses it, the gate clears the courtyard; frames against `references/metal/courtyard-<single|local|phone-local>-*.png` (alone, two on a desktop, two on a portrait phone) |
| `tests/keeper.og.ts` | the keeper's breathing, walk and IK (`tests/keeper.og.ts`) | the keeper breathes with both feet reached and its lantern lit; the courier stops at it |
| `tests/seats.og.ts` | each seat's keys, pad and touch reach its og.PlayerInput and move its own courier | the same |

The top-down playtests live in `tests/courtyard.ts`; `smoke.og.ts` and
`keeper.og.ts` play them when the view is top-down-2d. Record the frames
again with `OG_FX_CAPTURE=1 og test .` in a top-down scaffold.

What a game made from it changes first: the art (the courier and the city
are the runner archetype's, `templates/archetypes/runner/tools/make-assets.py`
draws them; the courtyard, its tiles and the courier from above are drawn
by `tools/make-courtyard.py` in the engine repository, which imports that
file), the level (from above: `assets/maps/courtyard.ogt`, its rows and
objects in `tools/make-courtyard.py`), the side-on level (`assets/maps/level.ogt`, or a Tiled `.tmj` beside
it, og-game §8), the controller's tunables (og-feel §12) and what the map's
objects mean. The 2D archetypes (`templates/archetypes/`) are whole games
to start from instead: a runner, a top-down room, match-3, a roguelike,
tower defence, an idle game and a rhythm game.

Asset licences: `LICENSES.md`.
