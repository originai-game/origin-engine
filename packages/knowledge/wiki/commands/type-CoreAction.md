<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="type-CoreAction"></a>

### `CoreAction` (enum)

The actions the engine ships. Arguments follow the action, separated by `:`.

See `testing §7`.

A string: one of the variants below. The zero value is `"screenshot"`.

| Variant | Doc |
|---|---|
| `"screenshot"` | `screenshot[:path][:full\|:N]`: capture the next frame. Without a renderer it writes nothing and logs, not as a warning, that the frame is unchecked. |
| `"listen"` | `listen[:path][:seconds]`: write the last seconds of rendered audio as a WAV. |
| `"touch_move_to"` | `touch_move_to:<ui id>`: move the pointer to a UI element. |
| `"touch_move"` | `touch_move:x,y`: move the pointer to CSS pixel coordinates. |
| `"touch_start"` | `touch_start`: press the pointer where it is. |
| `"touch_end"` | `touch_end`: release the pointer. |
| `"scroll"` | `scroll:dx,dy`: scroll at the pointer. |
| `"touch"` | `touch:<id>:<down\|move\|up>:<x,y \| ui id>`: a further touch (pointer 1 to 4) held while the pointer is busy elsewhere. |
| `"gamepad"` | `gamepad:<a+b+...\|->[@pad][:lx,ly[,rx,ry]][:lt,rt]`: set a pad's whole state (pad 0 unless `@1` to `@3` follows the buttons); `gamepad:off` disconnects it. |
| `"intent"` | `intent:<Event>[:<fields as JSON>]`: send a gameplay intent through og.input.intent's recorded input path for the next tick, its fields from the JSON object and the rest at their defaults (`intent:Buy:{"item":3}`). |
| `"key_down"` | `key_down:[Mod+]<Key>`: hold a key by its code, with the modifiers named before it (`Shift+ArrowLeft`, `Meta+ArrowLeft` for Command, apart from Ctrl). |
| `"key_up"` | `key_up:[Mod+]<Key>`: release a key, then the modifiers named before it. |
| `"type"` | `type:<text>`: type text, two ticks per character: its key press and the character as typed text (input.TextInput); a character no key types is typed text alone. The text runs to the `;` exactly, trailing space included; after it a text field takes typed text only, so a bare key_down types nothing. |
| `"compose"` | `compose:<text>`: an input method's composition so far (input.TextInput composing); a focused text field shows it underlined at the caret, `compose:` with nothing ends it. |
| `"commit"` | `commit:<text>`: an input method commits text; a focused text field inserts it and the composition ends. |
| `"move_to_text"` | `move_to_text:[=]text[@N]`: move the pointer to the UI element showing the text (exactly with `=`, the Nth with `@N`). |
| `"teleport"` | `teleport:<entity>:x,y,z`: set an entity's position. |
| `"frame_entity"` | `frame_entity:<entity>`: point the camera at an entity. |
| `"dump_world"` | `dump_world[:path]`: tables, systems and their matched counts. |
| `"dump_entity"` | `dump_entity:<name>[:path]`: one entity's components and transform. |
| `"dump_ui"` | `dump_ui[:path]`: the UI tree with bounds, and `style`, the configured UI style's name (empty for the engine's default style). |
| `"set"` | `set:<entity>.<component>.<field>=<value>` or `set:<Resource>.<field>=<value>`: a Tune write through the schema: a number, `true` or `false` for a bool, `x,y,z` for a vector, an enum's variant name or its index; refused outside the field's FieldOptions min/max, never clamped. |
| `"get"` | `get:<Resource>[.<field>]` or `get:<entity>[.<component>[.<field>]][:path]`: read through the schema as game code sees it (enums by variant name, og.Time whole, lights as drawn), logged and written as JSON to `get-<tick>.json` (more unnamed ones on one tick: `get-<tick>-2.json`, ...). |
| `"seed"` | `seed:<n>`: reseed the Rng resource. |
| `"pause"` | `pause`: hold the clock; advance still steps. |
| `"resume"` | `resume`: release the clock. |
| `"advance"` | `advance:<ticks>`: take fixed steps; the steps the script schedules for the same time after it run once it has stepped (a time is seconds: `0:advance:300;0:screenshot` shoots tick 300). |
| `"quit"` | `quit`: exit. |
| `"describe"` | `describe[:path]`: write the frame as data (probe.Description): the visible entities with screen boxes, depth order and occlusion from the entity-id pass, what each secondary camera (og.CameraView, og.Mirror) shows from its own pass, the UI elements with rects and text, the camera. |
| `"expect"` | `expect:<what>`: assert at this tick (probe.ExpectResult), an `entity:`, `event:`, `ui:text:`, `region:`, `frame:`, `state:` or `timings:` form. A failure warns with the tick and observed value; `og run --json` lists each result, exiting 1 on a failure. Frame checks need a renderer, else unchecked. |
| `"invariant"` | `invariant:<what>`: from now on, check an expectation (any `expect:` form but `region` and `frame`) after every tick: the first failing tick is an assertion violation with the tick and the observed value, which fails `og run --json` (exit 1) and the playtest. Playtest `invariants` are these steps. |
| `"save_scene"` | `save_scene[:path][:spawned]`: write each scene the world was built from back from it, rewriting only the lines whose values changed; `spawned` adds the entities spawned at run time. To each scene's source, or with a path the boot scene's to that file in the output directory, the others beside it. |
| `"help"` | `help`: list every action the channel knows, core and registered. |
| `"mouse_move"` | `mouse_move:dx,dy`: move the mouse by relative CSS pixels (movementX/Y, summed in og.Pointers.movement: mouse-look) with the pointer's position held, as a locked pointer moves. |
| `"pointer_lock"` | `pointer_lock:<on\|off>`: report the pointer locked or released (og.PointerLock.locked), as the web shell does from pointerlockchange; headless hosts never lock on their own. |
| `"apply_prefab"` | `apply_prefab:<entity>[:<Component>[.<field>]][:base]`: move an entity's prefab overrides (all, a component's or a field's; run-time too) to its node in the prefab's `.ogt`, out of the scene's instance block: the scene cooks the same world and every instance takes them; `base`: to a variant's base. |
| `"revert_prefab"` | `revert_prefab:<entity>[:<Component>[.<field>]]`: drop an entity's prefab overrides (all, a component's or one field) from its scene's instance block, and set the live values back to the prefab's. |

```ogt
enum CoreAction {
  doc "The actions the engine ships. Arguments follow the action, separated by `:`."
  guide "testing §7"
  variant screenshot "`screenshot[:path][:full|:N]`: capture the next frame. Without a renderer it writes nothing and logs, not as a warning, that the frame is unchecked."
  variant listen "`listen[:path][:seconds]`: write the last seconds of rendered audio as a WAV."
  variant touch_move_to "`touch_move_to:<ui id>`: move the pointer to a UI element."
  variant touch_move "`touch_move:x,y`: move the pointer to CSS pixel coordinates."
  variant touch_start "`touch_start`: press the pointer where it is."
  variant touch_end "`touch_end`: release the pointer."
  variant scroll "`scroll:dx,dy`: scroll at the pointer."
  variant touch "`touch:<id>:<down|move|up>:<x,y | ui id>`: a further touch (pointer 1 to 4) held while the pointer is busy elsewhere."
  variant gamepad "`gamepad:<a+b+...|->[@pad][:lx,ly[,rx,ry]][:lt,rt]`: set a pad's whole state (pad 0 unless `@1` to `@3` follows the buttons); `gamepad:off` disconnects it."
  variant intent "`intent:<Event>[:<fields as JSON>]`: send a gameplay intent through og.input.intent's recorded input path for the next tick, its fields from the JSON object and the rest at their defaults (`intent:Buy:{\"item\":3}`)."
  variant key_down "`key_down:[Mod+]<Key>`: hold a key by its code, with the modifiers named before it (`Shift+ArrowLeft`, `Meta+ArrowLeft` for Command, apart from Ctrl)."
  variant key_up "`key_up:[Mod+]<Key>`: release a key, then the modifiers named before it."
  variant type "`type:<text>`: type text, two ticks per character: its key press and the character as typed text (input.TextInput); a character no key types is typed text alone. The text runs to the `;` exactly, trailing space included; after it a text field takes typed text only, so a bare key_down types nothing."
  variant compose "`compose:<text>`: an input method's composition so far (input.TextInput composing); a focused text field shows it underlined at the caret, `compose:` with nothing ends it."
  variant commit "`commit:<text>`: an input method commits text; a focused text field inserts it and the composition ends."
  variant move_to_text "`move_to_text:[=]text[@N]`: move the pointer to the UI element showing the text (exactly with `=`, the Nth with `@N`)."
  variant teleport "`teleport:<entity>:x,y,z`: set an entity's position."
  variant frame_entity "`frame_entity:<entity>`: point the camera at an entity."
  variant dump_world "`dump_world[:path]`: tables, systems and their matched counts."
  variant dump_entity "`dump_entity:<name>[:path]`: one entity's components and transform."
  variant dump_ui "`dump_ui[:path]`: the UI tree with bounds, and `style`, the configured UI style's name (empty for the engine's default style)."
  variant set "`set:<entity>.<component>.<field>=<value>` or `set:<Resource>.<field>=<value>`: a Tune write through the schema: a number, `true` or `false` for a bool, `x,y,z` for a vector, an enum's variant name or its index; refused outside the field's FieldOptions min/max, never clamped."
  variant get "`get:<Resource>[.<field>]` or `get:<entity>[.<component>[.<field>]][:path]`: read through the schema as game code sees it (enums by variant name, og.Time whole, lights as drawn), logged and written as JSON to `get-<tick>.json` (more unnamed ones on one tick: `get-<tick>-2.json`, ...)."
  variant seed "`seed:<n>`: reseed the Rng resource."
  variant pause "`pause`: hold the clock; advance still steps."
  variant resume "`resume`: release the clock."
  variant advance "`advance:<ticks>`: take fixed steps; the steps the script schedules for the same time after it run once it has stepped (a time is seconds: `0:advance:300;0:screenshot` shoots tick 300)."
  variant quit "`quit`: exit."
  variant describe "`describe[:path]`: write the frame as data (probe.Description): the visible entities with screen boxes, depth order and occlusion from the entity-id pass, what each secondary camera (og.CameraView, og.Mirror) shows from its own pass, the UI elements with rects and text, the camera."
  variant expect "`expect:<what>`: assert at this tick (probe.ExpectResult), an `entity:`, `event:`, `ui:text:`, `region:`, `frame:`, `state:` or `timings:` form. A failure warns with the tick and observed value; `og run --json` lists each result, exiting 1 on a failure. Frame checks need a renderer, else unchecked."
  variant invariant "`invariant:<what>`: from now on, check an expectation (any `expect:` form but `region` and `frame`) after every tick: the first failing tick is an assertion violation with the tick and the observed value, which fails `og run --json` (exit 1) and the playtest. Playtest `invariants` are these steps."
  variant save_scene "`save_scene[:path][:spawned]`: write each scene the world was built from back from it, rewriting only the lines whose values changed; `spawned` adds the entities spawned at run time. To each scene's source, or with a path the boot scene's to that file in the output directory, the others beside it."
  variant help "`help`: list every action the channel knows, core and registered."
  variant mouse_move "`mouse_move:dx,dy`: move the mouse by relative CSS pixels (movementX/Y, summed in og.Pointers.movement: mouse-look) with the pointer's position held, as a locked pointer moves."
  variant pointer_lock "`pointer_lock:<on|off>`: report the pointer locked or released (og.PointerLock.locked), as the web shell does from pointerlockchange; headless hosts never lock on their own."
  variant apply_prefab "`apply_prefab:<entity>[:<Component>[.<field>]][:base]`: move an entity's prefab overrides (all, a component's or a field's; run-time too) to its node in the prefab's `.ogt`, out of the scene's instance block: the scene cooks the same world and every instance takes them; `base`: to a variant's base."
  variant revert_prefab "`revert_prefab:<entity>[:<Component>[.<field>]]`: drop an entity's prefab overrides (all, a component's or one field) from its scene's instance block, and set the live values back to the prefab's."
}
```

<sub>Module `commands` line 8</sub>
