<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeGetResult"></a>

### `ProbeGetResult` (struct)

Outcome of a synchronous probe get call.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ok` | `boolean` | `false` | — | — | — | True when the target was read. |
| `value` | `unknown` | `null` | — | — | optional | The read value, when ok is true. |
| `error` | `string` | `""` | — | — | optional | Why the read was refused, when ok is false. |

```ogt
struct ProbeGetResult {
  doc "Outcome of a synchronous probe get call."
  field ok bool "True when the target was read."
  field value Json "The read value, when ok is true." {
    optional true
  }
  field error String "Why the read was refused, when ok is false." {
    optional true
  }
}
```

<sub>Module `probe` line 801</sub>
