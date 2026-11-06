<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`art`](../art.md) · [index](../index.md)

<a id="record-ArtPalette"></a>

### `og.ArtPalette` (resource)

The style's colours: the palette the assets' texels sit on, how many a single asset may use, how many shades a hue ramps through, which colours are kept for signals, and how two colours mix where a tone falls between them. The frame's own quantise is Stylise.palette; this is what the assets are made of, 2D and 3D alike. Every field is no constraint at its zero value.

Also known as: colour palette, color palette, limited palette, palette lock, colour scheme, swatches

Token `og.ArtPalette`, registered under the schema name `og.ArtPalette`; its value type is `ArtPalette`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `colors` | `Palette` | `0` | — | — | — | The palette (og.load.palette of a `.hex` or GIMP `.gpl` file; a look names one under assets/ or a built-in `@palettes/<name>`); 0 is none, any colour. |
| `strict` | `boolean` | `false` | — | — | — | Every opaque texel is exactly a colour of colors (indexed colour, as hand-placed pixel art); false lets texels between them (shading, filtering) and holds only the hues to the palette. |
| `maxColors` | `number` | `0` | 0 … 256 | — | — | The most distinct colours one asset (a sprite sheet, a texture, a model's colours) may use; 0 is no limit. |
| `rampSteps` | `number` | `0` | 0 … 32 | — | — | The most shades one hue ramps through from its shadow to its highlight (3 or 4 for 8-bit art, 4 to 6 for 16-bit); 0 is no limit. |
| `accents` | `readonly number[]` | `[]` | — | — | a list of at most 8 `u32` (`List.u32.8`): reads a frozen copy, writes a whole array | The colours of colors kept for signals (the player, danger, the goal, a pickup), by their index in the palette (0 its first colour): scenery and backgrounds stay off them. A look gives them on one line (`accents 3 7`); empty keeps none. |
| `dither` | `DitherPattern` | `"none"` | — | — | — | How an asset mixes the two nearest colours where a tone falls between them: none in flat bands (the zero value), or an ordered or noise pattern as Stylise.dither. |

```ogt
resource ArtPalette {
  doc "The style's colours: the palette the assets' texels sit on, how many a single asset may use, how many shades a hue ramps through, which colours are kept for signals, and how two colours mix where a tone falls between them. The frame's own quantise is Stylise.palette; this is what the assets are made of, 2D and 3D alike. Every field is no constraint at its zero value."
  aliases "colour palette, color palette, limited palette, palette lock, colour scheme, swatches"
  field colors Palette "The palette (og.load.palette of a `.hex` or GIMP `.gpl` file; a look names one under assets/ or a built-in `@palettes/<name>`); 0 is none, any colour."
  field strict bool "Every opaque texel is exactly a colour of colors (indexed colour, as hand-placed pixel art); false lets texels between them (shading, filtering) and holds only the hues to the palette."
  field maxColors u32 "The most distinct colours one asset (a sprite sheet, a texture, a model's colours) may use; 0 is no limit." {
    min 0
    max 256
  }
  field rampSteps u32 "The most shades one hue ramps through from its shadow to its highlight (3 or 4 for 8-bit art, 4 to 6 for 16-bit); 0 is no limit." {
    min 0
    max 32
  }
  field accents List.u32.8 "The colours of colors kept for signals (the player, danger, the goal, a pickup), by their index in the palette (0 its first colour): scenery and backgrounds stay off them. A look gives them on one line (`accents 3 7`); empty keeps none."
  field dither DitherPattern "How an asset mixes the two nearest colours where a tone falls between them: none in flat bands (the zero value), or an ordered or noise pattern as Stylise.dither."
}
```

<sub>Module `art` line 105</sub>
