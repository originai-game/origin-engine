<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeLogEntry"></a>

### `ProbeLogEntry` (struct)

One probe log or warning entry.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick when it was recorded. |
| `warning` | `boolean` | `false` | — | — | — | True for a warning or refusal reason. |
| `text` | `string` | `""` | — | — | — | The message. |

```ogt
struct ProbeLogEntry {
  doc "One probe log or warning entry."
  field tick u32 "The tick when it was recorded."
  field warning bool "True for a warning or refusal reason."
  field text String "The message."
}
```

<sub>Module `probe` line 780</sub>
