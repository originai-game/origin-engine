# Archetypes

Complete games a Maker starts from, each a project like `templates/game`
and the 2D starter `templates/game-2d` (`project.ogt`, `src/`, `assets/`,
`tests/`, `package.json`, `tsconfig.json`) that `og run`, `og test` and
`og build` take as is; `npm create @origingame/game --archetype <name>`
writes one, with its own profile. They
rebuild the platform's six Godot Discover prototypes on Origin Engine,
keeping each prototype's premise and lessons and none of its code or art,
and add a top-down room built for the engine's 2D and game-feel modules
and a third-person vale, the engine's mid-size 3D slice.

| Directory | Game | Profile | Input | Core loop |
|---|---|---|---|---|
| `runner/` | Rooftop Run | 2d | one button: tap to hop, hold to jump | read the next crate or fence, time the jump, keep the pace |
| `match3/` | Tile Swap | 2d | taps or drags | spot a line, swap, chase the cascade, beat the target in the moves |
| `topdown/` | Tomb Room | 2d | WASD or arrows, Space to strike, Shift to dash | cross two rooms of a Tiled map, beat the slime, collect the coins and the key, open the door |
| `tower-defense/` | Path Guard | 2d | taps, or the arrows and Space, or a pad | pick a plant, set it on the lawn, watch the visitors re-route round it, hold five waves |
| `roguelike/` | Lamp Dive | 2d | the arrows or WASD, a tap on the next cell, a pad, or the HUD's d-pad | step a turn at a time through a fresh tomb, strike what the lamp shows, take what it finds, reach the relic on floor 2 with one life |
| `idle/` | Spark Orb | 2d | taps or Space on the orb, 1–4 or the cards to buy, a pad | tap for sparks, buy generators that make them, double each kind every tenth, collapse into a supernova for stardust, come back to what it made while away |
| `rhythm/` | Three-Lane Beat | 2d | A S D or the arrows, fingers on the lanes, a pad | read the notes falling to the line, hit each on its time, keep the combo and the groove, clear the song |
| `arena/` | Coin Duel | 2d | WASD or the arrows, Space or Shift to dash, a pad; two players over a room | race the other player to the coin, first to five; played in lockstep through `og relay` or the platform's rooms |
| `third-person/` | Ember Vale | 3d | WASD or the arrows, Shift, Space, hold F, Q and E; a pad; a touch joystick, Jump and Kindle | walk, wade and jump a lake vale at dusk, kindle four beacons before night while stone sentinels on their rounds hunt you |

Each archetype's README has its rules, components, systems, tunables, the
commands its playtests use and what a Maker changes first.

## What they share (and what each owns)

- **Pixel-art sprites, flipbooks and tilemaps.** `tools/make-assets.py`
  draws every sheet texel by texel (standard library only, through its
  `tools/oglite.py`, the same file in every archetype) and writes its
  `type flipbook` sheet (clips, modes, frame events), the tilesets and maps, and a `texture { kind pixel }`
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
  once it can. The four later archetypes (tower-defense, roguelike, idle,
  rhythm) keep the play in game resources (`Game`, `Tuning`, `Prefs`)
  that every system reads and writes; a system still writes only the rows
  its query declares, so what another system's rows must show is asked for
  through a resource field and done by the owner the same tick.
- **The platform and the screens.** The later four keep their settings
  through `og.platform.loadSettings`/`saveSettings` and a record (a best,
  a save) in the store at the revision read, merged after a stale write;
  their title, pause, settings and end cards are og-ui widgets on a
  recoloured style pack from `templates/ui-kit`, with every string in
  `assets/strings/en.ogt` and `zh-CN.ogt`, and a phone layout inside the
  safe area. Their `og polish` reports are in the engine repository's
  `tests/evidence/P2/archetypes/`.
- **The 3D one.** `third-person/` is the exception to the pixel-art
  sprites: a heightmap terrain with water, the engine's CC0 humanoids on
  og.CharacterController with blend trees and IK, navmesh sentinels on
  og.ai, particles, decals and the post stack, on the fantasy style pack
  (its README).
- **The look as data.** `idle/` names its frame treatment and skin in
  `assets/looks/idle.ogt` through `GameDesc.look` (og-taste §12); the
  others set theirs in `start` until they move to a look. A look that
  extends a catalogue preset changes the frames, so `references/` are
  recorded again with it.
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

`@origingame/create-game` scaffolds `templates/game`, or an archetype with
`--archetype <directory>`:

```sh
bun packages/create/bin/create.ts my-dive --archetype roguelike --name "My Dive"   # in the engine repository
cd my-dive && bun install && og test . && og run .
```

It copies the directory (without `cooked/`, `dist/`, `og-out/` and
`.og/`), sets `name` and `id` in `project.ogt` and `package.json`, and
writes the archetype's `gitattributes` (media through Git LFS) as
`.gitattributes`. It gives the game its look, `assets/looks/<id>.ogt`
named by `GameDesc.look` (the archetype's own carried over, or one that
sets nothing), and `--look <slug>` or `--premise "<line>"` makes it extend
a catalogue preset: the archetype's reference frames then no longer match,
and create says to record them again (`packages/create/README.md`).
