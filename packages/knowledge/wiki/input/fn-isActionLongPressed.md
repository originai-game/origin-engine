<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-isActionLongPressed"></a>

### `og.input.isActionLongPressed`

Whether an action has been down for at least its ActionDesc.holdMs: a long press, true from that tick until it goes up. isActionDown is down at all.

Also known as: long press, press and hold, hold button, isActionHeld

```ts
og.input.isActionLongPressed(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_input_is_action_long_pressed` · publish builds: kept

```ogt
fn isActionLongPressed {
  doc "Whether an action has been down for at least its ActionDesc.holdMs: a long press, true from that tick until it goes up. isActionDown is down at all."
  aliases "long press, press and hold, hold button, isActionHeld"
  param world World "The world."
  param name String "The action."
  returns bool
}
```

<sub>Module `input` line 361</sub>
