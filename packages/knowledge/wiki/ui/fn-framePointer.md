<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-framePointer"></a>

### `og.ui.framePointer`

The primary pointer of the frame being built, without the world: for a component that hit-tests a shape it draws itself (a pie's wedges, a wheel's slices) by angle and radius rather than by rectangles. Keys and pads reach the same choices through focusable parts.

```ts
og.ui.framePointer(): UiPointer
```

**Returns** `UiPointer`.

Binding symbol `og_ui_frame_pointer` · publish builds: kept

```ogt
fn framePointer {
  doc "The primary pointer of the frame being built, without the world: for a component that hit-tests a shape it draws itself (a pie's wedges, a wheel's slices) by angle and radius rather than by rectangles. Keys and pads reach the same choices through focusable parts."
  returns UiPointer
}
```

<sub>Module `ui` line 1173</sub>
