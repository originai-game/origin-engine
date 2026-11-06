<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-actionPressed"></a>

### `og.input.actionPressed`

Whether an action went down this tick.

```ts
og.input.actionPressed(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_action_pressed` · publish builds: kept

```ogt
fn actionPressed {
  doc "Whether an action went down this tick."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>`api.ogt` line 2317</sub>
