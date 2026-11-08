<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ParticlesDesc"></a>

### `ParticlesDesc` (desc struct)

UI particles from an element (docs/ui.md §3.3): confetti, sparks, coin streams, rarity bursts. Drawn over it on its layer, clipped by its clip, seeded so a replay throws the same. Nothing under reduced motion.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rate` | `number` | `0` | ≥ 0 | — | — | Particles a second while it is built. |
| `burst` | `number` | `0` | — | — | — | Particles at once on a tick fire is set. |
| `fire` | `boolean` | `false` | — | — | — | Throw the burst this tick. |
| `lifetimeMs` | `number` | `0` | ≥ 0 | `ms` | — | How long a particle lives; 0 is 900. |
| `shape` | `ParticleShape` | `"square"` | — | — | — | Its shape. |
| `size` | `Vector2` | `{ x: 8, y: 8 }` | — | `cssPx` | — | Its size. |
| `colors` | `readonly Paint[]` | `[]` | — | — | — | Picked by each particle's seed; empty is the accent. |
| `travelMin` | `number` | `40` | ≥ 0 | `cssPx` | — | The shortest way a particle flies over its life. |
| `travelMax` | `number` | `120` | ≥ 0 | `cssPx` | — | The longest. |
| `direction` | `number` | `-1.5707964` | — | `rad` | — | Its heading, clockwise from +x (-π/2 is up). |
| `spread` | `number` | `6.2831855` | ≥ 0 | `rad` | — | The spread about the heading (2π is every way). |
| `gravity` | `number` | `0` | — | `cssPx` | — | Added downward over its life, this much at its end. |
| `drag` | `boolean` | `true` | — | — | — | Its flight slows to rest. |
| `spin` | `number` | `0` | ≥ 0 | — | — | Turns over its life, up to this many either way. |
| `align` | `boolean` | `false` | — | — | — | Its rotation follows its heading (a streak flying outward). |
| `from` | `EmitFrom` | `"center"` | — | — | — | Where it starts. |
| `endScale` | `number` | `1` | ≥ 0 | — | — | Its scale at the end of its life. |
| `fade` | `boolean` | `true` | — | — | — | Fades out toward its end. |
| `clip` | `boolean` | `true` | — | — | — | Clipped by the element's clip (a scroll view, a masked panel). |

```ogt
struct ParticlesDesc {
  doc "UI particles from an element (docs/ui.md §3.3): confetti, sparks, coin streams, rarity bursts. Drawn over it on its layer, clipped by its clip, seeded so a replay throws the same. Nothing under reduced motion."
  desc true
  field rate f32 "Particles a second while it is built." {
    min 0
  }
  field burst u32 "Particles at once on a tick fire is set."
  field fire bool "Throw the burst this tick."
  field lifetimeMs f32 "How long a particle lives; 0 is 900." {
    unit ms
    min 0
  }
  field shape ParticleShape "Its shape."
  field size Vector2 "Its size." {
    default 8 8
    unit cssPx
  }
  field colors List.Paint "Picked by each particle's seed; empty is the accent."
  field travelMin f32 "The shortest way a particle flies over its life." {
    default 40
    unit cssPx
    min 0
  }
  field travelMax f32 "The longest." {
    default 120
    unit cssPx
    min 0
  }
  field direction f32 "Its heading, clockwise from +x (-π/2 is up)." {
    default -1.5707964
    unit rad
  }
  field spread f32 "The spread about the heading (2π is every way)." {
    default 6.2831855
    unit rad
    min 0
  }
  field gravity f32 "Added downward over its life, this much at its end." {
    unit cssPx
  }
  field drag bool "Its flight slows to rest." {
    default true
  }
  field spin f32 "Turns over its life, up to this many either way." {
    min 0
  }
  field align bool "Its rotation follows its heading (a streak flying outward)."
  field from EmitFrom "Where it starts."
  field endScale f32 "Its scale at the end of its life." {
    default 1
    min 0
  }
  field fade bool "Fades out toward its end." {
    default true
  }
  field clip bool "Clipped by the element's clip (a scroll view, a masked panel)." {
    default true
  }
}
```

<sub>Module `ui` line 5781</sub>
