<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-KeyUp"></a>

### `og.KeyUp` (event)

A key went up.

Token `og.KeyUp`, registered under the schema name `og.KeyUp`; its value type is `KeyUp`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |
| `at` | `number` | `0` | -1 … 1 | — | — | When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it. |

```ogt
event KeyUp {
  doc "A key went up."
  field key String "The physical key code."
  field at f32 "When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it." {
    min -1
    max 1
  }
}
```

<sub>`api.ogt` line 3159</sub>
