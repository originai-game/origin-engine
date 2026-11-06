<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeProgramResult"></a>

### `ProbeProgramResult` (struct)

What a program observed: one answer per operation that ran. A false, or entity's null, stops the program and `refused` names its index.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `results` | `readonly unknown[]` | `[]` | — | — | — | The answers, in order. |
| `refused` | `number \| null` | `null` | — | — | — | The index of the refusing operation, or null. |
| `tick` | `number` | `0` | — | — | — | The tick after the last operation that ran. |

```ogt
struct ProbeProgramResult {
  doc "What a program observed: one answer per operation that ran. A false, or entity's null, stops the program and `refused` names its index."
  field results List.Json "The answers, in order."
  field refused Option.u32 "The index of the refusing operation, or null."
  field tick u32 "The tick after the last operation that ran."
}
```

<sub>`api.ogt` line 24190</sub>
