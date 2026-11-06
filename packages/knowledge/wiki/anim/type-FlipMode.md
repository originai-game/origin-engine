<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-FlipMode"></a>

### `FlipMode` (enum)

How a flipbook clip runs past its last frame.

A string: one of the variants below. The zero value is `"loop"`.

| Variant | Doc |
|---|---|
| `"loop"` | Back to the first frame. |
| `"pingPong"` | Back and forth, without repeating the end frames. |
| `"once"` | Hold the last frame and set Flipbook.finished. |

```ogt
enum FlipMode {
  doc "How a flipbook clip runs past its last frame."
  variant loop "Back to the first frame."
  variant pingPong "Back and forth, without repeating the end frames."
  variant once "Hold the last frame and set Flipbook.finished."
}
```

<sub>`api.ogt` line 12085</sub>
