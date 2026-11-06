<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-WfcResult"></a>

### `WfcResult` (struct)

What og.procgen.wfc made, row-major from the top.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tiles` | `readonly number[]` | `[]` | — | — | — | Each cell's tile index. |
| `rotations` | `readonly number[]` | `[]` | — | — | — | Each cell's quarter turns clockwise. |
| `attempt` | `number` | `0` | — | — | — | The try that succeeded, from 1. |

```ogt
struct WfcResult {
  doc "What og.procgen.wfc made, row-major from the top."
  field tiles List.u32 "Each cell's tile index."
  field rotations List.u32 "Each cell's quarter turns clockwise."
  field attempt u32 "The try that succeeded, from 1."
}
```

<sub>Module `procgen` line 212</sub>
