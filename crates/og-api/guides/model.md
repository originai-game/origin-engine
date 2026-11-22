# Model programs and the og model tools

How to build a prop as a model program (`.shape.json`): parts from primitives and generators, CSG, bevels, materials and parameters, cooked by `og cook` like a `.glb`. It also covers the `og model` commands, which build, measure, preview, compare, process and texture a model, whether it is a program or a mesh file. The `type model` text, made from og.render's geometry descs, is a separate format, described in `models`.

The command reference: `og model help`.

## 1. The program

A program is JSON: a `name`, optional `params`, and `ops`, a list that is
replayed in order. Every operation that makes a part answers an `id`
(0, 1, 2, … in the order parts are made), and later operations name parts
by that id. The same program always builds the same bytes, on every machine.

- Units: millimetres unless the first op is `{"op": "units", "unit": "cm"}`
  (or `"m"`). Angles are radians.
- Axes: +Z is up and the model faces −Y. The cooked model (the GLB) is in
  metres with +Y up, facing +Z.
- A frame places a part: `{"origin": [x, y, z], "x": [...], "y": [...],
  "z": [...]}`, its axes unit length and right-handed. A generator's `on`
  is its frame; a box stands on the frame's XY plane, centred on its origin.
- An unknown field is an error that lists the fields the op takes.

## 2. Parts

Primitives: `box` (`width`, `depth`, `height`), `cylinder` (`radius`,
`radiusTop` for a taper, `height`), `cone`, `sphere`, `dome`, `tube`
(`outer`, `inner`), `torus`, `wedge`, `knurl`, `gear`, `thread`.

Generators, for curved and tapered parts:

- `extrude`: a profile pushed along +Z by `height`, with optional `twist`
  and `taper`.
- `revolve`: a profile turned about the axis (x is the radius): bowls,
  vases, table legs with a turned profile.
- `sweep`: a profile along a 3D path, with `scaleEnd` and `twist`: rails,
  handles, pipes.
- `loft`: `sections` `[{profile, z}]` blended into one solid: seats,
  hulls, bottles.
- `wire`: a round section along a path with filleted corners: chair
  frames, wire racks.
- `body`: a solid from its `plan`, `top` and `bottom` views: organic
  furniture shapes.

A profile is an object with one key: `rect`, `circle`, `ellipse`, `slot`,
`polygon`, `poly` (points, each with an optional corner `radius` or
`bulge`), `spline`, `superellipse`, `text`, `offset`, `union`, `subtract`,
`intersect`, `hull`, `mirror`, `translate`, `rotate` or `scale`. A path is
a start point followed by pieces: a point (a straight line), `{"arc":
{"through", "to"}}`, `{"tangentArc": {"to"}}`, `{"spline": {"points"}}`,
or in 3D `{"helix": {...}}`.

A `poly` point's `bulge` arches the edge from that point to the next, as
in DXF: the arch's middle stands `bulge` half-chords from the straight
edge, so 1 is a half circle and 0.3 on a 370 mm rail rises 55.5 mm.
Positive bows out of the outline, negative into it. An arched chair rail
is four points, not a sampled curve:
`{"poly": {"points": [[-185, 0], [185, 0], {"x": 185, "y": 70, "bulge": 0.3}, [-185, 70]]}}`.

Arranging parts: `copy`, `mirror` (`plane`: a frame, mirrored across its
XY plane), `at` and `transform` (a new frame), `scale`, `group`,
`assembly` and `attach`.

## 3. Shaping

- CSG: `cut`, `join` and `intersect` take a `target` and a `tool`, and
  report `voids`, touching faces and crossing pairs as facts. `merge`
  welds two parts without a boolean.
- Edits: `hole` (with `counterbore` or `countersink`), `pocket`, `split`,
  `shell` (hollow out, with an `open` face), `dent` and `subdivide`.
- Bevels: `fillet` (`radius`), `chamfer` (`distance`) and `soften`, on
  every edge or on the named `edges`, such as `"side"` or `"top"`.

## 4. Materials and parameters

`{"op": "material", "ids": [...], "name": "oak", "color": "#9a6b3f",
"roughness": 0.7}` colours the listed parts and everything under them.
The fields are three.js names: `color`, `metalness`, `roughness`,
`emissive`, `opacity`, `map`. Give parts that share a look one material
with one name, so a texture is painted once for all of them.

`"params": {"height": 720}` declares a parameter, and any field of any op
may then be `"$height"` or an expression over parameters and numbers:
`"$height - 30"`, `"($width - 2 * $leg) / 2"` (`+ - * /`, parentheses and
unary minus, in f64). A name that is not declared, or a division by 0, is
an error that names the expression. A
variant is a `type model` text beside the program that names it and sets
its own values:

```text
type model
version 1
---
program "table.shape.json"
params {
  height 900
}
material "oak" {
  color 0.4 0.25 0.15
}
```

`og cook` builds both the program at its own values and every variant.
A variant's `material` blocks replace only the fields they name (`color`
is linear here, as in glTF). `atlas <px>` packs every part's UVs into one
atlas, the layout `og model bake` paints its maps on.

## 5. From concept views to a prop

Fit the program to the concept first, then texture it:

1. Read the sizes off the views. The front view gives the width and
   height, `left` the depth, and `top` the width and depth. Write each
   size that the views fix as a parameter, and derive the other sizes
   from those with expressions.
2. Build and check every view in one call: `og model compare
   chair.shape.json --image front=f.png --image left=l.png --image
   top=t.png --json`. `byAspect` puts the view with the worst proportions
   first. Its `ratio` is model over image, so divide the parameter that
   sets that view's width or height by the ratio. `byIou` points at a
   view whose outline differs (a missing part or a wrong profile), and
   `byColour` points at a part whose material is off. `byWidth` lists the
   parts thinnest first: a part under two pixels (`resolved` false) cannot
   match the silhouette at that size, and each view's `iouOnePixel` is the
   most a fit of that silhouette can show, so a chair's legs read as the
   view's resolution, not a wrong shape. Fix them all, then compare
   again. Previews are lit and compare is not: `og model preview --views
   front --unlit` draws the colours compare measures.
3. `og model measure` lists every remaining problem in `issues`, each with
   the option that fixes it. A program's parts that touch are an
   assembly, not an issue.
4. `og model bake chair.shape.json --project front=f.png --project
   left=l.png --project top=t.png` paints the base colour, and
   `og model bake tall.ogt --from chair.ogt` carries that colour to each
   variant.

A generated mesh goes through `og model process` instead. Its `issues`
and `next` name what to run after it.

## 6. Recipe

A side table: a filleted top on four tapered legs, with its size and leg
radius as parameters. Each part is made once and mirrored into place.

```json program
{
  "name": "side_table",
  "params": { "width": 600, "depth": 450, "top": 30, "leg": 22 },
  "ops": [
    { "op": "box", "name": "top", "width": "$width", "depth": "$depth", "height": "$top",
      "on": { "origin": [0, 0, 690], "x": [1, 0, 0], "y": [0, 1, 0], "z": [0, 0, 1] } },
    { "op": "fillet", "id": 0, "radius": 8 },
    { "op": "cylinder", "name": "leg", "radius": "$leg", "radiusTop": 16, "height": 690,
      "on": { "origin": [250, 175, 0], "x": [1, 0, 0], "y": [0, 1, 0], "z": [0, 0, 1] } },
    { "op": "mirror", "id": 1, "name": "leg_left", "plane":
      { "origin": [0, 0, 0], "x": [0, 1, 0], "y": [0, 0, 1], "z": [1, 0, 0] } },
    { "op": "mirror", "id": 1, "name": "leg_back", "plane":
      { "origin": [0, 0, 0], "x": [0, 0, 1], "y": [1, 0, 0], "z": [0, 1, 0] } },
    { "op": "mirror", "id": 2, "name": "leg_back_left", "plane":
      { "origin": [0, 0, 0], "x": [0, 0, 1], "y": [1, 0, 0], "z": [0, 1, 0] } },
    { "op": "material", "ids": [0, 1, 2, 3, 4], "name": "oak", "color": "#9a6b3f", "roughness": 0.7 }
  ]
}
```

## 7. The og model commands

Each command takes a program, a mesh file (`.glb`, `.gltf`, `.obj`,
`.fbx`), a mesh's import settings (`x.glb.ogt`) or a variant `.ogt`. With
`--json` it prints one JSON object. A failure prints `{"error": "…"}` and
exits 2 for a usage mistake or 1 for a failure. Lengths without a unit
are metres. Every report gives facts and never a verdict: the thresholds
are the game's.

- `build`, `measure`: the parts, and the model's `size` (metres, GLB
  axes), triangles, materials and the look's facts. `issues` lists every defect
  (open, non-manifold or flipped edges, degenerate triangles, floating
  pieces, a budget the look states and the model exceeds), each with its fix. A fact the look does
  not state is null, with the reason in `look.absent`. Every 3D look
  preset states a prop budget (`ArtShape.propFaces`) and, where it has
  textures, the largest one (`ArtTexels.maxSize`); a look with a free
  palette states no colours.
- `preview --views front,left,top --out DIR [--unlit]`: one PNG per view
  on a transparent background, lit with edges, or with `--unlit` each
  part its material's colour as compare draws it. The views are `front`, `back`, `left`, `right`,
  `top`, `three_quarter`, or `camera:{"eye", "target", "up", "fov"?}`.
- `compare --image view.png --view front`: silhouette IoU, the bounding
  box's aspect against the image's, and the colour distance per part. `--image front=f.png --image left=l.png …` compares
  every view in one call and ranks them (`byIou`, `byAspect`, `byColour`,
  `byWidth`). Each part reports its width in the image's pixels and
  whether the view resolves it, and each view `iouOnePixel`, the IoU a
  one-pixel misfit leaves.
- `process mesh.glb --out DIR [--height 1.2 | --size x,y,z]`: repair (only
  what `--weld`, `--non-manifold` and `--holes` ask), decimation, scaling,
  a base pivot, import settings with levels of detail and a collider, and
  convex hulls. It reports the edges and texture memory before and after. Its `issues` lists every problem left with the
  option that fixes it, and `next` gives the command to run next.
- `bake model --project front=a.png --project left=b.png`: the base colour
  painted from concept images onto the UVs, one texture per material
  beside the model, with each material's coverage.
- `bake tall.ogt --from chair.ogt`: a variant takes its baked base's maps
  part by part, with no new images. Parts that grew read the nearest
  region of the same part, at the same texel density; the report gives
  each part's `transferred` and `extended` share.
- `repair`, `decimate`, `unwrap`, `collide`, `place`, `export`, `rig`:
  each step on its own.

## 8. Pitfalls

- The ids count every part made, including copies and mirrors, in order.
  Look them up with `og model build` before naming them.
- A mirror plane is a frame. Its `z` is the plane's normal, and its axes
  must be right-handed.
- A part with a `map` needs UVs: start the program with `{"op": "uvs"}`.
  `og model bake` and an `atlas` give the program its UVs themselves.
