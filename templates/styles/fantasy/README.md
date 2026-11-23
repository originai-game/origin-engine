# fantasy: style pack

A dark-fantasy RPG dress: warm textured ground, brass rims with filigree
corners, painted button plates (crimson primary, dark iron secondary) and
serif type in letter-spaced small caps. Load it with
`og.load.style(world, "@styles/fantasy")`, or name it in a look
(`ui "fantasy"`).

The pack is one file, `pack.json`, over the engine's default pack
(crates/og-ui-next/CONVENTIONS.md §Theme and packs): what it does not set
keeps the default's value. A built-in component asks for its slot's dress
with `packClass`, and the pack answers with classes; it selects no
component part. Review every component under it with
`bun packages/engine/src/node/gallery.ts --packs fantasy --schemes dark,light`.

| File | What it holds |
| --- | --- |
| `pack.json` | The seeds, the theme's own colours, radii and font families, the faces, the nine-slice art, the slots' classes, the sounds and the icons |
| `art/` | The art `pack.json`'s `images` and `icons` name, each with its import `.ogt` (colour art sRGB; masks linear). Art no entry names yet waits for the slots still to be dressed |

## What it sets

- **`seeds`** (`accent`, `danger`, `info`, `neutral`, `primary`, `success`,
  `team`, `warn`): the palette is derived from them in both schemes at the
  contrast its pairs need, so `og.ui.configure`'s `seeds` retints the pack.
- **`theme`**: the 30 colours it writes over the derived palette, 5 radii
  and the font families (Fantasy Caps, Fantasy Display, Fantasy Serif,
  Fantasy Text).
- **`fonts`**: each family's faces with their weight ranges, its CJK face
  last with a `unicodeRange`.
- **`images`**: nine-slice art by name (`frame-filigree`, `button-iron`,
  `button-iron-hover`, `button-iron-pressed`, `button-red`,
  `button-red-hover`, `button-red-pressed`, `button-disabled`, `plate`,
  `chip`, and `button-primary`, `button-primary-hover`,
  `button-primary-pressed` as the red ones, so the primary art and its
  `art-primary-fg` ink agree); a class `nine-<name>` draws one.
- **`variants`**: the classes of the slots it dresses (`Button.plate`,
  `Button.plate.primary`, `Panel.frame`, `Plate.frame`, `Chip.frame`).
- **`sounds`**: every event (`press`, `hover`, `select`, `confirm`, `cancel`, `back`, `open`, `close`, `error`, `reward`, `toggle`, `tab`, `levelup`, `purchase`, `countdown`) from the pack's own clips in `sfx/` (`tap`, `toggle`, `tab`, `countdown`, `back`, `close`, `open`, `confirm`, `error`, `purchase`, `reward`, `levelup`; `.ogg`, mastered by `templates/tools/master-ui-sounds.py`), and a hover for `Button.plate`, `Button.plate.primary`.
- **`icons`**: 3 of the default set's names drawn by its own art.
- **Icon tones** (`icon-*` colours in `theme`): the engine's game icon set
  (`@styles/shared/icons/`, its names in that README) in antique gold and deep enamels, an umber rim and a heavier drop; the
  geometry is the default pack's.

## Art provenance

The SVGs were drawn by hand for this pack. `ground.png` and
`filigree-{tl,tr,bl,br}.png` were generated for it with gpt-image-2.6
through the OriginGame gateway (2026-11) and post-processed (cropped,
mirrored, keyed to alpha); `glow.png` is a computed soft radial falloff.
Fonts come from `@styles/shared` under their own licences there.
