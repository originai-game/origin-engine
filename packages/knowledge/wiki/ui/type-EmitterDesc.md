<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-EmitterDesc"></a>

### `EmitterDesc` (desc struct)

An ambient particle emitter over the area of its parent.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `preset` | `EmitterPreset` | `"sparkle"` | — | — | — | What it throws. |
| `rate` | `number` | `0` | ≥ 0 | — | — | Particles a second; 0 is the preset's. |
| `colors` | `readonly ThemeColor[]` | `[]` | — | — | — | Its colours; empty is the preset's (the style's primary, highlight and gold). |
| `active` | `boolean` | `true` | — | — | — | It throws (particles already thrown live out their lives when it stops). |
| `size` | `number` | `1` | ≥ 0 | — | — | Its particles' size, times the preset's. |

```ogt
struct EmitterDesc {
  doc "An ambient particle emitter over the area of its parent."
  desc true
  field preset EmitterPreset "What it throws."
  field rate f32 "Particles a second; 0 is the preset's." {
    min 0
  }
  field colors List.ThemeColor "Its colours; empty is the preset's (the style's primary, highlight and gold)."
  field active bool "It throws (particles already thrown live out their lives when it stops)." {
    default true
  }
  field size f32 "Its particles' size, times the preset's." {
    default 1
    min 0
  }
}
```

<sub>`api.ogt` line 19148</sub>
