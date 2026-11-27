<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-consume"></a>

### `og.input.consume`

Take an action's buffered press: true once per press, in the tick it went down or the ActionDesc.bufferTicks after it, then false until the next press. What a combo, a jump or a cancel window calls when it can act on the press, so one press starts one move.

Also known as: input buffer, buffered input, consume input, jump buffer, combo buffer

```ts
og.input.consume(world: World, name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The action. |

**Returns** `boolean`: Whether a press was waiting; false for an unknown action.

Binding symbol `og_input_consume` · publish builds: kept

```ogt
fn consume {
  doc "Take an action's buffered press: true once per press, in the tick it went down or the ActionDesc.bufferTicks after it, then false until the next press. What a combo, a jump or a cancel window calls when it can act on the press, so one press starts one move."
  aliases "input buffer, buffered input, consume input, jump buffer, combo buffer"
  param world World "The world."
  param name String "The action."
  returns bool "Whether a press was waiting; false for an unknown action."
}
```

<sub>Module `input` line 323</sub>
