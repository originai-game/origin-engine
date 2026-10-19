# Archetypes

Complete games a Maker starts from, each a project like `templates/game`
(`project.ogt`, `src/`, `assets/`, `tests/`, `package.json`,
`tsconfig.json`) that `og run`, `og test` and `og build` take as is. They
rebuild the platform's six Godot Discover prototypes on Origin Engine,
keeping each prototype's premise and lessons and none of its code or art,
and add a top-down room built for the engine's 2D and game-feel modules.

| Directory | Game | Profile | Input | Core loop |
|---|---|---|---|---|
| `runner/` | Rooftop Run | 2d | one button: tap to hop, hold to jump | read the next crate or fence, time the jump, keep the pace |
| `match3/` | Tile Swap | 2d | taps or drags | spot a line, swap, chase the cascade, beat the target in the moves |
| `topdown/` | Tomb Room | 2d | WASD or arrows, Space to strike, Shift to dash | cross two rooms of a Tiled map, beat the slime, collect the coins and the key, open the door |
| `tower-defense/` | Path Guard | 2d | taps, or the arrows and Space, or a pad | pick a plant, set it on the lawn, watch the visitors re-route round it, hold five waves |

Each archetype's README has its rules, components, systems, tunables, the
commands its playtests use and what a Maker changes first.

## What they share (and what each owns)

- **Pixel-art sprites, flipbooks and tilemaps.** `tools/make-assets.py`
  draws every sheet texel by texel (standard library only, through the
  copied `tools/oglite.py`) and writes its `type flipbook` sheet (clips,
  modes, frame events), the tilesets and maps, and a `texture { kind pixel }`
  import for each PNG; two runs give the same bytes. Drawn entities carry
  `og.Sprite` (y-sorted, blob shadows) and play `og.Flipbook` clips; the
  runner's skyline is three parallax `og.Tilemap` strips, the board's
  checker a tilemap, the tomb a Tiled `.tmj` with 47-blob auto-tiled walls
  and merged colliders. The same script makes the sounds and subsets a
  font to the HUD's characters.
- **Game feel from the engine.** `og.tween` (with the squash and flash
  presets and completion events by tag), `og.time.hitStop`,
  `og.camera.shake`/`kick`/`punch`, `og.render.floatingText` and
  `og.Afterimage`; each README's Feel table names where each is used and
  how much. The amounts follow the og-feel skill.
- **One row shape.** Every drawn entity carries the same components, and
  the round's state and tuning ride on every row with one row (the runner,
  the board) as the authority: a system sees one query and cannot reach
  another entity or a game resource yet. Each README says how this changes
  once it can.
- **Pools, not spawning in systems.** Obstacles, coins and tiles are spawned
  in `start` and recycled by systems.
- **Tunables.** The `Tuning` block (the scene's, or the component's
  defaults in the topdown) holds them; `set:<entity>.Tuning.<field>=<value>` and Tune write
  them live.
- **Playtests through real input.** `tests/*.og.ts` drive keys, taps and
  drags through the command channel, plus a game command or two (`place`,
  `deal`, `swap`) for states a test needs at once, and assert on
  `dump_entity` of the authority row. `tests/frames.og.ts` compares frames
  with `references/<tier>/` (`OG_FX_CAPTURE=1 og test .` records them) and
  `tests/replay.og.ts` runs the same input twice in fresh processes and
  requires one world hash. `og test .` runs them.

## Scaffolding one

`@origingame/create` scaffolds `templates/game` only. Until it takes an
archetype, copy the directory:

```sh
cp -R templates/archetypes/runner my-runner && cd my-runner
og test . && og run .
```

then change `name` and `id` in `project.ogt` and `package.json`.
