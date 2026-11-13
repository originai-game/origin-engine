<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-KeyUp"></a>

### `og.KeyUp` (event)

A key went up.

Token `og.KeyUp`, registered under the schema name `og.KeyUp`; its value type is `KeyUp`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |
| `at` | `number` | `0` | -1 … 1 | — | — | When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. Part of the event, so a replay carries it. See `input §6`. |

```ogt
event KeyUp {
  doc "A key went up."
  field key String "The physical key code."
  field at f32 "When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. Part of the event, so a replay carries it." {
    guide "input §6"
    min -1
    max 1
  }
}
```

<sub>Module `input` line 1230</sub>
