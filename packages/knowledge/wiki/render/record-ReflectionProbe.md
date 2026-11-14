<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-ReflectionProbe"></a>

### `og.ReflectionProbe` (component)

A local reflection: a cube of what is around the entity's position, sampled by every lit surface inside its box with box projection (a room's walls reflect where they are, not at infinity), fading into the environment map over og.Volume.fade past the box's faces. The box is the og.Volume on its entity, as the world's axes bound it; none: everywhere, as at infinity. Overlapping probes: the smallest wins. The cube is rendered from the probe's position without it and without the probes' own reflections, and holds the light as it reaches surfaces.

See `looks §15`.

Token `og.ReflectionProbe`, registered under the schema name `og.ReflectionProbe`; its value type is `ReflectionProbe`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `resolution` | `number` | `128` | 16 … 1024 | — | — | The side of each cube face in texels; it is rounded to a power of two. |
| `mode` | `ReflectionProbeMode` | `"baked"` | — | — | — | baked (rendered once, or loaded from `og bake probes`) or realtime (every intervalTicks ticks). |
| `intervalTicks` | `number` | `30` | ≥ 1 | `ticks` | — | With realtime: ticks between updates (1 is every frame). |
| `intensity` | `number` | `1` | ≥ 0 | — | tunable | A multiplier on the probe's light. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is sampled. |

```ogt
component ReflectionProbe {
  doc "A local reflection: a cube of what is around the entity's position, sampled by every lit surface inside its box with box projection (a room's walls reflect where they are, not at infinity), fading into the environment map over og.Volume.fade past the box's faces. The box is the og.Volume on its entity, as the world's axes bound it; none: everywhere, as at infinity. Overlapping probes: the smallest wins. The cube is rendered from the probe's position without it and without the probes' own reflections, and holds the light as it reaches surfaces."
  guide "looks §15"
  field resolution u32 "The side of each cube face in texels; it is rounded to a power of two." {
    default 128
    min 16
    max 1024
  }
  field mode ReflectionProbeMode "baked (rendered once, or loaded from `og bake probes`) or realtime (every intervalTicks ticks)."
  field intervalTicks u32 "With realtime: ticks between updates (1 is every frame)." {
    default 30
    min 1
    unit ticks
  }
  field intensity f32 "A multiplier on the probe's light." {
    default 1
    min 0
    tunable true
  }
  field visible bool "Whether it is sampled." {
    default true
  }
}
```

<sub>Module `render` line 2908</sub>
