# Archetypes

Complete games a Maker starts from, each a project like `templates/game`
(`project.ogt`, `src/`, `assets/`, `tests/`, `package.json`,
`tsconfig.json`) that `og run`, `og test` and `og build` take as is. They
rebuild two of the platform's Godot Discover prototypes on Origin Engine,
keeping each prototype's rules and feel and none of its code or art.

| Directory | Game | Profile | Input | Core loop |
|---|---|---|---|---|
| `runner/` | Rooftop Run | 2d | one button: tap to hop, hold to jump | read the next crate or fence, time the jump, keep the pace |
| `match3/` | Tile Swap | 2d | taps or drags | spot a line, swap, chase the cascade, beat the target in the moves |

Each archetype's README has its rules, components, systems, tunables, the
commands its playtests use and what a Maker changes first.

## What they share (and what each owns)

- **Sprites as glTF quads.** `tools/make-assets.py` draws every sprite from
  vector shapes and packs them into `assets/models/sprites.glb`, one quad
  mesh and one emissive, alpha-masked material per sprite; `src/art.ts`
  names them in the same order. `start` sets the renderer to show emissive
  colour exactly (no environment light, a dark sun, tone mapping off). The
  same script makes the sounds and subsets the template's Noto Sans SC to
  the HUD's characters. `tools/oglite.py` (the drawing, PNG, GLB and WAV
  writer, standard library only) is copied into each archetype so each
  stands alone.
- **One row shape.** Every drawn entity carries the same components, and
  the round's state and tuning ride on every row with one row (the runner,
  the board) as the authority: a system sees one query and cannot reach
  another entity or a game resource yet. Each README says how this changes
  once it can.
- **Pools, not spawning in systems.** Obstacles, coins and tiles are spawned
  in `start` and recycled by systems.
- **Tunables in the scene.** The `Tuning` block in `assets/scenes/main.ogt`
  holds the defaults; `set:<entity>.Tuning.<field>=<value>` and Tune write
  them live.
- **Playtests through real input.** `tests/*.og.ts` drive keys, taps and
  drags through the command channel, plus a game command or two (`place`,
  `deal`, `swap`) for states a test needs at once, and assert on
  `dump_entity` of the authority row. `og test .` runs them.

## Scaffolding one

`@origingame/create` scaffolds `templates/game` only. Until it takes an
archetype, copy the directory:

```sh
cp -R templates/archetypes/runner my-runner && cd my-runner
og test . && og run .
```

then change `name` and `id` in `project.ogt` and `package.json`.
