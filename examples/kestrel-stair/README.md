# The Kestrel Stair

A small third-person action-adventure on Origin Engine, about five minutes
of play, written the way a game author would write it: from the knowledge
package's skills and wiki and the generated types, with nothing but
TypeScript, generated assets and `og`. It is the third dogfood game (after
the 3D top-down [`lantern-vale`](../lantern-vale) and the 2D platformer
[`hollowbell`](../hollowbell)), written to exercise what those two did not:
a 3D character controller over a glTF level, navmesh minds, retargeted
skinned characters, cloth and rope, spatial audio and the UI templates
neither used. What it found is [`FINDINGS.md`](FINDINGS.md).

The afternoon is going. Ilse the courier carries the Abbess's sealed
letter up the Kestrel Stair, a ruined hillside monastery, to Oswin the
bell-keeper on the summit, and must ring his bell before nightfall at
19:30 (five minutes of play). The stair is overrun: thorn imps patrol the
lower court, stone wardens the terraces. Ilse walks and runs (WASD or the
arrows, Shift; the left stick), jumps (Space, A), strikes with her sword
(J or the left button, X), uses what is in reach (F, Y): three sigils
raise the summit gate (one on the crypt's altar, one atop the ledges east
of the crypt terrace, one in the chest of the terrace's warden), hillwort
restores strength (H, or Use in the bag), waystones restore it too and
keep the climb (they are the save slot and the chapters' starts). Q and E
(the right stick) turn the view; Tab (I, the pad's Back) opens the bag;
Esc pauses. Imps chase, wind up and bite, and flee when hurt; wardens see
from farther, close in slowly and slam with a long telegraphed wind-up and
a lunge. Strength gone, or night, ends the climb; the bell ends it well.

## Run it

```sh
bun tools/make-assets.ts        # once after a clone: every asset, and the 29 MB music (not committed)
bun tools/fetch-font.ts         # only to change a Japanese string: Noto Sans JP's source (4.5 MB, not committed)
python3 tools/subset-font.py tools/fonts/NotoSansJP-Regular.otf assets/fonts/NotoSansJP-kestrel.otf --locale ja --cjk-only
og cook .
og run . --headless --commands "1:screenshot:title.png;1.1:play:0:quick;1.5:key_down:KeyW;3:key_up:KeyW;3.5:screenshot:court.png;3.6:quit"
og test .                       # the thirteen playtests, the reference frames against references/<tier>/
og polish .                     # taste 99, feel 97 on metal (og-out/polish/report.md)
OG_FX_CAPTURE=1 og test .       # records this tier's reference frames
bun run typecheck               # tsc over src, tests and tools
```

Inside the engine repository `og` is `target/debug/og` (`cargo build -p
og-cli`), and the playtests load the dev addon at
`packages/engine/native/origin-engine.<platform>.node` (`cargo build -p
og-host-node`, copied as `packages/engine/scripts/build.sh` does).
`bun tools/make-assets.ts --no-music` remakes everything but the music.
Every asset is made by code, byte for byte the same on every run (two runs
of the script hash the same): the level, the characters, the clip library,
the textures, the sky, the particles, the sounds, the music, the
behaviours and the navmesh file; the skin is the engine's paper pack,
copied and retinted by the same script. The one thing fetched is the
Japanese font's source (`tools/fetch-font.ts`, checked against its
SHA-256), from which the committed 79 KB subset is cut.

## What it exercises

| Area | How | Where |
|---|---|---|
| A glTF level | `models/stair.glb` written by `tools/level.ts` from the shared table `src/layout.ts`: the lower court, a broad ramp (18°), a steep path (34°) and a scree (63°), the crypt terrace with the crypt (walls, a door, a roof, pillars, an altar) and five stone ledges each a jump higher, sixteen stairs to the high terrace, the gate wall, sixteen more to the summit; textured paving, stone, grass, plaster and wood drawn by code; a triangle-mesh collider per primitive | `tools/level.ts`, `src/world.ts` |
| `og.CharacterController` | the courier's capsule: walking 3.2 and running 6.2 m/s relative to the view, slopes to 45°, steps to 0.32 m, ground snap, coyote time, the jump; `og.CharacterEvent` for jumps and landings (sound, squash, a shake on a hard landing) | `src/hero.ts`, `tests/move.og.ts` |
| Skinned characters, one clip library, retargeting | `anims/humanoid.glb` (a T-posed skeleton with Blender names, no mesh; idle, walk, run, attack, hurt, die, jump, bite, slam, with `hit` markers in `extras."og.events"`); the courier (Mixamo names, an A-pose) gets them at cook time (`animation { library }`); the imp (Unreal names, detected) and the warden (its own names, a `type rig` file) at run time with `og.anim.retarget` | `tools/rig.ts`, `src/anim.ts` |
| Blend trees, one-shots, root motion | an `og.AnimationGraph` per character (a 1D blend of idle, walk and run by speed; the courier's air state), an `og.AnimationPlayer` over it at weight 1 for strikes, bites, slams, hurts and falls; strikes, bites and slams land on the clips' `hit` markers (`og.AnimationEvent`); the warden's slam lunges 0.9 m by root motion (`Skeleton.rootMotion` planar while it plays) | `src/hero.ts`, `src/foes.ts`, `tests/anim.og.ts`, `tests/foes.og.ts` |
| Navmesh and minds | `nav/stair.ogt` baked by `og cook` from the level model; the summit gate an obstacle (`og.nav.setObstacle`) lifted when it rises; five agents with `og.Steering` separation and collider avoidance; the imps' behaviour tree (patrol, chase, a 0.25 s wind-up and `bite`, flee when hurt) and the wardens' state machine (patrol, `alert`, chase, `slam`, search), their commands handled by the game; `og.Perception` sight cones the level and the gate occlude | `tools/level.ts`, `assets/ai/`, `src/foes.ts` |
| Cloth and rope | the courier's cape (`og.Cloth`, pinned at the shoulders, trailing as she runs), four banners in a gusting wind; the crypt's chain (`og.Rope`) with its lamp hung at the last link (`og.physics.ropePoints`) | `src/hero.ts`, `src/world.ts`, `src/fx.ts` |
| Spatial audio, a reverb zone, streamed music | the braziers' crackle, the imps' chitter, the wardens' slams, the crypt's drip and the bell are spatial sources heard from the listener on the camera (distance, pan, `distanceCutoff`, `occlude` behind the crypt's walls); the crypt is a reverb zone the game drives (the effects bus's send to the shared reverb, the room size, the sky's light dimmed); "The Kestrel Stair" (2:40, 48 kHz stereo) cooked in 4-second chunks and streamed on the music bus, ducked under the effects; a limiter on the master | `src/sound.ts`, `src/fx.ts`, `tests/audio.og.ts` |
| A painterly look and a day | `looks/kestrel.ogt` extends `watercolor-wash` (pigment pooling, bleed, cold-pressed paper laid on the surfaces), with AgX, bloom, a violet shadow tint, a pale fog and the synthetic sky; `og.DayNight` held at the run's hour from 16:30 to 19:30; `looks/summit.ogt` extends `luminous-layered-painting` (depth bands down the stair) for the ending and the results | `assets/looks/`, `src/world.ts`, `src/ui.ts` |
| og.ui templates on the paper pack | `mainMenu` (Continue names the slot), `cutscene` (the intro and the ending), `tutorialOverlay` (strength, sigils, clock, bag), `inventoryScreen` (the bag: drag to move, Use to eat), `levelSelect` (the chapters, locked until reached, stars), `leaderboardScreen` (fastest and around you), `pauseMenu`, `settingsScreen` (every word from the tables), `resultsScreen`; a HUD of capsules and a meter; a wipe between screens; toasts at the top | `src/ui.ts` |
| Two languages | `strings/en.ogt` and `strings/ja.ogt`; Noto Sans JP subset to the Japanese table by `tools/subset-font.py` (CJK only, so key names in a Japanese line stay in the skin's Latin face), the skin's fallback | `assets/strings/`, `tools/` |
| Saves | resting at a waystone saves the whole world (`world.snapshot()`, about 78 KB, through `og.platform.saveSlot`: two parts of a key's 64 KiB and the manifest; FINDINGS #13, fixed) beside the slot's place and hour; Continue restores it (`world.restore`); the progress (chapters, stars, the tutorial, the best) at the revision read, a conflict merged; the settings through `saveSettings` | `src/save.ts`, `tests/save.og.ts` |
| The leaderboard | a won ascent from chapter I goes on the `ascent` board (`order: "asc"`, seconds); the Records screen reads the top ten or the ten around the courier | `src/save.ts`, `tests/win.og.ts` |
| Feel | strikes: hit stop by weight (4 ticks on an imp, 7 on a warden, 14 on a warden's fall), a shake and a kick, the foe's flinch (a squash: FINDINGS #11), sparks, a number, the sound; a light aim towards the nearest foe; strikes buffered a fifth of a second; hurts: a red flash, a knock-back, the hurt clip, a number; telegraphed enemy attacks | `src/hero.ts`, `src/foes.ts` |
| Accessibility | `og.ui.a11y()` counted by the Ui system; the locales playtest requires no focusable control unnamed on five screens | `src/ui.ts`, `tests/locales.og.ts` |

## Playtests

| File | Proves |
|---|---|
| `move.og.ts` | walk and run speeds, the ramp's surface, the steep path climbed, the scree refused, the stairs to the high terrace, a jump onto a ledge with its sounds |
| `foes.og.ts` | an imp's patrol, sight, navmesh chase, bite and flight; a warden's alert, its slam's root-motion lunge and its hit |
| `combat.og.ts` | a swing on its press tick, the hit on the clip's marker with hit stop, a flinch, a shake, a number and sounds; a warden felled, its sigil dropped and taken; a slam's hurt |
| `anim.og.ts` | three 21-joint skeletons on retargeted clips through blend trees, the air state; the cape hanging and trailing, a banner stirring, the lamp on its chain |
| `audio.og.ts` | the music chunked, looping and unclipped; a brazier near and far (gain, pan, distance dulling); the drip occluded and clear; the crypt's reverb zone |
| `story.og.ts` | the intro walked with Enter, the tutorial's four steps, the progress keeping it seen |
| `bag.og.ts` | Tab, a drag between slots, an item's details, Use, H, the error sound, the bag from pause |
| `save.og.ts` | a rest saves the world in parts; a relaunch (`store: "keep"`) continues it by that waystone, focused Continue, the welcome, chapter II unlocked |
| `win.og.ts` | the gate refused and raised, the bell, the ending under the summit look, the results, the leaderboard entry and the Records screen, Again |
| `lose.og.ts` | night at 19:30 and its results (no board entry); strength gone, the fall and its results; Title |
| `locales.og.ts` | the title, intro, tutorial, HUD, bag, pause, settings and results in Japanese, then English; the accessibility tree on five screens |
| `replay.og.ts` | a session of chases, bites, strikes, a slam, a rest, the gate and the bell, run twice in fresh processes, ends on one world hash |
| `frames.og.ts` | nine reference frames at 1280 x 720 (`references/metal/`): the title, the intro, the court, a strike, the crypt, the bag, the chapters, the records, the results |

`og polish .` on metal (dev profile, a machine shared with other builds,
on the runtime-ui-2 branch of wave 11): taste 99 (contrast 98: the bag's
stack count, 3.4 to 4.0:1 in the paper pack's ink, is the one text under
4.5:1; "You" on the records row reads 9.9:1 now that the ink is measured
over the glyphs, FINDINGS #20 fixed), feel 97 (input latency, hit
feedback and idle frames 100; sound on input 93; frame time 93: step
about 4.8 ms median and 6.0–7.0 ms p95, render about 2.2 ms, frame p95
8.5–9.7 ms, with the systems writing the Game resource through its live
handle since FINDINGS #19 was fixed).

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `play[:<chapter>[:quick]]` | a run from chapter 0 to 3 now, as New journey or Chapters does; `quick` skips the story and the tutorial |
| `warp:<place>[:<dx>:<dz>]` / `warp:<x>:<z>[:<y>]` | the courier at a named place (`waystone-N`, `sigil-N`, `herb-N`, `crypt`, `ledge-N`, `gate`, `bell`, a foe's name) or on the ground at x, z |
| `give:<sigils\|herbs>:<n>`, `hp:<n>`, `dusk:<seconds>` | progress, strength (a loss lands as a hurt), the run's clock |
| `rest:<waystone>`, `ring` | rest there now (the save), raise the gate and ring the bell |
| `language:<en\|ja>`, `open:<settings\|bag\|chapters\|records>` | the language, a screen |
| `bite`, `alert`, `slam` | the foes' behaviour commands (og.ai sends the foe's entity; a foe index works too) |

`facts` report the phase, ending, chapter, strength, hits, kills, hurts,
sigils, hillwort, the clock, what is in reach, the courier's position,
the gate, the crypt zone, the look, the saves and loads, the best, the
score and rank, the progress, the language, the accessibility counts and
each foe's state.

## Layout

`src/layout.ts` the level's numbers (shared with `tools/`);
`src/components.ts` the state; `src/state.ts` the Game resource as a
system's copy; `src/world.ts` the stair; `src/anim.ts` the rigs and blend
trees; `src/hero.ts` the courier; `src/foes.ts` the imps and wardens;
`src/pickups.ts` sigils, hillwort, waystones, the gate and the bell;
`src/view.ts` the camera and the listener; `src/fx.ts` the lamp, the
braziers, the cloth measures and the crypt's zone; `src/sound.ts` the
voices and the music; `src/ui.ts` the screens and the clock; `src/save.ts`
the platform; `src/main.ts` the game, its commands and facts. Systems in
tick order: `og.systems.cameras`, Ui, Hero, Foes, Pickups, View, Fx,
Voices.

## The climb

| Chapter | Holds |
|---|---|
| I · The Lower Court | the start, two imps, hillwort, a waystone; the ramp, the steep path and the scree up to the crypt terrace |
| II · The Crypt Terrace | the crypt (the first sigil on its altar, the chain and lamp, the drip, the reverb), the terrace's warden (the third sigil), the ledges (the second sigil on the last), a waystone, the stairs |
| III · The High Terrace | an imp, a warden before the gate, braziers and banners, a waystone |
| IV · The Summit Gate | the gate, the summit stair, the bell |
