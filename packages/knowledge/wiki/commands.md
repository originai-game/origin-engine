<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `commands` module

The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands().

Core actions ship with the engine; a module or a game registers more. Unknown actions warn and continue. Not in publish builds.

Outputs are relative to the output directory: an absolute path, a `..` component or a name starting with `.` (kept for the harness, such as a playtest's kept store) is refused. A file an earlier step of the script wrote is never overwritten: the later step's is written as `<stem>-2.<ext>` with a warning naming both steps.

- **TypeScript**: functions are the namespace `og.commands`; components, resources and events are tokens on `og`.
- **Publish builds**: dropped (members may override).
- **Skill**: `og-build`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module commands {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 10260</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`CoreAction`](#type-CoreAction) | enum | The actions the engine ships. Arguments follow the action, separated by `:`. |
| [`CommandHandler`](#type-CommandHandler) | callback | A registered action. It runs at the start of the tick it was scheduled for. |
| [`CommandDesc`](#type-CommandDesc) | desc struct | How a registered action describes itself in `og run --help` and the wiki. |
| [`CommandStep`](#type-CommandStep) | struct | One parsed step of a command script. |
| [`CommandInfo`](#type-CommandInfo) | struct | One action as help lists it. |
| [`CommandsReport`](#type-CommandsReport) | struct | What scheduling a script did. |
| [`og.commands.register`](#fn-register) | function | Register an action into the command channel. GameDesc.commands does this for a game's own actions. |
| [`og.commands.parse`](#fn-parse) | function | Parse a script without scheduling it. |
| [`og.commands.help`](#fn-help) | function | Every action the channel knows: core actions in CoreAction order, then registered ones by name. |
| [`og.commands.schedule`](#fn-schedule) | function | Schedule a script on simulation time, from the current tick. |

## Functions

<a id="fn-register"></a>

### `og.commands.register`

Register an action into the command channel. GameDesc.commands does this for a game's own actions.

```ts
og.commands.register(name: string, handler: CommandHandler, desc?: CommandDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The action name, snake_case, such as `spawn_wave`. |
| `handler` | `CommandHandler` | no | What it does. |
| `desc` | `CommandDesc` | yes | Its usage line and doc. |

**Returns** `Result<void>`: Refused for a name a core action or another registration already uses.

Binding symbol `og_commands_register` · publish builds: dropped

```ogt
fn register {
  doc "Register an action into the command channel. GameDesc.commands does this for a game's own actions."
  param name String "The action name, snake_case, such as `spawn_wave`."
  param handler CommandHandler "What it does."
  param desc CommandDesc "Its usage line and doc."
  returns Result.Void "Refused for a name a core action or another registration already uses."
}
```

<sub>`api.ogt` line 10336</sub>

<a id="fn-parse"></a>

### `og.commands.parse`

Parse a script without scheduling it.

```ts
og.commands.parse(script: string): Result<readonly CommandStep[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `script` | `string` | no | `t:action[:args];...` with t in simulation seconds. |

**Returns** `Result<readonly CommandStep[]>`: The steps, or the first malformed step.

Binding symbol `og_commands_parse` · publish builds: dropped

```ogt
fn parse {
  doc "Parse a script without scheduling it."
  param script String "`t:action[:args];...` with t in simulation seconds."
  returns Result.List.CommandStep "The steps, or the first malformed step."
}
```

<sub>`api.ogt` line 10344</sub>

<a id="fn-help"></a>

### `og.commands.help`

Every action the channel knows: core actions in CoreAction order, then registered ones by name.

```ts
og.commands.help(world: World): readonly CommandInfo[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose channel is asked. |

**Returns** `readonly CommandInfo[]`.

Binding symbol `og_commands_help` · publish builds: dropped

```ogt
fn help {
  doc "Every action the channel knows: core actions in CoreAction order, then registered ones by name."
  param world World "The world whose channel is asked."
  returns List.CommandInfo
}
```

<sub>`api.ogt` line 10350</sub>

<a id="fn-schedule"></a>

### `og.commands.schedule`

Schedule a script on simulation time, from the current tick.

```ts
og.commands.schedule(world: World, script: string): Result<CommandsReport>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `script` | `string` | no | `t:action[:args];...` with t in simulation seconds. |

**Returns** `Result<CommandsReport>`: The report, or the first malformed step.

Binding symbol `og_commands_schedule` · publish builds: dropped

```ogt
fn schedule {
  doc "Schedule a script on simulation time, from the current tick."
  param world World "The world."
  param script String "`t:action[:args];...` with t in simulation seconds."
  returns Result.CommandsReport "The report, or the first malformed step."
}
```

<sub>`api.ogt` line 10356</sub>

## Types

<a id="type-CoreAction"></a>

### `CoreAction` (enum)

The actions the engine ships. Arguments follow the action, separated by `:`.

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
| `"key_down"` | `key_down:[Mod+]<Key>`: hold a key by its code, with the modifiers named before it (`Shift+ArrowLeft`, `Meta+ArrowLeft` for Command, apart from Ctrl). |
| `"key_up"` | `key_up:[Mod+]<Key>`: release a key, then the modifiers named before it. |
| `"type"` | `type:<text>`: type text, two ticks per character: its key press and the character as typed text (input.TextInput); a character no key types is typed text alone. The text runs to the `;` exactly, a trailing space included; after it a text field takes typed text only, so a bare key_down types nothing. |
| `"compose"` | `compose:<text>`: an input method's composition so far (input.TextInput composing); a focused text field shows it underlined at the caret, `compose:` with nothing ends it. |
| `"commit"` | `commit:<text>`: an input method commits text; a focused text field inserts it and the composition ends. |
| `"move_to_text"` | `move_to_text:[=]text[@N]`: move the pointer to the UI element showing the text (exactly with `=`, the Nth with `@N`). |
| `"teleport"` | `teleport:<entity>:x,y,z`: set an entity's position. |
| `"frame_entity"` | `frame_entity:<entity>`: point the camera at an entity. |
| `"dump_world"` | `dump_world[:path]`: tables, systems and their matched counts. |
| `"dump_entity"` | `dump_entity:<name>[:path]`: one entity's components and transform. |
| `"dump_ui"` | `dump_ui[:path]`: the UI tree with bounds. |
| `"set"` | `set:<entity>.<component>.<field>=<value>` or `set:<Resource>.<field>=<value>`: a Tune write through the schema: a number, `true` or `false` for a bool, `x,y,z` for a vector, an enum's variant name or its index; refused outside the field's FieldOptions min/max, never clamped. |
| `"get"` | `get:<Resource>[.<field>]` or `get:<entity>[.<component>[.<field>]][:path]`: read through the schema as game code sees it (an enum as its variant's name, `"music"`; og.Time whole, with rate and the time scale; the DirectionalLight og.DayNight drives with the colour, intensity and direction it is drawn with), logged and written as JSON to `get-<tick>.json` (more unnamed ones on one tick: `get-<tick>-2.json`, ...). |
| `"seed"` | `seed:<n>`: reseed the Rng resource. |
| `"pause"` | `pause`: hold the clock; advance still steps. |
| `"resume"` | `resume`: release the clock. |
| `"advance"` | `advance:<ticks>`: take fixed steps. |
| `"quit"` | `quit`: exit. |
| `"help"` | `help`: list every action the channel knows, core and registered. |

```ogt
enum CoreAction {
  doc "The actions the engine ships. Arguments follow the action, separated by `:`."
  variant screenshot "`screenshot[:path][:full|:N]`: capture the next frame. Without a renderer it writes nothing and logs, not as a warning, that the frame is unchecked."
  variant listen "`listen[:path][:seconds]`: write the last seconds of rendered audio as a WAV."
  variant touch_move_to "`touch_move_to:<ui id>`: move the pointer to a UI element."
  variant touch_move "`touch_move:x,y`: move the pointer to CSS pixel coordinates."
  variant touch_start "`touch_start`: press the pointer where it is."
  variant touch_end "`touch_end`: release the pointer."
  variant scroll "`scroll:dx,dy`: scroll at the pointer."
  variant touch "`touch:<id>:<down|move|up>:<x,y | ui id>`: a further touch (pointer 1 to 4) held while the pointer is busy elsewhere."
  variant gamepad "`gamepad:<a+b+...|->[@pad][:lx,ly[,rx,ry]][:lt,rt]`: set a pad's whole state (pad 0 unless `@1` to `@3` follows the buttons); `gamepad:off` disconnects it."
  variant key_down "`key_down:[Mod+]<Key>`: hold a key by its code, with the modifiers named before it (`Shift+ArrowLeft`, `Meta+ArrowLeft` for Command, apart from Ctrl)."
  variant key_up "`key_up:[Mod+]<Key>`: release a key, then the modifiers named before it."
  variant type "`type:<text>`: type text, two ticks per character: its key press and the character as typed text (input.TextInput); a character no key types is typed text alone. The text runs to the `;` exactly, a trailing space included; after it a text field takes typed text only, so a bare key_down types nothing."
  variant compose "`compose:<text>`: an input method's composition so far (input.TextInput composing); a focused text field shows it underlined at the caret, `compose:` with nothing ends it."
  variant commit "`commit:<text>`: an input method commits text; a focused text field inserts it and the composition ends."
  variant move_to_text "`move_to_text:[=]text[@N]`: move the pointer to the UI element showing the text (exactly with `=`, the Nth with `@N`)."
  variant teleport "`teleport:<entity>:x,y,z`: set an entity's position."
  variant frame_entity "`frame_entity:<entity>`: point the camera at an entity."
  variant dump_world "`dump_world[:path]`: tables, systems and their matched counts."
  variant dump_entity "`dump_entity:<name>[:path]`: one entity's components and transform."
  variant dump_ui "`dump_ui[:path]`: the UI tree with bounds."
  variant set "`set:<entity>.<component>.<field>=<value>` or `set:<Resource>.<field>=<value>`: a Tune write through the schema: a number, `true` or `false` for a bool, `x,y,z` for a vector, an enum's variant name or its index; refused outside the field's FieldOptions min/max, never clamped."
  variant get "`get:<Resource>[.<field>]` or `get:<entity>[.<component>[.<field>]][:path]`: read through the schema as game code sees it (an enum as its variant's name, `\"music\"`; og.Time whole, with rate and the time scale; the DirectionalLight og.DayNight drives with the colour, intensity and direction it is drawn with), logged and written as JSON to `get-<tick>.json` (more unnamed ones on one tick: `get-<tick>-2.json`, ...)."
  variant seed "`seed:<n>`: reseed the Rng resource."
  variant pause "`pause`: hold the clock; advance still steps."
  variant resume "`resume`: release the clock."
  variant advance "`advance:<ticks>`: take fixed steps."
  variant quit "`quit`: exit."
  variant help "`help`: list every action the channel knows, core and registered."
}
```

<sub>`api.ogt` line 10267</sub>

<a id="type-CommandHandler"></a>

### `CommandHandler` (callback)

A registered action. It runs at the start of the tick it was scheduled for.

Game code supplies this function.

```ts
type CommandHandler = (world: World, args: readonly string[]) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `args` | `readonly string[]` | no | The `:`-separated arguments after the action name. |

```ogt
callback CommandHandler {
  doc "A registered action. It runs at the start of the tick it was scheduled for."
  param world World "The world."
  param args List.String "The `:`-separated arguments after the action name."
}
```

<sub>`api.ogt` line 10299</sub>

<a id="type-CommandDesc"></a>

### `CommandDesc` (desc struct)

How a registered action describes itself in `og run --help` and the wiki.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `usage` | `string` | `""` | — | — | — | The argument form, such as `<count>`. |
| `doc` | `string` | `""` | — | — | — | One sentence on what it does. |

```ogt
struct CommandDesc {
  doc "How a registered action describes itself in `og run --help` and the wiki."
  desc true
  field usage String "The argument form, such as `<count>`."
  field doc String "One sentence on what it does."
}
```

<sub>`api.ogt` line 10305</sub>

<a id="type-CommandStep"></a>

### `CommandStep` (struct)

One parsed step of a command script.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `at` | `number` | `0` | — | `s` | — | When it runs, in simulation seconds from the start of the script. |
| `action` | `string` | `""` | — | — | — | The action name. |
| `args` | `readonly string[]` | `[]` | — | — | — | The arguments. |

```ogt
struct CommandStep {
  doc "One parsed step of a command script."
  field at f32 "When it runs, in simulation seconds from the start of the script." {
    unit s
  }
  field action String "The action name."
  field args List.String "The arguments."
}
```

<sub>`api.ogt` line 10312</sub>

<a id="type-CommandInfo"></a>

### `CommandInfo` (struct)

One action as help lists it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | The action name. |
| `usage` | `string` | `""` | — | — | — | Its argument form, such as `screenshot[:path][:full\|:N]`. |
| `doc` | `string` | `""` | — | — | — | One sentence on what it does. |
| `core` | `boolean` | `false` | — | — | — | Shipped with the engine (true) or registered by a module or a game. |

```ogt
struct CommandInfo {
  doc "One action as help lists it."
  field name String "The action name."
  field usage String "Its argument form, such as `screenshot[:path][:full|:N]`."
  field doc String "One sentence on what it does."
  field core bool "Shipped with the engine (true) or registered by a module or a game."
}
```

<sub>`api.ogt` line 10321</sub>

<a id="type-CommandsReport"></a>

### `CommandsReport` (struct)

What scheduling a script did.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheduled` | `number` | `0` | — | — | — | Steps scheduled. |
| `untilTick` | `number` | `0` | — | — | — | The tick of the last step. |
| `warnings` | `readonly string[]` | `[]` | — | — | — | Unknown actions and other skipped steps, each naming the fix. |

```ogt
struct CommandsReport {
  doc "What scheduling a script did."
  field scheduled u32 "Steps scheduled."
  field untilTick u32 "The tick of the last step."
  field warnings List.String "Unknown actions and other skipped steps, each naming the fix."
}
```

<sub>`api.ogt` line 10329</sub>
