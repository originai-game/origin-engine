<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavBox"></a>

### `NavBox` (desc struct)

An axis-aligned box.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Its lowest corner. |
| `max` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Its highest corner. |

```ogt
struct NavBox {
  doc "An axis-aligned box."
  desc true
  field min Vector3 "Its lowest corner." {
    unit m
  }
  field max Vector3 "Its highest corner." {
    unit m
  }
}
```

<sub>Module `nav` line 433</sub>
