<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-LayoutMode"></a>

### `LayoutMode` (enum)

How a container lays out its children.

A string: one of the variants below. The zero value is `"flex"`.

| Variant | Doc |
|---|---|
| `"flex"` | The row and column walks of its direction (the default). |
| `"stack"` | Children overlaid in the same rectangle, each placed by its insets, else by the container's align (across) and cross (down): a card's art, frame, cost gem and name. |
| `"grid"` | Tracks and cells (ElementProps.grid). |

```ogt
enum LayoutMode {
  doc "How a container lays out its children."
  variant flex "The row and column walks of its direction (the default)."
  variant stack "Children overlaid in the same rectangle, each placed by its insets, else by the container's align (across) and cross (down): a card's art, frame, cost gem and name."
  variant grid "Tracks and cells (ElementProps.grid)."
}
```

<sub>Module `ui` line 4440</sub>
