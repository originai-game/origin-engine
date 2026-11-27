# Authored art kept with its look

A few looks of the catalogue, and some of their variants, are content
styles (`look.support` content, or a variant's `authored`): the conform and
frame layers cannot make their art, so the art was drawn for them. That art
lived in the art samples (`examples/art-*`, deleted on 2026-11-06) and is
kept here, beside the look's preset, under the look's slug and, for a
variant, the variant's name:

- `<slug>/art/` and `<slug>/<variant>/art/`: the 2D preview scene's beats
  drawn in pixel, pixel's topdown variant and cartoon-2d: hero, tree, crate, lantern, slime, gem, bush,
  heart, platform, the ground's tile set (`tiles`), the tree line (`mid`)
  and the far layer (`backdrop`); topdown has a floor, a path and a cliff
  tile set (`grass`, `path`, `wall`) in place of the last four.
  `preview/2d/tools/art.ts` makes them from each look's art brief.
- `<slug>/models/` and `<slug>/<variant>/models/`: the 3D beats built in the
  style (checked `file.model` TypeScript), for neon-vector and toy-3d's
  paper variant.

`preview/` draws every look on its preset scene with this art
(`preview/preview.json`) into the catalogue's previews. Nothing here is a
preset: the cook reads only `templates/looks/*.ts`,
`variants/`, `assets/` and `moods/`, and `pack-template.ts` ships only those.
