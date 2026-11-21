<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiSeeds"></a>

### `UiSeeds` (desc struct)

The colours a UI palette is derived from (packages/design/tokens/seed.css): every ground, ink, accent, state, status and team colour follows them, each scheme keeping its text and team inks at 4.5:1, its focus ring and rims at 3:1 and teams 1 to 6 apart under every colour vision whatever they are. Each a linear RGB colour (`color.fromHex(0xc8102e)` for a hex or CSS colour); a seed left out or null takes the style's.

Also known as: seed colour, brand colour, accent colour, palette, retint, faction colour, primary colour

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `primary` | `Color \| null` | `null` | — | — | — | The brand: the primary fill, its hover and pressed tones, its soft wash and the ink on it. |
| `accent` | `Color \| null` | `null` | — | — | — | The second accent: the highlight ink (emphasised text, a badge, the current tab), the accent role and the focus ring. The style's, which by default follows primary. |
| `neutral` | `Color \| null` | `null` | — | — | — | The tint of the grounds, surfaces, lines, fills and inks: its hue, its chroma up to 0.04 (OKLCH). The style's, which by default is primary's hue at chroma up to 0.012. |
| `danger` | `Color \| null` | `null` | — | — | — | The danger hue (errors, damage, a destructive button). |
| `warn` | `Color \| null` | `null` | — | — | — | The warning hue. |
| `success` | `Color \| null` | `null` | — | — | — | The success hue (the ok state, healing). |
| `info` | `Color \| null` | `null` | — | — | — | The information hue. |
| `team` | `Color \| null` | `null` | — | — | — | Picks the first team (`--team1` to `--team8`): of six colours every colour vision tells apart (vermilion, yellow, bluish green, blue, violet, reddish purple), the one nearest its hue; teams 2 to 6 follow in Okabe-Ito order, 7 and 8 are fixed. |

```ogt
struct UiSeeds {
  doc "The colours a UI palette is derived from (packages/design/tokens/seed.css): every ground, ink, accent, state, status and team colour follows them, each scheme keeping its text and team inks at 4.5:1, its focus ring and rims at 3:1 and teams 1 to 6 apart under every colour vision whatever they are. Each a linear RGB colour (`color.fromHex(0xc8102e)` for a hex or CSS colour); a seed left out or null takes the style's."
  aliases "seed colour, brand colour, accent colour, palette, retint, faction colour, primary colour"
  desc true
  field primary Option.Color "The brand: the primary fill, its hover and pressed tones, its soft wash and the ink on it."
  field accent Option.Color "The second accent: the highlight ink (emphasised text, a badge, the current tab), the accent role and the focus ring. The style's, which by default follows primary."
  field neutral Option.Color "The tint of the grounds, surfaces, lines, fills and inks: its hue, its chroma up to 0.04 (OKLCH). The style's, which by default is primary's hue at chroma up to 0.012."
  field danger Option.Color "The danger hue (errors, damage, a destructive button)."
  field warn Option.Color "The warning hue."
  field success Option.Color "The success hue (the ok state, healing)."
  field info Option.Color "The information hue."
  field team Option.Color "Picks the first team (`--team1` to `--team8`): of six colours every colour vision tells apart (vermilion, yellow, bluish green, blue, violet, reddish purple), the one nearest its hue; teams 2 to 6 follow in Okabe-Ito order, 7 and 8 are fixed."
}
```

<sub>Module `ui` line 218</sub>
