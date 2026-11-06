<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-MotionPresentationState"></a>

### `MotionPresentationState` (enum)

Presentation progress, dependent on GPU timing, never simulation state.

A string: one of the variants below. The zero value is `"loading"`.

| Variant | Doc |
|---|---|
| `"loading"` | Waiting for the model or renderer. |
| `"warming"` | Weights are resident; this agent has no generated palette yet. |
| `"ready"` | Generated frames and their palette are ready for the current rendered frame; authored fallback never counts. |
| `"failed"` | Loading, inference or this agent's retarget failed; reason explains why. |

```ogt
enum MotionPresentationState {
  doc "Presentation progress, dependent on GPU timing, never simulation state."
  variant loading "Waiting for the model or renderer."
  variant warming "Weights are resident; this agent has no generated palette yet."
  variant ready "Generated frames and their palette are ready for the current rendered frame; authored fallback never counts."
  variant failed "Loading, inference or this agent's retarget failed; reason explains why."
}
```

<sub>`api.ogt` line 26984</sub>
