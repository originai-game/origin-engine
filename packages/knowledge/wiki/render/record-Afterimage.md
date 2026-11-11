<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Afterimage"></a>

### `og.Afterimage` (component)

Ghosts of the entity's og.Sprite: every `interval` ticks its pose (position, rotation, scale, frame, flips) is kept, and the last `count` are drawn behind it, fading from `opacity` to 0, tinted by color. A dash, a speed-up, a teleport. The history is world state (the og.sprite state block).

Token `og.Afterimage`, registered under the schema name `og.Afterimage`; its value type is `Afterimage`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `4` | 0 … 16 | — | — | How many ghosts. |
| `interval` | `number` | `3` | ≥ 1 | `ticks` | — | Ticks between ghosts. |
| `opacity` | `number` | `0.5` | 0 … 1 | — | tunable | The newest ghost's alpha. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | A linear tint (above 1 with additive, a glow). |
| `additive` | `boolean` | `false` | — | — | — | Add the ghosts instead of blending them. |
| `active` | `boolean` | `true` | — | — | — | Record new ghosts; false lets the existing ones fade out. |

```ogt
component Afterimage {
  doc "Ghosts of the entity's og.Sprite: every `interval` ticks its pose (position, rotation, scale, frame, flips) is kept, and the last `count` are drawn behind it, fading from `opacity` to 0, tinted by color. A dash, a speed-up, a teleport. The history is world state (the og.sprite state block)."
  field count u32 "How many ghosts." {
    default 4
    min 0
    max 16
  }
  field interval u32 "Ticks between ghosts." {
    default 3
    min 1
    unit ticks
  }
  field opacity f32 "The newest ghost's alpha." {
    default 0.5
    min 0
    max 1
    tunable true
  }
  field color Color "A linear tint (above 1 with additive, a glow)." {
    default 1 1 1
  }
  field additive bool "Add the ghosts instead of blending them."
  field active bool "Record new ghosts; false lets the existing ones fade out." {
    default true
  }
}
```

<sub>Module `render` line 2572</sub>
