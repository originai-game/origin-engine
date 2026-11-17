# Authored art kept with its look

A few looks of the catalogue, and some of their variants, are content
styles (`look.support` content, or a variant's `authored`): the conform and
frame layers cannot make their art, so the art was drawn for them. That art
lived in the art samples (`examples/art-*`, deleted on 2026-11-06) and is
kept here, beside the look's preset, under the look's slug and, for a
variant, the variant's name:

- `<slug>/art/` and `<slug>/<variant>/art/`: the 2D preview scene's beats
  drawn in a 2D look or variant (pixel and its 8bit, coarse, detailed,
  luminous, monochrome and topdown; cartoon-2d and its watercolor,
  storybook and rubber-hose): hero, tree, crate, lantern, slime, gem, bush,
  heart, platform, the ground's tile set (`tiles`), the tree line (`mid`)
  and the far layer (`backdrop`); topdown has a floor, a path and a cliff
  tile set (`grass`, `path`, `wall`) in place of the last four.
  `preview/2d/tools/art.ts` makes them from each look's art brief.
  Edited by hand after that: monochrome's `backdrop.png` (the earlier
  sample's, its sun moved from the top right corner to the open sky, clear
  of the HUD), and storybook's and rubber-hose's `crate.png` and
  `lantern.png` (the earlier sample's, a strip of a neighbouring sprite
  cut from the top of each).
- `pixel-3d-hybrid/art/`: its hero and pine, the pixel sprites it stands
  in a 3D set.
- `painterly-3d/<variant>/art/`: the 2D sample beats drawn in its
  expressionist, lush and luminous-layers variants (hero, tree, crate,
  lantern, ground, backdrop).
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

A preview is one still frame, so it cannot show a trait that lives in
motion. cartoon-2d's rubber-hose variant names gate weave among its intended
traits (the catalogue's `intended`): its preview does not show it, and the
engine has no gate-weave effect, so the variant layer sets none.
