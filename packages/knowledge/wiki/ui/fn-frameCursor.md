<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-frameCursor"></a>

### `og.ui.frameCursor`

The pointer's shape the last frame resolved (never auto): the `cursor-*` class of the TSX element the pointer is over, or of its nearest ancestor that names one, else default; none while the virtual cursor shows. The web shell sets the canvas's CSS cursor from each frame's answer (`cursor`); a native host that draws its own window (og dev, Origin Desktop's shared frames) reads the same answer.

```ts
og.ui.frameCursor(): Cursor
```

**Returns** `Cursor`.

Binding symbol `og_ui_frame_cursor` · publish builds: kept

```ogt
fn frameCursor {
  doc "The pointer's shape the last frame resolved (never auto): the `cursor-*` class of the TSX element the pointer is over, or of its nearest ancestor that names one, else default; none while the virtual cursor shows. The web shell sets the canvas's CSS cursor from each frame's answer (`cursor`); a native host that draws its own window (og dev, Origin Desktop's shared frames) reads the same answer."
  returns Cursor
}
```

<sub>Module `ui` line 288</sub>
