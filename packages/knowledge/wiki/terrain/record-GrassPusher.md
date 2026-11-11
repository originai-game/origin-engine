<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="record-GrassPusher"></a>

### `og.GrassPusher` (component)

Presses og.Grass blades and og.TerrainScatter copies where the entity stands: a vertical capsule from its position up `height`, reaching `radius` on the ground. Blades under it bend away from it and flatten, and stay pressed after it passes, standing again over the field's `recovery` (a trail behind a walker or a horse); swaying copies part the same way. Blades shorter than the gap under a pusher floating above them are not touched. Visual only, where the renderer can (RenderCapabilities.compute; elsewhere it does nothing): nothing in the simulation reads it. The 32 nearest the camera press.

Token `og.GrassPusher`, registered under the schema name `og.GrassPusher`; its value type is `GrassPusher`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `0.5` | 0 … 20 | `m` | tunable | How far it reaches on the ground. |
| `height` | `number` | `1.8` | 0 … 50 | `m` | tunable | How tall it is: its bottom is the entity's position. |
| `strength` | `number` | `1` | 0 … 1 | `ratio` | tunable | How hard it presses: 1 flattens the blades, 0.3 parts them. |

```ogt
component GrassPusher {
  doc "Presses og.Grass blades and og.TerrainScatter copies where the entity stands: a vertical capsule from its position up `height`, reaching `radius` on the ground. Blades under it bend away from it and flatten, and stay pressed after it passes, standing again over the field's `recovery` (a trail behind a walker or a horse); swaying copies part the same way. Blades shorter than the gap under a pusher floating above them are not touched. Visual only, where the renderer can (RenderCapabilities.compute; elsewhere it does nothing): nothing in the simulation reads it. The 32 nearest the camera press."
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

<sub>Module `terrain` line 389</sub>
