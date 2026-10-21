# Lantern Vale

A small 3D top-down adventure on Origin Engine, about five minutes of play,
written the way a game author would write it: from the knowledge package's
skills and wiki, with nothing but TypeScript, generated assets and `og`.
It is the engine eating its own food; what it found is
[`FINDINGS.md`](FINDINGS.md).

The sun is going down over a valley and four lanterns stand dark along its
paths. The lamplighter walks there (WASD or the arrows, Shift to run, the
left stick) and holds E (the pad's X) by a lantern to light it. Three
slimes patrol the meadow: one that sees the lamplighter turns alert and
bounds after it, and its touch costs a heart. Light all four before night
(19:30, five minutes of play) to win; three hearts gone, or night, loses.
The results come at the next dawn, under a second look. Esc pauses; the
settings set the music and effects, the lamplighter's name and the
language (English or 简体中文).

## Run it

```sh
bun tools/make-assets.ts        # once after a clone: writes the 23 MB music (not committed)
og cook .
og run . --headless --commands "1:screenshot:title.png;1.1:run;2:warp:0;2.2:key_down:KeyE;3.6:key_up:KeyE;4:screenshot:lit.png;4.1:quit"
og test .                       # the eleven playtests, the reference frames against references/<tier>/
og polish .                     # taste 99, feel 90 on metal (og-out/polish/report.md)
OG_FX_CAPTURE=1 og test .       # records this tier's reference frames
```

Inside the engine repository `og` is `target/debug/og` (`cargo build -p
og-cli`), and the playtests load the dev addon at
`packages/engine/native/origin-engine.<platform>.node` (`cargo build -p
og-host-node`, copied as `packages/engine/scripts/build.sh` does).
`tools/make-assets.ts` remakes every asset deterministically;
`python3 tools/subset-font.py` (fontTools) remakes the Chinese font subset
after a string table changes.

## What it exercises

| Area | How | Where |
|---|---|---|
| Looks | `GameDesc.look` applies `looks/lantern-vale.ogt` (extends `anime-cel-3d`: AgX, the cool shadow side, ink, bloom, fog, the environment map, the skin) before `start`; `og.load.look` and `og.render.applyLook` swap in `looks/dawn.ogt` (extends `luminous-layered-painting`: five depth bands) for the results and back | `src/ui.ts` `looks` |
| `og.Stylise`, `og.MaterialStyle` | through the looks: the anime cel preset's toon and outlines (a material's outline draws the voxel models' silhouettes and contact lines; the lanterns' lacquer and the slimes' jelly keep their physical shading under the toon), ink on silhouettes, a tinted shadow side and rim light; depth bands at dawn, whose look takes the toon off | `assets/looks/`, `src/world.ts` `styleModels` |
| Physical materials | the lanterns' red lacquer (`clearcoat` on the voxel model's material), the slimes' jelly (`KHR_materials_transmission`, volume and ior in their glTF) | `src/world.ts`, `tools/make-assets.ts` `slime` |
| `og.Environment` | a 64 x 32 synthetic dusk `.hdr`, prefiltered by the cook (`env/dusk.ogt`), named by both looks' `Environment` block; AgX with auto exposure adapting 1 EV/s up, 3 down | `assets/env/`, `assets/looks/` |
| `.vox` models | hero, lantern, tree, boulder, pine and arch written voxel by voxel, cooked with `model { voxScale voxOrigin }` | `tools/make-assets.ts` |
| Skeletal animation on twos | the slime (its voxels meshed into a skinned glTF: two joints, `idle` and `chase`) on `og.AnimationPlayer` with `stepFps 12` | `src/slimes.ts` |
| Texture arrays, drop shadows | each slime's expression bubble is an `og.Sprite` drawing a layer (`arrayLayer`) of a texture array, with `dropShadow` | `src/slimes.ts`, `assets/sprites/` |
| Streaming | the far side's pines, arch and banner are a `stream "far-side"` group (`scenes/main.ogt`, a distance trigger); the game follows `og.StreamEvent`, `og.stream.status` and `og.stream.budget` | `src/ui.ts` `stream`, `tests/stream.og.ts` |
| Physics | the hero on `og.CharacterController` over the terrain's height field; a pile of ten boulders (dynamic bodies) it shoves; trunks that stop it; `og.physics.overlap` on the lantern layer finds the lantern within reach | `src/hero.ts`, `src/lanterns.ts` |
| Terrain | a 96 m valley from a 16-bit height map and a splat map (grass, path, rock on slopes, moss on the far side) | `assets/terrain/` |
| Audio | a 4-minute track synthesised to WAV and cooked in 4-second chunks (`sound { stream true }`), looping on the music bus, ducked under the effects; `og sfx` presets on the sfx and ui buses; reverb on effects, a limiter on master | `src/sound.ts` |
| og.ui | the fantasy pack copied as the game's skin; `mainMenu`, a HUD of capsules, `pauseMenu`, a settings dialog (two sliders, a `textField` for the name, a language radio, a toggle, a label mixing Latin and Chinese), `resultsScreen`; a dissolve between screens; toasts | `src/ui.ts` |
| i18n | `strings/en.ogt` and `strings/zh-CN.ogt`; Noto Sans SC subset to the table's characters as the Chinese table's font and the skin's fallback | `assets/strings/`, `tools/subset-font.py` |
| Accessibility | `og.ui.a11y()` read by the game; the settings playtest asserts no focusable node is unnamed | `src/ui.ts`, `tests/settings.og.ts` |
| Feel | a touch: 6-tick hit stop, shake, red flash, knock-back, "-1", the hurt sound; a lantern: punch, burst, "+1", toast, chime | `src/slimes.ts`, `src/lanterns.ts` |

## Playtests

| File | Proves |
|---|---|
| `flow.og.ts` | Set out, walk and run on the terrain, the overlap query finds a lantern, holding E lights it |
| `slimes.og.ts` | a slime's sight, its alert bubble (array layer 1, drop shadow), the chase clip on twos, its touch with hit stop, shake and sound |
| `settings.og.ts` | Escape pauses (time scale 0), the Music slider, a name typed partly through an input method, the mixed-script label, Chinese, saved through og.platform, a named accessibility tree |
| `win.og.ts` | four lit: the results at dawn under `looks/dawn.ogt`, Stylise replaced, Again under the dusk look |
| `lose.og.ts` | no hearts left, night at dusk's end, Again and Title |
| `stream.og.ts` | the far side unloaded, resident as the camera nears it, unloaded as it leaves; two runs, one hash |
| `music.og.ts` | the music cooked in 60 chunks and heard from the title |
| `physics.og.ts` | the controller shoves the pile and a trunk stops it |
| `locales.og.ts` | the title, HUD, pause and results in Chinese, then English |
| `replay.og.ts` | a session with all of the above replays to the same hash |
| `frames.og.ts` | seven reference frames at 1280 x 720 (`references/metal/`) |

**Frame time** (wave 12, `tests/evidence/P4/perf-games/lantern-vale.md`,
on a machine at a load average of 7 to 16): the 600-tick play run's
step is 0.71 ms median and 1.78 ms p95 on the release addon (2.13 / 4.98
dev); paced at 60 Hz the render is 1.18 / 1.60 (2.07 / 2.40 dev), and
og polish scores frame time 100 on release and 85 on dev (its dev
playtests' step p95 reaches 9.5 ms). Run unpaced, the render is the
GPU's time for the look: turning its features off one at a time (at a
load of 15 to 23), bloom is about half of it (6.1 to 3.5 ms median) and
auto exposure the next (to 5.1); all five of bloom, auto exposure, ink,
fog and the vignette off, 1.6 ms. A lower `bloomIntensity` does not make
bloom cheaper; a second look without it, applied with
`og.render.applyLook` where the GPU is small, does (og-game §14 and
§15). On the step, `Slimes` and `Hero` read and write each slime's and
rock's components with `world.get` and `world.set` every tick (8 % of a
dev frame in those calls); one query over the slimes' columns is the
og-game §14 way.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `run` | starts a run, as Set out does |
| `warp:<lantern>` / `warp:<x>:<z>` | puts the hero beside a lantern (0 to 3; 3 is on the far side) or on the ground at x, z |
| `light:<lantern>` | the lantern catches now (the fourth wins) |
| `hearts:<n>` | sets the hearts left; a heart lost lands as a slime's touch does |
| `dusk:<seconds>` | jumps the run's clock (night at `Tuning.duskSeconds`, 300) |
| `language:<en\|zh-CN>` | switches the language as the settings do |
| `settings` | opens the settings |

## Layout

`src/components.ts` the state (Game, Tuning, Prefs, Hero, Lantern, Slime,
Rock, Sfx); `src/world.ts` the vale; `src/hero.ts` the hero and the camera;
`src/slimes.ts`, `src/lanterns.ts`, `src/sound.ts`, `src/ui.ts` the rest;
`src/main.ts` the game, its commands and facts. Systems in tick order:
`og.systems.cameras`, Ui, Hero, Lanterns, Slimes, View, Voices.
