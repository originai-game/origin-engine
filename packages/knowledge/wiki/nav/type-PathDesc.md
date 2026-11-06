<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-PathDesc"></a>

### `PathDesc` (desc struct)

A path query.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `nav` | `Navigation` | `0` | — | — | — | The grid or navmesh. |
| `from` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The start. |
| `to` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The end. |
| `algorithm` | `NavAlgorithm` | `"jps"` | — | — | — | The grid search. |
| `smooth` | `boolean` | `true` | — | — | — | Pull a grid path straight where nothing is in the way. |
| `radius` | `number` | `0` | 0 … 10 | `m` | — | Keep this clear of grid walls when smoothing. |
| `areaCosts` | `readonly number[]` | `[]` | — | — | — | On a navmesh: per area id, the cost of a metre in it, as og.NavFilter's. |
| `excluded` | `readonly number[]` | `[]` | — | — | — | On a navmesh: area ids the path never enters. |

```ogt
struct PathDesc {
  doc "A path query."
  desc true
  field nav Navigation "The grid or navmesh."
  field from Vector3 "The start." {
    unit m
  }
  field to Vector3 "The end." {
    unit m
  }
  field algorithm NavAlgorithm "The grid search."
  field smooth bool "Pull a grid path straight where nothing is in the way." {
    default true
  }
  field radius f32 "Keep this clear of grid walls when smoothing." {
    min 0
    max 10
    unit m
  }
  field areaCosts List.f32 "On a navmesh: per area id, the cost of a metre in it, as og.NavFilter's."
  field excluded List.u32 "On a navmesh: area ids the path never enters."
}
```

<sub>`api.ogt` line 13910</sub>
