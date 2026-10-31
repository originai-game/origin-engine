---
name: og-model
description: Building a 3D prop or stylised character for Origin Engine as code from one reference image, the way img2threejs does it — an intake that inventories every identity-defining detail, a spec, eight build passes (blockout, structure, form, material, surface detail, lighting, interaction, optimisation) each closed by a gate, the og-geom generators (latheGeometry, extrudeGeometry with og.Shape and og.Path2, sweepGeometry, csgGeometry, mergeGeometries, shapeGeometry, the polyhedra) and their recipes, the triangle and draw budgets, the `type model` asset the cook builds a finished prop into, and the deterministic review — the view and turntable commands and og compare's silhouette IoU, per-region ΔE00 and one reference-beside-render sheet. Use when a game needs a model that is not a downloaded glTF, when asked to rebuild an object from a picture, or when a procedural model looks wrong.
---

A model on Origin Engine can be TypeScript: a function that builds an
object from generators, every part an entity under one root. That code is
the source, diffable and reviewable, and it is built from a reference image
the way img2threejs builds a three.js factory: a spec first, then eight
passes from blockout to optimisation, each closed by a gate read off one
sheet. The engine does the mechanical half of every gate (fixed cameras,
the silhouette and colour numbers, the sheet), so the one judgment left is
yours: does this sheet pass.

The generators' fields are §8 and the `type model` asset §9 (every field:
`og api show og.render.extrudeGeometry` and the like); the rest of this
skill is how to build a model with them and how to prove it matches.

What is reachable from one image: hard-surface props (crates, lanterns,
tools, furniture, vehicles, weapons, machines) and stylised characters
read from a front or three-quarter view. What is not: photoreal likeness,
the exact dimensions of something seen at an angle, and the sides the
image does not show. Say so in the spec instead of inventing them, and ask
for another view when a hidden side matters.

## What works

This skill's lantern (§5) was built and reviewed through this loop on the
native host: the first pass's side and orbit views showed the handle
floating 0.18 m above the cap and the crest standing off the cap's slope,
neither visible from the front; the second pass moved the handle's ends
onto the cap and laid the crest on its 47° slope, and every view agreed.
`og compare` of a lamp's front view against itself gave IoU 1 and ΔE00 0,
against its back view (the mirror) IoU 0.24, and against the turntable's
first frame (the front from 20° up, in perspective) with `--align` IoU
0.74, the foot's ellipse and the arm's foreshortening outlined in red and
blue on the sheet.

## 1. The loop

```text
reference ─▶ intake (§2) ─▶ spec ─▶ pass n (§4) ─▶ views (§3) ─▶ og compare ─▶ sheet
                                       ▲                                        │
                                       └──── refine-code / refine-spec ◀────────┤
                                                                     continue ──▶ pass n + 1
```

1. **Intake** (§2): judge the reference, write the detail inventory and
   the region mask, list what the image hides.
2. **Spec**: one file in the project, `review/<model>/spec.md`, outside
   `assets/` (nothing cooks it): the parts and their hierarchy, each
   part's generator and material, the pivots, the critical features (at
   most five), the budget (§6) and the inventory's table. The spec is the
   authority; the code follows it.
3. **Build one pass** (§4), and only that pass: what a later pass owns
   stays rough.
4. **Render the views** (§3): the reference's view, the other three
   orthographic views and a turntable, at the reference's size.
5. **Compare**: `og compare` against the reference, with its mask from the
   material pass on. Read the numbers against the pass's gate, then read
   the sheet.
6. **Decide**, and write the decision with its numbers in the spec:
   - `continue`: the gate passes and no critical feature is wrong;
   - `refine-code`: the spec is sound and the build misses it;
   - `refine-spec`: the spec is wrong or too shallow (a part missing, a
     form classed as the wrong generator); fix the spec, then the code;
   - `request-input`: the image cannot answer the question (a hidden
     side, an ambiguous material);
   - `stop`: the target cannot be reached from this image; say why.

   At most three corrections per pass and six over the model: past them,
   stop and report what still does not match. A number that passes does
   not overrule a critical feature that is wrong on the sheet, and a
   feature that looks right does not overrule a number that fails.

## 2. Intake: the reference, the detail inventory, the mask

**The reference.** One object, whole in the frame and not cropped, on a
plain backdrop (the silhouette is keyed from the border), lit evenly
enough that the materials read, from a known view (front, side or three
quarters). A cut-out with transparency is best: its alpha is the
silhouette. Refuse, or ask for another, when the object is cut off, mostly
occluded, motion-blurred or shot with a wide lens close up (its proportions
are not the object's).

**The detail inventory.** Split the reference into a 3 × 3 grid and list,
zone by zone, every detail that makes the object this object: the
silhouette's breaks, joints and seams, repeated systems (bolts, teeth,
ribs, slats, rivets), relief (bevels, grooves, panel lines, chamfers),
material regions and markings. Name each in 3D terms and give it a home:
a part and the generator that makes it, or a material region; never prose
alone. Mark the critical ones (at most five, the features a person would
name to recognise it) and the pass that builds each:

| Zone | Detail | In 3D | Part, generator | Pass |
|---|---|---|---|---|
| top centre | arched handle, thinner at its ends | swept circle, scale 0.6 → 1 → 0.6 | `handle`, sweepGeometry | 3 |
| middle | four glazed windows | box minus two crossed boxes | `frame`, csgGeometry | 2 |
| bottom | four rivets round the foot | one sphere, four placements | `rivets`, mergeGeometries | 5 |

Write down what one view cannot show (the back, the underside, the depth
of a part seen face on) and the confidence of each region, and decide each
from the object's kind, not from nothing: a lantern's back is its front.

**The region mask.** A PNG the reference's size, drawn over it: one flat
colour per material region (brass, iron, glass), black where nothing is
judged, no antialiasing (a blended edge is a region of its own; at most 64
colours). `og compare` names each region by its hex, so keep the legend in
the spec: `#c08030 brass`, `#202024 iron`.

## 3. The review frames: views, turntable, compare

**Views.** The command channel frames one entity's drawn bounds (its
og.Parent children with it) from a fixed direction, in place of og.Camera,
which it never writes:

| Step | Camera |
|---|---|
| `view:<entity>:front` | orthographic from +Z looking down -Z, +X right: the face of a model that faces +Z (the glTF and three.js convention) |
| `view:<entity>:back` | orthographic from -Z, +X left |
| `view:<entity>:side` | orthographic from +X, +Z left: a model facing +Z shows its left side |
| `view:<entity>:top` | orthographic from +Y, the front at the bottom |
| `view:<entity>:orbit:<degrees>` | a 30° perspective from 20° above the horizon, the azimuth from the front towards +X |
| `view:off` | og.Camera again |
| `turntable:<entity>:<n>[:<prefix>]` | n orbit frames 360/n apart (at most 72), all of the next tick, written `<prefix>-00.png` on; the view before comes back |

The view holds until `view:off` and frames what the entity draws each
frame, with a tenth of the frame free on each side. Build the model
facing +Z with y up, its feet on y = 0, so front means front. Its
og.Mesh and og.Model parts are framed; a skinned model's posed parts and
sprites are not.

Render at the reference's size, so the frames compare without
resampling, and give og.Camera a backdrop the object does not share
(Camera.background), so the silhouette keys cleanly:

```sh
og run . --headless --size 512x512 --commands "0.1:view:lantern:front;0.1:screenshot:review/front.png;0.1:view:lantern:side;0.1:screenshot:review/side.png;0.1:view:lantern:top;0.1:screenshot:review/top.png;0.1:view:lantern:back;0.1:screenshot:review/back.png;0.2:turntable:lantern:8:review/spin;0.3:quit"
```

The frames land under `og-out/` at the paths named. Every view of one
tick shows the same world, so a script can take all of them at once.

**Compare.**

```sh
og compare refs/lantern-front.png og-out/review/front.png --mask refs/lantern-front.mask.png
```

prints, for a two-region mask:

```text
silhouette IoU 0.9312 (512x512: reference 61210 px, render 63004 px, both 58450 px)
ΔE00 over the overlap: mean 6.84, p95 18.20
region     pixels     both    IoU  ΔE00 mean   p95
#202024     40122    37610 0.9133       4.10 11.52
#c08030     12090    11211 0.8890       9.75 21.04
```

- **Silhouette IoU** is the object's pixels in both over the pixels in
  either. The silhouette is alpha ≥ 128 when an image has transparency,
  else every pixel more than `--key` ΔE00 (8) from the backdrop (the median
  of the border); raise `--key` for a noisy photo backdrop.
- **ΔE00** is the CIEDE2000 difference of the colours over the pixels both
  cover, per mask region (each with its own IoU) or over the whole overlap
  without a mask: mean and 95th percentile. Under 2 is the same colour to
  the eye, 2 to 5 close, 5 to 10 a visible difference, above 10 another
  colour. The render's light is part of its colour: compare under light
  like the reference's, and read ΔE as a material number only once the
  lighting pass is settled.
- **The sheet** is one PNG beside the render (`front.sheet.png`, or
  `--sheet <path>`): the reference, the render as compared, and the two
  silhouettes overlaid (grey both, red the reference only, so missing;
  blue the render only, so extra), labelled with the ΔE00 and the IoU.
  Read it whole before deciding.
- The render is compared in the reference's frame: resampled when the two
  have the same shape, refused when not (render at `--size` of the
  reference's). `--align` scales the render's silhouette box to the
  reference's and centres it, for a reference whose framing is not the
  view's (a photo with its own margins); the framing then no longer counts
  against the IoU, and proportions still do. `--json` prints the numbers
  as one object.

**What the numbers cannot see.** The IoU reads the outline only: a model
with a feature missing inside its outline (a window, an eye, a panel
line) scores the same. A straight cone in roughly the right place passes
a curved horn's outline. One view says nothing of the depth. So every
pass reads the side, top and turntable frames too (a part that floats, a
hole through the body, a hat at the hip show there and only there), the
mask's regions measure inside the outline, and the critical features are
judged one by one on the sheet.

## 4. The eight passes and their gates

Build in order; each pass touches only its own layer. The gates are
defaults for a prop in a front reference with `--align`; raise them for a
hero prop, and write the gate the spec uses.

| # | Pass | Builds | Gate (numbers, then the sheet) |
|---|---|---|---|
| 1 | Blockout | the primary volumes as boxes, cylinders and lathes at their size and place; the root at the object's base, facing +Z | IoU ≥ 0.80; the bounds' width, height and depth in the reference's proportions within 5 %; nothing floats in the side and top views |
| 2 | Structure | every part of the inventory as its own entity under the root (og.Parent), its pivot where it turns or opens; repeated systems as one part placed many times | IoU ≥ 0.88; every part the spec names answers `get:<part>`; each critical feature is present on the sheet |
| 3 | Form | the outlines refined: lathe profiles, sweeps with their taper and twist, extrusions with their curves and bevels, the cuts | IoU ≥ 0.92 (≥ 0.95 for a hero prop); the turntable shows no hole, gap or floating part; a curve claimed in the spec reads as a curve |
| 4 | Material | a material per mask region: colour, roughness, metalness, emissive | per region ΔE00 mean ≤ 10 (≤ 6 on a critical region), p95 ≤ 20; IoU not below pass 3's |
| 5 | Surface detail | the inventory's small details: grooves and holes (csgGeometry), rivets and bolts (mergeGeometries), plates and badges (extrudeGeometry, shapeGeometry), bevels | every inventory row ticked on the sheet; no region's ΔE00 worse than pass 4 by more than 1 |
| 6 | Lighting | the model under the game's light and look (og-taste picks it): the frame the player will see | no critical feature lost in shadow or blown out; ΔE00 recorded, not gated (the game's light is not the photo's) |
| 7 | Interaction | what moves: hinges and lids turn about their pivots, named parts answer a pick, colliders where the game needs them | a scripted run turns each moving part through its range with no part leaving its joint; `og.render.raycast` at a part answers that part's entity |
| 8 | Optimisation | the budget (§6): static parts sharing a material merged, segment counts lowered where the silhouette does not need them; a static prop written as a `type model` asset (§5) | within the budget; IoU not below pass 3's by more than 0.01; the turntable unchanged to the eye |

A gate that fails is a decision (§1), not a reason to move on. When the
number and the sheet disagree, the sheet's specific defect wins: write it
in the spec and fix that.

## 5. The generators: recipes

Pick the generator by the form, never by habit: a continuous organic form
is a lathe or a sweep, not a stack of boxes.

| Form | Generator | Notes |
|---|---|---|
| turned: vases, bottles, lamp bases, wheels, columns, domes | `og.render.latheGeometry` | a profile of points, x from the axis, y up; `segments` 24 to 48 for a hero prop, 12 to 16 for a prop |
| a flat profile with depth: gears, signs, keys, brackets, badges | `og.render.extrudeGeometry` with an og.Shape (holes in `holes`, as og.Path2) | the bevel is on by default (three.js's 0.2 thick): `bevelEnabled: false` for a crisp plate, or set `bevelThickness` and `bevelSize` to the edge you see |
| tapered and curved: horns, tusks, handles, tails, blades, cables | `og.render.sweepGeometry` along an og.path | `scale` and `twist` keys over the path's length, 0 to 1 |
| holes, notches, windows, pips, grooves | `og.render.csgGeometry` with `subtract` | both operands closed; low-poly operands, cut once at load; chain calls for more cuts |
| joined solids that must be one surface | `csgGeometry` with `union` | for a single closed mesh; parts that only touch stay separate parts |
| repeated parts: rivets, teeth, slats, legs, a symmetric half | `og.render.mergeGeometries` | one placement per copy; `mirror: "x"` for the other half |
| faceted: gems, rocks, low-poly spheres | `og.render.icosahedronGeometry` and the other polyhedra | `detail` 0 faceted, 2 or more round |
| thin flat plates, decals, a shape seen face on | `og.render.shapeGeometry` | one-sided: give its material `side: "double"` when seen from behind |
| pipes and rails through points | `og.render.tubeGeometry` | or with `path` to follow an og.path |

**Left and right.** A left and right pair is a reflection, never a
rotation: `mirror: "x"` (or a scale of -1 on x) negates x and nothing
else, and mergeGeometries turns the mirrored triangles back so they face
out. A model facing +Z has its own left at +X.

**Parts are entities.** Give each part its own entity under the root
(og.Parent), named as the spec names it: the review frames the root, a
pick answers the part, a lid turns about its own pivot, and the model can
be pulled apart to check it. Merge only what never moves and shares a
material, and only in the optimisation pass.

The lantern of the review above, every generator once:

```ts
import { og } from "@origingame/engine";
import type { World, Entity, Geometry, Material } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** A signal lantern, facing +Z, its foot on y = 0: every part its own entity under the root. */
export function buildLantern(world: World): Entity {
  const iron = must(og.render.standardMaterial(world, { color: { r: 0.12, g: 0.12, b: 0.13 }, roughness: 0.6, metalness: 0.8 }));
  const brass = must(og.render.standardMaterial(world, { color: { r: 0.78, g: 0.55, b: 0.22 }, roughness: 0.35, metalness: 1 }));
  const glass = must(og.render.standardMaterial(world, { color: { r: 1, g: 0.75, b: 0.35 }, emissive: { r: 1, g: 0.6, b: 0.2 }, emissiveIntensity: 2 }));
  const root = must(world.spawn({ name: "lantern" }));
  const part = (name: string, geometry: Geometry, material: Material, position = { x: 0, y: 0, z: 0 }) => {
    const e = must(world.spawn({ name, position, parent: root }));
    world.insert(e, og.Mesh, { geometry, material });
    return e;
  };

  // Turned forms: the foot and the cap, profiles turned round +Y.
  const foot = must(og.render.latheGeometry(world, {
    points: [{ x: 0, y: 0 }, { x: 0.42, y: 0 }, { x: 0.42, y: 0.06 }, { x: 0.34, y: 0.12 }, { x: 0.3, y: 0.2 }, { x: 0, y: 0.2 }],
    segments: 32,
  }));
  part("foot", foot, iron);
  const cap = must(og.render.latheGeometry(world, {
    points: [{ x: 0, y: 0.32 }, { x: 0.08, y: 0.3 }, { x: 0.36, y: 0.04 }, { x: 0.34, y: 0 }, { x: 0, y: 0 }],
    segments: 32,
  }));
  part("cap", cap, iron, { x: 0, y: 0.9, z: 0 });

  // Cuts: the housing, a box with a window bored through each pair of sides.
  const body = must(og.render.boxGeometry(world, { width: 0.5, height: 0.7, depth: 0.5 }));
  const pane = must(og.render.boxGeometry(world, { width: 0.34, height: 0.5, depth: 0.7 }));
  const quarter = og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, Math.PI / 2);
  const front = must(og.render.csgGeometry(world, { op: "subtract", a: { geometry: body }, b: { geometry: pane } }));
  const frame = must(og.render.csgGeometry(world, { op: "subtract", a: { geometry: front }, b: { geometry: pane, quaternion: quarter } }));
  part("frame", frame, iron, { x: 0, y: 0.55, z: 0 });
  const flame = must(og.render.icosahedronGeometry(world, { radius: 0.12, detail: 2 }));
  part("flame", flame, glass, { x: 0, y: 0.5, z: 0 });

  // A tapered sweep: the handle, an arc whose ends sit on the cap, thinner at the ends.
  const arc = must(og.path.create(world, {
    kind: "catmullRom",
    points: [{ x: -0.26, y: 1.0, z: 0 }, { x: -0.18, y: 1.4, z: 0 }, { x: 0.18, y: 1.4, z: 0 }, { x: 0.26, y: 1.0, z: 0 }],
  }));
  const ring = new og.Shape().absarc(0, 0, 0.025, 0, Math.PI * 2, false);
  const handle = must(og.render.sweepGeometry(world, {
    section: ring, path: arc, steps: 48,
    scale: [{ t: 0, value: 0.6 }, { t: 0.5, value: 1 }, { t: 1, value: 0.6 }],
  }));
  part("handle", handle, brass);

  // A flat profile pushed to depth: the crest, with a hole and a crisp
  // edge, laid on the cap's slope (47° back) so it sits on the cap.
  const crest = new og.Shape().moveTo(-0.12, 0).lineTo(0.12, 0).lineTo(0, 0.2).closePath();
  crest.holes.push(new og.Path2().absarc(0, 0.07, 0.035, 0, Math.PI * 2, true));
  const plate = must(og.render.extrudeGeometry(world, { shapes: crest, depth: 0.02, bevelEnabled: false }));
  const badge = part("crest", plate, brass, { x: 0, y: 0.96, z: 0.33 });
  world.set(badge, og.Transform, { quaternion: og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -0.82) });

  // A repeated part: four rivets round the foot, one draw.
  const rivet = must(og.render.sphereGeometry(world, { radius: 0.025 }));
  const rivets = must(og.render.mergeGeometries(world, {
    parts: [0, 1, 2, 3].map((k) => {
      const a = (k * Math.PI) / 2 + Math.PI / 4;
      return { geometry: rivet, position: { x: 0.36 * Math.sin(a), y: 0.09, z: 0.36 * Math.cos(a) } };
    }),
  }));
  part("rivets", rivets, brass);
  return root;
}
```

Keep the numbers of the spec in the code (the profile's points, the
heights, the taper's keys) and nowhere else, so a refine-code is an edit
of a number the spec names.

**One recipe per shape language.** A model is built in the game's style
from its first pass, not restyled at the end: read the look's `ArtShape`
(og-taste §12) before the blockout. The art brief prints it for the model
(`brief.ts prop "<subject>" --project .`, og-taste §1: the face budget,
`normals`, `voxel`, `bevel`, the texture size), and the spec takes its
numbers. The cook holds the contract on a `type model` asset and every
glTF anyway (decimation to the budget, flat or smooth normals, the bevel,
voxels: og-build §8), but a model built in its shape language from the
start loses nothing to that pass, while a smooth 10,000-triangle model cut
to 200 does.

| Shape language (styles) | The contract | Build with | Materials |
|---|---|---|---|
| **Faceted** (`faceted-low-poly`, `minimal-geometric`, `low-resolution-3d`) | `normals flat`, `propFaces` 50 to 500 | every generator at its lowest counts that still keep the silhouette: `og.render.icosahedronGeometry` with `detail` 0 for rocks, canopies and gems, `latheGeometry` with `segments` 5 to 8, `cylinderGeometry` with `radialSegments` 5 to 7, `extrudeGeometry` with `bevelEnabled: false` and `curveSegments` 2 to 4; facets placed where the light turns, never at random | `flatShading: true`, one flat colour per material from the palette, no map (or one small palette texture) |
| **Voxel** (`voxel-worlds`) | `voxel` (the cube edge), `normals flat` | one `og.render.boxGeometry` of the edge placed on the grid by `mergeGeometries` (every position a whole multiple of the edge, in the model's own space), one merge per palette colour; a turned part stays on the grid (rebuild its cells, never rotate the merge); or a `.vox` model from a voxel editor (og-build §7) | `flatShading: true`, one palette colour per merge |
| **Clay bevel** (`sculpted-clay`, `miniature-diorama`) | `bevel` (metres), `normals smooth` | soft volumes: lathes whose profiles round every corner with two or three extra points, `og.render.capsuleGeometry` for limbs, `extrudeGeometry` with `bevelThickness` and `bevelSize` at the contract's bevel and `bevelSegments` 3 or more, lumps of `sphereGeometry` merged; no knife edge anywhere | matte (`roughness` 0.6 to 0.9, `metalness` 0), saturated flat colours; a fingerprint or tool texture only where the style's checks ask for it |
| **Paper fold** (`paper-craft`) | `normals flat` | sheets, not solids: `extrudeGeometry` of an og.Shape outline at a card's thickness (`depth` 0.002 to 0.01, `bevelEnabled: false`), each panel its own part turned about its fold line (a pivot on the crease), or `shapeGeometry` for a single sheet; volumes are boxes and prisms folded from those panels | `flatShading: true`, `roughness` 1, one flat colour per sheet, `side: "double"` on single sheets |
| **Wireframe** (`neon-vector`) | a palette of black and 2 to 4 hues; the line is the shape | edges, not faces: `og.render.tubeGeometry` through the `points` of each line that draws the silhouette (`radius` 0.01 to 0.03, `radialSegments` 4 to 6), merged per hue; or a low-poly solid with `wireframe: true` for one-pixel lines (every triangle edge shows, so keep it to the polyhedra) | `unlit: true`, `emissive` at a palette hue with `emissiveIntensity` 2 to 4 so the look's bloom lights it; faces, where there are any, near-black |

The voxel and the wireframe recipes, as a factory's parts:

```ts
/** A voxel tree: cells on a 0.1 m grid (ArtShape.voxel), one merge per palette colour. */
function voxelTree(world: World, root: Entity, bark: Material, leaf: Material) {
  const edge = 0.1;
  const cube = must(og.render.boxGeometry(world, { width: edge, height: edge, depth: edge }));
  const cells = (pick: (x: number, y: number, z: number) => boolean, size: number, height: number) => {
    const parts = [];
    for (let x = -size; x <= size; x++) for (let z = -size; z <= size; z++) for (let y = 0; y < height; y++) {
      if (pick(x, y, z)) parts.push({ geometry: cube, position: { x: x * edge, y: (y + 0.5) * edge, z: z * edge } });
    }
    return must(og.render.mergeGeometries(world, { parts }));
  };
  const trunk = cells((x, y, z) => x === 0 && z === 0 && y < 8, 0, 8);
  const crown = cells((x, y, z) => y >= 6 && x * x + (y - 9) * (y - 9) + z * z <= 9, 3, 13);
  for (const [name, geometry, material] of [["trunk", trunk, bark], ["crown", crown, leaf]] as const) {
    const e = must(world.spawn({ name, parent: root }));
    world.insert(e, og.Mesh, { geometry, material });
  }
}

/** A neon gate: glowing tubes along the lines of its silhouette, one hue. */
function neonGate(world: World, root: Entity) {
  const glow = must(og.render.standardMaterial(world, { color: { r: 0, g: 0, b: 0 }, unlit: true, emissive: { r: 0.25, g: 0.94, b: 1 }, emissiveIntensity: 3 }));
  const line = (points: { x: number; y: number; z: number }[]) => must(og.render.tubeGeometry(world, { points, radius: 0.02, radialSegments: 5, tubularSegments: points.length * 4 }));
  const frame = must(og.render.mergeGeometries(world, {
    parts: [
      { geometry: line([{ x: -1, y: 0, z: 0 }, { x: -1, y: 2, z: 0 }, { x: 0, y: 2.6, z: 0 }, { x: 1, y: 2, z: 0 }, { x: 1, y: 0, z: 0 }]) },
      { geometry: line([{ x: -1, y: 1.2, z: 0 }, { x: 1, y: 1.2, z: 0 }]) },
    ],
  }));
  const e = must(world.spawn({ name: "gate", parent: root }));
  world.insert(e, og.Mesh, { geometry: frame, material: glow });
}
```

Judge it in the game's look (pass 6): a faceted model whose facets do not
read under the look's light has too many of them or facets in the wrong
places; a clay model with one hard edge reads as plastic; voxels off the
grid read as noise; a neon line too thin to bloom disappears at play
distance.

**The same model as an asset: `type model`.** A prop whose parts depend on
nothing the game computes belongs in an asset once its passes are
closed: an `.ogt` of `type model` under `assets/` (§9 has the
grammar and a whole example) names each part's generator by its
`og.render` name with the same desc fields, places it as
`mergeGeometries` places a part (`at { }` blocks for the copies of a
repeated part), gives it a `material` and hangs it from a named `pivot`,
the node a lid or a wheel turns about. `og cook` builds it into the
model blob a glTF cooks to: meshopt, levels of detail with their measured
errors, the raycast BVH, the nodes, repeated parts drawn instanced. It
refuses the model past project.ogt's triangle and vertex budgets (write
§6's row for the model there, so the budget is the cook's, not a
promise), and `og cook --report` lists its triangles, parts and draws.
Load it with `og.load.gltf(world, "models/lantern.ogt")` and
`world.instantiate`, or name it on a scene's `model` line; `og dev`
re-cooks an edit under the same handle. Keep the TypeScript factory while
a pass is open (a refine-code is an edit and a rerun), write the asset in
the optimisation pass, and review the asset's views against the factory's
with `og compare` before deleting the factory: a region that moved is a
part the asset places differently.

**A gray level painted from views** is a different job from a prop: a
blockout glb (y up; uvs optional, charted by xatlas when missing) whose surfaces get one texture painted by
an image model from cameras a player could stand at. `og bake projection`
does the engine half on the CPU (`init`, `plan`, `prepare` per view,
`project` the model's picture, `holes`, `export` a textured glb;
docs/design.md §Projection painting); the `level-texturing` skill
drives it (its `scripts/texture-level.ts`) and owns the model calls and
the review. Check the
exported glb lit, from the planned views and from one the plan did not
take.

## 6. Budgets

| Use | Triangles | Materials (draws) | Lathe and sweep segments |
|---|---|---|---|
| hero prop, seen close and turned in the hand | ≤ 20,000 | ≤ 6 | 32 to 64 |
| prop in the play space | ≤ 5,000 | ≤ 3 | 16 to 24 |
| background and repeated set dressing | ≤ 1,000 | 1 | 8 to 12 |
| stylised character | ≤ 15,000 | ≤ 4 | 24 to 32 |

- The game's look wins over this table: its `ArtShape.propFaces` and
  `characterFaces` (og-taste §12, printed by the art brief) are the budget
  the cook decimates to, so build under them, not under the row above. A
  character's `type model` says `conform character` at its top (a skinned
  glTF needs nothing), else the cook holds it to `propFaces`; a `type
  model` past its budget is built again with fewer segments before
  anything is decimated (og-build §8).
- Count with `og.render.geometryInfo(world, g).triangles` over the parts,
  and in a frame with `dump_render` (the main pass's triangles).
- A draw is a mesh and material pair: parts that never move and share a
  material merge into one with mergeGeometries in the optimisation pass,
  not before.
- csgGeometry's cost grows with the product of its operands' triangles
  where their boxes overlap: cut low-poly operands once at load, never in
  a system.
- `curveSegments` 8 to 12 is enough for an og.Shape's arcs at prop size; a
  silhouette that shows facets in the front view wants more, an inner hole
  seldom does.
- A model used many times in a scene is built once and its geometries
  shared: call the factory's generators once and give each copy its own
  entities with the same geometry and material handles.

## 7. Reporting

Say what each pass changed with its numbers (the IoU and ΔE00 before and
after, the parts added, the triangles), what still does not match, and the
sheet it was judged on. Never say done when the sheet says improved. A
model approximated, stylised or guessed at its hidden sides is reported as
such, region by region.

## 8. Shapes, extrusions, lathes and solids: the generators' fields

**Shapes, extrusions, lathes and solids.** Three.js's modelling classes
keep their names, parameters and defaults, so a three.js extrusion is a
paste. `og.Shape` and `og.Path2` are three.js's Shape and Path:
`moveTo`, `lineTo`, `quadraticCurveTo`, `bezierCurveTo`, `splineThru`,
`arc`, `absarc`, `ellipse`, `absellipse`, `closePath`, `holes` and
`getPoints(divisions)`. They flatten to points before they cross to the
engine, so a points array works wherever a shape does
(`og api show shapes`).
- `shapeGeometry` is a flat shape with holes, one-sided like a circle.
- `extrudeGeometry` pushes shapes along +Z (`depth`, `steps`) or along an
  og.path (`extrudePath`). `bevelEnabled`, `bevelThickness`, `bevelSize`,
  `bevelOffset` and `bevelSegments` round its edges, as in three.js.
- `latheGeometry` turns a profile of `points` round +Y (`segments`,
  `phiStart`, `phiLength`), for vases, bottles and columns.
- `polyhedronGeometry` takes `vertices`, `indices`, `radius` and `detail`.
  `icosahedronGeometry`, `dodecahedronGeometry`, `octahedronGeometry` and
  `tetrahedronGeometry` take `radius` and `detail`.
- `sweepGeometry` sweeps a 2D `section` along an og.path. Its `scale` and
  `twist` are keyed over the path's length from 0 to 1, for horns, tusks,
  handles and twisted columns.
- `mergeGeometries` places `parts` (each a geometry with `position`,
  `quaternion`, `scale` and a `mirror` axis) into one draw. Write an
  array as the same part at several placements.
- `csgGeometry` is `union`, `subtract` or `intersect` of two placed
  closed geometries. Chain calls for more operands.

Every result is a geometry like `bufferGeometry`'s (og-render §12), with a BVH, `physics`
and `dynamic`, and answered by `geometryInfo`:

```ts
const gear = new og.Shape().absarc(0, 0, 1, 0, Math.PI * 2, false);
gear.holes.push(new og.Path2().absarc(0, 0, 0.3, 0, Math.PI * 2, true));
const disc = must(og.render.extrudeGeometry(world, {
  shapes: gear, depth: 0.2, bevelEnabled: true, bevelThickness: 0.05, bevelSize: 0.05,
}));
const vase = must(og.render.latheGeometry(world, {
  points: [{ x: 0, y: 0 }, { x: 0.5, y: 0.1 }, { x: 0.3, y: 1 }, { x: 0.4, y: 1.4 }],
  segments: 32,
}));
const cube = must(og.render.boxGeometry(world, { width: 1, height: 1, depth: 1 }));
const ball = must(og.render.sphereGeometry(world, { radius: 0.15 }));
const dice = must(og.render.csgGeometry(world, {
  op: "subtract", a: { geometry: cube }, b: { geometry: ball, position: { x: 0, y: 0, z: 0.5 } },
}));
```

The generators and the CSG decide every cut with exact predicates and use
no platform trigonometry, so every host makes the same triangles. A CSG
operand must be closed (every edge shared by two triangles). Its cost
grows with the product of the two triangle counts where their boxes
overlap, so cut low-poly operands at load, not every tick.

Every generator above takes `uvRepeat` as `boxGeometry` does (its uvs
scaled, so a texture tiles). `bevelSize` left out (or null) is
`bevelThickness - 0.1`, as in three.js: 0.1 at the default thickness.

## 9. Models written as text: `type model`

**Models written as text: `type model`.** A prop made of these parts
belongs in an asset, not in `start`: an `.ogt` of `type model` under
`assets/` names each part's geometry by its `og.render` call, with the
desc's field names and defaults, places it as `mergeGeometries` places a
part, gives it a material and hangs it from a named pivot. `og cook`
builds it into the same model blob a glTF cooks to: meshopt, levels of
detail with their measured errors, the raycast BVH, the nodes. Load it
with `og.load.gltf(world, "models/clockwork.ogt")` and `world.instantiate`,
or name it in a scene (`model "models/clockwork.ogt"`), as any model.

```
type model
version 1
---
material "brass" {
  color 0.78 0.52 0.16       # linear, as glTF's baseColorFactor
  metalness 1
  roughness 0.32
}
material "glaze" {
  color 0.12 0.34 0.72
  roughness 0.3
}
pivot "axle" {               # a node: code and animation find it by name
  position 0 1.25 0
}
part "gear" {
  pivot "axle"               # hangs from the pivot (default: the root)
  material "brass"
  position 0 0 -0.1
  extrudeGeometry {
    shape {                  # og.Shape's calls, one a line
      moveTo 1 0
      absarc 0 0 1 0 6.283185307179586 false
      hole {
        absarc 0 0 0.28 0 6.283185307179586 true
      }
    }
    depth 0.16
    bevelThickness 0.04
    bevelSize 0.03
  }
}
part "bolt" {                # one mesh under three nodes: drawn instanced
  pivot "axle"
  material "brass"
  cylinderGeometry {
    radiusTop 0.06
    radiusBottom 0.06
    height 0.36
  }
  at {
    position 0.55 0 0
    quaternion 0.7071068 0 0 0.7071068
  }
  at {
    position -0.275 0.476 0
    quaternion 0.7071068 0 0 0.7071068
  }
  at {
    position -0.275 -0.476 0
    quaternion 0.7071068 0 0 0.7071068
  }
}
part "vase" {
  material "glaze"
  position -2.2 0 0
  latheGeometry {
    points 0 0 0.32 0.04 0.42 0.4 0.24 0.95 0.3 1.25
    segments 40
  }
}
part "horn" {
  position 2.2 0 0
  sweepGeometry {
    section {
      absarc 0 0 0.22 0 6.283185307179586 false
    }
    path {                   # og.path's lines: kind, closed, tension, point
      kind catmullRom
      point 0 0 0
      point 0.15 0.6 0
      point -0.45 1.35 0.15
    }
    scale 0 1                # t value, repeatable (twist the same)
    scale 1 0.05
  }
}
part "dice" {
  position 1.1 0.3 0.55
  csgGeometry {
    op subtract
    a {                      # an operand: a geometry block, placed
      boxGeometry {
        width 0.6
        height 0.6
        depth 0.6
      }
    }
    b {
      position 0 0 0.3
      icosahedronGeometry {
        radius 0.08
        detail 1
      }
    }
  }
}
```

- Geometries: `boxGeometry`, `planeGeometry`, `sphereGeometry`,
  `cylinderGeometry`, `coneGeometry`, `capsuleGeometry`, `torusGeometry`,
  `shapeGeometry`, `extrudeGeometry` (`extrudePath { }` a path block),
  `latheGeometry` (`points x y x y ...`), `polyhedronGeometry`, the four
  solids, `sweepGeometry`, `csgGeometry` (`a { }`, `b { }`) and
  `mergeGeometries` (`part { }` blocks), each with `uvRepeat u v`. A
  part holds exactly one; combine more with `csgGeometry` or
  `mergeGeometries`.
- Shapes: `moveTo`, `lineTo`, `quadraticCurveTo`, `bezierCurveTo`,
  `splineThru x y ...`, `arc`, `absarc`, `ellipse`, `absellipse` (angles in
  radians, then `true` for clockwise), `closePath`, `points x y ...`
  (setFromPoints) and `hole { }`. They flatten to the points og.Shape
  gives in TypeScript, bit for bit.
- Placement: `position`, `quaternion x y z w`, `scale`, `mirror x|y|z`,
  on the part or in `at { }` blocks, one node each. The same part at
  several placements is one mesh drawn instanced; a mirrored placement is
  baked into a mesh of its own.
- Materials: `color`, `opacity`, `metalness` (default 0), `roughness`
  (default 1), `emissive`, `emissiveIntensity`, `doubleSided`,
  `alphaTest`. A part without one draws glTF's grey default.
- `og cook --report` lists the model's `triangles`, `parts`, `draws` (the
  distinct meshes; instanced placements are one) and `bvhBytes`/`bvhShare`.
  Past `budgets.triangles` or `budgets.vertices` in project.ogt the cook
  refuses it, so lower `segments`, `curveSegments`, `steps` or `detail`.
  Import settings (`models/clockwork.ogt.ogt`, `model { lods ... raycast
  false }`) apply as to a glTF.
- `og dev` re-cooks it on an edit and the running game draws the edit
  under the same asset handle.
