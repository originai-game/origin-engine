<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantGrowth"></a>

### `PlantGrowth` (enum)

Branching architecture of a procedural plant.

A string: one of the variants below. The zero value is `"deciduous"`.

| Variant | Doc |
|---|---|
| `"deciduous"` | Broadleaf branching: every branch continues into deeper child levels. |
| `"evergreen"` | Conifer branching: tapered stem with whorls shortening towards the apex. |

```ogt
enum PlantGrowth {
  doc "Branching architecture of a procedural plant."
  variant deciduous "Broadleaf branching: every branch continues into deeper child levels."
  variant evergreen "Conifer branching: tapered stem with whorls shortening towards the apex."
}
```

<sub>Module `procgen` line 568</sub>
