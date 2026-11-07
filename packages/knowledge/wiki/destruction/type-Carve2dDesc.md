<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="type-Carve2dDesc"></a>

### `Carve2dDesc` (desc struct)

A round hole for og.destruction.carve2d.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Its centre, in world space (its xy). |
| `radius` | `number` | `1` | ≥ 0 | `m` | — | Its radius: every cell whose centre is within it is emptied (bedrock stays). |

```ogt
struct Carve2dDesc {
  doc "A round hole for og.destruction.carve2d."
  desc true
  field point Vector3 "Its centre, in world space (its xy)." {
    unit m
  }
  field radius f32 "Its radius: every cell whose centre is within it is emptied (bedrock stays)." {
    default 1
    min 0
    unit m
  }
}
```

<sub>Module `destruction` line 411</sub>
