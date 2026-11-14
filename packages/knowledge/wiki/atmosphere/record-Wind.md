<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-Wind"></a>

### `og.Wind` (resource)

The world's wind: where the air moves and how fast. Everything in the air moves in it and adds its own `wind`: vegetation leans downwind and sways in gusts (og.TerrainScatter, og.Grass, trees, og.Model with a sway: 0.05 of its height per m/s), og.Clouds drift, og.Weather slants, and the tick pushes og.Rope, og.Cloth, og.Garment and bodies with RigidBody.airDrag. Material shader hooks read it as `h.wind` (xyz where it blows, w the lean). Where RenderCapabilities.compute allows, vegetation reads one simulated field that carries og.WindMotor pushes. An og.Water's sea keeps its own wind.

See `render §8`.

Token `og.Wind`, registered under the schema name `og.Wind`; its value type is `Wind`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `direction` | `Vector3` | `{ x: 1, y: 0, z: 0 }` | — | — | tunable | Where the wind blows: its horizontal part, normalised. |
| `speed` | `number` | `0` | 0 … 50 | `mps` | tunable | How fast the air moves, and the gusts travel downwind (3: a light breeze, 8: fresh, branches move; 20: a gale); 0 is still. |
| `gust` | `number` | `0.6` | 0 … 1 | `ratio` | tunable | How much the lean varies in the gusts: 0 a steady lean, 1 from upright to twice the mean. |
| `turbulence` | `number` | `0.3` | 0 … 1 | `ratio` | tunable | How much the wind swirls on its own between the gusts, where the renderer simulates it (RenderCapabilities.compute): 0 the gusts alone, 1 eddies as strong as the wind. |
| `gustSize` | `number` | `14` | 1 … 200 | `m` | tunable | How large the gust patches are. |

```ogt
resource Wind {
  doc "The world's wind: where the air moves and how fast. Everything in the air moves in it and adds its own `wind`: vegetation leans downwind and sways in gusts (og.TerrainScatter, og.Grass, trees, og.Model with a sway: 0.05 of its height per m/s), og.Clouds drift, og.Weather slants, and the tick pushes og.Rope, og.Cloth, og.Garment and bodies with RigidBody.airDrag. Material shader hooks read it as `h.wind` (xyz where it blows, w the lean). Where RenderCapabilities.compute allows, vegetation reads one simulated field that carries og.WindMotor pushes. An og.Water's sea keeps its own wind."
  guide "render §8"
  field direction Vector3 "Where the wind blows: its horizontal part, normalised." {
    default 1 0 0
    tunable true
  }
  field speed f32 "How fast the air moves, and the gusts travel downwind (3: a light breeze, 8: fresh, branches move; 20: a gale); 0 is still." {
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

<sub>Module `atmosphere` line 591</sub>
