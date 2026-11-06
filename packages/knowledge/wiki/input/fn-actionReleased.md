<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-actionReleased"></a>

### `og.input.actionReleased`

Whether an action went up this tick.

```ts
og.input.actionReleased(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_action_released` · publish builds: kept

```ogt
fn actionReleased {
  doc "Whether an action went up this tick."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>`api.ogt` line 2324</sub>
