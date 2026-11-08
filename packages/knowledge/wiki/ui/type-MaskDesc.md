<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MaskDesc"></a>

### `MaskDesc` (desc struct)

An alpha mask over an element's subtree: a gradient's alpha, else an image's.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `gradient` | `Gradient \| null` | `null` | — | — | — | The gradient whose alpha masks (a list fading at its edge). |
| `texture` | `Texture \| null` | `null` | — | — | — | The image whose alpha masks, stretched over the element. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The image's region; null is the whole texture. |

```ogt
struct MaskDesc {
  doc "An alpha mask over an element's subtree: a gradient's alpha, else an image's."
  desc true
  field gradient Option.Gradient "The gradient whose alpha masks (a list fading at its edge)."
  field texture Option.Texture "The image whose alpha masks, stretched over the element."
  field uv Option.Array.f32.4 "The image's region; null is the whole texture."
}
```

<sub>Module `ui` line 2862</sub>
