<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-AdvanceOperation"></a>

### `AdvanceOperation` (struct)

The advance operation's payload.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `steps` | `number` | `0` | — | — | — | Fixed steps to take. |

```ogt
struct AdvanceOperation {
  doc "The advance operation's payload."
  field steps u32 "Fixed steps to take."
}
```

<sub>Module `probe` line 104</sub>
