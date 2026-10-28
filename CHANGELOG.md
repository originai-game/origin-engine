# Changelog

One version across `@origingame/engine` (the knowledge inside it as
`knowledge/`), `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64`
(`@origingame/engine-win32-x64` from F7) and `@origingame/create-game`
(docs/release.md §3). Before 1.0.0 a minor bump (0.x.0) is a break: an
IDL member removed or renamed, a type, default or saved value whose
meaning changed, behaviour a game relied on changed; a game may need
edits to upgrade, and the section says which. A patch bump (0.x.y) is
everything compatible: new members that are off by default (an existing
game draws and runs as before), variants appended to an enum, fixes,
frames and speed. Either way every IDL member added, changed or removed
is one line here, written at the merge that changes
`crates/og-api/api.ogt`.

## Unreleased

The art contract (docs/art.md §2.1, stream A1): a game's style as data
every asset holds, beside the frame's look. Data only: nothing conforms to
it or reads it yet (the cook's conformance and the runtime defaults are
streams A2 to A4), so every existing game cooks and draws as before; the
simulation is unchanged (the golden replays). The new records join every
world's registry, so `world_hash` moves; the replays' portable hash does not.

- `og.ArtPalette` (new resource, module `art`): the palette the assets'
  texels sit on: `colors`, `strict`, `maxColors`, `rampSteps`, `accents`
  (palette indices kept for signals), `dither`; no constraint at zero.
- `og.ArtPixels` (new resource): the pixel grid of 2D art: `unit` (logical
  pixels per metre), `logicalHeight`, `spriteSizes`, `rotation`,
  `rotationSteps`, `snap`; no constraint at zero.
- `og.ArtTexels` (new resource): texel density on 3D surfaces: `density`,
  `maxSize`, `filter`, `mips`; no constraint at zero.
- `og.ArtShape` (new resource): the shape language of models:
  `propFaces`, `characterFaces`, `normals`, `normalAngle`, `bevel`,
  `voxel`, `silhouette`; no constraint at zero.
- `og.ArtLine` (new resource): the line the assets carry: `width`,
  `color`, `place`, `distanceFalloff`; no line at width 0.
- `og.ArtMotion` (new resource): `stepFps` (animation on twos or threes),
  `maxFrames` per clip, `squash`; no constraint at zero.
- `og.ArtCamera` (new resource): `projection`, `fixedAngle`, `pitch`,
  `yaw`, `fovMin`, `fovMax`, `pixelPerfect`; no constraint at zero.
- `og.ArtRotation` (new enum): free, nearest, cleanEdge, prerotated.
- `og.ArtFilter` (new enum): any, nearest, linear.
- `og.ArtMips` (new enum): any, full, none.
- `og.ArtNormals` (new enum): any, smooth, flat, angle.
- `og.ArtLinePlace` (new enum): outside, inside, both.
- `og.ArtProjection` (new enum): any, perspective, orthographic.
- `GameDesc.look`: left out or empty, project.ogt's new `look
  "looks/<id>.ogt"` line is applied when it has one (as `tick`, `seed`,
  `scenes` and `scene` default to the manifest); the two naming different
  looks is refused. A project without the line runs as before.
- `LookInfo.resources`: names the art resources a look sets too.
- `og.render.applyLook`: inside a system, an art resource the system does
  not declare among its writes is left as it is instead of refusing the
  look, so a system declaring only the frame's resources applies a preset
  that carries a contract as before.
- `og.load.look`: a look's blocks include the seven art resources, checked
  against their records as the frame's are.
- Looks: a look sets the art resources by block as it sets the render
  ones; the 37 presets in `templates/looks/` carry their catalogue style's
  values (from each entry's `palette` and `assets`).
- Cook: project.ogt's `look` names the look whose art contract the cook
  reads before any asset cooks; its report notes what the contract sets.
  `create` writes the line beside `GameDesc.look`.
- `og.ArtLine.brush` (stream A2): the brush the cook draws a 2D line
  with, of the new enum `og.ArtBrush` (diamond, circle, square); diamond
  at zero.
- **Breaking** (the release is 0.7.0, a minor): a project whose look sets
  art blocks cooks its assets conformed to them, so its frames change.
  Fix: keep it, or `conform off` per asset, project.ogt's
  `conform { off ... }`, or the preset's art blocks set to zero in the
  game's look.
- Cook, 2D conformance (stream A2, docs/art.md §2.3): with a look whose
  art blocks set a value, sprites, flipbooks, tile sets and UI images are
  conformed in the cook (the pixel grid by a dominant-colour downscale,
  1-bit alpha, the palette in Oklab with its dither, `maxColors` and
  `rampSteps`, the line, prerotated cells by cleanEdge, painted 2D in
  texel space), with an `<id>#index` texture for palette swaps beside
  them; sources are never written. `conform off` in import settings and
  project.ogt's `conform { off ... }` opt out; `og cook --report` lists
  every change and opt-out. A game whose look extends a preset with an
  art contract cooks its 2D art conformed (a behaviour change, for the
  release's breaking entry); without a look, or with every art block at
  zero, `cooked/` is byte-identical to before.
- Knowledge and create, the art brief (stream A5, docs/art.md §2.2):
  `artBrief(look, kind, subject)` in the knowledge's `scripts/brief.ts`,
  re-exported by `@origingame/create-game` (which now has a `main`):
  the request for one asset (`sprite`, `tileset`, `portrait`, `backdrop`,
  `prop`, `character`, `texture`, `icon`) from the flattened look and its
  catalogue entry: the prompt (direction, the palette as named colours with
  hex, the sizes from `ArtPixels` and `ArtTexels`, the line, the shape
  language, what to avoid), the references (the style's preview, the
  game's accepted assets) and the generator's parameters (transparent and
  size; the face budget and texture size of a model). `readLook(dir)`
  reads a project's look from disk, `presetLook(slug)` a preset's; both
  packages carry the presets and built-in palettes under `presets/`. A
  public contract (the knowledge README's §The art brief).

## 0.6.6 — 2026-10-28

No IDL member changed; the simulation is unchanged (the golden replays).

- Knowledge: six craft skills move from origin-agent into the knowledge
  package, mounted at `game/skills/<name>/` under the names games already
  pin: `architecture-discipline`, `component-forge`, `level-texturing`
  (with its `scripts/texture-level.ts`), `localization`, `recipe-craft`
  and `visual-acceptance`. They name no platform tool or service and
  point to the og-* skills and the wiki for every API; `localization` is
  the craft beside og-i18n's API. texture-level.ts's providers are now
  `handoff` (the default: it prints the view to paint and exits 3) and
  `http` (any OpenAI-compatible image endpoint, `--endpoint`,
  `OG_IMAGE_API_KEY`) beside `dashscope`; a run whose settings say `agent`
  or `gateway` resumes under the new names. The validator resolves a
  section reference to any skill it ships (`recipe-craft §2`, and
  `og-i18n §6`, which it read as the citing skill's own).
- Knowledge: `payload(roots)` and `defaultRoots(pkg?)` of the knowledge's
  `scripts/lib.ts`, with the `PayloadFile` and `Roots` shapes, are a
  public contract beside `installed`, `mount` and `summary` (AGENTS.md):
  the platform reads the knowledge straight from the installed
  `@origingame/engine` with `payload(installed(dir).roots)` instead of a
  mounted copy. Nothing about them changed.

## 0.6.5 — 2026-10-27

No IDL member changed; the simulation is unchanged (the golden replays).

- Fixes: `og check` on a TypeScript install that `bun x` left incomplete
  (tsgo, where tsrs has no package: Windows, Intel macOS). A lost
  `lib.d.ts` made tsgo panic with a Go stack trace, and a lost default lib
  read as `Cannot find global type` errors in the game. og check now
  verifies the install before tsgo runs, and reads tsgo's panic, and says
  which directory is incomplete and the fix: delete it or
  `bun pm cache rm`, then rerun. `og doctor` reports it as broken.
- Fixes: the knowledge package's `scripts/mount.ts` CLI takes an
  origin-agent checkout and mounts into its `knowledge/`, where the
  platform's knowledge moved from OriginGame's
  `packages/game-knowledge/production` in the agent-repo split. Its
  exports `installed`, `mount` and `summary`, which origin-agent's
  `mount-engine.ts` imports, are unchanged.
- Fixes: og-ui glass on WebGL2 in Chrome and Edge on Windows (ANGLE on
  D3D11) blurred the panel about twice as wide as metal, WebGPU and DX12.
  Each level of the glass pyramid is drawn from the one above, read
  through a view whose base mip level is that level, and took its tap
  spacing from `textureDimensions`, which ANGLE on D3D11 answers with
  level 2b's size for base level b. The pyramid's levels, and a view's
  glow chain (og.Glow in a viewport or texture view), which read their
  levels the same way, now take the size from a uniform; native frames
  are unchanged.
- Fixes: WebGL2's cold start on Windows (ANGLE on D3D11), where each lit
  program took 6 to 11 s to link. Programs now link in the background
  with `KHR_parallel_shader_compile` (wgpu-hal's GL backend reads a
  link's status when a pass first draws with it, ORIGIN.md patch 2), and
  the GL tier's lit programs leave the rect-light (LTC) code out until
  the scene has a rect light, through a pipeline constant. fx-showcase's
  cold first frame: 30.4–31.7 s → 7.0–7.5 s in Chrome 155, 30.3–31.2 s →
  7.6–7.7 s in Edge 155; its five stations 56–58 s → 16–17 s. WebGPU,
  metal and DX12 are unchanged and no frame moved
  (`tests/evidence/release/windows-x64-webgl2/COLDSTART.md`). The
  trade-off: on WebGL2, a scene that gets its first rect light during play
  links its lit programs again at that moment (about 5 to 8 s on ANGLE
  D3D11, waited for by the next lit frame); to pay it at load instead,
  put an `og.RectLight` in the first scene (one at intensity 0 counts).

## 0.6.4 — 2026-10-27

No IDL member changed; the simulation is unchanged (the golden replays).

- og: `og bake projection` (`init`, `plan`, `prepare`, `project`,
  `holes`, `export`), a gray level glb painted from views by an
  image model the caller runs, into one texture and a textured glb, on the
  CPU (docs/design.md §Projection painting).

## 0.6.3 — 2026-10-27

No IDL member changed. The simulation changed: the golden replays are
re-recorded (`unreleased.simulation` is `changed` in `changelog.json`).

- Fixes: cross-platform yaw drift. A `NavAgent` with `faceMovement` turned
  through glam's `Quat::slerp`, which uses the host's `sinf`, so its
  rotation differed by up to 16 ulp between Windows and Linux while it
  turned (0.6.2's golden replays on win32, ticks 62–65). The turn is now a
  yaw step through og-core's deterministic math, which moves the turning
  ticks' rotation by about an ulp on every host. `og.terrain.sample`'s
  `slope` uses og-core's `acos` too.
- Fixes (held): `camera.worldToScreen` and `camera.screenToRay`, which a
  system may call inside the tick, answer the same bits on every host.
  Their perspective was already built with og-core's `sin_cos`, not glam's
  host-libm `perspective`; a test now pins their bits, and glam's
  perspective constructors (`glam::camera::{rh,lh}::proj::*::perspective*`)
  are on the disallowed list. The engine's one other use, the debug labels'
  projection, is render-only and takes a scoped `#[allow]`.
- engine (Rust): `og_core::math::quat_slerp` and `acos`, and a clippy
  `disallowed-methods` list (the root `clippy.toml`) that refuses the host's
  libm transcendentals and glam's libm-backed rotations in the simulation
  crates. Crates that only render, cook or run tools opt out in their own
  `clippy.toml`; render-only code in a simulation crate takes a scoped
  `#[allow]`.

## 0.6.2 — 2026-10-27

No IDL member changed; the simulation is unchanged (the golden replays).

- package: `changelog.json`, this changelog for machines (schema 1): each
  version's `kind` (patch or minor), whether the `simulation` changed, its
  breaking entries with their fix (a `codemod` or the identifiers to
  `find`), additions and fixes. `release.sh bump` moves its `unreleased`
  entry into the version and refuses a patch with a break.
- og: `og upgrade [project] [--to <v>] [--dry-run] [--json]` moves the pin
  in project.ogt and package.json's dependency, applies the codemods of
  the versions crossed and lists each line to edit by hand; exit 0 done,
  3 hand edits listed, 1 failed.
- og: `og doctor --json`, the checks as one object with `pin: {pinned,
  installed, compatible}`; a pin of the installed minor that is not newer
  is compatible and no longer reported as a difference.
- og: `og replay record` and `og replay compare`: a scripted headless
  run's portable world hash per tick and a save every K ticks, replayed
  with another version to the first divergent tick and the keyframe's
  entity, component and field diff (docs/release.md §3).
- release: the patch gate, `scripts/replay-gate.ts` over
  `tests/replay/golden/` (the 3D and 2D starters), run by `release.sh
  bump`: a divergence fails unless `unreleased.simulation` is `changed`.

## 0.6.1 — 2026-10-27

- render, `PostEffects`: `screenBlur` (`none`, `zoom`, `spin`,
  `directional`; RedGPU's zoom, radial and directional blurs, the spin
  and the directional blur measured in pixels on any aspect) with
  `screenBlurStrength` (off at 0), `screenBlurCenter`, `screenBlurAngle`
  and `screenBlurSamples`. One HDR pass after motion blur, before bloom
  and the tone curve; the UI is not blurred (og-render `ScreenBlur`).
- atmosphere: resource `og.Wind` (`strength`, `direction`, `speed`,
  `gust`; off at strength 0) sways every `og.TerrainScatter`'s copies:
  a lean downwind that grows with the vertex's height, varied by gusts
  that sweep across the field (RedGPU's wind sway, with the wave
  travelling downwind). Drawn from the tick in the lit, depth and shadow
  passes alike, its motion vectors included; nothing in the simulation
  reads it. terrain, `TerrainScatter`: `sway` (1 as the wind blows, 0
  still, for stones). og-render `Wind`, `RenderScene::wind`.
- render, `PostEffects`: `lensDistortion` (barrel above 0, pincushion
  below; RedGPU's lensDistortion with the radius in pixels whatever the
  aspect and a barrel fitted to its farthest corner, so no black edge)
  and `lensDistortionCenter`. One pass while on, after the Stylise passes
  that read the depth and before the palette quantisation; the UI is not
  bent (og-render `LensDistortion`).
- render, `RenderSettings.toneMapping`: `neutral`, Khronos PBR Neutral
  (three.js `NeutralToneMapping`, RedGPU's `khronosPbrNeutral`): colours
  up to 0.76 come out as authored less a dielectric's 0.04, only
  highlights roll off towards white (og-render `Tonemap::Neutral`). The
  enum's existing indices are unchanged.
- render, `PostEffects`: colour adjustments after the LUT (RedGPU's
  adjustments, ported): `temperature` and `tint` (white balance),
  `brightness`, `contrast`, `balanceShadows`, `balanceMidtones`,
  `balanceHighlights` with `preserveLuminosity` (colour balance), `hue`,
  `saturation`, `vibrance`, `grayscale`, `invert`, `threshold`. Every
  one is off at 0; they are baked with the LUT into one 32³ table on the
  CPU when one changes and add no pass (og-render `ColorAdjust`).
- release: the pack is a GitHub Actions matrix (`.github/workflows/release.yml`):
  the wasm, the linux-x64 addon, the engine and create-game packages on one
  Linux job, the darwin-arm64 and win32-x64 addons on their runners
  (`build.sh --native`, `pack.sh --native`), one manifest over the five
  tarballs; `scripts/release.sh fetch <run-id>` brings them to the machine
  that runs `verify` and `publish` (docs/release.md §3). No API change.
- Fixes: WebGL2 starts on Windows (Chrome and Edge, ANGLE on D3D11): the
  GL tier's light, LTC and probe tables no longer index a uniform vector
  by a variable, which FXC refused (X3003), and og-render's
  `tests/shaders.rs` holds every GL-tier module to that; lit shaders
  link there in 8–11 s on a cold start. The 2D publish build's refusal
  of a 3D world names its scenes with `/` on Windows. A debug
  `og-gfx` no longer overflows Windows' 1 MiB main-thread stack (the
  test scenes are built one closure each). The web test harness runs on
  Windows (the Chrome profile under the OS temp directory, Chrome closed
  over CDP). No API change.

## 0.6.0 — 2026-10-26

- three.js parity for ports, from the Grand View Garden engine fork
  (§1–§51 on 0.1.1, merged in PR #2). Every field is off at its default
  and draws every existing frame unchanged.
  - render, `Water`: `geometry` (water over any mesh instead of the
    grid), `f0` and `reflectionBias` (Water.js's Fresnel floor),
    `normalMap`, `normalScale` and `flow` (Water.js's drifting and
    Water2's flowing normal-map ripples), `glint` (Water.js's sun glint
    and shading).
  - render, `DirectionalLight`: `shadowBox` (one fixed shadow box
    instead of the cascades), `shadowFocus`, `shadowFocusDrop`,
    `shadowBias`, `shadowNormalBias` (three.js' bias and normalBias).
  - render, `Environment`: `hemiSky`, `hemiGround`, `hemiIntensity`
    (HemisphereLight).
  - render, `PostEffects`: `aoDirect` (AO darkens direct and emitted
    light too), `aoRadiusScale` (a radius that grows with distance),
    `sharpen` (CAS); `AoMethod.screen` (three.js-era depth-only screen
    AO).
  - render, `RenderSettings`: `bloomRadius` and `bloomContrast`
    (UnrealBloomPass).
  - render, materials: `alphaToCoverage` (with MSAA; scatter too).
  - Material shaders: `Hook.eye`, `Hook.front`, and the functions
    `planar_reflection(p, shift)` and `screen_ao(p)`.
  - terrain: `Terrain.visible` (false: ground for its scatter only),
    `TerrainScatter.castShadows`; scatter runs its material's hooks and
    takes the screen AO with `aoDirect` above 0.
  - Water and the screen AO: with `aoDirect` above 0 the AO sees the
    water's surface (as three.js' Water meshes wrote depth) and darkens
    its reflection and body; at the default 0 the water stays outside
    the AO, as before.
  - og-ui: a style's `ramp` line takes a fifth number.
- Debug views for the lit image: `DebugView` gains `cascades`, `ao`,
  `albedo`, `iblDiffuse`, `iblSpecular`, `direct`, `aoDepth` and
  `aoRaw` (RenderOptions.debugView, og.debug.setView, `debug:<view>`),
  drawn in place of the lit image; publish builds draw none. They
  replace the fork's `OG_DEBUG_VIEW` and `OG_AO_DEPTH` environment
  variables. The fork's `OG_RENDER_STATS` is gone: the probe's
  `timings` (passes, GPU passes, triangles) and `og run --trace` carry
  the frame.
- Colour textures as UASTC per project: project.ogt's `cook {
  color_textures uastc }` (etc1s by default), which `publish {
  color_textures etc1s }` overrides for the web export's cook (`og
  build`, `og run --device`). It replaces the fork's
  `OG_COOK_COLOR_UASTC=1`.
- render, `RenderOptions.planarReflections` (1 or 2, default 1): how
  many water surfaces mirror the scene in a pass of their own at once;
  with 2 a second surface in view mirrors too and material shaders can
  read the first's mirror (`planar_reflection`; the full tier only,
  WebGL2 reads 0). `RenderCapabilities.planarReflections` reports it.
  At 1 a second surface that asks reflects as if it did not, and the log
  says so once.
- Fixes: a hemisphere light keeps its own strength when the environment's
  intensity is 0; ambient occlusion with a 3D water surface no longer
  panics in the fx post bind group; og-ui's text measurement cache keys
  on a style's CJK tracking, so a style that changes only its CJK
  tracking measures again.
- Frames: 17 gfx scenes for the fork's fields (`crates/og-tests/src/fork_scenes.rs`,
  metal references), two of them with a WebGL2 difference on record
  (`hooks-planar` full tier only, `water-ripples` its own WebGL2
  reference; `docs/ci.md` §Browser reference tiers).

## 0.5.0 — 2026-10-25

- Audio decoding no longer runs on the engine thread: a sound entering
  the bank is indexed, not decoded, and its QOA frames are decoded beside
  the tick (natively on a thread of the bank, on the web by a decoder
  Worker the engine Worker feeds), ahead of the playheads as each tick
  plans it. A frame the mixer reaches first is decoded there, to the same
  samples, so every mix is bit for bit what it was and no world hash
  moves. The memory figure for audio counts a QOA sound's file and its
  decoded frames; the hosts' audio stats add `decodedAhead`,
  `decodeMisses` and `decodeQueued`.
- Cheaper ambient occlusion: `PostEffects.aoResolution` (`AoResolution`,
  `full` by default, today's frame) set to `half` searches and blurs a
  quarter of the pixels and brings them to full resolution with a
  depth-aware upsample (an `ao-upsample` pass). At `full` the AO blur
  reads its taps as gathers on WebGPU and skips the sky; every frame is
  unchanged. Worlds hash differently once (PostEffects has a new layout).
- The high graphics preset buys more than bigger shadow maps:
  `RenderSettings` gains `aoResolution` (`AoResolutionSetting`:
  `postEffects` by default, `full`, `half`; read before
  PostEffects.aoResolution), `ssrStepScale` and `volumetricStepScale`,
  and `og.render.applyQuality` writes high as full-resolution AO and
  twice the SSR steps and fog slices, low as half. Medium, the default,
  draws every frame as before. `RenderCapabilities.maxMsaaSamples` says
  the MSAA the device gives, the cap of `RenderSettings.msaaSamples`.
  Worlds hash differently once (RenderSettings has a new layout).
- Instance uploads send only the records a draw reads: a batch drawn from
  its culled copies (under 90 % in view, the CPU culling path, so every
  WebGL2 frame) no longer also sends its sorted records, and object
  motion's previous transforms are written only where they changed. A
  field of 20,000 moving spheres a fifth off screen sends 1.04 MB a frame
  where it sent 2.32 MB; frames are unchanged.
- Spring bone colliders and falloff: `og.SpringCollider` (a sphere or a
  capsule on a joint, three-vrm's VRMSpringBoneCollider) keeps the tails
  of that skeleton's `og.SpringBone`s out by its `radius` plus the
  spring's new `radius`, picked by `layers` against the spring's
  `colliders`; `og.SpringBone.tipFrequency` and `tipDamping` scale the
  last joint's frequency and damping, the joints between in proportion.
  At their defaults a chain swings bit for bit as before; worlds that
  install og-anim hash differently once (a new type, a new layout).
- Twist bones follow clips that are not retargeted: a clip authored for
  a rig that turns an arm or leg segment but leaves its twist bones
  (`…twist…`, Rigify's `DEF-….001`) still now gives them, per tick, the
  share of the segment's twist retargeting gives (after IK and look-at).
  Retargeted clips, rigs without twist bones and clips that key them play
  bit for bit as before.
- Unity's legacy thumb names are read from the metacarpal: on a side
  whose thumb has an `Intermediate` and no `Metacarpal` (`Left Thumb
  Proximal`, `Intermediate`, `Distal`; VRM 0.x's `leftThumbIntermediate`),
  Proximal is `leftThumbMetacarpal` and Intermediate `leftThumbProximal`,
  so those thumbs retarget one bone further in than they did. A `type rig`
  file naming them still wins. The cook's retarget stamp moves to
  `retarget-v3`: models with an `animation { library }` re-cook once.
- A model drawn without an `og.Skeleton` honours `KHR_node_visibility`:
  the model blob's node rows (`og.ModelNode` schema 2) carry each node's
  rest visibility and og-runtime's static draw skips a hidden node and
  its descendants, so a `.vox` without a skeleton draws frame 0 only
  (its other frames were zero-size draws). Every model, proxy and `#anim`
  re-cooks once (the type hash and the `nodes-v2` stamp move).
- glTF files that animate visibility through `KHR_animation_pointer`
  (onto `KHR_node_visibility/visible`, as Blender and three.js export)
  now cook: the loader rewrites such channels as the engine's
  `OG_animation_visibility` before the gltf crate validates the file. A
  pointer onto another property is dropped with a warning, or refused
  when the file requires the extension. Files without one load byte for
  byte as before.
- Faster capture PNGs: the probe's `capture`, `sweep` and `screenshot`
  encode with fdeflate (rows filtered `Up`, deflated as they are
  filtered) instead of zlib level 6, 10 to 35 times faster on a
  1280×720 frame, natively and in the browser. The pixels decode exactly
  as before; the files are larger (about 1.5 times on a detailed frame, up
  to 8 times on a flat one), and a renderer reuses the readback buffer
  of the last capture of the same size.
- The WebGL2 fallback draws through a state filter in the engine Worker
  (`src/web/gl-state.ts`, docs/web-export.md): a WebGL call that sets
  what the context already holds is dropped, and the enables and
  bindings are made when a call first reads them. Frames are the same;
  the reference scene's 3,797 WebGL calls a frame become 761 for the same
  41 draws.
- Mirroring without `og.anim.mirror`: `mirror true` on an `animgraph`
  clip plays it on the other side of the skeleton (made once per
  skeleton and clip and kept, the same clip `og.anim.mirror` makes), and
  the cook's `animation { mirror "<clip>" ["<name>"] }` writes a mirrored
  copy into the model's `#anim`.
- Retargeting of hands and twist bones, and per-character foot heights:
  the humanoid map gains VRM 1.0's 30 finger bones (read from Mixamo,
  VRoid, Rigify, Unreal and plain names; `og.anim.humanoid` lists them),
  fingers turn from their hand; the target's twist bones (`…twist…`,
  Rigify's `DEF-….001`) take a share of their segment's twist; a `type
  rig` file's `footHeight` (one, or left and right) sets where a
  character's feet stand above the ground, so a rig whose rest pose does
  not stand on it plants its feet there. Rigs without fingers, twist
  bones or `footHeight` retarget bit for bit as before (the cook's
  retarget stamp moves to `retarget-v2`).
- `og.SpringBone` (spring/jiggle bones, three-vrm's VRMSpringBone: a
  joint chain whose tails follow their animated places by a damped spring
  under gravity, in world or model space, held at bone length; `lag`
  written). The tails are saved in the `og.anim` state block (a tagged
  section written only when a spring exists).
- Visibility tracks in og-anim: a clip's `ChannelPath::Visibility`
  channel (STEP, 0 or 1) hides a node and what is under it, and og-runtime
  draws no primitive of a hidden node on either host. Blends take the
  heavier pose's flags (a cross-fade swaps at its middle); mirroring and
  retargeting carry the channel; the flags are saved in the `og.anim`
  state block (a tagged section written only when a node is hidden). The
  cook reads `KHR_node_visibility` (rest) and the engine's
  `OG_animation_visibility` animation extension; the `#anim` blob's
  schema is 2 (a `hidden` table), so every `#anim` re-cooks. An animated
  `.vox` folds its frames with visibility instead of scaling joints to
  0: the same frame shows on every tick, one draw instead of one per
  frame model.
- Billboards: `og.Sprite.facing` and `SpriteDesc.facing` (enum
  `SpriteFacing`: `none`, the default and the old xy plane; `camera`, the
  view plane; `cameraY`, upright and turned about y), with the
  `SpriteFacing` column factory. A faced world sprite turns to every camera
  that draws it (the main one, og.CameraView, og.Mirror, reflection probe
  faces) in the sprite shader on both tiers, sorts back to front within
  its layer, is culled by its bounds, and casts its drop shadow from a
  sun-facing card. Render-only: no world state changes. og-render:
  `RenderScene::sprite_facing`, `SpriteFacing`, `Sprite::anchored_at`,
  `ShadowGround::fit_facing`, `sun_card_right`,
  `FrameStats::sprites_culled`; the sprite instance grows 4 bytes (104).
  New og-tests scene `sprites-billboard` (`vulkan-cpu` reference; `metal`
  owed).
- Transition interruption rules in the `animgraph` grammar: `interrupt
  any | none | source | next` on a transition says which transitions may
  cut its cross-fade (default `any`, the old behaviour); the guard of a
  fade in progress is saved in the `og.anim` state block.
- Layer masks and sync groups in the `animgraph` grammar: `mask "<node>"
  [weight]` lines give a layer per-joint weights (a node's nearest named
  ancestor, 0 elsewhere); `sync <group>` in a state's block plays every
  later layer's state of the group at the first layer's phase, and a
  transition inside a group keeps the phase. `og.anim.mirror` (and
  `og_anim::mirror`): a clip played on the other side, left and right
  paired by the naming schemes' side tokens, reflected in model space.
- `og.FootLock` (a planted foot held where it touched the ground across
  clip blends and root motion until the clips lift it, then released over
  `release`; `planted` and `locked` written). "Planted" is retargeting's
  test, now `og_anim::FootPlants`. The locks are saved in the `og.anim`
  state block (a tagged section written only when a lock is held).
- `og.LookAt` (an aim constraint after the clips and IK: a joint's `axis`
  turned onto `target`, its `upAxis` towards the `up` hint, held within
  `maxYaw`/`maxPitch`, shared over `bones` joints, `clamped` written) and
  `og.JointLimit` (a joint's swing cone and twist range from rest, which
  og.IkChain's two-bone and FABRIK solvers and og.LookAt clamp to; a hinge
  is `swing` 0 with the bend as the twist range). New records: every
  world's registry, and so its `world_hash`, gains them.
- Doppler on spatial audio: `og.AudioSource.dopplerFactor` (1; 0 off),
  `og.AudioListener.dopplerFactor` (1) and `og.Audio.speedOfSound`
  (343.3 m/s). The pitch follows OpenAL's model from the source's and the
  listener's tick velocities, composed down `og.Parent`; a jump faster
  than sound in one tick is a teleport and shifts nothing. Moving spatial
  sources and listeners now sound shifted: set `dopplerFactor` 0 to keep
  the old pitch. The records' layouts change (saves restore with
  `migrate`).
- Cone directivity: `og.AudioSource.coneInnerAngle`, `coneOuterAngle`
  (360°) and `coneOuterGain` (0), Web Audio's PannerNode cone about the
  source's +Z as three.js's `setDirectionalCone`.
- HRTF for headphones: `og.AudioListener.panningModel` (`PanningModel`:
  `equalpower` default, `hrtf`), a spherical-head model (Brown and Duda:
  interaural delay and head shadow) in the mixer, steered by a new `hrtf`
  command (op 8) in the stream; with `equalpower` the mix is unchanged.
- Windowed-sinc resampling: `og.Audio.resampler` (`AudioResampler`:
  `sinc` default, `linear`), a Kaiser-windowed sinc (β 7, 16 taps, 2048
  phases, widened to 64 taps pitched up to 4×). Pitched voices and sounds
  not at the output's rate now render other samples: set `linear` to keep
  the mix as it was; unpitched sounds at the output's rate are unchanged.
- Virtual voices: `og.AudioSource.virtualize` keeps a source past
  `og.Audio.maxVoices` silent with its playhead running and heard again
  from where it would be (`og.AudioPlayback.virtual`), instead of culled.
- Retrigger: `og.AudioSource.retrigger` restarts a sound from the top in
  one tick, playing or not; the engine clears it. `SoundEnded.reason`
  `SOUND_RETRIGGERED` (5).
- `og.CollisionStarted` gains `impulse` (N·s, the normal impulse the
  solver gave the pair in the step that made the contact, summed over
  substeps and contact points: `b` takes it along `normal`, `a` against
  it) and `relativeVelocity` (m/s, `a`'s velocity relative to `b` at
  `point` as that step began, each body's angular point velocity
  included; its part along `normal` is the closing speed), in both
  profiles (Rumpus Isle FINDINGS #9). `point` is declared in `m`. The
  unit `newtonSecond` is new. The event's layout hash changes: a save
  taken before this restores with `migrate`.
- Field factories for the handle types that had none: `Font()`, `Style()`,
  `Look()`, `Shader()` and `Rig()` (columns in `crates/og-api/api.ogt`,
  64-bit like every handle), so a font is kept in a `Font()` field, never
  an `f64` or an `Asset()` cast (rumpus-isle #2). Every handle type of the
  API now has its factory; `packages/engine/tests/api-surface.test.ts`
  holds it.
- A framed play page joins the Player's room again: the Player's hello
  usually arrives while the engine loads, before the shell's hello handler
  that reads it (`shell/page.ts`), so a Play link's room was never joined
  under a real Player. The shell now replays that hello once the handler
  is installed, as the top-level page already did (the platform's
  rooms/player-protocol-2 acceptance found it: two engines on one `?room=`
  link join as peers 0 and 1 with the fix, neither without).

## 0.4.0 — 2026-10-24

Player hello time now reaches `og.platform` on attachment through both
web transports. Older Players without a stamp retain the host clock.

Round 3, procedural modelling (`docs/phase-1.md`, "Plan 2026-10-24,
round 3").

- `geom-1`: the crate `og-geom` (three.js's generators with three.js's
  names, defaults, vertex orders and counts; a sweep along a path with
  scale and twist; merges with a mirror; CSG with exact predicates,
  deterministic on every host) reached from module render beside
  `tubeGeometry`: `shapeGeometry`, `extrudeGeometry`, `latheGeometry`,
  `polyhedronGeometry`, `icosahedronGeometry`, `dodecahedronGeometry`,
  `octahedronGeometry`, `tetrahedronGeometry`, `sweepGeometry`,
  `csgGeometry` and `mergeGeometries`, each a Geometry as
  `bufferGeometry` makes one (BVH, `physics`, `dynamic`). A new root
  module `shapes`: `og.Shape` and `og.Path2` with three.js's methods,
  flattened to points on the wire. The IDL gains `This`, `prop { host
  false }` and class values. The og-game skill §18 and
  `reference/shapes.md`; the og-tests gfx scene `geometry-shapes`.
- `profile-quality`: `og build --publish` and `og run --device` refuse
  a `profile "2d"` project whose scenes hold Terrain, TerrainScatter,
  Water, WaterSurface or Fluid3D, one `<scene>:<line>: <Component>` per
  hit; under `profile2d` og-runtime's terrain is absent (`og.terrain.*`
  and heightfield colliders refuse, naming `RenderCapabilities.world3d`).
  `GraphicsQuality` moved from module ui to render and is read by the
  renderer: `RenderSettings` gains `rectShadows`, `shadowedLights`,
  `shadowMapScale`, `volumetricFog`, `softParticles`, `ssr`, `ao`,
  `msaaSamples` and `quality: QualityPreset`;
  `og.render.applyQuality(world, preset)` writes a preset's row (the
  table is in its doc); `SettingsDesc.world` makes the UI kit's settings
  screen apply a picked preset.
- `terrain-generate`: `type terrain` makes its field with a `generate`
  block instead of reading a heightmap (refused beside one): `seed`,
  `samples`, fractal noise with a domain warp, `mask island|ridge`,
  hydraulic and thermal erosion, `terraces`, `sea` with a `shore` band,
  carved `rivers`, `moisture` and `biome` rules writing the control map;
  the same tile pyramid comes out, deterministic across hosts. Every
  cooked terrain recooks once (TerrainMeta v4). `TerrainInfo.seaLevel`
  (none without `sea`); `og cook --report` reports a generated terrain's
  histogram, land fraction and river length and writes its height and
  control maps under `og-out/terrain-maps/`. The og-game skill §11; the
  og-tests gfx scene `terrain-generated`; fx-showcase's `islands` station.
- `model-review`: `og run --commands` review cameras `view:<entity>:front|
  back|side|top|orbit[:deg]`, `view:off` and `turntable:<entity>:<n>[:prefix]`
  (held in render state, never written to og.Camera; not in publish);
  `og compare` prints silhouette IoU and ΔE00 (mean and p95) per mask
  region and writes one sheet (reference | render | silhouettes), with
  `--align` and `--json`; the `og-model` skill (intake, eight passes with
  gates, generator recipes, budgets) and design §2.11.
- `probe-faces`: a probe capture's faces draw 3D fluids, and each face
  gets its own depth prepass (the views' prepass, shared), so water in a
  face refracts what lies below it and soft particles fade against the
  face's depth; a Depth32Float attachment cannot be loaded on GLSL ES, so
  a face's depth is a linear-depth colour target like a view's. Nothing
  runs without a due capture; per face, water adds a prepass and a copy,
  fluids six passes.
- `geom-2`: `type model`, a model written as `.ogt` text of og-geom parts
  (the render descs by name: og-render's primitives, shape, extrude with
  `extrudePath`, lathe, polyhedron, the four solids, sweep, csg, merge),
  cooked through a generated glb into the same model blob a glTF gives
  (meshopt, LODs with measured errors, BVH, nodes; repeated parts one
  instanced mesh); refused past project.ogt's triangle and vertex
  budgets; it hot-reloads under its handle. `og cook --report` gains
  `parts` and `draws` per model. `uvRepeat` on the eight og-geom descs;
  `bevelSize` is `Option.f32`, bevelThickness − 0.1 when unset. og-geom
  gains a Rust `Path2`/`Shape` flattening to the TypeScript runtime's
  points bit for bit. fx-showcase's `models` station; og-game skill §18.
- `threejs-migrate`: the knowledge guide `three-to-og` (eleven sections,
  three.js's classes and idioms to og's, the gaps named) mounted in
  og-game's pack and held to the skills' rules by the validator; `og
  migrate three <file.ts> [--out|--write] [--json]` rewrites what is
  mechanical (imports, the geometry constructors with named arguments,
  materials, colours, vectors, meshes, groups and lights to spawns, the
  transforms, the camera, texture loads) and leaves the rest under an
  `// og migrate: … (three-to-og §N)` line, listed with its line. og-build
  §4 points to `og compare` and the review views; og-model names the
  `type model` route.
- release F2: og-cook's `.vox` reads a node's `_loop` and `META`'s
  `_anim_range` (the frames walked, the default clip's `frames`); og-ui
  under mac keys gains the emacs kill buffer (Ctrl+K/U/W kill into it,
  kills in a row add up, Ctrl+Y yanks, Ctrl+T transposes) and ← and →
  move visually across a bidi edge, keeping the caret's place at the
  edge (`UiKey::KeyT`, save state version 9); og-ui's cooked-font text
  frames (`text-bidi`, `text-kerning`, `text-edit`, `text-mixed-runs`)
  run in the browser tiers, `reference-gfx.ts` cooking their fonts with
  `og cook`.
- release F4: `web-reference` runs Brasswick's nine and Meridian Loop's
  ten frames on WebGPU and WebGL2 from each game's frames playtest
  (`reference.ts --playtest`), making the games' generated assets first
  on a checkout without them; `published.ts` has plans for both games;
  `og run --device` runs on another project (`device.ts --project`).
  Brasswick's Metal `pause` and `settings-es` frames re-recorded (the
  44 px HUD pause button drew as a circle in the frames recorded before
  ui-prose's nine-slice fix, a rounded square since), with `title`,
  `levels` and `results` moved slightly by the games' regenerated
  `style.ogt` (the fantasy and neon packs' `gold` had changed under the
  committed copies).
- Fixes at the round's first wave-end: the 2D web modules' world carries
  the `og.terrain` state block (no edits) the full build writes, so a 2D
  save and world hash are the same bytes on every host, and a save with
  terrain edits is refused there naming the profile; the third-person
  `beacon`, `title` and `dawn` frames re-recorded on Metal and the
  beacon on WebGL2 after the sprite sampler's anisotropy (ui-aniso).
- Fixes at the round's third batch: `engine.command` (and so `og run
  --setup`) reaches the actions the engine's modules register with the
  probe (`locale`, `view`, `cover`), not only the game's; the command
  `cover:<x>,<y>,<width>,<height>|none` sets `Viewport.covered` on a
  native run as a host's own control would (the published page's Back);
  `PlaytestDesc.setup` runs such steps once after `start` in every process
  of a playtest. Meridian Loop's published playtest covers the page's
  Back rectangle and drives the straight without the steering tap that
  put the car into the wall; its four Metal `pub-*` frames re-recorded.
  `device.ts` finds the engine module the page loaded (`-lanes` where it
  is cross-origin isolated) for its COOP/COEP check.
- Browser acceptance: published input now enters at the same tick boundary
  as the native command channel, rather than one tick early. Meridian
  Loop's WebGPU published frames pass against Metal unchanged. Its WebGL2
  references retain the tier's MSAA/bloom appearance (ten regular frames
  and three published frames); the published pause still compares to Metal.
- Unknown `engine.command` actions return their refusal and the available
  game and module actions without a conflicting probe borrow panicking
  in the native addon.

Debt round 2 (`docs/phase-1.md`, "Plan 2026-10-23, debt round 2").

- `records-list`: records take a fixed-capacity list column of one number
  type (`List.X.N` in the IDL, `S[N]` in the schema, the `list` factory in
  TypeScript; through og-core, og-ecs snapshots, og-net, og-probe, og-cook
  scene text, the codec and both runtimes). `PostEffects.custom` is one
  `List.Post.8`; `custom1..custom8` are gone and their callers migrated
  (fx-showcase's shaders station, the og-game skill, design §2.6).
  `maxCustomPosts` stays the bound.
- `ui-prose`: og-ui's nine-slice no longer squeezes a middle column or row
  past twice its art (it shows the centre of the art instead), which
  removes the light band through short buttons such as Rumpus Isle's
  Pause: the band came from the mipmapped atlas sampling a coarse level
  for a 6× squeeze, not from the corners. The games, archetypes and
  knowledge skills call a UI style a style (the old `skins/<name>` paths
  are `styles/<name>`), and knowledge `validate` refuses a skill or wiki
  page that calls one a skin.
- `physics-push`: the raycast record's hierarchy refresh keeps the world
  matrices that cannot have moved (a still tick of 20k meshes 14.2 → 4.0 ms,
  dev profile); `ext::push` splits a tick's push budget across every body
  hit in the same fraction instead of giving it to the first; Lantern
  Vale's rock pile is convex hulls that stand still without the hero.
  brasswick keeps its `pushSpeed` slowdown as the game's push speed: the
  force-capped push alone runs the 2 kg crates past their plates.
- `anim-2d-2`: `og.Skeleton2D.skin` and `Rig2DInfo.skins` (a rig2d `skin`
  block swaps the default skin's attachments; `default` first), and
  `og.Skeleton2DEvent.armature` names the child armature an event came
  from. The DragonBones import brings over `clockwise` multi-turn
  rotations (a `rotate` key's `clockwise <n>`), sine `tweenEasing` as
  Hermite curve keys (worst error 1.3e-5°), path displays and path
  constraints (rig2d `path`, `along`), Spine's `transform` constraint, and
  every skin; the lamp keeper turns over 12 ticks and its feet settle on
  arrival instead of snapping.
- Mesh levels of detail are picked by measured error, not by triangle
  fraction. The cook measures each level's distance from the full mesh
  (two-way nearest surface through the BVHs; `LodRow` version 2,
  `lodErrors` in `cook --report`) and the renderer draws the coarsest
  level whose error projects under a pixel at the model's near side, with
  hysteresis; `RenderSettings.lodBias` and `Model.lodBias` multiply that
  error (0 still draws the full mesh). The old rule decimated small props
  while they filled a few hundred pixels, which is where the reference
  drift of lantern-vale and meridian-loop came from.
- `ui-aniso`: sprites and the UI's images sample anisotropically
  (`anisotropy_clamp` 16) where the device filters so (Metal, Vulkan,
  WebGPU, WebGL2 with `EXT_texture_filter_anisotropic`), reported as
  `RenderCapabilities.spriteAnisotropy`. A nine-slice middle squeezed
  along one axis now reads its mip level by the other axis, so the
  outline through it is as crisp as the corners: the softness ui-prose's
  2× bound left on short buttons such as Rumpus Isle's Pause is gone.
  Text (MSDF), the decal atlas and the 2D light map keep a trilinear
  sampler; a software rasterizer (lavapipe) stays at 1, where its
  anisotropic filter softens even a 1:1 draw. The last eight "skin" lines
  for a UI style (the tower-defense archetype, packages/engine's
  `look.test.ts` and `base.ogt`) say style, and `create`'s test refuses
  one in a shipped template.
- `render-views-2`: views (`og.View`) draw everything the main pass
  draws: trails, ribbons and beams facing the view's eye, 2D-lit sprite
  layers from a light map per view, 2D fluids from metaball fields per
  view, soft particles against the view's own depth (they vanished from
  views before: their depth load went out of bounds) and sprite glow
  through a per-view glow chain. A full `custom_lit.wgsl` lights rect
  lights through `surface_local_light` with their shadow and cookies
  (it lit them as a point before). Skinned instances pick their
  reflection probe from their posed bounds. Probe captures draw
  particles, lines, points, world sprites and water (as its deep colour:
  a face has no depth). Left: 3D fluids and depth in probe faces.
- `size`: the publish wasm leaves out GPU timestamp queries and the
  probe's tool members (describe and the views' id pass, `save_scene`,
  `reload`, `get`, the dumps, the command channel's tool actions: a
  timeline event or `og.ai` command naming one logs a refusal there);
  the 2D profile's web modules leave out terrain, planar water, the 3D
  fluids, scattered copies and volumetric fog and their WGSL
  (`RenderCapabilities.world3d` says so; a 2D scene holding them draws
  them in `og dev` and natively, not in its 2D publish build). Shipping
  `wasm` profile, brotli: 3D publish 3,532,992 → 3,456,521 B (82.4 %), 2D
  publish 3,357,559 → 3,241,126 B (77.3 %). `og cook --report` and `og
  doctor --project` warn when a model's raycast BVH is past
  `budgets.bvhShare` (30 %, at least 16 KiB); the BVH stays cooked by
  default (dropship 38.6 %, fx-showcase's tree and trickster,
  meridian-loop's track warn and keep it).

## 0.3.0 (2026-10-23)

The depth wave (`docs/depth.md`, nine streams), its debt round and the
UI styles branch; a minor bump for the API changes below, breaking before
1.0.0 where a member was renamed (`og.load.skin` is `og.load.style`).

- Windows x64 (F7, merged after 0.2.0 was published, so the package first
  ships with the next version): build and pack the MSVC `og.exe` and napi
  addon as `@origingame/engine-win32-x64`, with DX12 rendering, embedded
  DXC and the same TypeScript API. `og check` deliberately uses the pinned
  tsgo/Bun fallback, reported by doctor. Windows paths, source line
  endings, HDR fixture checkout bytes and knowledge mount permissions are
  portable. The tarball install test passes on Windows 11; the five gfx
  tests that compare DX12 frames against the Metal references differ only
  by edge and noise dither (reviewed by the coordinator, nothing moved; no
  DX12 reference tier). Evidence: `tests/evidence/release/windows-x64.md`.
  `@origingame/engine@0.2.0` on the registry does not reference the
  win32-x64 package; the engine's `optionalDependencies` pin it from this
  commit on.

UI styles (branch `ui/style`): a game's UI is the game's own, never the
platform's. API changes (a minor bump; breaking before 1.0.0):

- og-ui owns its values (`crates/og-ui/src/theme.rs`); the vendored
  platform `tokens.json` and its generator are gone. Without a style the UI
  draws in `prototype`, a placeholder (greys, radii 1/2/1/3, a cyan focus).
- Palette roles renamed to the engine's 33 (`ThemeColor`: `backdrop`,
  `panel`, `ink`, `inkMuted`, `inkFaint`, `primary`, `onPrimary`,
  `primarySoft`, `control`, `line`, `hud`, ...); `accent` is `primary`
  (`Tone`, `MeterKind`, `Ink`); `UiDesc.primaryColor` replaces the accent
  and hue (`HueName` removed); `DensityMode` is `auto`, `mouse`, `pad`,
  `touch`; radii are `small`, `control`, `bar`, `panel`.
- The skin is a **style**: `og.load.style` (was `og.load.skin`),
  `UiDesc.style`, `UiTheme.style`, `LookInfo.style`, the `Style` handle,
  `StylePart` with `hud` (was `capsule`), blob `og.UiStyle`, `type style`
  files that name every palette role and radius (`type skin` and `base`
  are refused with the fix), `og style pack` (was `og skin pack`).
- A look's `ui` line names the style: a built-in by slug
  (`templates/styles/{pixel,cartoon,fantasy,scifi,paper,neon}`, cooked
  under `@styles/`) or the game's own; every catalogue preset names one
  (`looks/catalogue.json`'s `ui`), so a game extending a preset is dressed.
- `og.ui.actionButton` is square, cornered by `radius.control` (was a
  circle); the cartoon style's corners drop from 10/18 to 6/10 and its
  primary deepens to `#8c360f` (7:1 as drawn).
- `dump_ui` carries `style` ("" for prototype); `og polish` lists a UI
  style finding for a game left in prototype.
- The kit's cutscene draws its letterbox bars black in every style (at
  the merge they took the `backdrop` role, which a paper style turned
  into cream panels over Kestrel Stair's intro).

Depth wave 1 (`docs/depth.md` §1–§3, merged 2026-10-23), API additions (a
minor bump, §3), by stream:

- `geometry` (S1), module render: `og.render.bufferGeometry`,
  `updateGeometry` (dynamic geometries only) and `geometryInfo`, with
  `BufferGeometryDesc` and `GeometryInfo`; a made geometry is also a
  `Collider.mesh`. The three.js primitives `cylinderGeometry`,
  `coneGeometry`, `capsuleGeometry`, `torusGeometry`, `ringGeometry`,
  `circleGeometry` and `tubeGeometry` (centripetal Catmull-Rom) with their
  desc structs. Immediate `og.render.lines` (`LinesDesc`, `LineMode`) and
  `og.render.points` (`PointsDesc`); the retained components `og.Lines`
  and `og.Points`. `og.render.raycast` (`RaycastGeometryDesc`,
  `GeometryHit`) through a BVH per geometry, over `og.Mesh` and `og.Model`.
  `StandardMaterialDesc.wireframe`. `RenderOptions.debugView` is the new
  `DebugView` enum (`none`, `physics`, `nav`, `bounds`), also the command
  channel's `debug:<view>`. New module `debug` (`publish false`):
  `og.debug.line`, `box`, `sphere`, `arrow`, `axes`, `text`, `clear`,
  `setView`/`view`, `setEnabled`/`enabled`, with `DebugDrawDesc`.
- `timeline` (S2), new modules `path` and `timeline`: `og.path.create`
  (`CurveDesc`: catmullRom, bezier or linear, closed, tension), `find` (a
  scene's `path "<name>" { ... }`), `sample` (`PathSample`), `length` and
  `nearest` (`PathNearest`), the `Path` handle, `PathKind` and the
  `og.PathFollower` component; `og.CameraRail.path` follows a path instead
  of its points. `og.load.timeline` loads an `.ogt` of `type timeline`
  (the `TimelineAsset` handle); `og.Timeline` plays it with
  `og.timeline.play`, `pause`, `seek`, `stop` and `info` (`TimelineInfo`,
  `TimelineTrackInfo`), raising `TimelineStarted`, `TimelineMarker` and
  `TimelineEnded`; the command channel's `timeline:play:<name>`,
  `timeline:seek:<s>[:<name>]`, `timeline:pause`, `resume`, `stop`. A
  timeline's time is a whole count of thousandths of a tick, so a seek
  hashes as a play. The desc is `CurveDesc`, not `PathDesc`, which og.nav
  already declares.
- `video` (S3), new module `video`: `og.load.video` and `videoInfo` (the
  `Video` handle, `VideoInfo`) for an `.ogt` of `type video`, cooked by
  ffmpeg into a KTX2 texture array and a QOA sound; the `og.VideoPlayer`
  component whose frame is a Texture and whose tick keeps the sound in
  step. `og render` (native only): `--commands`/`--timeline`, `--fps`
  (must divide the tick rate), `--from/--to`, `--size`, `--out
  dir|.mp4|.webm`, `--frames png|jpg`, `--audio`, `--contact-sheet`,
  `--jobs`, `--json`, with a `frames.json` of sha256 per frame; two runs
  are byte-identical and `--jobs` matches one job. `og doctor` reports
  ffmpeg and ffprobe.

Depth wave 2 (`docs/depth.md` §4, §5 and §9, merged 2026-10-23), API
additions (a minor bump, §3), by stream:

- `probe-depth` (S5), module probe: `describe` (the frame as data, a
  `Description` of `DescribedEntity` boxes with depth and occlusion from a
  CPU entity-id pass at a quarter of the pixels, `DescribedUi` and
  `DescribedCamera`); `expect` with six operations (`ExpectOperation`),
  as `expect:` script steps, a program op or members; `expectations`
  (`ExpectReport`, `ExpectResult`, `ExpectStatus`); `saveScene`
  (`SavedScene`: the boot scene's `.ogt` written back line by line from
  the world, comments, order and spelling kept); `state().timings.systems`
  and `.passes` (`ProbeTiming`). `og run --json` carries the expectations
  and exits 1 on a failure, and a playtest may be `expect` steps only.
  `og run --trace out.json` writes the run in the Chrome trace format;
  `og cook --report` writes `og-out/cook-report.json` and `og doctor
  --project <dir> [--json]` checks it against project.ogt's `budgets { }`.
- `two-d-math` (S9), new module `math` (`host false`): three.js's methods
  as pure functions over the plain structs, with `Box3`, `Sphere`, `Plane`
  and `Frustum`. Module anim: `og.load.rig2d` (the `Rig2D` column, an
  `.ogt` of `type rig2d` or a DragonBones 5.5 export), the `og.Skeleton2D`
  and `og.IkTarget2D` components, the `og.Skeleton2DEvent` event,
  `og.anim.rig2dInfo` (`Rig2DInfo`, `Rig2DClipInfo`) and `clip`; a
  skeleton draws as sprite meshes in the sprite order. New template
  `templates/game-2d` (the lamp keeper).
- `rooms` (S4), module platform: `roomCreate`, `roomJoin`, `roomLeave`,
  `roomSend`, `roomPeers`, `roomInfo` and `roomMessage` (`RoomDesc`,
  `RoomPeer`, `RoomInfo`, `RoomMessageData`; the events `RoomJoined`,
  `RoomPeerJoined`, `RoomPeerLeft`, `RoomMessage`). New module `net`:
  `og.Lockstep`, the `og.PlayerInput` component, the `og.Desync` event,
  `submit`, `status` and `hash` (`InputDesc`, `LockstepSeat`,
  `LockstepStatus`); the probe's `dump_desync` (`DesyncDump`) names the
  first divergent tick, entity, component and field with both values.
  `og relay [--port] [--host]` and `og run --room
  ws://127.0.0.1:P/<room>`; `PlaytestDesc.room` runs a playtest as several
  processes through a relay it starts. New archetype
  `templates/archetypes/arena` (Coin Duel). On the web, rooms go through
  the Player's `room.*` (platform-asks #16): until the platform lands it,
  `roomInfo` answers `host: none`.

Depth wave 3 (`docs/depth.md` §6–§8, merged 2026-10-23), API additions (a
minor bump, §3), by stream:

- `render-lod` (S6), module render: the cook makes LOD levels of every
  model (`lods 1 0.5 0.25` by default, `lodError`; glTF `MSFT_lod` levels
  kept; `cook --report` lists them) and the renderer picks one by projected
  size with hysteresis. `RenderSettings.lodBias` and `og.Model.lodBias`
  (0 draws the full mesh), `DebugView.lod` (the `debug:lod` command tints
  by level), `dump_entity`'s `render.lod`, `ProbeTimings.triangles`.
  `og.ReflectionProbe` (`size`, `resolution`, `mode` `ReflectionProbeMode`
  baked or realtime, `interval`, `intensity`, `blendDistance`, `visible`;
  `MAX_REFLECTION_PROBES` 8): six faces captured in a `reflection-probes`
  pass, GGX-prefiltered, box-projected and blended into the lit shaders.
  `og bake probes [project] [--commands] [--at] [--size]` writes
  `cooked/probes/<key>.ogb`, which the runtime loads instead of capturing.
  Screen-space reflections on the full tier: `PostEffects.ssr`,
  `ssrMaxDistance`, `ssrThickness`, `ssrSteps`, `ssrFade`, `ssrIntensity`;
  `RenderCapabilities.ssr` and `reflectionProbes` (`ssr` is false on
  WebGL2). The prepass's linear depth is `Rg32Float` now, its second
  channel the reflectivity. fx-showcase stations `lod` (2,000 trees:
  1,300,812 triangles with LOD against 5,150,788 at `lodBias` 0) and
  `reflections`.
- `render-targets` (S8), module render: `og.render.renderTarget(world,
  { width, height, format, depth })` makes an `ldr` or `hdr` target whose
  colour is an ordinary `Texture` (a material map, a UI image, a mirror);
  depth is the full tier's (`RenderCapabilities.renderTargetDepth`,
  `maxRenderTargetSize`, `maxCustomPosts`). `og.CameraView` renders a
  second camera into a target or a viewport rect (a monitor, a split
  screen); `og.Mirror` fits an off-axis frustum to its glass. `layers u32`
  (default 1) on `og.Mesh`, `og.Model`, `og.Sprite`, `og.Lines`,
  `og.Points`, `og.Camera`, `og.CameraView` and `og.Mirror`, tested with
  three.js's `&`; `og.render.raycast` reads each mesh's mask; shadows
  take every caster. `type shader` gains `mode full lit|unlit`: vertex
  and fragment entries that the prepass and shadow passes run too. `type
  post` with `og.load.post`, `PostEffects.custom1..4` (four fixed slots),
  `og.render.setPostParam` and `og.render.postErrors` (a pass a device
  fails to compile is skipped and listed). A view draws opaque meshes,
  terrain and sky only, and a frame with views culls on the CPU.
  fx-showcase station `views` (new) and `shaders` (a full-shader dissolve
  and a CRT post pass).
- `render-light` (S7), module render: the og-bake crate (xatlas UVs, a
  BVH, a progressive CPU path tracer, L1 SH light volumes). `og bake
  lightmaps [project] [--samples] [--bounces] [--texels-per-metre]
  [--max-size] [--sky] [--seconds] [--threads] [--scene]` writes
  `<scene>.lightmap.ogt` and `.ktx2` beside the scene, which the cook makes
  an `og.Lightmap` blob (`cook --report` lists it). `og.Model.static`
  marks what the bake covers; the runtime draws a scene's bake when its
  scene and model hashes still match and otherwise reports why through
  `og.render.bakedLighting` (`BakedScene`); `RenderSettings.bakedLighting`
  (on by default) turns it off; `og doctor --project` warns `unbaked`.
  `og.LightVolume` (`size`, `spacing`): baked SH probes sampled as ambient
  light by what moves through them. `og.RectLight` (`color`, `intensity`,
  `width`, `height`, `twoSided`) shaded with LTC. `cookie` on
  `og.SpotLight` (a texture) and `og.PointLight` (a cube), 24 layers at
  most. `Fog.volumetric` with `scattering`, `anisotropy` and
  `volumetricDistance`: a froxel compute pass named `volumetric` on the full tier, a
  sun-only raymarch on WebGL2; `RenderCapabilities.volumetricFog` names
  which. fx-showcase stations `baked` (a Cornell box, `baked:on|off`) and
  `volumetric` (window shafts under a day-night sun, a rect light, a
  lattice cookie, a lantern's cube cookie); gfx scenes `rect-lights`,
  `light-cookies`, `volumetric-fog` and, natively, `baked-cornell`.
- Wave 3's end (`scripts/wave-end.sh`, docs/phase-1.md): the lightmap
  reader builds for wasm32; fx-showcase's light-stage entity is
  `render-light` (its name clashed with the fluids station's `light`
  fluid when one world shows every station) and its `none` look also
  puts the HUD back in the prototype style; tier references recorded
  for WebGL2's sun-only volumetric fog (`volumetric`, `volumetric-fog`),
  the fluid surfaces (`fluids3d` on WebGPU, `water` on WebGL2),
  third-person's `beacon` and the two webgl2 looks whose badge wears the
  preset's style.

Debt round (merged 2026-10-23, in progress: what the depth streams and
`ui/style` left owed, `tests/evidence/debt/<stream>/NOTES.md` each), API
changes by stream:

- `render-targets`: an `og.CameraView` or `og.Mirror` draws what the main
  pass draws on its layers: transparents sorted from the view's eye, world
  sprites and text, particles (`og.ParticleSystem.layers`), lines and
  points; an ldr target or viewport carries exposure and the tone curve,
  an hdr target linear radiance. The main camera keeps GPU culling in a
  frame with views (views and probe faces cull on the CPU and append; the
  GPU cull tests the layer mask). A target's `depth` texture is drawn on
  every tier, WebGL2 included (`RenderCapabilities.renderTargetDepth` is
  true wherever a renderer exists). `PostEffects.custom1` to `custom8`
  (eight slots, `maxCustomPosts` 8 on every tier). `describe` entries name
  the `views` an entity shows in and `Description.views` lists each
  view's own id pass. The mirror's near plane sits a hair off the glass.
  Gfx scenes `views-layers` and `views-depth`.
- `render-lod`: reflection probes are picked per instance (a shared mesh
  splits into one batch per probe); captures draw skinned meshes in their
  pose, terrain, scatter near the box, the sky and transparents; terrain,
  scatter, custom, water and fluid shaders take the probe's ambient and
  reflected light; the prepass reflectivity reads the roughness and
  metalness maps. `expect:entity:<id>:<path><op><value>` and
  `expect:timings:<path><op><value>`; fx-showcase playtests `lod` and
  `reflections`. A probe bake's key hashes the room's content
  (`cooked/probes/<key>.ogt` beside it) and `og doctor --project` reports
  `stale-probe`. Gfx scenes `probe-two-rooms` and `probe-outdoor`.
- `style-names`: og-ui's Rust names say style (`style_file`, `Art`,
  `Theme::art`, an element's `frame`); ui-kit's `style:` command and
  `kit-style-*` sheets; Path Guard and Rumpus Isle on the current cartoon
  art (modest corners, no pills; Path Guard's primary `#974009` for 7:1).
  `og.load.look`'s docs call the look's `ui` line its style.
- `lantern-vale-physics`: the pile's rocks are density 150 (46 kg), light
  enough for the controller's capped push (`pushStrength`, 1,000 N); the
  physics playtest checks the rocks the hero walks into, not the pile's
  own slump. No engine change.
- `geometry-video` (first item): the `debug-overlay` gfx scene draws two
  `og.debug.text` labels. The og-cli doctor tests clear `OG_TSRS` and
  `OG_ENGINE_PACKAGE` themselves and skip an installed tsrs.
- `render-light`: every loaded scene's bake draws (each mesh binds its
  own scene's pages); the physical, array, material-shader and compact
  batches draw lightmaps and light volumes; rect lights light terrain,
  scatter and fluids through LTC and cast shadows (`RectLight.castShadow`,
  a 150° view from the centre read as PCSS); `RenderCapabilities.
  maxLightCookies` (24) with a warning naming the lights left without;
  the WebGL2 raymarch is lit by point, spot and rect lights
  (`RenderCapabilities.volumetricLights`, 8); froxels take eight jittered
  depth samples (no column streaks). Gfx scene `baked-batches`.
- `geometry-video`: the cook builds each model's raycast BVH (`model {
  raycast false }` opts out; `og cook --report` shows `bvhBytes`); tubes
  along an `og.path` space their rings by arc length; timelines hot
  reload; music voices loop (`looping`); per-player video textures; web
  frame capture; video on a mesh's material map and an `og.ui` image.
- `two-d-math`: the DragonBones import carries skew, colour offsets,
  zOrder, IK timelines, nested armatures, bounding boxes
  (`og.anim.skeleton2dBoxes`) and Bézier curves; IK over chains of any
  length (one and two bones keep their exact solvers, longer chains
  FABRIK). Behaviour change: `og.IkTarget2D.weight` multiplies the
  constraint's own weight; it used to replace it. The game-2d template's
  lamp keeper walks (`keeper:<x>`, feet planted by IK).
- `probe-rooms`: trace spans carry their real starts (systems, passes,
  og-lanes jobs on their lane threads, cook outputs); GPU pass timing
  through timestamp queries (`ProbeTimings.gpuPasses`, the trace's gpu
  thread, `RenderCapabilities.gpuTimestamps`; every pass writes its own
  pair, resolved once the frame is done, so Metal reads real numbers);
  the web trace (the shell's `trace()`/`saveTrace()`, the play page's
  `?trace`); scene write-back to each instantiated scene's own `.ogt`
  with spawned entities under a marked section (`resource <Name> { }`
  scene entries); opaque UI backgrounds count in `describe`'s occlusion;
  `cook --report` figures for every asset type with new budgets
  (`soundSeconds`, `videoSeconds`, `fontGlyphs`, `navPolygons`,
  `sceneEntities`, `emitterParticles`); room playtests take `then` and
  `quit` (a lockstep gate's restore no longer moves scheduled steps);
  `?room=<id>` Play links (`RoomInfo.link`); the arena archetype polished
  to taste 100 / feel 66.
- The wave's end: a lightmap bake is keyed on the scene's source text
  and each static model's geometry (positions, normals, uvs, colours,
  indices of its LOD-0 parts), no longer on the cooked blobs' hashes, so
  a re-cook that keeps the geometry (a cooker revision, the BVH in the
  cook, a texture or raycast option) keeps the bake; before, every cook
  format move staled every bake of every project (the fx-showcase
  Cornell box drew unbaked on both browser tiers). `og doctor --project`
  and `og.render.bakedLighting` read the same key. The browser test
  scripts cook their project on every run, as `og test` does.

## 0.2.0 (2026-10-23)

API additions and changes (a minor bump, §3), by the stream that landed
each:

- API: every word of the kit HUDs is a field with its English default
  (kit-words, 2026-10-22): `PlatformerHudDesc.jumpLabel`,
  `ShooterHudDesc.killsLabel` and `fireLabel`, `PuzzleHudDesc.movesLabel`
  and `targetLabel` (`{target}`), `IdleHudDesc.levelLabel` (`{level}`) and
  `rateLabel` (`{rate}`), and `pauseLabel` on each of them (the pause
  button's name for a screen reader); a localised game passes its string
  table's (`og.i18n.t`). Additive; the defaults are the words the HUDs
  showed before.

The `platformer-2d` stream (Brasswick's runtime findings,
`examples/brasswick/FINDINGS.md`, 2026-10-22):

- `og.WorldRestored` (event, module world): delivered the tick after a
  `world.restore` applies, with `from` (the tick the world was at) and
  `tick` (the saved tick), for systems that reset what lives in the session
  (#8). `world.restore`'s doc says so, and that an `og run --commands`
  script goes on to its last step whichever way a restore moved og.Time.
- `og.PARTICLES_ARMED` (const, module render): `ParticleSystem.startTick`'s
  default, never written; a one-shot (`looping false`) emitter stays armed
  and emits nothing until a tick is written, a looping one emits as if
  started at tick 0 (#3). `startTick` carries the default.
- `og.CharacterController.pushStrength` and
  `og.PlatformerController.pushBodies` (default true) and `pushStrength`
  (newtons, default 1000, tunable): the push on the dynamic bodies a
  controller walks or runs into is an impulse capped by the strength, so a
  light body is shoved and a heavy one holds (#6). Unit `newton` added.
- `og.TilemapLayer.colliderLayers` and `colliderMask`: a collision layer's
  own `og.Collider` layers and mask, from its `.ogt` line or Tiled int
  property; 0 takes `Tilemap.colliderLayers` and every layer (#5).
- `PlatformAnswer.time`: a slot call (`saveSlot`, `loadSlot`) answers the
  time of the store answer that ended it (#7). `CharacterEvent`'s doc says
  the event is read the tick after the move (#14).

Also:

- `og run --commands`: the script ends when its last step has run, wherever
  a restore moved og.Time (#13).
- `create-game --profile 2d` writes the 2D starter `templates/game-2d`
  (shipped as `template-2d/`), as does a premise that says 2D, platformer,
  side-scroller, metroidvania or pixel art without saying 3D (#1). The
  install test scaffolds and tests it.
- Knowledge: og-game §8 gives the flipbook grammar line by line (#2);
  og-feel §12 reads the jump event the tick after (#14).

The `dogfood-7` stream (Rumpus Isle, a local-multiplayer party game,
`examples/rumpus-isle/FINDINGS.md`, 2026-10-22):

- `og test --filter <path>`: selects suite files by a substring of their
  project-relative path before anything runs; an empty selection fails
  (#14). No API change.
- Knowledge: og-build on headless rendering (`OG_RENDERER=none`, the
  lavapipe fallback) (#7); og-ui on the UI's entrance timing and the
  presses it rejects (#10) and the iris midpoint (#12); og-game on the
  post-solver velocity a contact event carries (#9).

## 0.1.1 (2026-10-22)

- Fix: the published template, archetypes and knowledge scaffold carried no
  `.gitignore` (npm drops the file from every tarball), so a scaffolded game
  and the platform's overlay had none. Stored as `gitignore` in the packages
  and written back as `.gitignore` by `create-game` and the knowledge mount,
  as `gitattributes` already was. The install test checks both files.
- No API change.

## 0.1.0 (2026-10-22)

The first published version: the API as `crates/og-api/api.ogt` stands at
the tag (modules math, reflect, world, system, game, transform, time, rng,
input, camera, render, load, physics, atmosphere, anim, tween, tilemap,
audio, nav, ai, platform, i18n, ui, systems, commands, probe, terrain,
stream), the `og` tool (run, dev, test, polish, build, cook, probe, check,
sfx, skin, doctor, api), the project template and eight archetypes, the
ten `og-*` skills and the generated wiki. Everything before it is the
build-out recorded in `docs/phase-1.md`.
