# Art wave: a game's style as a contract on every asset, not a filter on the frame

Decided by the owner on 2026-10-28, after the review recorded in the
thread of that day
(https://ampcode.com/threads/T-6c20835f-dd62-4743-b673-a1cfbe426f7f). This
document is the execution package: the problem, the design, ten streams
(each one agent thread on its own branch, merged by the coordinator, this
repository's rules in `AGENTS.md` §Build and gate discipline), the
platform's part, and the release that ships it (0.7.0).

## 1. What the review found

- **A look is only the frame.** `type look` (`crates/og-cook/src/look.rs`)
  writes seven render resources (`RenderSettings`, `PostEffects`,
  `Stylise`, `MaterialStyle`, `Fog`, `Sky`, `Environment`) and the UI
  style. Models, textures, sprites, flipbooks, tile sets, animation timing
  and the camera are untouched by it: a style is put over whatever assets
  the game has.
- **The catalogue promises more than the frame can hold.** 28 of the 37
  styles say `look.support: full`, which only means every render primitive
  the style names draws. Rendered side by side with a target image of each
  style (Painter, 37 frames, `.amp/in/artifacts/style-compare-*.jpg` in the
  review thread), about nine read as their style on the showcase diorama
  (the filter-defined ones: 8-bit, monochrome and coarse pixel, monochrome
  dither 3D, pencil hatching, black-figure pottery, layered silhouette,
  the comic outlines). Faceted low-poly, sculpted clay, miniature diorama,
  painterly 3D, paper craft, voxel worlds, neon vector, top-down tile
  pixel and low-resolution 3D are indistinguishable from the plain
  diorama, because what makes them is the assets: facets, voxels, clay
  bevels, a tile grid, wireframes.
- **Asset production does not know the style.** `producing-game-assets`
  says "direction boards settle palette, shape language, atmosphere and
  materials", `og-model` looks at a model "under the game's light and
  look"; nothing turns a style's `assets` and `palette` (prose today) into
  a constraint a generator, the cook or a check can hold.
- **The pixel-art reference does it at the asset.** Pixelorama
  (github.com/Orama-Interactive/Pixelorama, MIT) styles pixels in the
  editor: a non-destructive stack of layer effects (Palettize, Posterize,
  Outline, Drop Shadow, Gradient Map, Pixelize, HSV, curves; its
  `src/UI/Timeline/LayerEffects/LayerEffectsSettings.gd`), indexed colour,
  palette management, pixel-art rotation (cleanEdge, OmniScale, rotxel)
  and tile maps, baked into the exported sheet. The style is in the asset
  before any frame is drawn.
- **No preview.** The catalogue has no image of any style; the only
  pictures are fx-showcase's regression frames, one grey diorama 37 times.
- **The platform never asks.** origin-agent's creation flow (the
  Library's `GameCreation`) takes a prompt; ask 12 of
  `docs/platform-asks.md` (the style step) is open.

## 2. The design

A style is one contract, held at four places:

```text
            ┌────────────── the contract: the game's look (.ogt, one source) ──────────────┐
            │ the frame (as today)   ArtPalette  ArtPixels  ArtTexels  ArtShape              │
            │ ArtLine  ArtMotion  ArtCamera   ui   (checks, preview: the catalogue)          │
            └────────┬──────────────────┬──────────────────┬──────────────────┬────────────┘
                     ▼                  ▼                  ▼                  ▼
            1 produce             2 conform (cook)     3 run               4 check
            the contract          every asset made     the frame's own     og style check:
            compiled into each    to the contract,     treatment, sprite   the assets' and
            generator request     deterministically,   outlines, palette   the frame's
            (images, models,      sources untouched    swaps, pixel-art    numbers; the
            og-model recipes)                          rotation, camera    critic's checks
```

The root fix is 1 and 2: the assets carry the style. 3 keeps what really
is a property of the frame (light, tone, fog, post) and adds the runtime
pieces a style needs on a sprite. 4 makes the style measurable.

### 2.1 The contract: seven art resources in the look

New IDL module `art` (`crates/og-api/api.ogt`), seven resources a look
sets like the render ones (by reflected name, `og-cook/src/look.rs`
`LOOK_RESOURCES` grows by seven). Each field's zero value is "no
constraint", so a look without them, and every existing game, cooks and
draws exactly as before. They are resources, so game code can read the
contract too (a sprite snapping to `ArtPixels.unit`, a spawner picking
from `ArtPalette`). The names (not final field lists; the stream settles
them in the IDL, three.js vocabulary where one exists):

| Resource | Holds | Read by |
|---|---|---|
| `ArtPalette` | `colors` (a palette asset, `@palettes/*` or the game's), `strict` (every opaque texel on a palette colour), `rampSteps` (shades per hue), `accents` (colours reserved for signals), `dither` (none, bayer2/4/8, blueNoise) | conform 2D and 3D textures, check |
| `ArtPixels` | `unit` (logical pixels per world unit, 2D), `logicalHeight` (as `Stylise.pixelHeight`), `spriteSizes` (allowed cell sizes, e.g. 16, 32), `rotation` (nearest, cleanEdge, prerotated N), `snap` | conform sprites, flipbooks, tile sets; runtime sprites; check |
| `ArtTexels` | `density` (texels per metre), `maxSize`, `filter` (nearest, linear), `mips` | conform 3D textures, check |
| `ArtShape` | `faceBudget` (per prop, per character), `normals` (smooth, flat, angle), `bevel` (radius in metres), `voxel` (edge in metres, 0 none), `silhouette` (min feature, metres) | conform models, og-model recipes, check |
| `ArtLine` | `width` (logical px for sprites, px for meshes), `color`, `place` (outside, inside, both), `distanceFalloff` | conform sprites (baked outline), runtime meshes (MaterialStyle outline), check |
| `ArtMotion` | `stepFps` (animation on 2s/3s: 12, 8), `maxFrames` per clip, `squash` (0..1 default squash/stretch for og.tween presets) | conform flipbooks and clips, runtime `AnimationPlayer.stepFps` default, check |
| `ArtCamera` | `projection` (perspective, orthographic), `pitch`/`yaw` (isometric presets), `fovRange`, `pixelPerfect` | runtime camera defaults (`og.camera` presets), check |

The catalogue entry's prose `assets` and `palette` become these values in
each of the 37 presets (`templates/looks/<slug>.ogt`), and `og-taste` §12
documents them. A game's look `extends` a preset and overrides field by
field, as today.

**The project names its look.** The cook needs the contract before game
code runs, so `project.ogt` gains `look "looks/<id>.ogt"` (the same id
`GameDesc.look` takes; `GameDesc.look` unset falls back to it). `create`
writes both. A project without the line conforms nothing.

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

As built (stream A5, `tests/evidence/art/brief/NOTES.md`):

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
  and its tests hold equal; its package.json gains `main`. On the command
  line: `bun <knowledge>/scripts/brief.ts <kind> "<subject>" [--project
  <dir> | --look <slug>] [--reference <path>]... [--metres <n>] [--json]`.

The skills change with it: `producing-game-assets` (platform knowledge,
mounted from origin-agent; its engine side is the "consumer on Origin
Engine" section, owned here through og-build: og-build §8, request, import,
cook and read the report, opt out, check), `og-model` (§5: one recipe per
shape language, faceted, voxel, clay bevel, paper fold, wireframe for neon
vector, with the og-geom generators; §6: the look's budget over the table)
and `og-taste` (§1: the look is the contract and every asset request goes
through the brief, with the command; §12: the cook conforms, the art bible
keeps the brief of each kind). og-build §8 names `og style check` (§2.5)
as coming until stream A6 lands. The craft skills `visual-acceptance`,
`level-texturing`, `component-forge` and `recipe-craft` judge and make
assets against the brief and the cook's report.

### 2.3 Conform: the cook makes every asset meet the contract

The cook reads the project's look and conforms each asset as it cooks it.
The source is never written; the cooked blob conforms, and the cook
report says what it changed per asset. Deterministic, as the rest of the
cook (same input, same bytes), and cached by the look's hash with the
asset's. An asset opts out with `conform off` in its `.ogt` (or a
`conform` block in `project.ogt` naming paths); every opt-out is listed
by `og cook --report` and `og style check`.

| Asset | Conformed by | What it does |
|---|---|---|
| texture used by sprites, flipbooks, tile sets, UI images (2D) | `ArtPixels`, `ArtPalette`, `ArtLine` | resample to the pixel unit with a pixel-art downscale (a dominant-colour box filter, never bilinear), alpha to 1 bit, palette mapping by Oklab nearest with the palette's dither, ramp limiting per hue, the outline drawn into the cell (Pixelorama's OutlineInline, diamond/circle/square brush, outside or inside), an index texture beside it for runtime swaps, `rotation prerotated N` bakes N rotations with cleanEdge |
| painted 2D (paint and line styles) | `Stylise`'s paint fields read as texel space | the Kuwahara, pigment pooling and canvas grain applied in the texture's own texels, so the stroke follows the sprite, not the screen |
| texture used by meshes (3D) | `ArtTexels`, `ArtPalette`, `Stylise`'s paint fields | resample to the density for the model's size, then (painted styles) the same texel-space paint as painted 2D on a model's own colour textures, so strokes follow the surface (not on a texture no model claims, nor on normal or data maps), palette mapping where `strict`, nearest filter and no mips where `filter nearest` (low-resolution 3D) |
| model (glTF, `type model`, `.vox`) | `ArtShape`, `ArtTexels`, `ArtPalette` | decimate to the face budget (meshopt simplify, already a cook dependency, else quadric), flat normals or an angle split, bevel (a chamfer on hard edges), voxelise at `voxel` (the `.vox` importer's mesher) with colours from the palette, bake vertex colours where the style is untextured (faceted, minimal geometric), the outline hull's data for `ArtLine` |
| animation clips and flipbooks | `ArtMotion` | step the sampled clip at `stepFps`, drop frames beyond `maxFrames` with a warning |

Algorithms taken from Pixelorama (MIT) and cleanEdge (torcado, MIT) keep
their notices in `third_party/<name>/LICENSE` beside the ported code's
header; OmniScale is not taken
until its licence is read.

### 2.4 Run: what belongs to the frame, and a sprite's own

The look's frame part stays as it is. New runtime pieces, all off at
their zero value:

- **Sprite outline** (`Sprite.outline`, `outlineColor`, from `ArtLine`
  when unset and the look sets one): a pixel outline drawn by the sprite
  shader from the alpha at the logical pixel size, for sprites whose
  outline cannot be baked (scaled, tinted, generated at run time).
- **Palette swap**: a sprite or tile layer draws its index texture through
  a palette row (`Sprite.paletteRow`, a palette texture of N rows):
  team colours, hit flashes in palette, day and night ramps.
- **Pixel-art rotation**: `TextureFilter` gains `cleanEdge`, a sampler
  mode in the sprite shader that rotates pixel art without blur or
  jaggies (cleanEdge, MIT).
- **Camera presets** from `ArtCamera`: the main camera's projection,
  angle and field of view default to the contract; `pixelPerfect` snaps
  the 2D camera (today's `Stylise.pixelSnap` path).
- **The anime-cel frame.** In the showcase's `looks-anime-cel-3d`
  reference the spheres show no cel bands and no outline although the
  preset sets `toon true` and `outlineWidth 2.5`; find out whether the
  material style reaches the showcase's materials (a fix, or the station
  sets `styled false`), and record the frame.

### 2.5 Check: the style measured

`og style check` (and a line in `og doctor --project`): per asset,
palette hit rate (share of opaque texels within ΔE00 2 of a palette
colour), distinct colours, pixel-grid alignment (texels per logical pixel
constant, no half pixels), texel density spread, faces against the
budget, normal smoothness (flat share), outline width found; per frame
(`og run --headless --commands` captures of title, play, results): colour
count, palette hit rate, edge statistics, and the three catalogue checks
handed to the critic as written (`--critic` prints the prompt and the
frames; the platform's critic answers them). JSON with `--json`.
`og build --publish` prints its summary as warnings; it never refuses a
build (numbers cannot judge a style, og-taste §11).

### 2.6 The catalogue: honest layers, previews, sample scenes

`looks/catalogue.json` goes to `schemaVersion` 2:

- `preview`: `previews/<slug>.webp` beside the catalogue, 960×540, one
  picture of the style as a game (the review thread's Painter frames,
  `packages/knowledge/looks/previews/`), checked present by the validator.
- `look.layers`: which layers the style needs, among `frame` (the look's
  render resources), `conform` (the cook's asset conformance), `runtime`
  (§2.4), `authored` (art no conformance can make: painted portraits,
  hand animation). `look.support` stays, now derived: `full` when every
  needed layer but `authored` is implemented, `content` when `authored`
  is needed, `partial` otherwise with `missing` naming what.
- `look.family`: one of eight sample scenes (§3 stream A7, `examples/art-<family>`) the style is
  proven on.

The showcase's 37-preset station stays as the frame regression it is; the
proof that a style reads is its family's sample scene with conformed
assets, one frame per style.

## 3. Streams

Each stream is one thread on branch `art/<stream>` from the main named in
its brief, in `external-agent:claude-code` mode on Claude Opus 5.5 high,
at most three at once (`AGENTS.md`). Agents' loop: `cargo check -p`, the
unit tests of the crate they edit, `og api check` after an IDL change,
`og run --headless --commands` frames read at 640×360 on lavapipe. They
commit on the branch, push it, and report; they never merge, run the
gate, record `metal` frames or publish. Evidence:
`tests/evidence/art/<stream>/NOTES.md`.

| # | Stream | Needs | Repo | Owns |
|---|---|---|---|---|
| A0 | `art/doc` | — | engine | this document, the catalogue's previews (37 webp), schema 2's `preview` and `layers` with honest values, the validator, og-taste §12's table (coordinator) |
| A1 | `art/contract` | A0 | engine | the `art` IDL module (seven resources), look blocks, `project.ogt` `look`, `GameDesc.look` fallback, the 37 presets' values, `create` writing `project.ogt`'s look, wiki and `.d.ts` regenerated, og-game and og-taste docs, CHANGELOG lines |
| A2 | `art/conform-2d` | A1 | engine | §2.3's 2D rows: the conform pass for textures used by sprites, flipbooks, tile sets and UI images, painted texel-space, index textures, prerotation; cook report entries; og-cook tests with fixtures whose expected texels are computed independently |
| A3 | `art/conform-3d` | A1 | engine | §2.3's 3D rows and animation: decimation, normals, bevel, voxelise, vertex colours, texel density, palette, step fps; og-cook tests |
| A4 | `art/runtime` | A1 | engine | §2.4: sprite outline, palette swap, cleanEdge filter, camera presets, `ArtMotion.stepFps` default, the anime-cel finding; og-tests scenes with their reference frames named (metal recorded by the coordinator) |
| A5 | `art/brief` | A1 | engine | §2.2: `artBrief` in the knowledge package and its re-export from create-game, the skill changes (og-taste, og-model recipes, og-build, the consumer section) |
| A6 | `art/check` | A2, A3 | engine | §2.5: `og style check`, the doctor line, the publish summary, `--critic` |
| A7 | `art/samples` | A2, A3, A4 | engine | the eight family scenes (pixel side-on, pixel top-down, pixel-in-3D, painted 2D, ink and line 3D, shape and paper 3D, toon and comic 3D, cinematic 3D) in a new fx-showcase station or `examples/art-<family>`, each with assets made through `artBrief` and the cook's conformance; one frame per style; `look.family` filled |
| P1 | `style-step` (merged into origin-agent `main` 2026-10-28: `0c54d85`, `cd2f6cc`, `89e00b4`) | A0 (catalogue 2, previews on main) | origin-agent | §4 |
| R | release 0.7.0 | all | engine (macrunner) | §5 |

Order: A0 now (the coordinator); A1 and P1 at once; after A1 merges, A2,
A3 and A4; then A5, A6, A7; R last, then P1's pin move.

Shared files and how they are touched: `api.ogt` (A1 adds the module; A4
appends to `render`'s sprite records; nothing reorders), `og-cook`'s
`cook/mod.rs` (A2 and A3 each add one call into a new `cook/conform/`
module of their own: `conform/texture2d.rs`, `conform/mesh.rs`),
`og-render`'s sprite shader (A4 only), `templates/looks/*.ogt` (A1 only;
later streams change values only with the coordinator), the catalogue
(A0, then A7's `family`).

## 4. The platform: the style is chosen when a game is created

Owner's requirement (2026-10-28): the home page's creation asks for the
style, and the game is created with it. origin-agent (`web/`, `server/`,
`packages/game`), one thread, branch `style-step`:

- **The picker.** Creating a game (the Library's `GameCreation`, the
  New Game prompt) shows, under the prompt, the style catalogue as cards
  (preview, name, descriptor, 2D or 3D, a badge when the style needs
  authored art). As the person types the premise, the three best matches
  (`pickStyle`'s weights, exported by `@origingame/create-game`) lead the
  list; the rest is browsable by family. A style must be picked before
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
  the captured frames to the blind critic (ask 12 as written).
- **Previews** come from the mounted knowledge
  (`knowledge/game/skills/og-taste/looks/previews/`), served as static
  files by the web build; no image is copied by hand.
- **Pin.** Develop against the engine's main (the mount script from a
  checkout); move `server/scripts/e2b/origin-engine.json` to 0.7.0 when R
  publishes, and remount.

## 5. Release 0.7.0

A minor: a game whose look extends a preset now has its assets conformed
by the cook (behaviour a game relied on changes), and the presets gain
values. CHANGELOG and `changelog.json` carry one line per IDL member and
a breaking entry for the conformance with its fix (`conform off`, or
override the preset's art blocks to zero). The coordinator, on
`macrunner` (`/Volumes/app/origin-engine`): merge, `og api gen --wiki`,
record the `metal` frames the streams named and read them, the functional
lane, the replay gate (the simulation is unchanged: conformance is cook
time, the camera presets apply only through a look), `release.sh bump
0.7.0`, the release build and verify on Mac, Linux and Windows, publish,
tag, the receipt `tests/evidence/release/0.7.0.md`, §3 of
`docs/release.md`. Then the platform's pin (§4).

## 6. Done means

- Each of the 37 styles has a preview in the package, honest layers, and
  a frame on its family's sample scene that a reader names as the style
  without being told (the coordinator reads them beside the previews).
- A new game made from the home page with any style has, before its first
  prompt, the look, the project's look line and the art bible; an asset
  it adds is conformed by the cook and passes `og style check`, or the
  check says what is off.
- 0.7.0 is on npm and pinned by the platform.
