<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-dismissToast"></a>

### `og.ui.dismissToast`

Start a toast's exit.

```ts
og.ui.dismissToast(world: World, toastId: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `toastId` | `number` | no | The id toast() answered. |

**Returns** `void`.

Binding symbol `og_ui_dismiss_toast` · publish builds: kept

```ogt
fn dismissToast {
  doc "Start a toast's exit."
  param world World "The world."
  param toastId u32 "The id toast() answered."
  returns Void
}
```

<sub>Module `ui` line 1090</sub>
