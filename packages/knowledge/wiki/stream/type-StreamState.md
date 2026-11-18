<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="type-StreamState"></a>

### `StreamState` (enum)

Where a streaming group is.

A string: one of the variants below. The zero value is `"unloaded"`.

| Variant | Doc |
|---|---|
| `"unloaded"` | Not wanted; models stay hidden, textures use stand-ins and sounds play silence. |
| `"queued"` | Wanted, waiting for room in the memory budget. |
| `"loading"` | Admitted: its pack is being read (its memory already counts). |
| `"resident"` | Its pack is available. Textures may still be preparing; StreamStatus.ready is presentation readiness. Its tick is host input: the same on every native run, not on the web; simulation that must match across hosts waits on a cell group (world §4). |
| `"failed"` | Its pack could not be read, or it alone needs more than the budget (StreamStatus.reason says which). |

```ogt
enum StreamState {
  doc "Where a streaming group is."
  variant unloaded "Not wanted; models stay hidden, textures use stand-ins and sounds play silence."
  variant queued "Wanted, waiting for room in the memory budget."
  variant loading "Admitted: its pack is being read (its memory already counts)."
  variant resident "Its pack is available. Textures may still be preparing; StreamStatus.ready is presentation readiness. Its tick is host input: the same on every native run, not on the web; simulation that must match across hosts waits on a cell group (world §4)."
  variant failed "Its pack could not be read, or it alone needs more than the budget (StreamStatus.reason says which)."
}
```

<sub>Module `stream` line 57</sub>
