<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-carousel"></a>

### `og.ui.carousel`

Pages a drag or a flick moves, snapping to a page, with arrows and dots.

```ts
og.ui.carousel(id: string, desc: CarouselDesc, build: UiItemBuild): CarouselResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `CarouselDesc` | no | Count, the current page and options. |
| `build` | `UiItemBuild` | no | Builds a page by its index (the current one and two each side). |

**Returns** `CarouselResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn carousel {
  doc "Pages a drag or a flick moves, snapping to a page, with arrows and dots."
  host false
  param id String "Its id."
  param desc CarouselDesc "Count, the current page and options."
  param build UiItemBuild "Builds a page by its index (the current one and two each side)."
  returns CarouselResult
}
```

<sub>Module `ui` line 2926</sub>
