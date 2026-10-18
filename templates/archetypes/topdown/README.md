# Tomb Room — the topdown archetype (2D)

A small Zelda-like. Two rooms of a tomb: a slime guards the west room and
the key in it, coins lie in both, and a locked door stands between them.
Walk with WASD or the arrows, swing the sword with Space, dash with Shift.
Take the key to the door and the east room opens.

古墓房间：一个小型俯视角冒险。西边的房间里有史莱姆守着钥匙，两个房间都散落着金币，中间是一扇锁着的门。WASD 或方向键移动，空格挥剑，Shift 冲刺；拿到钥匙走到门前，东边的房间就打开了。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests. It shows
the 2D modules working together: a Tiled map with an auto-tiled, colliding
wall terrain; flipbook sprites with frame events; y-sorted sprites with
blob shadows; tweens, hit stop, camera trauma and kick, floating damage
numbers and afterimages; a 2D follow camera held inside each room by a
camera zone; and an enemy whose mind is data: a behaviour tree
(`assets/ai/slime.ogt`, og.ai) walking a grid made from the map's wall
collision (og.nav).

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.2:key_down:KeyD;1.2:key_up:KeyD;1.3:screenshot:play.png;1.4:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og build .                   # dist/: the play page for the browser
```

## Rules

- **Moving.** WASD or the arrows move the hero at `Tuning.speed` on the
  tick the keys are read; a diagonal is normalised. The hero faces the
  stronger axis (a diagonal keeps the old facing when it is one of its keys).
  Walls stop the body; the slime and the hero pass through each other.
- **The sword.** Space (or J) swings in the facing direction. The hero is
  rooted for the swing's 15 ticks. Damage lands on the attack clip's strike
  frame (its `"hit"` event), within `Tuning.reach` of a point one reach
  ahead of the chest. Hits deal 12, 12, then a 26 crit: the slime (50 hp)
  dies on the third.
- **The dash.** Shift (or K) dashes `Tuning.dashTicks` ticks at
  `Tuning.dashSpeed` along the keys (or the facing), leaving afterimages;
  then `Tuning.dashCooldown` ticks before the next.
- **The slime.** Its behaviour tree, `assets/ai/slime.ogt`, is re-checked
  every tick, first branch first: under 30 hp, and having seen the hero in
  the last 3 s, it **flees** to 5 m away; seeing the hero within 0.8 m it
  **bites** (the game's `bite` command, every 0.5 s at most); seeing the
  hero it **chases** at `Tuning.slimeSpeed`; having lost the hero in the
  last 2 s it **searches** where it last saw it; otherwise it **patrols**
  its beat (four points round the key's corner, 0.6 s at each). It sees all
  round within `Tuning.slimeSight`, but not through walls (the pillar hides
  the hero). og.nav walks it on the map's grid around the walls, writing its
  velocity. A bite costs a heart, knocks the hero away and makes it blink
  for `Tuning.iframes` ticks; the slime bounces back off the bite.
- **Pickups.** Coins and the key are taken when the hero's chest touches
  them. The key is spent opening the door.
- **The door.** Without the key, walking up to it says "locked". With it,
  the doorway's four wall cells clear (`og.tilemap.setTile` re-tiles their
  neighbours and drops their colliders) and the hero walks into room 2.
- **Falling.** At 0 hearts the hero lies still for 90 ticks, then stands up
  at the spawn with full hearts. Coins, the key and the door stay as they
  are.

Everything is deterministic from the project seed: the clock is the fixed
tick, og.ai and og.nav are deterministic and in snapshots, the hovering is a triangle
wave, and `tests/replay.og.ts` replays a whole visit to the same hash.

## How a hit feels

Each sword hit does five things at once, all in `hitSlime`
(`src/systems.ts`):

| Effect | Call | Default |
|---|---|---|
| flash and squash | `og.tween.flash`, `og.tween.squash` (their impact holds through the hit stop) | white for 0.14 s; 1.45 × 0.6 recovering in 0.28 s |
| knockback | the slime's `knockX/knockY`, decaying ×0.82 a tick | `Tuning.knockback` (9 m/s), 16 ticks stunned |
| hit stop | `og.time.hitStop` | `Tuning.hitStop` (4 ticks), 7 on the crit |
| camera | `og.camera.shake`, `og.camera.kick` | trauma `Tuning.shake` (0.25, ×1.8 on the crit); a 0.12 m kick away from the hero |
| damage number | `og.render.floatingText` | white, the crit gold and bigger |

The death is two more tweens: the scale to (1.7, 0) with `backIn` and the
sprite's opacity to 0; the slime hides 30 ticks later. Pickups pop up,
stretch and fade, and squash the hero a little.

## The map

`assets/maps/tomb.tmj` is a Tiled map (orthogonal, 40 × 12 cells of
16 px, uncompressed layers) over the tileset `assets/tiles/dungeon.ogt`
(Tiled's `../tiles/dungeon.tsx` reference resolves to it). One cell is one
world unit; the map's bottom-left corner is the world origin, y up.

| Layer | Kind | Holds |
|---|---|---|
| `floor` | tiles | three slab variants and moss |
| `walls` | terrain `wall` (`autotile`), `collision` | the outer walls, the dividing wall at x 19–20, three pillars; blob47 picks each cell's shape and the solid tiles collide |
| `door` | tiles | the door over the doorway's wall cells |
| `decor` | tiles, `z` = 15 | the animated torches on the north wall, drawn above the actors |
| `objects` | objects | `spawn`, `enemy` (the slime), `coin` ×6, `key`, `door` (the doorway's rectangle) and `room` ×2 (each room's rectangle) |

`start` (`src/main.ts`) reads the objects: actors spawn at their points,
the door's rectangle gives the cells to clear, and each room's rectangle
becomes an `og.CameraZone`. The hero carries `og.Follow2D` (a small dead
zone, `smoothTime` 0.12, `lookAhead` 0.15), and the orthographic view is
12 cells tall: a whole room, as in the old top-down adventures, with the
camera cutting to room 2 when the hero crosses into it.

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | hearts, coins, the key, the door, the room, the slime's hp and hits, the hero's position and facing for the other systems, the strike of this tick, footsteps, invulnerability, knockback, the spawn, the door's cells, sound requests, the font |
| `Tuning` | resource | the knobs below |
| `Hero` | the hero | the swing and dash timers, the dash direction, dashes so far |
| `Slime` | the slime | hp, knockback, stun, bites, death, its clip indices (it also has `og.NavAgent`, `og.AiAgent` and `og.Perception`; the hero has `og.Perceivable`) |
| `Pickup` | coins and the key | kind, taken, the resting height |
| `Sfx` | one entity per sound | its `Sound.*` bit (the entity also has `og.AudioSource`) |
| `Report` | the hero | the run's facts, copied from `Game` each tick so `dump_entity:hero` shows them |

## Systems (in tick order)

| System | Reads | Writes | Job |
|---|---|---|---|
| `Control` | `og.Input`, `Tuning`, `og.FlipbookEvent` | `Game`, `Hero`, `og.Transform`, `og.Velocity`, `og.Flipbook`, `og.Sprite`, `og.Afterimage` | footsteps and strikes from the hero's frame events; walk, swing, dash, knockback, blinking, falling; the clip and flip |
| `Slimes` | `Tuning` | `Game`, `Slime`, `og.Transform`, `og.Velocity`, `og.Flipbook`, `og.Sprite`, `og.NavAgent`, `og.AiAgent`, `og.Perception` | take strikes (all five effects), knockback and stun (`NavAgent.drive` off meanwhile), death (the AI stops), sight from `Tuning.slimeSight` |
| `Pickups` | — | `Game`, `Pickup`, `og.Transform`, `og.Sprite` | hover, take by overlap, pop and fade |
| `Door` | — | `Game` | the room, "locked", opening the doorway |
| `Voices` | `Sfx` | `Game`, `og.AudioSource` | plays each requested sound once |
| `Hud` | `Game` | `Report` | screen text: hearts, coins, the key, the room, the controls for 5 s; the hero's `Report` |

og.ai runs before these systems (the slime's decision for the tick) and
og.nav after them (moving it). During a hit stop the systems still tick but
hold their timers, as physics, flipbooks and tweens do.

## Tunables (`Tuning`)

| Field | Default | Range | Unit | What it changes |
|---|---|---|---|---|
| `speed` | 4.5 | 1–10 | m/s | walking pace |
| `dashSpeed` | 13 | 4–24 | m/s | the dash |
| `dashTicks` | 9 | 2–30 | ticks | the dash's length (dashSpeed × dashTicks / 60 m) |
| `dashCooldown` | 36 | 0–120 | ticks | ticks between dashes |
| `reach` | 0.75 | 0.2–2 | m | the sword's reach |
| `hitStop` | 4 | 0–12 | ticks | the freeze per hit (the crit adds 3) |
| `shake` | 0.25 | 0–1 | trauma | camera shake per hit |
| `slimeSpeed` | 1.6 | 0–6 | m/s | the slime's chase and search (the behaviour reads it; it patrols at 0.8) |
| `slimeSight` | 5 | 0–12 | m | how far the slime sees |
| `knockback` | 9 | 0–20 | m/s | both ways: the struck slime and the bitten hero |
| `iframes` | 50 | 0–180 | ticks | the hero's blinking after a bite |

They are the field defaults in `src/components.ts`.

## What a Maker changes first

1. **Feel**: `speed`, `dashSpeed`, `hitStop`, `shake`, `knockback` in
   `Tuning`, and the damage list `DAMAGE` in `src/systems.ts`.
2. **The level**: open `assets/maps/tomb.tmj` in Tiled. Paint the `walls`
   layer with any tile (the terrain picks the shape), move the objects, add
   coins (`type` coin) or rooms (`type` room). Keep the door's rectangle on
   wall cells. Or change `map_layout` in `tools/make-assets.py`.
3. **Enemies**: another `enemy` object needs its own name (the `slime`
   command addresses `slime`); `Slimes` already loops over every slime.
   What a slime does is `assets/ai/slime.ogt`: reorder the branches, change
   the numbers, add a `wander` or a state machine (the nodes are listed in
   the og-ai skill). `dump_ai:slime` shows the running node, the path to it,
   the blackboard and what it sees; `dump_nav:slime` its path.
4. **Look**: the tiles and sprites are drawn in `tools/make-assets.py`;
   run `python3 tools/make-assets.py && og cook .`. The hero's clip order is
   fixed (`src/art.ts` checks it).
5. **Words**: the HUD strings in `Hud`. The font holds Latin-1 only; add
   characters to `subset_font`'s text and `chars` in `ui.ttf.ogt` first.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `place:<x>:<y>` | puts the hero's feet at a world point, standing still |
| `slime:<x>:<y>` | moves the slime |
| `give:<key\|coin\|door>` | the key, a coin, or the doorway opened |
| `face:<down\|up\|left\|right>` | turns the hero |
| `bite:<slime>` | the behaviour's attack (og.ai queues it with the slime's handle): hurts a hero within 0.85 m |
| `dump_ai:slime[:file]`, `dump_nav:slime[:file]` | the slime's behaviour, running node, blackboard and perception; its path |
| core actions | `key_down:KeyD`/`key_up:KeyD` walk, `Space` swings, `ShiftLeft` dashes; `dump_entity:hero` shows `Report` |

`facts` (the probe's `state()`) reports `room`, `hp`, `coins`, `hasKey`,
`enemyHp`, `doorOpen`, `doorWall`, the hero's and slime's positions,
`facing`, `steps`, `swings`, `hits`, `hurts` and `dashes`.

## Playtests (`og test .`)

| File | Proves |
|---|---|
| `tests/walk.og.ts` | one second of D walks 4.5 m; footsteps come from the walk clip's events; up and left stop at the wall terrain's colliders; facing, the side clip and its flip |
| `tests/ai.og.ts` | out of sight the slime patrols its beat on og.nav's path; the hero 3 m off is seen three ticks later and the patrol gives way to the chase at about 1.6 m/s; the bite lands through the `bite` command |
| `tests/flee.og.ts` | the pillar hides the hero 4.5 m away; below the pillar it is seen and chased; under 30 hp the slime flees and keeps over 3.5 m away |
| `tests/combat.og.ts` | a bite costs a heart and knocks the hero back; the hit stop holds the slime still, white and squashed; 12, 12, 26 kill it; it ends hidden and squashed to nothing |
| `tests/door.og.ts` | the doorway blocks without the key; walking over the key takes it; the door opens (the wall cell cleared), the key is spent, the hero reaches room 2 |
| `tests/frames.og.ts` | four frames match `references/<tier>/`: the west room at rest, the crit's freeze frame, a dash, the east room through the open doorway (`OG_FX_CAPTURE=1 og test .` records them) |
| `tests/replay.og.ts` | a whole visit, run twice in fresh processes, ends on the same world hash |

Each one fails when the behaviour it names is broken (checked by halving
the walk speed, dropping the hit's flash, leaving the doorway's walls in
place, drawing the map over the actors, and wandering by `Math.random`;
the AI ones by sight through walls, a bite that never lands and a flee
threshold of 0; `tests/evidence/E6/break-and-restore.md`).

## Assets

`tools/make-assets.py` regenerates every asset byte for byte: the tileset
and its blob47 rules, the Tiled map, the flipbook sheets, the sounds and the
font subset. `LICENSES.md` lists them.

## Known limits (engine gaps, not design)

- The command channel dumps entities, not resources, so the playtests read
  the run through the hero's `Report` copy, and `set:` cannot reach
  `Tuning` (a resource): its fields are tuned in `src/components.ts`.
- `og.CameraZone` clamps the frame only, so crossing into room 2 cuts the
  view rather than scrolling it.
- The blob shadows (0.35) barely show on the dark floor.
