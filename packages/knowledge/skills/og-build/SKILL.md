---
name: og-build
description: The Origin Engine build loop — og cook, og run with a scripted command channel (inputs, captures, dumps, get and set), screenshots and world dumps read as evidence, og dev hot reload, og test playtests, og polish's taste and feel scores (the one list of what it measures and where each finding is fixed), og build for publishing, og run --device for the first frame and frame times on a real browser or phone, and big games: streaming groups and what they may hold, .vox models, compressed animation, chunked music, the lanes and cross-origin isolation. Use whenever a change to a project with a project.ogt must be run, observed, scored or shipped (not a Godot project.godot).
---

The `og` tool is the whole loop: it cooks assets, runs the game headless on
the native host with a command script, serves the probe, reloads on save,
runs playtests, scores them and builds the publishable bundle. Nothing here
needs a browser or a Rust toolchain. The command channel is
`reference/commands.md`, the probe `reference/probe.md`.

## What works

The loop held for two whole games written from this package alone
(`examples/lantern-vale`, `examples/hollowbell`): `og cook` took 47 and 69
assets cold in 8.2 s and 3.6 s and under 100 ms after a change, without a
warning, and every refusal named the line and the fix (a copied skin
without its CJK `fallback` named the line and og-taste §1;
`PostEffects.vignette is true or false` named the line). Scripted runs gave
the same frames every time, `store: "keep"` relaunches and two-process
replays worked the first time they were asked for, and `og polish` scored
taste 99 for both and feel 90 and 92, naming the section that fixes each
finding. The third, The Kestrel Stair (`examples/kestrel-stair`, 3D
third-person), cooked 62 assets cold in 8.5 s (110 outputs, 9.9 MB) and
about 0.1 s after a change, without a warning, its refusals naming the
line; two processes of a full session (chases, bites, strikes, a rest, the
gate, the bell) ended on one world hash, and `og polish` found every
input answered within a tick and every damage moment with hit stop, shake,
a number and a sound (taste 98, feel 79).

## 1. The loop

1. **Cook** after any change under `assets/`: `og cook`. It is incremental
   (only changed sources re-cook) and writes `cooked/`, the only thing the
   runtime reads. A load that names an uncooked path, or a blob whose cook
   stamp does not match the manifest, is refused with the path; cook again.
2. **Run with a script**: `og run . --headless --commands "<script>"`. The
   script is `t:action[:args];...` with `t` in simulation seconds from the
   start of the run, so the same script always lands on the same ticks:

   ```sh
   og run . --headless --seed 1 --commands "0.5:touch_move_to:Start;0.6:touch_start;0.7:touch_end;2.0:screenshot;2.2:dump_world;2.5:quit"
   ```

   Core actions: `screenshot[:path][:full|:N]`, `listen[:path][:seconds]`,
   `touch_move_to:<ui id>`, `move_to_text:[=]text[@N]`, `touch_move:x,y`,
   `touch_start`, `touch_end`, `touch:<id>:<down|move|up>:<x,y | ui id>`
   (a second finger), `gamepad:<a+b+...|->[@pad][:lx,ly][:lt,rt]`, `scroll:dx,dy`,
   `key_down:<Key>`, `key_up:<Key>`, `type:<text>`, `compose:<text>` and
   `commit:<text>` (an input method, og-ui §7),
   `teleport:<entity>:x,y,z`, `frame_entity:<entity>`, `dump_world[:path]`,
   `dump_entity:<entity>[:path]`, `dump_ui[:path]`,
   `get:<Resource>[.<field>]` or `get:<entity>[.<Component>[.<field>]]`
   (logs the value as game code sees it, og.Time whole and a light og.DayNight
   drives as it is drawn, and writes `get-<tick>.json`, then
   `get-<tick>-2.json`, ... for more on one tick; a world dump lists
   resources by name only, so `get:` is how to read one),
   `set:<entity>.<Component>.<field>=<value>` or
   `set:<Resource>.<field>=<value>` (`set:og.Camera.trauma=0.5`,
   `set:Game.coins=10`, `set:og.Stylise.ink=false`: a number, `true` or
   `false`, or `x,y,z`; a value outside the field's `min`/`max` is refused
   naming the range, never clamped, while game code's own writes are stored
   as given: widen the range in the field's options to try one), `seed:<n>`, `pause`, `resume`, `advance:<ticks>`,
   `quit`, and `help` for the list. Modules add their own: `dump_ai` and
   `dump_nav` (og-ai §5), `dump_platform` (og-platform-sdk §4), `locale:<tag>`
   (og-i18n §6), `dump_stream` and `terrain_stream:<stream|resident>[:<MiB>]`
   (og-game §6 and §11). Unknown actions warn and the script continues; read the
   warnings.
3. **Read what it wrote.** Screenshots are PNG files, dumps are JSON files
   (`<action>-<tick>.png|json` unless the step names a path), written
   under `og-out/` in the project (`--out <dir>` chooses); `og run`
   prints each path. Open the frame and describe what it shows; read
   `dump_world` for the entity count, the archetype tables and each system's
   matched count. A system that matched 0 entities is usually a missing
   component on the spawn, not a logic bug.
4. **Fix and repeat** with the same script and seed. Keep the script in the
   change so the next run is the same run.

A game's own actions (`GameDesc.commands`, og-game §3) join the list; give
each its argument form and one sentence in `GameDesc.commandHelp`
(`{ spawn_wave: { usage: "<count>", doc: "Spawns a wave now." } }`)
so `help` explains it (it lists `spawn_wave:<count>`: the name, then the
usage). An entry naming no action is logged when the game
starts.

## 2. Live sessions: the probe and hot reload

- `og run .` without `--headless` keeps running and serves the probe over a
  loopback WebSocket; `og probe <method> [json]` calls it (`og probe hello`,
  `og probe state`, `og probe dump '{"what":"world"}'`,
  `og probe capture '{"width":640,"height":360}'`, `og probe patch …`). A
  refusal is `false` or `null`, never an error; an error means the call was
  malformed.
- `og dev .` watches `src/` and `assets/`: it rebuilds the bundle, re-cooks
  changed assets and reloads the running game through the probe, keeping the
  world state (components migrate field by field by name). The budget is
  300 ms from save to the first new frame; `og dev` reports each reload's
  latency and any dropped components or fields. A reload that fails keeps
  the old bundle running and prints why.

## 3. Tests

`og test .` runs the project's bun tests and every `tests/*.og.ts` playtest
script through `og run --headless --commands`. A playtest asserts on what the
game exposes (dumps, `state()` facts, entity fields, `get` answers), never on
how the code is written. Each playtest writes into its own
`og-out/test/<name>/`, emptied when it starts, and its `check` reads a
step's file by the name the step gave (`run.json("shop.json")`,
`run.png("title.png")`); give each step's file its own name. A test that ran no checks fails. Run it before
claiming a change works. A playtest runs with the host clock at
2026-01-01 00:00 UTC (`clock` to change it) and a fresh store; `store:
"keep"` with `then: [{ commands, clock }]` runs further processes over the
same store, for a relaunch (og-platform-sdk §4).

## 4. Evidence discipline

- Every change that alters rendering, animation, physics or the tick carries
  a scripted `og run --commands` run whose frame was opened, read and
  described in the change: what is on screen, where, and how that differs
  from before. A frame nobody looked at is not evidence.
- Name the exact command, seed and script with the frame and the dump, so
  anyone can rerun it and get the same bytes.
- Determinism is checkable: two runs with the same seed and script give the
  same world hash in `dump_world`. When they do not, find the wall-clock
  read or the unseeded randomness before anything else.
- Report what ran and what did not. A build that compiled is not a game
  that plays.

## 5. Polish, measured: `og polish`

This is the one list of what `og polish` measures; og-taste §10 and
og-feel §13 are the checklists it scores, and every other skill points
here. `og polish .` plays every playtest headless, samples every tick
(`dump_pulse`: time scale and hit stop, camera trauma, punch and kick,
floating texts, afterimages, tweens, particle systems, voices by bus, the
named entities' positions and flipbook frames, step and render time), takes
`dump_ui` beside every `screenshot` and captures the audio. It writes
`og-out/polish/report.md`: a **taste** and a **feel** score (0 to 100 each,
the mean of their rows; a row is n/a when no playtest exercises it), the
frames, and for each finding the skill section that fixes it. It refuses a
project without playtests: write them first (§3), with a `screenshot` on
every screen that matters.

| Axis | Measure | From | Passes at | Fixed in |
|---|---|---|---|---|
| taste | Contrast | each text element's rectangle in a shot: the ground (its median) against its ink, read over the text's own glyphs (the pixels at least half-way to the strongest ink; the farthest twentieth of them, the glyph cores) | 4.5:1 | og-taste §3 |
| taste | Text size | the tallest run of ink rows in each text element, CSS px | 9 px, a 13 px face's capitals | og-taste §4, §10 |
| taste | HUD occupancy | the union of leaf elements on gameplay shots, touch controls (joystick, d-pad, action buttons) left out; a shot with a dialog or loading layer, a conversation, a list over a quarter of the surface or three texts in the central third is a screen, not gameplay | at most 15 % of the surface and 5 % of the central third | og-taste §2 |
| taste | Screen transitions | a change of three or more elements with UI motion or a tween in the 20 ticks around it | every change | og-taste §5 |
| taste | UI sounds | a press that changed the UI and started a voice on the ui bus within 8 ticks | every press | og-taste §6, og-audio §3 |
| taste | Locale keys | `og cook`'s warnings | none | og-i18n §1 |
| feel | Input latency | ticks from the tick an input lands to the first thing seen or heard to change (a named entity accelerating or changing clip, the UI, a sound, a tween, a floating text, shake, hit stop, a particle burst); a press answered on its release counts from the release | 0–1 ticks | og-feel §1 |
| feel | Hit feedback | each damage moment (a fact named `hp`, `health`, `lives`… falling, or `hits`, `damage`, `kills`… rising): hit stop, shake, floating text and a sound within 8 ticks | three of the four | og-feel §3, §4, §7, og-audio §1 |
| feel | Idle frames | ticks after the first ten where nothing moves or animates, no UI moves, no sound starts or plays, no tween, text or particle runs | at most 10 % | og-feel §9, og-taste §5 |
| feel | Sound on input | inputs answered by a sound within 6 ticks | every action | og-audio §1 |
| feel | Frame time | step + render, p95, on the tier it ran on (`metal` on a Mac, `vulkan-cpu` on CI's lavapipe); the probe's own screenshots (render, read back, PNG) are not the game's step and are left out | 8.3 ms (half the frame) | og-game §14 |

The report also lists clipped samples in the mix (og-audio §4). Contrast and
text size read text only: icons and images (their "text" is a picture's
name), disabled controls and what they hold (WCAG exempts them), the
slivers a scroll container cuts, what a dialog's or a tutorial's scrim
dims on purpose and what is mid-transition in the shot (an element whose
bounds differ on the ticks before and after it: a toast sliding out over
a dialog, a panel popping in) are left out, and each element is measured
at the pixels of the surface `dump_ui` reports, so a phone preview
(390×844 at 3×) is read at its own size. The ink is measured over the
glyphs, not the rectangle, so a short word in a wide label (a three-letter
name on a 356 px leaderboard row) reads what its letters show, as a full
line does, and not its antialiased edges diluted by the empty rest of the
row. The ink is what the pixels show,
so it depends on the glyphs' coverage: thin strokes cover few pixels
whole, and a regular-weight CJK face at body sizes measures lower than
Latin in the same colours (a 15 px "设置" 3.1:1 where "Settings" reads
4.6:1 on one button), as it reads lower to the eye. Darken the ink in the
skin's palette, give the CJK face a heavier weight or the label a larger
step; the measure is not wrong there. Facts are the game's `facts`
(og-game §3): expose `hp`, `hits` or `kills` there or the hit measures stay
n/a. A baseline line says how often things change with no input at all;
when it is high (a runner that always moves), read latency together with
the frames.

The numbers cannot judge a style. Read the frames before changing anything,
fix the cause in the skin, the template or the system, never per call, and
run it again after the fix: the report is the evidence.

## 6. Shipping

`og build .` bundles `src/` to `dist/game.js`, cooks, and copies the engine
runtime and wasm next to it. `og build . --publish` produces the published
form: the probe keeps only its player-facing members (hello, tick, state,
pick, entity, input, capture) and the command channel is gone.
`og run --device chrome .` plays that publish build in a real browser
(`android[:serial]` over adb, `iphone[:udid]` over safaridriver), passes
the play gate and writes `og-out/device/report.md`: the first frame against
its budget (2.0 s, 1.5 s on an iPhone), the frame-time percentiles on the
tier the device chose, the device itself, and a fix per finding (og-game
§6 for a slow first frame, og-game §14 for missed frames). Run it on the
phone you ship for: og polish's native numbers are not a phone's. `og doctor`
names each missing tool (bun, the native addon, the wasm artifact) with its
fix.

## 7. Big games: what the cook packs and the page fetches

What `og cook` takes is decided by the file: `.glb`/`.gltf` and MagicaVoxel
`.vox` (meshed into a glTF: flat-shaded quads, palette colours, `model {
voxScale voxOrigin }` for the size and origin, frames as clips; og-game §5)
are models; `.ogt` files by their `type` (scene,
look, environment, texture-array, flipbook, particles, terrain, shader…);
`.cube` LUTs, `.hex` and `.gpl` palettes, images, WAVs and fonts by
extension. A look's built-in `@canvas/…` and `@palettes/…` assets are added
to the bundle only when a look names them; a game's own `assets/@canvas/`
is refused.

**Streaming groups** (og-game §6) are cooked into one pack each,
`cooked/stream/<name>.ogb`, and `og build --publish` ships each as its own
file the page fetches when the group loads. A group may hold glTF models
without skins or morph targets, textures, texture arrays, sounds,
environment maps, palettes, looks and LUTs:

```text
stream "night-market" {
  asset "models/stalls.glb"
  asset "env/night.ogt"            # an environment map
  asset "tiles/market.ogt"         # a texture array
  asset "palettes/neon.hex"
  asset "looks/night-market.ogt"   # applied once resident
  asset "grade/night.cube"
  volume "market-gate"
}
```

The cook refuses any other kind (a skinned model, a chunked sound, a scene)
in a group, naming the scene and the asset. Prove a group in a script:
move the camera into its trigger (`teleport`, or the game's own command),
`advance` a few ticks, then `dump_stream` lists each group's state,
memory and the assets kept past an unload (`retained`), and
`get:` of a field the game copies from `og.stream.budget` shows `used` and
`kept` against `budget`. A run with the same script must end on the same
world hash streamed or not: nothing a tick reads depends on what is
resident.

**Size knobs** in a model's or a sound's import settings (`<file>.ogt`,
`type import`):

```text
animation {
  compress true        # quantised clips, keyframes the curve rebuilds dropped
  tolerance 0.0001     # the most a sampled value may move (og-game §9)
}
sound {
  stream true          # cooked in chunks (the default past 1 MiB of QOA, about 27 s of stereo)
  chunk 4              # seconds per chunk (og-audio §5)
}
```

Compare `og cook`'s byte count before and after: the bundle is the
player's download.

**The lanes** spread physics, skinning, culling and extraction over the
cores with results identical to one thread (og-game §14). Natively
`OG_LANES=off` (or `1`) runs serial and `OG_LANES=<n>` uses n threads; a
world hash must not change between them, which is a check worth a
playtest of its own on a physics-heavy game. On the web the lanes need a
cross-origin isolated page: `project.ogt`'s `publish { isolation
required | preferred | off }` (`preferred` by default: lanes where the
host serves COOP/COEP, one thread otherwise, the same game either way).
The lanes build caps memory at 2 GiB. `dump_lanes` reports what ran where.
