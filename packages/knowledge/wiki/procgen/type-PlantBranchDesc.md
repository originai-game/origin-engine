<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantBranchDesc"></a>

### `PlantBranchDesc` (desc struct)

One limb of an explicit parent-first plant skeleton. No procedural branches are added.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `parent` | `number` | `-1` | — | — | — | Earlier branch index; -1 only for the first, sole root. |
| `path` | `readonly Vector3[]` | `[]` | — | — | — | 2..1025 ring centres in metres. The first child centre lies on its parent's centreline within 0.000001 m. Consecutive centres differ. |
| `radii` | `Float32Array` | — | — | — | — | Radius per ring, in metres, each finite and at least 0.0001. Exact authored radii replace taper. |
| `segments` | `number` | `6` | — | — | — | Sides per ring, 3..32. |

```ogt
struct PlantBranchDesc {
  doc "One limb of an explicit parent-first plant skeleton. No procedural branches are added."
  desc true
  field parent i32 "Earlier branch index; -1 only for the first, sole root." {
    default -1
  }
  field path List.Vector3 "2..1025 ring centres in metres. The first child centre lies on its parent's centreline within 0.000001 m. Consecutive centres differ."
  field radii Float32Array "Radius per ring, in metres, each finite and at least 0.0001. Exact authored radii replace taper."
  field segments u32 "Sides per ring, 3..32." {
    default 6
  }
}
```

<sub>Module `procgen` line 792</sub>
