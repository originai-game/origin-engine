<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Wind"></a>

### `og.Wind` (resource)

The wind the renderer sways vegetation with: og.TerrainScatter copies and their impostor cards, og.Grass blades, swaying trees, and og.Model instances with a sway, lean downwind and sway in gusts that sweep across the field, each vertex moved by its height (the root stays put, the tip moves furthest). Where the renderer can (RenderCapabilities.compute) it simulates the wind as one field around the camera that every one of them reads, so a gust front crosses grass, copies and trees together, eddies (`turbulence`), carries og.WindMotor pushes, and vegetation springs into a gust and settles after it; og.Model sway keeps the wind's own gusts. Off at strength 0. Visual only: drawn from the tick, so it sways the same on every run and after a restore, and nothing in the simulation reads it (og.Rope, og.Cloth and og.Weather keep their own wind).

Token `og.Wind`, registered under the schema name `og.Wind`; its value type is `Wind`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `strength` | `number` | `0` | 0 … 2 | — | tunable | The mean lean: how far a copy's tip moves downwind per metre of its height (0.2: a 1 m stem's tip leans 20 cm); 0 is still. |
| `direction` | `Vector3` | `{ x: 1, y: 0, z: 0 }` | — | — | tunable | Where the wind blows: its horizontal part, normalised. |
| `speed` | `number` | `3` | 0 … 50 | `mps` | tunable | How fast the gusts travel downwind, over a 10 m wave. |
| `gust` | `number` | `0.6` | 0 … 1 | `ratio` | tunable | How much the lean varies in the gusts: 0 a steady lean, 1 from upright to twice the mean. |
| `turbulence` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | How much the wind swirls on its own between the gusts, where the renderer simulates it (RenderCapabilities.compute): 0 the gusts alone, 1 eddies as strong as the wind. |
| `gustSize` | `number` | `14` | 1 … 200 | `m` | tunable | How large the gust patches are. |

```ogt
resource Wind {
  doc "The wind the renderer sways vegetation with: og.TerrainScatter copies and their impostor cards, og.Grass blades, swaying trees, and og.Model instances with a sway, lean downwind and sway in gusts that sweep across the field, each vertex moved by its height (the root stays put, the tip moves furthest). Where the renderer can (RenderCapabilities.compute) it simulates the wind as one field around the camera that every one of them reads, so a gust front crosses grass, copies and trees together, eddies (`turbulence`), carries og.WindMotor pushes, and vegetation springs into a gust and settles after it; og.Model sway keeps the wind's own gusts. Off at strength 0. Visual only: drawn from the tick, so it sways the same on every run and after a restore, and nothing in the simulation reads it (og.Rope, og.Cloth and og.Weather keep their own wind)."
  field strength f32 "The mean lean: how far a copy's tip moves downwind per metre of its height (0.2: a 1 m stem's tip leans 20 cm); 0 is still." {
    min 0
    max 2
    tunable true
  }
  field direction Vector3 "Where the wind blows: its horizontal part, normalised." {
    default 1 0 0
    tunable true
  }
  field speed f32 "How fast the gusts travel downwind, over a 10 m wave." {
    default 3
    min 0
    max 50
    unit mps
    tunable true
  }
  field gust f32 "How much the lean varies in the gusts: 0 a steady lean, 1 from upright to twice the mean." {
    default 0.6
    min 0
    max 1
    unit ratio
    tunable true
  }
  field turbulence f32 "How much the wind swirls on its own between the gusts, where the renderer simulates it (RenderCapabilities.compute): 0 the gusts alone, 1 eddies as strong as the wind." {
    default 0.3
    min 0
    max 1
    unit ratio
    tunable true
  }
  field gustSize f32 "How large the gust patches are." {
    default 14
    min 1
    max 200
    unit m
    tunable true
  }
}
```

<sub>`api.ogt` line 11144</sub>
