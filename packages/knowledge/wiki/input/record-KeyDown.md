<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-KeyDown"></a>

### `og.KeyDown` (event)

A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems.

Token `og.KeyDown`, registered under the schema name `og.KeyDown`; its value type is `KeyDown`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The physical key code. |
| `repeat` | `boolean` | `false` | — | — | — | Whether this is an auto-repeat. |
| `at` | `number` | `0` | -1 … 1 | — | — | When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it. |

```ogt
event KeyDown {
  doc "A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems."
  field key String "The physical key code."
  field repeat bool "Whether this is an auto-repeat."
  field at f32 "When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. The shell stamps it from the browser event's timeStamp, the command channel from the step's time; it is part of the event, so a replay carries it." {
    min -1
    max 1
  }
}
```

<sub>Module `input` line 1142</sub>
