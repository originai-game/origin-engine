---
name: og-build
description: The Origin Engine build loop — og cook, og run with a scripted command channel, screenshots and world dumps read as evidence, og dev hot reload, og test playtests, and og build for publishing. Use whenever a change to a project with a project.ogt must be run, observed or shipped (not a Godot project.godot).
---

The `og` tool is the whole loop: it cooks assets, runs the game headless on
the native host with a command script, serves the probe, reloads on save,
runs playtests and builds the publishable bundle. Nothing here needs a
browser or a Rust toolchain.

## The loop

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

   Core actions: `screenshot[:path][:full|:N]`, `touch_move_to:<ui id>`,
   `move_to_text:[=]text[@N]`, `touch_move:x,y`, `touch_start`, `touch_end`,
   `scroll:dx,dy`, `key_down:<Key>`, `key_up:<Key>`, `type:<text>`,
   `teleport:<entity>:x,y,z`, `frame_entity:<entity>`, `dump_world[:path]`,
   `dump_entity:<entity>[:path]`, `dump_ui[:path]`,
   `set:<entity>.<Component>.<field>=<value>`, `seed:<n>`, `pause`,
   `resume`, `advance:<ticks>`, `quit`, and `help` for the list including
   the game's own registered actions. Unknown actions warn and the script
   continues; read the warnings.
3. **Read what it wrote.** Screenshots are PNG files, dumps are JSON files
   (`<action>-<tick>.png|json` unless the step names a path); `og run`
   prints each path. Open the frame and describe what it shows; read
   `dump_world` for the entity count, the archetype tables and each system's
   matched count. A system that matched 0 entities is usually a missing
   component on the spawn, not a logic bug.
4. **Fix and repeat** with the same script and seed. Keep the script in the
   change so the next run is the same run.

## Live sessions: the probe and hot reload

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

## Tests

`og test .` runs the project's bun tests and every `tests/*.og.ts` playtest
script through `og run --headless --commands`. A playtest asserts on what the
game exposes (dumps, `state()` facts, entity fields), never on how the code
is written. A test that ran no checks fails. Run it before claiming a change
works.

## Evidence discipline

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

## Shipping

`og build .` bundles `src/` to `dist/game.js`, cooks, and copies the engine
runtime and wasm next to it. `og build . --publish` produces the published
form: the probe keeps only its player-facing members (hello, tick, state,
pick, entity, input, capture) and the command channel is gone. `og doctor`
names each missing tool (bun, the native addon, the wasm artifact) with its
fix.
