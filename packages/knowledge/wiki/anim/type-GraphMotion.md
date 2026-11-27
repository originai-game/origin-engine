<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphMotion"></a>

### `GraphMotion` (union)

A named clip leaf or blend; the kind selects its fields.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "clip"` | [`GraphClip`](../anim/type-GraphClip.md#type-GraphClip) | A named clip leaf. |
| `kind: "blend1d"` | [`GraphBlend1d`](../anim/type-GraphBlend1d.md#type-GraphBlend1d) | Linear interpolation between threshold children. |
| `kind: "blend2d"` | [`GraphBlend2d`](../anim/type-GraphBlend2d.md#type-GraphBlend2d) | Gradient-band interpolation between points. |

```ogt
union GraphMotion {
  doc "A named clip leaf or blend; the kind selects its fields."
  tag kind
  case clip GraphClip "A named clip leaf."
  case blend1d GraphBlend1d "Linear interpolation between threshold children."
  case blend2d GraphBlend2d "Gradient-band interpolation between points."
}
```

<sub>Module `anim` line 671</sub>
