<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-actionDown"></a>

### `og.input.actionDown`

Whether an action is down this tick (false for an unknown one).

```ts
og.input.actionDown(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_action_down` · publish builds: kept

```ogt
fn actionDown {
  doc "Whether an action is down this tick (false for an unknown one)."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>`api.ogt` line 2310</sub>
