<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeSetResult"></a>

### `ProbeSetResult` (struct)

Outcome of a synchronous probe set call.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ok` | `boolean` | `false` | — | — | — | True when the field was written. |
| `error` | `string` | `""` | — | — | optional | Why the write was refused, when ok is false. |

```ogt
struct ProbeSetResult {
  doc "Outcome of a synchronous probe set call."
  field ok bool "True when the field was written."
  field error String "Why the write was refused, when ok is false." {
    optional true
  }
}
```

<sub>Module `probe` line 793</sub>
