---
name: level-texturing
description: Textures a gray level glb by having an image model paint it from views a player could stand at and projecting the paintings into one texture with og bake projection (the trompe workflow, on the CPU). Use when a level blockout or kit-bashed level needs believable surface materials, when asked to texture or paint a level, or when a painted level shows seams, smears, gray holes or invented objects.
---

A level painted this way is a blockout plus one texture: the geometry stays
the level's author's, and the image model invents only surface (stone,
plaster, paving, tiles, windows drawn flat on walls). The engine does the
geometry half on the CPU, without a GPU or Blender: `og bake projection`
plans the cameras, renders each view's clay and the paint so far,
composites the model's picture and projects it into texels. You run the
model calls and judge the result.

A prop or character built as code is og-model's; this skill paints a
whole level's surface.

## 1. What it needs

- A glb with y up and metres. TEXCOORD_0 laid out without overlaps is used
  as it is; without TEXCOORD_0 `init` charts the level with xatlas at one
  texel density, and the exported glb carries those uvs. A level whose uvs
  overlap (tiling textures, mirrored halves) paints wrong: strip its
  TEXCOORD_0 and let the engine chart it. Name materials or meshes by what
  they are (`wall`, `roof`, `floor`, `door`, `stairs`): the names reach the
  prompts and decide which views are roof passes.
- Walkable floors: ground cameras stand where a player could, at eye
  height. A level with no floor (a facade, a floating island) needs
  `--aerial` views.
- An image model that repaints a picture while keeping its edges. Measured
  on gray level renders (2026-10): `qwen-image-2.1-pro` (the default, about
  35–50 s a view) and `qwen-image-3.0-pro` keep the geometry;
  `qwen-image-2.1-turbo` keeps it but paints thin materials;
  `qwen-image-2.0(-pro)` and `wan2.5-i2i-preview` move edges and add
  columns, so their paint lands on the wrong surfaces. None of them takes
  a mask, so the engine composites instead: known pixels are kept, new
  ones are colour-corrected into them, and what the model left gray is not
  projected.

## 2. The loop

`scripts/texture-level.ts` (bun), beside this file, drives one run
directory; everything is in it, so a run resumes and a view can be
re-projected. In a project the script is in the installed engine package,
`node_modules/@origingame/engine/knowledge/skills/level-texturing/scripts/`;
it runs the `og` on the PATH, or the one `--og` (or `OG_BIN`) names, such
as the project's `node_modules/.bin/og`.

```sh
bun node_modules/@origingame/engine/knowledge/skills/level-texturing/scripts/texture-level.ts \
  --level assets/models/village.glb --run runs/village \
  --style "a Mediterranean hill village: weathered limestone and ochre plaster walls, wooden doors and shutters, worn cobblestone paving" \
  --views 14 --roofs 4 --aerial 3 --holes 3
```

The default provider, `handoff`, calls no model itself: the script plans
the views, prepares the first one and prints `{"paint": {view, state,
prompt, painted, model}}` with exit code 3. Edit the `state` image with
that model and that prompt (whatever image-editing tool you have), save the
returned image to `painted`, and run the script again with `--run
runs/village --resume`. Repeat until it exits 0 with `done:` and the paths
of `texture.png` and `textured.glb`. Each view sees everything painted
before it, so paint them in order and one at a time. With a key of your
own, `--provider http --endpoint <url>` calls an OpenAI-compatible
`/v1/images/generations` endpoint (`OG_IMAGE_API_KEY`), and `--provider
dashscope` calls Alibaba Model Studio (`DASHSCOPE_API_KEY`); the script
then loops on its own.

Views come in passes: ground views from walkable points (`first` paints a
view with no paint yet, `fill` paints only the gray left in a painted
view), then `roof` passes with their own style (`--roof-style`), then an
aerial ring, then `--holes` views chosen for the most unpainted texels.
`--dry` writes every prompt without a model call; read them before
spending.

**In the game's style.** The `--style` text is the level's share of the
art brief, never words of your own: take the Style and Palette lines of
`bun node_modules/@origingame/engine/knowledge/scripts/brief.ts texture
"<the level's materials>" --project .` (og-taste §1), and keep that one
text for the whole run, roofs included (`--roof-style` from a second
brief naming the roofs). Pass the style's preview to the image model with
each view where it takes references. The exported glb is cooked under the
look like any model (og-build §8): with `ArtTexels` set, its texture is
resampled to the density over the level's size and held to `maxSize`, so a
`--size` above what the look keeps only spends model calls; with
`ArtShape`, its geometry is held to the budget too, so a level that must
keep its triangles takes `conform off` in its import settings.

Budget: the model calls scale with the surface a player can see, not with
triangles. A village square (1 000 triangles, 4096² texture) takes about
27 calls, 35–50 s each with the pro model (about $1.10 at $0.04 an image);
the engine adds about 3 s a view. A 240 m town needs hundreds of street
views (the plan predicts about 22 % of it from 200), so texture a large
level a district at a time, or stop at the views players actually stand
in. Plan with `--dry` first and report the calls, minutes and final
coverage the script prints.

## 3. Judging the result

Look at `textured.glb` lit in the engine, from the planned views and from
places the plan did not stand (visual-acceptance §2). Per view, `views/K/`
keeps `clay.png`, `state.png`, `painted.png` and `comp.png` (what was
projected; transparent where nothing was). Typical failures and what fixes
them:

- **Gray patches** left after the run: raise `--holes`, add `--aerial`, or
  check the patch is uv-mapped (a texel outside every view stays gray).
- **Invented objects** (awnings, lamps, crates painted onto walls): the
  prompt forbids additions the level does not have; if a view still adds
  them, delete that view's `painted.png`, remove its line from `log.jsonl`
  and repaint it. Do not raise the model's freedom to get richer pictures.
- **Smears down walls or across roofs**: a grazing view. The projection
  prefers the more head-on view for each texel, so painting a better view
  of that surface replaces it.
- **Seams between views**: colour correction works only when the new view
  overlaps paint; order the plan so each `fill` view sees painted surface,
  and keep one `--style` text for the whole run.
- **Paint on the wrong surface**: the model moved edges. Use a model from
  the list in §1; check `painted.png` against `clay.png` before blaming
  the projection.

The texture is one image in uv space, so the level must be re-textured
after its geometry or uvs change; a material or lighting change does not
need it.
