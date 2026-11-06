<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-actionHeld"></a>

### `og.input.actionHeld`

Whether an action has been down for its holdMs.

```ts
og.input.actionHeld(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_action_held` · publish builds: kept

```ogt
fn actionHeld {
  doc "Whether an action has been down for its holdMs."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>Module `input` line 329</sub>
