# The looks' preview scenes

A catalogue preview (`packages/knowledge/previews/<slug>.webp`) is "the
style as a game": the engine's own frame of the look's preset scene, so it
never promises more than a game in that look draws (docs/art.md §2.6).

- `3d/`: a traveller with a lamp on a path, a pine, an oak, boulders and a
  bush, a crate and a street lantern on a meadow that rolls away to hills
  and a village. Round forms and rolling land on purpose: a look's facets,
  cel bands, strokes or material must show on them. Every model is cooked,
  so the cook conforms each to the look's art contract. `make3d.py` writes
  the two glbs (the crate, and the 10 x 10 cell meadow, within every look's
  prop budget) from `sources/`; the rest is `type model` text.
- `2d/`: a far layer, a strip of ground, a tree, the hero, a crate and a
  lantern, each a sprite of the look's authored art.
- `flat/models/field.ogt`: a flat one-colour field for toy-3d's voxel
  variant, whose 0.1 m cubes would raise the rolling meadow over the props.
- `preview.json`: which scene each look and variant takes, and the art it
  takes from `templates/looks/` (a look's own models, pixel sprites stood in
  the 3D set, the 2D art and its sizes). A look not listed draws the 3D
  scene with its base models.

Draw them with the `og` and addon of the commit under test (a preset is
`include_str!` into og-cook, so rebuild `og` after a preset edit):

```sh
OG_NATIVE_ADDON=target/debug/libog_host_node.so OG_ENGINE_PACKAGE=packages/engine \
  bun templates/looks/preview/render.ts [--webp] [slug | slug:variant ...]
```

Frames go to `target/look-previews/frames/`; `--webp` also writes each
look's catalogue preview (960 x 540). No frame here is a reference: read
each against its catalogue description.

The sources: `sources/crate.jpg` and `sources/grass.jpg` are the textures
the art samples' crate and ground carried, generated for them through
`artBrief` (`tools/art-samples/make3d.py`, removed in 9665c6734); the 3D scene's hero, pine, lantern and hills are the
art samples' cinematic family's (removed in 3e8873037); `oak.ogt` and
`rocks.ogt` are new.
