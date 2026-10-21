# Brasswick

The fifth dogfood game of the Origin Engine build-out, written from the
knowledge package alone (`packages/knowledge`: the skills, the wiki, `og
--help`): a short 2D puzzle-platformer in a lamplit clockwork vault. Wick,
a small brass automaton with a candle for a wick, climbs out through three
levels. What the knowledge package got wrong or left out, and what the
engine did not do, is `FINDINGS.md`.

```sh
bun tools/make-assets.ts   # every asset (the music and the copied skin are not committed)
og cook .
og run .                   # headless with the probe; og dev . reloads on save
og test .                  # eight playtests, the frames against references/metal
bun run typecheck
og build . --publish       # the web build
```

## Controls

| Action | Keyboard | Pad |
|---|---|---|
| Run | Arrows or A, D | left stick, d-pad |
| Jump (wall jump on a wall) | Space, Z, Up or W | A |
| Drop through a grate | Down or S | d-pad down |
| Back to the last lamp | R | Y |
| Pause | Escape | Start |
| Menus | arrows, Tab, Enter, Escape | d-pad, A, B |

On a phone the HUD shows a d-pad and Jump.

## The levels

`tools/levels.ts` builds each as a character grid (a few rectangles and the
things placed in them); `tools/make-assets.ts` writes them as tilemaps in
the engine's own `type tilemap` grammar (`assets/maps/level-<n>.ogt`), and
the game reads every position from the maps' objects (`src/level.ts`). All
three are in the world at once, side by side in x, each inside its
`og.CameraZone`.

1. **The Boiler Stair**: a spike pit, a crate shoved against a ledge as a
   step, a lamp, a crate onto a plate that holds a door open, a hop over
   it, and a wall-jump shaft to the hatch.
2. **Counterweights**: grates, a crate shoved off a slab onto its plate,
   two ticks (one walks the second crate's floor), a crate dropped into a
   pit onto its plate.
3. **The Lantern Gate**: dark; a lift over spikes, a gate on a prismatic
   joint whose position motor slides it up while its plate is held, a long
   wall-jump climb, a tick on the top shelf.

## What it uses

| Part | Where | Skill |
|---|---|---|
| A tileset with a blob47 terrain, three maps in the `type tilemap` grammar: far pipes (`lit false`) and mid girders repeating behind with parallax, the back wall, the plate with collision, grates, decor | `tools/make-assets.ts`, `assets/tiles/vault.ogt`, `assets/maps/` | og-game §8, og-feel §11 |
| og.Lighting2D over the lit map layers, og.Light2D on Wick, the lamps, the wall lamps and the hatch, crates casting shadows, the 2D glow | `src/level.ts`, `src/main.ts`, `assets/looks/brasswick.ogt` | og-game §10, og-taste §8 |
| og.PlatformerController: run, jump, coyote time, a buffer, wall slide and wall jumps, one-way grates, riding the lift, shoving crates | `src/hero.ts`, `src/level.ts` | og-game §13, og-feel §12 |
| Crates (dynamic boxes), pressure plates (`og.physics.overlap`), doors (kinematic bodies moved by og.Velocity), the gate (og.Joint, `JOINT_PRISMATIC`, `MOTOR_POSITION`), the lift (kinematic) | `src/puzzle.ts` | og-game §13 |
| Flipbooks: Wick (idle, run, jump, fall, wall, push, hurt, cheer; `step` and `shove` events), ticks, gears, plates, lamps, the hatch | `assets/sprites/*.ogt`, `src/hero.ts`, `src/actors.ts` | og-game §8, og-feel §9 |
| Juice: hit stop, shake, squash and stretch, floating text, one-shot emitters restarted by `startTick` | `src/hero.ts`, `src/actors.ts`, `src/feel.ts` | og-feel §3–§7 |
| The title, level select, pause, settings and results templates; a HUD from widgets | `src/ui.ts` | og-ui §2–§6 |
| English and Spanish string tables, the settings' words from them | `assets/strings/`, `src/ui.ts` | og-i18n |
| og sfx presets on voices, the music streamed in chunks on its bus, ducked under the effects | `src/sound.ts`, `assets/audio/`, `assets/music/vault.wav.ogt` | og-audio |
| og.platform: the settings (loaded on the first tick, saved on close), the progress record at its revision, the whole world in a slot at each lamp, Continue restoring it | `src/save.ts`, `src/prefs.ts` | og-platform-sdk, og-game §17 |
| Keyboard and pad 0 | `src/hero.ts`, `src/ui.ts` | og-game §7, og-ui §4 |

## The look

`assets/looks/brasswick.ogt` extends the catalogue's `detailed-pixel`
(og-taste §12) at 360 lines (2x at 1280×720, 3x at 1920×1080: the preset's
270 leave a border at 720p), 24 tones a channel and a lighter dither, with
the fantasy skin recoloured brass (`skins/brasswick`, copied by the
generator) and the 2D glow.

## Playtests

`tests/`: `frames.og.ts` (nine frames against `references/metal/`),
`replay.og.ts` (the first level played by keys alone, twice, one hash),
`move.og.ts`, `puzzle.og.ts`, `actors.og.ts`, `save.og.ts` (a relaunch over
a kept store), `locales.og.ts` (Spanish on every screen), `audio.og.ts`.
`OG_FX_CAPTURE=1 og test .` records this tier's references.

Asset licences: `LICENSES.md`.
