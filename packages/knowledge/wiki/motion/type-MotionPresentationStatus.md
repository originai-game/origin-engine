<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-MotionPresentationStatus"></a>

### `MotionPresentationStatus` (struct)

Visual readiness only, outside snapshots and hashes.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `state` | `MotionPresentationState` | `"loading"` | — | — | — | The agent's presentation progress. |
| `reason` | `string` | `""` | — | — | — | Why it is loading, warming or failed; empty when ready. |

```ogt
struct MotionPresentationStatus {
  doc "Visual readiness only, outside snapshots and hashes."
  field state MotionPresentationState "The agent's presentation progress."
  field reason String "Why it is loading, warming or failed; empty when ready."
}
```

<sub>Module `motion` line 238</sub>
