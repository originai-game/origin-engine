<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-FoliageDesc"></a>

### `FoliageDesc` (desc struct)

Leaf lighting. Zero keeps ordinary opaque-surface lighting; this does not enable glass transmission or change alpha coverage.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `translucency` | `number` | `0` | 0 … 1 | — | — | Thin-leaf backlighting tinted by the base colour and attenuated by the light's shadow. |
| `domeNormals` | `number` | `0` | 0 … 1 | — | — | Blend leaf normals towards the canopy dome, before normal mapping. Requires cooked foliage anchors. |

```ogt
struct FoliageDesc {
  doc "Leaf lighting. Zero keeps ordinary opaque-surface lighting; this does not enable glass transmission or change alpha coverage."
  desc true
  field translucency f32 "Thin-leaf backlighting tinted by the base colour and attenuated by the light's shadow." {
    min 0
    max 1
  }
  field domeNormals f32 "Blend leaf normals towards the canopy dome, before normal mapping. Requires cooked foliage anchors." {
    min 0
    max 1
  }
}
```

<sub>Module `render` line 146</sub>
