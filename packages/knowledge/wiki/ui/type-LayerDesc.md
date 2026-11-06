<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-LayerDesc"></a>

### `LayerDesc` (desc struct)

The element's subtree drawn offscreen, then composited through a material (a card that dissolves, a hologram panel).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `material` | `MaterialName` | `"plain"` | — | — | — | The material. |
| `custom` | `UiMaterial \| null` | `null` | — | — | — | The custom material, for material custom. |
| `params` | `readonly number[]` | `[]` | — | — | — | Its parameters: up to four vec4s, flattened (16 numbers); for a custom material, missing ones are its defaults. |

```ogt
struct LayerDesc {
  doc "The element's subtree drawn offscreen, then composited through a material (a card that dissolves, a hologram panel)."
  desc true
  field material MaterialName "The material."
  field custom Option.UiMaterial "The custom material, for material custom."
  field params List.f32 "Its parameters: up to four vec4s, flattened (16 numbers); for a custom material, missing ones are its defaults."
}
```

<sub>`api.ogt` line 23042</sub>
