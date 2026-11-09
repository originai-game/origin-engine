# CSS pack loading contract

A pack is a directory of `.css` files and assets. Its name is the directory
basename. It has no `extension.json` or `style.ogt`. For example,
`assets/styles/example/main.css` can reference `images/frame.svg`,
`fonts/body.ttf`, and `sounds/press.wav` beside it.

`og.load.style(engine, "styles/example")` loads the cooked directory.
`og.ui.configure(engine, { style })` selects the returned handle. The Rust
loader uses `StylePack::parse(directory, sources)` and `Ui::set_style_pack`.
Direct-child CSS files apply in lexical filename order. Nested sheets apply
through `@import`. Imports and asset URLs resolve relative to the file that
contains them, not the pack entry file. A leading slash names an asset-root
path, including `/@styles/shared/` and `/@styles/cjk/`.

## Cascade order

Normal declarations increase in priority as follows:
`og-default < og-kit < og-pack < og-game < unlayered`.
CSS important declarations reverse layer priority. Sublayers stay below their
parent's normal declarations, even when sheets merge after parsing.

`crates/og-ui/default.css` declares `og-default`. A document whose asset ID
starts with `ui-kits/` places its `<style>` blocks and imports in `og-kit`.
The pack loader wraps its imports in `og-pack`. Game styles can declare
`og-game`; unlayered game styles retain standard CSS precedence.

Pack CSS applies to authored document elements and exposed `::part()` rules.
**Core widget host rules and inheritance land with parts-as-nodes.** Phase B
does not claim those immediate-mode hosts already participate in inheritance.

## Former metadata

- Tokens use `:root` custom properties and `var()`.
- Fonts use `@font-face` and `font-family`.
- Advanced motion uses `transition`, `@keyframes`, easing, and media queries,
  including `prefers-reduced-motion` and viewport breakpoints.
- Shape and elevation use borders, radii, shadows, and the existing material
  property. Rarity and colour-blind decoration use host `data-*` selectors.
  The semantic icon cue migration belongs to the audit stream.
- Icons use `mask-image`. Ornaments and former atlas slices use CSS images,
  backgrounds, and `border-image`. SVG files cook through resvg into ordinary
  textures. There is no author-visible atlas or recolouring format.
- Haptics remain in `og.*`.

The only new extension is `-og-sound`: CSS has no interaction-audio property.
Its registered grammar is
`none | [ <event> [ <url>+ [volume(<number> <number>?)]? [pitch(<number> <number>?)]? | none ] ]#`.
For example: `-og-sound: press url(a.wav) url(b.wav) volume(0.28 0.34)
pitch(0.98 1.02), hover none`. An event accepts up to four samples, preserving
the previous sound-set capacity. Volume is an ordered range within 0..1;
pitch is an ordered positive range. One number makes a constant range.
The presentation random stream is seeded from the engine and checkpointed
for replay, including non-repeating sample selection.

## Phase C boundary

Phase C owns `og style new/upgrade/check/art`, `og ui sheet/review`, user docs
and guides, and the CSS-only test pack. Removed-format commands refuse the
legacy format rather than generate another `style.ogt`. The general art
contract checker remains available. Shared fonts, CJK fonts, and sounds
remain assets; sounds formerly under `clean/` now live under `shared/`.
Internal `Art`, `Slice`, `Ornament`, `StylePart`, and colour/ramp readers stay
exported until their widget consumers migrate.
