<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `art-review` guide: Art judgment

Art judgment for an Origin Engine game. Choosing the style from the premise, requesting every asset through the art brief, the cook's conformance, and reviewing frames and models with measured thresholds. The engine measures; this guide says what is good enough.

The API: `og api show art`. The style catalogue is the engine package's
`knowledge/looks/catalogue.json`: every built-in look with its premise fit, palette
rules, asset specs, three checks a critic can judge on a frame, and the
traits that look like defects but are the style (`intended`).

<a id="guide-art-review-1"></a>
### `art-review §1` Choosing the style

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

<a id="guide-art-review-2"></a>
### `art-review §2` Every asset through the brief

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

<a id="guide-art-review-3"></a>
### `art-review §3` Conformance and its measures

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

<a id="guide-art-review-4"></a>
### `art-review §4` Reviewing frames

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
- **Nothing generic.** A frame that could be any engine's default scene
  (grey ground, default sky, uniform lighting) is not done.

A frame that fails is fixed and re-captured, never recorded as a
reference.

<a id="guide-art-review-5"></a>
### `art-review §5` Reviewing a model against a reference

For a model built or generated from a reference image, capture it at the
reference's angle and compare: `og compare reference.png render.png
--mask mask.png --sheet sheet.png`.

- Silhouette IoU at least 0.90 at the reference's view; below 0.80 the
  proportions are wrong, not the details.
- ΔE00 per mask region at most 10, and at most 5 on the regions that
  carry identity (face, emblem, signature colour).
- Look at the sheet as well as the numbers: an IoU of 0.93 with the arms
  in the wrong place is still wrong.
- Review the turntable (front, side, back, three-quarter) before
  accepting; a model built to one view collapses at the others.

<a id="guide-art-review-6"></a>
### `art-review §6` The art contract's resources

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
  `og style check` are docs/art.md §2.3 to §2.5.
