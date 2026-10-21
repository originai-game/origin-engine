<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `game` module

The game entry: what the bundle's default export declares.

- **TypeScript**: functions are members of the root object `og` (`og.<fn>`); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module game {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 457</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`FactsFn`](#type-FactsFn) | callback | The game's own facts for the probe's state(): a small JSON object, computed from the world without changing it. |
| [`StartFn`](#type-StartFn) | callback | The game's setup: called once before the first tick, after GameDesc.scene has spawned. Spawn, insert, load and set resources here; systems then only read and write their views. |
| [`GameDesc`](#type-GameDesc) | desc struct | A game: its tick, scenes, systems and commands. |
| [`Game`](#type-Game) | interface | A declared game, the bundle's default export. |
| [`og.game`](#fn-game) | function | Declare the game. The bundle's default export is its result. |

## Functions

<a id="fn-game"></a>

### `og.game`

Declare the game. The bundle's default export is its result.

```ts
og.game(desc?: GameDesc): Game
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `GameDesc` | yes | Tick, scenes, systems and commands. |

**Returns** `Game`.

Binding symbol `og_game_game` · publish builds: kept

```ogt
fn game {
  doc "Declare the game. The bundle's default export is its result."
  param desc GameDesc "Tick, scenes, systems and commands."
  returns Game
}
```

<sub>`api.ogt` line 509</sub>

## Interfaces

<a id="type-Game"></a>

### `Game` (interface)

A declared game, the bundle's default export.

```ogt
interface Game {
  doc "A declared game, the bundle's default export."
  # members follow, each quoted below
```

<sub>`api.ogt` line 503</sub>

<a id="member-Game-tick"></a>

#### `Game.tick` (property)

GameDesc.tick, or 60 when it is unset (the engine then runs at project.ogt's `tick` when the host passes the project, as og run and og build do).

```ts
readonly tick: number
```

Binding symbol `og_game_get_tick` · publish builds: kept

```ogt
prop tick u32 "GameDesc.tick, or 60 when it is unset (the engine then runs at project.ogt's `tick` when the host passes the project, as og run and og build do)."
```

<sub>`api.ogt` line 505</sub>

<a id="member-Game-systems"></a>

#### `Game.systems` (property)

The systems in declaration order.

```ts
readonly systems: readonly System[]
```

Binding symbol `og_game_get_systems` · publish builds: kept

```ogt
prop systems List.System "The systems in declaration order."
```

<sub>`api.ogt` line 506</sub>

## Types

<a id="type-FactsFn"></a>

### `FactsFn` (callback)

The game's own facts for the probe's state(): a small JSON object, computed from the world without changing it.

Game code supplies this function.

```ts
type FactsFn = (world: World) => unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world, read only. |

```ogt
callback FactsFn {
  doc "The game's own facts for the probe's state(): a small JSON object, computed from the world without changing it."
  param world World "The world, read only."
  returns Json
}
```

<sub>`api.ogt` line 462</sub>

<a id="type-StartFn"></a>

### `StartFn` (callback)

The game's setup: called once before the first tick, after GameDesc.scene has spawned. Spawn, insert, load and set resources here; systems then only read and write their views.

Game code supplies this function.

```ts
type StartFn = (world: World) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

```ogt
callback StartFn {
  doc "The game's setup: called once before the first tick, after GameDesc.scene has spawned. Spawn, insert, load and set resources here; systems then only read and write their views."
  param world World "The world."
}
```

<sub>`api.ogt` line 468</sub>

<a id="type-GameDesc"></a>

### `GameDesc` (desc struct)

A game: its tick, scenes, systems and commands.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `60` | 1 … 240 | `hz` | — | The fixed tick rate. Left unset, project.ogt's `tick` applies (else 60); tick, seed, scenes and scene set both here and in project.ogt to different values refuse the engine, naming both. |
| `seed` | `number` | `0` | — | — | — | The project seed every Rng stream derives from; left unset, project.ogt's `seed` (else 0). A replay overrides it. |
| `scenes` | `Readonly<Record<string, string>>` | `{}` | — | — | — | Scene names to `.ogt` paths, as `{ main: "scenes/main.ogt" }`; left unset, project.ogt's `scenes` block. |
| `scene` | `string` | `"main"` | — | — | — | The scene loaded first; left unset, project.ogt's `scene` (else main). |
| `systems` | `readonly System[]` | `[]` | — | — | — | The systems, in declaration order before `after`/`before` constraints apply. |
| `resources` | `readonly AnyResource[]` | `[]` | — | — | — | The game's own resources, created at their defaults. |
| `commands` | `Readonly<Record<string, CommandHandler>>` | `{}` | — | — | — | Game actions registered into the command channel, by action name. |
| `commandHelp` | `Readonly<Record<string, CommandDesc>>` | `{}` | — | — | — | Each game action's argument form and one sentence, by its name in commands, for `help`: `{ place: { usage: "<x>:<y>", doc: "Move the hero there." } }`. An action without an entry is listed by name alone. |
| `maxCatchUp` | `number` | `4` | 1 … 60 | `ticks` | — | The most ticks one frame may run to catch up with the clock; the accumulator drops the rest. |
| `facts` | `FactsFn \| null` | `null` | — | — | — | The game's own facts for the probe's state(). |
| `start` | `StartFn \| null` | `null` | — | — | — | Called once before the first tick, after the first scene spawned. A throw fails the run: no tick runs on the half-built world, `og run`, `og test` and `og dev` exit non-zero naming the error with its stack, and the web shell shows it. |
| `render` | `RenderOptions` | `{}` | — | — | — | How the renderer is created: antialiasing and shadow map size. Read once, when the host attaches the renderer. |
| `look` | `string` | `""` | — | — | — | A look (`looks/<name>.ogt`, og.load.look) applied after the first scene spawned and before start runs, so start may still change any of it; its antialias is the renderer's when render leaves antialias out. Empty applies none. |

```ogt
struct GameDesc {
  doc "A game: its tick, scenes, systems and commands."
  desc true
  field tick u32 "The fixed tick rate. Left unset, project.ogt's `tick` applies (else 60); tick, seed, scenes and scene set both here and in project.ogt to different values refuse the engine, naming both." {
    default 60
    min 1
    max 240
    unit hz
  }
  field seed u32 "The project seed every Rng stream derives from; left unset, project.ogt's `seed` (else 0). A replay overrides it."
  field scenes Map.String "Scene names to `.ogt` paths, as `{ main: \"scenes/main.ogt\" }`; left unset, project.ogt's `scenes` block."
  field scene String "The scene loaded first; left unset, project.ogt's `scene` (else main)." {
    default "main"
  }
  field systems List.System "The systems, in declaration order before `after`/`before` constraints apply."
  field resources List.AnyResource "The game's own resources, created at their defaults."
  field commands Map.CommandHandler "Game actions registered into the command channel, by action name."
  field commandHelp Map.CommandDesc "Each game action's argument form and one sentence, by its name in commands, for `help`: `{ place: { usage: \"<x>:<y>\", doc: \"Move the hero there.\" } }`. An action without an entry is listed by name alone."
  field maxCatchUp u32 "The most ticks one frame may run to catch up with the clock; the accumulator drops the rest." {
    default 4
    min 1
    max 60
    unit ticks
  }
  field facts Option.FactsFn "The game's own facts for the probe's state()."
  field start Option.StartFn "Called once before the first tick, after the first scene spawned. A throw fails the run: no tick runs on the half-built world, `og run`, `og test` and `og dev` exit non-zero naming the error with its stack, and the web shell shows it."
  field render RenderOptions "How the renderer is created: antialiasing and shadow map size. Read once, when the host attaches the renderer."
  field look String "A look (`looks/<name>.ogt`, og.load.look) applied after the first scene spawned and before start runs, so start may still change any of it; its antialias is the renderer's when render leaves antialias out. Empty applies none."
}
```

<sub>`api.ogt` line 473</sub>
