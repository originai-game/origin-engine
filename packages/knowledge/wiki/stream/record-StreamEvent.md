<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="record-StreamEvent"></a>

### `og.StreamEvent` (event)

A group changed state: sent by og.stream at the start of the tick after the change.

Token `og.StreamEvent`, registered under the schema name `og.StreamEvent`; its value type is `StreamEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `group` | `string` | `""` | — | — | — | The group's name (at most 63 bytes). |
| `state` | `number` | `0` | — | — | — | Its new state: STREAM_UNLOADED, STREAM_QUEUED, STREAM_LOADING, STREAM_RESIDENT or STREAM_FAILED (StreamState's order). |

```ogt
event StreamEvent {
  doc "A group changed state: sent by og.stream at the start of the tick after the change."
  field group String "The group's name (at most 63 bytes)."
  field state u32 "Its new state: STREAM_UNLOADED, STREAM_QUEUED, STREAM_LOADING, STREAM_RESIDENT or STREAM_FAILED (StreamState's order)."
}
```

<sub>`api.ogt` line 25572</sub>
