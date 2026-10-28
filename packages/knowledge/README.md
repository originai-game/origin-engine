# @origingame/engine-knowledge

What an agent needs to write a game on Origin Engine, as the platform's
knowledge packs (`knowledge/` of origin-agent, the platform's agent
repository; `packages/game-knowledge/production` of OriginGame before the
agent-repo split):

| Part | Path | Mounted at (`knowledge.json`) |
|---|---|---|
| API wiki, one page per IDL module plus `index.md` | `wiki/` (generated: `og api gen --wiki`; `og api check` fails on drift) | `game/skills/og-game/reference/` |
| Skills, numbered `## N.` sections cross-referenced as `og-feel §4`: `og-game` (the entry point and the map of the others: the project, ECS, scenes, assets, loading, input, 2D, animation, effects, atmosphere and water, material shaders, physics, performance, looks, light and shadows), `og-core` (engine rules), `og-build` (the `og` loop, the evidence discipline, and `og polish`: §5 is the one list of what it measures), `og-feel` (game feel: time scale, hit stop, tweens, camera, juice amounts, controller tunables by genre), `og-ui` (screens, HUDs, styles, motion, text entry, accessibility, gestures), `og-taste` (art direction: a look chosen from the premise out of the style catalogue `looks/catalogue.json`, atmosphere by intent, the done checklist), `og-ai` (navigation and enemy minds as data), `og-audio` (events, presets, the mix, music), `og-i18n` (string tables, fonts, line breaking, right to left), `og-model` (a 3D model built as code from a reference image: the intake's detail inventory, eight gated passes, the og-geom generators' recipes, budgets, the `type model` asset, and the review's `view` and `turntable` frames and `og compare` sheet), `og-platform-sdk` (`og.platform`: saves, settings, leaderboards, telemetry, and how to ask for what is missing; the platform's own `og-platform` covers the Player). Six craft skills beside them teach no module and point to the og-* skills for the API: `architecture-discipline` (the project's layers, splitting, the first playable, loud failure, tuning in data), `component-forge` (a unit built as code from a measured reference), `level-texturing` (a gray level painted from views with `og bake projection`; its driver is `scripts/texture-level.ts` beside it), `localization` (the discipline around og-i18n), `recipe-craft` (the retrieval ladder for example code, vendoring, distilling) and `visual-acceptance` (judging frames against references, baselines and the play camera). Each IDL module's `skill` names the one that teaches it; the wiki's index groups modules by it, and `bun run test` checks that skill names the module's page, no other does, and every section reference resolves | `skills/<name>/SKILL.md` | `game/skills/<name>/` |
| Guides: hand-written pages an agent reads beside a skill, held to the same rules as the skills (every `og.*` name and `Type.field` in the API, section references resolved, each named by og-game, listed in `GUIDES`): `three-to-og` maps three.js's classes and idioms to the engine's and names the gaps; `og migrate three` cites its sections | `guides/<name>.md` | `game/skills/og-game/guides/` |
| Scaffold: the project template, its engine pin and binary digests | `scaffold/` and the engine repository's `templates/game` | `game-scaffold/origin-engine-ts/` |

`bun run test` validates the package against the platform's rules
(`rules/platform.json`, transcribed from the platform by
`bun scripts/sync-rules.ts <origingame checkout>`; each rule names its
source) and runs the tests in `tests/`. It also holds the skills to the IDL: every
`og.*` name a skill writes (`og.render.applyLook`, `og.Sprite.array`,
code blocks included) and every `Type.field` of a struct, desc, component,
resource or event (`SpriteDesc.dropShadow`) must exist in
`packages/engine/schema/api.schema.json`; the few `og.` names that are not
API (a store key, an engine-owned component the dumps show) are listed
with their reason in `NOT_API`. `bun scripts/validate.ts --bundle`
prints the text payload in the platform's `readProductionBundle` shape
(`{ schemaVersion: 1, files }`).

`bun scripts/mount.ts <origin-agent checkout> [--check] [--commit <rev> | --package <dir>]`
writes the payload into the checkout's `knowledge/` (text files only; it
owns `game/skills/<each of its skills>/` and `game-scaffold/origin-engine-ts/` there and
removes what the package no longer has); `--check` exits 1 on any
difference, `--commit` mounts a commit instead of the working tree and
`--package` the knowledge of an installed engine. origin-agent itself runs
`scripts/knowledge/mount-engine.ts`, which imports `installed`, `mount` and
`summary` from the installed engine's `knowledge/scripts/mount.ts` with its
own knowledge root: those three signatures are a public contract
(AGENTS.md). So are `payload(roots)` and `defaultRoots(pkg?)` of
`scripts/lib.ts` and the `PayloadFile` and `Roots` shapes they use, for a
platform that reads the knowledge straight from the installed package
(`payload(installed(dir).roots)`: every file at its production path, with
its source and bytes) instead of mounting a copy. What the platform side
looked like at M7: the repository's `docs/m7-platform-notes.md`.

## The art brief

`scripts/brief.ts` compiles a game's look into the request for one asset
(docs/art.md §2.2 in the engine repository). Every asset request goes
through it (og-taste §1), so the palette, the sizes, the line and the shape
language a style holds reach the generator, and the cook's conformance has
little left to change. `@origingame/create-game` re-exports the same
functions (its `bin/brief.ts` is a byte-for-byte copy of this file).

```ts
import { artBrief, readLook, presetLook } from "@origingame/create-game/bin/create.ts";
// or, in a game: node_modules/@origingame/engine/knowledge/scripts/brief.ts

const look = readLook("path/to/game");           // project.ogt's look, flattened
const brief = artBrief(look, "sprite", "a lantern keeper with a long coat", {
  references: ["assets/sprites/courier.png"],    // accepted assets of the game
});
brief.prompt;      // the request as text, one statement per line
brief.references;  // [{ role: "style", path: ".../previews/<slug>.webp" }, { role: "accepted", path }]
brief.image;       // { transparent, width, height, grid: { width, height, scale } | null, tiling }
brief.model;       // for a 3D prop or character: { faceBudget, textureSize, normals, voxel, bevel, filter, mips, metres }
```

These names and shapes are a public contract, kept stable (a change is a
break, with a `changelog.json` breaking entry):

- `artBrief(look: BriefLook, kind: ArtKind, subject: string, options?:
  BriefOptions): ArtBrief`, pure. `kind` is one of `ART_KINDS`: `sprite`,
  `tileset`, `portrait`, `backdrop`, `prop`, `character`, `texture`,
  `icon`. `options.references` are the game's accepted assets (paths or
  URLs), passed on after the style's preview; `options.metres` is the
  asset's size (a prop 1, a character 2, a texture's tile 2 by default).
- `BriefLook { extends: string | null; art: ArtBlocks; profile?: "2d" |
  "3d"; palette?: BriefPalette | null; style?: BriefStyle | null; preview?:
  string }`: the flattened art blocks (`ArtBlocks`, by resource name of
  `ART_RESOURCES`, each field as the look writes it), the preset slug the
  `extends` chain reaches, the palette `ArtPalette.colors` names (else
  `Stylise.palette`) as named colours (`BriefPalette { id, from, colours:
  PaletteColour[] }`, `PaletteColour { name, hex }`), and the catalogue
  entry (`BriefStyle`, the fields of a `looks/catalogue.json` style the
  brief reads).
- `ArtBrief { kind, subject, style, medium: "image" | "model", prompt,
  avoid, palette, references: BriefReference[], image?: ImageParams,
  model?: ModelParams }`.
- `readLook(projectDir, options?)`: a project's look from disk:
  project.ogt's `look "looks/<id>.ogt"` and `profile`, the look's
  `extends` chain through the game's own looks (a slug names the game's
  `assets/looks/<slug>.ogt` first, a path another look under `assets/`) to
  the preset, merged field by field as the cook merges it, the palette
  file, the entry and its preview. `presetLook(slug, options?)`: a preset
  alone, for a game not written yet (the platform's style step).
- `parsePalette(file, text)` and `colourName(hex)`: a `.hex` or `.gpl`
  palette as named colours.

What it decides, from the look's numbers (a field at zero constrains
nothing, and the entry's prose stands in for it):

| Kind | 2D game, or a 2D kind | 3D game (`prop`, `character`) |
|---|---|---|
| `sprite`, `prop`, `character` | pixel art on a cell of `ArtPixels.unit` x the size, snapped up to `spriteSizes` (a character half as wide as tall), transparent | a model: `ArtShape.propFaces` or `characterFaces` as the face budget, `normals`, `voxel`, `bevel` |
| `tileset` | 4 x 4 tiles of one metre, opaque, tiling | the same |
| `icon`, `portrait` | the smallest cell; a portrait twice the largest, transparent | the same |
| `backdrop` | `logicalHeight` rows at 16:9, opaque | the same |
| `texture` | `ArtTexels.density` x 2 m to a power of two, at most `maxSize`; without texels, `unit` x 2 m of pixel art | the same |

Pixel art is drawn at the whole multiple of its grid nearest 1024 px
(`image.grid.scale`) and textures sampled `nearest` likewise, so the cook's
dominant-colour downscale (docs/art.md §2.3) lands on the grid; without a
grid an image is 1024 x 1024 (a backdrop 1536 x 864). A model's texture is
`ArtTexels.density` x its size to a power of two, at most `maxSize` (1024
when neither is set). On the command line: `bun scripts/brief.ts <kind>
"<subject>" [--project <dir> | --look <slug>] [--reference <path>]...
[--metres <n>] [--json]`.

Outside the engine repository the presets and palettes are read from
`presets/` (copied from `templates/looks` by `scripts/pack.ts`, not
mounted for the platform), the catalogue from `looks/` and the previews
from `previews/`.
