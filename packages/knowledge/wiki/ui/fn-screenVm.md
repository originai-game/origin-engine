<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-screenVm"></a>

### `og.ui.screenVm`

A layer's viewmodel as its two-way bindings left it (the top layer of that screen); null when it is not on the stack.

```ts
og.ui.screenVm<S extends ScreenName>(world: World, screen: S): ScreenProps<S> | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `screen` | `S` | no | The screen's name. |

**Returns** `ScreenProps<S> | null`.

Binding symbol `og_ui_screen_vm` · publish builds: kept

```ogt
fn screenVm {
  doc "A layer's viewmodel as its two-way bindings left it (the top layer of that screen); null when it is not on the stack."
  generic S ScreenName
  param world World "The world."
  param screen S "The screen's name."
  returns Option.ScreenProps.S
}
```

<sub>Module `ui` line 6176</sub>
