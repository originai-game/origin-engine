<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-WaterSurface"></a>

### `og.WaterSurface` (component)

3D height-field water centred on the entity at its rest level. Waves travel at waveSpeed with CFL substeps; moving hulls raise wakes. Finite-depth volumes merge by the highest enclosing level. og.Water on the same entity draws its heights.

Token `og.WaterSurface`, registered under the schema name `og.WaterSurface`; its value type is `WaterSurface`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `resolution` | `number` | `64` | 8 … 256 | — | — | Grid points along each side. |
| `size` | `number` | `16` | ≥ 0.1 | `m` | tunable | The side of the square of water. |
| `depth` | `number` | `0` | ≥ 0 | `m` | tunable | Vertical depth below the surface; zero disables the water volume. |
| `waveSpeed` | `number` | `3` | ≥ 0 | `mps` | tunable | How fast waves travel. |
| `damping` | `number` | `0.9` | 0 … 100 | `perSecond` | tunable | Exponential wave velocity damping per second; zero preserves motion. |
| `spongeWidth` | `number` | `0` | ≥ 0 | `m` | tunable | Width of the absorbing border; zero disables it. |
| `spongeDamping` | `number` | `0` | ≥ 0 | `perSecond` | tunable | Extra exponential damping at the outer edge; zero disables it. |
| `splash` | `number` | `1` | ≥ 0 | — | tunable | How strongly bodies push waves. |
| `buoyancy` | `number` | `9.81` | ≥ 0 | — | tunable | Upward force per displaced cubic metre. Planar convex hulls and oriented boxes use clipped volume; other shapes approximate the immersed fraction. Zero disables lift. |
| `drag` | `number` | `1.5` | ≥ 0 | `perSecond` | tunable | How much a submerged body's velocity is slowed. |

```ogt
component WaterSurface {
  doc "3D height-field water centred on the entity at its rest level. Waves travel at waveSpeed with CFL substeps; moving hulls raise wakes. Finite-depth volumes merge by the highest enclosing level. og.Water on the same entity draws its heights."
  field resolution u32 "Grid points along each side." {
    default 64
    min 8
    max 256
  }
  field size f32 "The side of the square of water." {
    default 16
    min 0.1
    unit m
    tunable true
  }
  field depth f32 "Vertical depth below the surface; zero disables the water volume." {
    min 0
    unit m
    tunable true
  }
  field waveSpeed f32 "How fast waves travel." {
    default 3
    min 0
    unit mps
    tunable true
  }
  field damping f32 "Exponential wave velocity damping per second; zero preserves motion." {
    default 0.9
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field spongeWidth f32 "Width of the absorbing border; zero disables it." {
    min 0
    unit m
    tunable true
  }
  field spongeDamping f32 "Extra exponential damping at the outer edge; zero disables it." {
    min 0
    unit perSecond
    tunable true
  }
  field splash f32 "How strongly bodies push waves." {
    default 1
    min 0
    tunable true
  }
  field buoyancy f32 "Upward force per displaced cubic metre. Planar convex hulls and oriented boxes use clipped volume; other shapes approximate the immersed fraction. Zero disables lift." {
    default 9.81
    min 0
    tunable true
  }
  field drag f32 "How much a submerged body's velocity is slowed." {
    default 1.5
    min 0
    unit perSecond
    tunable true
  }
}
```

<sub>Module `physics` line 1905</sub>
