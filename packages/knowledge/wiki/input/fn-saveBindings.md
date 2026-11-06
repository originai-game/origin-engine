<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-saveBindings"></a>

### `og.input.saveBindings`

The player's settings as text to keep (og.platform.saveSettings, a save): every rebound action's bindings, the toggles and the dead zones that differ from the defaults.

```ts
og.input.saveBindings(world: World): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `string`.

Binding symbol `og_input_save_bindings` · publish builds: kept

```ogt
fn saveBindings {
  doc "The player's settings as text to keep (og.platform.saveSettings, a save): every rebound action's bindings, the toggles and the dead zones that differ from the defaults."
  param world World "The world."
  returns String
}
```

<sub>Module `input` line 395</sub>
