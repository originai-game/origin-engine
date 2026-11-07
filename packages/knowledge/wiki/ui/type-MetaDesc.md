<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MetaDesc"></a>

### `MetaDesc` (desc struct)

A meta layer (docs/ui.md §1): full screen, over the scene and under the screen's elements, never taking input.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `MetaKind` | `"vignette"` | — | — | — | What it draws. |
| `color` | `Paint \| null` | `null` | — | — | — | Vignette and tint: its ink; null is a deep red. |
| `inner` | `number` | `0.55` | 0 … 1 | `ratio` | — | Vignette: clear out to this fraction of the half diagonal. |
| `outer` | `number` | `1` | 0 … 1 | `ratio` | — | Vignette: full colour from this fraction on. |
| `material` | `MaterialName` | `"plain"` | — | — | — | Material: the built-in material, or custom. |
| `custom` | `UiMaterial \| null` | `null` | — | — | — | Material: the custom material. |
| `params` | `readonly number[]` | `[]` | — | — | — | Material: its parameters, up to four vec4s flattened. |
| `intensity` | `number` | `1` | 0 … 1 | — | — | Its opacity. |
| `pulseHz` | `number` | `0` | ≥ 0 | `hz` | — | Pulses this many times a second (a low-health heartbeat); 0 is steady. |
| `pulseDepth` | `number` | `0.5` | 0 … 1 | `ratio` | — | How far a pulse dims it. |

```ogt
struct MetaDesc {
  doc "A meta layer (docs/ui.md §1): full screen, over the scene and under the screen's elements, never taking input."
  desc true
  field kind MetaKind "What it draws."
  field color Option.Paint "Vignette and tint: its ink; null is a deep red."
  field inner f32 "Vignette: clear out to this fraction of the half diagonal." {
    default 0.55
    unit ratio
    min 0
    max 1
  }
  field outer f32 "Vignette: full colour from this fraction on." {
    default 1
    unit ratio
    min 0
    max 1
  }
  field material MaterialName "Material: the built-in material, or custom."
  field custom Option.UiMaterial "Material: the custom material."
  field params List.f32 "Material: its parameters, up to four vec4s flattened."
  field intensity f32 "Its opacity." {
    default 1
    min 0
    max 1
  }
  field pulseHz f32 "Pulses this many times a second (a low-health heartbeat); 0 is steady." {
    unit hz
    min 0
  }
  field pulseDepth f32 "How far a pulse dims it." {
    default 0.5
    unit ratio
    min 0
    max 1
  }
}
```

<sub>Module `ui` line 7813</sub>
