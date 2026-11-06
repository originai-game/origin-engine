<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-Description"></a>

### `Description` (struct)

The frame as data: deterministic, the same JSON on every host and backend for the same world.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick. |
| `surface` | `UiSurface \| null` | `null` | — | — | — | The play surface. |
| `pass` | `PassSize \| null` | `null` | — | — | — | The entity-id pass; null without a scene. |
| `camera` | `DescribedCamera \| null` | `null` | — | — | — | The camera; null without a scene. |
| `entities` | `readonly DescribedEntity[]` | `[]` | — | — | — | The entities the main camera shows, nearest first (each names the views that also show it). |
| `views` | `readonly DescribedView[]` | `[]` | — | — | — | The secondary cameras the frame draws, in its view order, each with what it shows. |
| `ui` | `readonly DescribedUi[]` | `[]` | — | — | — | The UI elements in draw order. |
| `prefabs` | `readonly DescribedPrefab[]` | `[]` | — | — | optional | The prefab instances in the world, by prefab id; absent in a world without one. |

```ogt
struct Description {
  doc "The frame as data: deterministic, the same JSON on every host and backend for the same world."
  field tick u32 "The tick."
  field surface Option.UiSurface "The play surface."
  field pass Option.PassSize "The entity-id pass; null without a scene."
  field camera Option.DescribedCamera "The camera; null without a scene."
  field entities List.DescribedEntity "The entities the main camera shows, nearest first (each names the views that also show it)."
  field views List.DescribedView "The secondary cameras the frame draws, in its view order, each with what it shows."
  field ui List.DescribedUi "The UI elements in draw order."
  field prefabs List.DescribedPrefab "The prefab instances in the world, by prefab id; absent in a world without one." {
    optional true
  }
}
```

<sub>`api.ogt` line 24434</sub>
