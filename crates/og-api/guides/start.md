# Making a game on Origin Engine

Start here for any Origin Engine game: how a game begins from the empty skeleton and its create block, where the API is answered, the project's shape, and which guide covers what.

## 1. The skeleton and the create block

Creation (`og create`, from `@origingame/create-game`) writes one empty
project for every genre and view: `project.ogt`, `src/main.ts` exporting
`og.game({})`, `tests/smoke.og.ts`, `package.json`, `tsconfig.json` and an
empty `assets/`. There is no sample scene, gameplay or art to adapt: the
game is written from zero with the API.

The choices made at creation sit in project.ogt's `create` block: `genre`,
`view`, `look`, `mood`, `ui`, `platform`, `input`, `players`, `lang`,
`music`. They are constraints and hints, not settings:

- Nothing reads them at run time. `view side-2d` does not make the game 2D,
  `players online-lockstep` does not open a room, `lang zh` installs no
  font. The game's code and assets implement each choice.
- Treat them as the brief: a `platform phone-portrait` game is laid out
  for 390×844 and touch first; `input gamepad` means every screen is
  reachable by pad focus; `players online-lockstep` means the whole
  simulation follows determinism §3 from the first system written.
- When the user's request contradicts the block, the request wins: edit
  the block so it stays the record of the brief.
- `look`, `mood` and `ui` are also written into `assets/looks/<id>.ogt`,
  which project.ogt's `look` names; that file is real configuration
  (looks §1). `lang` installs each language's font pack, which og.i18n
  chains with no code; each language still needs its string table
  (screens §6). `og check` warns while a language has no font for its
  script.

## 2. The order of work

1. Read the create block and restate the brief as the playable loop: what
   the player does every few seconds, what ends a run.
2. Write the smallest loop that plays: entities, one system, input, a
   lose or win state. Run it (`og run --headless --commands ...`) before
   adding art.
3. Write its invariants and the playtest that drives it (testing §2)
   while the rules are fresh: the playtest is how every later change is
   known not to break the loop.
4. Then the look (`looks`, `art-review`), the UI (`screens`), motion and
   feel (`animation`), each checked as testing §5 describes.
5. Keep `og check` and `og test` green after every step. A game is done
   when its loop, its screens and its look hold under testing §5, not
   when it compiles.

## 3. Finding the API

Never write API from memory of other engines. The answers, in order:

- `og api find <words>`: matching entries, one line each.
- `og api show <name>`: one entry in full (`og api show og.physics.castRay`).
- `og api show <module>`: a module's overview and its members.
- `og api show <guide>`: one of the guides of §5 (`--guide` when a module
  has the same name).
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

## 4. The project's shape

- `src/` is game code only: TypeScript against `og`. No Rust, no `cargo`,
  no native addon; a feature that seems to need one is an engine request.
- `assets/` holds sources; `og cook` writes `cooked/`. Never edit or
  commit `cooked/`, `dist/` or `og-out/`.
- Scripts that generate sources (art, maps, tables) go in `tools/`.
  Nothing there is cooked or bundled, and a generated file names its
  script in its first comment.
- Simulation state lives in components and resources; UI state lives
  in named scope stores. Module-level values are outside the snapshot,
  so saves, restores, hot reload and replays lose them.
- Tuning values (speeds, costs, timings) go in components or resources
  with reflected ranges, not literals in systems: they show in the
  inspector and Tune, and the default invariants check their ranges.
- Splitting `src/` by feature (player, enemies, ui) with one file per
  system group keeps hot reload fast and diffs small.

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

## 5. The map

`og api show <guide>` prints a guide (`--guide` for one named like a
module: `og api show physics --guide`) and `og api show "<guide> §N"` one
section of it; `og api find` returns sections beside the API entries.
`og api show <module>` lists the module's core members, the ones games use
most; `--all` lists every member.

| To | Read | Start with |
|---|---|---|
| begin a game, find the API | `start` | `og.game`, `defineComponent`, `defineSystem` (`og api show system`) |
| write systems that replay and save | determinism §1 | `World`, `og.Time`, `og.Rng` |
| test the loop, measure frames | `testing` | `og.assert.always`, `og.assert.sometimes`, `og.assert.reachable` |
| save and load progress | `saves` | `World.snapshot`, `World.restore`, `og.platform.saveSlot`, `og.platform.loadSlot` |
| call the platform (store, scores, purchases) | saves §7 | `og.platform.storeGet`, `og.platform.result` |
| read the player's input | `input` | `og.Input`, `og.input.action`, `og.Gamepad` |
| move bodies, cast rays, control a character | `physics` | `og.RigidBody`, `og.Collider`, `og.CharacterController`, `og.physics.castRay` |
| break things | physics §9, determinism §4 | `og.Destructible`, `og.destruction.explode` |
| point the camera | `camera` | `og.Camera`, `og.Follow2D`, `og.camera.shake` |
| play sound and music | `audio` | `og.AudioSource`, `og.audio.createSound` |
| play online | `networking`, determinism §3 | `og.Lockstep`, `og.PlayerInput`, `og.net.submit` |
| load models, textures, sounds | `loading` | `og.load.gltf`, `og.load.texture`, `og.load.flipbook` |
| choose the look | `looks`, `art-review` | `og.render.applyLook`, `og.Stylise` |
| light and render the scene | `render`, looks §3, looks §5 | `og.Mesh`, `og.DirectionalLight`, `og.RenderSettings`, `og.render.standardMaterial` |
| draw 2D sprites and lights | looks §13, render §11 | `og.Sprite`, `og.Light2D`, `og.Lighting2D` |
| make models in code | `models` | the `type model` asset |
| animate characters and props | `animation` | `og.AnimationPlayer`, `og.anim.clip`, `og.MotionAgent`, `og.tween` |
| build terrain, tilemaps, paths, AI | `world` | `og.Terrain`, `og.Tilemap`, `og.tilemap.setTile`, `og.nav` |
| add effects | loading §11, `pixel-vfx`, `effect-packs` | `og.vfx.spawn`, `og.ParticleSystem` |
| write screens and HUDs | `screens`, `components`, `ui-materials` | `og.ui.configure`, `og.ui.scope`, `og.ui.push` |
| translate text | `localization`, screens §6 | `og.i18n.t` |
| write dialogue and quests | `yarn` | `og.story` |
| port three.js code | `three-to-og` | `og migrate three` |
| explore a game's states | `explore` | `og explore` |

## 6. Pitfalls

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
