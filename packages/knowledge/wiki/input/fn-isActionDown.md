<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-isActionDown"></a>

### `og.input.isActionDown`

Whether an action is down this tick (false for an unknown one).

```ts
og.input.isActionDown(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_is_action_down` · publish builds: kept

```ogt
fn isActionDown {
  doc "Whether an action is down this tick (false for an unknown one)."
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>Module `input` line 350</sub>
