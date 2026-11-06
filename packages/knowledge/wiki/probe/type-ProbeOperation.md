<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeOperation"></a>

### `ProbeOperation` (union)

One operation of a probe program, which runs in one evaluation so a scripted segment costs one round trip.

Tagged by `op`.

| Case | Payload | Doc |
|---|---|---|
| `op: "advance"` | [`AdvanceOperation`](../probe/type-AdvanceOperation.md#type-AdvanceOperation) | Take fixed steps; answers the tick. |
| `op: "input"` | [`InputOperation`](../probe/type-InputOperation.md#type-InputOperation) | Inject an event; answers true. |
| `op: "pause"` | — | Hold the clock; answers true. |
| `op: "resume"` | — | Release the clock; answers true. |
| `op: "read"` | — | Read the state; answers `{ tick, state }`. |
| `op: "pick"` | [`PickOperation`](../probe/type-PickOperation.md#type-PickOperation) | Pick a point; answers its result or null. |
| `op: "entity"` | [`EntityOperation`](../probe/type-EntityOperation.md#type-EntityOperation) | Read an entity's parameters; null is a refusal. |
| `op: "expect"` | [`ExpectOperation`](../probe/type-ExpectOperation.md#type-ExpectOperation) | Evaluate an expectation; answers its ExpectResult, failed or not (a malformed one is a refusal). |
| `op: "describe"` | — | Describe the frame; answers its Description. |

```ogt
union ProbeOperation {
  doc "One operation of a probe program, which runs in one evaluation so a scripted segment costs one round trip."
  tag op
  case advance AdvanceOperation "Take fixed steps; answers the tick."
  case input InputOperation "Inject an event; answers true."
  case pause "Hold the clock; answers true."
  case resume "Release the clock; answers true."
  case read "Read the state; answers `{ tick, state }`."
  case pick PickOperation "Pick a point; answers its result or null."
  case entity EntityOperation "Read an entity's parameters; null is a refusal."
  case expect ExpectOperation "Evaluate an expectation; answers its ExpectResult, failed or not (a malformed one is a refusal)."
  case describe "Describe the frame; answers its Description."
}
```

<sub>`api.ogt` line 24176</sub>
