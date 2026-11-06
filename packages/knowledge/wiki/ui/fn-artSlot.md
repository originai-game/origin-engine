<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-artSlot"></a>

### `og.ui.artSlot`

An image or art slot: the texture fitted and cut as its desc says, fading in when it arrives (the style's enter motion; at once under reduced motion); until then a placeholder drawn from the style (a gradient of its soft fills and the kind's silhouette, a passing sheen while loading), never a grey box. Parts: `Art.frame[placeholder][loading]` (a style dresses the frame by state), `Art.placeholder` (the kind's silhouette and sheen), `Art.image`. Role image when labelled. Probe: `<id>`, `<id>/Image` once loaded, `<id>/Placeholder` before.

```ts
og.ui.artSlot(id: string, desc?: ArtSlotDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ArtSlotDesc` | yes | The art, its slot and its placeholder. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn artSlot {
  doc "An image or art slot: the texture fitted and cut as its desc says, fading in when it arrives (the style's enter motion; at once under reduced motion); until then a placeholder drawn from the style (a gradient of its soft fills and the kind's silhouette, a passing sheen while loading), never a grey box. Parts: `Art.frame[placeholder][loading]` (a style dresses the frame by state), `Art.placeholder` (the kind's silhouette and sheen), `Art.image`. Role image when labelled. Probe: `<id>`, `<id>/Image` once loaded, `<id>/Placeholder` before."
  host false
  param id String "Its id."
  param desc ArtSlotDesc "The art, its slot and its placeholder."
  returns Void
}
```

<sub>`api.ogt` line 19048</sub>
