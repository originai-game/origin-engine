<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-setMouse"></a>

### `og.input.setMouse`

Set how the host reads the mouse (MouseDesc: raw input). Host only, never world state; fields left out keep their values.

```ts
og.input.setMouse(world: World, desc?: MouseDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `MouseDesc` | yes | The settings. |

**Returns** `Result<void>`: Refused for a world that is not alive.

Binding symbol `og_input_set_mouse` · publish builds: kept

```ogt
fn setMouse {
  doc "Set how the host reads the mouse (MouseDesc: raw input). Host only, never world state; fields left out keep their values."
  param world World "The world."
  param desc MouseDesc "The settings."
  returns Result.Void "Refused for a world that is not alive."
}
```

<sub>`api.ogt` line 2514</sub>
