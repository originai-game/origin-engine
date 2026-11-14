<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-ForceField"></a>

### `og.ForceField` (component)

An acceleration on what is inside the og.Volume on its entity (none: everywhere), whatever its mass, as gravity is. Dynamic bodies, ropes, cloths, garments, soft bodies and fluids take it; a character controller takes its vertical part. The same on every run.

See `physics §8`.

Also known as: gravity zone, low gravity room, gravity well, pull objects towards a point, attract or repel bodies, updraft, jump pad, tractor beam, anti-gravity area, whirlpool

Token `og.ForceField`, registered under the schema name `og.ForceField`; its value type is `ForceField`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ForceFieldKind` | `"directional"` | — | — | — | How it accelerates. |
| `direction` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | A directional field's direction and a vortex's axis, in the entity's space (it turns with the entity); its length does not matter. |
| `strength` | `number` | `10` | — | `mps2` | tunable | Its acceleration inside the volume, fading with the volume's `fade` past it. Negative reverses it: a radial field pulls in. |
| `enabled` | `boolean` | `true` | — | — | — | Whether it acts. |

```ogt
component ForceField {
  aliases "gravity zone, low gravity room, gravity well, pull objects towards a point, attract or repel bodies, updraft, jump pad, tractor beam, anti-gravity area, whirlpool"
  doc "An acceleration on what is inside the og.Volume on its entity (none: everywhere), whatever its mass, as gravity is. Dynamic bodies, ropes, cloths, garments, soft bodies and fluids take it; a character controller takes its vertical part. The same on every run."
  guide "physics §8"
  field kind ForceFieldKind "How it accelerates."
  field direction Vector3 "A directional field's direction and a vortex's axis, in the entity's space (it turns with the entity); its length does not matter." {
    default 0 1 0
  }
  field strength f32 "Its acceleration inside the volume, fading with the volume's `fade` past it. Negative reverses it: a radial field pulls in." {
    default 10
    unit mps2
    tunable true
  }
  field enabled bool "Whether it acts." {
    default true
  }
}
```

<sub>Module `physics` line 169</sub>
