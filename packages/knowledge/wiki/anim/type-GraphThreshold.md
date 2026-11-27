<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphThreshold"></a>

### `GraphThreshold` (struct)

One child of a threshold blend.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `motion` | `string` | `""` | — | — | — | A key in GraphDesc.motions. |
| `threshold` | `number` | `0` | — | — | — | Parameter value at this child. Children sort by threshold. |

```ogt
struct GraphThreshold {
  doc "One child of a threshold blend."
  field motion String "A key in GraphDesc.motions."
  field threshold f32 "Parameter value at this child. Children sort by threshold."
}
```

<sub>Module `anim` line 712</sub>
