<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-PointerDown"></a>

### `og.PointerDown` (event)

A pointer button went down (or a touch landed).

Token `og.PointerDown`, registered under the schema name `og.PointerDown`; its value type is `PointerDown`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | Horizontal position in CSS pixels. |
| `y` | `number` | `0` | — | — | — | Vertical position in CSS pixels. |
| `button` | `PointerButton` | `"none"` | — | — | — | The button. |
| `id` | `number` | `0` | — | — | — | Which pointer: 0 the primary one, 1 to 4 further touches (og.Pointers). |
| `at` | `number` | `0` | -1 … 1 | — | — | When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. Part of the event, so a replay carries it. See `input §6`. |

```ogt
event PointerDown {
  doc "A pointer button went down (or a touch landed)."
  field x f32 "Horizontal position in CSS pixels."
  field y f32 "Vertical position in CSS pixels."
  field button PointerButton "The button."
  field id u32 "Which pointer: 0 the primary one, 1 to 4 further touches (og.Pointers)."
  field at f32 "When it happened, in ticks from the start of the tick that reads it, -1 to 1 (0 when the host gave no time): its world time is `time.elapsed + at * time.dt`. Part of the event, so a replay carries it." {
    guide "input §6"
    min -1
    max 1
  }
}
```

<sub>Module `input` line 1230</sub>
