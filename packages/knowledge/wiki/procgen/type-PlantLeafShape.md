<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantLeafShape"></a>

### `PlantLeafShape` (enum)

Geometry built for each leaf.

A string: one of the variants below. The zero value is `"blade"`.

| Variant | Doc |
|---|---|
| `"blade"` | Six-vertex cupped pointed leaf blade (no texture required). |
| `"card"` | Single UV-mapped quad for a leaf alpha card. |
| `"cross"` | Two crossed UV-mapped quads. |

```ogt
enum PlantLeafShape {
  doc "Geometry built for each leaf."
  variant blade "Six-vertex cupped pointed leaf blade (no texture required)."
  variant card "Single UV-mapped quad for a leaf alpha card."
  variant cross "Two crossed UV-mapped quads."
}
```

<sub>Module `procgen` line 573</sub>
