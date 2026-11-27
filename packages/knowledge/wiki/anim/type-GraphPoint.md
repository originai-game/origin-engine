<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphPoint"></a>

### `GraphPoint` (struct)

One child of a Cartesian blend.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `motion` | `string` | `""` | — | — | — | A key in GraphDesc.motions. |
| `point` | `[number, number]` | `[0, 0]` | — | — | — | The child's x and y parameter values. |

```ogt
struct GraphPoint {
  doc "One child of a Cartesian blend."
  field motion String "A key in GraphDesc.motions."
  field point Array.f32.2 "The child's x and y parameter values."
}
```

<sub>Module `anim` line 717</sub>
