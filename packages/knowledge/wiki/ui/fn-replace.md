<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-replace"></a>

### `og.ui.replace`

Replace the active layer with a screen.

```ts
og.ui.replace(world: World, screen: string, vm?: unknown): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `screen` | `string` | no | The screen's name. |
| `vm` | `unknown` | yes | Its viewmodel. |

**Returns** `Result<number>`: The new layer's serial.

Binding symbol `og_ui_replace` · publish builds: kept

```ogt
fn replace {
  doc "Replace the active layer with a screen."
  param world World "The world."
  param screen String "The screen's name."
  param vm Json "Its viewmodel." {
    optional true
  }
  returns Result.u32 "The new layer's serial."
}
```

<sub>Module `ui` line 8157</sub>
