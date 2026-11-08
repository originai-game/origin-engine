<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-EffectLayerDesc"></a>

### `EffectLayerDesc` (desc struct)

A subtree drawn offscreen and composited through a material, with the material's one number that matters as `amount`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `material` | `MaterialName` | `"plain"` | — | — | — | The material. |
| `amount` | `number` | `0` | 0 … 1 | — | — | How far in it is, 0 to 1: dissolve's threshold, shine's sweep, hologram's flicker, glitch's strength, pixelate's cell, blur's radius, colorAdjust's grey. |
| `color` | `ThemeColor \| null` | `null` | — | — | — | The material's colour (a dissolve's edge, a shine, a hologram's tint, an outline); null is the style's highlight. |
| `sweep` | `boolean` | `false` | — | — | — | Run amount from 0 to 1 and back by the clock (a shine passing, a hologram flickering); still under reduced motion. |
| `periodMs` | `number` | `2400` | — | `ms` | — | One sweep's length. |
| `custom` | `UiMaterial \| null` | `null` | — | — | — | A custom material (material custom). |
| `params` | `readonly number[]` | `[]` | — | — | — | A custom material's parameters. |

```ogt
struct EffectLayerDesc {
  doc "A subtree drawn offscreen and composited through a material, with the material's one number that matters as `amount`."
  desc true
  field material MaterialName "The material."
  field amount f32 "How far in it is, 0 to 1: dissolve's threshold, shine's sweep, hologram's flicker, glitch's strength, pixelate's cell, blur's radius, colorAdjust's grey." {
    min 0
    max 1
  }
  field color Option.ThemeColor "The material's colour (a dissolve's edge, a shine, a hologram's tint, an outline); null is the style's highlight."
  field sweep bool "Run amount from 0 to 1 and back by the clock (a shine passing, a hologram flickering); still under reduced motion."
  field periodMs f32 "One sweep's length." {
    default 2400
    unit ms
  }
  field custom Option.UiMaterial "A custom material (material custom)."
  field params List.f32 "A custom material's parameters."
}
```

<sub>Module `ui` line 3407</sub>
