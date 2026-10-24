# threejs-migrate: the three-to-og guide and `og migrate three` (round 3)

Branch `r3/threejs-migrate`, cut from `34db4fde` (origin/main). The brief:
the `threejs-migrate` row of docs/phase-1.md, "Plan 2026-10-24, round 3",
plus two doc rows in packages/knowledge (og-build points to `og compare`,
og-model names the `type model` route).

## What landed

| Commit | What |
|---|---|
| `3da5b8a5` knowledge | og-build §4 (evidence discipline) gains the model review: the `view:` and `turntable:` commands, `og compare`'s IoU, ΔE00 and sheet, pointing to og-model §3 and §4. og-model §5 gains "The same model as an asset: `type model`": write a finished static prop as a `type model` asset (og-game §18), cooked with LODs, BVH and instancing, refused past the budgets, loaded with `og.load.gltf` of the `.ogt`. The factory is kept while a pass is open and compared against the asset before it is deleted. Pass 8's row and the skill's description name it too, and so does the README. |
| `8bb814ad` knowledge | `guides/three-to-og.md`, eleven numbered sections: the model side by side, the scene graph (entities, og.Parent), Object3D (og.Transform; a `lookAt` aiming an object's +Z is `lookAt(target, eye)`), geometry (every three.js class with its og.render call and what is not carried over), materials (Basic as `unlit`, Toon as `toon`, hex through `color.fromHex`), lights, the camera and the frame, loaders (og.load and the cook), the render loop (systems on the tick), picking (`og.render.raycast` with `camera.screenToRay`), the gaps by name, and `og migrate three`. `knowledge.json` mounts `guides/` at `game/skills/og-game/guides/`, `package.json` publishes it, and og-game §25 names the guide. The validator gains `GUIDES` and `GUIDE_MOUNT`. A guide is held to the skills' API names (og.*, Type.field, `key: value`), section references in both directions (`three-to-og §N` from a skill, `og-feel §N` from the guide), tools and wiki pages. An unlisted guide file, a guide og-game never names, a skill naming a guide that does not exist and an unmounted guide are refused. The section-reference walk moved into an exported `sectionRefs` and runs over skills and guides alike, with the same messages for the skills. |
| `cae26e1c` og-cli | `og migrate three <file.ts> [--out <file> \| --write] [--json]` (`crates/og-cli/src/migrate.rs`, the USAGE text, `main.rs`). Details below. |
| `fdb10005` knowledge | `GUIDE_CITERS` names `crates/og-cli/src/migrate.rs`: every `three-to-og §N` it writes must be a section of the guide (checked only in the repository; the published package has no crates). |
| (this commit) | The guide's §11 brought in line with what the command does (four light kinds; which renderer calls are deleted and which are listed), and these notes. |

### `og migrate three`

It reads the file as text. A logical line is physical lines joined while
a `(` or `[` is open; strings and comments are skipped when brackets are
matched. It is not a TypeScript parser, and a pattern that does not match
whole is left alone and listed, never guessed at.

- **Rewritten.** three's imports (`* as THREE`, named imports, and the
  `three/addons` and `three/examples` paths) become `import { og[, color][,
  quat] } from "@origingame/engine"`, and a `must` helper is added. The 16
  geometry constructors become their `og.render` call with the positional
  arguments named. Extra arguments og does not have are dropped and
  listed by name, and `undefined` arguments are skipped. TubeGeometry is
  rewritten when its curve is `new CatmullRomCurve3(points)`, and
  ExtrudeGeometry's options are spliced in, or spread when they are a
  variable. The six mesh materials become `standardMaterial`: Basic adds
  `unlit: true`, Toon `toon: true` and Lambert `roughness: 1`. Colour
  fields become `color.fromHex`, the side constants become strings,
  `vertexColors` is removed and other unknown keys are dropped and listed.
  `Vector2`, `Vector3`, `Quaternion` and `Color` become plain values, and
  `Shape`/`Path` become `og.Shape`/`og.Path2`. `MathUtils.degToRad` and
  `radToDeg` become arithmetic. `Mesh`, `Group`, `Object3D` and the
  directional, point, spot and rect area lights declared on their own line
  become a spawn named after the variable plus its component. A
  directional or spot light's `position.set` also aims it at the origin,
  three.js's default target. `add` becomes og.Parent; the scene's `add`
  of spawned entities is deleted; `remove` becomes `world.remove(…,
  og.Parent)` or `world.despawn`. `position`, `quaternion`, `rotation` and
  `scale` setters (`set`, `copy`, `setScalar`) become `world.set` of
  og.Transform, with `rotation` through `quat.fromEuler`. Mesh and light
  field assignments become `world.set` of the component. The cameras
  become `must(world.resource(og.Camera))` with `fov`/`near`/`far`
  (orthographic: `orthographic`, `height = top - bottom`), and their
  `position.set`, `lookAt` and `rotation.set` are rewritten. `aspect` and
  `updateProjectionMatrix` are deleted. `scene.background` becomes the
  camera's background, reached through the resource when the camera is
  declared later in the file (a TDZ otherwise). `TextureLoader` loads of
  a string literal become `og.load.texture` of the asset id. The scene,
  the renderer (`render`, `setSize`, `setPixelRatio`, `shadowMap`, colour
  space), `appendChild(renderer.domElement)` and `.dispose()` lines are
  deleted.
- **Listed and left** under `// og migrate: <what to do> (three-to-og
  §N)`: about forty three.js classes, each with its instruction
  (`manual_class`), and every later use of a variable holding one
  (`controls.update()`, `clock.getDelta()`). Also listed:
  `requestAnimationFrame`, `addEventListener`, `Math.random()`, in-place
  transform arithmetic (`rotation.y += …`), `lookAt` on an object, a Mesh
  or light inside an expression, a colour that is not a literal, a
  texture load with a callback, and a `scene.add` of something not spawned
  here (it is no longer silently deleted). So are the renderer's
  `setAnimationLoop`, `toneMapping` and `setClearColor`.
- Output goes to stdout (the list to stderr), to `--out`, or in place with
  `--write`. `--json` prints one `migrate` event: the counts, and the
  items with `line`, `kind`, `three`, `og` and `section`, plus `output`
  when nothing is written. Exit 1 for a file importing nothing from three,
  2 for usage.

## Tests, and how each fails on the old code

Everything here was run and passes. Each was checked to fail on the old
code, by stashing or by a mutation.

- **knowledge** `og-build points to og compare and og-model names the type
  model route`: with the two skills and the README stashed, it fails
  naming 8 missing phrases (checked).
- **knowledge** `the three-to-og guide maps three.js's classes and idioms
  and names the gaps` (36 names). With the guide absent it fails
  (`toBeDefined`). With the old validator and knowledge.json, the guide is
  not mounted, so it fails too (checked).
- **knowledge** `a guide is held to the skills' rules`: a misspelt
  `og.render.raycasts`, an unknown wiki page, its own §42, og-feel §99, a
  stray `guides/notes.md`, a skill naming `guides/three-to-ui.md`, og-game
  no longer naming the guide, and the guide unmounted. On the old
  validator it receives `[]` where 6 problems are expected (checked).
- **knowledge** `every section og migrate three cites is one the guide
  has`: a `three-to-og §12` added to a copy of migrate.rs is refused. With
  `GUIDE_CITERS` empty it fails (checked). It also asserts that the real
  source cites at least one section.
- **knowledge**: `bun run test` passes 48 tests, and `bun scripts/validate.ts`
  is valid (it was 44 at the base).
- **og-cli unit tests** (`migrate::tests`, 5). On the old code the module
  does not exist. Mutations checked:
  - `a_scene_file_is_rewritten_where_mechanical_and_listed_where_not`: a
    39-line scene (camera, renderer, a multi-line material, group, mesh,
    sun, ambient light, OrbitControls, a Clock loop). It asserts 14 exact
    output lines, 8 deleted fragments and the manual list `(line,
    section)`. Aiming the light the wrong way round
    (`lookAt(origin, p)`) fails it (checked). Swapping CylinderGeometry's
    `radiusTop`/`radiusBottom` in the table fails it (checked). The test
    uses 0.5 and 1.25, so a swap shows.
  - `named_imports_and_dropped_parameters`: named imports, the dropped
    BoxGeometry segment, the material's dropped `shininess`, PointLight's
    dropped decay, and `MathUtils.degToRad`.
  - `values_colours_and_loaders`: a Vector3 with a missing z, Quaternion's
    default w, Color with 3, 0 and 1 arguments, `#f80` expanded, a CSS
    name listed, asset ids stripped of `./assets/`, a load with a callback
    left, a tube along a CatmullRom curve and one along a variable, an
    extrude, and a Path.
  - `a_file_without_three_is_not_migrated`.
  - `the_tables_match_the_generated_api`: every og.render function, desc
    field, light field, og.Mesh field and og.Camera field the rewrite
    writes exists in `packages/engine/schema/api.schema.json`. The
    material list equals StandardMaterialDesc's fields, so a field added
    to the IDL passes through instead of being dropped.
- **og-cli CLI test** `migrate_three_rewrites_a_file_and_lists_what_is_not_mechanical`
  covers the binary: `--out` with `--json`, the counts, the manual item's
  line and section, the written file and the source left alone, exit 1
  for a plain file and 2 for `migrate babylon`. With the `migrate` arm
  removed from main.rs it fails on "unknown command" (checked).
- `cargo test -p og-cli`: 27 unit and 24 CLI tests pass.
  `cargo clippy -p og-cli --all-targets -- -D warnings` and `cargo fmt`
  are clean.

Not run, per AGENTS.md: scripts/check.sh, `og test` across projects,
web-reference, reference captures. Nothing here renders, so no frame is
owed. `og migrate`'s output was read by eye for the scene fixture (the
rewritten file in the test's assertions).

## Shared files touched

- `crates/og-api/api.ogt`: **untouched**, and so are the generated
  outputs (types, schema, wiki). Nothing to regenerate.
- `crates/og-cli/src/main.rs`: one `mod`, one match arm, the USAGE
  block, and the module doc's command list.
- `crates/og-cli/tests/cli.rs`: one test appended.
- `packages/knowledge/scripts/validate.ts`: the guides (constants,
  `guideFiles`, `sectionRefs` extracted from `coherence`, `apiNames`
  reading guides, `packageJson` requiring `guides` in `files`).
- `packages/knowledge/knowledge.json` (a mount), `package.json` (`files`,
  description), `README.md`, `tests/validate.test.ts` (5 tests appended).
- Skills: og-build, og-model and og-game (§25: three sentences naming
  the guide).

## Left out

- `og migrate` does not move setup into `GameDesc.start` or write systems:
  the output is not a game until the author does (the guide and the
  command's last line say so). Doing it mechanically would mean inventing
  the game's structure.
- No TypeScript parser. Patterns spanning several statements on one line,
  a declaration with a trailing comment, chained calls on a new object
  (other than `new TextureLoader().load`), and `const { a, b } = …`
  destructuring are listed or left, not rewritten.
- Not rewritten, only listed: `InstancedMesh` to entities, `OrbitControls`
  to og.OrbitCamera, `GLTFLoader` callbacks to `og.load.gltf`. Each needs
  the game's structure (which entity, which system), which the file does
  not show.
- `scene.fog`/`new THREE.Fog` and `setClearColor` could be mechanical
  (og.Fog's fields, the camera's background). They are listed because
  og.Fog needs `enabled` and three's linear `near`/`far` is not og's
  exponential default. Easy to add if wanted.
- The guide calls og's light intensities "not calibrated" against
  three.js's. No measurement backs a conversion factor, so the guide
  gives none.

## Decisions for the coordinator

1. **Where the page lives.** It is `guides/three-to-og.md`, mounted at
   `game/skills/og-game/guides/` (inside og-game's pack, so the platform's
   pack hashes and the mount script's ownership need no change), not a
   twelfth skill and not a wiki page. The wiki is generated and refuses
   hand-written pages, and a skill would be one more entry in the runner's
   list for what is a lookup table. If you would rather it be a skill
   (`og-three`) so the runner offers it by description, it is a move plus
   `SKILLS` and the platform's existing-skill rule.
2. **`GUIDE_CITERS`** ties the knowledge package's test to a Rust source
   file in the repository (skipped in the published package). The
   alternative is a Rust test reading the guide; I chose the validator
   because it already owns the section-reference rules.
3. **Rewrites that change behaviour, flagged in the guide.** Mesh
   `receiveShadow` defaults on in og, and a three.js DirectionalLight with
   no `position.set` shines down -Z in og, not down from (0, 1, 0). The
   migrator does not insert compensating lines. Say if you want it to,
   for example by aiming every directional light from (0, 1, 0) at
   declaration.
4. Batch-3 release rows F2 and F4 are not touched here.

## Delivery

Branch `r3/threejs-migrate` pushed to origin; base `34db4fde`. Not merged.
