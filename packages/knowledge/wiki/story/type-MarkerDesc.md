<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-MarkerDesc"></a>

### `MarkerDesc` (desc struct)

Where a quest's objective is, for the map, compass and tracker.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | An entity to point at (0: none). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | A point to point at, when there is no entity. |

```ogt
struct MarkerDesc {
  doc "Where a quest's objective is, for the map, compass and tracker."
  desc true
  field entity Entity "An entity to point at (0: none)."
  field position Vector3 "A point to point at, when there is no entity." {
    unit m
  }
}
```

<sub>Module `story` line 203</sub>
