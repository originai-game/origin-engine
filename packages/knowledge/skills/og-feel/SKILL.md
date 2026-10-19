---
name: og-feel
description: Game feel for an AI making a game on Origin Engine — input that answers on the same tick, anticipation and recovery, hit stop by impact weight, screen shake trauma and when not to shake, squash and stretch amounts, easing by intent, floating text, camera look-ahead, dead zones and zones, 2D frame counts and rates by action, sprite sorting, parallax ratios, platformer and character controller tunables by genre, and a checklist proved with og run --commands captures and world dumps. Use when a game works but feels flat, when adding hits, jumps, pickups or a camera, or before calling a 2D game done.
---

A game feels right when every input shows an answer at once, every impact
has weight, and the camera helps the player read what comes next. On
Origin Engine all of it is deterministic data: tweens, flipbooks, the time
scale, camera trauma and floating texts live in the world's state blocks,
so a replay shows the same juice and `og test` can check it. This skill's
modules are `reference/time.md` (time scale, hit stop), `reference/tween.md`,
`reference/camera.md` and `reference/systems.md` (the camera controllers the
engine ships in TypeScript); sprites, flipbooks, tilemaps and floating text
are og-game's (§8, §10). This skill is how much of each to use, and how to
prove it.

## 1. Answer on the same tick

- Read input in a system and act in **the same tick**: start the jump, the
  swing, the step. Never queue a press for "the next frame" or wait for an
  animation to finish before moving. At 60 Hz one tick is 17 ms; two ticks
  of delay are felt.
- Buffer early presses (a jump pressed up to 6 ticks before landing jumps
  on landing) and allow coyote time (a jump up to 6 ticks after leaving a
  ledge). Keep the counters in a component so snapshots carry them.
- Move gameplay by `time.scaledDt` (the systems' `time`): slow motion and
  hit stop then reach your code the same way they reach physics, og.anim,
  flipbooks and tweens. Use `time.dt` only for what must keep its pace
  (menus, input buffers, the UI).
- Prove it: `0.5:key_down:Space;0.517:dump_entity:hero:a.json` — the dump
  of the tick after the press already shows the new velocity or clip.

## 2. Anticipation, action, recovery

Every action has three beats; the action itself is the shortest.

| Beat | Frames at 60 Hz | How |
|---|---|---|
| Anticipation | 0 for player-driven moves (latency wins), 4–12 for enemies and big attacks | a wind-up clip or `og.tween.to` with `backIn` on scale or position |
| Action | 1–4 | the frame the hit lands: fire the hit on a flipbook `event` (`event 2 "hit"`), not on a timer |
| Recovery | 6–20 | a recover clip, `og.tween.squash` easing back, a settle with `backOut` |

The player's own attacks skip anticipation; telegraph enemy attacks with
it (a flash, a crouch, a 0.25–0.5 s wind-up) so the player can react.

## 3. Hit stop by impact weight

`og.time.hitStop(world, ticks)` freezes physics, og.anim, flipbooks and
scaled tweens while input, the UI, unscaled tweens and camera shake keep
moving. It never shortens one already running.

| Impact | Ticks at 60 Hz |
|---|---|
| Light (a jab, a coin, a small enemy hit) | 2–3 |
| Medium (a sword hit, a stomp, a 4-match) | 4–6 |
| Heavy (a charged attack, a boss hit, a big cascade) | 8–12 |
| Finishing blow, a death | 12–20, paired with slow motion |

- Pair a hit stop with a flash (`og.tween.flash`, 0.08–0.15 s) and a small
  shake; the stop is what sells the contact.
- Slow motion: `og.time.setScale(world, 0.3)` for a last kill or a near
  miss, then back to 1 after 0.5–1 s of real time (count ticks, not
  scaled time). Never leave the scale below 1 by accident: assert it in a
  test.
- Do not hit-stop on every bullet of a fast weapon or every coin of a
  stream; cap it to one stop per 10 ticks.

## 4. Screen shake: trauma, not offsets

`og.camera.shake(world, trauma)` adds trauma (0..1) to og.Camera, which
decays by `traumaDecay` (1.5 per second) on the unscaled tick. The offset
and roll grow with trauma squared, so small values stay subtle.

| Event | Trauma |
|---|---|
| A light hit, landing from a high jump | 0.1–0.2 |
| A medium hit taken, a big match | 0.25–0.4 |
| An explosion, a boss slam, the player's death | 0.5–0.8 |
| 1.0 | almost never: it is nauseating |

- Shake the camera for what hurts or hits hard; **never** for UI, pickups,
  menus, steady events (engine hum, rain), or every shot of a rapid weapon.
  A game that shakes all the time has no shake left for the moments that
  count. A refused press or a wrong answer shakes the element, not the
  camera (`og.ui.shake`, og-ui §6).
- Prefer `og.camera.kick(world, direction, distance)` (0.1–0.3 units in the
  direction of the force) for directional impacts and recoil, and
  `og.camera.punch(world, 0.05–0.15)` for a zoom-in on a crit or a slam.
  They are visual only (og.Camera's position stays put, the frame moves),
  so they never change the simulation.
- Offer a reduced-motion setting that scales trauma, kick and punch by 0.3.

## 5. Squash and stretch

`og.tween.squash(world, { entity, impact, duration, ease })` sets
og.Transform.scale to rest × impact on the tick it is called and eases
back (default backOut, 0.18 s). Keep the area roughly constant: x × y ≈ 1.

| Moment | impact (x, y) | duration |
|---|---|---|
| Landing (small hop) | (1.15, 0.85) | 0.12 s |
| Landing (big fall) | (1.3, 0.7) | 0.18 s |
| Take-off | (0.85, 1.15) | 0.1 s |
| Hit taken | (1.2, 0.8) | 0.15 s, with a flash |
| Pickup collected (on the pickup) | (1.4, 1.4) then shrink to 0 | 0.15 s |
| UI button press (the skin's `press` does it for og.ui) | (0.92, 0.92) | 0.08 s, unscaled |

Do not squash rigid things (crates, walls, the camera's target in a
cutscene) or anything more than once per 6 ticks.

## 6. Easing by intent

`og.tween.to(world, { entity, component, field: "position.y", to: [2],
duration, ease, delay, loops, yoyo })` animates any numeric component field;
`og.tween.sequence` and `og.tween.parallel` chain them, `og.TweenCompleted`
reports the end, and `unscaled: true` keeps a tween going through slow
motion and hit stop (UI motion). `og.tween.squash` (§5) and
`og.tween.flash` are the presets.

| Intent | Ease |
|---|---|
| Things arriving and settling (UI in, a gem landing) | `cubicOut`, `quartOut` |
| Things leaving (UI out, a dying enemy fading) | `cubicIn`, `expoIn` |
| A deliberate move from A to B (camera, a swap) | `cubicInOut` |
| A pop, a button, squash recovery | `backOut` |
| A wind-up (anticipation) | `backIn` |
| Springy, cartoon impact | `elasticOut` (sparingly: it rings for a while) |
| A drop that lands | `bounceOut` |
| Constant motion, timers, scrolling, colour cycles | `linear` |
| Most small motion when unsure | `quadOut` (the default) |

Durations: 0.1–0.2 s for feedback, 0.12–0.3 s for UI (the skin's
durations, og-taste §5), 0.2–0.35 s for swaps, 0.4–0.8 s for camera moves,
reveals and screen transitions (up to 0.7 s). Longer feels sluggish unless
it is a cutscene or a reward.

## 7. Floating text

`og.render.floatingText(world, { text, position, ... })` pops a text that
rises, drifts and fades by itself (world state, deterministic).

- Damage and score numbers above the hit point, not on the HUD: the eye is
  already there. Pickups: what was gained (`+1`, `key`), short.
- Colour by meaning and keep it consistent: white damage dealt, red damage
  taken, yellow gold, green heal. Always an outline (the default 0.6) so it
  reads on any background.
- Crits and combos: `crit: true` (critScale 1.6 and a stronger pop); the
  biggest number on screen should be the most important.
- Lifetime 0.6–1.0 s, rise 0.8–1.5 units, spread 0.2–0.4 so stacked hits do
  not overlap. At most 6–8 on screen: sum rapid hits into one number.

## 8. Camera: look-ahead, dead zones, zones, rails

- `og.Follow2D` with a dead zone of 0.5–1.5 units horizontally (smaller
  vertically) so small moves do not swim the screen; `smoothTime` 0.1–0.2 s,
  `smoothTimeY` 0.2–0.35 s in platformers (follow jumps lazily, land
  firmly).
- Look-ahead: `lookAhead` 0.15–0.3 s with `lookAheadMax` 2–4 units in the
  running direction; the player should see more of where they go than of
  where they were. Runners and shooters lean hard ahead; top-down games
  lightly.
- `og.CameraZone` per room or arena keeps the view inside it (applied to
  the frame, from the real aspect, so the simulation stays independent of
  the screen); a zone smaller than the view centres it. Its `scrollTime`
  (0.3–0.6 s, eased in and out) scrolls the view from the room the player
  left into this one when they cross; 0 cuts at once. og.Camera's
  `zoneScroll` says how far a transition has come.
- `og.CameraRail` for corridors and set pieces: the camera rides a
  Catmull-Rom spline to the point nearest the player.
- 3D: `og.OrbitCamera` (drag to turn, wheel to zoom) and `og.FollowCamera`
  on the entity the camera orbits or follows, driven by
  `og.systems.cameras` listed in `GameDesc.systems`.
- The camera never moves on its own during play except for shake, kick,
  punch and a zone's scroll. Do not tween the camera while the player has
  control.

## 9. 2D animation: frames and rates

Flipbooks (`og.Flipbook` on an entity with `og.Sprite`) play by scaled time,
fire `og.FlipbookEvent` on frames named in the sheet, and loop, ping-pong
or play once.

| Action | Frames | fps |
|---|---|---|
| Idle (breathing) | 2–4 | 4–6 (or ping-pong) |
| Walk | 4–8 | 8–12 |
| Run | 6–8 | 12–16 |
| Jump (up, apex, fall) | 1 each, chosen by vertical speed | — |
| Attack (wind-up, hit, recover) | 3–6, once | 12–20, `event` on the hit frame |
| Hurt | 1–2, once | 8–12, with a flash |
| Death | 4–8, once | 8–12 |
| Pickups, coins | 4–8 loop | 8–12 |

- Put footsteps and hits on `event` lines, never on timers: they stay in
  step under slow motion and speed changes.
- Match the walk rate to the move speed (the feet should not skate):
  `Flipbook.speed = speed / walkSpeed`.
- Face with `og.Sprite.flipX` instead of drawing left-facing frames; keep
  the pivot at the feet (`pivot 0.5 0`).
- Pixel art: `texture { kind pixel }` import settings (lossless, nearest)
  and whole-pixel scales: ortho `height` = visible tiles; one tile one unit.

## 10. Sprite sorting and layers

- Layers by role: tiles −10…−1 (`og.Tilemap.layer`), shadows and the floor
  decals 0, characters and props 0 with `sortY`, effects 10–20, floating
  text 100, the HUD in screen space.
- Top-down: `sortY: true` on everything that stands (characters, trees,
  pillars) and the pivot at the feet, so lower on the screen draws in
  front. Wall tops and roofs go in a tile layer above the characters.
- Blob shadows (`og.Sprite.shadow` 0.3–0.45) ground characters and
  pickups; they draw under every sprite of their layer.
- Trails on 2D sprites: `og.Trail` with `layered: true` and the layer
  below the sprite's. Afterimages (`og.Afterimage`, 3–5 ghosts every 2–3
  ticks) for dashes and speed bursts only.

## 11. Parallax

Tile layers scroll by their `parallax` (1 with the world, 0 with the
camera); repeat far layers along x (`repeat x`).

| Layer | parallax x | parallax y |
|---|---|---|
| Sky, far gradient | 0–0.05 | 0 |
| Distant skyline, mountains | 0.1–0.25 | 0.05–0.1 |
| Mid buildings, hills | 0.4–0.6 | 0.2–0.4 |
| Play layer | 1 | 1 |
| Foreground (out of focus, sparse) | 1.2–1.5 | 1–1.2 |

Keep parallax backgrounds darker and less saturated than the play layer,
and never put gameplay on a layer whose parallax is not 1.

## 12. Controllers by genre

Use `og.PlatformerController` (2D) and `og.CharacterController` (3D)
instead of moving bodies by hand: they already have coyote time, a jump
buffer, variable jump height and moving platforms. Tune by feel, starting
from these (the defaults are the first column):

| `og.PlatformerController` | precise (Celeste-like) | floaty (kid-friendly) | heavy (Castlevania-like) |
|---|---|---|---|
| `maxSpeed` (m/s) | 8 | 6 | 5 |
| `acceleration` / `deceleration` | 70 / 80 | 30 / 25 | 120 / 160 (snappy, committed) |
| `airAcceleration` / `airDeceleration` | 45 / 25 | 20 / 10 | 15 / 5 |
| `jumpHeight` (m) / `timeToApex` (s) | 3.2 / 0.4 | 3.5 / 0.55 | 2.4 / 0.35 |
| `fallGravity` (× rising) | 1.8 | 1.2 | 2.4 |
| `jumpCut` (× gravity after release) | 2.5 | 1.5 | 1 (no cut: a fixed arc) |
| `apexGravity` / `apexThreshold` | 0.6 / 1.5 | 0.4 / 2 | 1 / 0 |
| `coyoteTime` / `jumpBuffer` (s) | 0.1 / 0.12 | 0.15 / 0.2 | 0.05 / 0.08 |
| `airJumps`, `wallJumps` | 0, 1 | 1, 0 | 0, 0 |

- Size the jump to the level: `jumpHeight` 0.5 m above the highest ledge a
  player must reach, so a perfect press is not required.
- `cornerCorrection` (0.15 m) forgives head bumps; keep it under a third of
  the character's width.
- 3D characters: `maxSlope` 45° (55° for arcade), `stepHeight` a third of
  the capsule's height, `snapDistance` 0.2–0.3 m so stairs down do not
  bounce, `jumpSpeed` from the height you want: `sqrt(2 × 9.81 × h)`.
- Read `og.CharacterEvent` for juice: `CHARACTER_LANDED` with its `speed`
  scales the landing squash and dust; `CHARACTER_JUMPED` plays the jump
  stretch (§5) and sound on the same tick.
- Ragdolls: `blendTime` 0.15–0.3 s from the animation to the bones; set
  `weight` 1 on the killing blow's tick, with its hit stop, so the fall
  starts from the hit pose.
- Prove it: `0.5:key_down:Space;0.9:dump_entity:hero:apex.json` — the dump
  shows `og.PlatformerController.jumping` and a height within 10 % of
  `jumpHeight` at `timeToApex`. `templates/archetypes/runner`'s courier is a
  PlatformerController with its tuning in a `Tuning` resource;
  `templates/archetypes/topdown`'s hero is a CharacterController in its
  top-down mode.

## 13. The checklist, proved

Make each a playtest (`tests/*.og.ts`) or at least a scripted run whose
outputs you read:

1. **Same-tick response**: `t:key_down:X;t+1/60:dump_entity:hero:a.json`
   shows the new state on the next tick.
2. **Hit lands with weight**: during a hit, `dump_entity:enemy` shows
   `og.Sprite.flash` > 0 and its position equal across the hit-stop ticks;
   `dump_world` shows `og.Time` with `hitStop` > 0 on the stop's first tick.
3. **Shake decays**: the og.Camera resource in a world dump shows `trauma`
   rising on the hit and back to 0 within about a second.
4. **Feet do not skate, steps are heard**: count og.FlipbookEvent `step`
   events over a scripted walk (a game resource or a fact).
5. **Floating text appears**: a `screenshot` 0.2 s after a hit shows the
   number above the hit point (read the PNG).
6. **The camera leads**: two screenshots while running show more space ahead
   of the hero than behind; a dump shows `og.Follow2D.lead` non-zero.
7. **Sorting**: a screenshot where a character walks behind and in front of
   a pillar.
8. **Determinism**: a playtest with `runs: 2` and
   `t.equal(run.hashes[1], run.hashes[0], ...)` over a session with hits,
   pickups and a hit stop: the juice must replay exactly.
9. **No stray slow motion**: the last world dump shows `og.Time.scale` 1.
10. **Rooms scroll**: `get:og.Camera` on the ticks after the hero crosses a
    doorway shows `zoneScroll` rising from 0 to 1 (topdown's `door.og.ts`).

`templates/archetypes/topdown`, `runner` and `match3` do all of this, and
`tower-defense`, `roguelike`, `idle` and `rhythm` too (each README's feel
table names where); read their `src/` and `tests/` for working patterns.

## 14. Feel, measured: `og polish`

`og polish .` scores this checklist's measurable part (input latency, hit
feedback, idle frames, sound on input, frame time) next to og-taste's;
og-build §5 is the one list of its measures, thresholds and fixes. Expose
`hp`, `hits` or `kills` as facts (og-game §3) or the hit measures stay n/a.
