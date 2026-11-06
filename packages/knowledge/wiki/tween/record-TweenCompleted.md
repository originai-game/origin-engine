<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="record-TweenCompleted"></a>

### `og.TweenCompleted` (event)

A tween or group ended on its own (read in the tick after).

Token `og.TweenCompleted`, registered under the schema name `og.TweenCompleted`; its value type is `TweenCompleted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tween` | `Tween` | `0` | — | — | — | The handle og.tween.to, sequence, parallel, squash or flash answered. |
| `entity` | `Entity` | `0` | — | — | — | The entity of its (first) tween. |
| `tag` | `number` | `0` | — | — | — | Its tag. |

```ogt
event TweenCompleted {
  doc "A tween or group ended on its own (read in the tick after)."
  field tween Tween "The handle og.tween.to, sequence, parallel, squash or flash answered."
  field entity Entity "The entity of its (first) tween."
  field tag u32 "Its tag."
}
```

<sub>`api.ogt` line 12444</sub>
