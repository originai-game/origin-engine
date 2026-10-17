# og-probe

The command channel and the probe (design §3.3, §3.4; IDL `commands` and
`probe` in `crates/og-api/api.ogt`), over a world the host owns. It
implements `RuntimeProbe` protocol 5 (`origin/packages/contracts/src/probe.ts`)
plus the engine's wire members, served as JSON-RPC 2.0 over a loopback
WebSocket on native and through a transport-free `call(json) -> json` the
web host drives from its request ring. It depends on `og-core` and `og-ecs`
only: frames, the UI tree and bundle swaps come in through traits.

## API map

| Item | Entry points |
|---|---|
| `Probe` | `Probe::new(&mut world)` (registers `og.Input`), `with_input(world, sink)`; hooks `set_capture`, `set_ui`, `set_reloader`, `set_facts`, `set_timings`, `set_engine_version`; `names()` resolves ids over the world's names |
| driving | `frame(world, due)`, `step(world)`, `pump(world)`, `paused()`, `quit_requested()`, `take_outputs()`, `take_log()` |
| command channel | `schedule(world, script) -> CommandsReport`, `register_command(name, CommandDesc, handler)`, `help()`, `command::parse`, `command::CORE_ACTIONS` |
| probe members | `hello`, `tick`, `state`, `advance`, `pause`, `resume`, `input`, `accepted`, `pick`, `entity`, `bounds`, `patch`, `run`, `capture`, `sweep`, `reload`, `snapshot`, `restore`, `dump`, `commands` as typed methods; `invoke(world, method, params)` by name |
| JSON-RPC | `Probe::call(world, frame) -> frame`, `Probe::answer(world, &Call)`, `rpc::parse`, `rpc::{PARSE_ERROR, INVALID_REQUEST, METHOD_NOT_FOUND, INVALID_PARAMS, INTERNAL_ERROR}` |
| WebSocket (feature `ws`, native) | `server::ProbeServer::start(port)` (`None` for 0), `bind(addr)`, `poll()`, `Probe::serve(world, &server)`, `DEFAULT_PORT = 47800`, `PATH = "/probe"` |
| hooks | `Capture` (`tier`, `target_size`, `capture`, `pick`), `UiTree` (`elements`) + `StaticUi`, `Reloader` (`rebuild -> Rebuilt { world, ms }`), `InputSink` + `WorldInput` |
| ids | `Names`: an entity's probe id is its name, or `#<index>.<generation>` |

## What a host provides

1. **The loop.** Create the probe once per world, after registering the
   game's types and before adding systems that read `og.Input` and before the
   first snapshot. Drive every tick through `probe.frame(world, due)` (or
   `step`): due command steps run at the start of their tick, then the world
   steps, then input edges expire. A tick taken with `World::step` directly
   skips both. `frame` takes no ticks while `paused()`; stop when
   `quit_requested()`.
2. **Calls between ticks.** Native: `ProbeServer::start(port)` and
   `probe.serve(world, &server)` once per frame; the socket threads never
   touch the world. Web: hand each request-ring frame to `probe.call(world,
   text)` and post the answer. Never call during a system.
3. **Outputs.** The probe writes no files: `take_outputs()` yields
   `Output { tick, action, path, payload }` (`Png`, `Json`, `Text`) with
   `file_name()` (the script's path or `<action>-<tick:06>.<ext>`) and
   `bytes()`; `take_log()` yields warnings, each naming the fix.
4. **Hooks, when it has them.** `Capture` over og-render (render the world
   as of the current tick offscreen, PNG bytes; `pick` defaults to `None`
   until the renderer picks), `UiTree` from the UI module, `Reloader` for the
   bundle swap, `set_facts` for `GameDesc.facts`, and `set_timings` with the
   measured step and render milliseconds. Systems and names come from the
   world: `dump_world` reads `World::system_info`, and the host names each
   named spawn with `World::set_name`.

The probe is not `Send` (hooks and handlers may hold JavaScript values):
create it on the engine thread. It reads no clock.

## Command grammar

```text
script := step (';' step)*        whitespace around steps and empty steps are ignored
step   := time ':' action (':' arg)*
time   := simulation seconds, finite, ≥ 0, never below the previous step's
action := [a-z_][a-z0-9_]*
arg    := text; '\' escapes the next character ('\:' and '\;' are literal)
```

A step at `t` runs at the start of tick `now + round(t · tickRate)`, where
`now` is the tick the script was scheduled at. Steps run in time order, then
in script order; an `advance` runs the steps that fall due on its way. While
the clock is held only `advance` moves simulation time, so later steps wait
for it. A malformed script is refused whole with its first bad step
(`ParseError { step, text, reason }`). An unknown action is reported in the
`CommandsReport` warnings, kept (a module may register it before it runs),
and when it runs still unknown, warns and the script continues. `type` and
`set` rejoin their arguments with `:`.

| Action | Does | Without its hook |
|---|---|---|
| `screenshot[:path][:full\|:N]` | capture this tick: the render target's size (`full` the same), `N` a long edge of N px | warns |
| `touch_move_to:<ui id>` | pointer to the centre of an element: an id (topmost first) or a `Parent/Child` path | warns "target not found" |
| `move_to_text:[=]text[@N]` | pointer to the element showing the text: substring, `=` exact, Nth match (0-based, negative from the end), topmost first | warns |
| `touch_move:x,y` / `touch_start` / `touch_end` / `scroll:dx,dy` | pointer move, left press and release where the pointer is, wheel at the pointer | — |
| `key_down:<Key>` / `key_up:<Key>` / `type:<text>` | `KeyboardEvent.code` keys; `type` presses one key per character, two ticks apart, Shift for capitals and symbols (US layout) | — |
| `teleport:<entity>:x,y[,z]` | write `og.Transform.position` (two numbers keep z) | — |
| `frame_entity:<entity>` | turn `og.Camera.quaternion` so its -Z looks at the entity | warns without `og.Camera` |
| `dump_world[:path]` / `dump_entity:<entity>[:path]` / `dump_ui[:path]` | JSON outputs, as `dump` returns them | `dump_ui` is empty |
| `set:<entity>.<component>.<field>=<value>` | write any field through the schema (`1.5`, `1,2,3`); range-checked, never clamped | — |
| `seed:<n>` / `pause` / `resume` / `advance:<ticks>` / `quit` | reseed the world's `Rng`, hold and release the clock, take ≤ 3600 ticks now, end the run | — |
| `help` | a `Text` output listing every action, core and registered | — |

A target miss (`touch_move_to`, `move_to_text`) also cancels the bare
`touch_start`/`touch_end` after it, so a missed button is not clicked
somewhere else.

**Registry.** `register_command(name, CommandDesc { usage, doc }, handler)`
adds an action for a module (the UI, the renderer) or a game
(`GameDesc.commands`). The handler gets `(&mut World, &[String])` and runs at
the start of its tick; `Err(reason)` is a warning. Refused for a non
snake_case name, a core name or a taken name. `help()` lists core actions in
`CORE_ACTIONS` order, then registered ones by name.

## Probe members and their refusals

Probe ids are entity names, or `#<index>.<generation>`. A refusal is
`false` or `null` in `result`; an `error` is only a malformed call, an
unknown method, or a fault.

| Member | Params | Result | Refusal (`result`) |
|---|---|---|---|
| `hello` | — | `{protocol: 5, wire: 1, substrate: "origin-engine", engine, ticksPerSecond, tier, build, members}` | — (`tier: "none"` without a renderer) |
| `tick` | — | fixed steps since load | — |
| `state` | — | `{tick, paused, seed, timings: {step, render, frameMs}, entities: [{id}], facts}` | — |
| `advance` | `steps` 0–3600 | the tick after; runs due command steps | — |
| `pause` / `resume` | — | `true` | — |
| `input` | `event` (`InjectedInput`) | `true` | `false` for an event of no known shape or one the input layer refuses (an unknown key code) |
| `accepted` | — | events the input layer accepted | — |
| `pick` | `x`, `y` | `{entity, world, source}` | `null` when nothing is under the point, or no renderer picks yet |
| `entity` | `id` | tunable fields as `{"Component.field": value}` | `null` for an unknown or dead entity |
| `bounds` | `id` | `{"Component.field": {path, label?, min?, max?}}` from the schema | `null` for an unknown entity |
| `patch` | `id`, `parameter`, `value` | `true`, applied before the next tick | `false` for an unknown entity, component or field, a field without `tunable`, a component the entity lacks, a value of another kind, non-finite, non-integral for an integer, or outside `[min, max]` |
| `run` | `program` (≤ 256 ops) | `{results, refused, tick}` | `refused: i` at the first `false`, `entity` `null`, unknown op or malformed op (`advance` takes 1–3600); more than 256 ops: `refused: 0`, nothing runs |
| `capture` | `width`, `height` (both 1–4096, or both 0/absent) | `{tick, width, height, png}` (base64) | `null` without a renderer |
| `sweep` | `id`, `parameter`, `values` (1–64), `steps`, `width`, `height` | `{frames: [{value, tick, png}]}`; per value: restore the call's snapshot, patch, advance (no command steps), capture; the world is restored afterwards | `null` for an unknown entity or parameter, a non-tunable one, any refused value, or no renderer |
| `reload` | — | `{tick, ms, migrated, dropped, warnings}` | `null` without a `Reloader`, when the bundle fails to load, or when a type changed layout; the old bundle keeps running and the reason is logged |
| `snapshot` | — | `{id, tick, hash, bytes}` (the last 16 are kept) | — |
| `restore` | `id` | `true` | `false` for an unknown or dropped id, or a world it no longer fits |
| `dump` | `what`: `world`, `entity` (+ `id`), `ui` | `{kind: "world", tick, hash, entities, archetypes: [{components, count}], systems: [{name, matched, reads, writes, ms}], resources}` / `{kind: "entity", id, name, components}` / `{kind: "ui", tick, elements}` | `null` for an unknown entity |
| `commands` | `script` | `{scheduled, untilTick, warnings}` | `null` for a malformed script (the parse error is logged) |

Errors: `-32700` unparsable JSON; `-32600` a batch, no numeric or string id,
`jsonrpc` not `"2.0"`, a non-string method, a binary frame; `-32602`
params not an object or a param of the wrong shape (each listed above);
`-32601` an unknown method, including every member a `publish` build drops;
`-32603` a fault: a system that failed during `advance`, `run` or `sweep`
(the message names the tick and the system), or a panic. Every call is
answered, with its own id.

Notes: `dump_world` reports `reads`, `writes` and `matched` (entities its
queries match now) only for systems the host declared, `null` otherwise;
`ms` is `null` (the probe reads no clock). Archetypes are the non-empty
tables in creation order. An entity dump has every field of every component;
its transform is `components["og.Transform"]`.

## Input

`WorldInput`, the default `InputSink`, writes the reflected `og.Input`
resource: pointer, wheel, button and 128-bit key sets over
`input::KEY_CODES`, with one-tick edges (`press` and `click` release after the
next tick). Injected input is therefore world state: in snapshots, in the
world hash, replayed. The input module replaces it by implementing
`InputSink` over its own resource and the shell's ring (`Probe::with_input`).

## Builds

- Features: `ws` (default) the native WebSocket server, ignored on wasm32;
  `publish` keeps `hello`, `tick`, `state`, `pick`, `entity`, `input`,
  `capture` and answers `-32601` for the rest. `hello.build` is `publish`,
  `dev` (debug assertions) or `release`.
- `cargo check -p og-probe --target wasm32-unknown-unknown` builds with or
  without default features.

## Tests

`cargo test -p og-probe`:

- `tests/conformance.rs`: every member, refusal and error code through the
  JSON entry (writes `$CARGO_TARGET_TMPDIR/conformance-transcript.json`); a
  sweep of 8 values whose world hash is unchanged afterwards; a failing
  system as `-32603`; reload with the same bundle, a dropped component, a
  changed layout and a bundle that fails to load.
- `tests/commands.rs`: the scripted run
  `0.5:seed:7;1.0:advance:30;1.5:dump_world;2.0:quit` (writes
  `scripted-run-dump-world.json`), `help`, unknown actions, the grammar,
  steps inside `advance`, pointer and key actions, `teleport`/`set`/
  `frame_entity`/`screenshot`.
- `tests/websocket.rs`: a real tungstenite client against `ProbeServer`.
- `tests/publish.rs` (`--features publish`): the trim.

The world they drive is `tests/support/mod.rs`.
