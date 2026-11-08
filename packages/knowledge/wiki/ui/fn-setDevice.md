<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setDevice"></a>

### `og.ui.setDevice`

Set the device (the host's when it knows: a pad's family from its id); a pad family also becomes what pad input shows from then on.

```ts
og.ui.setDevice(world: World, device: UiDevice): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `device` | `UiDevice` | no | The device. |

**Returns** `void`.

Binding symbol `og_ui_set_device` · publish builds: kept

```ogt
fn setDevice {
  doc "Set the device (the host's when it knows: a pad's family from its id); a pad family also becomes what pad input shows from then on."
  param world World "The world."
  param device UiDevice "The device."
  returns Void
}
```

<sub>Module `ui` line 8222</sub>
