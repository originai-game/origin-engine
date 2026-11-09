# Review the CSS laboratory pack

`assets/styles/css-laboratory` contains only CSS and assets. It exercises
class-selected button variants, attribute-selected badge tones, control
states, gradients, multiple and inset shadows, transitions, border-image,
an SVG mask, a licensed font, sounds, and a data-attribute rarity cue.

From the repository root, render and compare the sheet:

```sh
source scripts/lock.sh
og_gpu_acquire css-pack-sheet
OG_ENGINE_PACKAGE="$PWD/packages/engine" target/debug/og ui sheet \
	tests/fixtures/ui-css-pack/assets/styles/css-laboratory \
	--out .amp/in/artifacts/css-pack-sheet
bun tests/fixtures/ui-css-pack/verify.ts .amp/in/artifacts/css-pack-sheet
```

This is an on-demand visual check, not a performance measurement or a
short gameplay fixture. `verify.ts` reads all 24 owner-approved Metal
reference cells, including scroll pages, and rejects missing cells or
changed pixels. Review every changed cell before replacing references.
The tool does not record them automatically.

`og ui review` accepts the same directory with `--locales game,pseudo,zh,ar`.
Locale review reports measurements without grading the pack.

Three rules remain marked pending parts-as-nodes in `main.css`:

- Host `font-family` inheritance into internal labels.
- `slot[data-rarity=epic]::part(frame)` on the core slot. The document
  `[data-rarity=epic]` sample already shows the double rim.
- `progress::part(fill)` opacity keyframes. The rule parses, but captured
  frames at 0.3-second intervals are identical before parts-as-nodes.
  The `prefers-reduced-motion` override remains beside it.

The CSS keyframe-name parser now accepts hyphenated identifiers, including
`laboratory-signal`. The existing part animation replay test covers that
syntax on a real button part. Progress draws its fill without a layout
node in `crates/og-ui/src/ui/node_draw.rs`. Its paint and command caches
reuse the static host's commands even while the virtual fill animates.
