<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `start` guide: Making a game on Origin Engine

Start here for any Origin Engine game: its connected menu shell and create block, where the API is answered, the project's shape, and which guide covers what.

<a id="guide-start-1"></a>
### `start §1` The skeleton and the create block

Creation (`og create`, from `@origingame/create-game`) writes a project
with a connected UI shell for every genre and view: `project.ogt`,
`src/main.ts`, `src/shell.tsx`, `src/shell-state.ts`, `src/shell-config.ts`,
`src/shell-locales.ts`, string tables, `tests/smoke.og.ts`, `package.json`
and `tsconfig.json`. Title, main menu, loading, pause (with a Quit to menu
confirmation), settings, controls rebinding, save/load and credits are
built-in screens with working navigation, world-snapshot saves and
persistent volume, text size, motion, language and binding settings. Their
motion (screens fading in and out, entries entering in turn, the menu's
bar gliding to the focused entry) comes from the
pack's motion tokens and follows the player's Reduce motion setting. The shell is
only built-in screens and controls, in the dark scheme: title and main menu
stand on the game's key art, else on whatever the game draws; pause,
settings and save/load stand on the frozen game frame. There is no sample
level, gameplay, rule set or balance to adapt.

Keep this shell and customize it first: change copy in `assets/strings/`,
load key art and a logo in `start` and pass their texture handles to
`setShellArt(world, { keyArt, logo })` before `startShell`. Layout, focus and motion come from the built-in screens; restyle them through the look's UI pack and its tokens, not by hand. Avoid
rebuilding entire screens with View/Text. Add game-specific screens beside
them. Initialize gameplay before `startShell(world)` in `src/main.ts`:
that call captures the New game reset point. Bind gameplay systems to
`state: "session", value: "playing"`; the shell freezes scaled simulation
and pauses the flow outside play, while UI and platform answers keep ticking.

A tool creates a game without questions: `og new <dir> --look <slug>
--premise "<one line>" --no-prompt` (it never prompts without a terminal
either), with `--force` to write into a nonempty directory and `--id` when
the directory's name is not a game id. `--look` takes one of the
catalogue's styles with a preset (an unknown slug exits 1 and lists them);
the other choices are inferred from the premise unless given. It exits 0
having written project.ogt with `engine "<this version>"`, `look
"looks/<id>.ogt"` and the `create` block, and `assets/looks/<id>.ogt`
extending the chosen look.

The first choice is `--form`: `2d`, `2.5d` or `3d`, the world's form.

- A game's `view` is one of its form's: `side-2d` or `top-down-2d` for
  2d, `isometric-2.5d` for 2.5d, `third-person-3d` or `first-person-3d`
  for 3d. Without one it is the genre's usual view in that form. A form and
  view that disagree, or a look that does not draw them (pixel in 3d), exit
  1 naming the looks that do.
- `--genre interactive-film` writes the same skeleton plus a flow to
  replace: the story in `assets/story/film.yarn`, a segment per flow node
  in `assets/segments/` and a placeholder stream in `assets/clips/`, run
  by project.ogt's `flow "story/film.yarn"` line, which the engine starts
  and draws with its FlowScreen underneath the same shell. The shell holds
  the flow at the opening frame until New game or Continue, and pauses it
  in menus. Its form is 3d
  unless named, the world its live scenes and levels draw in, and its view
  may be left out. Read the flow guide next (`og api show flow`): it
  covers the story, segments, choices, saves and route tests.
- `--variant <name>` picks one of the look's variants (looks), written into
  the look file. The look draws in its own mood, the preset's or the
  variant's; `--mood` is refused (to change it, write `mood "<word>"` in
  the look file, looks §1).

`og new --creation` prints every choice, its labels and the rules between
them as JSON, and writes nothing.

The choices made at creation sit in project.ogt's `create` block: `form`,
`genre`, `view`, `look`, `variant`, `ui`, `platform`, `input`,
`players`, `lang`, `music` (a film's genre is `interactive-film`, and it
has no `view` or `players`).
Creation turns some choices into configuration; the rest remain the brief:

- `form` selects the engine's 2D or 3D profile. `platform` and `input`
  configure the generated shell's device defaults: phone/touch keeps the
  UI at touch scale, keyboard and gamepad retain arrow/d-pad focus and
  Escape/Start pause. Screens adapt to the actual viewport and device,
  including safe areas, rather than forcing a fixed render resolution.
  `players online-lockstep` does not open a room: the game implements its
  rules and networking.
- Treat them as the brief: a `platform phone-portrait` game is laid out
  for 390×844 and touch first; `input gamepad` means every screen is
  reachable by pad focus; `players online-lockstep` means the whole
  simulation follows determinism §3 from the first system written.
- When the user's request contradicts the block, the request wins: edit
  the block so it stays the record of the brief.
- `look`, `variant` and `ui` are also written into `assets/looks/<id>.ogt`,
  which project.ogt's `look` names; that file is real configuration
  (looks §1). `lang` installs each language's font pack, which og.i18n
  chains with no code. Creation writes and imports each requested locale's
  shell table and selects the first language; available engine translations
  seed the tables, otherwise English seeds the entries for translation.
  Complete those translations and add the game's own strings before release.
  Changing the block later does not regenerate authored files: update
  `src/shell-config.ts`, locale imports and the look file with it.

<a id="guide-start-2"></a>
### `start §2` The order of work

1. Read the create block, customize the connected shell's copy and art,
   and run title → main menu → play → pause → main menu. Check settings
   and save/load at the target sizes and input modes before adding gameplay.
   Restate the brief as the playable loop: what the player does every few
   seconds, what ends a run.
2. Write the smallest loop that plays inside the shell's `playing` state:
   entities, one system, input, a lose or win state. Run it
   (`og run --headless --commands ...`) before adding world art.
3. Write its invariants and the playtest that drives it (testing §2)
   while the rules are fresh: the playtest is how every later change is
   known not to break the loop.
4. Refine the look (`looks`, `art-review`), add game-specific UI (`screens`),
   motion and feel (`animation`), each checked as testing §5 describes.
5. Keep `og check` and `og test` green after every step. A game is done
   when its loop, its screens and its look hold under testing §5, not
   when it compiles.

<a id="guide-start-3"></a>
### `start §3` Finding the API

Never write API from memory of other engines. The answers, in order:

- `og api find <words>`: matching entries, one line each.
- `og api show <name>`: one entry in full (`og api show og.physics.castRay`).
- `og api show <module>`: a module's overview and its members.
- `og api show <guide>`: one of the guides of §5 (`--guide` when a module
  has the same name).
- `og api list --kind file`: the source-file contracts the cook accepts;
  `og api show file:terrain` describes one, and `og api find erosion
  --kind file` searches their fields. Read `generating` for designed inputs
  and deterministic TS generators executed by cook.
- `node_modules/@origingame/engine/types/index.d.ts`: the generated types
  the type checker uses; what is not there does not exist.

The vocabulary follows three.js where it fits (Vector3, Quaternion, a
material's `color`, `roughness`, `metalness`). The differences that bite:
the world is ECS (components on entities, systems run each tick), not a
scene graph of objects with methods; there is no render loop to write;
assets are named by their path under `assets/`, cooked ahead of time.
`og migrate three <file.ts>` rewrites three.js code where the mapping is
mechanical and lists every line it is not, each with its section of the
`three-to-og` guide.

<a id="guide-start-4"></a>
### `start §4` The project's shape

- `src/` is game code only: TypeScript against `og`. No Rust, no `cargo`,
  no native addon; a feature that seems to need one is an engine request.
- `assets/` holds sources; `og cook` writes `cooked/`. Never edit or
  commit `cooked/`, `dist/` or `og-out/`.
- Obtain or generate the game's images and models, save the actual source
  files under `assets/`, cook them, then load and place them with `og.load`.
  Follow `art-review §2` through capture and review; a downloaded image or
  a successful load is not yet a visible asset in the game.
- For generated maps, meshes, scatter or audio, use the cook's existing
  mechanisms and deterministic TS inputs (`generating`), not an external
  script that reimplements the engine's generators. Keep the design inputs
  beside the assets they generate; cook caches the result.
- Simulation state lives in components and resources; UI state lives
  in named scope stores. Module-level values are outside the snapshot,
  so saves, restores, hot reload and replays lose them.
- Tuning values (speeds, costs, timings) go in components or resources
  with reflected ranges, not literals in systems: they show in the
  inspector and Tune, and the default invariants check their ranges.
- Splitting `src/` by feature (player, enemies, ui) with one file per
  system group keeps hot reload fast and diffs small.

An entity's name is an address, not its identity. Use `world.setName` to
rename a live entity without replacing its components or handle; `find`
and `nameOf` see it immediately. The ECS applies a system's name changes
when that system returns, so later systems and their snapshots see them.

A system's query matches entities carrying **all** component types in its
`reads` and `writes`; those lists are not two independent populations.
Do not add another kind's tag just to read it: that narrows the query.
Process disjoint populations in separate ordered systems, passing facts
through a reflected resource or event. Use `world.get` for a known entity
with its component declared in the system's access, or `world.spatial`
for nearby entities that have a Transform (`og api show World.spatial`).
`world.query` is for callbacks and commands between ticks, not a second
query inside a system. This restriction preserves the declared scheduler
and column-borrow contract.

Put gameplay initialization in `GameDesc.start` and named UI store
registration in `GameDesc.ui`. The engine calls `ui` after `start` and
again after a hot-reload restore, never per tick. It does not rerun
`start` on reload. Read the world in `ui` to choose initial values, but
do not write gameplay state there. Registration refuses gameplay writes
and checks that `world_hash` is unchanged. A kept store ID retains its
value and receives its new callback. New IDs use their initial values;
removed IDs lose their subscriptions. Duplicate IDs and incompatible
types are errors. UI values travel in the presentation checkpoint,
outside `world_hash`. Send gameplay actions through `og.input.intent`
with a reflected event type so recorded input can replay them without
the presentation checkpoint. A fresh-process restore runs `start`, then
`ui`, then restores saved values by ID.

<a id="guide-start-5"></a>
### `start §5` The map

`og api show <guide>` prints a guide (`--guide` for one named like a
module: `og api show physics --guide`) and `og api show "<guide> §N"` one
section of it; `og api find` returns sections beside the API entries.
`og api show <module>` lists the module's core members, the ones games use
most; `--all` lists every member.

| To | Read | Start with |
|---|---|---|
| begin a game, find the API | `start` | `og.game`, `defineComponent`, `defineSystem` (`og api show system`) |
| write systems that replay and save | determinism §1 | `World`, `og.Time`, `og.Rng` |
| read a unit, an angle, a row or a name the right way round | `conventions`, `facing` | `og.transform.face`, `og.transform.face2d` |
| test the loop, capture & measure frames | `testing`, `playtest`, `captures`, `perf` | `runPlaytest` (`@origingame/engine/playtest`), `og.assert.always`, `og.assert.sometimes`, `og.assert.reachable` |
| save and load progress | `saves` | `World.snapshot`, `World.restore`, `og.platform.saveSlot`, `og.platform.loadSlot` |
| call the platform (store, scores, purchases) | saves §7 | `og.platform.storeGet`, `og.platform.result` |
| read the player's input | `input` | `og.Input`, `og.input.action`, `og.Gamepad` |
| move bodies, cast rays, control a character | `physics` | `og.RigidBody`, `og.Collider`, `og.CharacterController`, `og.physics.castRay` |
| break things | physics §9, determinism §4 | `og.Destructible`, `og.destruction.explode` |
| compose combat, abilities, items and interactions | `combat`, `abilities`, `items`, `interaction` | `og.systems.combat`, `og.systems.abilities`, `og.systems.inventory`, `og.systems.interact` |
| author reflected data tables | `data` | `og.table`, `type table` |
| point the camera | `camera` | `og.Camera`, `og.Follow2D`, `og.camera.shake` |
| play sound and music | `audio` | `og.AudioSource`, `og.audio.createSound` |
| play online or build shared worlds | `networking`, `shared-worlds`, determinism §3 | `og.Lockstep`, `og.PlayerInput`, `og.net.submit`, `og.shared` |
| load models, textures, sounds & author `.ogt` assets | `loading`, `assets` | `og.load.gltf`, `og.load.texture`, `og.load.flipbook` |
| take generated key art, logos, icons and models into a reviewed game frame | art-review §2, screens §1, loading §2, `captures` | `og cook`, `og.load.texture`, `og.load.gltf`, `World.instantiate` |
| find cookable file types or design generated content in TS | `file-types`, `generating` | `og api list --kind file`, `og api show file:terrain`, `og script` |
| choose the look | `looks`, `art-review` | `og.render.applyLook`, `og.Stylise` |
| light and render the scene | `render`, looks §3, looks §5 | `og.Mesh`, `og.DirectionalLight`, `og.RenderSettings`, `og.render.standardMaterial` |
| set the time of day, the season and the weather | render §7, `seasons` | `og.DayNight`, `og.Season`, `og.Weather`, `og.atmosphere.season` |
| draw 2D sprites and lights | looks §13, render §11 | `og.Sprite`, `og.Light2D`, `og.Lighting2D` |
| make models in code or with `og model` | `models`, `model` | `type model`, `.shape.json`, `og model` |
| animate characters and props | `animation` | `og.AnimationPlayer`, `og.anim.clip`, `og.MotionAgent`, `og.tween` |
| build terrain, tilemaps, districts, paths, AI | `world`, `districts` | `og.Terrain`, `og.Tilemap`, `og.tilemap.setTile`, `og.nav` |
| add effects | `effects`, `effect-files`, `effect-packs`, `pixel-vfx` | `og.vfx.spawn`, `og.ParticleSystem` |
| write screens, themes, tokens and HUDs | `screens`, `ui-tsx`, `theme-tokens`, `themes`, `components`, `ui-materials` | `og.ui.configure`, `og.ui.scope`, `mount` from `@origingame/engine/ui` |
| translate text | `localization`, screens §6 | `og.i18n.t` |
| write dialogue, quests and interactive video | `yarn`, `flow`, `video-to-game` | `og.story`, `og.flow`, `og video study` |
| port three.js code | `three-to-og` | `og migrate three` |
| explore a game's states | `explore` | `og explore` |

<a id="guide-start-6"></a>
### `start §6` Pitfalls

- Platform services (saves, purchases, leaderboards, identity) go
  through og.platform only; never `localStorage`, `fetch` or a third-party
  SDK. The host may lack a service: check before relying on it, and keep
  the game playable without it.
- Audio on the web starts only after the player's first input; a title
  screen that waits for a press hides this.
- Load assets in `start` or through streaming groups; a load inside a
  per-tick system reloads nothing (loads are cached by path) but costs a
  lookup every tick and hides the dependency from the cook.
- `og upgrade` moves the engine pin and applies the changelog's codemods;
  never edit the pin by hand.
