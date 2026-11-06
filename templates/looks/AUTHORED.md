# Authored art kept with its look

A few looks of the catalogue are content styles (`look.support` content):
the conform and frame layers cannot make their art, so the art was drawn
for them. That art lived in the art samples (`examples/art-*`, deleted on
2026-11-06) and is kept here, beside the look's preset, under the look's
slug:

- `<slug>/art/`: the 2D sample beats drawn in the style (hero, tree,
  crate, lantern, ground, backdrop), for the paint-2d and pixel-side
  styles.
- `<slug>/models/`: the 3D beats built in the style (`type model` text,
  og-model §5), for neon-vector, paper-craft and low-resolution-3d.
- `cinematic-realism/realistic-set/`: the realistic asset set that
  cinematic-realism and muted-cinematic-3d share (packed glbs, their
  README with each model's source and licence).

Nothing here is a preset: the cook reads only `templates/looks/*.ogt`,
`assets/` and `moods/`, and `pack-template.ts` ships only those.
