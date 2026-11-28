# Origin Engine game project

Everything you edit is TypeScript. The engine is a prebuilt binary; never
write Rust or `.ogt` files.

## Layout

- `og-project.json`: the project marker (name, engine version, optional
  look reference such as `base/looks/<id>`, declared in `content/looks/<id>.ts`).
- `project.ts`: project configuration, `export default defineProject({...})`.
- `content/**/*.ts`: one asset declaration per file,
  `export default file.<type>({...})` from `@origingame/engine/author`.
  `content/a/b.ts` has the ID `base/a/b`; refer to it with `ref("base/a/b")`.
- `assets/`: raw files (images, models, audio, fonts). A raw file is cooked
  only when a `content/` file declares it with `file.import`.
- `src/`: game code; `src/main.ts` is the entry, UI is TSX.
- `tests/*.og.ts`: playtests.
- `.og/`, `cooked/`, `og-out/`, `dist/` are generated; do not edit them.

## Finding the API

Look things up when you need them instead of guessing:

- `og api find <words>`: search the API, file types and UI components.
- `og api show <name>`: one entry, e.g. `og api show og.physics`,
  `og api show file:terrain`, or a component name.
- `og api list --kind module|file|component`: everything of one kind.
- The types in `@origingame/engine` are the source of truth; `og check`
  reports what is wrong.

## Commands

- `og add content <module/id> <kind> --value '<json>'`: add a checked declaration.
- `og add source src/path.ts --from <file|->`: add game code (stdin with `-`); `--replace` updates an existing source/test file.
- `og add test tests/name.og.ts`: add a startup playtest; `--from` supplies your own.
- `og set <scene> <entity> <component> <field> '<json>'`: edit its source literal; computed values report their location without writing. `og set <content-id|project> <pointer> '<json>'` edits declaration or project configuration literals. All four accept `--json`.
- `og dev` (native) or `og dev --web`: run with hot reload.
- `og check`: type-check game code and declarations.
- `og test`: run tests and playtests.
- `og run --headless --commands "..."`: scripted run with screenshots and dumps.
- `og build`: export the web build to `dist/`.

Dev, run, test and build cook assets themselves.

## Rules

- Use the built-in UI components and the engine's default UI pack before writing your own UI. A look is optional: without one the engine renders realistically, with no stylised art contract. Genre never implies a look. Only apply a stylised look when the game asks for that style.
- Simulation is deterministic: use the engine's seeded randomness and tick
  time, never `Math.random()` or the wall clock in game logic.
