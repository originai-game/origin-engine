<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-replace"></a>

### `og.ui.replace`

Replace the active layer with a screen.

```ts
og.ui.replace<S extends ScreenName>(world: World, screen: S, vm?: ScreenProps<S> | null): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `screen` | `S` | no | The screen's name. |
| `vm` | `ScreenProps<S> \| null` | yes | Its viewmodel. |

**Returns** `Result<number>`: The new layer's serial.

Binding symbol `og_ui_replace` · publish builds: kept

```ogt
fn replace {
  doc "Replace the active layer with a screen."
  generic S ScreenName
  param world World "The world."
  param screen S "The screen's name."
  param vm Option.ScreenProps.S "Its viewmodel." {
    optional true
  }
  returns Result.u32 "The new layer's serial."
}
```

<sub>Module `ui` line 8121</sub>
