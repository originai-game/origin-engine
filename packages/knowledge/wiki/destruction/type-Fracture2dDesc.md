<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="type-Fracture2dDesc"></a>

### `Fracture2dDesc` (desc struct)

How og.destruction.fracture2d breaks a piece.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pieces` | `number` | `8` | 1 … 256 | — | — | How many Voronoi pieces (fewer when some are under the terrain's minPiece cells). |

```ogt
struct Fracture2dDesc {
  doc "How og.destruction.fracture2d breaks a piece."
  desc true
  field pieces u32 "How many Voronoi pieces (fewer when some are under the terrain's minPiece cells)." {
    default 8
    min 1
    max 256
  }
}
```

<sub>Module `destruction` line 465</sub>
