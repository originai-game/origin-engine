<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphCondition"></a>

### `GraphCondition` (struct)

A transition's parameter comparison; all its conditions must hold.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `param` | `string` | `""` | — | — | — | Declared parameter name. |
| `op` | `GraphComparison` | `"gt"` | — | — | — | Comparison. |
| `value` | `number` | `0` | — | — | — | Finite comparison value. |

```ogt
struct GraphCondition {
  doc "A transition's parameter comparison; all its conditions must hold."
  field param String "Declared parameter name."
  field op GraphComparison "Comparison."
  field value f32 "Finite comparison value."
}
```

<sub>Module `anim` line 755</sub>
