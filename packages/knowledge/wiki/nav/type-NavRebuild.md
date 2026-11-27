<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavRebuild"></a>

### `NavRebuild` (struct)

What an obstacle change rebuilt.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tiles` | `readonly number[]` | `[]` | — | — | — | The tiles rebuilt, by index. |
| `polygons` | `number` | `0` | — | — | — | The navmesh's polygons now. |

```ogt
struct NavRebuild {
  doc "What an obstacle change rebuilt."
  field tiles List.u32 "The tiles rebuilt, by index."
  field polygons u32 "The navmesh's polygons now."
}
```

<sub>Module `nav` line 496</sub>
