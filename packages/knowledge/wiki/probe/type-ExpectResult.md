<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ExpectResult"></a>

### `ExpectResult` (struct)

One evaluated expectation.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick it was evaluated at. |
| `at` | `number \| null` | `null` | — | `s` | — | The script step's time, when a script ran it. |
| `expect` | `string` | `""` | — | — | — | As written after `expect:`. |
| `kind` | `string` | `""` | — | — | — | `entity`, `event`, `ui`, `region`, `frame` or `state`. |
| `status` | `ExpectStatus` | `"pass"` | — | — | — | The outcome. |
| `expected` | `unknown` | `null` | — | — | — | What was expected. |
| `observed` | `unknown` | `null` | — | — | — | What was there. |
| `reason` | `string` | `""` | — | — | — | Why it failed or was unchecked; empty on a pass. |

```ogt
struct ExpectResult {
  doc "One evaluated expectation."
  field tick u32 "The tick it was evaluated at."
  field at Option.f32 "The script step's time, when a script ran it." {
    unit s
  }
  field expect String "As written after `expect:`."
  field kind String "`entity`, `event`, `ui`, `region`, `frame` or `state`."
  field status ExpectStatus "The outcome."
  field expected Json "What was expected."
  field observed Json "What was there."
  field reason String "Why it failed or was unchecked; empty on a pass."
}
```

<sub>Module `probe` line 620</sub>
