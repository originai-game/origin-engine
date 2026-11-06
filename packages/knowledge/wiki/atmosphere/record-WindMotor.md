<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`atmosphere`](../atmosphere.md) · [index](../index.md)

<a id="record-WindMotor"></a>

### `og.WindMotor` (component)

A local push on the wind the renderer sways vegetation with, at the entity's position: grass blades, og.TerrainScatter copies and their impostor cards, and swaying trees within `radius` lean away from it (omni), along it (directional) or around it (vortex), the push fading to nothing at its edge; the field carries and swirls the push, so it ripples out and settles after the motor stops. Needs no og.Wind (a motor in still air moves only what it reaches). Visual only, where the renderer simulates the wind (RenderCapabilities.compute; elsewhere it does nothing): nothing in the simulation reads it, and the same ticks draw the same push on every run. The 16 nearest the camera act.

Token `og.WindMotor`, registered under the schema name `og.WindMotor`; its value type is `WindMotor`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `WindMotorKind` | `"omni"` | — | — | — | How it pushes. |
| `radius` | `number` | `6` | 0 … 200 | `m` | tunable | How far it reaches. |
| `strength` | `number` | `10` | 0 … 100 | `mps` | tunable | Its push at its centre, as a wind speed; vegetation leans by about 0.05 per m/s of it in still air, by og.Wind's lean per m/s of its speed in a wind. |

```ogt
component WindMotor {
  doc "A local push on the wind the renderer sways vegetation with, at the entity's position: grass blades, og.TerrainScatter copies and their impostor cards, and swaying trees within `radius` lean away from it (omni), along it (directional) or around it (vortex), the push fading to nothing at its edge; the field carries and swirls the push, so it ripples out and settles after the motor stops. Needs no og.Wind (a motor in still air moves only what it reaches). Visual only, where the renderer simulates the wind (RenderCapabilities.compute; elsewhere it does nothing): nothing in the simulation reads it, and the same ticks draw the same push on every run. The 16 nearest the camera act."
  field kind WindMotorKind "How it pushes."
  field radius f32 "How far it reaches." {
    default 6
    min 0
    max 200
    unit m
    tunable true
  }
  field strength f32 "Its push at its centre, as a wind speed; vegetation leans by about 0.05 per m/s of it in still air, by og.Wind's lean per m/s of its speed in a wind." {
    default 10
    min 0
    max 100
    unit mps
    tunable true
  }
}
```

<sub>`api.ogt` line 11191</sub>
