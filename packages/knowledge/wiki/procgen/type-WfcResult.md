<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-WfcResult"></a>

### `WfcResult` (struct)

What og.procgen.wfc made, row-major from the top, returned as native TypedArrays over binary wire.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tiles` | `Uint32Array` | — | — | — | — | Each cell's tile index. |
| `rotations` | `Uint8Array` | — | — | — | — | Each cell's quarter turns clockwise (0 to 3). |
| `attempt` | `number` | `0` | — | — | — | The try that succeeded, from 1. |

```ogt
struct WfcResult {
  doc "What og.procgen.wfc made, row-major from the top, returned as native TypedArrays over binary wire."
  field tiles Uint32Array "Each cell's tile index."
  field rotations Uint8Array "Each cell's quarter turns clockwise (0 to 3)."
  field attempt u32 "The try that succeeded, from 1."
}
```

<sub>Module `procgen` line 286</sub>
