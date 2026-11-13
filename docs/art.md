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
            the contract          every asset made     the frame's own     og style
            compiled into each    to the contract,     treatment, sprite   check-project:
            generator request     deterministically,   outlines, palette   the assets' and
            (images, models,      sources untouched    swaps, pixel-art    the frame's
            model recipes)                             rotation, camera    numbers
```

1 and 2 make the assets carry the style, 3 keeps what is a property of the
frame (light, tone, fog, post) and adds a sprite's runtime pieces, and 4
measures the style. The engine supplies the facts; the agent judges them.

### 2.1 The contract: seven art resources in the look

The IDL module `art` (`crates/og-api/api/art.ogt`) declares seven
resources a look sets like the render ones (by reflected name,
`og-bundle/src/look.rs` `LOOK_RESOURCES`). The IDL holds their fields.
Each field's zero value is "no constraint", so a look without them cooks
and draws as a plain look. They are resources, so game code can read the
contract too.

| Resource | Constrains | Read by |
|---|---|---|
| `ArtPalette` | the palette, its strictness, colour count, ramp steps, accents and dither | conform 2D and 3D textures, check |
| `ArtPixels` | the pixel unit, logical height, sprite cell sizes, rotation and snap | conform sprites, flipbooks, tile sets; runtime sprites; check |
| `ArtTexels` | texel density, size, filter and mips | conform 3D textures, check |
| `ArtShape` | face budgets, normals, bevel, voxel edge and smallest feature | conform models, og-model recipes, check |
| `ArtLine` | outline width, colour, placement, falloff and brush | conform sprites, runtime meshes, check |
| `ArtMotion` | stepped animation rate, frames per clip and squash | conform flipbooks and clips, runtime defaults, check |
| `ArtCamera` | projection, fixed angle, pitch and yaw, field of view and pixel-perfect snap | runtime camera defaults, check |

Every preset (`templates/looks/<slug>.ogt`) sets these values, and the
`looks` guide says how a game uses them. A game's look `extends` a preset
and overrides it field by field.

**The project names its look.** The cook needs the contract before game
code runs, so `project.ogt` has `look "looks/<id>.ogt"` (the same id
`GameDesc.look` takes, and its fallback). `create` writes both. A project
without the line conforms nothing.

**Creation choices precede the art contract.** `og new --choices` records
`genre`, `view`, `look`, `mood`, `ui`, `platform`, `input`, `players`,
`lang` and `music` in a `create` block in `project.ogt`.
`packages/create/bin/choices.ts` owns the vocabulary and exports its JSON
schema, premise inference and `resolveCreationChoices` through
`@origingame/create-game`; these are additive public contracts beside the
art brief. The UI choice defaults to the look's `ui` line and the mood to
the look's own (§2.10). Music records intent only.

### 2.2 Produce: every request carries the contract

`artBrief(look, kind, subject, options?)` (`packages/knowledge/scripts/brief.ts`,
re-exported by `@origingame/create-game` from a byte-for-byte copy its tests
hold equal) compiles the flattened look and its catalogue entry into the
request for one asset: the prompt (direction, palette as named colours and
hex, sizes, line, shape language, avoid list), the style's reference images
and the generator's parameters. It is a pure function, so the platform and
an agent in a sandbox read the same. `readLook(projectDir)` reads a
project's look as the cook merges it, and `presetLook(slug)` a preset
alone. Their signatures, the kinds and what each kind decides are the
knowledge README's §The art brief, a public contract (AGENTS.md).

- **3D or 2D.** A prop or character of a 3D game is a model held to
  `ArtShape`'s face budget and `ArtTexels`; of a 2D game a sprite on the
  `ArtPixels` grid. A field at zero lets the entry's prose stand in.
- **Drawn for the conform.** Pixel art is requested at the whole multiple
  of its grid nearest 1024 px, so the cook's downscale (§2.3) lands on the
  grid.
- **The UI kinds.** After the eight asset kinds, `ART_KINDS` continues with
  the art-slot kinds of docs/ui.md §3.3, named as og-ui's `SlotKind`
  (`crates/og-ui-core/src/slot.rs`); `UI_KINDS` gives each its aspects,
  focal point, safe area, fit, legible size and nine-slice states, and
  og-ui's tests hold the two declarations equal. A UI brief's prompt names
  what the conform will do to it, so the art is drawn for it.

Every asset request goes through the brief: the platform's asset
production and the game agent alike (the `art-review` guide).

### 2.3 Conform: the cook makes every asset meet the contract

The cook reads the project's look and conforms each asset as it cooks it.
The source is never written; the cooked blob conforms, and the cook report
says what changed per asset. Conformance is deterministic (same input,
same bytes) and cached by the look's hash with the asset's. An asset opts
out with `conform off` in its `.ogt` (or a `conform` block in
`project.ogt` naming paths), and every opt-out is listed by
`og cook --report` and `og style check-project`. A model declares its role
with `conform character` (held to `characterFaces`, as a skinned model is)
or `conform effect` (an effect's mesh, which no face budget simplifies;
`og vfx import` writes it).

| Asset | Conformed by | What it does |
|---|---|---|
| 2D textures (sprites, flipbooks, tile sets, UI images) | `ArtPixels`, `ArtPalette`, `ArtLine` | resample to the pixel unit with a dominant-colour box filter (never bilinear), 1-bit alpha, Oklab-nearest palette mapping with the palette's dither, ramp limiting per hue, the outline drawn into the cell, an index texture for runtime swaps, and `rotation prerotated N` baked with cleanEdge |
| painted 2D (paint and line styles) | `Stylise`'s paint and watercolour fields, in texel space | the paint and watercolour passes applied in the texture's own texels, so the stroke follows the sprite; the frame then draws neither pass, so a sprite is painted once |
| 3D textures | `ArtTexels`, `ArtPalette`, `Stylise`'s paint fields | resample to the density for the model's size; the texel-space paint on a model's own colour textures (never normal or data maps); palette mapping where `strict`; nearest filter and no mips where `filter nearest`; otherwise held to `maxColors` and `rampSteps` and abstracted into flat regions of at least the texture's smallest shape |
| models (glTF, `type model`, `.vox`) | `ArtShape`, `ArtTexels`, `ArtPalette` | decimate to the face budget (meshopt), flat normals or an angle split, bevel hard edges, voxelise at `voxel` with palette colours, bake vertex colours for untextured styles, the outline hull's data for `ArtLine` |
| clips and flipbooks | `ArtMotion` | step the sampled clip at `stepFps`, drop frames beyond `maxFrames` with a warning |

Ported algorithms (Pixelorama, cleanEdge; both MIT) keep their notices in
`third_party/<name>/LICENSE` beside the ported code's header.

### 2.4 Run: what belongs to the frame, and a sprite's own

The look's frame part is the fallback: a preset keeps its `posterize` and
dither, so plain, generated or runtime-loaded materials still get its
steps and tones. What the cook conformed is not stepped again: a loaded
texture adds its palette (`<id>#palette`) to the renderer's conformed
colours, and the quantise leaves a pixel within 6 sRGB levels of one of
them, measured before exposure and the tone curve. The runtime pieces,
all off at their zero value:

- **Sprite outline** (`Sprite.outline`, `outlineColor`, defaulting to
  `ArtLine`): drawn by the sprite shader for outlines the cook cannot bake.
- **Palette swap**: a sprite or tile layer draws its index texture through
  a palette row (`Sprite.paletteRow`).
- **Pixel-art rotation**: `TextureFilter.cleanEdge` rotates pixel art
  without blur or jaggies.
- **Camera presets**: the main camera defaults to `ArtCamera`;
  `pixelPerfect` snaps the 2D camera.
- **Toon bands**: `MaterialStyle`'s toon fields and `outlinePlace` hold the
  toon presets' bands and outline.
- **Mesh lines**: where the frame's MaterialStyle sets no outline,
  `ArtLine` outlines the meshes.
- **`ArtMotion.squash`**: og.tween.squash's default impact.

### 2.5 Check: the style measured

`og style check-project` (and a line in `og doctor --project`) measures,
per asset, the palette hit rate, distinct colours, pixel-grid alignment,
texel density spread, faces against the budget, flat-normal share and
outline width; per clip, the rate, held poses and frames against
`ArtMotion`; and per frame (captures of every declared screen at each
viewport of `--sizes`, labelled `<screen>@<viewport>`), the colour count,
palette hit rate, edge statistics and the captured camera against
`ArtCamera`. `--json` carries the look's flattened art blocks and the
scenes no capture reaches. The engine measures and the agent judges (the
`art-review` guide holds the thresholds); `og build --publish` prints the
summary as warnings and never refuses a build.

### 2.6 The catalogue: honest layers, previews, sample scenes

`looks/catalogue.json` (`schemaVersion` 3) gives each style:

- `preview`: `previews/<slug>.webp`, 960×540, checked present by the
  validator.
- `look.layers`: which layers the style needs among `frame`, `conform`,
  `runtime` (§2.4) and `authored` (art no conformance can make).
  `look.support` is derived: `full` when every needed layer but `authored`
  is implemented, `content` when `authored` is needed, `partial` otherwise
  with `missing` naming what.
- For a creation UI: `category`, `look.views`, `look.mood` (§2.10) and,
  for `content` styles, `look.authored`, the art each brief kind must be;
  `artBrief` writes it into every request of the kind, and
  `authoredArt(look)` lists it.
- `look.frozen` (true or absent) marks a style whose look is being rebuilt
  (AGENTS.md, "Frozen is not obsolete"). Its preset still cooks, `extends`
  still builds on it and its brief still reads, so a game made on it is
  unchanged; create offers a new game only the rest (`creationLooks`), and
  the validator keeps an unfrozen style for every creation view. Eight are
  offered now: cinematic-realism, anime-cel-3d, inked-comic-3d,
  painterly-3d, faceted-low-poly, low-resolution-3d, 16bit-pixel and
  minimal-geometric.

A style's sample scene belongs to the game's art review; the engine ships
no sample-game gallery.

### 2.7 A style's painted art: the hand-off

The engine never paints a style's art, `og` makes no network call, and no
`og.*` member generates art. A style pack is a directory of CSS and
assets, and its painted frames, buttons and ornaments are standard CSS
images (`border-image`, `mask-image`). Whoever owns image generation (the
platform's agent for a game, an offline script for the engine's seed
styles) turns the pack's `art-requests.json` into prompts through
`styleArtBriefs` (§2.2).

### 2.8 The auto-rig's input layers

A character drawn in layers is rigged by the cook with no editor step: a
`type rig2d` file names the art and a template, and the cook makes the
bones, slots, leg IK, atlas and the template's clips
(`crates/og-cook/src/cook/systems/autorig.rs`). This section is the contract the
art meets. `rigParts(template)` of `@origingame/create-game` lists every
template's parts with their parent, joint end and motion, back to front,
and og-cook's `rig_parts_match_the_templates` holds it equal to the cook.

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

Any other line is refused.

**Templates.** In `humanoid-side` and `quadruped` the character faces
right, `-l` is the far side and `-r` the near side. In `humanoid-front`
and `creature` `-l` is the image's left and `-r` its right, and the
character faces the viewer. The humanoids and the quadruped have two-bone
IK on each leg; a creature has none, and its body squashes and stretches.
A front-facing humanoid's clips never move a foot inwards of where it is
drawn, so feet drawn side by side never cross.

**Naming.** A layer's or file's name is trimmed, lower-cased, with `_` and
spaces as `-`, and must be one of the template's parts. A name starting
with `_`, a layer in a group whose name starts with `_`, and a hidden PSD
layer are left out. A PSD group named for a part merges its layers into
that part. Each part is drawn once.

**Layers and size.** A PSD (RGB or greyscale, 8 or 16 bits) layer may have
any size and place; a folder's PNGs are all at the canvas size, each where
it is drawn. No margin is needed: the cook trims the parts and packs them
into an atlas of at most 4096 x 4096. The ground is the lowest drawn pixel,
and a world unit defaults to the drawn height over the template's height
(a humanoid 2, a quadruped 1.2, a creature 1) unless `unit` sets it.

**Joints.** A pixel is solid at alpha 128 or more.

- The root part's joint is the centre of its solid pixels.
- Every other part's joint is the centre of its solid pixels over its
  parent's, within its joint end's band. So draw each part overlapping its
  parent where they join, as a shoulder lies over the torso.
- Without an overlap, the joint is at the part's end by convention, and it
  must lie within 2 % of the drawn height (at least 3 pixels) of the
  parent, or the art is refused. `pivot <part> <x> <y>` sets a joint
  outright.
- Where a limb continues into its next part (elbow, knee, wrist, ankle,
  waist, neck), the overlap turns rigidly, so draw it round. A part hung
  from the trunk (shoulder, hip, cape, tail) hinges at its joint, so draw
  its end round where the parent covers it.
- A leg drawn straight bends the template's way; one drawn bent bends as
  drawn.
- A humanoid's head must be above its pelvis, and every foot or paw below
  the hips, or the art is refused.

**Draw order.** The parts draw back to front in the art's order: a PSD's
layer order (bottom first), or a folder's sorted file names, where a
`NN-` prefix numbers the order and is dropped from the part's name.

**Refusals** name the layer, the file and the fix: an unknown part (with
the nearest name), a part drawn twice, an empty part, missing required
parts, PNGs of different sizes, PSD layer effects, blend modes, clipping
or vector masks and adjustment layers (with the Photoshop menu that fixes
each), a part that does not touch its parent (with the `pivot` line), an
upside-down character, an overflowing atlas and a `pivot` for an absent
part.

### 2.9 The audio brief: every cue and effect a game needs

The audio counterpart of §2.2: `audioBrief({ music, mood, genre,
premise? })` in the same `brief.ts`, re-exported by
`@origingame/create-game`, is a pure, deterministic public contract (a
change is a break with a breaking changelog entry). `readAudioChoices`
reads its input from project.ogt's `create` block, and an unknown value
throws naming the known ones. It returns the shared direction and avoid
list, the music cues (title, the genre's cues, victory and defeat), the
sound effects (the UI's and the genre's) and, for each effect a
procedural og cook preset fits, the `type sfx` document that stands in
until the generated file exists. The knowledge README's §The audio brief
holds the shapes and formats. The platform's generation takes these
requests as they are (`docs/platform-asks.md` ask 26).

### 2.10 Mood: a parameter of the look

A look takes `mood "<word>"`, one of `bright`, `dark`, `warm`, `cold`,
`soft`, `high-contrast` (og-bundle's `MOODS`, the creation choice's words).
Every preset names the mood it is drawn in (the catalogue's `look.mood`).
A game's look naming another mood gets the mood's layer laid over the
preset its `extends` chain reaches, below the looks above it, so the
game's own fields still win (a chain without a preset gets it under its
deepest look). The cooked look carries the mood (`LookInfo.mood`).

The layers are data in `templates/looks/moods/` (its README), compiled
into the cook:

- `<mood>.ogt`, for every style: the frame's grading and light. A number
  moves the preset's value by its distance from the field's default,
  clamped to the field's range; any other value replaces the preset's.
- `<mood>/<slug>.ogt`, one style's own layer in place of the generic one,
  where grading alone fails it (palette-bound styles swap the palette, and
  posterized styles leave out what posterize breaks).

A posterized look whose moods must keep its hues names
`Stylise.posterizeMode tone`, which steps value and tint apart.

A layer sets only the frame's grading, light and palette (`PostEffects`,
`RenderSettings`, `Environment`, `Fog`, `Sky`, `Stylise.palette`,
`ArtPalette.colors`); the contract's sizes, shapes, lines and motion never
change with the mood. The art brief's prompt gains a `Mood` line (§2.2).
`og new --mood <word>` writes the line into the game's look; without one
the game keeps its preset's.

## 4. The platform: the style is chosen when a game is created

Creation on the platform (origin-agent; ask 12 of `docs/platform-asks.md`)
asks for the style:

- **The picker** shows the catalogue as cards, the three best matches for
  the premise first (`pickStyle`, exported by `@origingame/create-game`). A
  style must be picked before Create, and it can be changed until the first
  build.
- **Created with it.** The scaffold writes the look (`--look <slug>`),
  `project.ogt`'s `look` and the art bible into the game's design graph.
  The agent's first prompt says the look is a contract: every asset
  request goes through `artBrief`, and `og style check-project` runs
  before a screen is called done.
- **The critic** receives the entry's checks and the captured frames.
- **Previews** come from the engine package (`knowledge/previews/`); no
  image is copied by hand.
