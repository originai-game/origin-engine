<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-WheelPose"></a>

### `WheelPose` (struct)

A wheel of an og.Vehicle as the last step left it, for drawing.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The wheel's centre in world space. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Its rotation (steering and spin) in world space. |
| `contact` | `boolean` | `false` | — | — | — | Whether it touches the ground. |
| `slip` | `number` | `0` | 0 … 1 | — | — | How much the tyre slides (rapier's skid info turned round): 0 it grips, towards 1 it asks for more than its grip holds (0.5 is twice as much); 0 in the air and in 2D. See `physics §7`. |
| `sideSpeed` | `number` | `0` | — | `mps` | — | How fast the tyre's contact moves sideways over the ground (along its axle; 0 in the air and in 2D): a slide's speed, for the squeal's pitch or a skid mark's darkness. |
| `entity` | `Entity` | `0` | — | — | — | The wheel's og.VehicleWheel entity; 0 for a vehicle's implicit wheels. |

```ogt
struct WheelPose {
  doc "A wheel of an og.Vehicle as the last step left it, for drawing."
  field position Vector3 "The wheel's centre in world space." {
    unit m
  }
  field rotation Quaternion "Its rotation (steering and spin) in world space."
  field contact bool "Whether it touches the ground."
  field slip f32 "How much the tyre slides (rapier's skid info turned round): 0 it grips, towards 1 it asks for more than its grip holds (0.5 is twice as much); 0 in the air and in 2D." {
    guide "physics §7"
    min 0
    max 1
  }
  field sideSpeed f32 "How fast the tyre's contact moves sideways over the ground (along its axle; 0 in the air and in 2D): a slide's speed, for the squeal's pitch or a skid mark's darkness." {
    unit mps
  }
  field entity Entity "The wheel's og.VehicleWheel entity; 0 for a vehicle's implicit wheels."
}
```

<sub>Module `physics` line 2021</sub>
