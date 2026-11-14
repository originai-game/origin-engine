<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-WindMotor"></a>

### `og.WindMotor` (component)

A local push on the air at the entity's position, away from it (omni), along it (directional) or around it (vortex), fading to nothing at `radius`. Needs no og.Wind. In the tick it blows every og.Rope, og.Cloth and og.Garment hanging in its reach and carries a body with RigidBody.airDrag, the same on every run. Vegetation leans in it where the renderer simulates the wind (RenderCapabilities.compute; the 16 nearest the camera act), and the field carries the push on after the motor stops.

See `render §8`.

Token `og.WindMotor`, registered under the schema name `og.WindMotor`; its value type is `WindMotor`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `WindMotorKind` | `"omni"` | — | — | — | How it pushes. |
| `radius` | `number` | `6` | 0 … 200 | `m` | tunable | How far it reaches. |
| `strength` | `number` | `10` | 0 … 100 | `mps` | tunable | Its push at its centre, as a wind speed: vegetation leans by 0.05 of its height per m/s of it, as in og.Wind. |

```ogt
component WindMotor {
  doc "A local push on the air at the entity's position, away from it (omni), along it (directional) or around it (vortex), fading to nothing at `radius`. Needs no og.Wind. In the tick it blows every og.Rope, og.Cloth and og.Garment hanging in its reach and carries a body with RigidBody.airDrag, the same on every run. Vegetation leans in it where the renderer simulates the wind (RenderCapabilities.compute; the 16 nearest the camera act), and the field carries the push on after the motor stops."
  guide "render §8"
  field kind WindMotorKind "How it pushes."
  field radius f32 "How far it reaches." {
    default 6
    min 0
    max 200
    unit m
    tunable true
  }
  field strength f32 "Its push at its centre, as a wind speed: vegetation leans by 0.05 of its height per m/s of it, as in og.Wind." {
    default 10
    min 0
    max 100
    unit mps
    tunable true
  }
}
```

<sub>Module `atmosphere` line 648</sub>
