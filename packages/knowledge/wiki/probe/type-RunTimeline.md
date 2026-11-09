<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-RunTimeline"></a>

### `RunTimeline` (struct)

What happened since the last timeline call, in order.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick now. |
| `screens` | `readonly string[]` | `[]` | — | — | — | The UI's screen stack now, bottom first; empty without screens. |
| `events` | `readonly RunEvent[]` | `[]` | — | — | — | The events, in the order they happened (at most 4,096 between calls). |
| `dropped` | `number` | `0` | — | — | — | Events beyond 4,096 between calls, not kept. |

```ogt
struct RunTimeline {
  doc "What happened since the last timeline call, in order."
  field tick u32 "The tick now."
  field screens List.String "The UI's screen stack now, bottom first; empty without screens."
  field events List.RunEvent "The events, in the order they happened (at most 4,096 between calls)."
  field dropped u32 "Events beyond 4,096 between calls, not kept."
}
```

<sub>Module `probe` line 259</sub>
