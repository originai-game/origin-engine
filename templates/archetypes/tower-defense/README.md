# Path Guard — the tower-defense archetype (2D)

A night garden. Visitors (beetles, moths, snails) come through the gate in
the hedge and walk to the lantern on the doorstep; you set plants on the
lawn to turn them away. Thorns shoot, frost lilies slow, hedges only block,
and every plant re-routes the visitors, who always take the shortest way
round what stands on the lawn. Five waves; the lantern's light is gone
after ten visitors get through.

小径守卫：夜里的花园，甲虫、飞蛾和蜗牛从树篱的门进来，走向门口台阶上的灯笼。在草地上种植物把它们劝走：荆棘射击，霜百合减速，树篱只挡路；每种下一株，来客都会绕开它重新找最短的路。五波来客，有十个走到门口，灯就灭了。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests and
`og polish .` scores them. It rebuilds the platform's retired Discover
prototype *Bloomguard* (花园守夜人) on the engine: its night garden, its
three plant roles (one shoots, one supports, one walls) and its lesson that
a kill's time must scale with the walk's, but as a mazing defence on a
flow field rather than lanes, and none of its code or art.

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.3:begin;0.5:plant:thorn:3:4;0.5:plant:hedge:4:4;6:screenshot:wave.png;6.1:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og polish .                  # the playtests scored against og-taste and og-feel
og build . --publish         # dist/: the play page as it ships
```

## What it demonstrates, and the skills it exercises

- **Crowds on a flow field** (og-ai §1, §2): one `og.nav.flowField` over a
  grid made from the map's colliding layer (`og.nav.gridFromTilemap`)
  serves every visitor; each `og.NavAgent` follows it to the doorstep with
  `faceMovement` turning the sprite. A plant is `og.nav.setCost(…, 0)` on
  its cell and the field rebuilds; a plant that would close the path for
  the gate or any visitor on the lawn is found with `og.nav.findPath` and
  refused before it costs anything.
- **Targeting as data** (og-ai §3, §4): a shooting plant is an `og.AiAgent`
  running `assets/ai/plant.ogt` (set, reloaded, sees a visitor → the
  game's `fire` command) with an `og.Perception` of its range all round;
  the visitors are `og.Perceivable`. Damage, frost and the shot stay in the
  game (`fire` in `src/systems.ts`).
- **Towers on a tilemap** (og-game §8): `assets/maps/garden.ogt` is a
  hand-written tilemap whose `walls` layer collides; the `pads` layer starts
  empty and `og.tilemap.setTile` writes the plant's pad on its cell, so the
  map itself remembers what grows where.
- **Waves and an economy**: five waves of visitors in data (`WAVES`), a
  break between them that an early call (N, Y, or the Next wave button)
  turns into coins, bounties, a wave bonus, prices in `Tuning`.
- **Shots on tweens** (og-feel §6): a shot is a pooled sprite on an
  `og.tween.sequence` whose `tag` names the target's pool slot and the
  plant's kind; the hit lands on the tick `og.TweenCompleted` reports it.
- **Feel** (og-feel §3–§7): a pop is a coin count (`og.render.floatingText`),
  a burst of petals (`og.ParticleSystem` with `startTick`), a pitch-varied
  sound and a light hit stop at most every 10 ticks; a visitor at the
  doorstep is a red number, a 6-tick hit stop, trauma and a kick, the
  lantern flashing red and squashing. Plants pop up when set and squash
  when they shoot.
- **Screens on og.ui with a style** (og-ui §2–§6, og-taste §1–§7): the
  engine's cartoon pack recoloured as `assets/styles/pathguard` (cream
  panels on the night lawn, the lantern's orange as the one accent); the
  title (`og.ui.mainMenu`), the HUD as boxes in the corners with
  counting numbers (`og.ui.counter`), the plant tray (right on a wide
  screen, along the bottom on a phone), `og.ui.pauseMenu`, a settings
  dialog and the end card built from widgets (stars landing with
  `og.ui.burst`, the score counting up), an iris between screens (a fade
  with reduced motion), a refused plant shaking the purse with the error
  sound (`og.ui.shake`).
- **Sound** (og-audio §1–§5): a voice per event, the UI's sounds from
  `og sfx` presets, a night music loop on the music bus ducked under the
  effects (`og.AudioBusFx`).
- **Languages** (og-i18n §1–§5): every string is a key of
  `assets/strings/en.ogt` and `zh-CN.ogt`; the settings switch with
  `og.i18n.setLocale`; the Chinese font is Noto Sans SC subset to exactly
  the table's characters.
- **The platform** (og-platform-sdk §1–§4): settings through
  `loadSettings`/`saveSettings`, the best score read on tick 0 and written
  at the revision read, the board and a `round.over` event.

`og polish .` (2026-10-19, og and the native addon built release from
this checkout, load 16–18): taste 99, feel 100, in
`tests/evidence/P2/archetypes/tower-defense/` of the engine repository.
The same playtests at load 25–50 scored feel 80–89, frame time's p95
following the machine's load (`tests/evidence/P2/archetypes/frame-time.md`).

## Rules

- **Plants.** Pick one on the tray (or 1, 2, 3; the pad's X cycles), then
  tap a lawn cell (or move the cursor with the arrows, WASD or the d-pad
  and press Space, Enter or A). It costs its price at once and stands
  until the round ends.

  | Plant | Price | Does |
  |---|---|---|
  | Thorn | 50 | shoots the nearest visitor within 2.8 m: 14 every 30 ticks |
  | Frost lily | 70 | shoots the nearest within 2.2 m: 5 every 60 ticks, and frosts it to half speed for 2 s |
  | Hedge | 10 | blocks the cell: the maze's wall |

- **Refusals.** A cell inside the hedge that is not a stone and holds no
  plant and no visitor takes a plant, if the purse covers it and the gate
  and every visitor can still reach the doorstep. Otherwise nothing is
  spent; a red line says why and the purse shakes.
- **Visitors.** They walk the flow field to the doorstep. Beetles (34 hp,
  1.3 m/s, 6 coins), moths (22 hp, 2.1 m/s, 5 coins), snails (140 hp,
  0.75 m/s, 20 coins, three lights). Each wave's are 20 % tougher than the
  last's.
- **Waves.** The first comes 3 s after Play; then a wave comes after the
  last one's visitors are all gone and an 8 s break. Calling it early pays
  2 coins a second saved. A wave cleared pays 20 + 5 × its number.
- **The end.** Ten lights gone ends the round lost; the fifth wave cleared
  ends it won, with one to three stars by the light left. The score is
  kills × 10 + light × 50 + coins left.

Everything is deterministic from the project seed: the clock is the fixed
tick, og.nav, og.ai, tweens and particles are deterministic and in
snapshots, and `tests/replay.og.ts` replays a round to the same hash.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| a plant set | it pops up (squash 0.6 × 1.5 over 0.25 s), a wooden thunk on the ui bus | `Plants`, `place` |
| a refusal | a red line over the cell, the purse shakes with the error sound | `place`, `hud` |
| a shot | the plant's shooting frame and a 1.15 × 0.85 squash, a pitch-varied pluck or chime | `fire` |
| a hit | the visitor flashes white for 0.08 s; frost tints it blue and slows its legs | `Visitors` |
| a pop | `+6` in gold rising, a burst of petals, a pop, a 2-tick hit stop (at most one per 10 ticks) | `pop` |
| a leak | `-1` in red at the lantern, a 6-tick hit stop, trauma 0.4 and a kick down, the lantern flashes red and squashes, a low bell | `leak` |
| a wave | `Wave 3` over the gate, a horn; cleared: `Wave cleared +35` and an arpeggio | `startWave`, `Visitors` |
| the end | an iris to the end card, stars landing one by one with bursts, the score counting up | `endCard` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase, the light, the purse, the wave clock, the counts (kills, leaks, plants, refusals, shots), the cursor, the score and the best, the cells visitors stand on, sound requests, the grid, field and map handles |
| `Tuning` | resource | the knobs below |
| `Prefs` | resource | the volumes, the language, reduced motion, the settings request |
| `Bed` | one row per lawn cell | whether a plant stands there, its kind, its reload, its shots |
| `Visitor` | the 24 pooled visitors | slot, kind, health, speed, frost, the pop |
| `Shot` | the 24 pooled shots | the tween carrying it |
| `Cursor` | the cursor | its moves |
| `Sfx` | one row per sound | its `Sound.*` bit, its plays |

## Systems (in tick order)

| System | Job |
|---|---|
| `Control` | the cursor, the tray keys, placements by key, pad and tap, the next wave, pause; draws the cursor and the range |
| `Plants` | switches on the rows of cells planted this tick (the pads tile names the plant), reloads, sight from `Tuning`, the shooting clip back to the sway; clears the rows for a new round |
| `Visitors` | the wave clock and the gate, shots landing (`og.TweenCompleted`), frost, health bars, pops, leaks, the cells visitors stand on, the end of a wave and of the round |
| `Ui` | the screens, the camera framed for the surface, the buses, the platform's answers |
| `Voices` | plays each requested sound once |

og.ai runs before them (the plants' decisions, whose `fire` commands run
before the next tick) and og.nav after them (walking the visitors).

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `startGold`, `startLives` | 150, 10 | the purse and the light at Play |
| `waveBreak`, `spawnGap` | 8 s, 1 s | the break between waves, the gap between visitors |
| `hpGrowth` | 0.2 | how much tougher each wave's visitors are |
| `thornCost`, `thornDamage`, `thornReload`, `thornRange` | 50, 14, 30 ticks, 2.8 m | the thorn |
| `frostCost`, `frostDamage`, `frostReload`, `frostRange`, `frostSlow`, `frostTicks` | 70, 5, 60 ticks, 2.2 m, 0.5, 120 ticks | the frost lily |
| `hedgeCost` | 10 | the hedge |
| `shotSpeed` | 11 m/s | how fast shots fly (so how far they lead their targets) |
| `killStop`, `leakStop`, `leakShake` | 2, 6 ticks, 0.4 | the weight of a pop and of a leak |

The visitors' numbers are `KINDS` and the waves `WAVES` in `src/systems.ts`.

## What a Maker changes first

1. **The waves**: `WAVES` (one letter per visitor: b, m, s) and `KINDS`.
2. **The plants**: their numbers in `Tuning`; a fourth plant is a clip in
   `tools/make-assets.py`, a pad tile, a `PLANTS` entry and a case in
   `fire`.
3. **The field**: `MAP` in `tools/make-assets.py` (H hedge, R stone, G the
   gate, D the doorstep), then `python3 tools/make-assets.py && og cook .`.
4. **The look**: `assets/styles/pathguard/style.ogt`'s palette and the art
   in `tools/make-assets.py`.
5. **Words**: `assets/strings/*.ogt`; a new Chinese string needs the
   script run again to subset the font.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `begin` | starts a round, as Play does |
| `plant:<thorn\|frost\|hedge>:<x>:<y>` | sets a plant on a map cell (column, row from the top) as a placement does, refusals included |
| `gold:<n>` | sets the purse |
| `wave:<n>` | skips to the break before wave n |
| `lose`, `win` | puts the lantern out; ends the round won |
| `viewport:<desktop\|phone>` | lays the UI out for a phone in portrait (390 × 844 at 3×) |
| `language:<en\|zh-CN>` | switches the language |
| `fire:<plant>` | the plants' behaviour command (og.ai sends it) |
| core actions | `touch_move_to:play`, `Plant#1/Pick`, `Next`, `Pause`, `Retry`, `Menu`; keys `1`–`3`, arrows, `Space`, `N`, `Escape`; `gamepad:a` … |

`facts` (the probe's `state()`) reports the phase, the wave, the light
(`lives`), the purse, `kills`, `leaks`, `placed`, `refused` and why, `shots`,
the visitors walking, the selected plant, the cursor, the score and the
best, and the language. The playtests read the same values from the `Game`
resource with `get:Game`.

## Playtests (`og test .`)

| File | Proves |
|---|---|
| `tests/flow.og.ts` | Play by tap; a hedge picked on the tray and tapped onto the lawn; two thorns set by keys; the first wave walks the field at a beetle's pace, is shot and pays its bounties |
| `tests/maze.og.ts` | a row of hedges leaves one gap and the first visitor crosses through it only; a hedge closing the gap, a stone, a taken cell and an empty purse are refused and cost nothing |
| `tests/lose.og.ts` | unguarded, visitors reach the doorstep, the light runs out, the end card, Play again starts round two with the light back |
| `tests/win.og.ts` | a stand holds the last wave called early with N; the win, the score, the best written at the revision read, the board, the telemetry, Title |
| `tests/pause.og.ts` | Escape holds the garden (time scale 0); settings switch to 简体中文 and move the music; Done saves through og.platform; Resume |
| `tests/pad.og.ts` | a gamepad alone starts, moves, plants, picks, calls, pauses and resumes |
| `tests/phone.og.ts` | 390 × 844 at 3×: the tray along the bottom with touch-sized buttons inside the safe area, the pause button below the notch |
| `tests/frames.og.ts` | the title, a wave in a small maze and the end card match `references/<tier>/` (`OG_FX_CAPTURE=1 og test .` records them) |
| `tests/replay.og.ts` | a round with taps, keys, shots, frost and pops, run twice, ends on one hash |

Each one fails when the behaviour it names is broken (one was broken on
purpose and restored: `tests/evidence/P2/archetypes/break-and-restore.md`
in the engine repository).

## Assets

`tools/make-assets.py` regenerates every asset byte for byte (two runs
give the same bytes): the tileset, the map, the sprite sheet, the glow and
particle textures and emitters, the sounds and the music, the `og sfx`
preset files of the UI's sounds, and the font subset. `LICENSES.md` lists
them; the style's are in `assets/styles/pathguard/LICENSES.md`.

## Known limits (engine gaps, not design)

- og.ui's `settingsScreen` and `resultsScreen` carried English labels of
  their own ("Master", "Done", "Best", "New best") when this game was
  written, so it builds its settings dialog and its end card from widgets to
  keep every string in its tables (both now take a field for each of their
  words: og-i18n §2).
- A system may only `world.get`/`world.set` the components its query
  declares, and declaring one narrows the query, so a placement is written
  to the map's pads layer and the `Plants` system, which owns the plant
  rows, switches the row on in the same tick.
