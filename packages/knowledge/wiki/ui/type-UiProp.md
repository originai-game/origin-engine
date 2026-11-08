<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiProp"></a>

### `UiProp` (enum)

An animatable value of an element (docs/ui.md §3.1): a state, a timeline key, a presence or og.ui.animate moves it. Colours are sRGB components 0 to 1; rotation and skew are radians.

A string: one of the variants below. The zero value is `"x"`.

| Variant | Doc |
|---|---|
| `"x"` | Its horizontal offset, CSS px (the transform's x). |
| `"y"` | Its vertical offset, CSS px. |
| `"scaleX"` | Its horizontal scale (1: unscaled). |
| `"scaleY"` | Its vertical scale. |
| `"scale"` | Both scales at once. |
| `"rotate"` | Its turn, radians, clockwise on screen. |
| `"skewX"` | Its horizontal skew, radians. |
| `"skewY"` | Its vertical skew, radians. |
| `"opacity"` | Its opacity, 0 to 1, multiplied down the tree. |
| `"radius"` | Its corner radius, CSS px. |
| `"backgroundR"` | Its fill's red. |
| `"backgroundG"` | Its fill's green. |
| `"backgroundB"` | Its fill's blue. |
| `"backgroundA"` | Its fill's alpha. |
| `"inkR"` | Its text's red. |
| `"inkG"` | Its text's green. |
| `"inkB"` | Its text's blue. |
| `"inkA"` | Its text's alpha. |
| `"z"` | Its z among its siblings (stepped, never tweened). |
| `"inert"` | 1: it and its subtree take no input (stepped): a results screen's buttons wait for the sequence. |

```ogt
enum UiProp {
  doc "An animatable value of an element (docs/ui.md §3.1): a state, a timeline key, a presence or og.ui.animate moves it. Colours are sRGB components 0 to 1; rotation and skew are radians."
  variant x "Its horizontal offset, CSS px (the transform's x)."
  variant y "Its vertical offset, CSS px."
  variant scaleX "Its horizontal scale (1: unscaled)."
  variant scaleY "Its vertical scale."
  variant scale "Both scales at once."
  variant rotate "Its turn, radians, clockwise on screen."
  variant skewX "Its horizontal skew, radians."
  variant skewY "Its vertical skew, radians."
  variant opacity "Its opacity, 0 to 1, multiplied down the tree."
  variant radius "Its corner radius, CSS px."
  variant backgroundR "Its fill's red."
  variant backgroundG "Its fill's green."
  variant backgroundB "Its fill's blue."
  variant backgroundA "Its fill's alpha."
  variant inkR "Its text's red."
  variant inkG "Its text's green."
  variant inkB "Its text's blue."
  variant inkA "Its text's alpha."
  variant z "Its z among its siblings (stepped, never tweened)."
  variant inert "1: it and its subtree take no input (stepped): a results screen's buttons wait for the sequence."
}
```

<sub>Module `ui` line 2174</sub>
