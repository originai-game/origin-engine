# The Origin Engine scaffold

The scaffold is `packages/create/template/`: one empty project for every
genre and view. The platform mounts its text as
`game-scaffold/origin-engine-ts/`. It does not layer over the web scaffold.

| Path | Role |
|---|---|
| `og-project.json` | Sole project marker and authority for name, engine pin and optional look |
| `project.ts` | `defineProject`: id, entry, tick rate, seed, modules and publish settings |
| `src/main.ts` | An empty `export default og.game({})` |
| `package.json` | Commands that call the installed `og` |
| `tsconfig.json` | Strict TypeScript over source and tests |
| `tests/smoke.og.ts` | A start-and-quit playtest (bun:test, `runPlaytest`) |
| `assets/.gitkeep` | Keeps the empty source-asset directory in the payload |
| `.gitignore` | Excludes cooked assets, build output, and local installs |
| `.gitattributes` | Routes future media through Git LFS |

`og create` fills the name and id and writes the agent's choices in
`project.ts`'s `create` object. Genre, view, look and variant, UI style, platform, input, players,
languages, and music are constraints and hints. Create also writes
`content/looks/<id>.ts` with the preset, variant and UI (it draws in the
preset's own mood), and names its canonical `base/looks/<id>` reference in
og-project.json. Mounting the raw payload alone does not perform this step.
No gameplay or art is generated.

`origin-engine.json` pins the engine and probe protocol. `distribution.json`
keeps its schema and has an empty file list: the skeleton has no binary
assets. `bun scripts/scaffold.ts --write` regenerates that manifest.
