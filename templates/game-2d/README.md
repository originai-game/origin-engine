# The 2D starter

What `npm create @origingame/game` writes for a 2D game (`--profile 2d`, or
a premise that says 2D, platformer, side-scroller, metroidvania or pixel
art): the smallest whole 2D Origin Engine project. A courier runs and jumps
the rooftops of one level, takes coins, jumps up through one-way ledges and
lands on them, loses a life in the pit and clears the level at the far
wall; Escape (or the HUD's Pause) opens the pause menu. Beside the spawn a
lamp keeper, a 2D skeletal rig, breathes on a ramp with its feet planted.

```sh
og run .     # headless with the probe; og dev . reloads on save
og test .    # tests/smoke.og.ts, tests/keeper.og.ts
og build .   # the play page
```

## What it demonstrates, and the skills it exercises

| Part | Where | Skill |
|---|---|---|
| The project in the 2D profile: an orthographic camera, sprites, 2D physics | `project.ogt`, `start` in `src/main.ts` | og-game §2, §8 |
| A hand-written tilemap over a tileset: a parallax skyline, roofs that collide, ledges on their own collider layer (`colliderLayers 2`) that the controller makes one-way | `assets/maps/level.ogt`, `assets/tiles/city.ogt` | og-game §8 (the grammar) |
| A flipbook sheet, its clips numbered in file order and checked against the code | `assets/sprites/courier.ogt`, `CLIP`, `checkClips` | og-game §8 |
| A 2D skeletal rig: bones, slots, region attachments, a weighted-mesh cape the idle clip deforms, two-bone IK legs planted on a slope (og.IkTarget2D), its clip event read as og.Skeleton2DEvent, its lantern an og.Light2D under og.Lighting2D (layers 1 and 2) | `assets/rigs/keeper.ogt`, `spawnKeeper`, `Plant`, `tests/keeper.og.ts` | og-game §8 |
| og.PlatformerController: run, jump (held for the full height), coyote time, the jump buffer, one-way ledges, Down to drop | `Control`, the hero in `start` | og-game §13; og-feel §12 |
| The map's objects: the spawn, the coins, the goal | `start`, `Pickups` | og-game §8 |
| Input on the press's tick: keys, a pad, the HUD's touch controls on a phone | `Control`, `Hud` | og-feel §1; og-ui §5 |
| The UI kit's platformer HUD and pause menu, every word from the string table | `Hud`, `assets/strings/en.ogt` | og-ui §1, §6; og-i18n §1 |
| A camera that follows with a dead zone and look-ahead | og.Follow2D on the hero | og-feel §8 |
| The frame's art direction as data: a look extending the 16bit-pixel preset | `assets/looks/origin-game-2d.ogt` (a created game's is `<id>.ogt`) | og-taste §1, §12 |
| A playtest that asserts on dumps, never on the code | `tests/smoke.og.ts` | og-build §3, §4 |

What a game made from it changes first: the art (the courier and the city
are the runner archetype's, `templates/archetypes/runner/tools/make-assets.py`
draws them), the level (`assets/maps/level.ogt`, or a Tiled `.tmj` beside
it, og-game §8), the controller's tunables (og-feel §12) and what the map's
objects mean. The 2D archetypes (`templates/archetypes/`) are whole games
to start from instead: a runner, a top-down room, match-3, a roguelike,
tower defence, an idle game and a rhythm game.

Asset licences: `LICENSES.md`.
