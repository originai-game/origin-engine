# Art judgment

Art judgment for an Origin Engine game. Choose a style from the premise, request assets through its brief, inspect cook conformance, then review the actual frames and motion. Measurements help locate problems; neither conformance nor similarity to a previous frame proves quality.

The API: `og api show art`. The style catalogue is the engine package's
`knowledge/looks/catalogue.json`: every built-in look with its premise fit, palette
rules, asset specs, three checks a critic can judge on a frame, and the
traits that look like defects but are the style (`intended`).

## 1. Choosing the style

The creation step already picked a look from the premise; question it
once, early, while nothing is drawn in it.

- Read the catalogue entry of the create block's `look`: its `premise`
  fields say which games it suits, `look.views` the cameras it draws
  for, `look.support` whether every primitive exists.
- One style per game. Mixing a pixel UI over a painterly world, or
  realistic props among low-poly characters, reads as unfinished, not
  eclectic.
- Pick by what the player must read at a glance: a busy action game
  needs flat, high-contrast shapes; a slow exploration game can afford
  texture and atmosphere.
- A `content` support style needs authored art (`look.authored` names
  it); without that art it is a worse look, not a cheaper one.

## 2. Every asset through the brief

Every image, sprite sheet, texture or model request goes through
`artBrief` (`node_modules/@origingame/engine/knowledge/scripts/brief.ts`):
`readLook(gameDir)`, then `artBrief(look, kind, subject, { references })`.
The brief carries the look's palette, sizes, line and shape language, so
the generator is asked for what the cook would otherwise force.

- Pass the game's accepted assets as `references` from the second asset
  on; the first accepted asset sets the bar for the rest.
- One subject per request. A sheet of unrelated props comes back in
  inconsistent scales and palettes.
- Ask for the size the brief gives (`brief.image`, `brief.model`), never
  larger "to be safe": the cook conforms to the contract and the extra
  detail turns into noise.
- UI art (frames, icons, key art) uses the UI kinds with their aspect,
  safe area and focal point; a generic image cropped into a slot loses
  its subject.

### From generated assets to a reviewed frame

Generation is an authoring step, not a game-time loader. Use the brief
with the asset source or generation tools available in the authoring
environment; this engine neither calls an image/model service nor supplies
its credentials. The same path applies to commissioned or licensed art:

1. **Obtain the source.** Request key art with room for the title/actions,
   a transparent logo, icons legible at their displayed size, and models
   with the look's proportions and face budget. Inspect the returned image
   or model, not just the request or its task status. Keep accepted assets
   as references for the next request.
2. **Save real files.** Download the image bytes and final GLB/glTF under
   `assets/`, for example `textures/title.png`, `textures/logo.png`,
   `textures/icon.png` and `models/subject.glb`. Keep any glTF sidecar
   buffers/textures and asset licences. A task ID, a preview page or a
   remote URL is not a source file that the cook can import.
3. **Cook and inspect.** Run `og cook --report`; resolve import errors and
   review conformance changes before accepting the output (§3). Keep the
   sources, not edits to `cooked/`. Use `og model` (`model`, `loading §2`)
   for scale, orientation, mesh and material inspection or processing.
4. **Load, then place.** Use paths relative to `assets/`, check each
   `og.load.texture` or `og.load.gltf` result, and attach the resulting
   handles. `TitleScreen.keyArt` and `.logo` take textures (`screens §1`);
   an `Image` takes its texture in `src`. A model becomes visible through
   `world.instantiate` or an `og.Model` component, not by loading alone.
5. **Capture the game.** Run with a renderer at the target sizes and
   capture the actual screen or play camera (`captures §1`, `testing §3`).
   In this repository, visual acceptance uses Chrome or Vulkan on the
   NVIDIA L4; a headless load check or a generator's preview proves no
   rendered quality. Capture motion too for animated content.
6. **Review and repeat.** Check title crops and text safe areas on phone
   and desktop, logo/icon transparency and legibility at their real size,
   and the model's front, side, back and play view under the game's light.
   Fix composition in the source/request, import orientation in its
   settings, placement in the game, or lighting in the look, then cook and
   capture again. Do not replace a missing asset with a zero handle and
   call that final art. A theme supplies controls, not the game's imagery.

### A cook/load/place recipe, not an art preset

These tiny SVGs and a single-triangle glTF are input fixtures, not finished
art. Replace them with obtained assets at the same paths. The gate cooks
the files and runs the recipe twice, checking that each texture loads and
the model is instantiated; it does not judge the frame or call a generator.
The title and image APIs accept these same texture handles after loading.

```ts recipe ticks=2
import { og } from "@origingame/engine";
import { Image, View, mount } from "@origingame/engine/ui";
import { TitleScreen } from "@origingame/engine/ui/screens/title.tsx";

export default og.game({
  start(world) {
    const keyArt = og.load.texture(world, "textures/title.svg");
    const logo = og.load.texture(world, "textures/logo.svg");
    const icon = og.load.texture(world, "textures/icon.svg");
    if (!keyArt.ok) throw new Error(keyArt.reason);
    if (!logo.ok) throw new Error(logo.reason);
    if (!icon.ok) throw new Error(icon.reason);
    og.assert.always(keyArt.value !== 0 && logo.value !== 0 && icon.value !== 0, "all art slots have loaded textures", {});
    mount(world, TitleScreen, {
      keyArt: keyArt.value, logo: logo.value, name: "", version: "", legal: "",
      background: () => View({ class: "absolute bottom-4 left-4", children: Image({ src: icon.value, class: "size-8 object-contain", label: "Asset fixture" }) }),
    }, { layer: "menu" });
    const model = og.load.gltf(world, "models/subject.gltf", {});
    if (!model.ok) throw new Error(model.reason);
    const instance = world.instantiate(model.value, { name: "subject", position: { x: 2, y: 0, z: -4 } });
    if (!instance.ok) throw new Error(instance.reason);
    const placed = world.get(instance.value, og.Model);
    og.assert.always(placed.ok && placed.value!.asset === model.value, "loading is followed by a placed model", {});
  },
});
```

```text recipe file=assets/textures/title.svg
<svg xmlns="http://www.w3.org/2000/svg" width="160" height="90"><path fill="#142738" d="M0 0h160v90H0z"/><path fill="#529cbd" d="M0 90L32 37 70 90z"/></svg>
```

```text recipe file=assets/textures/logo.svg
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="50"><path fill="#f0ca73" d="M10 40V10h12v18h80v12z"/></svg>
```

```text recipe file=assets/textures/icon.svg
<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><path fill="#f0ca73" d="M16 2 29 16 16 30 3 16z"/></svg>
```

```json recipe file=assets/models/subject.gltf
{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],"buffers":[{"byteLength":36,"uri":"data:application/octet-stream;base64,AACAvwAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAQEAAAAAA"}],"bufferViews":[{"buffer":0,"byteLength":36}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[-1,0,0],"max":[2,3,0]}]}
```

## 3. Conformance and its measures

The cook conforms every asset to the look's art contract (palette,
pixel grid, texel density, face budget, normals, line). `og style check-project`
measures every cooked asset and the title, play and results screens
against the contract and lists each opt-out. Hold assets to:

- Palette hit rate at least 95 % for a palette-strict style (the pixel
  styles), at least 80 % otherwise. Below that the asset was generated
  off-palette; regenerate it with the brief rather than accepting the
  cook's remap.
- Texel density within 2× of its neighbours on screen; a sharp prop
  beside a blurry wall reads as a placeholder.
- Faces within the look's budget; a model far under it in a detailed
  style reads as a blockout.
- Every opt-out is a decision written down: an asset that opts out of the
  palette or grid needs a reason the user would accept.

## 4. Reviewing frames

Frames come from a run with a renderer (testing §3). Review at the
target size (a phone game at 390×844), at the play camera, in motion as
well as still (`og render` a clip):

- **Read the player first.** The player and anything that can hurt them
  must separate from the ground: `og frame measure <png> --region <player>
  --against <ground> --metric contrast` at least 3:1, and high-pass energy
  of the ground no more than about 1.5× the player's (`--metric
  highpass`), or detail buries the subject.
- **Value structure.** Squint (`--metric mean` over large regions): the
  play area keeps a clear light and dark ordering, with the focus of the
  screen brightest or most saturated.
- **The style's checks.** Judge each of the catalogue entry's three
  checks on the frame; do not count its `intended` traits as defects.
- **Consistency.** Line width, outline colour, light direction and
  shadow softness agree across characters, props and UI.
- **Facing.** Each moving model's nose points along its travel: a ship
  flying up the screen shows its nose up, a foe diving at the player
  points at the player, a walker leads with its face. A model whose nose
  is off is fixed in its import settings (`front`/`up`, loading
  §2), checked with `og model preview --axes`, not by a turn in
  game code.
- **Nothing generic.** A frame that could be any engine's default scene
  (grey ground, default sky, uniform lighting) is not done.

A frame that fails is fixed and re-captured, never recorded as a
reference.

## 5. Reviewing a model against a reference

For a model built or generated from a reference image, first align the
camera, framing, pose and light, then inspect the silhouette, proportions,
identity details, materials and readability at the play camera. A model
built to one image can collapse from the side: review front, side, back
and three-quarter views, and a clip if it moves.

`og compare reference.png render.png --mask mask.png --sheet sheet.png`
locates differences. Silhouette IoU measures overlap; ΔE00 measures colour
distance in a mask. Neither determines whether the model is good: matching
one outline can hide missing depth, and a deliberate pose or lighting
change can lower similarity without lowering quality. Inspect the images
and name the actual defect before changing the model. A previous engine
frame is a record, not an artistic target to preserve.

## 6. The art contract's resources

A look sets seven resources beside its render ones (docs/art.md §2.1):
ArtPalette (the palette texels sit on), ArtPixels (one pixel size for 2D
art), ArtTexels (one texel density for 3D surfaces), ArtShape (the models'
shape language), ArtLine, ArtMotion (the step of motion) and ArtCamera. A
style is the assets, not a filter on the frame: the render resources treat
the frame, these say what the assets themselves must be.

- Every field is no constraint at its zero value (0, false, the first
  variant, an empty list), so a look without these blocks, and every game
  made before them, cooks and draws as before.
- The built-in presets (templates/looks/<slug>.ogt) carry each catalogue
  style's values; a game's look overrides them field by field, and
  project.ogt's `look` names the look whose contract the cook reads.
- Game code reads the contract too: a sprite snapping to ArtPixels.unit, a
  spawner picking colours from ArtPalette.
- Writing them changes neither the frame nor the simulation. The cook's
  conformance (§3), the run-time defaults taken from them and
  `og style check-project` are docs/art.md §2.3 to §2.5.

## 7. Reviewing effects

Capture an effect's first frames, its impact and its last frames (captures §6), at the play camera and at the target size, over the game's real ground and characters as well as on the preview's dark stage. On each frame, look for:

- **Shapes that show the quad.** A soft glow or smoke puff must not read as a square or show its sprite's straight edges; a sprite flat on the ground must not be cut by it into steps or half-buried rings.
- **Blown-out colour.** Overlapping additive sprites must not burn to flat white over a large area; the impact may be bright, but it keeps a colour and a falloff.
- **Timing.** It appears on the tick it is triggered, peaks at its impact, and fades out instead of vanishing; a looping effect has no visible seam.
- **Scale and reading.** It is sized to what caused it (a sword hit is smaller than a fireball's explosion) and reads against the ground, and it never hides the player or a threat for longer than its impact.
- **Agreement.** Its colours and shapes belong to the game's style and to the other effects: one set of sparks, one kind of smoke.

An effect that fails is fixed in its files and previewed again. A warning in its preview is fixed first.

## 8. Route a visible or audible problem to its mechanism

Inspect the failing moment in the actual game before adding a generator
or changing a renderer. Keep the seed, command sequence, play camera and
target size so the same case can be replayed after the edit. Separate a
missing asset, a failed load, a missing attachment and a drawn-but-wrong
result (`start §5`, `captures`).

| Observed problem | Inspect and change |
|---|---|
| Title art or logo is absent, or an icon is a placeholder | Source bytes, cook report, load Result and non-zero texture attached to the screen/image (`§2`, `screens §1`, `loading`). |
| A distant ridge disappears or loses its shadow | Camera `far`, directional shadow distance, terrain detail and fog before replacing its height generator (`camera §1`, `render §3`, `world §2`; `og api find shadowDistance`). |
| Snow, a face or the whole scene loses detail to white or black | Material albedo in linear colour, direct/environment light, exposure and tone curve (`render §4–§7`). Inspect a fixed exposure as well as adaptation; do not retune every material to compensate for the meter. |
| A subject is cropped or too small, or a cut jumps | The active camera driver, aspect, shot framing, brain priorities and blends (`camera §1`, `§2`, `§5`). Use framing constraints rather than per-resolution magic offsets. |
| A character slides, moves twice, uses the wrong pose or loses its held prop | Clip/graph selection, root-motion ownership, retargeting and node-versus-joint addressing (`animation §3`, `§7`, `§8`). Review the motion, not only a posed thumbnail. |
| Music switches abruptly, dialogue is buried or a sound is silent | Loaded sound and source state, web input unlock, source fades, bus effects, ducking and mixer snapshots (`audio §1`, `§3–§5`). Listen to the transition; a deterministic command log is not an audition. |
| A UI finish loads but never appears | Attach a material to a primitive, then inspect parameters, subtree filters/mask/opacity and layer-budget warnings (`ui-materials §3–§4`). Backdrop blur is not a custom foreground material. |
| Smoke vanishes, a quad shows or an impact washes out | Inspect first/impact/last frames over both the preview stage and the game's ground; check texture alpha, blend, depth and look together (`§7`, `effects`, `captures §6`). Keep a renderer defect separate from an unconnected emitter. |

Fix the owning source, cook or reload it, replay the same case and inspect
new captures. Keep warnings and unchecked/missing-renderer results visible.
A changed frame is not a failure by itself; a black, clipped, broken or
unreadable result is not accepted because its measurements are close.
