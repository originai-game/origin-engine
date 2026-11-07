# The art contract: a game's style as a contract on every asset, not a filter on the frame

A look (`type look`) holds the frame's render resources and the UI style;
the art contract adds what a style asks of the assets themselves, so the
assets carry the style instead of a filter put over whatever the game has.
The platform's part (the style chosen at creation) is §4.

## 2. The design

A style is one contract, held at four places:

```text
            ┌────────────── the contract: the game's look (.ogt, one source) ──────────────┐
            │ the frame             ArtPalette  ArtPixels  ArtTexels  ArtShape              │
            │ ArtLine  ArtMotion  ArtCamera   ui   (preview: the catalogue)                  │
            └────────┬──────────────────┬──────────────────┬──────────────────┬────────────┘
                     ▼                  ▼                  ▼                  ▼
            1 produce             2 conform (cook)     3 run               4 check
            the contract          every asset made     the frame's own     og style check:
            compiled into each    to the contract,     treatment, sprite   the assets' and
            generator request     deterministically,   outlines, palette   the frame's
            (images, models,      sources untouched    swaps, pixel-art    numbers
            model recipes)                             rotation, camera
```

1 and 2 make the assets carry the style. 3 keeps what really is a
property of the frame (light, tone, fog, post) and adds the runtime
pieces a style needs on a sprite. 4 measures the style.

### 2.1 The contract: seven art resources in the look

The IDL module `art` (`crates/og-api/api/art.ogt`) declares seven
resources a look sets like the render ones (by reflected name,
`og-bundle/src/look.rs` `LOOK_RESOURCES`). Each field's zero value is "no
constraint", so a look without them cooks and draws as a plain look. They
are resources, so game code can read the contract too (a sprite snapping
to `ArtPixels.unit`, a spawner picking from `ArtPalette`). The IDL holds
the field lists; in outline:

| Resource | Holds | Read by |
|---|---|---|
| `ArtPalette` | `colors` (a palette asset, `@palettes/*` or the game's), `strict` (every opaque texel on a palette colour), `maxColors`, `rampSteps` (shades per hue), `accents` (colours reserved for signals), `dither` (none, bayer2/4/8, blueNoise) | conform 2D and 3D textures, check |
| `ArtPixels` | `unit` (logical pixels per world unit, 2D), `logicalHeight` (as `Stylise.pixelHeight`), `spriteSizes` (allowed cell sizes, e.g. 16, 32), `rotation` (nearest, cleanEdge, prerotated) with `rotationSteps`, `snap` | conform sprites, flipbooks, tile sets; runtime sprites; check |
| `ArtTexels` | `density` (texels per metre), `maxSize`, `filter` (nearest, linear), `mips` | conform 3D textures, check |
| `ArtShape` | `propFaces`, `characterFaces` (the face budgets), `normals` (smooth, flat, angle) with `normalAngle`, `bevel` (radius in metres), `voxel` (edge in metres, 0 none), `silhouette` (min feature, metres) | conform models, og-model recipes, check |
| `ArtLine` | `width` (logical px for sprites, px for meshes), `color`, `place` (outside, inside, both), `distanceFalloff`, `brush` | conform sprites (baked outline), runtime meshes (MaterialStyle outline), check |
| `ArtMotion` | `stepFps` (animation on 2s/3s: 12, 8), `maxFrames` per clip, `squash` (0..1 default squash/stretch for og.tween presets) | conform flipbooks and clips, runtime `AnimationPlayer.stepFps` default, check |
| `ArtCamera` | `projection` (perspective, orthographic), `fixedAngle`, `pitch`/`yaw` (isometric presets), `fovMin`/`fovMax`, `pixelPerfect` | runtime camera defaults (`og.camera` presets), check |

Each of the 37 presets (`templates/looks/<slug>.ogt`) sets these values
(the `looks` guide, `og api show looks`, says how a game uses them). A game's look `extends` a preset and
overrides field by field.

**The project names its look.** The cook needs the contract before game
code runs, so `project.ogt` has `look "looks/<id>.ogt"` (the same id
`GameDesc.look` takes; `GameDesc.look` unset falls back to it). `create`
writes both. A project without the line conforms nothing.

**Creation choices precede the art contract.** `og new --choices` records
`genre`, `view`, `look`, `mood`, `ui`, `platform`, `input`, `players`,
`lang` and `music` in a `create` block in `project.ogt`.
`packages/create/bin/choices.ts` owns the vocabulary and exports its JSON
schema and premise inference through `@origingame/create-game`.
`resolveCreationChoices` adds catalogue matching. These exports are
additive public contracts beside the art brief; existing signatures stay
unchanged. The UI choice defaults to the look's `ui` line but can override
it, and the mood to the look's own (§2.10), which the game's look then
names. Music still records intent only. See create's README for the
function signatures and `docs/platform-asks.md` for the creation UI ask.

### 2.2 Produce: every request carries the contract

`@origingame/create-game` and the knowledge package export a **spec
compiler**, `artBrief(look, kind, subject)`: from the flattened look and
the catalogue entry it returns the request for one asset of a kind
(`sprite`, `tileset`, `portrait`, `backdrop`, `prop`, `character`,
`texture`, `icon`): the prompt (the style's direction, the palette as
named colours and hex, the sizes, the line, the shape language, the
avoid list), the style's reference images (its preview, and the game's
own accepted assets once there are some) and the generator's parameters
(transparent, size; for 3D the face budget for remesh, the texture size
for retexture). It is a pure function with tests, written in TypeScript
in the knowledge package (`scripts/brief.ts`), so the platform and an
agent in a sandbox read the same.

- **The signature**, a public contract documented in the knowledge
  README's §The art brief: `artBrief(look: BriefLook, kind: ArtKind,
  subject: string, options?: { references?, metres? }): ArtBrief`.
  `BriefLook` is `{ extends, art, profile?, palette?, style?, preview? }`:
  the art blocks flattened over the `extends` chain, the preset slug the
  chain reaches, the palette `ArtPalette.colors` names (else the frame's
  `Stylise.palette`) as named colours, and the catalogue entry. `ArtBrief`
  is `{ kind, subject, style, medium, prompt, avoid, palette, references,
  image?, model? }`; `image` is `{ transparent, width, height, grid,
  tiling }`, `model` `{ faceBudget, textureSize, normals, voxel, bevel,
  filter, mips, metres }`.
- **Reading a look from disk**: `readLook(projectDir)` takes project.ogt's
  `look` line and `profile`, follows `extends` as the cook does (a slug is
  the game's own `assets/looks/<slug>.ogt` first, then the preset; a path
  another look under `assets/`), merges field by field, reads the palette
  file (`.hex`, `.gpl`, `@palettes/*`) and finds the entry and its
  preview. `presetLook(slug)` reads a preset alone, for the platform's
  style step before a game exists. The presets and built-in palettes are
  `templates/looks` in the repository and `presets/` in both published
  packages (copied at prepack, not mounted for the platform).
- **What it decides**: a prop or character of a 3D game is a model
  (`ArtShape.propFaces`/`characterFaces` as the face budget, the texture
  `ArtTexels.density` x its size to a power of two within `maxSize`); of a
  2D game a sprite. 2D cells are `ArtPixels.unit` x the size snapped up to
  `spriteSizes` (a character half as wide as tall, an icon the smallest
  cell, a portrait twice the largest, a tile set 4 x 4 one-metre tiles, a
  backdrop `logicalHeight` rows at 16:9), drawn at the whole multiple
  nearest 1024 px so the cook's dominant-colour downscale lands on the
  grid. Palette colours are named from their OKLCH (a `.gpl`'s own names
  kept); accents, strictness, `maxColors`, `rampSteps` and the dither are
  written out; the entry's prose (palette size and rules, asset specs,
  outline) stands in where the look's field is zero. So the same kind
  gives different requests: a `prop` is a 16 x 16 sprite drawn at 64x in
  `16bit-pixel` and a 200-triangle model with a 64-texel nearest texture
  in `faceted-low-poly`.
- **The re-export**: `@origingame/create-game` re-exports the functions
  and types from `bin/brief.ts`, a byte-for-byte copy its prepack writes
  and its tests hold equal; its package.json has `main`. On the command
  line: `bun <knowledge>/scripts/brief.ts <kind> "<subject>" [--project
  <dir> | --look <slug>] [--reference <path>]... [--metres <n>] [--json]`.

**The UI kinds** (docs/ui.md §3.3): `ART_KINDS`
continues, after the eight above and in this order, by the art-slot kinds of
docs/ui.md §3.3, named as og-ui's `SlotKind` (`crates/og-ui/src/slot.rs`):
`keyArt`, `capsule`, `logo`, `appIcon`, `characterSplash`, `fullBody`,
`bust`, `vnSprite`, `eventCg`, `itemIcon`, `abilityIcon`, `cardArt`,
`cardFrame`, `unitPortrait`, `levelThumb`, `songJacket`, `saveThumb`,
`menuBackground`, `menuPattern`, `banner`, `playerBanner`, `emblem`,
`achievement`, `emote`, `currency`, `mapGlyph`, `talentIcon`, `tutorial`,
`cosmeticPreview`, and the style's own art `styleFrame`, `styleButton`,
`ornament`, `iconSet`. They are additive: the signatures of
`artBrief`, `readLook` and `presetLook` are as above, `BriefOptions`
has the optional `aspect`, `cells` and `insets`, and `ArtBrief` the
optional `ui`.

- **`UI_KINDS`** (exported) gives each kind's spec: the aspects it is
  drawn at (the first the default; `fullBody`, `banner` and
  `playerBanner` also take a range), whether it is transparent or tiles,
  its focal point, its safe rectangle (where a title or controls lie over
  it: key art's left 40 %, a menu background's left 45 %, a splash's
  bottom fifth), its fit, the smallest size it must read at (an ability
  icon 24 px, a currency or map glyph 16), and for style art the
  nine-slice and its states (`normal`, `hover`, `pressed`, `disabled`,
  `focused` for a button). og-ui's tests hold the two declarations equal.
  Store capsules take Steamworks' exact sizes (2024): header 920 x 430,
  small 462 x 174, main 1232 x 706, vertical 748 x 896, library 600 x
  900, library hero 3840 x 1240.
- **What a UI brief adds**: `ui` is `{ aspect, aspects, focal, safe, fit,
  legiblePx, nineSlice: { insets, states } | null, cells, conform,
  generated }`, and the prompt gains an `Aspect`, a `Focal point`, a `Safe
  area`, a `Legibility` floor, and for style art a `Nine-slice` line (the
  corners keep their size, so the ornament goes there, the edges stay even
  and the centre plain for a label) and a `States` line (one cell per
  state, left to right on one sheet). `conform` names what the cook's
  conform pass (§2.3, the row of 2D textures, which UI images are) will
  do under the look: the strict palette mapping, the pixel grid and its
  dominant-colour downscale, the outline; the prompt says so in a
  `Conform` line, so the art is drawn for it.
- **Sizes**: the longest side 1024, the other by the aspect, to a
  multiple of 8 (key art 1024 x 576); a style button's five states share
  1024 (five cells of 200); a pixel-art look puts icons on its smallest
  sprite cell (an item of 1 x 2 cells is 16 x 32 in `16bit-pixel`), style
  art on its largest, and scenes on `logicalHeight` (180 when unset), drawn
  at the whole multiple nearest 1024 as for sprites. `logo` is the one kind
  whose prompt asks for lettering; `cosmeticPreview` is rendered from a
  viewport (docs/ui.md §3.3), and its brief describes the framing only.

Its callers are the platform's asset production (`producing-game-assets`,
mounted from origin-agent) and the game agent, which sends every asset
request through it (the `art-review` guide).

### 2.3 Conform: the cook makes every asset meet the contract

The cook reads the project's look and conforms each asset as it cooks it.
The source is never written; the cooked blob conforms, and the cook
report says what it changed per asset. Deterministic, as the rest of the
cook (same input, same bytes), and cached by the look's hash with the
asset's. An asset opts out with `conform off` in its `.ogt` (or a
`conform` block in `project.ogt` naming paths); every opt-out is listed
by `og cook --report` and `og style check`. A model says what it is to the
face budgets with `conform character` (held to `characterFaces`, as a
skinned one is) or `conform effect` (an effect's mesh, drawn through its
effect material: no face budget holds or simplifies it; `og vfx import`
writes it in the import settings of every mesh it makes).

| Asset | Conformed by | What it does |
|---|---|---|
| texture used by sprites, flipbooks, tile sets, UI images (2D) | `ArtPixels`, `ArtPalette`, `ArtLine` | resample to the pixel unit with a pixel-art downscale (a dominant-colour box filter, never bilinear), alpha to 1 bit, palette mapping by Oklab nearest with the palette's dither, ramp limiting per hue, the outline drawn into the cell (Pixelorama's OutlineInline, diamond/circle/square brush, outside or inside), an index texture beside it for runtime swaps, `rotation prerotated N` bakes N rotations with cleanEdge |
| painted 2D (paint and line styles) | `Stylise`'s paint and watercolor fields read as texel space | the Kuwahara, pigment pooling and canvas grain, then the watercolour wash (granulation, turbulence, rims, dry brush, wobble), applied in the texture's own texels, so the stroke follows the sprite, not the screen; the frame then draws neither the paint nor the watercolour pass (`fx::settings`), so a sprite is painted once and the look is tuned once |
| texture used by meshes (3D) | `ArtTexels`, `ArtPalette`, `Stylise`'s paint fields | resample to the density for the model's size, then (painted styles) the same texel-space paint as painted 2D on a model's own colour textures, so strokes follow the surface (not on a texture no model claims, nor on normal or data maps), palette mapping where `strict`, nearest filter and no mips where `filter nearest` (low-resolution 3D); every other colour texture held to `maxColors` and `rampSteps` and abstracted into flat regions (two Kuwahara passes, the asset's own Oklab variance cut with every texel counted alike (the filter has already smoothed the noise the 2D cut's activity weighting discounts, so what stays busy is a shape's border), and ramp merge, shades within Oklab 0.05 merged, then every region smaller than the texture's smallest shape, (3 × size/128)² texels, merged into its longest-bordering neighbour), so a flat look's ground and props are fields of colour, not photographs or speckle under its bands |
| model (glTF, `type model`, `.vox`) | `ArtShape`, `ArtTexels`, `ArtPalette` | decimate to the face budget (meshopt simplify, already a cook dependency, else quadric), flat normals or an angle split, bevel (a chamfer on hard edges), voxelise at `voxel` (the `.vox` importer's mesher) with colours from the palette, bake vertex colours where the style is untextured (faceted, minimal geometric), the outline hull's data for `ArtLine` |
| animation clips and flipbooks | `ArtMotion` | step the sampled clip at `stepFps`, drop frames beyond `maxFrames` with a warning |

Algorithms taken from Pixelorama (MIT) and cleanEdge (torcado, MIT) keep
their notices in `third_party/<name>/LICENSE` beside the ported code's
header; OmniScale is not taken
until its licence is read.

### 2.4 Run: what belongs to the frame, and a sprite's own

The look's frame part stays as it is, and it is the fallback: a preset
keeps its `posterize` and dither, so a game drawing the look over plain,
generated or runtime-loaded materials still gets pixel steps, bands and
tones. What the cook conformed is not stepped again: loading a texture
adds its own palette (`<id>#palette`) to the renderer's conformed
colours, and the quantise leaves a pixel whose drawn colour lies within
6 sRGB levels of one as drawn (unlit conformed art keeps its flat fields
through compression and filtering; a lit or fogged tone is quantised).
The drawn colour is the lit scene's before exposure and the tone curve,
so a tone-mapped look keeps its conformed art too, as the curve draws it.
The runtime pieces, all off at their zero value:

- **Sprite outline** (`Sprite.outline`, `outlineColor`, from `ArtLine`
  when unset and the look sets one): a pixel outline drawn by the sprite
  shader from the alpha at the logical pixel size, for sprites whose
  outline cannot be baked (scaled, tinted, generated at run time).
- **Palette swap**: a sprite or tile layer draws its index texture through
  a palette row (`Sprite.paletteRow`, a palette texture of N rows):
  team colours, hit flashes in palette, day and night ramps.
- **Pixel-art rotation**: `TextureFilter.cleanEdge`, a sampler
  mode in the sprite shader that rotates pixel art without blur or
  jaggies (cleanEdge, MIT).
- **Camera presets** from `ArtCamera`: the main camera's projection,
  angle and field of view default to the contract; `pixelPerfect` snaps
  the 2D camera (the `Stylise.pixelSnap` path).
- **Toon bands** (anime cel and the other toon presets): metals band in
  their own colour with a toon highlight, `MaterialStyle.toonAmbient` 0
  keeps the ambient flat under the bands, `toonSoftness` anti-aliases the
  band edges, and the preset's outline lies outside the silhouette
  (`outlinePlace`).
- **Mesh lines from `ArtLine`**: where the frame's MaterialStyle sets no
  outline, the contract's line outlines the meshes: its width, colour,
  `distanceFalloff` and `place` (`MaterialStyle.outlinePlace`: inside, or
  outside and both through a jump flood of the silhouettes).
- **`ArtMotion.squash`**: og.tween.squash's default impact (0.5 the
  preset's own, 1 rubber hose).

### 2.5 Check: the style measured

`og style check` (and a line in `og doctor --project`): per asset,
palette hit rate (share of opaque texels within ΔE00 2 of a palette
colour), distinct colours, pixel-grid alignment (texels per logical pixel
constant, no half pixels), texel density spread, faces against the
budget, normal smoothness (flat share), outline width found; per clip
(`ArtMotion`): a model clip's rate and held poses against `stepFps`, its
frames against `maxFrames`, a flipbook's frame durations as whole steps;
per frame (`og run --headless --commands` captures of every declared
screen: title, play and results first, then the rest of `screens {}` and
the `ui { screen }` entries, at each viewport of `--sizes`, each labelled
`<screen>@<viewport>` with its size and renderer tier): colour count,
palette hit rate, edge statistics, the camera it was captured with
(`og.Camera` through the probe) against `ArtCamera` (projection, pitch and
yaw with `fixedAngle`, the field of view's range). The catalogue's checks
and the style's `intended` traits go to a critic as the agent's step
(the `art-review` guide): the engine measures, the agent judges. JSON with
`--json`, carrying the look's flattened art blocks and the scenes no
capture script reaches. A reviewer measures a region of any of those
frames with `og frame measure`.
`og build --publish` prints its summary as warnings; it never refuses a
build (numbers cannot judge a style).

### 2.6 The catalogue: honest layers, previews, sample scenes

`looks/catalogue.json` (`schemaVersion` 3):

- `preview`: `previews/<slug>.webp` beside the catalogue, 960×540, one
  picture of the style as a game (`packages/knowledge/previews/`),
  checked present by the validator.
- `look.layers`: which layers the style needs, among `frame` (the look's
  render resources), `conform` (the cook's asset conformance), `runtime`
  (§2.4), `authored` (art no conformance can make: painted portraits,
  hand animation). `look.support` is derived: `full` when every
  needed layer but `authored` is implemented, `content` when `authored`
  is needed, `partial` otherwise with `missing` naming what.
- For a creation UI to filter and group by: `category` in plain terms
  (pixel art, retro 3D, hand-drawn and painted, ink and comic, toon and
  stylised 3D, low-poly, toy and handmade 3D, flat and graphic,
  realistic), `look.views` (the creation views it suits, within its
  profiles), `look.mood` (the mood its preset is drawn in, §2.10) and,
  for the `content` styles, `look.authored`: each art brief kind the
  style needs made and what it must be. `artBrief` writes that into every
  request of the kind (an `Authored` line) and `authoredArt(look)` lists
  them.
- A style's sample scene belongs to the game's art review, not the
  catalogue. The engine ships no sample-game gallery.

### 2.7 A style's painted art: the hand-off

A UI style's frames, buttons and ornaments are painted art, and the
engine never paints them. `og style art <pack>` writes what to paint,
checks what arrives and packs it. Fulfilment belongs to whoever owns image
generation: the platform's agent for a game, and an offline script for the
engine's own seed styles. `og` makes no network call, and no `og.*` member
generates art.

1. **Requests.** `og style art` completes `<pack>/art-requests.json` (what
   `og style new` writes: the style, look, premise, choices and palette).
   Each slot gets `file` (`art/<part>.png`), `size` (the cell in CSS px),
   `insets` (the nine-slice corners as fractions of the cell), `states`
   (a button sheet's cells, left to right: normal, hover, pressed,
   disabled, focused), `pixels` (the least to deliver: the 3× cell times
   the states), `dress` (the dressing each cell goes to, such as
   `Button.base[primary][hover]`) and `hash` (over the rest). A value
   already in the file wins, so a pack retunes a slot by editing it, and
   the hash moves with it.
2. **Fulfilment.** The fulfiller turns each slot into a prompt through
   `styleArtBriefs` (§2.2) and paints it, with the pack's concept sheets
   as references where it has them. It drops `art/<part>.png` on a
   transparent ground, at any size at least `pixels`, its states side by
   side. A button may arrive as one normal cell, and its other states are
   then derived. Into `art/provenance.json` it writes, keyed by the slot's
   name: `model`, `prompt`, `request` (the slot's `hash` when it was
   painted), and optionally `references`, `generation`, `date` and
   `insets` (the corners it actually painted, as fractions, when they
   differ from the request's).
3. **Verification and packing.** `og style art` refuses art without a
   provenance entry, reports art whose `request` is no longer the slot's
   hash as stale, and leaves a slot without art to the style's own
   dressing. `--check` writes nothing and fails on any of these. It then
   cuts each sheet into cells, trims them to one alpha box, and resamples
   each to its cell at 1×, 2× and 3×: Lanczos over premultiplied alpha,
   or for a pixel style the dominant colour per block, enlarged by whole
   pixels. It shelves the pieces below the style's atlas as it stood
   before any art (kept as `art/base.png`) and writes `atlas.png`,
   `atlas@2x.png` and `atlas@3x.png` with one layout. The style's `atlas`
   line names the densities (`atlas "atlas.png" W H 2 3`); each painted
   dressing gets an `authored` slice and its measured `tone`, and each
   ornament line its region. The UI draws from the density nearest at or
   above its device pixels per CSS px, and 3× above that (docs/ui.md
   §3.3). `art/sheet.png` (every piece at 2×) and `art/report.json` (what
   was packed from what, with the provenance) are for review, and the
   cook never bundles `art/`.

A rerun with nothing changed writes the same bytes, so a pack's art is
reviewed as a diff.

4. **Recolour.** Painted art keeps the colours it was painted in, so a
   game's copy of a pack (`og new`'s own style, `og style eject`) whose
   palette moves to the game's accent would still draw the pack's accent
   in every `authored` slice (cyber's yellow primary button under a blue
   palette). A style says which painted accents follow which palette role
   with `recolour "<colour as painted>" <role>` lines after its `atlas`
   line (every painted pack has `recolour "<its primary>" primary`); a
   dressing may set its own lines, or `recolour none`, over the style's.
   The cook then writes the atlas again beside the style, at every density
   the `atlas` line names (`<style>#atlas.png`, `<style>#atlas@2x.png` …),
   with each authored slice and ornament recoloured, points the cooked
   style at it and measures each recoloured frame's `tone` again, so a
   label still picks ink that reads on it. The recolour is the team-colour
   mask of strategy games (Warcraft III, StarCraft) and Photoshop's
   Replace Color, keyed on the art itself instead of a painted mask: a
   pixel within 30° of the accent's Oklab hue and with at least 45 % of
   its chroma is fully the accent (fading out to 50° and 20 %); it turns
   by the hue between the painted accent and the role's colour, scales
   its chroma and shifts its lightness by theirs, so a darker edge stays
   darker and the art's greys, blacks and other hues stay byte for byte.
   A palette that still has every accent as painted cooks nothing extra.
   For a change the hue shift cannot carry (gold leaf to steel, a brush
   stroke's texture), paint the slot again through the art brief: change
   the pack's `art-requests.json` palette and rerun steps 1–3.

### 2.8 The auto-rig's input layers

A character drawn in layers is rigged by the cook, with no editor step: a
`type rig2d` file names the art and a template, and the cook makes the
bones, the slots, the leg IK, the atlas and the template's clips
(`crates/og-cook/src/cook/autorig.rs`). This section is the contract the
art meets. `rigParts(template)` of `@origingame/create-game` gives the same
part lists as data (`packages/knowledge/scripts/brief.ts`, held equal to
the cook's templates by og-cook's `rig_parts_match_the_templates`).

```text
type rig2d
version 1
---
parts "knight.psd"          # a layered PSD (or PSB), or "knight/", a folder of PNGs
template humanoid-side      # humanoid-side, humanoid-front, quadruped, creature
pivot hand-r 212 340        # optional, repeatable: a part's joint, canvas pixels
unit 200                    # optional: pixels per world unit
filter nearest              # optional: pixel art
```

Any other line is refused (`autorig.rs` `build`).

**The templates and their parts** (`autorig/template.rs`). A part is
required or optional; each hangs from the first of its parents the art
has, and its joint is looked for at one end of it: its top, its bottom,
its middle (the root part) or wherever it meets its parent ("near"). A
part is rigid (drawn as it is, one bone), a limb (one bone, a mesh that
bends at its joints) or a chain of bones (a mesh that lags through them).

The two humanoid templates share their parts (the `humanoid!` list):

| Part | Needed | Hangs from | Joint at | Moves as |
|---|---|---|---|---|
| `pelvis` | required | (the root) | middle | rigid |
| `torso` | required | `pelvis` | bottom | limb |
| `neck` | optional | `torso` | bottom | limb |
| `head` | required | `neck`, else `torso` | bottom | rigid |
| `hair` | optional | `head` | top | chain of 2 |
| `ear-l`, `ear-r` | optional | `head` | near | rigid |
| `arm-upper-l`, `arm-upper-r` | required | `torso` | top | limb |
| `arm-lower-l`, `arm-lower-r` | required | the upper arm of its side | top | limb |
| `hand-l`, `hand-r` | required | the forearm of its side | top | rigid |
| `leg-upper-l`, `leg-upper-r` | required | `pelvis` | top | limb |
| `leg-lower-l`, `leg-lower-r` | required | the thigh of its side | top | limb |
| `foot-l`, `foot-r` | required | the shin of its side | top | rigid |
| `cape` | optional | `torso` | top | chain of 3 |
| `tail` | optional | `pelvis` | near | chain of 4 |
| `wing-l`, `wing-r` | optional | `torso` | near | chain of 2 |
| `weapon` | optional | `hand-r`, else `hand-l` | near | rigid |
| `shield` | optional | `hand-l`, else `hand-r` | near | rigid |

- `humanoid-side`: `-l` is the far side (drawn behind the body), `-r` the
  near side; the character faces right. Two-bone IK on each leg; a knee
  drawn straight bends forwards.
- `humanoid-front`: `-l` is the part on the image's left, `-r` on its
  right; the character faces the viewer. Two-bone IK on each leg; a knee
  drawn straight bends outwards. Its clips foreshorten each thigh (scaled
  to the hip's reach to its foot, 0.86 to 1.0) so a knee bends towards the
  viewer, keep the feet at their drawn angle (`Template.level`), and no
  standing clip moves a foot inwards of where it is drawn
  (`autorig/front.rs`), so boots drawn side by side, even a few pixels
  apart, do not slide over each other (og-cook's
  `seen_from_the_front_the_legs_never_cross_in_any_clip`).

`quadruped`: `-l` is the far side (drawn behind the body), `-r` the near
side; the animal faces right. Two-bone IK on each of the four legs.

| Part | Needed | Hangs from | Joint at | Moves as |
|---|---|---|---|---|
| `body` | required | (the root) | middle | limb |
| `neck` | optional | `body` | near | limb |
| `head` | required | `neck`, else `body` | near | rigid |
| `jaw` | optional | `head` | near | rigid |
| `ear-l`, `ear-r` | optional | `head` | near | rigid |
| `mane` | optional | `neck`, else `head` | near | chain of 2 |
| `leg-front-upper-l`, `-r`; `leg-back-upper-l`, `-r` | required | `body` | top | limb |
| `leg-front-lower-l`, `-r`; `leg-back-lower-l`, `-r` | required | the upper part of its leg | top | limb |
| `paw-front-l`, `-r`; `paw-back-l`, `-r` | required | the lower part of its leg | top | rigid |
| `tail` | optional | `body` | near | chain of 4 |
| `wing-l`, `wing-r` | optional | `body` | near | chain of 2 |

`creature`: `-l` is the part on the image's left, `-r` on its right; limbs
are numbered from the front (`limb-1`). No IK; the body squashes and
stretches, and the rigid parts on it keep their shape.

| Part | Needed | Hangs from | Joint at | Moves as |
|---|---|---|---|---|
| `body` | required | (the root) | middle | limb |
| `head` | optional | `body` | near | rigid |
| `eye-l`, `eye-r`, `ear-l`, `ear-r` | optional | `head`, else `body` | near | rigid |
| `limb-1` to `limb-8` | optional | `body` | near | chain of 3 |
| `tail` | optional | `body` | near | chain of 4 |
| `wing-l`, `wing-r` | optional | `body` | near | chain of 2 |

**Naming** (`autorig/parts.rs`). A layer's or file's name becomes a part
name trimmed, lower-cased, with `_` and spaces as `-` (`Arm Upper_L` is
`arm-upper-l`). Every name must be one of the template's parts: the side
suffixes are `-l` and `-r` exactly as the tables write them, and the
optional parts above are the only extras. A name starting with `_` (a
layer, a file, or a PSD group holding the layer) is left out, as are
hidden PSD layers (the cook's report lists them). A PSD group named for a
part merges the layers it holds into that part (composited in order,
normal blending), so a part may be painted in several layers. Each part is
drawn once.

**Layers and their size.** In a PSD (`.psd` or `.psb`, RGB or greyscale,
8 or 16 bits a channel; `cook/psd/layers.rs`) a layer may have any size
and place on the canvas; each part is its pixels as Photoshop shows them
over nothing (the layer's opacity, fill and enabled layer mask in its
alpha), trimmed to its pixels with any alpha. In a folder every PNG (a
`.png` file whose name does not start with `_`) is exported at the canvas
size, untrimmed, so each sits where it is drawn; the cook trims them. No
margin is needed, round a part or the character: the cook packs the
trimmed parts into the atlas 8 pixels apart and from its edges, and
bleeds each part's colour into that gap at zero alpha so no filtering or
mip level draws a fringe (`autorig/atlas.rs`). The atlas is at most
4096 x 4096. The ground is the lowest drawn pixel of any part, and the
rig's root stands on it under the root part's joint; by default a world
unit is the drawn height over the template's height (a humanoid 2 units,
a quadruped 1.2, a creature 1), and `unit` overrides it.

**Overlaps and pivots: how joints are found** (`autorig/skeleton.rs`).
A pixel is solid at alpha 128 or more.

- The root part's (`pelvis`, `body`) joint is the centre of its solid
  pixels.
- Every other part's joint is the centre of its solid pixels that lie
  over its parent's solid pixels, counting only those in its joint end's
  band: for a top or bottom joint, the part's top (or bottom) 35 % of its
  height, or half its narrower side when that is more; for a near joint,
  anywhere. So draw each part overlapping its parent where they join, as
  a shoulder lies over the torso.
- With no such overlap, the joint is at its end by convention (the centre
  of the solid pixels within half its narrower side of its top or
  bottom; for a near joint, its pixel nearest the centre of its parent),
  and that point must be within 2 % of the drawn height (at least 3
  pixels) of the parent's solid pixels, or the art is refused.
- `pivot <part> <x> <y>` (canvas pixels) sets a part's joint, overlap or
  not.
- A bone runs from its part's joint to the joint of the part it leads to
  (the forearm's, for an upper arm); a part at the end of its line (a
  hand, a head) points from its joint to the centre of its pixels, and a
  chain runs to its farthest pixel, split into equal bones.
- Where a limb continues into its next part (an elbow, a knee, a wrist,
  an ankle, the waist, the neck) the overlap turns at half the joint's
  angle, so the pixels there turn rigidly instead of pinching: draw that
  overlap round (a knee cap). A part that hangs from the trunk (a
  shoulder, a hip, a cape, a tail) hinges at its joint: draw its end
  round where the parent covers it.
- A leg drawn straight (within about a degree) has its knee put on the
  line from the hip to the ankle, and bends the template's way; a leg
  drawn bent bends as drawn.
- Upright: in the humanoid templates the head's joint is above the
  pelvis's, and in every template with legs each foot or paw is below
  the hips; otherwise the art is refused.

**Draw order.** The parts draw back to front in the art's order: a PSD's
layer order (bottom first; a part merged from a group at its first
layer's place), or a folder's file names sorted, so a `NN-` prefix
(digits, then `-`, `_`, a space or `.`) numbers the order and is dropped
from the part's name (`01-leg-upper-l.png` is `leg-upper-l`). A few clips
bring an arm in front for their length (the attack's striking arm, the
wave's hand). `rigParts` lists each template's parts in the order the
example art draws them (og-cook's autorig tests), the far or image-left
side first.

**Refusals name the layer and the file, and the fix** (`autorig.rs`,
`autorig/parts.rs`, `autorig/skeleton.rs`, `autorig/atlas.rs`): a name
that is no part of the template (`chars/hero.psd: layer \`Arm Uper R\``
or `chars/hero/13-arm-uper-r.png`, the nearest part suggested within
three edits, every part listed, and the `_` way to leave it out); a
second layer or file of one part (merge them, or group them under the
part's name); a part with no pixels; the required parts missing, each
with what it is and what `-l` and `-r` mean in the template; a PNG not at
the first PNG's size, or a folder with none; a PSD layer with layer
effects, a blend mode other than normal (or in a group with one), a
clipping mask, a vector mask, or an adjustment or fill layer, each with
the Photoshop menu that fixes it; a part that does not touch its parent
(how far its joint end is, and the `pivot` line that sets it); a part
whose parents the art lacks; a head drawn below the pelvis or a foot
above the hips; parts that overflow the atlas; a `pivot` for a part the
art does not have.

### 2.9 The audio brief: every cue and effect a game needs

The audio counterpart of §2.2. The knowledge
package's `scripts/brief.ts` exports `audioBrief(input: AudioBriefInput):
AudioBrief`, re-exported by `@origingame/create-game` beside `artBrief`; an
additive contract, kept stable like it (a change is a break with a
`changelog.json` breaking entry). It is pure and deterministic.

- **Input**: `{ music, mood, genre, premise? }`, the creation choices'
  values (`CREATION_CHOICES.music`, `.mood`, `.genre`; create-game's tests
  hold every combination accepted) and the premise as written.
  `readAudioChoices(projectDir)` reads the first three from project.ogt's
  `create` block. An unknown value throws naming the known ones.
- **Output**: `{ music, mood, genre, premise, direction, avoid, cues,
  sfx, presets }`. `direction` is what every request shares (the style's
  instruments and texture, the mood, the effects' character); `avoid` what
  none may contain (the style's, plus lyrics, copyrighted material,
  clipping, silence at a loop's ends).
- **`cues: MusicCueBrief[]`**: the title theme, the genre's cues (an RPG's
  town, field, battle and boss; a rhythm game's song and calibration loop;
  an idle game's loop and milestone stinger) and the victory and defeat
  jingles. Each has `id`, `role` (`title`, `explore`, `action`, `boss`,
  `calm`, `victory`, `defeat`, `stinger`), `prompt`, `bpm` (the style's
  tempo range at the mood's place in it, offset by the cue), `mode`,
  `beatsPerBar` 4, `bars`, `seconds`, `loop` (seamless at the bar line;
  jingles and stingers play once), `stems` (the style's layers for the
  play loops, sample-aligned so the music system can bring them in and out),
  `intensity`, `file` (`music/<id>.ogg`) and `format` (Ogg Vorbis, 48 kHz
  stereo, −16 LUFS, true peak −1 dBFS). A rhythm song's prompt asks for a
  strict tempo and a count-in, since the game judges presses on
  og.Music's clock at that bpm.
- **`sfx: SfxBrief[]`**: the four UI sounds, then the genre's events
  (jump, land, footsteps, coins for a platformer; swap, match, cascade for
  a match-3). Each has `id`, `event`, `prompt`, `seconds`, `bus` (`ui` or
  `sfx`), `variants` (takes, so repeats do not machine-gun), `preset`
  (the stand-in's id or null), `file` (`audio/<id>.wav`) and `format`
  (WAV, 48 kHz mono, peak −1 dBFS).
- **`presets: SfxPresetBrief[]`**: for each effect one of og cook's
  procedural presets fits (`SFX_PRESETS`, held equal to og-cook's
  `sfx::PRESETS` by the knowledge tests), the `type sfx` document to write
  to `assets/audio/<id>.ogt`, its seed from the id and its pitch and length
  from the style and mood (chiptune 1 and 0.9, orchestral 0.9 and 1.2; a
  dark mood 0.85, a bright one 1.08). A game plays the preset until the
  generated file exists, then switches the load to the `.wav`. An engine
  loop or wind ambience has no preset and waits for its file.
- **Command line**: `bun <knowledge>/scripts/brief.ts audio [--project
  <dir>] [--music --mood --genre] [--premise "<line>"] [--json]`.

The platform's music and sound generation (origin-agent) takes these
requests as they are: `docs/platform-asks.md` ask 26.

### 2.10 Mood: a parameter of the look

A look takes `mood "<word>"`, one of `bright`, `dark`, `warm`, `cold`,
`soft`, `high-contrast` (og-bundle's `MOODS`, the creation choice's words
in `packages/create/bin/choices.ts`). Every preset names the mood it is
drawn in (the catalogue's `look.mood`). A game's look that names another
mood asks the preset for it: the cook lays the mood's layer over the
preset its `extends` chain reaches, before the looks above it, so the
game's own fields still win (`og-cook`'s `cook::look`). A chain without a
preset lays the layer under its deepest look. The cooked look carries the
mood (`LookInfo.mood`).

The layers are data, `templates/looks/moods/` (its README), compiled into
the cook:

- `<mood>.ogt`, one for every style: the frame's grading (`PostEffects`'
  white balance, contrast, saturation, vibrance, colour balance and
  vignette, which the renderer bakes into its LUT with the look's own
  table), its light (`RenderSettings.exposure` and the ambient
  `environmentIntensity`) written as the mood reads over a neutral look.
  A number moves the preset's value by its distance from the field's
  default, clamped to the field's range: `dark`'s `exposure 0.72` takes a
  preset at 0.8 to 0.52. Any other value replaces the preset's.
- `<mood>/<slug>.ogt`, one style's layer in place of the generic one: the
  two palette-bound styles quantise every pixel to a fixed palette after
  the grading, so grading alone barely moves them; their layers swap the
  palette (`Stylise.palette` and `ArtPalette.colors`) for the mood's
  (`@palettes/handheld4-<mood>`, `@palettes/1bit-<mood>`). A posterized
  style's layer leaves out what posterize breaks: the vignette (its
  falloff steps into rings over a flat sky or ground; minimal-geometric
  and bold-comic-illustration in dark and high-contrast), and, under
  per-channel posterize, the shadow balance (bold-comic-illustration's
  dark sky, one ring per channel). muted-cinematic-3d's under-exposed,
  grey-fogged stage takes its own bright (a stop more light, a paler
  fog) and soft (a lower-contrast curve, lifted shadows, a paler fog).

A posterized look whose moods must keep its colours' hues names
`Stylise.posterizeMode tone`: the value and the tint (the linear channels
over the largest) are stepped apart, so light, exposure and a mood move a
colour's value and never its hue. Per channel (`channel`, the default), a
low-saturation colour flips hue as light pushes one channel across its
step before the others: minimal-geometric's grey-green ground read grey
in its own mood and mint, green or teal in the others, heavy-ink-shadows'
red-brown hills olive and pastel. minimal-geometric and
heavy-ink-shadows name `tone`.

A layer sets the frame's grading, light and palette only (`PostEffects`,
`RenderSettings`, `Environment`, `Fog`, `Sky`, `Stylise.palette`,
`ArtPalette.colors`); the art contract's sizes, shapes, lines and motion
never change with the mood. The art brief reads the mood and the palette
the layer names (§2.2): its prompt gains a `Mood` line, so generated
assets are lit and coloured for it. `og new --mood <word>` writes the
line into the game's look; with no mood asked, the game keeps its
preset's.

## 4. The platform: the style is chosen when a game is created

The home page's creation asks for the style, and the game is created with
it (origin-agent `web/`, `server/`, `packages/game`; ask 12 of
`docs/platform-asks.md`):

- **The picker.** Creating a game (the Library's `GameCreation`, the
  New Game prompt) shows, under the prompt, the style catalogue as cards
  (preview, name, descriptor, 2D or 3D, a badge when the style needs
  authored art). As the person types the premise, the three best matches
  (`pickStyle`'s weights, exported by `@origingame/create-game`) lead the
  list; the rest is browsable by category. A style must be picked before
  Create (the recommendation preselected); the choice shows as a chip on
  the composer and can be changed until the first build.
- **Created with it.** The scaffold writes the look with `--look <slug>`
  (or the same through create-game's API), `project.ogt`'s `look`, and the
  art bible (`artBibleTemplate` filled with the premise, the camera, the
  platform and the entry) into the game's design graph as the art
  direction node. The agent's first prompt carries the chosen style, and
  `server/prompts/create.md` says the look is a contract: every asset
  request goes through `artBrief`, `og style check` before calling a
  screen done.
- **The critic.** The evaluation stage hands the entry's three checks and
  the captured frames to the blind critic (ask 12, landed; in git history).
- **Previews** come from the engine package (`knowledge/previews/`), served as static
  files by the web build; no image is copied by hand.
