---
name: og-build
description: The Origin Engine build loop — og cook, og run with a scripted command channel (inputs, captures, dumps, get and set), screenshots and world dumps read as evidence, og dev hot reload, og test playtests, og polish's taste and feel scores (the one list of what it measures and where each finding is fixed), the cook's art conformance (what og cook --report says it changed, conform off, og style check), og build for publishing, og run --device for the first frame and frame times on a real browser or phone, and big games: streaming groups and what they may hold, .vox models, compressed animation, chunked music, the lanes and cross-origin isolation. Use whenever a change to a project with a project.ogt must be run, observed, scored or shipped (not a Godot project.godot).
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
warning, and every refusal named the line and the fix (a copied UI style, as
styles were then called, without its CJK `fallback` named the line and og-taste §1;
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
   runtime reads. With project.ogt's `look` line it also makes every asset
   meet the look's art contract (§8). A load that names an uncooked path, or a blob whose cook
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
   `get-<tick>-2.json`, ... for more on one tick; `get:Game:game.json`
   names the file; a world dump lists resources by name only, so `get:`
   is how to read one),
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

   Headless means no window, not no rendering: `og run`, `og test` and
   `og polish` still render every tick when an adapter is available. A
   sandbox without a GPU may have Mesa lavapipe (`vulkan-cpu` in
   `og doctor`), which renders on the CPU and can take minutes per
   game-second. Use `OG_RENDERER=none og test .` (likewise `og run` and
   `og polish`) for simulation-only iteration there. Do not treat this as
   visual approval: record and compare frames on the retained reference
   tier, and run visual polish with a renderer.

   Where there is no renderer (`OG_RENDERER=none`, or no usable adapter: `og run --json` says
   `"tier": "none"`, `og doctor` says the renderer is `none`), the run is
   the same run: the same ticks, dumps and world hash. A `screenshot`
   writes no PNG and logs "no renderer, frame unchecked" with the file it
   would have written, not as a warning; `og run --json` lists those files in
   `unchecked`, and the probe's `capture` answers null. Read the dumps
   there, and look at the frames where there is a renderer (§4).
4. **Fix and repeat** with the same script and seed. Keep the script in the
   change so the next run is the same run.

A game's own actions (`GameDesc.commands`, og-game §3) join the list; give
each its argument form and one sentence in `GameDesc.commandHelp`
(`{ spawn_wave: { usage: "<count>", doc: "Spawns a wave now." } }`)
so `help` explains it (it lists `spawn_wave:<count>`: the name, then the
usage). An entry naming no action is logged when the game
starts. A step's arguments are its colon-delimited fields after the
action, never shell words: `0.5:records:12:34.5` hands the handler
`args[0]` `"12"` and `args[1]` `"34.5"`, and its usage is written the same
way, `usage: "<berries>:<seconds>"` (a space stays inside its field:
`records:12 34.5` is one argument). A game may not take a core action's
name; og-game §3 lists them, with `open_pause` beside the engine's `pause`.

**File names.** Every output of one script has its own file. An unnamed
output takes the action and the tick (`get-000030.json`), and a second on
the same tick is suffixed quietly (`get-000030-2.json`). A named one that
an earlier step of the script already wrote, at any time (a copy-pasted
`0.5:get:Game:met.json`), is written as `met-2.json`, then `met-3.json`,
never over the first, and logs a WARN naming both steps ("'met.json' was
already written by the 0.5s get step, so this one is 'met-2.json'"). The
WARN is in `run.warnings`, so a playtest that asserts a clean run
(`t.equal(run.warnings.length, 0, ...)`) fails on the clash, and
`run.json("met.json")` reads the first step's: give each named step its
own file.

**Values in the JSON.** A dump and a `get` answer write a vector as an
array in lane order (`Vector3` `[x, y, z]`, `Quaternion` `[x, y, z, w]`,
`Color` `[r, g, b]`), where the API's values are objects
(`{ x, y, z }`); `set:` takes the lanes comma-separated, without spaces
(`set:box.og.Transform.position=4,5,6`). Read a dump by index:

```ts
// 0.3:get:box.og.Transform:t.json writes {"kind":"get","target":"box.og.Transform","tick":18,"value":{"position":[1,2.5,-3],...}}
const [x, y, z] = run.json<{ value: { position: [number, number, number] } }>("t.json").value.position;
t.near(y, 2.5, 1e-4, "the box stands on the ledge");
```

An `f32` field is IEEE-754 single precision. Game code reads it as the
f32's exact value promoted to a JavaScript number, so `2.7` written to an
f32 reads back as `2.700000047683716`, and what game code sends carries
that number as JSON (the facts, an og.platform payload, so
`dump_platform`); `get` and the dumps print the f32's shortest form,
`2.7`. Compare numbers with a tolerance in a test (`t.near(a, b, 1e-4,
...)`), round only to display one (`toFixed(1)` on the label), and keep a
value that must round-trip exactly (a payload a test compares with `===`)
in an `f64` field.

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
claiming a change works.

While iterating, `og test . --filter tests/lobby.og.ts` runs only that
suite. The filter is a case-sensitive substring of the project-relative
file path, using `/` separators: `--filter lobby` selects every matching
`.og.ts` and `.test.ts` file; it does not match test titles or use globs.
No matching files is a failure, not a passing empty run. The project's
type check still runs. Drop the filter for the complete regression run.

Put every check that reads a frame (`run.png`, a decoded screenshot, a
comparison with `references/`) inside `t.frame(files, () => ...)`:

```ts
check(run, t) {
  t.equal(run.json<Game>("shop.json").coins, 10, "the coin was banked");
  t.frame(["title.png", "shop.png"], () => checkFrames(run, t, ["title.png", "shop.png"], ""));
},
```

Without a renderer (`run.rendered` false, `run.tier` `"none"`) the scope
does not run: `og test` prints `~ <file>: no renderer, frame unchecked` for
each screenshot, counts them in its summary (`og test: 7 suite(s) passed;
9 frame(s) unchecked`) and passes on every other check, which runs as it
would with a renderer. A check that reads a frame outside `t.frame` stops
there (the report says the checks after it did not run), so keep frames in
the scope. Unchecked frames never change `og test`'s exit status; they are
verified where there is a renderer: the author's machine, the
browser-check sandbox (the published page, §6) or an executor with a GPU. A playtest runs with the host clock at
2026-01-01 00:00 UTC (`clock` to change it) and a fresh store; `store:
"keep"` with `then: [{ commands, clock }]` runs further processes over the
same store, for a relaunch (og-platform-sdk §4).

## 4. Evidence discipline

- Every change that alters rendering, animation, physics or the tick carries
  a scripted `og run --commands` run whose frame was opened, read and
  described in the change: what is on screen, where, and how that differs
  from before. A frame nobody looked at is not evidence. Where there is no
  renderer the frame is unchecked, not seen: say so, and look at it where
  there is one (§3) before calling a visible change done.
- Name the exact command, seed and script with the frame and the dump, so
  anyone can rerun it and get the same bytes.
- Determinism is checkable: two runs with the same seed and script give the
  same world hash in `dump_world`. When they do not, find the wall-clock
  read or the unseeded randomness before anything else.
- A model built as code (og-model) is judged on numbers and one sheet,
  not on a frame read by eye: `view:<entity>:front|back|side|top`,
  `view:<entity>:orbit:<degrees>` and `turntable:<entity>:<n>` frame it
  from fixed cameras, and `og compare <reference.png> <render.png>`
  prints the silhouette IoU and the ΔE00 per mask region and writes the
  reference beside the render as one sheet PNG (`--mask`, `--align`,
  `--json`). Two runs give the same numbers and the same sheet bytes, so
  quote them with the command. og-model §3 has the commands and how to
  read them, og-model §4 the gate each pass must meet.
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
| feel | Hit feedback | each damage moment (a fact named `hp`, `health`, `lives`… falling, or `hits`, `damage`, `kills`… rising): hit stop, shake, floating text and a sound within 8 ticks | three of the four, per moment (a moment with three owes no fourth: a bump that holds, shakes and sounds is not asked for a number) | og-feel §3, §4, §7, og-audio §1 |
| feel | Idle frames | ticks after the first ten where nothing moves or animates, no UI moves, no sound starts or plays, no tween, text or particle runs | at most 10 % | og-feel §9, og-taste §5 |
| feel | Sound on input | inputs answered by a sound within 6 ticks | every action | og-audio §1 |
| feel | Frame time | step + render, p95, on the tier it ran on (`metal` on a Mac, `vulkan` on a GPU); the probe's own screenshots (render, read back, PNG) are not the game's step and are left out. Without a renderer the row says `n/a (no renderer)`, as do contrast, text size and HUD occupancy, which read frames | 8.3 ms (half the frame) | og-game §14 |

The report also lists clipped samples in the mix (og-audio §4), and a UI
style finding (no score) when a shot's `dump_ui` reports `style` empty: the
UI draws in prototype, the engine's placeholder, not a style of the game's
own (og-taste §1). Contrast and
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
style's palette, give the CJK face a heavier weight or the label a larger
step; the measure is not wrong there. Facts are the game's `facts`
(og-game §3): expose `hp`, `hits` or `kills` there or the hit measures stay
n/a. A baseline line says how often things change with no input at all;
when it is high (a runner that always moves), read latency together with
the frames.

The numbers cannot judge a style. Read the frames before changing anything,
fix the cause in the style, the template or the system, never per call, and
run it again after the fix: the report is the evidence.

## 6. Shipping

`og build .` bundles `src/` to `dist/game.js`, cooks, and copies the engine
runtime and wasm next to it. `og build . --publish` produces the published
form: the probe keeps only its player-facing members (hello, tick, state,
pick, entity, input, capture) and the command channel is gone. A `2d`
project publishes the 2D module, which has no 3D world
(`RenderCapabilities.world3d` false): the build refuses scenes holding
`Terrain`, `TerrainScatter`, `Water`, `WaterSurface` or `Fluid3D`, naming
each by scene file and line (og-game §2).
What ships is still checked by script, through what it keeps: real input
on the published page and the probe's `tick`, `state` and `capture`. A
playtest written in input steps (`key_down`, `type`, `touch_move_to` an
id, `move_to_text`, `touch_start`) and `screenshot` steps runs there as it
runs under `og test`, each frame compared with the same references; a
game command (warp, give) has no channel there and is refused, so keep
the steps a published check needs free of them. The page shows the
Player's Back over the canvas's top-left corner (12, 12, 40 by 40 CSS px
on a desktop page; og-ui §5), which anchored HUD groups keep out of, so
a playtest the page is held to gives its native run the same cover with
`setup: "cover:12,12,40,40"` (`PlaytestDesc.setup`: command steps run
once after `start`, before the first tick, never part of the page's
script) and its frames lay out as the page's. The engine runs this on
the published build of its own games (its `tests/web/published.ts`: the
boot, the title, a press, a typed name and a frame against the metal
reference, with a clean console).
`og run --device chrome .` plays that publish build in a real browser
(`android[:serial]` over adb, `iphone[:udid]` over safaridriver), passes
the play gate and writes `og-out/device/report.md`: the first frame against
its budget (2.0 s, 1.5 s on an iPhone), the frame-time percentiles on the
tier the device chose, the device itself, and a fix per finding (og-game
§6 for a slow first frame, og-game §14 for missed frames). Run it on the
phone you ship for: og polish's native numbers are not a phone's. `og doctor`
names each missing tool (bun, the native addon, the wasm artifact) with its
fix, and says whether this machine has a renderer (none is not a problem:
the playtests' frames are then unchecked, §3).

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

A sound cooked in chunks may join a group too: its first chunk goes in
the pack and every other chunk stays a file of its own, read ahead of the
playheads once the group is resident (og-audio §5). The cook refuses any
other kind (a skinned model, a scene) in a group, naming the scene and
the asset. Prove a group in a script:
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

**What a player downloads** is what `og build --publish` prints under its
size table, brotli `-q 11` per file as the size gate measures the engine:

```text
download (brotli-11, what a player fetches; a WebGPU and a WebGL2 page fetch the same files):
  boot, lanes page (isolated)      92 files     4323854 B  (engine 2976.8 kB, cooked 1044.3 kB, shell 85.5 kB, basis 217.3 kB)
  boot, single-threaded page       91 files     4284988 B  (engine 2937.9 kB, cooked 1044.3 kB, shell 85.5 kB, basis 217.3 kB)
  later: 20 sound chunks 2427050 B (ahead of their playheads); the audio worklet 875 B (first gesture); the service worker and icons 32634 B
```

The boot is everything fetched before the first frame: one engine module
(the lanes module on a cross-origin isolated page, else the
single-threaded one), the shell, the Basis transcoder and the cooked first
group (everything no streaming group holds). A sound cooked in chunks
costs the boot its first chunk only (about `chunk` seconds of QOA): the
rest are fetched ahead of its playheads, and none before it plays; in a
streaming group it costs the boot nothing (its first chunk comes in the
group's pack). The
table's `total` is every file of the directory (both engine modules, the
licences, the icons), never a download. Compare the boot line before and
after a change; `--json` carries the same as `downloads`.

**The lanes** spread physics, skinning, culling and extraction over the
cores with results identical to one thread (og-game §14). Natively
`OG_LANES=off` (or `1`) runs serial and `OG_LANES=<n>` uses n threads; a
world hash must not change between them, which is a check worth a
playtest of its own on a physics-heavy game. On the web the lanes need a
cross-origin isolated page: `project.ogt`'s `publish { isolation
required | preferred | off }` (`preferred` by default: lanes where the
host serves COOP/COEP, one thread otherwise, the same game either way).
The lanes build caps memory at 2 GiB. `dump_lanes` reports what ran where.

## 8. Art conformance: the assets meet the look

The game's look carries an art contract (og-taste §12: `ArtPalette`,
`ArtPixels`, `ArtTexels`, `ArtShape`, `ArtLine`, `ArtMotion`,
`ArtCamera`), and project.ogt's `look "looks/<game>.ogt"` names it. With
that line, `og cook` makes every asset meet the contract as it cooks it:
the source under `assets/` is never written, the cooked blob conforms, and
the same sources and look give the same bytes. Without the line, or with
every art block at zero, nothing changes. This is the consumer's half of
producing an asset on Origin Engine:

1. **Request it through the brief.** `bun
   node_modules/@origingame/engine/knowledge/scripts/brief.ts <kind>
   "<subject>" --project .` prints the prompt, the references and the
   generator's parameters for one asset in the game's style (og-taste §1);
   generate with them as they are.
2. **Import it** under `assets/` as any asset (og-game §5): a sprite
   sheet with its flipbook, a tile set, a glTF model, a texture.
3. **Cook and read the report.** `og cook --report` ends with the art
   contract's lines for 2D art:

   ```text
   og cook: the art contract changed 2 asset(s), 1 met it as they were, 1 opted out:
     sprites/courier.png (changed): 32x40 cells to 26x32 by the dominant-colour box (270x82 to 222x66 texels); alpha to 1 bit (96 texels); line 1 px inside (diamond) in #272730: 210 texels; index texture sprites/courier.png#index
     sprites/courier.ogt (changed): its cells follow sprites/courier.png's conformed layout (222x66 texels)
     sprites/coin.png (unchanged): meets the contract as it is
     ui/logo.png (off): conform off (assets/ui/logo.png.ogt)
   ```

   Models, the textures on them, their clips and every flipbook carry their
   changes as notes on the asset beginning "conform:" (the triangles
   before and after against the budget, flat or smooth normals, the voxel
   edge, the bevel, a texture's new size and filter, a clip stepped to
   `stepFps`); one beginning "conform: warning:" says what was dropped
   (frames past `maxFrames`).
   `og cook --report out.md` tables every note per asset, and
   `og-out/cook-report.json` (and `--json`) carries the 2D entries as
   `conform`. A note shows on the cook that made the output: after
   `og cook` found nothing to do, change the look or the asset (or delete
   its outputs in `cooked/`) to see it again.
4. **Look at it in the game** (visual-acceptance §2): the conformed sprite
   or model in the play camera, beside the style's preview. A large change
   in the report (a sprite halved, a model cut to a fifth of its
   triangles) means the request missed the contract: fix the request and
   generate again rather than leaving the cook to rescue it.

**Opting out.** An asset that must stay exactly as made (a photograph, a
logo, licensed art, a hero model the style deliberately breaks) takes
`conform off`: in an image's import settings (the `type import` file
beside it), in a flipbook's, tile set's, 2D rig's or `type model`'s own
`.ogt`, or in a glTF's import settings. For 2D art, project.ogt's block
names paths at once:

```text
conform {
  off "sprites/photos/*" "ui/logo.png"
}
```

Asset ids, `*` within a path segment, `**` across, a trailing `/` a
folder. Every opt-out is listed by `og cook --report` with its reason. To
release one constraint for the whole game instead, set its field to zero
in the game's look (og-taste §12): `ArtShape { propFaces 0 }` keeps every
prop's triangles.

**Checking the style.** `og style check [project]` cooks the project and
measures every cooked asset, from `cooked/` itself (not the cook's
notes), against the contract of the look project.ogt names:

| Asset | Measured | Judged against |
|---|---|---|
| sprite (any texture of a `profile "2d"` project, or one cooked as pixel art) | distinct colours, palette hit rate (opaque texels within ΔE00 2 of a palette colour), texels per logical pixel and the half pixels off that grid, the line's width round the silhouette (per tile for a tile set) | `ArtPalette.maxColors`, `strict` (100 %), the project's grid (its sprites' most common), `ArtLine.width` |
| texture on a 3D surface | colours, palette hit rate, size, nearest or linear, mip levels | `maxColors`, `strict`, `ArtTexels.maxSize`, `filter`, `mips` |
| model | triangles drawn (each primitive times its nodes), flat-normal share, texel density (texels per metre, area-weighted median), untextured colours | `ArtShape.propFaces` (`characterFaces` for a skinned model), `normals flat` (99 %), `ArtTexels.density` (within ×2), `maxColors` |

A 2D rig's parts and a UI atlas keep their layout, so their line is not
judged, nor a UI atlas's grid (the UI draws at its own scale). A
lossy-encoded texture's colours are measured, not judged. The texel
density spread across the models is printed. Every opt-out is listed with
why; an opted-out asset is still measured, since it is still the game's
art.

Frames: name the screens the check captures in project.ogt, each a
command script that reaches it (a `screenshot` is added at its last step
when it has none):

```text
screens {
  title "0.5:screenshot"
  play "3.5:screenshot"
  results "1:key_down:Escape;1.05:key_up:Escape;2:screenshot"
}
```

or pass `--title`, `--play`, `--results` (and `--size 640x360`). Each is
run with `og run --headless --commands` into `og-out/style/<screen>/` and
measured: colours (and how few cover 95 % of the pixels), the palette hit
rate, edge density, the share of hard one-pixel steps (pixel art and hard
outlines are high, filtered and blurred edges low) and the frame's pixel
grid. `--no-frames` skips them.

`--critic` prints the style's three catalogue checks as written and the
frames' paths, and nothing that names the style (no name, slug or look
id): hand it to a blind critic, who answers each check from the frames.
`--json` prints one document (`assets`, `optOuts`, `frames`, `summary`).
The exit status is 0 when every asset meets the contract, 3 when one does
not (each failed number against its term), 1 when the cook or a capture
failed; a project whose project.ogt names no look has no contract, says
so and exits 0. `og doctor --project` carries one `style` line of it, and
`og build --publish` prints its summary as warnings: numbers cannot judge
a style (og-taste §11), so neither ever refuses a build. Run it before
calling a screen done, and read the frames yourself beside the style's
preview: the numbers say what is off, the eye says whether it reads.
