# A5 `art/brief`: every asset request carries the contract

Stream A5 of docs/art.md (§2.2, the A5 row of §3), branch `art/brief` from
`c52344b2` (origin/main, with A1, A2 and A3). TypeScript and knowledge
only: no Rust, IDL, preset or catalogue file changed, so no frame moves.

## What landed

| Commit | What |
|---|---|
| `6a8bf57b` | `packages/knowledge/scripts/brief.ts`: `artBrief(look, kind, subject, options?)` (pure), `readLook(projectDir)`, `presetLook(slug)`, `parsePalette`, `colourName`, `parseLook`, `ART_KINDS`, `ART_RESOURCES`, and a command line. `tests/brief.test.ts` (10 tests). `scripts/pack.ts` copies `templates/looks/*.ogt` and `assets/palettes/` into `presets/` (Git-ignored, in `files`, not mounted for the platform). README §The art brief: the contract. |
| `80a24cdd` | `@origingame/create-game`: `bin/brief.ts` is a byte-for-byte copy (the prepack writes it). `create.ts` re-exports the functions and types, `package.json` gains `main: bin/create.ts` (so `import { artBrief } from "@origingame/create-game"` works) and `presets` in `files`. Two tests: the copy equals the knowledge's, a scaffold briefs, and an installed layout briefs from its own `presets/`, `looks/` and `previews/` with no repository beside it. README section, CHANGELOG line, `changelog.json` `unreleased.added` entry `artBrief`. |
| `6d7d2993` | Skills: og-taste §1 (the look is the contract, every request goes through the brief, the command) and §12 (the cook conforms, the bible keeps the brief of each kind), og-model §5 (one recipe per shape language) and §6, og-build §8 (new: request, import, cook and read the report, opt out, `og style check` marked as coming), and the craft skills `visual-acceptance`, `level-texturing`, `component-forge`, `recipe-craft`. |
| this commit | docs/art.md §2.2 brought up to date with what was built; these notes and `briefs.txt` (six briefs as the command prints them). |

## The contract

```ts
artBrief(look: BriefLook, kind: ArtKind, subject: string, options?: { references?: string[]; metres?: number }): ArtBrief
BriefLook = { extends: string | null; art: ArtBlocks; profile?: "2d" | "3d"; palette?: BriefPalette | null; style?: BriefStyle | null; preview?: string }
ArtBrief  = { kind, subject, style, medium: "image" | "model", prompt, avoid, palette, references: { role: "style" | "accepted"; path }[], image?, model? }
image     = { transparent, width, height, grid: { width, height, scale } | null, tiling }
model     = { faceBudget: number | null, textureSize, normals, voxel, bevel, filter, mips, metres }
```

The platform reaches it as `@origingame/create-game` (main) or the engine
package's `knowledge/scripts/brief.ts`; both read the same file.

## What it decides

- **Medium.** `prop` and `character` are models in a 3D game, sprites in a
  2D one. `profile` comes from project.ogt, or for a preset from the
  style's only profile, else 3D. Every other kind is an image.
- **2D grid.** The cell is `ArtPixels.unit` x the size in metres (a prop
  1, a character and a sprite 2), snapped up to `spriteSizes` (the largest
  when past them all). A character is half as wide as tall. An icon takes
  the smallest cell, a portrait twice the largest, a tile set 4 x 4
  one-metre tiles, and a backdrop `logicalHeight` rows at 16:9. Each is
  drawn at the whole number nearest 1024 / its longest side, so the cook's
  dominant-colour downscale (A2) lands on the grid.
- **Textures.** A model's texture, and a `texture` in a style with
  `ArtTexels`, is `density` x the size, to a power of two, within the
  largest power of two at or below `maxSize`. It is 1024 when neither is
  set. A `nearest` texture is drawn as pixel art at a whole multiple. A
  pixel style without texels takes `unit` x 2 m of pixel art.
- **Prompt.** One statement per line: what it is, the style (name,
  descriptor, art-bible direction), the palette, the size, cells,
  background, tiling, line, shape, motion and view, then the references
  and what to avoid. The palette gives the colours as names with hex
  (named from OKLCH; a `.gpl` keeps its own names), then strictness,
  `maxColors`, `rampSteps`, the accents by name and the dither. The
  entry's prose (palette size and rules, asset specs, outline) stands in
  where a field is zero. On a model the line is the engine's outline, so
  the texture paints none.
- **References.** The style's preview (an absolute path from `readLook`
  and `presetLook`), then the caller's accepted assets in order.

## Verification

- `packages/knowledge`: `bun run test` (the validator, then 60 tests in 4
  files) passed. That includes `tests/brief.test.ts`'s 10 tests (1,671
  expects), whose expected values are worked out by hand in the comments:
  - the same `prop` gives a 16 x 16 sprite at 64x in `16bit-pixel`, but a
    200-triangle model with a 64-texel nearest texture in
    `faceted-low-poly`;
  - a 16bit character is 16 x 32 at 32x (512 x 1024);
  - a synthetic look (unit 10, cells 12/24/40) gives a 2 m character of
    12 x 24 at 43x (516 x 1032), a 3.5 m sprite of 40 at 26x, a tile set
    48 x 48 at 21x, and a 180-line backdrop 320 x 180 at 3x;
  - density 48 over 3 m: 256, or 64 under `maxSize 100`;
  - the palette named with hex, with accents and strictness;
  - colour names, and `.hex`/`.gpl` parsing;
  - references in order;
  - `readLook` through a game look → a game base → the preset, merged
    field by field, with the game's own palette file; a game's own copy of
    a preset under its slug; no look line refused with the fix;
  - every kind for all 37 presets.
- Mutations, each run once and reverted: `floor` for `round` in the grid
  scale, `maxSize` used without its power of two, and a character's width
  and height swapped. Each one fails at least one test.
- `packages/create`: `bun run test` (prepack, then 18 tests) passed.
- `brief.ts` typechecks clean under `tsc --strict
  --noUncheckedIndexedAccess` (TypeScript 5.6 with bun's types, in a
  scratch directory). The og-model snippets (the voxel tree and the neon
  gate) typecheck against `packages/engine/types/index.d.ts`. A
  deliberately wrong field (`radiuss`) is refused, which shows the check
  ran.
- No Rust was touched, so there was no `cargo check`. Not run, as briefed:
  scripts/check.sh, test lanes, `og test`, wasm or release builds.

## For the coordinator

- **A6 has not landed**: og-build §8 names `og style check` "coming with
  the art wave's check stream". Once A6 merges, drop that clause and check
  that the metrics listed match what it prints. visual-acceptance §2 says
  "when the engine you pin has it".
- **Packaging, new**:
  - both packages carry `presets/` (37 `.ogt` and 5 palettes, about
    40 kB), copied at prepack;
  - `packages/engine/scripts/pack.sh` picks it up through the knowledge's
    `files`;
  - create-game gains `main`, without an `exports` map, so deep imports
    such as `bin/create.ts` keep working;
  - R should confirm the tarballs list `presets/` and `bin/brief.ts`.
- **Public contract**: `artBrief`, `readLook` and `presetLook` and their
  shapes are documented as stable in the knowledge README. AGENTS.md's
  list of public knowledge contracts (`installed`, `mount`, `summary`,
  `payload`, `defaultRoots`) could name them too; I left AGENTS.md alone.
- **A preset tension the brief shows**: `monochrome-pixel` has `unit 16`
  but `spriteSizes 12 16 24`, so a 2 m character (32 px) is clamped to a
  24-pixel cell, which is 1.5 m. `8bit-pixel` (unit 16, cells up to 24)
  and `coarse-pixel` (unit 8, cells up to 12) do the same; the other pixel
  presets fit a 2 m character. This is A1's file: the coordinator decides whether the sizes or the unit
  move. The brief follows the cells, as the cook does.
- **The platform (P1)** can call `presetLook(slug)` in the style step
  and `readLook(dir)` afterwards. The image generator's `transparent`
  maps to `image.transparent`. A 3D generator's target polycount is
  `model.faceBudget`, and its texture budget is `model.textureSize`.
