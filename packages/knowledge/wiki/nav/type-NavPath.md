<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavPath"></a>

### `NavPath` (struct)

A path.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | — | — | Its corners, start and end included. |
| `length` | `number` | `0` | — | `m` | — | Its length. |
| `complete` | `boolean` | `false` | — | — | — | Whether it reaches the end (false: it stops at the reachable point nearest it). |
| `expanded` | `number` | `0` | — | — | — | Cells or polygons the search took off its open list. |
| `algorithm` | `string` | `""` | — | — | — | What ran: jps, astar or navmesh. |
| `links` | `readonly number[]` | `[]` | — | — | — | Where it crosses off-mesh links: each an index into points, the link's start (the next point is where it lands). |

```ogt
struct NavPath {
  doc "A path."
  field points List.Vector3 "Its corners, start and end included."
  field length f32 "Its length." {
    unit m
  }
  field complete bool "Whether it reaches the end (false: it stops at the reachable point nearest it)."
  field expanded u32 "Cells or polygons the search took off its open list."
  field algorithm String "What ran: jps, astar or navmesh."
  field links List.u32 "Where it crosses off-mesh links: each an index into points, the link's start (the next point is where it lands)."
}
```

<sub>Module `nav` line 414</sub>
