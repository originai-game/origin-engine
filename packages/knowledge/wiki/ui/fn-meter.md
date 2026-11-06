<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-meter"></a>

### `og.ui.meter`

A health, mana or xp bar. A drop shows at once and leaves a pale trail that follows after a beat; a rise grows into place with a ghost of the target; both by the tick.

```ts
og.ui.meter(id: string, value: number, max: number, desc?: MeterDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The current value. |
| `max` | `number` | no | The full value. |
| `desc` | `MeterDesc` | yes | Kind, label, size and notches. |

**Returns** `void`.

Binding symbol `og_ui_meter` · publish builds: kept

```ogt
fn meter {
  doc "A health, mana or xp bar. A drop shows at once and leaves a pale trail that follows after a beat; a rise grows into place with a ghost of the target; both by the tick."
  param id String "Its id."
  param value f32 "The current value."
  param max f32 "The full value."
  param desc MeterDesc "Kind, label, size and notches."
  returns Void
}
```

<sub>`api.ogt` line 17258</sub>
