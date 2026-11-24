# UI themes

How a game's UI gets its look: the engine's themes, picking one through the
look, deriving the game's own from one in a few lines, replacing a piece of
art by hand, and what the cook guarantees about contrast. Screens, slots and
review are in the screens guide.

## 1. What a theme is

A theme is a recipe, `theme.json`: one value for each dimension of a look
and the colours it starts from. The cook compiles it into the style pack
the built-in components read, and draws the theme's nine-slice art
(buttons with their hover, pressed and disabled states, panels, plates,
chips, slots, fields) from its material, shape and ornament. Nothing is
drawn by hand and nothing is copied into the game.

| Key | What it picks | Values |
| --- | --- | --- |
| `color` | `source`, `accent`, `neutral`, `danger` (hex), `variant`, `scheme` (`light`, `dark`) | any |
| `material` | the faces' surface | `cloth`, `flat`, `glass`, `ink`, `matte`, `metal`, `paper`, `pixel`, `plastic`, `stone`, `wood` |
| `shape` | the outline | `brush`, `cut`, `cut4`, `pixel`, `round`, `sharp`, `soft` |
| `type` | display, text and figure faces | `classic-serif`, `condensed-stencil`, `condensed-tech`, `pixel`, `rounded-display`, `sans`, `song`, `storybook-serif` |
| `motion` | how screens, presses and lists move | `bouncy`, `calm`, `snappy`, `standard`, `stepped` |
| `ornament` | corner marks, one value or one per piece (`button`, `panel`, `plate`, `slot`, `chip`) | `bar`, `brackets`, `brush-corner`, `filigree`, `hairline`, `none`, `notch`, `runes`, `stitch`, `studs`, `tab` |
| `effects` | glows, sparks, text halos | `dust`, `glow`, `ink-splash`, `pixel-burst`, `sheen`, `spark` |
| `sound` | the UI's taps, opens and rewards | `arcade`, `chiptune`, `cute`, `glass`, `mechanical`, `none`, `soft`, `wood` |
| `icons` | the icon set's drawing | `glow`, `ink`, `pixel`, `plate`, `sticker` |
| `density` | spacing and target size | `comfortable`, `compact`, `touch` |

## 2. The engine's themes

| Theme | Material, shape, type, motion | Fits |
| --- | --- | --- |
| `anime` | glass, soft, sans, standard | anime RPG, gacha, action RPG, open world |
| `candy` | plastic, round, rounded display, bouncy | match-3, idle, hyper-casual, merge |
| `cartoon` | flat with a header band, round, rounded display, bouncy | party, kart, family, casual action |
| `fantasy` | wood, round, classic serif, standard | CRPG, MMO, dungeon, fantasy cards |
| `ink` | ink, brush, Song, calm | wuxia, xianxia, guofeng |
| `minimal` | matte, soft, sans, calm | puzzle, sim, realism, tools |
| `pixel` | pixel, pixel, pixel, stepped | retro platformer, roguelike, farming |
| `scifi` | glass, four cut corners, condensed tech, snappy | space, sci-fi shooter, cyber, racing |
| `storybook` | paper, soft, storybook serif, calm | narrative, visual novel, cozy |
| `tactical` | metal, cut, stencil, snappy | military, survival, battle royale |

`tactical-arctic` is `tactical` in an ice palette, an example of a derived
theme. A look preset names its theme in its `ui` line (`toy-3d` names
`candy`); the game's look keeps it unless it names another.

## 3. Picking and deriving

1. Pick the engine theme nearest the game (§2) and name it in the game's
   look: `ui "fantasy"`.
2. For the game's own, write `assets/styles/<name>/theme.json` and name
   `<name>` in the look instead. Start from an engine theme and change
   only what differs:

```json
{
  "extends": "@themes/tactical",
  "color": { "source": "#5cc8ff", "accent": "#bfe9ff", "neutral": "#5d6b78" },
  "ornament": { "panel": "hairline" }
}
```

3. Change two dimensions or more (a material and a shape, a type and a
   motion) for a look of its own; a theme that differs only in colour is
   a derived one.

`extends` reads the parent whole, then the recipe's own keys lie over it:
an object merges key by key, anything else is replaced. An unknown key, a
dimension value with no file, or an `extends` loop fails the cook and
names the file.

## 4. Hand-drawn art

`pack` is a pack.json written as one, laid over everything the recipe
made. A hand-drawn or generated piece replaces the drawn one of the same
name, and every other piece still comes from the generator, so the theme
is never missing one:

```json
{
  "extends": "@themes/fantasy",
  "pack": { "images": { "panel": { "src": "art/panel-hand.png", "slice": 24 } } }
}
```

The image's slice is its edge: text sits inside it, on the stretched
centre. A piece's centre is the ground the text reads on (§5).

## 5. What the cook guarantees

- Every ink role, and each art slot's own ink, holds 4.5:1 (3:1 for a
  large figure's ink such as `gold-hi`) on every ground of its scheme:
  the design's surfaces and each piece of art as drawn, its centre's
  darkest and lightest tenths, so a gloss, a grain or a header band
  counts. Only the inks' lightness moves; hue and chroma stay the
  theme's.
- Each fill's own ink (`accent-fg` on `accent`) holds 4.5:1 on it.
- High contrast (`prefers-contrast: more`) holds the same inks at 7:1
  (4.5:1 for the large figures).
- A colour the game pins in `pack` is laid as written and is not moved;
  `og check` reports text that then misses its ratio (OG9107).

## 6. Review

Capture the game's screens with `og ui preview` at the platform's
breakpoints in both schemes and look at every frame (screens guide §5).
The theme is chosen when the game is made and stays: a game switches its
scheme and contrast at run time, not its theme.
