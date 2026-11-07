<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MetaKind"></a>

### `MetaKind` (enum)

What a meta layer draws over the scene.

A string: one of the variants below. The zero value is `"vignette"`.

| Variant | Doc |
|---|---|
| `"vignette"` | Its colour at the edges, clear in the middle (a damage vignette). |
| `"tint"` | A flat tint. |
| `"material"` | The frame under it composited through a UI material (a ripple for sanity, a glitch for a hit). |

```ogt
enum MetaKind {
  doc "What a meta layer draws over the scene."
  variant vignette "Its colour at the edges, clear in the middle (a damage vignette)."
  variant tint "A flat tint."
  variant material "The frame under it composited through a UI material (a ripple for sanity, a glitch for a hit)."
}
```

<sub>Module `ui` line 7938</sub>
