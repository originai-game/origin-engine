<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-KeyEase"></a>

### `KeyEase` (union)

How the segment into a key moves.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "linear"` | [`LinearKey`](../ui/type-LinearKey.md#type-LinearKey) | Constant speed. |
| `kind: "ease"` | [`EaseKey`](../ui/type-EaseKey.md#type-EaseKey) | A named ease. |
| `kind: "bezier"` | [`BezierKey`](../ui/type-BezierKey.md#type-BezierKey) | A cubic Bézier. |
| `kind: "spring"` | [`SpringSpec`](../ui/type-SpringSpec.md#type-SpringSpec) | On a spring from the previous key's value (released at it). |
| `kind: "hold"` | [`HoldKey`](../ui/type-HoldKey.md#type-HoldKey) | Holds, then jumps. |

```ogt
union KeyEase {
  doc "How the segment into a key moves."
  tag kind
  case linear LinearKey "Constant speed."
  case ease EaseKey "A named ease."
  case bezier BezierKey "A cubic Bézier."
  case spring SpringSpec "On a spring from the previous key's value (released at it)."
  case hold HoldKey "Holds, then jumps."
}
```

<sub>`api.ogt` line 20717</sub>
