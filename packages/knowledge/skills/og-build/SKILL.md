---
name: og-build
description: The Origin Engine build loop — og cook, og run with a scripted command channel (inputs, captures, dumps, get and set), screenshots and world dumps read as evidence, og dev hot reload, og test playtests, og polish's taste and feel scores (the one list of what it measures and where each finding is fixed), and og build for publishing. Use whenever a change to a project with a project.ogt must be run, observed, scored or shipped (not a Godot project.godot).
---

The `og` tool is the whole loop: it cooks assets, runs the game headless on
the native host with a command script, serves the probe, reloads on save,
runs playtests, scores them and builds the publishable bundle. Nothing here
needs a browser or a Rust toolchain. The command channel is
`reference/commands.md`, the probe `reference/probe.md`.

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
   (a second finger), `gamepad:<a+b+...|->[:lx,ly][:lt,rt]`, `scroll:dx,dy`,
   `key_down:<Key>`, `key_up:<Key>`, `type:<text>`,
   `teleport:<entity>:x,y,z`, `frame_entity:<entity>`, `dump_world[:path]`,
   `dump_entity:<entity>[:path]`, `dump_ui[:path]`,
   `get:<Resource>[.<field>]` or `get:<entity>[.<Component>[.<field>]]`
   (logs the value and writes `get-<tick>.json`),
   `set:<entity>.<Component>.<field>=<value>` or
   `set:<Resource>.<field>=<value>` (`set:og.Camera.trauma=0.5`,
   `set:Game.coins=10`), `seed:<n>`, `pause`, `resume`, `advance:<ticks>`,
   `quit`, and `help` for the list. Modules add their own: `dump_ai` and
   `dump_nav` (og-ai §5), `dump_platform` (og-platform-sdk §4), `locale:<tag>`
   (og-i18n §6). Unknown actions warn and the script continues; read the
   warnings.
3. **Read what it wrote.** Screenshots are PNG files, dumps are JSON files
   (`<action>-<tick>.png|json` unless the step names a path); `og run`
   prints each path. Open the frame and describe what it shows; read
   `dump_world` for the entity count, the archetype tables and each system's
   matched count. A system that matched 0 entities is usually a missing
   component on the spawn, not a logic bug.
4. **Fix and repeat** with the same script and seed. Keep the script in the
   change so the next run is the same run.

A game's own actions (`GameDesc.commands`, og-game §3) join the list; give
each its usage and one sentence in `GameDesc.commandHelp`
(`{ spawn_wave: { usage: "spawn_wave:<count>", doc: "Spawns a wave now." } }`)
so `help` explains it. An entry naming no action is logged when the game
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
how the code is written. A test that ran no checks fails. Run it before
claiming a change works.

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
| taste | Contrast | each text element's rectangle in a shot: the ground (its median) against its ink (the 1 % farthest pixels) | 4.5:1 | og-taste §3 |
| taste | Text size | the tallest run of ink rows in each text element, CSS px | 9 px, a 13 px face's capitals | og-taste §4, §10 |
| taste | HUD occupancy | the union of leaf elements on gameplay shots (a shot with three text elements in the central third counts as a menu) | at most 15 % of the surface and 5 % of the central third | og-taste §2 |
| taste | Screen transitions | a change of three or more elements with UI motion or a tween in the 20 ticks around it | every change | og-taste §5 |
| taste | UI sounds | a press that changed the UI and started a voice on the ui bus within 8 ticks | every press | og-taste §6, og-audio §3 |
| taste | Locale keys | `og cook`'s warnings | none | og-i18n §1 |
| feel | Input latency | ticks from the tick an input lands to the first thing seen or heard to change (a named entity accelerating or changing clip, the UI, a sound, a tween, a floating text, shake, hit stop, a particle burst); a press answered on its release counts from the release | 0–1 ticks | og-feel §1 |
| feel | Hit feedback | each damage moment (a fact named `hp`, `health`, `lives`… falling, or `hits`, `damage`, `kills`… rising): hit stop, shake, floating text and a sound within 8 ticks | three of the four | og-feel §3, §4, §7, og-audio §1 |
| feel | Idle frames | ticks after the first ten where nothing moves or animates, no UI moves, no sound starts or plays, no tween, text or particle runs | at most 10 % | og-feel §9, og-taste §5 |
| feel | Sound on input | inputs answered by a sound within 6 ticks | every action | og-audio §1 |
| feel | Frame time | step + render, p95, on the tier it ran on (`metal` on a Mac, `vulkan-cpu` on CI's lavapipe) | 8.3 ms (half the frame) | og-game §14 |

The report also lists clipped samples in the mix (og-audio §4). Contrast and
text size read text only: icons and images (their "text" is a picture's
name), disabled controls and what they hold (WCAG exempts them) and the
slivers a scroll container cuts are left out, and each element is measured
at the pixels of the surface `dump_ui` reports, so a phone preview
(390×844 at 3×) is read at its own size. Facts are the game's `facts`
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
pick, entity, input, capture) and the command channel is gone. `og doctor`
names each missing tool (bun, the native addon, the wasm artifact) with its
fix.
