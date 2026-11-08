<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-frameDirection"></a>

### `og.ui.frameDirection`

The writing direction of the frame being built, `ltr` or `rtl` (og.i18n's locale sets it): code that draws its own arrows, chevrons or progress along a line mirrors them by it; layout mirrors rows and insets itself.

```ts
og.ui.frameDirection(): string
```

**Returns** `string`.

Binding symbol `og_ui_frame_direction` · publish builds: kept

```ogt
fn frameDirection {
  doc "The writing direction of the frame being built, `ltr` or `rtl` (og.i18n's locale sets it): code that draws its own arrows, chevrons or progress along a line mirrors them by it; layout mirrors rows and insets itself."
  returns String
}
```

<sub>Module `ui` line 1216</sub>
