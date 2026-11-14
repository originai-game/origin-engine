<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-hudStep"></a>

### `og.ui.hudStep`

Steps the selected element's size and opacity: each -1, 0 or +1 step (a tenth larger or smaller, a tenth more or less opaque), within its limits.

```ts
og.ui.hudStep(scale: number, opacity: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `scale` | `number` | no | The size step. |
| `opacity` | `number` | no | The opacity step. |

**Returns** `void`.

Binding symbol `og_ui_hud_step` · publish builds: kept

```ogt
fn hudStep {
  doc "Steps the selected element's size and opacity: each -1, 0 or +1 step (a tenth larger or smaller, a tenth more or less opaque), within its limits."
  param scale i32 "The size step."
  param opacity i32 "The opacity step."
  returns Void
}
```

<sub>Module `ui` line 681</sub>
