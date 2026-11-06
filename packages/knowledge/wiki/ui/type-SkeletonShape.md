<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SkeletonShape"></a>

### `SkeletonShape` (enum)

What a skeleton stands in for.

A string: one of the variants below. The zero value is `"lines"`.

| Variant | Doc |
|---|---|
| `"lines"` | Lines of text: the last one shorter. |
| `"card"` | A card: an art block, a title and two lines. |
| `"row"` | A list row: an avatar, a title and a line. |
| `"avatar"` | A round avatar. |
| `"block"` | A block of the given size. |

```ogt
enum SkeletonShape {
  doc "What a skeleton stands in for."
  variant lines "Lines of text: the last one shorter."
  variant card "A card: an art block, a title and two lines."
  variant row "A list row: an avatar, a title and a line."
  variant avatar "A round avatar."
  variant block "A block of the given size."
}
```

<sub>Module `ui` line 5690</sub>
