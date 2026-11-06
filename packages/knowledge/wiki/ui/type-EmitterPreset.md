<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-EmitterPreset"></a>

### `EmitterPreset` (enum)

A particle look.

A string: one of the variants below. The zero value is `"sparkle"`.

| Variant | Doc |
|---|---|
| `"sparkle"` | Twinkling stars rising a little (the default): something rare, a reward. |
| `"confetti"` | Paper squares falling and turning. |
| `"embers"` | Sparks rising and fading: fire, heat, a forge. |
| `"snow"` | Flakes drifting down. |
| `"bubbles"` | Discs rising and swelling. |
| `"dust"` | Motes drifting slowly every way: ambience behind a menu. |

```ogt
enum EmitterPreset {
  doc "A particle look."
  variant sparkle "Twinkling stars rising a little (the default): something rare, a reward."
  variant confetti "Paper squares falling and turning."
  variant embers "Sparks rising and fading: fire, heat, a forge."
  variant snow "Flakes drifting down."
  variant bubbles "Discs rising and swelling."
  variant dust "Motes drifting slowly every way: ambience behind a menu."
}
```

<sub>`api.ogt` line 19229</sub>
