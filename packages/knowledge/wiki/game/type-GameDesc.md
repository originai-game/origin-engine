<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-GameDesc"></a>

### `GameDesc` (desc struct)

A game: its tick, scenes, systems and commands.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `60` | 1 … 240 | `hz` | — | The fixed tick rate. Left unset, checked project.ts startup data applies (else 60); conflicting explicit values in GameDesc and startup data refuse the engine, naming both. |
| `seed` | `number` | `0` | — | — | — | The seed every Rng stream derives from; left unset, checked project.ts startup seed (else 0). CreateOptions.seed overrides it for a replay. |
| `scenes` | `Readonly<Record<string, string>>` | `{}` | — | — | — | Scene names to cooked asset IDs, as `{ harbor: "scenes/harbor.ogt" }`; left unset, checked project.ts startup scenes. |
| `scene` | `string` | `"main"` | — | — | — | The scene loaded first; left unset, checked project.ts startup scene (else main). |
| `systems` | `readonly System[]` | `[]` | — | — | — | The systems, in declaration order before `after`/`before` constraints apply. |
| `hooks` | `readonly Hook[]` | `[]` | — | — | — | The hooks (og.defineHook), in the order they run. |
| `states` | `readonly StateDesc[]` | `[]` | — | — | — | The state machines systems bind to (SystemDesc.state). |
| `resources` | `readonly AnyResource[]` | `[]` | — | — | — | The game's own resources, created at their defaults. |
| `commands` | `Readonly<Record<string, CommandHandler>>` | `{}` | — | — | — | Game actions registered into the command channel, by action name. |
| `commandHelp` | `Readonly<Record<string, CommandDesc>>` | `{}` | — | — | — | Each game action's argument form and one sentence, by its name in commands, for `help`: `{ place: { usage: "<x>:<y>", doc: "Move the hero there." } }`. An action without an entry is listed by name alone. |
| `maxCatchUpTicks` | `number` | `4` | 1 … 60 | `ticks` | — | The most ticks one frame may run to catch up with the clock; the accumulator drops the rest. |
| `facts` | `FactsFn \| null` | `null` | — | — | — | The game's own facts for the probe's state(). |
| `start` | `StartFn \| null` | `null` | — | — | — | Called once before the first tick, after the first scene spawned. A throw fails the run: no tick runs on the half-built world, `og run`, `og test` and `og dev` exit non-zero naming the error with its stack, and the web shell shows it. |
| `ui` | `UiSetupFn \| null` | `null` | — | — | — | Register retained UI after start and after a hot-reload restore; never reruns start. The engine checks world_hash before and after: duplicate IDs, incompatible store types and gameplay writes fail registration. A fresh-process restore runs start, then ui, then restores named presentation values. |
| `render` | `RenderOptions` | `{}` | — | — | — | How the renderer is created: antialiasing and shadow map size. Read once, when the host attaches the renderer. |
| `saveVersion` | `number` | `1` | ≥ 1 | — | — | The game's save version, written into every save: raise it when a release changes a saved type, and add the step from the old version to GameDesc.migrations. Left unset, checked project.ts startup saveVersion (else 1); conflicting explicit values refuse the engine. |
| `migrations` | `readonly SaveMigration[]` | `[]` | — | — | — | The game's explicit save edits: world.restore runs each one whose `from` lies between the save's version and saveVersion, in order. The save must have the current engine version and og-save:4 envelope; other versions are refused, never converted. Engine records and state are never upgraded. |
| `explore` | `ExploreDesc` | `{}` | — | — | — | How `og explore` searches this game's states: what a cell reads and which inputs it presses. Left out, cells come from every field of the game's resources and components and the inputs from its own button actions and none of the intents. Read by the tool only; the engine and the world never see it. |
| `look` | `string` | `""` | — | — | — | A cooked asset ID (og.load.look), applied after the first scene spawned, before start; its antialias applies when render leaves it out. Left unset, checked startup look from og-project.json (whose art contract the cook reads), else none; conflicting explicit values refuse the engine. |

```ogt
struct GameDesc {
  doc "A game: its tick, scenes, systems and commands."
  desc true
  field tick u32 "The fixed tick rate. Left unset, checked project.ts startup data applies (else 60); conflicting explicit values in GameDesc and startup data refuse the engine, naming both." {
    default 60
    min 1
    max 240
    unit hz
  }
  field seed u32 "The seed every Rng stream derives from; left unset, checked project.ts startup seed (else 0). CreateOptions.seed overrides it for a replay."
  field scenes Map.String "Scene names to cooked asset IDs, as `{ harbor: \"scenes/harbor.ogt\" }`; left unset, checked project.ts startup scenes."
  field scene String "The scene loaded first; left unset, checked project.ts startup scene (else main)." {
    default "main"
  }
  field systems List.System "The systems, in declaration order before `after`/`before` constraints apply."
  field hooks List.Hook "The hooks (og.defineHook), in the order they run."
  field states List.StateDesc "The state machines systems bind to (SystemDesc.state)."
  field resources List.AnyResource "The game's own resources, created at their defaults."
  field commands Map.CommandHandler "Game actions registered into the command channel, by action name."
  field commandHelp Map.CommandDesc "Each game action's argument form and one sentence, by its name in commands, for `help`: `{ place: { usage: \"<x>:<y>\", doc: \"Move the hero there.\" } }`. An action without an entry is listed by name alone."
  field maxCatchUpTicks u32 "The most ticks one frame may run to catch up with the clock; the accumulator drops the rest." {
    default 4
    min 1
    max 60
    unit ticks
  }
  field facts Option.FactsFn "The game's own facts for the probe's state()."
  field start Option.StartFn "Called once before the first tick, after the first scene spawned. A throw fails the run: no tick runs on the half-built world, `og run`, `og test` and `og dev` exit non-zero naming the error with its stack, and the web shell shows it."
  field ui Option.UiSetupFn "Register retained UI after start and after a hot-reload restore; never reruns start. The engine checks world_hash before and after: duplicate IDs, incompatible store types and gameplay writes fail registration. A fresh-process restore runs start, then ui, then restores named presentation values."
  field render RenderOptions "How the renderer is created: antialiasing and shadow map size. Read once, when the host attaches the renderer."
  field saveVersion u32 "The game's save version, written into every save: raise it when a release changes a saved type, and add the step from the old version to GameDesc.migrations. Left unset, checked project.ts startup saveVersion (else 1); conflicting explicit values refuse the engine." {
    default 1
    min 1
  }
  field migrations List.SaveMigration "The game's explicit save edits: world.restore runs each one whose `from` lies between the save's version and saveVersion, in order. The save must have the current engine version and og-save:4 envelope; other versions are refused, never converted. Engine records and state are never upgraded." {
  }
  field explore ExploreDesc "How `og explore` searches this game's states: what a cell reads and which inputs it presses. Left out, cells come from every field of the game's resources and components and the inputs from its own button actions and none of the intents. Read by the tool only; the engine and the world never see it."
  field look String "A cooked asset ID (og.load.look), applied after the first scene spawned, before start; its antialias applies when render leaves it out. Left unset, checked startup look from og-project.json (whose art contract the cook reads), else none; conflicting explicit values refuse the engine."
}
```

<sub>Module `game` line 76</sub>
