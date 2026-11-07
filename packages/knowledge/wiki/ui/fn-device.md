<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-device"></a>

### `og.ui.device`

The device the player used last (glyphs follow it live): a key or the mouse, a pad of the family set with og.ui.setDevice (Xbox by default), or a touch on a coarse pointer.

```ts
og.ui.device(world: World): UiDevice
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `UiDevice`.

Binding symbol `og_ui_device` · publish builds: kept

```ogt
fn device {
  doc "The device the player used last (glyphs follow it live): a key or the mouse, a pad of the family set with og.ui.setDevice (Xbox by default), or a touch on a coarse pointer."
  param world World "The world."
  returns UiDevice
}
```

<sub>Module `ui` line 8118</sub>
