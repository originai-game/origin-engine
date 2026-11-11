<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-isActionJustPressed"></a>

### `og.input.isActionJustPressed`

Whether an action went down this tick.

Also known as: button down this frame, GetButtonDown, is_action_just_pressed

```ts
og.input.isActionJustPressed(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_is_action_just_pressed` · publish builds: kept

```ogt
fn isActionJustPressed {
  doc "Whether an action went down this tick."
  aliases "button down this frame, GetButtonDown, is_action_just_pressed"
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>Module `input` line 356</sub>
