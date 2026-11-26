<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantBoundsDesc"></a>

### `PlantBoundsDesc` (desc struct)

Optional centreline growth box in local metres, min < max on every axis.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Inclusive minimum corner. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Inclusive maximum corner. |

```ogt
struct PlantBoundsDesc {
  doc "Optional centreline growth box in local metres, min < max on every axis."
  desc true
  field min Vector3 "Inclusive minimum corner."
  field max Vector3 "Inclusive maximum corner."
}
```

<sub>Module `procgen` line 805</sub>
