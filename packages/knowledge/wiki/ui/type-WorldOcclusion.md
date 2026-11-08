<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-WorldOcclusion"></a>

### `WorldOcclusion` (enum)

What the scene in front of an anchored element does to it: a depth test of its point against the frame's depth, on the GPU.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Drawn over the scene. |
| `"fade"` | Drawn at occludedAlpha where the scene is in front. |
| `"hide"` | Not drawn where the scene is in front. |

```ogt
enum WorldOcclusion {
  doc "What the scene in front of an anchored element does to it: a depth test of its point against the frame's depth, on the GPU."
  variant none "Drawn over the scene."
  variant fade "Drawn at occludedAlpha where the scene is in front."
  variant hide "Not drawn where the scene is in front."
}
```

<sub>Module `ui` line 2894</sub>
