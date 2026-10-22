# The Origin Engine scaffold

What `npm create @origingame/game` (`@origingame/create-game`) produces: a copy
of the engine repository's `templates/game/` with the project's `name` and
`id` filled in. The platform mounts it as the scaffold overlay
`game-scaffold/origin-engine-ts/` (`knowledge.json`); it stands alone, never
on top of the Web scaffold's `shared/`.

| Path | Role |
|---|---|
| `project.ogt` | The manifest every `og` command reads: `name`, `id`, `engine` (the pinned `@origingame/engine` version), `entry`, `profile` (`"2d"` or `"3d"`), `tick`, `seed`, `scenes` (name to scene path under `assets/`), `scene` (the first one) and `publish` (`probe player`: a publish build keeps only the player-facing probe members; `out` the build directory) |
| `src/main.ts` | The game entry: components, systems and `export default og.game({...})`, starting from the design's hello world and loading `scenes/main.ogt` |
| `package.json` | `@origingame/engine` pinned to the same version as `project.ogt`; scripts `dev`, `start`, `build`, `publish-build`, `cook`, `test` calling `og` |
| `tsconfig.json` | Strict TypeScript over `src/` and `tests/` |
| `tests/smoke.og.ts` | The first playtest `og test` runs headless through `og run --commands` |
| `assets/` | Source assets with their import settings and licences: `scenes/main.ogt`, the game's look `looks/<id>.ogt` (extending a catalogue preset; `GameDesc.look` names it, og-taste §12), two glTF models, two textures (one normal map), a sound and a CJK-subset font |
| `LICENSES.md` | One row per asset: licence and origin |
| `.gitignore` | `cooked/`, `dist/` and `node_modules/` are build output |
| `.gitattributes` | Media (models, textures, audio, fonts) through Git LFS; `.ogt` and `.ts` text with LF endings. The template stores it as `gitattributes`, renamed by the scaffolder and the mount, because the engine repository keeps the template's media as plain blobs |

`origin-engine.json` pins the engine package and version, the template and
the probe protocol (5, wire 1). `distribution.json` names every binary file
(models, textures, audio, fonts) with its size, SHA-256 and licence, because
the platform's knowledge payload is UTF-8 text only; the scaffolder copies
binaries from the published template and checks the digests.
`bun scripts/scaffold.ts --write` regenerates it after a deliberate asset
change, and `bun run test` fails while it is stale.
