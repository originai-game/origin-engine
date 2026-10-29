# A7 `art/samples`: one scene per family, one frame per style

Stream A7 of docs/art.md (§2.6, the A7 row of §3, §6), branch `art/samples`
from `a0fc69f5`, rebased onto `68c8597b` (origin/main with the decimation
fix `dda9b360`) before the final frames. Every frame below was taken after
the rebase, with this branch's dev `og` and addon (`OG_NATIVE_ADDON`,
`OG_ENGINE_PACKAGE`), on lavapipe at 1280x720 and read at 640x360 beside
its preview.

## What landed

| Commit | What |
|---|---|
| `807e8deb` | og-cook: voxels answer to the face budget when the source is within it (below), with `conform_mesh::a_model_within_its_budget_still_holds_its_voxels_to_it`, which fails on the old code (71,738 faces at 0.1 m against propFaces 300) |
| `ef6e9d77` | `examples/art-<family>` (the eight scenes, their sources and per-style sources) and `tools/art-samples/` (capture, prepare, slice, make3d, neon, catalogue, verdicts; README) |
| `d939449f` | the catalogue's `look.support` and `missing` per style, og-taste §12's table, one validator test made data-independent |
| this commit | these notes, `frames/` (37 frames at 640x360), `contact-sheet.jpg`, `style-check.json` |

## Examples, not a station

One example project per family (`examples/art-<family>`), not a new
fx-showcase station. The point of the stream is that the cook conforms the
assets to the look `project.ogt` names, and that is per project: one
project can hold one contract, so a station inside fx-showcase (one
project, one look line, runtime geometry the cook never sees, as A3's
notes found) cannot show 37 contracts. A game on the platform is one
project with one look, so an example per family is also the honest shape.
`tools/art-samples/capture.ts` copies the family's project per style,
points its look at the style, cooks, runs one frame and `og style check`.
The examples sit outside every gate lane (no `package.json`, no `tests/`),
so they cost the gate nothing; each still passes `og check` (tsrs) when it
runs.

## How the assets were made

- **Through the brief.** `beats.json` in each example names every asset's
  `artBrief` kind, subject and size; `prepare.ts` briefs them under the
  style's look on every run (`<slug>.briefs.txt` beside each capture) and
  writes the image's import settings from the brief's grid (`conform {
  size N }`) and the sprite sizes (`src/cells.ts`). Images were generated
  from those prompts with the style's preview as the reference image.
- **Generator: Painter (Amp's image tool), not the origin MCP.** The origin
  `generate_image` answered `Downstream HTTP 503 ... model_not_found` ("no
  channel") for its default model and for `gpt-image-2.5-flare` and
  `-sunburst` (the only values it accepts) with OG_PLATFORM_DEVELOPMENT_KEY,
  every try on 2026-10-28, so no image came from it. 56 images were made
  with Painter: per family a base set (hero, tree, crate, lantern, ground,
  far layer), then per style whose direction differs a 2 x 2 sheet of the
  four props (cut by `slice.py`), a far layer and a ground (the paint
  styles' ground is the far layer's own floor band).
- **3D props** are og-model §5 generators as `type model` text (the hero of
  capsules, a sphere and a cone; the stepped tree of a cylinder and three
  cones; the lantern; the far hills and village), the same for every 3D
  style, so the cook's conformance makes the difference; the crate and the
  ground are glbs around a Painter texture (`make3d.py`). pixel-3d stands
  the hero and the tree as billboarded pixel sprites (the 16bit set);
  shape-3d takes an untextured field; neon-vector its own black-and-glow
  models (`neon.py`). `generate_3d` was not used.
- **Bytes:** the eight examples' sources are 4,544,446 bytes (2D families
  carry per-style sets: paint-2d 2.37 MB, pixel-side 1.10 MB; each 3D
  family about 0.17 MB). `.gitattributes` has no LFS rule, so they are
  ordinary blobs. This evidence is 2.7 MB.

## The table

Read at 640x360 beside `packages/knowledge/previews/<slug>.webp`; the
check is `og style check --no-frames` on the style's copy of the scene
(`style-check.json`). Support: `content` where the style needs authored
art, `full` where it reads and the check passes, else `partial`.

| Style | Family | Frame | Reads as the style | og style check | Support |
|---|---|---|---|---|---|
| `expressionist-painting` | paint-2d | [expressionist-painting.jpg](frames/expressionist-painting.jpg) | yes: clashing violet and orange brush masses, broken edges, the figures painted into the scene | pass | content |
| `bold-comic-illustration` | ink-3d | [bold-comic-illustration.jpg](frames/bold-comic-illustration.jpg) | yes: thick black outlines on every form, flat posterised fills, a printed-colour sky | pass | full |
| `watercolor-wash` | paint-2d | [watercolor-wash.jpg](frames/watercolor-wash.jpg) | yes: translucent washes with pooled rims on white paper, the cook's texel-space paint on every sprite | pass | full |
| `pixel-3d-hybrid` | pixel-3d | [pixel-3d-hybrid.jpg](frames/pixel-3d-hybrid.jpg) | yes: conformed pixel sprites (hero, tree) stand as billboards with drop shadows among low-detail 3D props, dithered and depth-blurred | pass | full |
| `faceted-low-poly` | shape-3d | [faceted-low-poly.jpg](frames/faceted-low-poly.jpg) | no: the tree's facets read, but the hero is kept whole at 332 faces (over propFaces 200) and stays round, and the crate keeps its wood texture | off: models/hero.ogt: faces 332 against <= 200 (ArtShape.propFaces); models/hills.ogt: faces 500 against <= 200 (ArtShape.propFaces) | partial |
| `sculpted-clay` | shape-3d | [sculpted-clay.jpg](frames/sculpted-clay.jpg) | no: the 2 cm bevel is too small to see at this distance and the surfaces are plain PBR: nothing reads as clay | pass | partial |
| `8bit-pixel` | pixel-side | [8bit-pixel.jpg](frames/8bit-pixel.jpg) | yes: flat 4-step blocks, tiny sprites on an exact grid, a banded sky: the early-console language of the preview | pass | full |
| `voxel-worlds` | shape-3d | [voxel-worlds.jpg](frames/voxel-worlds.jpg) | yes: the tree, hero and hills are stepped cubes on one grid, single-colour faces | pass | full |
| `illuminated-manuscript` | paint-2d | [illuminated-manuscript.jpg](frames/illuminated-manuscript.jpg) | yes: parchment, gold sun, ink-outlined flat figures, an ornamental border and floral frieze | pass | content |
| `rubber-hose-animation` | paint-2d | [rubber-hose-animation.jpg](frames/rubber-hose-animation.jpg) | yes (since the preset tuning below): the sepia film print with grain and scratches, the hose figure and inked props in flat fills; the inked line still measures 5 to 7 px against ArtLine.width 3, the generated art's ink | off: art/crate.png: line 6 px against 3 px (ArtLine.width); art/hero.png: line 7 px against 3 px (ArtLine.width); art/lantern.png: line 5 px against 3 px (ArtLine.width); art/tree.png: line 6 px against 3 px (ArtLine.width) | content |
| `pencil-hatching` | ink-3d | [pencil-hatching.jpg](frames/pencil-hatching.jpg) | yes: grey graphite hatching over every surface, contour lines, paper white | pass | full |
| `gothic-contour` | paint-2d | [gothic-contour.jpg](frames/gothic-contour.jpg) | yes (since the preset tuning below): misty blue-grey spires fading into haze behind dark ornamental arches, the pale hooded figure, ink contours and the vignette | pass | content |
| `lush-painted-worlds` | paint-2d | [lush-painted-worlds.jpg](frames/lush-painted-worlds.jpg) | yes: opaque warm-lit masses with cool shadow, brushwork in the foliage | pass | content |
| `decorative-storybook` | paint-2d | [decorative-storybook.jpg](frames/decorative-storybook.jpg) | yes: flat patterned planes, ornamental trees and flowers, coloured line | pass | content |
| `classical-painting-collage` | paint-2d | [classical-painting-collage.jpg](frames/classical-painting-collage.jpg) | yes: old-master fragments with cut seams under one varnish tone, cut-out figures | pass | content |
| `detailed-doodle` | paint-2d | [detailed-doodle.jpg](frames/detailed-doodle.jpg) | yes: fine irregular line over everything, tiny houses and mills, very little colour | pass | content |
| `black-figure-pottery` | paint-2d | [black-figure-pottery.jpg](frames/black-figure-pottery.jpg) | yes: black silhouettes with incised detail on ochre, meander bands top and bottom | pass | content |
| `heavy-ink-shadows` | ink-3d | [heavy-ink-shadows.jpg](frames/heavy-ink-shadows.jpg) | yes (since art/render-full): solid black masses where the sun does not reach (MaterialStyle.shadowDarkness 1), thick ink on every form, muted posterised fills under a near-black sky and blood-red haze; 36 % of the frame is near-black | pass | full |
| `monochrome-pixel` | pixel-side | [monochrome-pixel.jpg](frames/monochrome-pixel.jpg) | yes (art/samples-full): the handheld greens in Bayer dither, silhouettes on pale ground, one ink on one paper (the frame as before; only the check moved: the ground tile was on its grid, the grid detector counted its flat rows) | pass | full |
| `16bit-pixel` | pixel-side | [16bit-pixel.jpg](frames/16bit-pixel.jpg) | yes: ramped shading, the cook's 1-px inside line on every sprite, parallax far layer, saturated keeper against calmer scenery | pass | full |
| `detailed-pixel` | pixel-side | [detailed-pixel.jpg](frames/detailed-pixel.jpg) | yes: dense hue-shifted clusters at 32 px per metre, the grid still visible, scenery rich but quieter | pass | full |
| `luminous-pixel` | pixel-side | [luminous-pixel.jpg](frames/luminous-pixel.jpg) | yes: near-black masses, cyan rim light, the lantern and windows the only warm glows | pass | full |
| `topdown-tile-pixel` | pixel-topdown | [topdown-tile-pixel.jpg](frames/topdown-tile-pixel.jpg) | yes (art/samples-full): a strict 16 px tile grid seen from above: a raised ledge with its cliff face and stone steps, an auto-tiled dirt path and pond, grass with bushes and stumps, upright y-sorted sprites standing on it; floor, path, wall face and water stay apart in greyscale | pass | full |
| `low-resolution-3d` | pixel-3d | [low-resolution-3d.jpg](frames/low-resolution-3d.jpg) | yes (since art/render-full): a 240-line frame with nearest texels and a Bayer-dithered sixteen-tone grade, short dark fog the colour of the sky closing the draw distance, faceted 6 x 4 hills (the style's own source, 264 faces); vertices snap to the grid and textures interpolate affinely (stylise-retro holds both) | pass | full |
| `coarse-pixel` | pixel-side | [coarse-pixel.jpg](frames/coarse-pixel.jpg) | yes: a few huge pixels per figure, one dominant night-blue mass, the lamp the only warm point | pass | full |
| `anime-cel-3d` | toon-3d | [anime-cel-3d.jpg](frames/anime-cel-3d.jpg) | yes: two-tone cel bands, dark outlines on every model, a clean sky | pass | full |
| `inked-comic-3d` | toon-3d | [inked-comic-3d.jpg](frames/inked-comic-3d.jpg) | yes: heavy ink contours and hatched fills over the whole frame | pass | full |
| `paper-craft` | shape-3d | [paper-craft.jpg](frames/paper-craft.jpg) | no: flat facets and outlines draw, but there is no paper grain, fold or layered card | pass | partial |
| `miniature-diorama` | shape-3d | [miniature-diorama.jpg](frames/miniature-diorama.jpg) | no: the tilt-shift blur reads as miniature, but the props are plain PBR, not toy-like or bevelled | pass | partial |
| `painterly-3d` | cinematic-3d | [painterly-3d.jpg](frames/painterly-3d.jpg) | no: the crate's and ground's textures carry brush strokes (the cook's texel-space paint), but the hero, tree, lantern and hills are flat-coloured blockouts with no painted texture and realistic proportions: nothing for a brush to work on | pass | content |
| `cinematic-realism` | cinematic-3d | [cinematic-realism.jpg](frames/cinematic-realism.jpg) | yes (art/samples-full): scanned and generated models (a Meshy lantern keeper, Poly Haven crate, street lamp, hand lantern and stump, a stand of Meshy pines) on a Poly Haven forest ground under a low dusk sun: long soft shadows, warm lamp pools, a hazed ridge, AgX grade, grain and vignette | pass | full |
| `muted-cinematic-3d` | cinematic-3d | [muted-cinematic-3d.jpg](frames/muted-cinematic-3d.jpg) | yes (art/samples-full): the same real models in grey haze between depth layers, the lamp and hand lantern the only warm accent (mean saturation 0.07), soft volume and no outlines | pass | full |
| `layered-silhouette` | cinematic-3d | [layered-silhouette.jpg](frames/layered-silhouette.jpg) | yes: every form a flat dark silhouette in depth bands against a pale sky | pass | full |
| `minimal-geometric` | shape-3d | [minimal-geometric.jpg](frames/minimal-geometric.jpg) | yes: flat untextured colour planes, baked one colour per face, no texture anywhere | pass | full |
| `neon-vector` | shape-3d | [neon-vector.jpg](frames/neon-vector.jpg) | yes: black faces under glowing cyan lines on a dark field, bloom on the lantern (neon-vector's own sources, tools/art-samples/neon.py) | pass | full |
| `monochrome-dither-3d` | pixel-3d | [monochrome-dither-3d.jpg](frames/monochrome-dither-3d.jpg) | yes (since the Bayer fix below): two colours, a fine ordered dither carries the light: the tree, hero, crate, lantern and hills keep their forms in perspective | pass | full |
| `luminous-layered-painting` | paint-2d | [luminous-layered-painting.jpg](frames/luminous-layered-painting.jpg) | yes: dark cool painted layers, the lantern and village the luminous focal points, mist into depth | pass | content |

**22 full, 4 partial, 11 content** (13, 14 and 10 at A7; the flat-normal fix, G2, G3 and the
preset tuning below moved ten to full; painterly-3d moved to content). The 10 paint-2d content styles read as their style on the sample; the authored assets that carry each are its own
Painter set in `examples/art-paint-2d/styles/<slug>/art/` (watercolor-wash,
also paint-2d, is not content: its set is the family's base in
`assets/art/`).

![contact sheet: every frame beside its preview](contact-sheet.jpg)

Reference names: the frame of `<slug>` is
`examples/art-<family>/references/<tier>/<slug>.png` (1280x720). No
`metal` file is committed: `bun tools/art-samples/capture.ts --only
<family> --out examples/art-<family>/references/metal` on macrunner
records a family's. No existing reference frame moves (the og-cook fix
changes only voxelised models past their budget, and no shipped project
voxelises).

## Findings

1. **Fixed (`807e8deb`): voxels ignored the face budget for a model within
   it.** The decimation plan is made from the source's triangles, so a
   model under its budget before voxelising had no target and kept its
   voxels at the first edge that fit the 256-cell grid: the 40 m ground
   cooked to 147,318 faces against voxel-worlds' propFaces 5000. Each
   primitive's voxels now answer to its share of the budget.
2. **Fixed (branch `art/flat-normals`): flat share under `normals flat`.**
   The hero measured 95.4 % (faceted), 98.0 % (paper, minimal) and 92.6 %
   (voxel) flat against 99 %. The check was right: every off corner was
   1.7° to 90° from its face (none near the 1° tolerance; 16-bit
   octahedral normals err by under 0.01°), and two cook faults made
   them. (a) A lathe's pole triangles (og-model's sphere, capsule and
   cone: two corners at the pole, area about 1e-8 m²) have no normal of
   their own; quantised to the 16-bit grid they decode as slivers facing
   any way. They draw nothing (thinner than one grid step) except through
   the outline hull, where their arbitrary hull normals drew a brown band
   across the hero's neck. `shade` now drops every face thinner than one
   grid step. (b) Voxels are made in model space and brought back under
   the part's node; their normals went back through the inverse's
   transpose, which is the inverse transpose of the placement, not of its
   inverse. A part turned 45° (the arm, the hills' turned blocks) got
   normals turned 90° off their faces. They now go back through the
   placement's transpose. Every model of the four styles measures 100 %
   flat; voxel-worlds and minimal-geometric are `full`. Frames move (read
   at 640x360 on lavapipe): voxel-worlds on the turned parts (909 px),
   paper-craft and minimal-geometric under the hero's head (70 and 29 px,
   the neck band gone), faceted-low-poly and sculpted-clay by a level on
   12 and 10 px; `references/metal` of those styles wants recording again.
3. **Preset frames that work against their own style** (A1's files, the
   coordinator's call): `rubber-hose-animation`'s `Stylise.posterize 4`
   and `gothic-contour`'s `posterize 5` band the frame into olive/pink and
   flat greys (with `posterize 0` both read, checked by frame);
   `monochrome-dither-3d`'s dither cell swallows every form. Settled by
   the preset tuning below.
4. **Decimation within the silhouette bound cannot reach small budgets on
   round models:** the hero stops at 332 faces against faceted-low-poly's
   200, the hills at 596 against low-resolution-3d's 300 and 500 against
   faceted-low-poly's 200; the cook keeps them whole
   and the check reports them (the honest off of `dda9b360`).
5. **Samples' own limits,** named in `missing`: top-down needs a real tile
   set (walls, cliffs, paths); monochrome-pixel's generated ground tile is
   off its grid (half rows, an asset fault); cinematic-realism's crate and
   ground are below its density 512; the cinematic, painterly, clay,
   paper and diorama styles need assets built for them (realistic scans,
   painted textures, clay surfaces, paper sheets, toy materials) that one
   shared set of 3D primitives cannot stand in for.
6. `rubber-hose-animation`'s inked line measures 5 to 7 px against
   ArtLine.width 3: the generated art's ink is thicker than the contract.
7. docs/art.md §2.6 says the families are "§3 stream A10"; it is A7.

## Verification

- `cargo test -p og-cook --test tests conform`: 20 passed, the new test
  among them (and failing with the fix stashed);
  `cargo clippy -p og-cook --tests -D warnings`, `cargo fmt --check`: clean.
- `packages/knowledge`: `bun run test` (validator + 60 tests) passed;
  `packages/create`: `bun run test` (18) passed.
- `capture.ts` over all 37 styles: 37 frames, every run exit 0; 8 checks
  off (table).
- Not run, per the stream rules: scripts/check.sh, test lanes, `og test`,
  web-reference, wasm or release builds.

## Preset tuning (`art/preset-tuning`)

Branch `art/preset-tuning` from `9ccaf706`, for finding 3. Frames taken
with the branch's dev `og` and addon (`OG_NATIVE_ADDON`,
`OG_ENGINE_PACKAGE`) on lavapipe at 1280x720, read at 640x360 beside
the old frame and the preview: `tuning/<slug>.jpg` is before (`9ccaf706`)
| after | preview.

- **Why posterize bands.** Not the curve: `stylise.wgsl`'s `quantize`
  rounds each channel to `posterize` uniform steps in display space,
  which is the right space (ImageMagick's linear-space posterize of the
  same backdrop bands worse). It is the per-channel rounding at a low
  level count: a near-neutral colour whose channels sit closer than one
  step (85 codes at 4, 64 at 5) lands on unequal steps and changes hue.
  The rubber-hose backdrop's sepia `(189,151,104)` becomes `(170,170,85)`,
  olive; gothic's blue-grey haze grey or violet. At `posterize 8` both
  frames still shift (a purple tree, yellow and cyan patches), so no level
  count keeps these palettes. Both presets drop `Stylise.posterize`: the
  cel bands stay the toon material's (rubber-hose), the contours the ink
  pass's, with the shadow tint and vignette (gothic).
- **Why the dither swallowed the forms.** Not the cell: `bayer()` in
  `stylise.wgsl` (and og-cook's mirror for texture conformance) took the
  position's coarsest bit as the threshold's most significant, a
  quadrant-clustered matrix (`0 2 8 10 / 3 1 11 9 / ...`), so a quarter
  tone drew as one 4 x 4 block per 8 x 8: at 240 lines upscaled 3x, a
  24 px period checkerboard. Fixed to the dispersed Bayer matrix
  (`0 8 2 10 / 12 4 14 6 / ...`); the preset is unchanged (`bayer8` at
  `pixelHeight 240` reads once the matrix is right; `bayer4` and
  `pixelHeight 360` were tried and are not needed).
  `palette::tests::bayer_matches_the_frame` pins the matrix and fails on
  the old code.
- **Frames.** `rubber-hose-animation`, `gothic-contour`,
  `monochrome-dither-3d` read now (table above; style check unchanged:
  rubber-hose's line is still off, the other two pass). The Bayer fix
  also moves every other `bayer4`/`bayer8` style: `16bit-pixel`,
  `detailed-pixel`, `luminous-pixel`, `monochrome-pixel` and
  `pixel-3d-hybrid` were recaptured, read as before (monochrome-pixel's
  sky is a fine pattern where it was a coarse checker), and their checks
  are unchanged; their `frames/` and the contact sheet are refreshed.
- **Reference frames that move** (not re-recorded here; Metal is the
  coordinator's): `examples/art-paint-2d/references/metal/`
  `rubber-hose-animation.png`, `gothic-contour.png`;
  `examples/art-pixel-3d/references/metal/` `monochrome-dither-3d.png`,
  `pixel-3d-hybrid.png`; `examples/art-pixel-side/references/metal/`
  `16bit-pixel.png`, `detailed-pixel.png`, `luminous-pixel.png`,
  `monochrome-pixel.png`; `templates/fx-showcase/references/metal/`
  `looks-<slug>.png` for the same eight slugs (web-reference `--looks`
  compares against them); `crates/og-tests/references/{metal,webgl2}/`
  `stylise-pixel.png` (`Bayer4`); and the games on a `bayer4`/`bayer8`
  look: `examples/hollowbell/references/metal/*` (16bit-pixel, its own
  `bayer4`) and `examples/brasswick/references/metal/*` (detailed-pixel,
  `bayer8`), every frame that draws the world through the look. `bayer2`
  styles (8bit-pixel, low-resolution-3d, hollowbell's dawn) do not move.

## art/render-full: low-resolution-3d, heavy-ink-shadows, painterly-3d

Captured on lavapipe (`capture.ts --only <slug>`, read at 640x360) with
the branch's `og` and addon. The table's totals above predate this stream
and the parallel G1 and G3 streams; the coordinator recounts at the merge
(these three: two to full, one to content).

- **heavy-ink-shadows → full.** Cause: `shadowTint "#000000"` at
  `shadowTintAmount 0` is off, and `posterize 3` never reaches black: the
  environment's ambient light (and the toon ramp's 0.22 dark band) lifts
  every shadowed surface above the lowest step, so the shadow side kept
  its lit hue. Fix: `MaterialStyle.shadowDarkness` (new, 0 off;
  `pbr.wgsl` `shadow_keep`, in the standard and physical lit shaders):
  where the sun does not reach, a surface loses that share of the sun's
  ramp and of the ambient light. The preset sets 1, six posterize steps,
  a near-black sky, a dark red fog and muted grading. 36 % of the frame
  is near-black (luminance under 12 %; 3 % before).
- **low-resolution-3d → full.** Vertex snapping and affine texturing were
  already in the engine (`retro_clip`, `stylised_uv`; og-tests'
  `stylise-retro`): turning them off moves the sample's frame. What did
  not read was the scene: a long pale fog and sky. The preset's fog is now
  short (0.07) and dark, the colour of a flat sky (the clear colour, as
  the catalogue asks). The hills: four 24 x 12 spheres (2,376 triangles
  drawn) cannot be decimated to `propFaces 300` within the silhouette
  bound, so the style brings its own source as its brief asks for a prop:
  `examples/art-pixel-3d/styles/low-resolution-3d/models/hills.ogt`, 6 x 4
  spheres, 264 faces (the family's other styles keep theirs).
- **painterly-3d → content.** The frame's paint filter cannot show on
  these assets at any radius: a Kuwahara over flat-coloured blockouts and
  a smooth grass texture returns them as they were (radius 5, surface
  canvas and a warm grade still read as smoothed 3D). The engine part is
  the cook: under a look with `Stylise` paint fields the mesh conformance
  paints a model's own colour textures in texel space, as watercolor-wash's
  sprites are painted, so the crate and ground carry strokes. The
  preview's hand-painted textures and shaped forms on the hero, tree and
  props are authored art.

Reference frames that move (not re-recorded; Metal is the
coordinator's): `examples/art-ink-3d/references/metal/heavy-ink-shadows.png`,
`examples/art-pixel-3d/references/metal/low-resolution-3d.png`,
`examples/art-cinematic-3d/references/metal/painterly-3d.png`,
`examples/art-shape-3d/references/metal/sculpted-clay.png` and
`paper-craft.png` (their crate and ground textures are painted now);
`templates/fx-showcase/references/metal/looks-heavy-ink-shadows.png` and
`looks-low-resolution-3d.png` (the presets changed; the showcase has no
project look, so the cook's paint does not reach it). No og-tests
reference moves: `shadowDarkness` is 0 in every scene. The contact sheet
is not regenerated here.

## art/samples-full: monochrome-pixel, topdown-tile-pixel, cinematic-realism, muted-cinematic-3d

Branch `art/samples-full` from `16c1eb00`, rebased onto `08c9da54` (G2)
before the final frames. Frames taken with the branch's dev `og` and
addon on lavapipe at 1280x720, read at 640x360 beside the preview;
`full/<slug>.jpg` is before (`08c9da54`) | after | preview. The four rows
of the table above are this stream's; the totals line is the
coordinator's to recount (these four: partial to full).

- **monochrome-pixel → full. Cause: the check, not the asset or the
  cook.** A7 read `art/ground.png: grid 1x2, half pixels at rows 2 4` as
  an asset fault. The cooked tile (32 x 32) is one texel per logical
  pixel: A2's conformance had put it on the grid (`128x128 cells to 32x32
  by the dominant-colour box`, two colours of the handheld palette). Its
  grass is in the top five rows, solid ink below, so its row boundaries
  are at rows 1 to 5 only. `og style check`'s grid detector counted all 31
  row positions: the 26 flat rows carried no boundary on or off any grid,
  which let a period of 2 pass with rows 2 and 4 as its "half pixels".
  Fix (`og-cli/src/style_check.rs`, `axis_grid`): only the positions from
  the first boundary to the last are evidence.
  `a_tile_of_flat_ground_under_a_fringe_is_not_taken_for_a_coarser_grid`
  (the tile's own rows) fails on the old code (ky 2, half rows 2 4). The
  frame does not move; the check passes.
- **topdown-tile-pixel → full.** Cause: the sample had no tile set (one
  grass tile and a far layer drawn as one picture). Now
  `examples/art-pixel-topdown` is an `og.Tilemap` (`assets/maps/meadow.ogt`,
  20 x 12 tiles of one metre) on `art/terrain.ogt`: 16 plain tiles from one
  generated 4 x 4 sheet (grass, path, cliff face, steps, the cliff's foot,
  water, bush, stump; brief kind `tileset`; Painter, `generate_image`
  failing as below), sampled onto its 64 x 64 logical grid
  (`sources/terrain-sheet.png`), and three blob47 terrains (path, pond,
  raised ledge) that `tools/art-samples/tiles.py` draws from those
  swatches, 157 tiles at 2 texels a pixel. The cook conforms the atlas
  per tile (`conform { size 16 }`: `prepare.ts` writes a tile set's tile,
  not its 64 px sheet); the sprites stand on the map with `sortY`. Check:
  pass (`terrain.png` 63 colours, one texel per pixel). In greyscale the
  floor, path, wall face and water stay apart.
  **Engine gap:** the brief's `tileset` kind asks for a 4 x 4 sheet, and
  nothing in the engine turns swatches into a terrain's 47 (or 16)
  transition tiles: a game following the brief gets swatches that
  `terrain` rules cannot use. tiles.py does it outside the engine (ragged
  edges, rounded outer and notched inner corners, the rim shading per
  terrain). It belongs in the cook (a terrain from a fill and a ground
  swatch) or in the brief (a blob template sheet).
- **cinematic-realism and muted-cinematic-3d → full.** Cause: the family's
  primitives under the grade read as a blockout, and the crate and ground
  measured 213 and 128 texels/m (their 256 and 512 texel sources) against
  density 512. Change: a second asset set of the family,
  `examples/art-cinematic-3d/sets/realistic/` (its README lists every
  source and how it was packed), named by both styles
  (`styles/<slug>/set`, read by `capture.ts`), so painterly-3d and
  layered-silhouette keep the base set. Poly Haven's crate, street lamp,
  hand lantern, stump and forest ground (CC0, from the origin asset
  catalogue); a hero and a pine from Meshy through `generate_3d`; a far
  ridge from `make3d.py`; a low dusk sun, a procedural dusk sky and the
  lamps' lights in the set's `src/main.ts`. Checks pass: crate 695,
  ground 512, hero 491, hand lantern 456, stump 395, street lamp 332,
  pine 319 texels/m (256 to 1024); muted-cinematic-3d's mean saturation
  0.07 (its check: below 0.25). The hero keeps 21,110 faces under
  muted-cinematic-3d: it has no skin, so `characterFaces 10000` does not
  count it as a character and the look sets no `propFaces`.
  **Engine fix on the way (`og-cook`):** the cook sized a model's
  texture as if laid once across the model's longest side; the check
  measures through the UVs. Poly Haven's crate with its 2k diffuse was cut
  to 512 "over the model's 1.06 m" and then measured 174 texels/m. The
  cook now takes the span each image covers from the triangles that
  sample it (`conform::mesh::texel_span`), and `conform-texture` moves to
  v3 so the shared texture cache does not serve the old sizes.
  `a_texture_is_sized_by_the_surface_its_uvs_lay_it_over_not_the_model_extent`
  fails on the old code (64 against 16). Captured with the old and the
  new rule (texture cache off), bold-comic-illustration, anime-cel-3d,
  inked-comic-3d, low-resolution-3d, painterly-3d and layered-silhouette
  are pixel-identical: the base set's textures are below either size.

**Full, not content.** The catalogue: full when every layer but
`authored` is implemented, content when authored art is needed. None of
the four has the `authored` layer, and none is carried by art drawn for
it. monochrome-pixel's set is its own Painter set like every pixel-side
style's (the brief differs per style; the cook does the grid, palette and
dither). The top-down tiles are one generated sheet composed by a
generic script, not drawn per style; what the script does is the engine
gap above, a tooling gap rather than authored art. The realistic set is
catalogue scans and generated models, taken unmodified by two styles
whose looks make the difference (density 512 for one, characterFaces
10000 and the stage grade for the other, though the unrigged hero is not
held to the character budget), as a game gets its assets
through the brief. The honest limit: realism needs realistic inputs. The
style's layers cannot make a blockout realistic, so its proof needs a
set the family's primitives are not, and the frame is a sparse forest on
one ground material, plainer than the preview's.

**Assets and bytes.** 5,166,508 bytes of models
(`sets/realistic/assets/models/`: pine 1.77 MB, the 2k colour map the
density needs at 3.6 m; hero 0.82 MB; crate 0.72 MB; stump 0.56 MB; ground
0.44 MB; street lamp 0.39 MB; hand lantern 0.29 MB; ridge 0.18 MB); 47,530
bytes for the tile set (atlas 37,349 and its source sheet 10,181); the
evidence frames and strips about 0.5 MB. The topdown sample dropped its
ground and far layer.

**Generators.** The origin MCP's `generate_image` still answers
`Downstream HTTP 503 model_not_found` (`gpt-image-2.5-sunburst` and
`-flare`: "no channel") on 2026-10-29, so the tile sheet is Painter's.
`generate_tripo_3d` answered `code 2010: provider credits insufficient`
for both models. `generate_3d` (Meshy, meshy-6) made the hero (415k
triangles, though target_polycount said 20000) and the pine (2.7M); both
are simplified with gltf-transform. The catalogue's realistic search
found no realistic pine or hooded character.

**Reference frames that move** (not re-recorded; Metal is the
coordinator's): `examples/art-pixel-topdown/references/metal/topdown-tile-pixel.png`,
`examples/art-cinematic-3d/references/metal/cinematic-realism.png` and
`muted-cinematic-3d.png`. monochrome-pixel's frame does not move, and
the og-cook fix moves no sample (above). The contact sheet is not
regenerated here.
