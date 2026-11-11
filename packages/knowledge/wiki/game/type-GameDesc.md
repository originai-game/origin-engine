<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

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
| `systems` | `readonly (System \| number)[]` | `[]` | — | — | — | The systems, in declaration order before `after`/`before` constraints apply. -1 is skipped: the TypeScript runtime sends it for a token defineSystem did not make. |
| `hooks` | `readonly (Hook \| number)[]` | `[]` | — | — | — | The hooks (og.defineHook), in the order they run. -1 is skipped: the TypeScript runtime sends it for a token og.defineHook did not make. |
| `states` | `readonly StateDesc[]` | `[]` | — | — | — | The state machines systems bind to (SystemDesc.state). |
| `resources` | `readonly AnyResource[]` | `[]` | — | — | — | The game's own resources, created at their defaults. |
| `commands` | `Readonly<Record<string, CommandHandler>>` | `{}` | — | — | — | Game actions registered into the command channel, by action name. |
| `commandHelp` | `Readonly<Record<string, CommandDesc>>` | `{}` | — | — | — | Each game action's argument form and one sentence, by its name in commands, for `help`: `{ place: { usage: "<x>:<y>", doc: "Move the hero there." } }`. An action without an entry is listed by name alone. |
| `maxCatchUp` | `number` | `4` | 1 … 60 | `ticks` | — | The most ticks one frame may run to catch up with the clock; the accumulator drops the rest. |
| `facts` | `FactsFn \| null` | `null` | — | — | — | The game's own facts for the probe's state(). |
| `start` | `StartFn \| null` | `null` | — | — | — | Called once before the first tick, after the first scene spawned. A throw fails the run: no tick runs on the half-built world, `og run`, `og test` and `og dev` exit non-zero naming the error with its stack, and the web shell shows it. |
| `ui` | `UiSetupFn \| null` | `null` | — | — | — | Register retained UI after start and after a hot-reload restore. The engine checks world_hash before and after. Duplicate IDs, incompatible store types and gameplay writes fail registration as errors; ui never reruns start. Fresh-process restore runs start, then ui, then restores named presentation values. |
| `render` | `RenderOptions` | `{}` | — | — | — | How the renderer is created: antialiasing and shadow map size. Read once, when the host attaches the renderer. |
| `saveVersion` | `number` | `1` | ≥ 1 | — | — | The game's save version, written into every save: raise it when a release changes a saved type, and add the step from the old version to GameDesc.migrations. Left unset, project.ogt's `saveVersion` (else 1); the two set to different values refuse the engine. |
| `migrations` | `readonly (SaveMigration \| number)[]` | `[]` | — | — | — | The game's save migrations: world.restore runs each one whose `from` lies between the save's version and saveVersion, in order (rename a field, transform values with the old record in hand, drop or split a type, add an entity). A number is a step's `from` alone, as the TypeScript runtime sends it (the steps run in TypeScript). The save envelope must be og-save:3; earlier envelope formats are refused with both versions in the error, never converted. |
| `explore` | `ExploreDesc` | `{}` | — | — | — | How `og explore` searches this game's states: what a cell reads and which inputs it presses. Left out, cells come from every field of the game's resources and components and the inputs from every button action and none of the intents. Read by the tool only; the engine and the world never see it. |
| `look` | `string` | `""` | — | — | — | A look (`looks/<name>.ogt`, og.load.look) applied after the first scene spawned and before start runs, so start may still change any of it; its antialias is the renderer's when render leaves antialias out. Left out or empty, project.ogt's `look "looks/<name>.ogt"` (the look whose art contract the cook reads) is applied when it has one, else none; the two naming different looks is refused (one value in two places). |

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
  field systems List.OneOf.System.i32 "The systems, in declaration order before `after`/`before` constraints apply. -1 is skipped: the TypeScript runtime sends it for a token defineSystem did not make."
  field hooks List.OneOf.Hook.i32 "The hooks (og.defineHook), in the order they run. -1 is skipped: the TypeScript runtime sends it for a token og.defineHook did not make."
  field states List.StateDesc "The state machines systems bind to (SystemDesc.state)."
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
  field ui Option.UiSetupFn "Register retained UI after start and after a hot-reload restore. The engine checks world_hash before and after. Duplicate IDs, incompatible store types and gameplay writes fail registration as errors; ui never reruns start. Fresh-process restore runs start, then ui, then restores named presentation values."
  field render RenderOptions "How the renderer is created: antialiasing and shadow map size. Read once, when the host attaches the renderer."
  field saveVersion u32 "The game's save version, written into every save: raise it when a release changes a saved type, and add the step from the old version to GameDesc.migrations. Left unset, project.ogt's `saveVersion` (else 1); the two set to different values refuse the engine." {
    default 1
    min 1
  }
  field migrations List.OneOf.SaveMigration.u32 "The game's save migrations: world.restore runs each one whose `from` lies between the save's version and saveVersion, in order (rename a field, transform values with the old record in hand, drop or split a type, add an entity). A number is a step's `from` alone, as the TypeScript runtime sends it (the steps run in TypeScript). The save envelope must be og-save:3; earlier envelope formats are refused with both versions in the error, never converted."
  field explore ExploreDesc "How `og explore` searches this game's states: what a cell reads and which inputs it presses. Left out, cells come from every field of the game's resources and components and the inputs from every button action and none of the intents. Read by the tool only; the engine and the world never see it."
  field look String "A look (`looks/<name>.ogt`, og.load.look) applied after the first scene spawned and before start runs, so start may still change any of it; its antialias is the renderer's when render leaves antialias out. Left out or empty, project.ogt's `look \"looks/<name>.ogt\"` (the look whose art contract the cook reads) is applied when it has one, else none; the two naming different looks is refused (one value in two places)."
}
```

<sub>Module `game` line 21</sub>
