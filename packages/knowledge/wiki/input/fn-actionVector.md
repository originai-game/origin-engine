<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-actionVector"></a>

### `og.input.actionVector`

A two-axis action's value (x right, y up; each -1 to 1 but the mouse's movement); (value, 0) for the others.

```ts
og.input.actionVector(world: World, name: string): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `Vector2`.

Binding symbol `og_input_action_vector` · publish builds: kept

```ogt
fn actionVector {
  doc "A two-axis action's value (x right, y up; each -1 to 1 but the mouse's movement); (value, 0) for the others."
  param world World "The world."
  param name String "The action."
  returns Vector2
}
```

<sub>Module `input` line 343</sub>
