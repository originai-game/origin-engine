# cartoon: style pack

A casual cartoon dress, in the spirit of Brawl Stars, Clash Royale and Fall
Guys: bright candy colours on a sky ground, one thick dark outline around
every control, frame and tile, chunky rounded buttons that stand on a
raised bottom edge and sink onto it when pressed, bold rounded display
type and soft drop shadows. Load it with
`og.load.style(world, "@styles/cartoon")`, or name it in a look
(`ui "cartoon"`). It has a dark scheme (indigo panels on a deep sky) and a
light one (cream panels on a day sky).

The pack is one file, `pack.json`, over the engine's default pack
(crates/og-ui-next/CONVENTIONS.md §Theme and packs): what it does not set
keeps the default's value. A built-in component asks for its slot's dress
with `packClass`, and the pack answers with classes; it selects no
component part. Review every component under it with
`bun packages/engine/src/node/gallery.ts --packs cartoon --schemes dark,light`.

| File | What it holds |
| --- | --- |
| `pack.json` | The seeds, the theme's own colours, radii and font families, the faces, the nine-slice art, the slots' classes, the sounds and the icons |
| `art/` | The art `pack.json`'s `images` and `icons` name, each with its import `.ogt` (colour art sRGB; masks linear). Art no entry names yet waits for the slots still to be dressed |

## What it sets

- **`seeds`** (`accent`, `danger`, `info`, `neutral`, `primary`, `success`,
  `team`, `warn`): the palette is derived from them in both schemes at the
  contrast its pairs need, so `og.ui.configure`'s `seeds` retints the pack.
- **`theme`**: the 44 colours it writes over the derived palette, 7 radii
  and the font families (Cartoon Display, Cartoon Fallback, Cartoon Label,
  Cartoon Text).
- **`fonts`**: each family's faces with their weight ranges, its CJK face
  last with a `unicodeRange`.
- **`images`**: nine-slice art by name (`button`, `button-hover`,
  `button-pressed`, `button-disabled`, `button-primary`,
  `button-primary-hover`, `button-primary-pressed`, `panel`, `plate`,
  `chip`); a class `nine-<name>` draws one.
- **`variants`**: the classes of the slots it dresses (`Button.plate`,
  `Button.plate.primary`, `Panel.frame`, `Plate.frame`, `Chip.frame`).
- **`sounds`**: every event (`press`, `hover`, `select`, `confirm`, `cancel`, `back`, `open`, `close`, `error`, `reward`, `toggle`, `tab`, `levelup`, `purchase`, `countdown`) from the pack's own clips in `sfx/` (`tap`, `toggle`, `tab`, `countdown`, `back`, `close`, `open`, `confirm`, `error`, `purchase`, `reward`, `levelup`; `.ogg`, mastered by `templates/tools/master-ui-sounds.py`), and a hover for `Button.plate`, `Button.plate.primary`, `Slot.frame`.
- **`icons`**: 5 of the default set's names drawn by its own art.
- **Icon tones** (`icon-*` colours in `theme`): the engine's game icon set
  (`@styles/shared/icons/`, its names in that README) in candy colours, the pack's navy ink as the rim and a hard navy drop; the
  geometry is the default pack's.

## Fonts and art

- Lilita One (Juan Montoreano, 2011), SIL Open Font License 1.1
  (`OFL-LilitaOne.txt`): display, buttons and figures. The file is the
  release from Google Fonts (`ofl/lilitaone`, version 1.002), unmodified.
- Fredoka (The Fredoka Project Authors, 2016), SIL Open Font License 1.1
  (`OFL-Fredoka.txt`): text and labels. `Fredoka[wdth,wght].ttf` is the
  variable font from Google Fonts (`ofl/fredoka`), unmodified, which
  pack.json names for wght 300–700. Fredoka declares no reserved font
  name.
- Arrows and any other glyph the two faces lack come from Instrument Sans
  (`@styles/shared`, the last family in every font list) and Chinese from ZCOOL KuaiLe for display and Noto Sans SC
  (variable, wght 100–900) for text and labels (`@styles/cjk`), each under
  its own licence there.
- No face here carries Arabic, so the `ar` locale warns of missing glyphs
  under this pack; a game that ships Arabic adds a face for it.
- The SVGs in `art/` were drawn for this pack; nothing in it is generated.
- Its sounds (`sfx/`) were generated for this pack; see `../shared/LICENSES.md`.
