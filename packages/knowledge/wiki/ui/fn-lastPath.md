<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-lastPath"></a>

### `og.ui.lastPath`

The full path of the element built just before (a container just closed, or a widget), as og.ui.bounds, og.ui.gesture, og.ui.forceState and the probe name it: a component reads its own parts' rectangles and gestures by it. Empty when nothing was built before it.

```ts
og.ui.lastPath(): string
```

**Returns** `string`.

Binding symbol `og_ui_last_path` · publish builds: kept

```ogt
fn lastPath {
  doc "The full path of the element built just before (a container just closed, or a widget), as og.ui.bounds, og.ui.gesture, og.ui.forceState and the probe name it: a component reads its own parts' rectangles and gestures by it. Empty when nothing was built before it."
  returns String
}
```

<sub>`api.ogt` line 16960</sub>
