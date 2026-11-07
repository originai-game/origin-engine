# The Origin Engine scaffold

The scaffold is `packages/create/template/`: one empty project for every
genre and view. The platform mounts its text as
`game-scaffold/origin-engine-ts/`. It does not layer over the web scaffold.

| Path | Role |
|---|---|
| `project.ogt` | Name, id, engine pin, entry, tick rate, seed, and publish settings |
| `src/main.ts` | An empty `export default og.game({})` |
| `package.json` | Commands that call the installed `og` |
| `tsconfig.json` | Strict TypeScript over source and tests |
| `tests/smoke.og.ts` | A start-and-quit playtest (bun:test, `runPlaytest`) |
| `assets/.gitkeep` | Keeps the empty source-asset directory in the payload |
| `.gitignore` | Excludes cooked assets, build output, and local installs |
| `.gitattributes` | Routes future media through Git LFS |

`og create` fills the name and id and appends the agent's choices in a
`create` block. Genre, view, look, mood, UI style, platform, input, players,
languages, and music are constraints and hints. Create also writes
`assets/looks/<id>.ogt` with the preset, mood and UI, and names it in
project.ogt. Mounting the raw payload alone does not perform this step.
No gameplay or art is generated.

`origin-engine.json` pins the engine and probe protocol. `distribution.json`
keeps its schema and has an empty file list: the skeleton has no binary
assets. `bun scripts/scaffold.ts --write` regenerates that manifest.
