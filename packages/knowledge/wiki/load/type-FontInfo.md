<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-FontInfo"></a>

### `FontInfo` (struct)

A loaded font's metrics, in em units unless stated.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `family` | `string` | `""` | — | — | — | The family its file names, as a font-family list calls it; empty for an icon set. |
| `weight` | `number` | `0` | — | — | — | Its default weight (OS/2's weight class, 400 regular). |
| `axes` | `readonly string[]` | `[]` | — | — | — | Its variation axes' tags (wght, wdth, opsz …); empty for a static face. |
| `ascender` | `number` | `0` | — | — | — | Ascender above the baseline. |
| `descender` | `number` | `0` | — | — | — | Descender below the baseline (negative). |
| `lineHeight` | `number` | `0` | — | — | — | Ascender minus descender plus the line gap. |
| `icons` | `number` | `0` | — | — | — | The icons of an icon set (an `.ogt` of `type icons`, SVGs cooked like a font for UiDesc.icons.font); 0 for a text font. |

```ogt
struct FontInfo {
  doc "A loaded font's metrics, in em units unless stated."
  field family String "The family its file names, as a font-family list calls it; empty for an icon set."
  field weight f32 "Its default weight (OS/2's weight class, 400 regular)."
  field axes List.String "Its variation axes' tags (wght, wdth, opsz …); empty for a static face."
  field ascender f32 "Ascender above the baseline."
  field descender f32 "Descender below the baseline (negative)."
  field lineHeight f32 "Ascender minus descender plus the line gap."
  field icons u32 "The icons of an icon set (an `.ogt` of `type icons`, SVGs cooked like a font for UiDesc.icons.font); 0 for a text font."
}
```

<sub>Module `load` line 108</sub>
