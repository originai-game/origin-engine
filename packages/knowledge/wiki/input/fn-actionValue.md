<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-actionValue"></a>

### `og.input.actionValue`

An action's value: a button's strength (0 to 1), an axis's value, a two-axis value's length. 0 for an unknown action.

```ts
og.input.actionValue(world: World, name: string): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `number`.

Binding symbol `og_input_action_value` · publish builds: kept

```ogt
fn actionValue {
  doc "An action's value: a button's strength (0 to 1), an axis's value, a two-axis value's length. 0 for an unknown action."
  param world World "The world."
  param name String "The action."
  returns f32
}
```

<sub>Module `input` line 336</sub>
