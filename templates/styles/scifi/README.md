# scifi: style pack

A science-fiction instrument dress: dark glass over the scene, thin cyan
rims, chamfered frames with corner brackets, condensed technical type, a
lit cyan primary and an amber accent for what is chosen, tracked or
focused. Warning is yellow and danger red. It has a dark scheme and a
light one (`og.ui.configure(world, { scheme: "light" })`). Load it with
`og.load.style(world, "@styles/scifi")`, or name it in a look
(`ui "scifi"`).

The pack is one file, `pack.json`, over the engine's default pack
(crates/og-ui-next/CONVENTIONS.md §Theme and packs): what it does not set
keeps the default's value. A built-in component asks for its slot's dress
with `packClass`, and the pack answers with classes; it selects no
component part. Review every component under it with
`bun packages/engine/src/node/gallery.ts --packs scifi --schemes dark,light`.

| File | What it holds |
| --- | --- |
| `pack.json` | The seeds, the theme's own colours, radii and font families, the faces, the nine-slice art, the slots' classes, the sounds and the icons |
| `art/` | The art `pack.json`'s `images` and `icons` name, each with its import `.ogt` (colour art sRGB; masks linear). Art no entry names yet waits for the slots still to be dressed |

## What it sets

- **`seeds`** (`accent`, `danger`, `info`, `neutral`, `primary`, `success`,
  `team`, `warn`): the palette is derived from them in both schemes at the
  contrast its pairs need, so `og.ui.configure`'s `seeds` retints the pack.
- **`theme`**: the 16 colours it writes over the derived palette, 5 radii
  and the font families (Scifi Display, Scifi Figure, Scifi Label, Scifi
  Text).
- **`fonts`**: each family's faces with their weight ranges, its CJK face
  last with a `unicodeRange`.
- **`images`**: nine-slice art by name (`frame-brackets`, `cut-frame`,
  `cut-plate`, `cut-cell`, `cut-chip`, `cut-button`, `cut-field`, `button`,
  `button-hover`, `button-pressed`, `button-disabled`, `button-primary`,
  `button-primary-hover`, `button-primary-pressed`, `panel`, `plate`,
  `chip`); a class `nine-<name>` draws one.
- **`variants`**: the classes of the slots it dresses (`Button.plate`,
  `Panel.frame`, `Plate.frame`, `Slot.frame`, `Chip.frame`, `Field.frame`,
  `Button.plate.primary`).
- **`sounds`**: `back`, `cancel`, `close`, `confirm`, `error`, `open`,
  `press`, `reward`, `select` for the whole pack, and its own for
  `Button.plate`, `Button.plate.primary`, from the shared set
  (`@styles/shared/sfx-*`).
- **`icons`**: 2 of the default set's names drawn by its own art.

## Art provenance

Every SVG in `art/` was drawn by hand for this pack; there are no generated
textures. Fonts (Chakra Petch, Barlow Condensed, Barlow Semi Condensed,
Rajdhani) come from `@styles/shared` under their own licences there.
