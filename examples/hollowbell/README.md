# Hollowbell

A small 2D pixel-art platformer on Origin Engine, about five minutes of
play, written the way a game author would write it: from the knowledge
package's skills and wiki and the generated types, with nothing but
TypeScript, generated assets and `og`. It is the second dogfood game (the
first, the 3D [`lantern-vale`](../lantern-vale), exercised the 3D half);
what it found is [`FINDINGS.md`](FINDINGS.md).

The village bell of Hollowbell has lost its three chimes to the hollow
under the hill. Wren the bell-keeper goes down the old well after them:
runs and jumps (the arrows or A/D, Space or Z, the pad's stick and A)
through the dark caves, drops through planks (Down), swings a bronze
mallet (X or J, the pad's X) at beetles and moths, trades glimmers with
Old Burrow the mole (Up beside him: the talk, then his wares: a honey
cake, a brighter wick, a wing cloak that gives an air jump), rides the
lift over the bramble pit, beats the gloam in its hall for the third
chime, and climbs the root shaft back to the surface to ring the bell.
The lanterns along the way keep the run: lighting one restores the hearts
and saves to the platform's store, and Continue on the title picks the run
up there. Losing every heart wakes Wren by the last lantern. Esc pauses;
the settings set the music and effects, the language (English or
简体中文), reduced motion and high contrast.

## Run it

```sh
bun tools/make-assets.ts        # once after a clone: every asset, and the 14 MB music (not committed)
python3 tools/subset-font.py    # after a string table changes: the Chinese font subset (fontTools)
og cook .
og run . --headless --commands "1:screenshot:title.png;1.1:play;1.6:key_down:ArrowRight;2.4:key_up:ArrowRight;3.5:screenshot:well.png;3.6:quit"
og test .                       # the thirteen playtests, the reference frames against references/<tier>/
og polish .                     # taste 99, feel 92 on metal (og-out/polish/report.md)
OG_FX_CAPTURE=1 og test .       # records this tier's reference frames
```

Inside the engine repository `og` is `target/debug/og` (`cargo build -p
og-cli`), and the playtests load the dev addon at
`packages/engine/native/origin-engine.<platform>.node` (`cargo build -p
og-host-node`, copied as `packages/engine/scripts/build.sh` does).
`bun tools/make-assets.ts --no-music` remakes everything but the music.
Every asset is drawn or synthesised by code, byte for byte the same on
every run: the palette, the tileset, the Tiled map, the flipbook sheets,
the texture arrays, the particles, the sounds and the music.

## What it exercises

| Area | How | Where |
|---|---|---|
| A Tiled map | `maps/hollow.tmj` (96 x 48 cells of 16 px, one cell one metre) over `tiles/hollow.ogt`: far hills on Tiled's `parallaxx`/`parallaxy`, repeating along x (`repeatx`), the cave's back wall, the rock as an auto-tiled blob47 terrain whose solid tiles become merged static colliders, planks, animated torches, and 100-odd objects the game spawns from | `tools/make-assets.ts` `tiledMap`, `src/world.ts` |
| 2D physics | Wren on `og.PlatformerController` (coyote time, jump buffer, variable height, one-way planks on their own collision layer with Down to drop through, an air jump from the cloak, riding the lift); the lift a kinematic body; stones as dynamic bodies the mallet knocks about; enemies as kinematic sensor bodies; the strike an `og.physics.overlap` of the mallet's box on the enemy and stone layers | `src/hero.ts`, `src/props.ts`, `src/enemies.ts` |
| Flipbooks | Wren (idle, run with `step` events, jump, fall, attack with its `hit` event, hurt), the beetle (walk, wind-up, dash, hurt), the moth, the mole, the bell (`toll` events); faced with `flipX`, the run's rate matched to the speed | `sprites/*.ogt`, `src/hero.ts`, `src/enemies.ts` |
| 2D light and shadow | `og.Lighting2D` with the ambient eased per area (daylight in the village, 0.14 in the deep hollow), `og.Light2D` on Wren (wider with the wick), the torches, crystals, lanterns, chimes and the mole's candle; `og.Occluder2D` rocks and `castShadow` enemies; the map's tile layers lit like the sprites (the cave's back wall drawn as it looks in full light, the dark the ambient's), Wren's lantern fading out in daylight (FINDINGS #9, fixed in the engine: the map was tinted and light-pool sprites added over it) | `src/world.ts`, `src/props.ts` |
| Looks, the pixel grid and a palette | `GameDesc.look` applies `looks/hollowbell.ogt` (extends `16bit-pixel` on a 240-line grid, 3x at 720p (FINDINGS #7, fixed), pixel snap, the game's own 32-colour `.hex` palette through a Bayer dither, the 2D glow, a vignette); `og.render.applyLook` swaps in `looks/dawn.ogt` (extends `8bit-pixel` with the engine's `@palettes/handheld4`) for the results | `assets/looks/` |
| Texture arrays | the pickups (glimmers spinning, chimes shimmering, hearts, the lanterns) are world `og.Sprite`s on layers of `sprites/items.ogt`; the HUD's chimes are screen sprites (`og.render.sprite` with `screen`) of the same array; the caves' motes are a particle emitter drawing `fx/motes-array.ogt`'s layers at random | `src/props.ts`, `src/ui.ts`, `fx/motes.ogt` |
| Particles | hit sparks, landing dust, a chime's fountain, the gloam's burst (one-shot emitters moved and restarted by `startTick`), torch embers, cave motes | `fx/*.ogt`, `src/props.ts` `Effects` |
| Feel | strikes: hit stop, a white flash and squash, knock-back, shake and kick, a floating number, sparks and a sound; hurts: a red flash, knock-back, blinking, hit stop; the gloam's fall: a long stop, slow motion for 40 real ticks, a punch; jumps stretch, landings squash and puff; swings buffered; a telegraphed charge | `src/feel.ts`, `src/enemies.ts`, `src/hero.ts` |
| Camera | `og.Follow2D` (dead zone, look-ahead, clamped to the map); the gloam's hall an `og.CameraZone` smaller than the view, so the camera holds the hall | `src/world.ts` |
| og.ui on a skin pack | the pixel pack copied and recoloured as `skins/hollowbell` (pale-gold accent, a 16 px type ramp for DotGothic16); `mainMenu`, a name dialog (`modal`, `textField`), a HUD (heart and glimmer capsules, the chimes, Pause, prompts), `cutscene` for the mole's talk, `shopScreen`, `pauseMenu`, a settings modal (sliders, a language radio, toggles, a mixed-script label), `resultsScreen`; an iris between screens; toasts at the top | `src/ui.ts` |
| i18n | `strings/en.ogt` and `strings/zh-CN.ogt` (plurals, arguments), Noto Sans SC subset to the table's characters as the Chinese table's font and the skin's fallback | `assets/strings/`, `tools/subset-font.py` |
| `str(N)` fields | `Prefs.name` (`str(48)`: the bell-keeper's name, the save slot's), `Game.saveJson` (`str(1024)`: the save Continue restores), `Area.key` | `src/components.ts`, `src/world.ts` |
| Saves | progress written to `og.platform`'s store (`save`, at the revision read; a conflict is read again and written over) by a lantern and by Save and quit, read on tick 0 for Continue; the best score under `best`; settings through `saveSettings` (a record rather than `world.snapshot`, which FINDINGS #1 asked for and the engine now has: `src/save.ts` says why) | `src/save.ts` |
| Audio | `og sfx` presets for every action and the UI, a synthesised bell, pitch varied from `og.Rng`; "Under the Hill" (85 s, 44.1 kHz stereo) cooked in 4-second chunks and streamed on the music bus, ducked under the effects; reverb on the effects, a limiter on the master | `src/sound.ts` |
| Accessibility | `og.ui.a11y()` counted by the game; the settings and name playtests require no unnamed focusable control | `src/ui.ts` |

## Playtests

| File | Proves |
|---|---|
| `move.og.ts` | a press answered on its tick, running, a jump to `jumpHeight` at `timeToApex`, the stretch and dust, three one-way planks of the root climb from below, Down through one, the lift's ride |
| `combat.og.ts` | a strike's hit stop, flash, squash, shake and number; two strikes beat a beetle (glimmers, the kill mask); a touch costs a heart with knock-back and blinking; the sounds |
| `gloam.og.ts` | the hall's camera zone, the gloam waking, its wind-up flashing and its charge, its fall in slow motion and back to 1, the third chime |
| `shop.og.ts` | Up by the mole holds the world; Enter walks the talk; a refused and a bought ware; Escape; the cloak's air jump |
| `save.og.ts` | a lantern lights and saves; Save and quit; a relaunch (`store: "keep"`) offers Continue, names the slot and continues by the lantern with the chime, glimmers and name |
| `settings.og.ts` | Escape pauses (time scale 0); a slider, Chinese, reduced motion, a named accessibility tree, saved through og.platform; Resume |
| `name.og.ts` | New game's dialog: a name typed and composed through an input method into a `str` field, the field named by its label, Begin |
| `win.og.ts` | the bell refuses two chimes and rings with three (tolls from flipbook events); the results under the dawn look with a new best in the store; Play again under the game's look |
| `locales.og.ts` | the title, HUD, pause, talk, shop and results in Chinese, then English |
| `lighting.og.ts` | daylight in the village, the dark hollow, the map lit round the lantern and a torch, brighter than the rock in a frame, the wick's wider light |
| `music.og.ts` | the music cooked in chunks, looping on its bus and heard from the title; the mix unclipped in play |
| `replay.og.ts` | a session with strikes, a touch, the lift, the gloam's slow motion, a save and the bell, run twice in fresh processes, ends on one world hash |
| `frames.og.ts` | nine reference frames at 1280 x 720 (`references/metal/`): title, play, a strike, the talk, the shop, pause, the settings in Chinese, the gloam, the results |

`og polish .` on metal (dev profile, a machine busy with other builds;
at 5cc97a90, before wave 9): taste 99 (contrast 93: the pixel pack's
primary button labels, FINDINGS #20, since fixed in the pack), feel 92
(input latency, hit feedback and idle frames 100; sound on input 88;
frame time 72, step about 7 ms median and 10–12 ms p95, render about
3.3 ms).

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `play` | a new run now, as Begin in the name dialog does |
| `continue` | the saved run, as Continue does |
| `warp:<x>:<y>` / `warp:<entity>[:<dx>[:<dy>]]` | Wren's feet at a point, or by a named entity (`lantern-1`, `mole`, `bell`, `lift`, `gloam`, `chime-2`): 1.5 m left of it unless dx, dy say |
| `give:<chimes\|glimmers\|wick\|cloak>[:<n>]` | progress at once |
| `hearts:<n>` | set the hearts (a loss lands as a touch) |
| `ring`, `talk`, `save` | ring the bell (three chimes), open the mole's talk, write the save now |
| `language:<en\|zh-CN>`, `settings`, `name:<text>` | the language, the settings, the bell-keeper's name |

`facts` report the phase, `hp`, `hits`, `kills`, the chimes and
glimmers, Wren's position, the area, the ambient light, the gloam, the
saves, the score, the look and the accessibility counts.

## Layout

`src/components.ts` the state; `src/world.ts` the hollow built from the
map; `src/hero.ts` Wren (`Control`); `src/enemies.ts` the creatures;
`src/props.ts` pickups, lanterns, the lift, stones, lights, the bell, the
mole, the pit, areas, the sky, effects;
`src/ui.ts` the screens; `src/save.ts` the store; `src/sound.ts` the
voices and music; `src/feel.ts` shared juice; `src/main.ts` the game, its
commands and facts. Systems in tick order: `og.systems.cameras`, Ui,
Control, Enemies, Pickups, Lanterns, Bells, Moles, Lifts, Stones, Hazards,
Areas, Flickers, Backdrop, Effects, Voices.

## The map

| Area | Holds |
|---|---|
| The village (surface) | the spawn, the bell in its frame, the far hills; the well's mouth and the root climb's top |
| The old well | two planks on the way down (one way: its foot opens high over the upper hollow) |
| The upper hollow | lantern 0, two beetles, the stones, a ledge, stalactites, planks up to the first chime and a heart |
| Old Burrow's nook | the mole, his candle |
| The deep hollow | lantern 1 under the drop, a beetle, two moths, the gloam's hall (two planks above its charge), the bramble pit and its lift, the second chime on a ledge, lantern 2 |
| The root climb | ten planks three rows apart back to the village |
