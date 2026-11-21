<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-isActionJustReleased"></a>

### `og.input.isActionJustReleased`

Whether an action went up this tick.

```ts
og.input.isActionJustReleased(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_is_action_just_released` · publish builds: kept

```ogt
fn isActionJustReleased {
  doc "Whether an action went up this tick."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>Module `input` line 361</sub>
