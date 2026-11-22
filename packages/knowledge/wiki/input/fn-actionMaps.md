<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-actionMaps"></a>

### `og.input.actionMaps`

The live action maps as og.input.setActionMaps set them (empty: every action map).

```ts
og.input.actionMaps(world: World): readonly string[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly string[]`.

Binding symbol `og_input_action_maps` · publish builds: kept

```ogt
fn actionMaps {
  doc "The live action maps as og.input.setActionMaps set them (empty: every action map)."
  param world World "The world."
  returns List.String
}
```

<sub>Module `input` line 399</sub>
