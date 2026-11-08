<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TransitionKind"></a>

### `TransitionKind` (enum)

How a transition moves its content in and out. Under reduced motion every kind only fades.

A string: one of the variants below. The zero value is `"fade"`.

| Variant | Doc |
|---|---|
| `"fade"` | Opacity only. |
| `"slideUp"` | Rises into place from below. |
| `"slideDown"` | Drops into place from above. |
| `"slideLeft"` | Enters from the right. |
| `"slideRight"` | Enters from the left. |
| `"scale"` | Grows from slightly smaller around its centre. |
| `"pop"` | Springs up from smaller around its centre, overshoots and settles. |
| `"glitch"` | Tears in: the content draws through the glitch material (slices pushed sideways, colours split) as it fades in or out, over 150 to 180 ms whatever the style's base duration; as deterministic as every frame (from the element's id and the tick-counted UI clock). A style sets its strength (`enter glitch 0.8`). |

```ogt
enum TransitionKind {
  doc "How a transition moves its content in and out. Under reduced motion every kind only fades."
  variant fade "Opacity only."
  variant slideUp "Rises into place from below."
  variant slideDown "Drops into place from above."
  variant slideLeft "Enters from the right."
  variant slideRight "Enters from the left."
  variant scale "Grows from slightly smaller around its centre."
  variant pop "Springs up from smaller around its centre, overshoots and settles."
  variant glitch "Tears in: the content draws through the glitch material (slices pushed sideways, colours split) as it fades in or out, over 150 to 180 ms whatever the style's base duration; as deterministic as every frame (from the element's id and the tick-counted UI clock). A style sets its strength (`enter glitch 0.8`)."
}
```

<sub>Module `ui` line 725</sub>
