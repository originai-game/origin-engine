<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-setToggle"></a>

### `og.input.setToggle`

The player's choice for a button action: a press turns it on and the next off (true), or it is down while held (false).

```ts
og.input.setToggle(world: World, action: string, on: boolean): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |
| `on` | `boolean` | no | Toggle. |

**Returns** `Result<void>`.

Binding symbol `og_input_set_toggle` · publish builds: kept

```ogt
fn setToggle {
  doc "The player's choice for a button action: a press turns it on and the next off (true), or it is down while held (false)."
  param world World "The world."
  param action String "The action."
  param on bool "Toggle."
  returns Result.Void
}
```

<sub>`api.ogt` line 2401</sub>
