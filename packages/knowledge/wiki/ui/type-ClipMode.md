<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ClipMode"></a>

### `ClipMode` (enum)

How an element cuts its children.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Not at all (a scroll container still clips to its rectangle). |
| `"shape"` | To its shape, antialiased (a round avatar, a hex frame). |
| `"mask"` | By DrawDesc.mask's alpha (the subtree draws offscreen). |

```ogt
enum ClipMode {
  doc "How an element cuts its children."
  variant none "Not at all (a scroll container still clips to its rectangle)."
  variant shape "To its shape, antialiased (a round avatar, a hex frame)."
  variant mask "By DrawDesc.mask's alpha (the subtree draws offscreen)."
}
```

<sub>`api.ogt` line 23012</sub>
