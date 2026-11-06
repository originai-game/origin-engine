<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-AnchorScaleMode"></a>

### `AnchorScaleMode` (enum)

How an anchored element's size follows its distance from the camera.

A string: one of the variants below. The zero value is `"constant"`.

| Variant | Doc |
|---|---|
| `"constant"` | Its laid-out size at every distance (a nameplate that stays readable). |
| `"distance"` | 1 at reference world units, larger nearer and smaller farther, between minScale and maxScale (a tag that feels in the world). |

```ogt
enum AnchorScaleMode {
  doc "How an anchored element's size follows its distance from the camera."
  variant constant "Its laid-out size at every distance (a nameplate that stays readable)."
  variant distance "1 at reference world units, larger nearer and smaller farther, between minScale and maxScale (a tag that feels in the world)."
}
```

<sub>`api.ogt` line 23139</sub>
