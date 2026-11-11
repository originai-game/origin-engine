<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-ReflectionProbe"></a>

### `og.ReflectionProbe` (component)

A local reflection: a cube of what is around the entity's position, sampled by every lit surface inside its box with box projection (a room's walls reflect where they are, not at infinity) and blended with the environment map over blendDistance at the box's faces. Overlapping probes: the smallest wins. The cube is rendered from the probe's position without it and without the probes' own reflections, and holds the light as it reaches surfaces.

See `looks §15`.

Token `og.ReflectionProbe`, registered under the schema name `og.ReflectionProbe`; its value type is `ReflectionProbe`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector3` | `{ x: 10, y: 10, z: 10 }` | — | — | — | The box's full extents in world units, centred on the entity (its rotation is ignored: the box is axis-aligned). |
| `resolution` | `number` | `128` | 16 … 1024 | — | — | The side of each cube face in texels; it is rounded to a power of two. |
| `mode` | `ReflectionProbeMode` | `"baked"` | — | — | — | baked (rendered once, or loaded from `og bake probes`) or realtime (every interval ticks). |
| `interval` | `number` | `30` | ≥ 1 | `ticks` | — | With realtime: ticks between updates (1 is every frame). |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | A multiplier on the probe's light. |
| `blendDistance` | `number` | `1` | ≥ 0 | `m` | — | The distance inside the box's faces over which the probe fades into the environment map. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is sampled. |

```ogt
component ReflectionProbe {
  doc "A local reflection: a cube of what is around the entity's position, sampled by every lit surface inside its box with box projection (a room's walls reflect where they are, not at infinity) and blended with the environment map over blendDistance at the box's faces. Overlapping probes: the smallest wins. The cube is rendered from the probe's position without it and without the probes' own reflections, and holds the light as it reaches surfaces."
  guide "looks §15"
  field size Vector3 "The box's full extents in world units, centred on the entity (its rotation is ignored: the box is axis-aligned)." {
    default 10 10 10
  }
  field resolution u32 "The side of each cube face in texels; it is rounded to a power of two." {
    default 128
    min 16
    max 1024
  }
  field mode ReflectionProbeMode "baked (rendered once, or loaded from `og bake probes`) or realtime (every interval ticks)."
  field interval u32 "With realtime: ticks between updates (1 is every frame)." {
    default 30
    min 1
    unit ticks
  }
  field intensity f32 "A multiplier on the probe's light." {
    default 1
    min 0
    tunable true
  }
  field blendDistance f32 "The distance inside the box's faces over which the probe fades into the environment map." {
    default 1
    min 0
    unit m
  }
  field visible bool "Whether it is sampled." {
    default true
  }
}
```

<sub>Module `render` line 2843</sub>
