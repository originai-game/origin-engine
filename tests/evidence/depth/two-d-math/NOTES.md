# S9 `two-d-math`: evidence

Branch `depth/two-d-math`, rebased onto main `24116264` (wave 1 merged).
Every frame here is a lavapipe frame (`vulkan-cpu`, llvmpipe 15, this orb).
The Metal reference of the gfx test is owed by the coordinator.

## Commits

| Commit | What |
|---|---|
| `c78af4eb` | The math layer: IDL `math` structs Box3, Sphere, Plane, Frustum and `host false` interfaces Vector3Math … ColorMath; `packages/engine/src/math.ts` (`vec3 vec2 quat mat4 box3 sphere plane frustum ray color`); `tests/math.test.ts` (104 cases against three.js's documented results). og-idl: a `host false` method gets no binding symbol. |
| `f7483859`, `8dc5999f` | og-render: `RenderScene::sprite_meshes` (`SpriteMesh`, `SpriteMeshVertex`), drawn by `sprite_mesh.rs` + `shaders/sprite_mesh.wgsl` in the sprite order (`RunSource::Mesh`), lit like the sprites of their layer. The sprites.rs test that failed after the rebase expected 4 runs; layers 0 and 1 of its sprites batch into one run, so 3 is right. |
| `805c46ca` | og-anim: `rig2d.rs` (the `type rig2d` text, pose, triangles, arithmetic-only sincos/atan2; the module docs give the grammar and the Spine converter mapping) and `skeleton2d.rs` (og.Skeleton2D, og.Skeleton2DEvent, og.IkTarget2D, system `og.skeleton2d` with its state block). |
| `3f2d2a37` | IDL: `og.load.rig2d`, `og.anim.rig2dInfo`, `og.anim.clip(Rig2DInfo, name)`, handle Rig2D, Rig2DInfo/Rig2DClipInfo; og-cook: `Kind::Rig2d`, the `og.Rig2D` blob, the DragonBones 5.5 import (`dragonbones` + `atlas` lines). |
| `50253ede` | og-runtime: og.skeleton2d installed with the feel modules, Skeleton2DEvent registered beside FlipbookEvent, rigs replayed on reload, extraction (`skeleton2d.rs`) draws each skeleton as one SpriteMesh; both hosts; TS runtime `og.load.rig2d`, `og.anim.rig2dInfo`. |
| `3196325e` | templates/game-2d: the lamp keeper (`assets/rigs/keeper.ogt` + `.png`), `spawnKeeper`, the `Plant` system, the `keeper:<x>` command, `tests/keeper.og.ts`; the codec stores Rig2D handles; `bun:test` declarations gain `test.each`. |
| `97e97a81` | og-tests: native scene `skeleton-2d` and `skeleton_2d_draws_rigs_as_sprite_triangles` (reference `skeleton-2d.png`). |
| `1aa5d870` | docs/design.md §2.7 and §3.2; og-game §9 "2D skeletons: the rig2d grammar"; og-core §6 the math functions. |

## Frames read

**`game-2d-keeper-tick60.png`**: `og run templates/game-2d --headless
--commands "1.0:screenshot:keeper.png;…"`, the screenshot written at tick 60.
The rooftop level as before (courier at the spawn, skyline, coins, HUD).
Right of the courier the lamp keeper stands on a brown plank ramp rising
12° from the roof: brown hair, pale face facing left (flipX), teal coat with
gold buttons, the lantern glowing yellow at the hand and lighting the coat
and legs warm, a crimson cape hanging behind (right) of the body, both
boots on the plank with the knees slightly bent, the back leg dimmer (its
slot colour). The cape is a weighted mesh of 12 vertices over the hip and
two cape bones.

**`game-2d-keeper-tick90.png`** (tick 90, `.amp` zoom of both side by
side): the head has turned, the hip has dropped, and the cape's hem has
flared outward and lower as the idle clip's cape-bone and deform keys
move it, while both boots stay on the plank. The legs bent differently to
keep the feet there.

The dumps from the same run: both `og.IkTarget2D` have `reached: true`,
with `error` ≤ 1e-7. Their targets are `(5.85, 2.406956)` and `(6.15,
2.470724)`, the ramp's top plus the ankle as computed by hand. After
`keeper:7` they are `(6.85, 2.619516)` and `(7.15, 2.683284)`, still
reached. `Game.inhales` is 2 at 2.1 s (the event at 0 of a 2 s loop, twice).

**`skeleton-2d.vulkan-cpu.png`**: `og-gfx skeleton-2d`. Three keepers on
three planks (flat, rising 20°, falling 20°) at idle times 0, 0.5 and 1,
the third mirrored. Each foot is on its plank at that plank's slope height,
the knees bent. The capes hang left of the two facing right and right of
the mirrored one, and their shapes differ with the clip time. The middle
keeper, nearest the warm Light2d, is lit yellow; the outer ones are under
the cool ambient. Stats: sprites 3, sprite_draws 4 (one plank run, one
triangle run per rig), lights_2d 1. The test passed against this frame
captured locally (the capture was deleted; nothing is committed under
`references/`).

## Shared files touched

- `crates/og-api/api.ogt` (math structs and interfaces; `load.rig2d`
  before AssetReady; `anim` additions) and the generated outputs (`api
  gen`, `api gen --wiki`, `api check` clean); `crates/og-api/tests/modules.rs`.
- `crates/og-idl/src/gen/mod.rs` (`host false` interface methods).
- og-render: `sprites.rs` (`RunSource::Mesh`, the test's expected runs),
  `lib.rs` (the renderer field, group creation, upload, the draw branch),
  `scene.rs` (`sprite_meshes`), `overlay.rs` (match arms), `shaders.rs`
  (registry entry); new `sprite_mesh.rs`, `shaders/sprite_mesh.wgsl`.
- og-cook: `cook/mod.rs`, `format.rs`, `runtime.rs`, `cook/flipbook.rs`
  (`assets_root`, `resolve` pub(super)); new `cook/rig2d.rs`.
- og-runtime: `engine.rs`, `feel.rs`, `extract.rs` (`Hierarchy::parent_world`,
  the sprite-mesh clear), `codec.rs`, `lib.rs`; new `skeleton2d.rs`.
- Both hosts' `host.rs`.
- `packages/engine/src`: `index.ts`, `runtime.ts`, `env.d.ts`; new `math.ts`.
- og-tests: `scenes.rs`, `lib.rs`, `tests/gfx.rs`; new
  `skeleton_scenes.rs`, `assets/rig2d/`.

## Owed and left out

- **Metal reference** `crates/og-tests/references/metal/skeleton-2d.png`
  is owed by the coordinator (`OG_GFX_CAPTURE=skeleton-2d`). The scene is
  native-only (`NATIVE_SCENES`), because og-anim poses it.
- **Not run (as briefed)**: `og test` (the keeper playtest's script was
  run through `og run --headless --commands` and its checks verified by
  hand on the dumps), scripts/check.sh, web-reference, wasm and release
  builds. The headless run used an addon built from this branch
  (`OG_NATIVE_ADDON`); `packages/engine/native/` was not rebuilt.
- **Spine import**: left out by design (its licence). `rig2d.rs`'s module
  docs give the converter mapping.
- **Not imported from DragonBones**: skew (only skX = skY rotation),
  colour offsets, zOrder timelines, IK timelines, nested armatures,
  bounding boxes. Bézier curves play as linear.
- **IK** is one or two bones (no FABRIK chains in 2D). The clip's `loop`
  line is informational: `Skeleton2D.loop` decides.
- **Not in the frame**: the keeper does not walk. The brief's character
  "walks", but it stands on its ramp with the idle clip and moves only
  with `keeper:<x>`. The IK planting on a slope and the mesh-deformed cape
  are shown; a walk cycle is a clip to add.
