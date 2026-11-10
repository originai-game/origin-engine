<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-Light2DKind"></a>

### `Light2DKind` (enum)

What shape of light a Light2D is.

A string: one of the variants below. The zero value is `"point"`.

| Variant | Doc |
|---|---|
| `"point"` | Light all around its position, to radius. |
| `"spot"` | A cone along the entity's local +x (turned with its Transform about z), innerAngle to outerAngle either side of the axis, to radius. |
| `"global"` | Light on every lit pixel of its layers at full intensity, from the direction of the entity's local +x at height (for normal maps); no radius and no shadows. |

```ogt
enum Light2DKind {
  doc "What shape of light a Light2D is."
  variant point "Light all around its position, to radius."
  variant spot "A cone along the entity's local +x (turned with its Transform about z), innerAngle to outerAngle either side of the axis, to radius."
  variant global "Light on every lit pixel of its layers at full intensity, from the direction of the entity's local +x at height (for normal maps); no radius and no shadows."
}
```

<sub>Module `render` line 1858</sub>
