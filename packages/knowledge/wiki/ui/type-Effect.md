<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Effect"></a>

### `Effect` (union)

An effect of the element.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "dropShadow"` | [`ShadowEffect`](../ui/type-ShadowEffect.md#type-ShadowEffect) | A drop shadow (several layer). |
| `kind: "innerShadow"` | [`ShadowEffect`](../ui/type-ShadowEffect.md#type-ShadowEffect) | An inner shadow. |
| `kind: "outerGlow"` | [`GlowEffect`](../ui/type-GlowEffect.md#type-GlowEffect) | A glow around it. |
| `kind: "innerGlow"` | [`GlowEffect`](../ui/type-GlowEffect.md#type-GlowEffect) | A glow along its inside. |
| `kind: "layerBlur"` | [`BlurEffect`](../ui/type-BlurEffect.md#type-BlurEffect) | Its subtree blurred. |
| `kind: "backdrop"` | [`BackdropEffect`](../ui/type-BackdropEffect.md#type-BackdropEffect) | Glass: what is under it blurred. |

```ogt
union Effect {
  doc "An effect of the element."
  tag kind
  case dropShadow ShadowEffect "A drop shadow (several layer)."
  case innerShadow ShadowEffect "An inner shadow."
  case outerGlow GlowEffect "A glow around it."
  case innerGlow GlowEffect "A glow along its inside."
  case layerBlur BlurEffect "Its subtree blurred."
  case backdrop BackdropEffect "Glass: what is under it blurred."
}
```

<sub>Module `ui` line 7230</sub>
