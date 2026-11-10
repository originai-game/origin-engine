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

The following remain pending parts-as-nodes in `main.css`:

- Host `font-family` inheritance into internal labels.
- `slot[data-rarity=epic]::part(frame)` on the core slot. The document
  `[data-rarity=epic]` sample already shows the double rim.
- Hover and active states on text-field frames. Those states are not yet
  emitted by the field painter, so the fixture does not claim them.

The progress fill runs the `laboratory-signal` opacity animation; its three
neighbouring meters stay static. Paint reuse accounts for part animations
even when the host does not change. The regression test checks advancement,
alternate direction and checkpoint replay. The `prefers-reduced-motion`
override remains beside the rule.
