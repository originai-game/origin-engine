<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-ProjectStartup"></a>

### `ProjectStartup` (struct)

Checked project.ts startup data for CreateOptions.project on both hosts. Not project source text or the full manifest. Omitted simulation fields retain GameDesc defaults; conflicting explicit values refuse creation.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | 1 … 240 | — | optional | Fixed ticks per second, 1 to 240. |
| `seed` | `number` | `0` | — | — | optional | The project's deterministic seed; CreateOptions.seed is a separate replay override. |
| `scenes` | `Readonly<Record<string, string>>` | `{}` | — | — | optional | Startup scene names to physical cooked IDs, resolved by og project from checked project refs; no scene contents or pack assembly. |
| `scene` | `string` | `""` | — | — | optional | Initial scene name. |
| `look` | `string` | `""` | — | — | optional | The look's physical cooked ID, resolved by og project from its checked project ref. |
| `saveVersion` | `number` | `0` | ≥ 1 | — | optional | The game's save version, at least 1. |
| `loading` | `LoadingScreen` | `"engine"` | — | — | optional | Who draws the loader; omitted means engine. |
| `flow` | `ProjectStartupFlow` | `{ dialogue: "" }` | — | — | optional | Dialogue to start before GameDesc.start. |
| `profile` | `ProjectProfile` | `"3d"` | — | — | optional | Normalized profile; omitted means 3d. Tools always emit this field. |

```ogt
struct ProjectStartup {
  doc "Checked project.ts startup data for CreateOptions.project on both hosts. Not project source text or the full manifest. Omitted simulation fields retain GameDesc defaults; conflicting explicit values refuse creation."
  field tick u32 "Fixed ticks per second, 1 to 240." {
    optional true
    min 1
    max 240
  }
  field seed u32 "The project's deterministic seed; CreateOptions.seed is a separate replay override." {
    optional true
  }
  field scenes Map.String "Startup scene names to physical cooked IDs, resolved by og project from checked project refs; no scene contents or pack assembly." {
    optional true
  }
  field scene String "Initial scene name." {
    optional true
  }
  field look String "The look's physical cooked ID, resolved by og project from its checked project ref." {
    optional true
  }
  field saveVersion u32 "The game's save version, at least 1." {
    optional true
    min 1
  }
  field loading LoadingScreen "Who draws the loader; omitted means engine." {
    optional true
  }
  field flow ProjectStartupFlow "Dialogue to start before GameDesc.start." {
    optional true
  }
  field profile ProjectProfile "Normalized profile; omitted means 3d. Tools always emit this field." {
    optional true
  }
}
```

<sub>Module `game` line 26</sub>
