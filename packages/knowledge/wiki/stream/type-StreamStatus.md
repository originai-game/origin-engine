<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="type-StreamStatus"></a>

### `StreamStatus` (struct)

One streaming group.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `scene` | `string` | `""` | — | — | — | The scene that declares it. |
| `state` | `StreamState` | `"unloaded"` | — | — | — | Where it is. |
| `wanted` | `boolean` | `false` | — | — | — | Whether its trigger or a request wants it now. |
| `requested` | `boolean` | `false` | — | — | — | Whether og.stream.request asked for it (until og.stream.release). |
| `priority` | `number` | `0` | — | — | — | Higher loads first and is evicted last. |
| `trigger` | `string` | `""` | — | — | — | manual, distance, volume or cell. |
| `memory` | `number` | `0` | — | `bytes` | — | What its assets take once loaded, in bytes. |
| `bytes` | `number` | `0` | — | `bytes` | — | Its pack's size, in bytes. |
| `assets` | `readonly string[]` | `[]` | — | — | — | Its asset ids. |
| `reason` | `string` | `""` | — | — | — | Why it is queued or failed; empty otherwise. |
| `since` | `number` | `0` | — | — | — | The tick of its last change. |
| `loads` | `number` | `0` | — | — | — | How many times it became resident. |
| `kept` | `number` | `0` | — | `bytes` | — | The bytes of its assets kept for the frame after it unloaded (counted in StreamBudget.used until let go); 0 otherwise. |

```ogt
struct StreamStatus {
  doc "One streaming group."
  field name String "Its name."
  field scene String "The scene that declares it."
  field state StreamState "Where it is."
  field wanted bool "Whether its trigger or a request wants it now."
  field requested bool "Whether og.stream.request asked for it (until og.stream.release)."
  field priority i32 "Higher loads first and is evicted last."
  field trigger String "manual, distance, volume or cell."
  field memory f64 "What its assets take once loaded, in bytes." {
    unit bytes
  }
  field bytes f64 "Its pack's size, in bytes." {
    unit bytes
  }
  field assets List.String "Its asset ids."
  field reason String "Why it is queued or failed; empty otherwise."
  field since f64 "The tick of its last change."
  field loads u32 "How many times it became resident."
  field kept f64 "The bytes of its assets kept for the frame after it unloaded (counted in StreamBudget.used until let go); 0 otherwise." {
    unit bytes
  }
}
```

<sub>Module `stream` line 80</sub>
