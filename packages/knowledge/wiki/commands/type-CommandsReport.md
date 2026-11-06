<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`commands`](../commands.md) · [index](../index.md)

<a id="type-CommandsReport"></a>

### `CommandsReport` (struct)

What scheduling a script did.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheduled` | `number` | `0` | — | — | — | Steps scheduled. |
| `untilTick` | `number` | `0` | — | — | — | The tick of the last step. |
| `warnings` | `readonly string[]` | `[]` | — | — | — | Unknown actions and other skipped steps, each naming the fix. |

```ogt
struct CommandsReport {
  doc "What scheduling a script did."
  field scheduled u32 "Steps scheduled."
  field untilTick u32 "The tick of the last step."
  field warnings List.String "Unknown actions and other skipped steps, each naming the fix."
}
```

<sub>`api.ogt` line 23987</sub>
