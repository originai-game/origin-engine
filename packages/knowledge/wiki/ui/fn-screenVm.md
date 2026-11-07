<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-screenVm"></a>

### `og.ui.screenVm`

A layer's viewmodel as its two-way bindings left it (the top layer of that screen); null when it is not on the stack.

```ts
og.ui.screenVm(world: World, screen: string): unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `screen` | `string` | no | The screen's name. |

**Returns** `unknown`.

Binding symbol `og_ui_screen_vm` · publish builds: kept

```ogt
fn screenVm {
  doc "A layer's viewmodel as its two-way bindings left it (the top layer of that screen); null when it is not on the stack."
  param world World "The world."
  param screen String "The screen's name."
  returns Json
}
```

<sub>Module `ui` line 8041</sub>
