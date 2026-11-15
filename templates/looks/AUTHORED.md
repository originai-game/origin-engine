# Authored art kept with its look

A few looks of the catalogue, and some of their variants, are content
styles (`look.support` content, or a variant's `authored`): the conform and
frame layers cannot make their art, so the art was drawn for them. That art
lived in the art samples (`examples/art-*`, deleted on 2026-11-06) and is
kept here, beside the look's preset, under the look's slug and, for a
variant, the variant's name:

- `<slug>/art/`: a look's own sprites: pixel's beats (the art sample's
  16-bit set, with its import settings) and pixel-3d-hybrid's hero and
  pine, the pixel sprites it stands in a 3D set.
- `<slug>/<variant>/art/`: the 2D sample beats drawn in a variant (hero,
  tree, crate, lantern, ground, backdrop): pixel's 8bit, coarse, detailed,
  luminous and monochrome, cartoon-2d's storybook and rubber-hose, and
  painterly-3d's expressionist, lush and luminous-layers.
- `<slug>/models/` and `<slug>/<variant>/models/`: the 3D beats built in the
  style (`type model` text), for neon-vector, low-resolution-3d and toy-3d's
  paper variant.
- `cinematic-realism/realistic-set/`: the realistic asset set that
  cinematic-realism and its muted variant share (packed glbs, their README
  with each model's source and licence).

`preview/` draws every look on its preset scene with this art
(`preview/preview.json`) into the catalogue's previews. Nothing here is a
preset: the cook reads only `templates/looks/*.ogt`,
`variants/`, `assets/` and `moods/`, and `pack-template.ts` ships only those.
