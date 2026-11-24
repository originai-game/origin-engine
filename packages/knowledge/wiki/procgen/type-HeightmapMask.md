<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-HeightmapMask"></a>

### `HeightmapMask` (enum)

Macro shaping mask applied to generated terrain heights.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No mask (raw domain-warped FBM). |
| `"island"` | Radial coastal falloff pushing edges down to sea level. |
| `"ridge"` | Ridged mountain shaping blending sharp crests into the base FBM. |

```ogt
enum HeightmapMask {
  doc "Macro shaping mask applied to generated terrain heights."
  variant none "No mask (raw domain-warped FBM)."
  variant island "Radial coastal falloff pushing edges down to sea level."
  variant ridge "Ridged mountain shaping blending sharp crests into the base FBM."
}
```

<sub>Module `procgen` line 288</sub>
