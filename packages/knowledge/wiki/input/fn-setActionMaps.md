<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-setActionMaps"></a>

### `og.input.setActionMaps`

The action maps whose actions are live, first the one whose bindings win a control another action map also binds (`["vehicle", "gameplay"]`: E exits the car, not interacts). Empty (the default) is every action map, none winning. Takes effect from the next tick's actions.

```ts
og.input.setActionMaps(world: World, actionMaps: readonly string[]): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `actionMaps` | `readonly string[]` | no | The live action maps, by priority. |

**Returns** `void`.

Binding symbol `og_input_set_action_maps` · publish builds: kept

```ogt
fn setActionMaps {
  doc "The action maps whose actions are live, first the one whose bindings win a control another action map also binds (`[\"vehicle\", \"gameplay\"]`: E exits the car, not interacts). Empty (the default) is every action map, none winning. Takes effect from the next tick's actions."
  param world World "The world."
  param actionMaps List.String "The live action maps, by priority."
  returns Void
}
```

<sub>Module `input` line 389</sub>
