<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-action"></a>

### `og.input.action`

An action this tick: down, the edges, held, its value and vector, and the device that drove it. Null for an unknown action.

```ts
og.input.action(world: World, name: string): ActionState | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `ActionState | null`.

Binding symbol `og_input_action` · publish builds: kept

```ogt
fn action {
  doc "An action this tick: down, the edges, held, its value and vector, and the device that drove it. Null for an unknown action."
  param world World "The world."
  param name String "The action."
  returns Option.ActionState
}
```

<sub>`api.ogt` line 2303</sub>
