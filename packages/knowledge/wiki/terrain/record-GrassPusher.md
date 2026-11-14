<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="record-GrassPusher"></a>

### `og.GrassPusher` (component)

Presses what covers the ground where the entity stands: a vertical capsule from its position up `height`, reaching `radius` on the ground. og.Grass blades bend away and flatten, standing again over the field's `recovery` after it passes (a trail behind a walker); og.TerrainScatter copies part the same way; snow and sand lying on a terrain (og.Weather) keep a track that fills in over four minutes. What a pusher floats above is not touched. Visual only, where the renderer can (RenderCapabilities.compute; elsewhere nothing): the simulation reads none of it. The 32 nearest the camera press.

Token `og.GrassPusher`, registered under the schema name `og.GrassPusher`; its value type is `GrassPusher`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `0.5` | 0 … 20 | `m` | tunable | How far it reaches on the ground. |
| `height` | `number` | `1.8` | 0 … 50 | `m` | tunable | How tall it is: its bottom is the entity's position. |
| `strength` | `number` | `1` | 0 … 1 | `ratio` | tunable | How hard it presses: 1 flattens the blades, 0.3 parts them. |

```ogt
component GrassPusher {
  doc "Presses what covers the ground where the entity stands: a vertical capsule from its position up `height`, reaching `radius` on the ground. og.Grass blades bend away and flatten, standing again over the field's `recovery` after it passes (a trail behind a walker); og.TerrainScatter copies part the same way; snow and sand lying on a terrain (og.Weather) keep a track that fills in over four minutes. What a pusher floats above is not touched. Visual only, where the renderer can (RenderCapabilities.compute; elsewhere nothing): the simulation reads none of it. The 32 nearest the camera press."
  field radius f32 "How far it reaches on the ground." {
    default 0.5
    min 0
    max 20
    unit m
    tunable true
  }
  field height f32 "How tall it is: its bottom is the entity's position." {
    default 1.8
    min 0
    max 50
    unit m
    tunable true
  }
  field strength f32 "How hard it presses: 1 flattens the blades, 0.3 parts them." {
    default 1
    min 0
    max 1
    unit ratio
    tunable true
  }
}
```

<sub>Module `terrain` line 393</sub>
