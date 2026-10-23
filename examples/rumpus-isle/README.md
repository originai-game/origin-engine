# Rumpus Isle

The sixth dogfood game of the Origin Engine build-out, written from the
knowledge package alone (`packages/knowledge`: the skills, the wiki, `og
--help`): a local-multiplayer party game for two to four on one screen.
Round critters on a little island bump each other about in three short
minigames, with a scoreboard after each and a podium at the end. What the
knowledge package got wrong or left out, and what the engine did not do,
is `FINDINGS.md`.

```sh
bun tools/make-assets.ts   # every asset (the music, a splash and the copied style's art are not committed)
og cook .
og run .                   # headless with the probe; og dev . reloads on save
og test .                  # nine playtests, the frames against references/<tier>
og check .
og polish .                # the taste and feel scores of the playtests
og build . --publish       # the web build
```

## Controls

Up to four players: any of the four pads, and two halves of one keyboard.

| Action | WASD half | Arrows half | Pad |
|---|---|---|---|
| Move | W, A, S, D | the arrows | left stick, d-pad |
| Sit down in the lobby, dash in a round | Space | Enter | A |
| Stand up (lobby) | Q | Backspace | B |
| Pick a colour (lobby) | A, D | Left, Right | its own focus ring (`og.ui.setPlayers`) |
| Pause | Escape | Escape | Start (any seated pad) |
| Menus | arrows, Tab, Enter, Escape | | d-pad, A, B |

The camera looks down the island from the south, so up on the screen is
away from the camera.

## The rounds

A match is three rounds (`src/rules.ts`), each a card with how it is
played, a count-in, play, a beat with a banner naming the winner, then the
scoreboard; places give points (3, 2, 1, 0) and the podium follows the
third. New players can sit down between rounds and join from the next.

1. **Bump Ring** (40 s): shove the others into the sea; a dash (A, Space,
   Enter) hits harder and stuns. The last one on the island wins; at the
   whistle those still on are ranked by how near the middle they stand.
2. **Berry Rush** (30 s): berries come up on the island, up to six at a
   time, a gold one worth three; the most when the whistle goes wins. A dip
   in the sea costs a berry and a moment off the island.
3. **Crumble Tiles** (45 s): the island sinks away under a grid of tiles;
   a tile stood on cracks and drops after a moment. The last one standing
   wins, and the longest stood is the isle's record.

## What it uses

| Part | Where | Skill |
|---|---|---|
| Dynamic sphere critters (rotation locked but about +y), pushed towards the speed their player asks, a dash, bumps answered from `og.CollisionStarted` with an impulse by how fast they closed, a stun for a dash hit | `src/critters.ts`, `src/arena.ts` | og-game §13, og-feel §12 |
| Six input sources on one world: four `og.Gamepad` pads and two halves of `og.Input`'s keyboard, each a seat | `src/input.ts`, `src/lobby.ts`, `src/seats.ts` | og-game §7, og-ui §4 |
| A kinematic island that sinks for the crumble round, kinematic tiles that shake and drop, the podium's steps as ground, `og.Water` with an `og.WaterSurface` the critters splash | `src/arena.ts`, `src/rules.ts` | og-game §13 |
| Three glTF models from a script (the island, a tile, a berry, the podium, palms), `og.load.gltf` and `world.instantiate` | `tools/make-assets.ts`, `tools/glb.ts`, `src/arena.ts` | og-game §6 |
| Juice: hit stop by impact weight, trauma shake, squash on a bump, one-shot emitters (splash, dust, sparkle, crumbs, confetti) restarted by `startTick`, slow motion on the last knock | `src/critters.ts`, `src/feel.ts`, `assets/fx/` | og-feel §3–§6, og-game §10 |
| Name tags as screen text over each critter, placed by `og.Camera.worldToScreen` | `src/feel.ts` | og-ui §3 |
| The title (`og.ui.mainMenu`), a lobby from widgets with a focus ring per pad (`og.ui.setPlayers`), a HUD from widgets, the round's card, `og.ui.pauseMenu`, `og.ui.settingsScreen`, the scoreboard and podium, every screen change through an iris (`og.ui.screenTransition`), count-ups and bursts | `src/ui.ts` | og-ui §2–§6 |
| English and French string tables with plurals and selects, the settings' words from them | `assets/strings/`, `src/ui.ts`, `src/prefs.ts` | og-i18n |
| og sfx presets on voices, a splash from a script, the music streamed in chunks on its bus and ducked under the effects, a limiter on the master | `src/sound.ts`, `assets/audio/`, `assets/music/romp.wav.ogt` | og-audio |
| og.platform: the settings (read on the first tick, saved when the settings close) and the isle's records written at the revision read, merged on a conflict | `src/save.ts`, `src/prefs.ts` | og-platform-sdk, og-game §17 |
| A look extending a catalogue preset, with the cartoon style pack copied and recoloured | `assets/looks/rumpus-isle.ogt`, `assets/styles/rumpus/` | og-taste §1, §12 |

## The look

`assets/looks/rumpus-isle.ogt` extends the catalogue's `miniature-diorama`
(og-taste §12: a tabletop model, tilt-shift focus, a soft sun, a high
camera), with the depth of field focused on the island from the camera's
19 m, and the cartoon style recoloured sea and coral (`styles/rumpus`, copied
by the generator). `create --premise` matched `rubber-hose-animation`, a 2D
flipbook style, for the premise (`FINDINGS.md` #1).

## Playtests

`tests/`: `frames.og.ts` (eleven frames against `references/<tier>/`),
`replay.og.ts` (a match by the keys from the title through two rounds,
twice, one hash), `lobby.og.ts`, `bump.og.ts`, `berry.og.ts`,
`crumble.og.ts`, `flow.og.ts` (a whole match for three with pause),
`save.og.ts` (a relaunch over a kept store), `locales.og.ts` (French on
every screen). `OG_FX_CAPTURE=1 og test .` records this tier's references
(`metal` is the one kept); with no renderer (`OG_RENDERER=none`, the
switch for a sandbox whose only adapter is Mesa lavapipe, which renders
every tick at a quarter of a second: `FINDINGS.md` #7) the frames are
reported unchecked and the rest runs the same.

All nine suites pass on Metal (239 checks, 2026-10-22). All eleven
reference frames were inspected on the owner's Mac and reproduced at
RMSE 0. The scoreboard capture waits until its iris and counters settle.
With this branch's CLI, `og test . --filter tests/lobby.og.ts` runs just the
lobby suite while iterating; run without the filter before delivery.

`og polish .` (2026-10-22, `OG_RENDERER=none`): taste 100, feel 93. The feel row
below 100 is sound on input (71: a stick held is an input every tick, and
only a dash and a bump sound); hit feedback is 100, though its fixes table
still asks for a floating number on every bump, and a party game's bumps
are not numbers (`FINDINGS.md` #13).

Asset licences: `LICENSES.md`.
