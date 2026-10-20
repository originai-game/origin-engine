# Effects showcase

Every og-render effect a game declares from TypeScript (the IDL's
`render` and `load` modules: `PostEffects`, `Lighting2D`, `Light2D`,
`Occluder2D`, `Decal`, `ParticleSystem`, `Trail`, `Beam`,
`og.render.ribbon`, the sprite and material effect fields, screen-space
sprites and text, `og.load.lut`, `og.load.emitter`, a skinned model posed
by `og.anim`), on five stations, with a screen-space HUD and an `og.ui`
badge over every frame. The E5 round adds eight stations (`src/e5.ts`):
atmosphere, weather, water, material shaders and the physics beyond rigid
bodies, each with its own playtest. An E5 station's world is built when it
is first shown (a marker entity `e5-<station>` records it), so a run that
never shows one never simulates its fluid, cloth, ropes and ragdoll.
Phase 2 adds the `terrain` station (`src/p2.ts`), built the same way, and
the animation and physics round four more (`src/anim-physics.ts`):
retargeting, a 2D ragdoll, cloth that folds on itself and fluids that
layer and blend. The
render round adds four more the same way (`src/render.ts`): lights, moon,
materials and skinned, the 3D fluid round one (`src/sph.ts`):
fluids3d, and phase 4's image-based lighting round one (`src/ibl.ts`):
environment.

```sh
og run templates/fx-showcase                        # the window
og test templates/fx-showcase                       # the playtest below
OG_FX_CAPTURE=1 og test templates/fx-showcase       # re-record this tier's references
python3 templates/fx-showcase/tools/make-assets.py templates/fx-showcase/assets   # regenerate textures and the LUT
python3 templates/fx-showcase/tools/make-terrain.py templates/fx-showcase/assets  # regenerate the valley's maps and layer textures
python3 crates/og-tests/scripts/make-studio-hdr.py templates/fx-showcase/assets/env/studio.hdr 256   # regenerate the studio HDR
```

## What it demonstrates, and the skills it exercises

Every effect, the atmosphere and water, material shaders, skeletal animation
and the physics beyond rigid bodies, each declared from TypeScript as data
and proved by a reference frame on four tiers: og-game §9 (the walker's
clip, blendshapes, the ragdoll that follows its walk), §10 (post, particles,
trails and beams, decals, 2D lighting, materials, the HUD layer), §11 (fog,
sky, day and night, weather, water, terrain), §12 (a dissolve and wind-swayed grass in
WGSL) and §13 (joints, a character controller, a vehicle, ropes, cloth, a 2D
fluid, a ragdoll); og-taste §8 for the looks; og-build §3, §4 for the frames
as evidence. `og polish .` (2026-10-19): taste 100, feel 97.


| Station (`station:<name>`) | What it shows |
|---|---|
| `post` | ambient occlusion (GTAO), outlines, god rays from a low sun, bloom on an emissive ball, the `luts/teal-orange.cube` grading, vignette, chromatic aberration, film grain |
| `lens` | depth of field (high quality, focused on the near sphere) and per-object motion blur on a box orbiting fast |
| `fx` | a campfire: `fx/fire.ogt` (a flickering flame sheet), `fx/smoke.ogt` (alpha, noise), `fx/sparks.ogt` (bursts, velocity-stretched, bouncing off the depth buffer); decals from `textures/decals.png` (scorch, an emissive rune, an arrow, a splat); a trail behind an orbiting wisp; a jittering, scrolling beam between two pylons; a rainbow ribbon submitted every tick |
| `2d` | an orthographic camera over lit tiles: two `Light2D`s (one orbiting), crates that cast shadows (`castShadow`), an `Occluder2D` wall, a crate with a hit flash, an additive glowing orb (`glow`, `PostEffects.glow`), `fx/embers2d.ogt` drawn as 2D sprites |
| `stylised` | toon (with `textures/toon.png` as the ramp) and an outline, rim light, dissolve with a glowing edge, a hit flash pulse, screen flash and camera shake on the same beat, the skinned `walker.glb` walking with its first blendshape breathing |
| `sky` | dusk from `og.DayNight` (the sun, sky, light and ambient from the clock, the image-based light regenerated every 5 game minutes) over a colonnade, `og.Fog` distance and height fog tinted to the horizon |
| `weather` | `og.Weather` rain over a yard: streaks falling with the wind, the preset's fog and dimmed light, under an overcast `og.Sky` |
| `water` | `og.Water` over an `og.WaterSurface`: colour by depth over a sloping bed, foam at the posts and the buoy, screen-space reflections (the sky's cubemap on WebGL2), a crate dropped in at 0.33 s that splashes and floats |
| `shaders` | `og.load.shader` materials (`shaders/dissolve.ogt`, `shaders/grass.ogt`, WGSL hooks compiled by the cook for every back end): a sphere dissolving by noise as `og.render.setShaderParam` sweeps its amount, a cut cube, 600 wind-swayed grass blades |
| `controller` | `og.CharacterController` up three steps and a slope, a jump, then off the landing onto a lift (a kinematic body) that carries it |
| `joints` | `og.Joint`: a motorised hinge, a chain of ball joints, a spring; the walker as an `og.Ragdoll` (rig from its skeleton) going limp from its walk at 0.67 s |
| `rope` | `og.Rope` drawn to a ball on a rope joint, a slack rope in the wind, two `og.Cloth` flags in gusts (pinned on the top edge and at the top corners) |
| `fluid` | an orthographic view of `og.Fluid2D` water and a viscous orange fluid drawn as metaballs in a basin, and an `og.WaterLine` pond drawn by `og.Water2D` where a dropped crate floats |
| `terrain` | `terrain/valley.ogt`, a 256 m valley loaded with `og.terrain.load` and drawn by `og.Terrain` as a clipmap: grass, a path painted in the control map, triplanar rock on every slope steeper than 32° and sand below the shore by altitude, blended by the layers' heights; a lake (`og.Water` with `planarReflection` 0.5, `shadows` and `caustics`) mirroring the mountain behind it; a character (`og.CharacterController` on a `SHAPE_HEIGHTFIELD` collider) walking the path up a knoll, its ground read with `og.terrain.sample` |
| `retarget` | one walk from `anims/humanoid-clips.glb` on three characters of other proportions and bone names: the walker (Blender names) and the stout (Unreal names) retargeted at run time with `og.anim.retarget`, the strider (Mixamo names, `rigs/strider.ogt`) by the cook from `models/strider.glb.ogt`'s `animation { library }`; each strides at its legs' length with its feet planted (foot IK) |
| `ragdoll2d` | a 2D world (`og.Physics.profile` 2): the side-on `models/puppet2d.glb` walks on a ledge, then goes limp as an `og.Ragdoll` of rapier2d bones on hinges about z and is knocked off it |
| `cloth` | an `og.Cloth` sheet with `thickness` (self-collision), `substeps` and `friction`, lowered onto the floor while its bar sweeps back and forth, laid in folds layer on layer without passing through itself |
| `fluids` | eight `og.Fluid2D` in one frame: a heavy blue fluid (`density` 3) released beside a light orange one slides under it (one `group`, `mixing` 0), red poured on yellow blends into one orange (`mixing` 1), and four cups of lone fluids |
| `lights` | a night plaza under 256 `og.PointLight` and `og.SpotLight` entities in many hues, drawn by Forward+ clustered lighting (every light shades only its cluster's pixels) |
| `moon` | `og.DayNight` at 21:00 with `moonSize` 2.2 and `moonPhase` 0.36: a waxing gibbous moon over the rooftops of a square, moonlit, with stars around it |
| `materials` | every `StandardMaterialDesc` feature from TypeScript: metalness and roughness rows, emissive, `textures/bumps.png` as a normal map (glTF's frame), clearcoat glossy and rough, `textures/grille.png` cut by `alphaTest`, transparent glass, a double-sided panel, unlit, a squashed sphere (normals under a non-uniform scale) |
| `skinned` | the walker with `shaders/ripple.ogt` set on its loaded materials by `og.render.setMaterialShader`: gold bands rippling up its skinned body as it walks |
| `fluids3d` | two `og.Fluid3D` of one `group` in a tiled basin (their `bounds`): a blue pool of 1600 particles, and a heavy orange fluid (`density` 3, `mixing` 0) poured into its left end by an emitter (`emitRate`, up to `maxCount` 500) that sinks under it; drawn as one screen-space surface, the tiles seen through it tinted and bent, lit by the sun and an orange lamp, the low front wall cutting it (about 2 ms of CPU a tick at two substeps) |
| `environment` | `og.load.environment(world, "env/studio.ogt")` (the cook prefiltered `env/studio.hdr`, a synthetic 256 × 128 studio, into a 128² GGX cube and irradiance harmonics) set as `og.Environment.map`, turned 40°, drawn behind with `background`: five spheres on a plinth (mirror chrome, brushed chrome, gold, red lacquer, chalk) lit by it alone but for a weak sun along the key light; `RenderSettings.toneMapping` `agx`, auto exposure adapting at 1 EV/s up and 2.5 down |

`tests/frames.og.ts` shows each station, waits for its effects (TAA,
particles) to settle, takes its frame at 1280×720 and compares it with
`references/<tier>/<station>.png` under og-tests' rule; it also checks
that the HUD's opaque accent bar is exactly its authored colour on every
station (the overlay is drawn after tone mapping, grading, vignette, grain
and TAA). `references/metal` is recorded on Apple Metal, `references/vulkan-cpu`
on Mesa lavapipe (CI's tier), in the platform's E2B sandbox or a local linux/amd64 Docker container (`tests/evidence/E3/fx-showcase-badge.md`).

`dump_entity:stage` reads the `Stage` component (the station and how many
ticks it has been shown).

Each E5 and phase 2 station has its playtest, `tests/<station>.og.ts` (`tests/station.ts`):
the station shown at 0.5 s in a fresh run, its frame taken after it has run
(and one half a second earlier), compared with `references/<tier>/<station>.png`,
and checks of what it shows from dumped entities and the two frames (the
crate floats, the character rides the lift, the hinge turned and the ragdoll
blended to its bones, the ball stays within its rope's reach, rain and
flags move). `OG_FX_CAPTURE=water.png og test .` records named frames only.
The E5 and phase 2 stations have references on all four tiers: `metal`, `vulkan-cpu`
(lavapipe in the og-lavapipe container, reproduced at RMSE 0) and the browser
tiers `webgpu` and `webgl2`, which come from
`packages/engine/tests/web/reference-e5.ts` (`docs/ci.md` §Browser reference
tiers).
