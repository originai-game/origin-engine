# The looks' preview scenes

A catalogue preview (`packages/knowledge/previews/<slug>.webp`) is "the
style as a game": the engine's own frame of the look's preset scene, so it
never promises more than a game in that look draws (docs/art.md §2.6).

- `3d/`: a traveller with a lamp on a path, a pine, an oak, boulders and a
  bush, a crate and a street lantern on a meadow that rolls away to hills
  and a village. Round forms and rolling land on purpose: a look's facets,
  cel bands, strokes or material must show on them. Every model is cooked,
  so the cook conforms each to the look's art contract. `bun make3d.ts` writes
  the two glbs (the crate, and the 10 x 10 cell meadow, within every look's
  prop budget) from `sources/`; the rest is checked `file.model` TypeScript
  under `content/models/`, loaded by canonical content ID.
- `2d/`: one place as a 2D game's screen, drawn in every 2D look: a hooded
  traveller with a lantern on a level's ground (a tilemap of a 4 x 4 tile
  set, with a ledge two tiles up), a floating platform with gems over it, a
  street lantern, a crate at the ledge's foot and a slime on it, trees and
  ground tufts; behind them a tree line and the far valley as two more
  layers; the lanterns' 2D lights; and a HUD of hearts and a gem count
  (the count is screen text at the frame's own resolution, over the
  finished frame, so no look's grid or dither reaches it). The `top` plan is the same place from above, for top-down
  looks: the meadow and its paved path, the plateau's cliff, the same
  traveller, lantern, crate, slime, gems and trees, sorted by their feet.
  Every beat is a sprite or tile set of the look's authored art, at the
  size the look's contract gives it: on a pixel grid a texel is a logical
  pixel and a tile is a metre. `stage.ts` copies a look's art in and writes its import settings,
  tile set, map and sizes.
- `preview.json`: which scene each look and variant takes, and the art it
  takes from `templates/looks/` (a look's own models, pixel sprites stood in
  the 3D set, the 2D art with its sizes, pixel unit, frame size and light).
  A look not listed draws the 3D scene with its base models.

Draw them with the `og` and addon of the commit under test (a preset is
`include_str!` into og-cook, so rebuild `og` after a preset edit):

```sh
# bun loads an addon only under a .node name: a hard link (a symlink is not enough).
mkdir -p target/look-previews
ln -f target/debug/libog_host_node.so target/look-previews/og-host.node
OG_NATIVE_ADDON=$PWD/target/look-previews/og-host.node OG_ENGINE_PACKAGE=$PWD/packages/engine \
  bun templates/looks/preview/render.ts [--webp] [slug | slug:variant ...]
```

Frames go to `target/look-previews/frames/`; `--webp` also writes each
look's catalogue preview (960 x 540), a variant's as `<slug>.<variant>.webp`.
A release draws them itself, with the creation document's every slot
(`--slots`: looks, variants, moods and UI packs), on the GPU host
(`--gpu`): `scripts/release.sh previews` (docs/release.md §1 The looks). A 2D look is drawn at the size its
entry names, a whole multiple of its pixel grid, so no frame has a border.
No frame here is a reference: read each against its catalogue description.

The 2D art is made by `2d/tools/art.ts`: each beat's request is the look's
art brief (`artBrief`, in the variant), sent to the origin gateway's image
generator, journalled in `2d/tools/gateway-journal.md`, then cut to its
subject and brought to the look's grid. The answers as generated stay under
`target/look-previews/raw/` on the machine that made them. A 2D look's
`light` is day, night or none; `none` draws the art unlit.

## The hero scenes in every look

`heroes.ts` draws The Last Light (`showcase/the-last-light`)
in every 3D look and variant: the game as it is,
its set, camera, light and code, with only its look swapped. Each look gets
a copy of the project under `target/look-previews/heroes/<hero>/` whose look
file is the game's own with its `extends` line naming the preset (and a
`variant` line after it), so the game's mood, exposure and grade lie over
every preset, as they would for a game that changed its look; `game` is the
hero's own look, unchanged. Nothing under `showcase/` is written, and no
prop is rebuilt for a look: where a look's art is content (voxel, paper,
neon line art, pixel sprites), the frame shows what the conform and frame
layers alone do to the hero's models.

```sh
OG_NATIVE_ADDON=$PWD/target/look-previews/og-host.node OG_ENGINE_PACKAGE=$PWD/packages/engine \
  bun templates/looks/preview/heroes.ts the-last-light --sheet [slug | slug:variant | game ...]
```

Frames go to `target/look-previews/heroes/<hero>/frames/`, with
`sheet.jpg`, a labelled contact sheet of them (`--no-draw --sheet` makes it
from the frames already there). The scene's own seconds are stepped without
drawing and `--settle` seconds are drawn before the frame (1 s by default).
On a software renderer a frame of these scenes takes minutes: pass
`--settle 0.1` and an optimised `og` (`--og`), and take those frames as
previews; the frames of record come from the GPU host.

What a hero's own code writes each tick is the scene's, not the look's: The
Last Light sets the exposure, the colour temperature and the saturation
(`src/climate.ts`), and its fog and sky, so a preset's own Fog,
Sky, exposure or saturation does not show in these frames.

The sources: `sources/crate.jpg` and `sources/grass.jpg` are the textures
the art samples' crate and ground carried, generated for them through
`artBrief` (`tools/art-samples/make3d.py`, removed in 9665c6734); the 3D scene's hero, pine, lantern and hills are the
art samples' cinematic family's (removed in 3e8873037); `oak.ogt` and
`rocks.ogt` are new.
