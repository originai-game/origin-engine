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
`crates/og-api/api/<module>.ogt`.

## Unreleased

- **Global illumination: screen rays across the screen, `both` counted
  once, cheaper probe sampling.** `og.GlobalIllumination.maxDistance`
  defaults to 16 m (it was 4): a screen ray stops where it leaves the
  screen and spreads its steps over what it crosses there, so it
  gathers about 0.65 of a room's bounce where it gathered 0.40. With
  `method: both` the screen rays reach one probe spacing and replace the
  probes' light only from the directions they hit, so the bounce is no
  longer counted twice (it was 1.1 to 1.2 times a path-traced
  reference). A lit surface gathers the probes once for its diffuse
  light and its reflections' occlusion (the probe GI's main pass at
  1080p on Metal: 7.98 ms to 3.70 ms), and a probe moved out of a wall
  no longer leaves a dark spot on it. Doc change on `maxDistance`.
  Behaviour change: frames with the screen GI on are brighter where
  light comes from farther than 4 m; set `maxDistance: 4` to keep the
  old reach.

- **Stylise: conformed art under a tone curve, washes and fading strokes.**
  The quantise tells conformed art by the lit colour before exposure and
  the tone curve, so a tone-mapped look keeps it unstepped too.
  `hatchingKeepColor` lays a light wash (as light as one-ink hatching's
  paper, keeping the hue and more of a strong colour's chroma than a weak
  tint's; conformed art keeps its authored colour, lifted as one-ink
  paper is) under its strokes, a 1-bit quantise dithers conformed art too
  (its dither is its only shading), and every
  hatching stroke set thins and fades in from clean paper at its tone
  instead of starting at a third of its ink. The cook's own-palette cut
  weighs each texel by how smooth its neighbourhood is, so a busy
  backdrop's sky and its clouds keep two colours. Doc changes only on
  `Stylise.hatching` and `hatchingKeepColor`. Behaviour change: frames
  with hatching, colour hatching, tone-mapped conformed art or a
  conformed busy texture move; detailed-doodle hatches a colour wash.

- **GPU-driven crowds.** og.CrowdAnimation members draw as og-render
  crowds: culled (frustum, two-phase Hi-Z), given their level and drawn
  indirect on the GPU, shadows included, with animated impostor cards as a
  crowd's far level (`Crowd::impostor`). Metal M4 Pro, 1280×720 TAA:
  5k 18.4 → 5.2 ms, 20k 63.4 → 10.2 ms, 50k 129 → 26.3 ms a frame. New
  `RenderCapabilities.gpuCrowds` (false where the device lacks indirect
  first instances: crowds then take the CPU path) and
  `RenderDesc::without_features`.

- **Weather on surfaces and cloth sheen (Wave B).** New
  `StandardMaterialDesc.weathering` (default 1; 0 opts a material out of the
  weather). New `WeatherState.dust` (read-only), `dustTime`, `washTime` and
  `dustColor`: a `WEATHER_DUST` storm settles dust and sand on what faces up,
  and rain washes it off. Lit surfaces now dry in patches from their high
  spots once the rain stops, run with rain streaks on walls, splash under
  the rain, and take their relief from `aoMap`, whose crevices fill with
  puddles, snow and dust first. The screen-space reflections read the wet
  roughness and the puddles' flat, rippled normal from the prepass. Grass
  darkens and glosses in the rain, and lying snow buries its roots and bows
  its blades (every pass alike, the shadows and motion included). A
  terrain's puddles, snow and dust follow its layers' height maps (the
  mortar joints fill first), its prepass writes the wet gloss for the
  reflections, and lying snow lifts the drawn ground by `snowDepth`, at
  most 10 cm (drawn only). Sheen
  is now Estevez and Kulla's Charlie with their shadowing and soft
  terminator, lit under the image-based light through its directional
  albedo (the BRDF table's blue channel) and energy-conserving. Velvet and
  cotton draw differently from before; materials without sheen are unchanged.
  `og.StandardMaterial` reflects at version 11.

- **Destruction (og.destruction).** A `fracture { }` block in a `type model`
  or a glTF's import settings makes the cook cut the model into chunks by
  a fracture profile per material (`style voronoi|concrete|brick|wood|glass`:
  Voronoi cells, Lloyd-relaxed chunky lumps, runs of one to three bricks
  along each course of a running bond, cut along its mortar with their
  bricks as children, splinters along `grain`,
  radial and concentric shards round `impact` in `rings`; `chunks`,
  `seed`, `children`, `brick`, `pattern volume|slab|auto`, `impact`,
  `anchor bottom|ends|none`, `interiorScale`, `interior`), written as
  `<model>#fracture` (`og.Fracture`). New `og.Destructible` (toughness,
  strength, friction (Mohr–Coulomb), tension (a fraction of strength:
  masonry about 0.1; at 0.1 or below a standing block needs its weight
  within the middle third of what bears it from below, or comes loose),
  density, impactThreshold,
  impactRadius, anchored, breakFx, hitFx,
  debris; a hit drives the pieces it breaks loose along its path; written fractured, broken, loose), `og.Chunk`, the event
  `og.Fractured`, the resource `og.Destruction` (maxChunks, debrisVolume,
  debrisLifetime, fxLifetime, stressIterations: 0 solves until it
  converges, stressBudget: a tick's solver work in bond-iterations, past
  which a solve waits for the next tick), `DamageDesc`,
  `ExplosionDesc`, `DestructibleInfo`, `DestructionCapabilities`,
  `CAUSE_*`, `MATERIAL_*` (brick, concrete, wood, glass, stone) with
  `DestructibleMaterial`, and `og.destruction.damage`, `explode` (its
  `impulse` per square metre of the face each piece shows the blast, so a
  brick flies faster than a run of bricks but not by their count),
  `fracture`, `info`, `material` (a material's og.Destructible tuning, to
  spread into the component; concrete's impactThreshold 1500, so a
  column's own falling top leaves its stump standing) and `capabilities`.
  A blast's uneven share and turn per piece fade with its push, so the
  pier feet it barely reaches keep their footing. A loose run of
  no-tension masonry longer than three blocks comes apart into its
  blocks as it breaks loose. Prelude units `m3`, `pascal`, `newtonSecondPerM2`. The
  stress solver is NVIDIA Blast's (BSD-3), each joint judged at both its
  edges (a pressed joint whose load lies off its middle opens). Lazy: a world without
  destructibles keeps its hash. The skill `og-destruction`; the example
  `examples/demolition-yard`. `type model` materials take `map`,
  `normalMap` and `normalScale` (image files beside the model) and the
  physical `transmission`, `ior`, `thickness`, `clearcoat` and
  `clearcoatRoughness`.

- **UI phase 0 grammar, typography and sheets (breaking; next minor).**
  `typography` drives every legacy ramp alias, including weight and tracking;
  `hero` adds the top step and `case upper` uses locale-aware Unicode casing.
  `og upgrade` converts v1 styles and archives redundant v2 ramps for review.
  What moves on a shipped screen, and the fix for each: `xl3` (and `xl4`)
  follow the title role, so a ramp's `xl3 30 34` draws at the title's size;
  keep it by setting title, or accept the title's size, since one role
  cannot hold both. `displayXl2` (and `xl7`, `xl8`, `displayXl`) follow
  `hero`, which without a line of its own is display × 1.5 (a 60 px line
  drew at 81); `og upgrade` now writes a `hero` line keeping what
  displayXl2 drew. `meta` follows caption, as every style's ramp had it.
  Separately, text in the numeric font role (`font: "mono"`) now sets
  tabular lining figures, so a timer holds its width as it ticks; its
  digits take the widest figure's advance (a lone "1" widens from 8 px to
  14, a lap time by about 3). Leave room for that, or set digits that must
  stay proportional in another font role.
  Parent rims now draw below selection, hollow focus and raised child badges.
  New `LabelDesc.onScene`, `IconDesc.secondary` and `IconDesc.secondaryTint`
  support scheme-aware scene text and two-tone icons. Document `tabs`,
  arbitrary painted selectors, ornament tint, token alpha and independent
  selection/accent/scene roles complete the dressing grammar. MSDF overlap
  queries handle negative bearings; sign correction precedes error correction.
  `og ui sheet <pack|kit>` is an on-demand baseline tool, never a gate step.

- **Terrain displacement (terrain v2, T4).** Two new members:
  `TerrainLayerDesc.displacement` (metres, 0: flat; the terrain file's
  layer key `displacement`) and og.Terrain's `displacementDistance`
  (metres, default 40, 0: none). A layer with a displacement raises and
  lowers the drawn ground by its height map (its albedo's alpha, about
  its middle) near the camera. The layers and weights are the ones the
  lit pass shades with, the maps are read over each level's spacing, and
  the relief fades out over the last 40% of the distance and on steep
  ground. The silhouette and the shadows then carry the relief. A
  displaced terrain whose `lodSpacing` is 0 draws its finest clipmap
  levels finer than its samples: halved down to a 32nd of the finest
  displacing layer's `tile`, and at most to a 32nd of a sample. Between samples every level now reads the field on its
  cells' split triangles, as collision does, instead of bilinearly; this
  also changes a `lodSpacing` finer than the samples. In the morph band
  even corners now also move, to the coarser level's corner as that level
  reads it, so displaced levels meet without cracks. A layer without a
  normal map is shaded with its relief's slope; the shadow lookup keeps
  the field's normal (offset along the drawn one, a face turned to a low
  sun shadowed itself in squares of shadow texels). Collision,
  `og.terrain.sample` and picking keep the undisplaced field. The terrain
  blob's layer row is v4, so terrains cook again. Off by default:
  existing terrains draw as before.

- **The clipmap morphs in height only, and tiles fade in over seconds
  (terrain v2, T2).** A clipmap cell is now four triangles fanned around
  its centre. Near a level's outer edge an odd corner sinks or rises in
  place to the coarser level's surface under it (CDLOD's midpoint
  geomorph), and a cell's centre moves from its own diagonal to its
  coarser cell's. Vertices no longer slide sideways, and a cell's
  diagonal no longer flips mid-morph. Zero-area triangles along each
  morphing level's edge close its T-junctions. A peak inside a morph band
  now sinks smoothly towards the coarser level's surface instead of
  sliding up to one quad sideways. A streamed tile fades in over 0.2 s of
  the host's frames (`frame(elapsed)`) instead of 12 frames, so it takes
  as long at 30 or 144 Hz. Terrain frames move.
  Each level's morph band is placed by the height error the next coarser
  level would show on screen (chunked and CDLOD terrain's screen-space
  error): where that error is over og.Terrain's new `lodError` pixels
  (default 1), the finer level reaches further out, and the levels grow
  to 128 quads a side when their ring cannot hold it, so a peak that is
  large on screen keeps its shape. The errors are measured per block of
  each decimation, from the field in memory (again only where an edit
  changed it) or with each streamed tile (`TerrainTile.error`); nothing is
  cooked. On an M4 Pro the island costs 0.8 to 1.3 ms more a frame (seen
  from 150 m: 6.0 ms instead of 4.8); a lodRing of 256 costs 8.1.

- **Terrain collision follows an edit's region (terrain v2, T6).** A
  static SHAPE_HEIGHTFIELD collider over more than 256 cells a side is
  built as one collider per 256-cell chunk, and `og.terrain.edit` and
  `og.terrain.stamp` hand physics the rectangle they changed
  (`PhysicsSystem::edit_heightfield`): the next step rebuilds only the
  chunks under it, so bodies elsewhere keep their contacts and sleep. A
  1 m brush on a 4097 × 4097 field steps in 1.5 ms where rebuilding the
  field took 83 ms. A body on two chunks touches the terrain once (one
  CollisionStarted). Worlds whose collision terrain is over 257 samples a
  side save and hash differently from before (their colliders are the
  chunks); smaller ones are unchanged.

- **UI extensions and authored data packs (phase 0).** `og ui add/list/eject`
  and `og style add` discover directory extensions with `extension.json`.
  Installed documents cook as ordinary assets on both hosts; kits export
  typed namespaces beside the frozen `og.ui`. Pack choices, skills and
  nested assets live with each style. `og style new --seeds` is removed:
  edit authored packs directly, and bootstrap only new packs. The primary
  colour is no longer contrast-lifted. `og style check` finds a look declared
  in `GameDesc` when the project file does not name one, without starting
  a world. Pack acceptance uses parse/asset checks, not contrast thresholds.

- **A reviewer's measures (the art director, origin-agent).** `og frame
  measure <png> --region x,y,w,h [--against …] [--units fraction]` prints
  WCAG contrast, high-pass energy, mean colour and a row ramp for any
  region of a frame, the same on every machine. `og style check` captures
  every screen `screens {}` or `ui { screen }` declares (any name, not
  only title, play and results), at several viewports with `--sizes
  desktop,phone`, labels each frame `<screen>@<viewport>` with its size
  and renderer tier, and its `--json` carries the look's flattened art
  blocks and the scenes no capture script reaches. `og ui check --json`
  findings carry their `value`, `bar`, `unit` and a box per cell (`at`)
  with the cell's frame. The catalogue gains `intended` per style (the
  traits that look like defects but are the style), and visual-acceptance
  gains §5 (the art bar) and §6 (verdicts). All additive.
- **Probe global illumination.** These members are new and off by
  default, so no existing frame moves.
  - `render.GiMethod` (`screen`, `probes`, `both`).
  - `GlobalIllumination.method`, default `screen`.
  - `GlobalIllumination.probeSpacing`, `probeCascades` and `probeRays`.
  - With `probes`, DDGI probes traced in software through a simplified
    scene light every surface's ambient light: bounce from off screen,
    and the sky occluded under roofs and in rooms.
  - With `PostEffects.ao`'s GTAO on, the probes are read along its bent
    normals.
- **Shadows: weather, filtering and the box-base leak.** A cloud cover or
  og.Weather's dimming now reaches the sun's shadow too: the light it
  scatters fills in up to 65 % of the umbra where the sky over it is open
  (under a roof it stays shade) and softens the edges (PCSS) under an
  overcast, where the shadows stayed hard and black; a broken sky draws as
  before. The sun's hard shadows filter with a 3×3 tent over the
  receiver's plane (Isidoro's receiver-plane bias) instead of a 2×2 PCF,
  so edges no longer step texel by texel and thin casters keep their
  shadows; grass and scattered copies keep the 2×2. A soft sun's
  penumbra is the light's convolved with that filter. The sun's PCSS
  spiral turns per pixel and, under TAA, per frame (interleaved gradient
  noise) instead of printing rings. Point
  and spot shadows on lit surfaces filter over 7×7 texels away from the
  caster, hardening to the 3×3 tent at contact, so a far edge no longer
  steps texel by texel. A closed, single-sided receiver compares its
  shadows past its own depth, so the floor along a box's shadowed base
  is no longer lit through the box's bottom face (a grey 1 px line). No
  IDL change; render-only, no simulation pins move.
- **Terrain horizon shadows and sky occlusion (T5).** `og cook` bakes each
  terrain tile's horizon (the highest ground in 8 directions, from a max
  pyramid so no ridge is missed; tile row v4, `og cook` redoes terrain
  blobs). Past the sun's shadow cascades a valley behind a ridge now falls
  into its shade out to the horizon, and the sky a valley's walls hide
  lights it less, so far mountains read their form. New `og.Terrain`
  field `horizonShadows` (default on). og.Terrain is now registered
  lazily, as og.Grass is: a world without a terrain keeps its type table
  and hash as the component grows; the five pinned world hashes moved
  once for it.
- **Stamps meet the ground without a trench.** `og.terrain.stamp`'s carve
  lowers the samples on the mesh's edge to its underside, not `depth`
  under it: the cell beside a cliff's foot no longer dips into a dark
  line. `og.TerrainStamp.blend` is wholly the ground over its lowest third,
  so a blended foot is the ground it meets.
- **Terrain block culling (T2).** Each clipmap block is culled by its box
  with the height range of the tiles under it (not the whole terrain's),
  and blocks off the terrain draw nothing.

- **Terrain edits cost what they touch (T6).** An `og.terrain.edit` or a
  stamp's carve now builds again only the tiles its brush reaches (a few
  per level, where every resident tile was rebuilt and uploaded), and
  updates the height and control pyramids only there. A brush visits only
  the cells and control texels under it, a field is copied once per edit
  instead of twice, and a terrain's height range is measured when its data
  is set instead of every frame. Collision still rebuilds the whole height
  field per edit (parry's height field takes no partial update).
- **Terrain far field (T2).** A terrain's coarser tile levels hold the
  mean of the ground under each sample (a centred tent over the finer
  level) instead of one decimated sample, so far silhouettes no longer
  shimmer as the camera moves; collision and `og.terrain.sample` are
  unchanged. Its normal mips average the unit normals and keep their
  spread (Toksvig), which widens the far specular instead of the slopes
  flattening and shining. Each clipmap level is culled in 4 × 4 blocks:
  looking out from a terrain's edge draws under half the triangles.
  Terrain blobs move to tile row v3 (`og cook` redoes them).
- **Terrains in describe and pick.** The probe's entity-id pass draws each
  og.Terrain as its clipmap draws it (levels, morph, adaptive diagonal,
  holes cut per pixel, its placement) and each og.TerrainStamp's mesh, so
  `describe` lists them (labels `terrain`, `terrainStamp`) with depth,
  coverage and occlusion, and they hide what stands behind them. `pick`
  now answers its point in the world's coordinates far from the zero (it
  answered the render origin's). A clipmap of `lodRing` 16 no longer
  morphs its inner edge off the finer level (cracks of up to half a quad).
- **Terrain stamps.** `og.TerrainStamp { data, geometry, blend }` draws a
  mesh (a cliff, arch or overhang) as part of a terrain: shaded with the
  terrain's layers by its own normal, its lowest `blend` metres taking the
  ground's look and normal so it meets the terrain without a seam.
  `og.terrain.stamp` carves the height field to the mesh's underside (or
  cuts holes under it) deterministically; the carve is logged as its patch,
  so snapshots, the world hash and restores keep it without the mesh, and
  `og.terrain.unstamp` takes it back. `TerrainEdit` gains `stamp`.
- **Terrain material v2.** `TerrainLayerDesc` adds `antiTiling` (hex-tiling
  after Mikkelsen 2022: no visible repeat), `projectSlope` (planar below the
  slope, biplanar past it, so steep faces show their texture square instead
  of streaking), `heightContrast` (a per-layer height-blend window) and
  `macroVariation` (brightness and hue drift, a coarser repeat far off); the
  terrain file takes the same keys per layer. All default to 0: a terrain
  that does not set them draws as before. The terrain prepass now reads only
  the layers' weights and heights for its roughness, never their normal maps.
- **Stylisation: line, toon and paint.** Render-only; a game that sets
  none of the new members draws as before except where noted.
  - `Stylise.paintRadius` is now an anisotropic Kuwahara filter with eight
    sectors over a smoothed structure tensor (Acerola's port, MIT, in
    `third_party/acerola-postprocessing`), in the frame and in the cook's
    texel-space paint alike: thin lines and edges keep their shape, noise
    and grass paint over, no rectangular sector artefacts. Every painterly
    look's frames move. Pigment pooling (`paintEdgeDarkening`) is a
    pigment density at region rims (MNPR).
  - New `Stylise.paintDetail` (default 1) and `paintCharacterDetail`
    (default 0.8), `StandardMaterialDesc.paintDetail`: no stroke crosses
    an object's edge, and detail (a material's, a skinned mesh's face and
    hands) keeps its shape; with them a painted 3D frame runs the
    depth-normal prepass. The site's close-shot `paintRadius` workaround
    is no longer needed.
  - Toon: metals band in their own colour (the toon diffuse no longer
    loses 1 − metallic), a toon highlight sized by roughness; new
    `MaterialStyle.toonAmbient` (default 0: flat ambient and reflection
    under toon, which changes every toon look's frames; 1 the former PBR
    ambient) and `toonSoftness` (default 1 px: anti-aliased band edges).
  - New enum `OutlinePlace` and `MaterialStyle.outlinePlace` (default
    inside, as before): outside and both draw the silhouette line round
    the object through a jump flood (Ben Golus's wide outlines), round and
    anti-aliased at any width; the outline pass's line is coverage
    anti-aliased. `ArtLine` (width, colour, `distanceFalloff`, `place`)
    outlines meshes under the contract where MaterialStyle sets no outline.
  - New `StandardMaterialDesc.wireframeAngle` and `MaterialStyle.wireframe`,
    `wireframeAngle` (default 30): wireframes of feature edges only
    (creases and borders); the neon-vector look uses them.
  - `ArtMotion.squash` sets og.tween.squash's default impact under the
    contract (0.5 the preset's own, 1 rubber hose).
  - `RenderCapabilities.stylise` gains `paintDetail`, `toonAmbient`,
    `outlinePlace` and `wireframe`. The prepass's reflectivity channel
    carries the painterly detail byte above its 16 bits (ssr.wgsl reads
    the low bits).

- **og.TerrainScatter and og.Wind registered lazily.** Like og.Grass,
  og.Water and og.Aurora, they enter a world's type table only once the
  game names them, so a world without scattered copies or wind keeps its
  hash as their fields grow (area 4 added placement rules and the
  simulated wind). Behaviour change: the raw hash of every world without
  them moves once; the pinned world hashes are re-pinned with notes.
- **Grass fields in describe and pick, at field fidelity.** The probe's
  entity-id pass draws each og.Grass under its own entity (label `grass`):
  not per blade (the blades live on the GPU and are never read back) but
  as the field's coverage, the terrain's surface lifted half the tallest
  blade, cut per pixel where the field grows nothing (its layer's weight
  times its density map under one half, a hole, past its distance). A
  pixel between blades answers the field; a box lower than half the
  blades is hidden in it. Over a streamed terrain the cut reads its
  resident level-0 tiles, so the pass stays synchronous with no readback.
  `IdSources.grass` beside `terrains` and `stamps`.
- **Better impostor bakes: a material atlas, backlight and self-shadow
  (G5).** The cook bakes each impostor frame 4 × 4 supersampled with its
  textures mip-filtered (a fine texture bakes to its average instead of
  aliasing), and a third atlas, `#impostor-material` (roughness,
  translucency, metallic; bundle table `impostor_materials`), beside the
  colour and normal-depth ones. A model with a translucent material
  (`KHR_materials_diffuse_transmission`, `KHR_materials_transmission`, or a
  double-sided alpha-masked leaf) defaults to 12 × 12 frames, anything else
  to 8 × 8, where `impostorGrid` does not say. The card reads the atlas:
  its roughness and metallic per texel, the sun through the leaves when
  lit from behind (diffuse plus a Henyey-Greenstein forward lobe), and the
  model's shadow on itself from the depth atlas's frame nearest the light.
  The model cook layout moves to version 3, so models re-cook; impostor
  frames move.
- **Grass shadows by default, self-shadowed canopy (G5).** Behaviour
  change: `Grass.castShadows` now defaults to true, and blades cast into
  the sun's first two cascades instead of one. Every blade is also
  shaded by the field's own canopy between it and the sun (Beer-Lambert
  through the blades' density and width), so dense grass darkens towards
  its roots under a low sun. Grass frames move.
- **Scatter placement rules and GPU culling (G4).** og.TerrainScatter
  gains `noiseScale`/`noiseThreshold` (patches), `footprint` and
  `priority` (later scatters keep clear of earlier ones' copies),
  `avoidWater` (no copies within that height of an og.Water surface) and
  `densityMap`. Placement stays one deterministic function on the CPU
  (`Terrain::scatter_among`, what the renderer draws); the main camera's
  copies are now culled one by one on the GPU (distance, frustum, the
  frame's Hi-Z pyramid where it has one, the density map) and drawn
  indirectly in the prepass and lit pass. New members, compatible.
- **Several kinds of blade per grass field, painted maps, no popping
  (G4).** og.Grass gains `types` (up to three more og.GrassType kinds:
  height, width, curve, colours, share; each clump grows one), `typeMap`,
  `densityMap` and `heightMap` over the terrain's extent. Blades are now
  cubic Bezier curves (Ghost of Tsushima's stem, midpoint and tip), and a
  blade about to be thinned out shrinks to nothing first instead of
  popping. New members, compatible; grass frames move slightly.
- **Grass trample (G3).** Where compute runs, a camera-centred 64 m
  trample map (0.25 m cells) records where `og.GrassPusher` capsules
  stand; grass blades under them are pressed down and away, and
  TerrainScatter copies part, both standing again over `Grass.recovery`
  seconds (default 4). New members, compatible: `Grass.recovery`,
  `component GrassPusher { radius, height, strength }`. Visual only: the
  map never reads back to the simulation.
- **One wind field for all vegetation (G2).** Where compute runs, the
  renderer simulates og.Wind as a camera-centred stable-fluids field that
  grass, TerrainScatter copies, impostor cards and meshlet trees all lean
  by, with a damped spring so gusts arrive and settle. New members (off by
  default in effect, compatible): `Wind.turbulence`, `Wind.gustSize`,
  `enum WindMotorKind`, `component WindMotor { kind, radius, strength }`.
  Impostor cards now sway with their meshes. Wind frames move.
- **Scatter fades without grain.** TerrainScatter's `fade` now shrinks and
  thins copies out (each copy shrinks over its own part of the fade) instead
  of dithering their pixels, and the mesh-to-impostor crossfade thresholds by
  blue noise turned every frame, which TAA resolves. Lit, depth and shadow
  passes agree. Doc changes only on `TerrainScatter.fade` and `impostor`;
  frames with fading scatter move. `dump_render` adds `antialias`,
  `frameIndex` and `taaHistoryFrames`.
- **Shore-aware sea (W1).** `og.Water` adds `seabed` (an og.Terrain
  entity), `shore`, `shoreWaveLength` and `shoreFoam`. Over a seabed every
  wave shoals by `tanh(k d)` and the sea holds under its breaking height
  (H_s ≤ 0.78 d); shore trains roll in parallel to the beach and break
  into surf, and the sand the surf runs up is wet. `og.physics.waterHeight`
  and og.Buoyancy ride the same shoaled sea, deterministically; the
  renderer draws it from the same field. Off by default (seabed 0).
- **Render-rate sea (W8).** A display frame between ticks draws the sea
  at the time its blended transforms are drawn at (render interpolation's
  alpha), so waves move at the display's rate in step with floaters; a
  frame at the tick, a capture and `og test` draw the tick as before. The
  foam history holds through a step back of under a tick.
- **Stylise: palette-true quantise, anchored dither, process print.** The
  frame's quantise keeps a pixel already on a `Stylise.palette` colour as
  it is (art the cook put on the palette no longer turns into a dither
  checker); `Stylise.palette` defaults to `ArtPalette.colors` under the
  art contract. New `Stylise.ditherSpace` (`StyliseSpace`: the dither's
  cells on the surfaces, the sky's on a cube round the camera),
  `halftoneMode` (new enum `HalftoneMode`: `ink`, `cmyk`),
  `halftoneMisregistration`, `halftoneRegion` (new enum `HalftoneRegion`:
  `all`, `shade`, `highlight`) and `hatchingKeepColor`; all off at their
  zero value. `RenderCapabilities.stylise` adds `ditherSpace`,
  `halftoneCmyk` and `hatchingColor`.
- **Conformed art skips the frame quantise; the quantise stays the
  fallback.** Loading a texture adds the cook's `<id>#palette` to the
  renderer's conformed colours (`RenderScene::add_conformed_colors`), and
  Stylise's posterise and dither leave a pixel whose drawn colour lies
  within 6 sRGB levels of one as drawn:
  conformed pixel art keeps its flat fields, while plain, generated or
  runtime-loaded materials still take the look's steps and dither. Under
  `halftoneRegion` shade or highlight the dots are Ben-Day dots, a tint
  over the region's colour at most 0.4 of a cell wide, so a deep shade no
  longer merges into ink with a checker of paper holes, and they fade to
  flat ink on surfaces turned away from the camera, so a floor's cast
  shadow stays one colour. Behaviour change: shade and highlight
  halftones, the pixel looks and bold-comic, move.
- **Scatter model space.** `load.GltfInfo.parts` exposes each primitive's
  geometry, material and primitive-to-model matrix; `geometries` remains
  mesh-local. TerrainScatter applies the named impostor model's placements
  to its geometry, so node-scaled meshes and their cards agree at every
  distance. Mismatched bounds skip the scatter with a runtime warning and
  `render.scatterIssue` in the entity dump. Render-only; no simulation pins change.
- **Breaking: insert ranges.** `world.insert` now refuses values outside
  reflected min/max on both hosts, before mutation or deferred insertion.
  Fix out-of-range component initializers; ordinary field writes are unchanged.
- **Lens flare.** The new lazy resource `og.LensFlare` (off by default;
  a world that never names it keeps its type table and hash) adds Filament's
  screen-space flare (ghosts, a halo with chromatic aberration and a
  starburst on the halo) after bloom, plus the sun's and up to seven point
  and spot lights' own ghosts, smooth aperture discs a few times their
  source's size that grow fainter as they are magnified and keep off their
  own source's glare, each weighted by its visibility in the prepass depth
  on the GPU. It is added in exposed light before the tone curve, so HDR
  output applies to it too. Its members: `enabled`, `intensity`, `ghosts`,
  `ghostSpacing`, `ghostThreshold`, `haloRadius`, `haloThickness`,
  `haloThreshold`, `chromaticAberration`, `starburst`, `sun` and `lights`.
- **Terrain projection diagnostics.** `terrainProjection` shows the dominant
  raw layer's unobscured XYZ weights; `terrainDensity` shows its per-axis
  vertical albedo texel footprints. Entity dumps document their units.
- **Debug shading coverage.** Terrain and grass now expose albedo and
  separated lighting terms. Unsupported 3D shading diagnostics draw a
  magenta/black checker instead of beauty shading or an unrelated term.
  `dump_render.debugCoverage` states supported paths and overlay scope.
- **Model skin diagnostics.** Missing model skin data and incompatible
  palette counts, joint order or inverse bind matrices warn in the log and
  appear as `render.skinIssue` in entity probes and dumps. Equivalent rigs
  from different asset paths remain valid; draw behavior is unchanged.
- **Terrain diagnostics.** `DebugView` adds `terrain` (projection weights,
  clipmap cells and morph seams), `terrainLayers` (strongest raw layer)
  and `terrainMip` (normal-map mip footprint estimate). `dump_entity`
  reports terrain clipmap regions, triangle spacing, seams and layer rules.
- **Adaptive terrain diagonals (simulation change).** Terrain cells split along
  the diagonal whose corner heights differ less; ties retain the original
  (-x, -z) to (+x, +z) split. Rendering, `og.terrain.sample`, navigation baking
  and the native height-field collider agree. This removes fixed-diagonal
  lips at steep contours. Terrain contacts and affected simulation replays
  can change; re-record them after checking the new surface.
- **Garment draw compute.** Per-frame skinning, simulated offsets and welded
  normals run in compute, including `og.motion`'s generated palette. Previous
  drawn positions stay on the GPU for TAA and motion blur; devices without
  compute or vertex storage keep the CPU draw fallback. The CPU solver,
  bind-pose constraints, queries and saved state are unchanged.
- **Cloth and soft-body draw surfaces.** Compute builds area-weighted normals
  and a Phong-tessellated surface, with previous fine positions for TAA and
  motion blur. `DisplaySettings.clothSubdivisions` selects 1–8 segments per
  coarse edge (0 selects two). CPU solvers, point queries, volume and centre
  are unchanged; devices without compute or vertex storage keep the coarse
  mesh and CPU normals. Simulated surfaces use regular draws, not meshlets.
- **Garment bind-space constraints.** `og.Garment` measures its rest lengths
  and pin band after the skeleton's rest-pose skin transform, rather than
  from the raw mesh. Nonidentity bind palettes no longer stretch the cloth
  on its first tick without forces. Garment simulation hashes change;
  older saved garments rebuild on the skin at their next posed tick so
  their old constraint lengths do not survive the fix. Mesh topology and
  `garmentPoints` ordering are unchanged.
- **Motion performance: tick-owned looks and authored reactions.**
  `og.motion.lookAt` / `LookAtDesc` add a timed spine, neck and head look
  above locomotion; `og.motion.react` / `ReactionDesc` play a humanoid
  `type motion-action` asset with entry, looping hold, exit and optional
  head-local hand contact. Empty descriptors release either overlay.
  `MotionStatus.looking` and `.reacting` include their blend intervals.
- **Breaking: `WalkGoalDesc.deceleration` and `.arrivalHeading`.**
  Walks brake at 3 m/s² by default and finish after the optional arrival
  turn. With `og.CharacterController`, physics owns collision and ground
  height. Schedule arrival from `motion.status`, not distance / speed;
  existing walks now take longer. Existing saved walks retain their old
  constant-speed rule; new saves include the performance envelopes.
- **Added: `og.Aurora`, the aurora in the procedural sky.** A resource
  (`enabled`, `intensity`, `activity`, `curtains`, `direction`,
  `distance`, `spread`, `energy`, `speed`, `seed`, `greenColor`,
  `redColor`, `nitrogenColor`), off by default. It ports Lawlor and
  Genetti, "Interactive Volume Rendering Aurora on the GPU" (Journal of
  WSCG 19, 2011). The curtains are curves along the auroral oval, folded
  by travelling waves. Their curls come from a Kelvin–Helmholtz
  stable-fluids strip wrapped along each curve. The 2D electron-flux
  footprint is drawn each frame with a distance field the sky's march
  steps by, and is extruded upward by the Lazarev energy deposition over
  the U.S. Standard Atmosphere 1976, tabulated by altitude and
  characteristic energy. The line heights and colours follow Whiter et
  al., "The altitude of green OI 557.7 nm and blue N₂⁺ 427.8 nm aurora"
  (Ann. Geophys. 41, 2023, CC BY 4.0): green and blue peak near 114 km,
  red glows above 200 km, and a nitrogen lower border appears for hard
  electrons. The aurora is drawn under the clouds and dimmed by the air
  toward the horizon. The sky pass gains one texture, made only the
  first frame a scene has an aurora (8 of 16 sampled textures).
  Frames: `aurora-arcs`, `aurora-breakup`.
- **Changed: og.Garment reads its freedom from the vertex colour's red
  times its alpha.** A textured garment can paint the mask in the alpha
  over a white colour, which an opaque material does not show; the red,
  which multiplies the base colour, tinted the garment where it pinned
  it. Garments painted in red with alpha 1 (or without colours) behave as
  before.

- **Added: `og.PlatformNoticeKind.host`, a message from the framing page.**
  A Player sends `platform` `{ notice: { kind: "host", data: { name, ...
  } } }` and the game reads it as og.PlatformNotice at the next tick's
  start, like a purchase or an invite notice. The engine gives it no
  meaning; the game and the page that frames it agree on the names
  (engine.origingame.ai's hero preloads and switches scenes with it). A
  publish build has no other page-to-game channel.
- **Added: `og.Buoyancy`, floating bodies on water (R5).** A component
  beside og.RigidBody and og.Collider (`waterDensity`, `samples`, `drag`,
  `angularDrag`, written `submerged`): the body's colliders voxelised into
  a hull whose cells under an og.Water (a flat lake, or a sea's waves as
  the grid draws them) or an og.WaterSurface (its rest level) lift it by
  the water they displace at each cell, so hulls right themselves and
  pitch and roll with the waves, computed in the tick and the same through
  a restore. `og.physics.waterHeight`'s sea is the same surface. Lazily
  registered: a world that never names it hashes as before.
- **Screen documents and `og ui eject` (docs/ui.md §11.1).** The engine
  package ships `src/ui/screens/`: 59 `type screen` documents, one or
  more for every screen type of docs/ui.md Appendix B, each with its
  viewmodel, events, slots, string keys and a preview fixture. `og ui
  eject <screen> [project] [--force]` copies one into the game's
  `assets/ui/` to edit as its own; `og ui eject --list` names them with
  their rows.
- **The inspector and gizmos (development builds).** F1 opens a panel over
  the game: entities and resources, every reflected field edited through
  the probe's `set` with undo, Save through `save_scene`; translate, rotate
  and scale gizmos on the selected entity (W, E, R; world or local axes;
  Ctrl snaps), each drag one undo step; a prefab instance's overrides
  applied into the prefab or reverted. On a web page Save hands the texts
  to a Stage that takes `scene_saved`, or downloads them (the frame
  answer's new `saved`). The probe gains `inspect`, `dump_inspector` and
  `Probe::entity_prefab` (docs/design.md §3.6). A publish build has none
  of it.
- **Breaking: the fixed screen templates are removed (docs/ui.md §11).**
  `og.ui.mainMenu`, `pauseMenu`, `settingsScreen`, the five HUDs
  (`platformerHud`, `shooterHud`, `puzzleHud`, `idleHud`, `racingHud`),
  `cutscene`, `shopScreen`, `inventoryScreen`, `levelSelect`,
  `resultsScreen`, `leaderboardScreen`, `tutorialOverlay`,
  `loadingScreen`, `confirmDialog` and `alertDialog` are gone with the
  types only they used (`GameSettings`, `SettingsDesc`, `MainMenuDesc` …).
  A game ejects the screen document (`og ui eject main-menu`, `pause`,
  `settings`, `results`, `leaderboard`, `tutorial`, `loading`,
  `dialogue`, `store`, `inventory`, `saga-map`) and fills its slots with
  the components (`og.ui.settingsTree`, `conversation`, `storeFront`,
  `inventoryGrid`, `sagaMap`, `table`); a HUD is the HUD components
  (`statBar`, `ammoCounter`, `hudTimer`, `resourceBar`, `speedometer`,
  `lapTimer` …); `og.ui.confirm` and `og.ui.dialog` replace the two
  dialogs. `changelog.json` maps each one.
- **Breaking: a refused GPU adapter fails the run.** A native `og run`,
  `og test`, `og render`, `og polish` or `og ui check` whose adapter
  request fails exits 2 with "no GPU adapter: <wgpu's error>; set
  OG_RENDERER=none to run without a renderer", instead of going on as tier
  `none`. The engine draws on a GPU only (docs/design.md §2.6);
  `OG_RENDERER=none` is the only way to run without a renderer, and with it
  a run is what it was. Sandboxes without a GPU set it
  (`docs/platform-asks.md` ask 20).
- **Fixed: the missing-glyph warning without a renderer.** og-ui reads
  each frame's texts against each face's metrics, and `og.render.text` and
  `og.render.floatingText` are read when queued, so the warning is the same
  with or without a renderer, once per face and character, on the tick
  that drew it.
- **Added: `og.motion`, generated motion for humanoids** (docs/design.md
  §2.7): the engine's motion model (a student distilled from NVIDIA
  MotionBricks, og-motion) animates an entity with an og.Skeleton and the
  new component `og.MotionAgent` (`rig`, `drive`, `turnSpeed`, `lookahead`)
  from goals the game queues: `og.motion.pose`, `place`, `face`, `walkTo`,
  `clear`, `status`, `capabilities`, `poses`, with the types
  `MotionGoalKind`, `MotionBackend`, `PoseGoalDesc`, `PlaceGoalDesc`,
  `FaceGoalDesc`, `WalkGoalDesc`, `MotionStatus`, `MotionCapabilities`, and
  the probe's `dump_motion`. The pose is presentation only; goals and the
  `drive` mover are world state (the `og.motion` state block), registered
  only once a game names og.MotionAgent, so other games hash as before.
  The weights (35.4 MB, f16) ship in the package's `motion/`; `og cook`
  writes `cooked/motion/model.ogb` for a project whose code uses og.motion,
  fetched on first use, never at boot. The tarball grows from 1.8 MB to
  34.6 MB.

- **Timeline tracks for cinematics (R10, batch 1).** A `type timeline`
  now eases any reflected field (`track property "lamp" { field
  "og.PointLight.distance" ... }`, a resource's without the entity), an
  entity's light, its drawn visibility (stepped), `og.PostEffects`,
  `og.Blendshapes` weights and a material's parameters, nests other
  timelines (`track timeline { key 3 { play "timelines/door.ogt" speed 2
  } }`, at most four deep, markers named `door/open`), and rolls and
  focuses the camera (`roll`, `focus`, `focusOn`, `focusRange`). Values
  are read from the time, so seeking equals playing. The cook checks the
  fields against the engine's records and refuses nesting cycles; an
  unknown track kind names itself and the engine version. A material
  track is presentation: applied every frame from the time and put back
  at the end, so a restored save draws the right material at once. The
  og.Timeline blob is unchanged (text payload, same version and type
  hash; older timelines cook to the same bytes). Load timelines in setup:
  loading grants og.timeline the records its tracks write.
- **Virtual cameras (R10, batch 2).** `og.VirtualCamera` is a shot:
  its own position or `follow` an entity (`followOffset`, `smoothTime`),
  `lookAt` one, `fov`, `roll`, `focus`, and `collide` (a sphere of
  `radius` kept clear of colliders between the look point and the camera,
  pulled in at once and eased out over `damping`). The resource
  `og.CameraBrain` makes the solo shot (`solo`), else the enabled one with
  the highest priority (ties: the later enabled), live and blends
  og.Camera to it over the incoming shot's `blendTime` by its `blendEase`
  (position lerped, rotation slerped, fov and focus lerped); a change
  mid-blend starts from the blended pose. `og.CameraCut` reports each
  change. All three are registered only when a game names one, so other
  worlds hash as before; the brain's state is record fields, so a restore
  mid-blend continues it. A timeline's camera track still owns og.Camera
  while it plays. The column `Ease` lets records hold og.tween's curves.
- **Dialogue and timeline shots (R10, batch 3).** A dialogue line takes
  `shot <camera>` (a node's covers its lines; `shot none` shows a line
  without one): while it shows, og.story makes that og.VirtualCamera
  `og.CameraBrain.solo` and hands the game's own solo back when no line
  has a shot (or the solo the game set meanwhile). The camera is an
  entity's name, or a cast role: the role's entity's own og.VirtualCamera,
  else the first one that looks at it. `DialogueView.shot` names the
  line's shot. A timeline's `track shot { key 1 { camera "wide" blend 0.5
  ease linear } key 4 { none } }` soloes its cameras from its keys and
  hands the solo back at a `none` key and its end. og.CameraBrain gains
  `soloBlend` (-1: the camera's own blend) and `soloEase`. The story save
  is version 3 (versions 1 and 2 load); the og.Timeline blob is unchanged.
  Load dialogues and timelines with shots in setup: loading registers the
  virtual cameras' records and grants their writes.
- **Two built-in styles: `wasteland` and `foundry`** (packs of the sixteen
  the owner settled, each painted from its approved sheet). `wasteland`:
  rusted riveted plate, duct-tape corners, safety orange, a stencil display
  (Big Shoulders Stencil) over Barlow, stencil icons (survival, crafting,
  extraction). `foundry`: blackened iron in brass rims with rivets, a
  furnace-orange primary, wide-set serif capitals (Spectral SC over
  Spectral), struck-plate icons (city builders, factory, 4X). `ui
  "wasteland"` and `ui "foundry"` cook; `og style new` starts from them for
  their look families. Its choices gain `on_primary` (the primary's label)
  and `display_tracking`, both off for every other seed, which is unchanged.
- **Removed: six built-in styles, folded into the sixteen the owner settled**
  (2026-11-02): `bubbly` into `cartoon`, `arcade` into `pixel`, `paper`
  into `storybook`, `inkwash` into `jade` (a new pack, coming), `neon`
  into `cyber`, `holo` into `scifi`. Their `@styles/<name>/` packs, kit
  looks, playtests and sheets are gone and a look naming one no longer
  cooks. The look presets that named `paper` name `storybook`, those that
  named `neon` name `cyber`. The fonts and icons the seed styles shared
  with paper and neon moved to `@styles/shared/` (`ink-icons.png`,
  `glow-icons.png`, Instrument Sans, Newsreader SemiBold, PT Mono, Chakra
  Petch, Rajdhani). `og style new` still starts from the folded seeds'
  choices for their look families.
- **World cells and sub-scenes (G3-3, batch 1).** An entity's
  `scene "<id>"` line places another scene under it, kept as its own
  blob (not flattened; a cycle is refused naming the chain). With an
  `og.StreamCell` the entity is a world cell: its sub-scene's entities,
  colliders and bodies included, spawn as each tick starts while an
  `og.StreamAnchor` is within `load` of the cell's box, and are despawned
  once every anchor is past `unload` (or while `pin` holds them). The
  decision reads world state only, cells by entity and a sub-scene in its
  text order, so replays, rollbacks and restored saves spawn the same
  entity ids; each change sends `og.CellEvent` in its tick, and every
  entity a cell spawned carries `og.CellMember`. Without anchors the
  camera loads cells, refused in an og.net session. `dump_stream` lists
  the cells. The four records are lazy: a world without cells hashes and
  saves as before.
- **Auto-rigged 2D characters from layered art (G3-1).** A `type rig2d`
  file may name `parts "hero.psd"` (a layered PSD) or `parts "hero/"` (a
  folder of canvas-sized PNG parts) and `template humanoid-side`: the
  cook reads each part's pixels and alpha, puts each joint where the part
  overlaps its parent, builds the template's bones and leg IK, meshes
  the parts that bend (weighted across each joint through a half-way
  joint bone, so elbows and knees bend instead of hinging), and packs
  the parts into an atlas texture (`<rig>#atlas`), into the same rig
  og.Skeleton2D plays. Optional `pivot <part> x y`, `unit` and `filter`.
  No IDL member changes.
- **The auto-rig's clip library (G3-1).** A rig made from layered art
  with `template humanoid-side` carries ten clips: idle, walk, run,
  jump, attack, hit, die, wave, cheer and nod, keyed by hand and scaled
  to the rig's leg length. The feet are placed by the legs' IK (walk and
  run plant them from contact), a held weapon keeps its line, and a
  cape, a tail or hair lags through its chain. `step`, `takeoff`,
  `land`, `hit` and `fall` events mark the moments a game reacts to.
- **The auto-rig's layer contract (G3-1).** docs/art.md §2.8 states the
  layers each template reads, and `@origingame/create-game` exports
  `rigParts(template)` (with `RIG_TEMPLATES`): a template's parts back to
  front, each one's layer name, whether it is required, its side, the
  bone it drives and how its joint is found. Additive; og-cook's tests
  hold it equal to the cook's templates.
- **FBX and OBJ models (G3-1).** The cook takes `.fbx` (binary and ASCII)
  and `.obj` with its `.mtl` as models, read by ufbx into a glTF binary,
  so they cook to the same model and `#anim` blobs and load with
  `og.load.gltf` and `og.load.animation`: meshes in metres with +Y up,
  materials, embedded and beside-the-file textures, skins, morph targets,
  one clip per animation stack, punctual lights. No IDL member changes.
- **Removed: the built-in `minimal` style** (the owner, 2026-11-02). No
  `@styles/minimal/` pack ships and `og style list` no longer names it; a
  look with `ui "minimal"` no longer cooks. `og style new` still starts
  from its quiet light choices when a look matches no seed family.
  Pre-release, so nothing migrates: name another built-in style, or eject
  `minimal` from an earlier engine first and name the game's own copy.
- **Dialogue and quests (G2-8).** A new module, `og.story`, over a new
  crate og-story: dialogues (`type dialogue`) and quests (`type quest`)
  cook into new `og.Dialogue` and `og.Quest` blobs (new type hashes; no
  existing blob changes), validated with each error's line and fix, and
  every key they show checked against the base string table. A
  conversation's view (`og.story.view`) carries the whole line resolved
  through og.i18n, its speaker and choices (`reason` from a key, `visited`,
  checks with their chance, timers); quests list with `og.story.quests`.
  The story is world state (the `og.story` state block, by ids); og.Story
  and the events `og.DialogueShown`, `og.DialogueEnded`,
  `og.DialogueCheck`, `og.QuestUpdated` and `og.StoryEvent` are lazy, so a
  world without a story keeps its hash. Knowledge: skill `og-story`.
- **og.story completed: barks, voice-over, event objectives (G2-8,
  batch 2).** A dialogue's `bark` blocks are ambient lines an entity says
  on its own: a topic, lines in cycle or random order, a `when` that may
  read the world (`Record.field`, `self.Component.field`), priority,
  weight, per-entity and shared cooldowns, `once`. `og.story.bark` asks
  for one, `og.story.ambient` asks every so often (its jitter drawn on the
  story's stream), `og.story.barks` lists those being said and
  `og.BarkSaid` (lazy) announces each. A line's `voice` clip plays through
  og.audio on its own entity, and `DialogueView.voiceLeft` says when it
  ends. A quest objective with `on <Event>` (and `where` over
  `event.<field>`) counts that og event with no game code. `dump_story`
  writes the whole story. `og i18n extract` lists the keys the dialogues
  and quests show and, per locale, the keys a table lacks; `--check` exits
  1 on a gap and `--write` adds the missing keys to the base table. The
  `og.story` state block saves as version 2 (version 1 still loads).
- **Formatting and time groups (G2-6, batch 3).** `og.i18n` formats
  compact numbers, currencies, dates, times and durations
  (`formatCompact`, `formatCurrency`, `formatDate`, `formatTime`,
  `formatDuration`; in messages `{n, number, compact}`,
  `{p, number, currency/EUR}`, `{d, date, short}`, `{t, time}`) from a
  CLDR 49 subset in the engine: og-i18n's 45 plural languages plus
  zh-Hant, 40 currencies, about 61 KB of data. Moments are seconds the
  game passes in, never a clock. `og.TimeGroup` and
  `og.time.setGroupScale` slow or freeze one group of entities (their
  animation, flipbooks, 2D skeletons, tweens, nav agents and particle
  systems) on top of the global scale; physics stays global. Both types
  are lazy, so a world without them hashes and saves as before. Fonts
  now cook the Unicode spaces their files lack (U+00A0, U+2000-U+200A,
  U+202F, U+205F) with the right width, so "3:05 PM" keeps its space;
  every font re-cooks once.
- **Tilemap orientations and run-time maps (G2-6, batch 2).** Maps are
  orthogonal, isometric, staggered or hex (pointy- or flat-top), from a
  `.ogt` `orientation` line or Tiled's isometric, staggered and hexagonal
  maps; picking, centres, `og.tilemap.neighbours`, drawing order and
  colliders (new Collider shapes 7 diamond and 8 hexagon) follow it.
  `og.tilemap.create` makes a map in code and `og.tilemap.resize` grows or
  crops one; both are world state saved in the og.tilemap block's trailer
  only once used. The orientation is an optional blob section: orthogonal
  maps cook byte for byte as before under the same type hash.
  Solid cells off the orthogonal grid merge: each connected region is one
  static collider, its outline (a hollow polyline in 2D, a prism in 3D),
  saved with the world; a layer's `cellColliders` keeps one shape per
  cell. A 256 × 256 hex map, half solid, steps in 0.26 ms instead of
  3.9 ms (release). `SHAPE_TRIMESH` works in 2D as a mesh's outline.
- **Procedural generation (G2-6, batch 1).** `og.procgen`: Perlin and
  simplex noise in 2D and 3D with fractal octaves (`noise`, `noiseGrid`),
  Poisson-disk points (`poissonDisk`), BSP rooms with corridors (`bsp`)
  and wave function collapse with rotations, fixed cells and wrapping
  (`wfc`). Each is a pure function of a desc holding a u32 seed
  (`world.resource(og.Rng)`'s value `.fork("label").seed`), bit-identical
  on both hosts (integer and IEEE f32 arithmetic only; og-core pins every
  generator's output). og.Rng and RngStream gain `weighted` and
  `permutation`; `og.tilemap.fill` writes a generated block of cells,
  re-tiling and rebuilding colliders once. All additive: no type,
  resource or saved value changes, so every world's hash and saves are as
  before.
- **Prefabs (G2-1).** An `.ogt` of `type prefab` holds one entity tree; a
  scene (or another prefab) places it with a `prefab "<id>"` line in an
  entity, whose block then lists only overrides (component blocks for the
  root, `override "<node>" { ... }` for a descendant, `remove "<node>"
  [Component]`, added children). Prefabs nest, `base "<id>"` makes a
  variant, cycles are refused naming the chain. The cook flattens them
  into the scene blob and names a placed descendant `<instance>/<node>`.
  An orphaned override (its node, component or field gone from the
  prefab) is a dev-cook warning, an `og prefab check` failure and a
  publish-cook refusal. Added to the IDL: `og.load.prefab`,
  `og.PrefabLink`, `World.prefabOf`. Added to `og`: `og prefab check`,
  `show`, `extract`. Added to the probe, strictly additive to protocol 5:
  the `apply_prefab` and `revert_prefab` actions and `dump_entity`'s
  optional `prefab`; `save_scene` keeps instances as instances. A world
  that places a prefab hashes differently from the same entities written
  by hand (its `og.PrefabLink`s) and stably across a re-cook; a game
  without prefabs never registers the link and saves and hashes as
  before, so no save migration is needed.

- Added (physics, G2-3): compound colliders, an og.Collider on a child
  entity of an og.RigidBody being part of that body (`RaycastHit.body`
  names it); `og.physics.raycastAll` (`RaycastAllDesc`,
  `MAX_RAYCAST_HITS`) and `og.physics.projectPoint` (`ProjectPointDesc`,
  `PointProjection`). The og.physics state block gains a tagged extension
  trailer, written only when used: a world without compounds saves and
  hashes as before.

- Added (physics, G2-3): sleep. `Physics.sleepTicks` (0, off, by
  default) and `Physics.sleepSpeed`; `RigidBody.canSleep` and
  `RigidBody.sleeping` (written; write false to wake); events
  `og.BodySlept` and `og.BodyWoke`, in entity order. A body sleeps only
  with every dynamic body it touches, counted in ticks and saved with the
  world. og.RigidBody and og.Physics are version 2: saves of 0.7.7
  restore with the new fields at their defaults (sleep off). The raw
  world hash moves once for every engine world (the new layouts are in
  its type table); the portable hash and the golden replays do not.

- Added (physics, G2-3): breaking joints and contact hooks.
  `Joint.breakForce`, `Joint.breakTorque` (0, never, by default) and
  `Joint.broken` (written; write false to mend), with the event
  `og.JointBroken`; the component `og.ContactHook` (`oneWay`,
  `surfaceVelocity`, `friction`, `restitution`, `ignore`), data the solver
  reads, never a callback. og.Joint is version 2: saves of 0.7.7 restore
  with joints that never break. The unit `newtonMetre`.
  og.BodySlept, og.BodyWoke, og.JointBroken and og.ContactHook are
  registered only once a game names them.

- Added (physics, G2-3): a drivetrain car. `Vehicle.model`
  (`VEHICLE_ARCADE`, the default, or `VEHICLE_DRIVETRAIN`), `gears`,
  `reverseRatio`, `finalDrive`, `idleRpm`, `maxRpm`, `torqueCurve`,
  `automatic`, `shiftUpRpm`, `shiftDownRpm`, `clutchTime`, `gear` and
  `rpm` (written); `handbrake`, `handbrakeGrip` and `driftGrip` for both
  models; the component `og.VehicleWheel` (N wheels on child entities, up
  to 16) and `WheelPose.entity`; the unit `rpm`. og.Vehicle is version 2:
  saves of 0.7.7 restore as the arcade car they were.

- Added (physics, G2-3): PBD soft volumes. The component `og.SoftBody`
  (`SOFT_SPHERE`, `SOFT_BOX`, `SOFT_MESH`; edge `stiffness`, `pressure`
  and `volumeStiffness`, `pinTop`, written `volume` and `center`),
  `og.physics.softBodyPoints` and `SoftBodyPoints`. Registered only once a
  game names it. og-ecs gains `SystemContext::query_with`, a query a
  system makes while it runs over types granted to it after it was
  scheduled.

- **A reactive ECS with spatial queries (G2-2).** Added to the IDL:
  `SystemDesc.changed`, `added`, `removed`, `state`, `value`, `when`
  (`StateWhen`) and `runIf` (`RunCondition`); `QueryDesc.changed`,
  `added`, `removed`; `ArchetypeView.rows`; `og.defineHook` (`HookDesc`,
  `HookEvent`, `HookRun`, `Hook`) and `GameDesc.hooks`; `GameDesc.states`
  (`StateDesc`); `World.children`, `removed`, `spatial`, `state`,
  `setState`; `SpatialDesc`, `SpatialIndex`, `Neighbour`. Change ticks
  count system runs (never a clock) and travel in the save as an `og.ecs`
  state block, with the state machines; a world that uses none of this
  writes no such block and saves and hashes as before, so no save
  migration is needed (an older save restores with nothing tracked yet).

- Added (audio, G2-5): `og.AudioBusNode` and `AudioSource.userBus`, a
  bus graph of user buses 4 to 15 under the built-ins; `og.AudioSnapshot`,
  mixer snapshots that blend over `fade` seconds, linear to the sample;
  `AudioBusFx.userBus` and its equaliser (`eqLow`, `eqLowFreq`, `eqMid`,
  `eqMidFreq`, `eqMidQ`, `eqHigh`, `eqHighFreq`), echo (`delay`,
  `delayFeedback`, `delayMix`) and waveshaper (`distortion`,
  `distortionMix`); `og.AudioEvent` and `AudioEventOrder`, sounds chosen
  per trigger from the engine's seeded stream with jitter, a cooldown and
  a voice limit; `og.audio.createSound` and `SoundDesc`, PCM from
  TypeScript; Opus sounds (`sound { codec opus bitrate 96 }`, `.opus`
  sources), decoded beside the tick and streamed on both hosts. The
  `og.Sound` blob is version 3: cooked bundles re-cook.
  `og.AudioSource` and `og.AudioBusFx` are version 2: a save of 0.7.7
  restores with the new fields at their defaults.

- Fixed: a save holds the names given since the last tick began. An
  entity named by `world.spawn` between ticks or by an earlier system
  in the same tick restores with its name; a name given in a system goes
  into the world when the system returns, with its spawn.

- **Saves that survive game and engine updates (G1-5).** Added to the
  IDL: `GameDesc.saveVersion`, `GameDesc.migrations`, `SaveMigration`,
  `World.openSave`, `SaveEdit` (`version`, `types`, `has`, `records`,
  `setRecords`, `transform`, `split`, `renameType`, `renameField`,
  `dropType`, `spawn`, `text`, `close`), `SaveType` and `SnapshotDesc`
  (`world.snapshot(desc)`). Every save carries the game's save version
  (`og-save:2:`; `og-save:1:` saves read as version 1), and a restore
  runs the game's migrations from it to the current version on the save,
  refusing with the step's reason and the world untouched when one
  fails. The engine migrates its own state in an older build's save
  first (og.ui's block 9 → 11: a 0.7.7 save restores without `migrate`).
  A selection saves and restores only tagged entities and chosen
  resources. The slot cap stays 512 KiB (its two banks are the
  platform's 1 MiB a player): a 5,000-entity world saves to 437 KiB.

- Fixed: `World.restore` takes a save without `migrate` when an engine
  type (an og.* component, resource or event) only gained fields since
  the save: each added field gets its declared default (not zero), so an
  engine field added in an upgrade no longer refuses every player's save.
  A removed, renamed or retyped engine field, and any change to a game's
  own types, is still refused naming the fields.

- Fixed: `World.restore` compares a changed type's layout only when the
  save holds data of it (a component row, a resource value, an event); a
  type the save holds none of takes this build's layout, so a field added
  to a type an old save never used no longer refuses it without `migrate`.

- **Gameplay actions, rebinding and gamepads (G1-2).** Added to the
  IDL: `og.input.defineAction`, `actions`, `action`, `actionDown`,
  `actionPressed`, `actionReleased`, `actionHeld`, `actionValue`,
  `actionVector`, `setContexts`, `contexts`, `rebind`, `conflicts`,
  `resetBindings`, `setToggle`, `saveBindings`, `loadBindings`,
  `startCapture`, `capture`, `cancelCapture`, `applyCapture`, `glyph`,
  `glyphs`, `controlGlyph`, `device`, `padDevice`, `setDeadzones` and
  `rumble`, with `ActionDesc`, `InputBinding`, `ActionState`,
  `ActionInfo`, `RebindDesc`, `RebindAnswer`, `BindingConflict`,
  `ConflictPolicy`, `CaptureDesc`, `CaptureAccept`, `CaptureState`,
  `InputGlyph`, `InputDevice`, `ActionKind`, `TouchZone`,
  `DeadzoneDesc`, `RumbleDesc`, `UiActionDesc.action` and
  `GamepadInput.device`. Actions bind keys, the mouse, pad buttons,
  sticks, triggers and touch (WASD as one 2D axis), in contexts, with
  sensitivity and dead zones; the player rebinds them with conflicts
  reported and keeps them as text; a capture serves a controls screen;
  prompts read the binding's glyph on the device used last. They are read
  before the game's systems and are world state, so they replay.
- **og.Gamepad is version 2: eight pads** (`MAX_GAMEPADS` 8, was 4) and
  each pad's family (`devices`). The web shell maps pads whose mapping is
  not "standard" through a table instead of dropping them, and
  `og.input.rumble` shakes a pad. A save from a four-pad build restores
  without `migrate`: the input a restore keeps from the host (og.Input,
  og.Pointers, og.Gamepad, og.PointerLock, og.Audio) is mapped field by
  field when its layout changed.
- Simulation: every world carries the action map's state block
  (`og.input.actions`) and og.Gamepad's new layout, so world hashes move
  (a game's own recorded hashes need recording again); what a game
  computes does not change.
- Fixed: keys held when the window loses focus or the page is hidden are
  released, and pads come to rest with the keys they map to.

- **Rendering interpolates between ticks (docs/design.md §2.4).** A
  behaviour change of the realtime display, released in the next minor
  (the release's breaking entry): a display frame between two ticks (a
  60 Hz tick on a 120 or 144 Hz screen) draws every entity's world
  transform and og.Camera's pose blended from the tick before the last to
  the last by the accumulator's fraction, so the picture is smooth and up
  to a tick late. Skinned models blend each joint (translation and scale
  lerped, rotation slerped) and their blendshape weights, with root motion
  and every pose the simulation reads still at the tick; UI anchored to
  an entity stays on its drawn place; object motion blur moves from the
  frame drawn before, never across a snap or a camera cut, and a camera
  cut (`RenderScene.camera_cut`) restarts TAA's history; the probe's
  `teleport` snaps. Immediate-mode sprites, text and lines
  (`og.render.sprite`, `og.render.text`, `og.debug`) and shader time stay
  at the tick. Frames on
  tick boundaries (og run without --realtime, og render, og test), the
  probe's screenshots and captures draw the tick as before, so no
  reference frame moves, and the simulation, its hash and replays are
  unchanged. On by default: a game that teleports an entity calls
  `og.render.snap` on that tick, or it glides there over one tick unless
  it jumped past `RenderSettings.snapDistance`. Added to the IDL:
  `RenderSettings.interpolate` (true) and `snapDistance` (10 m), the
  component `og.Interpolation { off }`, `og.render.snap(world, entity)`
  and `og.render.snapCamera(world)`; `dump_render` reports the frame's
  `interpolation`.
- Field factories `VectorArt()` and `UiMaterial()` for the two handle
  types U1 added without one (og.load.vector, og.load.uiMaterial): every
  handle type of the API has its factory again, as
  `packages/engine/tests/api-surface.test.ts` holds.
- `og test --jobs N` runs up to N playtests at once (default 1, as
  before); each one's output is printed whole, in the playtests' order.
- **UI styles are systems (docs/ui.md U4b).** Widgets draw as style
  parts (`Button.base[primary][pressed]`, `Slot.frame[rarity=epic]`,
  `Meter.fill[kind=mana]`, ...) from the style's most specific `dress`
  blocks with the UI pass's paints, strokes, effects and materials; a
  part a style does not dress draws from its tokens (shape language per
  element class, elevation, strokes), and every state moves on the
  style's `spring ui`. Widget frames move (the built-in packs' sheets,
  og-ui's references); an existing game draws in its style as before
  otherwise. Added to the IDL: `ElementDesc.part` and `partClass`,
  `UiTheme.tokens` (the style's tokens as JSON), `og.ui.feedback(event,
  rise)` and `og.ui.setHaptics(intensity)`. Each UI event plays a sound
  of its set (never the same twice running) at a volume and pitch from
  its ranges, and its haptic pattern reaches the pad's rumble and the
  phone's vibration on both hosts, scaled or silenced by the player.
- Added: `og style new <name> --look <family> --premise <text>` (and
  `--seed`, `--scheme`, `--shape`, `--ornament`, `--motion`, `--finish`)
  generates a version 2 style with its sounds and the art slots it asks
  for (`art-requests.json`); thirteen generated seed styles are built in,
  one per look family: `gilded`, `storybook`, `tactical`, `holo`,
  `cyber`, `bubbly`, `kawaii`, `arcade`, `craft`, `minimal`, `dread`,
  `broadcast`, `inkwash` (`og style list`). The cook bundles the packs'
  fonts and icons a style names.
- Added: `og ui slots [project] [--json]` lists every art slot of a game
  with its spec; an art slot without a texture draws the style's
  placeholder for its kind. `styleArtBriefs(look, requests)` in the art
  brief (`@origingame/create-game`) turns a generated style's
  art requests into briefs (an addition: `artBrief`, `readLook` and
  `presetLook` are unchanged).
- The archetypes' and examples' styles are version 2 (`og style
  upgrade`); version 1 still loads.
- **Breaking: WebGL2 is removed; the web is WebGPU only** (the owner,
  2026-11-01, design §8). A browser without WebGPU gets the player
  shell's "This browser does not support WebGPU" card (reported to the
  Player as a non-retryable `engine_start` failure) instead of a degraded
  engine. Removed: the `webgl2` variant of `RenderTier`; the shell's
  `webgl` option and the play page's `webgl=1` parameter; `og run --device
  --webgl` and every browser test runner's `--webgl` tier; og-render's
  `Backend::WebGl2`, `Tier` and `RenderInfo.tier`; the vendored
  `third_party/wgpu-hal` (its patches were GLES-only). With a renderer,
  `RenderCapabilities.compute` and `ssr` are always true,
  `maxShadowedLights` is 8 and `volumetricFog` is `froxel` unless the
  game asked for `raymarch`. The `auto` defaults now always mean TAA,
  cube point shadows, GTAO and the froxel grid; MSAA, FXAA, paraboloid
  shadows, SSAO, the raymarch, CPU culling and CPU particles stay as
  choices on every device. A game that set none of these draws as it did
  on WebGPU.
- Fixed: `world.resource(og.Input)` carries every field the IDL declares.
  It held only `pointer`, `pointerInside` and `wheel`, so `buttons`,
  `buttonsPressed`, `buttonsReleased`, `buttonsReleaseNext` and the key
  words (`keys0..3`, `pressed0..3`, `released0..3`, `releaseNext0..3`)
  read `undefined` on both hosts; og.Input is now read through og-input's
  registered layout like og.Pointers, all fields read only.
- Entity arguments say what went wrong: a value that is not an entity
  number (`null` from `world.find`, `undefined`, a Result) is thrown naming
  the call and parameter, alike on both hosts (`world.set: entity is null
  (world.find returns null when no entity has the name)`), a non-number
  `SpawnDesc.parent` is refused instead of ignored, a dead handle is refused
  as `world.despawn: no live entity has the handle N` (for a bare index with
  `Archetype.entities holds (index, generation) pairs; use a.entity(i) for a
  handle`, logged once per system and call), and `world.spawn` in a system
  that did not declare `og.Parent` or `og.Transform` says spawn wrote it.
- The wasm download budget is 16 MB brotli (was 4 MB): a ceiling against
  accidental growth, not a reason to cut a feature (design §7).
- Added: `og style eject <slug> [<name>] [--project <dir>] [--force]` writes
  a built-in UI style (style.ogt, atlas, icons, fonts with their import
  settings, licences) byte for byte into the game's `assets/styles/<name>/`
  to change at will, never over existing files without `--force`, and
  prints the look's `ui "styles/<name>/style.ogt"` line; `og style list`
  names the built-ins. The og-ui and og-taste skills use it instead of
  copying `templates/styles/`, which the installed package lacks.
- `og api show <module>` (`og api show input`) prints the module's overview
  from its wiki page; a symbol's name still prints its entry.
- The og-game skill is a 40 KB entry point; its chapters moved into
  og-render, og-physics, og-2d, og-anim and og-net (new) and og-build,
  og-model, og-taste. Skills point at `og api show <module>`; the API wiki
  is no longer mounted under og-game.
- Added (docs/ui.md U3): UI documents, `type screen` and `type component`
  `.ogt` cooked to `og.UiDocument` (grammar in crates/og-ui/README.md);
  `og.load.document`; module ui's `screen`, `component`, `openSlot`,
  `push`, `pop`, `replace`, `clear`, `layers`, `screenVm`, `docEvents`,
  `gameInput`, `paused`, `defineAction`, `action`, `actionHeld`, `glyph`,
  `device`, `setDevice`, `actionBar`, `documentSchema`, `setProp`,
  `saveEdits`, with the types `UiDocEvent`, `UiDocResponse`, `UiLayer`,
  `UiActionDesc`, `UiActionTrigger`, `UiGlyph`, `UiDevice`,
  `UiActionHint`. The probe (protocol 5, optional fields only): elements
  gain `path`, `layer`, `source`, `bindings`, `visualStates`, `edits`,
  `motion` and `slot`; `dump_ui` gains `layers`, `motion`, `actions`,
  `documents` and `edits`; new actions `dump_ui:layers` and `ui:<verb>`
  (set, state, play, seek, push, pop, save, clear, action). A UI snapshot
  is version 11 (it carries the layer stack).
- **Decals with normal maps, and no 32 cap (R7, batch 1).** `og.Decal`
  gains `normalMin`, `normalMax` and `normalScale`: a region of
  `RenderSettings.decalAtlas` holding a tangent-space normal map (glTF's
  convention; an sRGB atlas is read back to its stored values) that bends
  the surface's normal under the decal, after the surface's own normal
  map. A frame now draws up to 65,536 decals: their records are a storage
  buffer and they are binned into the light clusters' grid, so a pixel
  walks only the decals whose box can reach it; at most 255 overlap in one
  cluster (the newest kept, the rest counted in `decals_dropped`). Later
  decals still blend over earlier ones. Existing decals draw as before.
- **Up to 64 blendshapes (R9, batch 3).** A mesh keeps up to 64 morph
  targets (eight before), their deltas packed sparsely per vertex.
  `og.Blendshapes` (version 2) keeps `w0`…`w7` and adds `influences`, the
  weights of targets 9…64; the state block writes them only when one is
  non-zero. The weights ride in the joint palette buffer after the joints,
  and the previous frame's weights in its previous section, so the
  skinned prepass's motion vectors follow a morphing face under TAA. A
  save of version 1 restores through the migrating restore.
- **Mesh, lit and distorting particles; sub-emitters and particle events
  (R7, batch 2).** An emitter file takes `render mesh <box | sphere |
  tetrahedron | "model id">` (each particle a small shape that tumbles
  with its rotation), `lit true` (billboards and meshes shaded by the sun,
  the sky and local lights), `distortion <0..1>` (billboards that bend the
  scene behind them: heat haze, shock rings), `subEmitter <birth | death |
  collision> "<emitter id>" <count> [<inherit velocity>]` (up to four, one
  level deep) and `events <birth | death | collision>...`. New
  `og.ParticleEvent` (entity, kind, index, position, velocity) and
  `ParticleEventKind`: once a game names the event, systems whose emitter
  has an `events` line are simulated in the tick by the og.particles
  system (engine randomness, fixed math, carried by snapshots) and drawn
  as simulated, so gameplay reads exactly the particles shown. A
  collision event is an impact faster than 0.5 m/s. The cook refuses an
  emitter whose mesh model or sub-emitter is missing, or that the renderer
  cannot draw (distortion on a mesh, a fifth sub-emitter).
- **Cloth on skinned garments (R9, batch 4).** `og.Garment` (lazy, 3D)
  on a posed model simulates one of its glTF meshes as a garment: each
  tick, after the animation, the runtime skins the mesh's welded points
  and og.physics steps them in the deterministic tick (gravity, gusting
  wind, edges and bends, each point within `maxDistance` of the skin by
  its vertex colour's red, `pinTop`, `backstop`, push-out of solids at
  `radius`), saved in the og.physics block. The renderer draws them as
  the skinned instance's own vertices (`SkinnedInstance::deformed`, in
  the joint palette buffer, with the previous frame's for TAA's motion
  vectors). `og.physics.garmentPoints` reads them.
- **GPU crowds (R9, batch 5).** `og.CrowdAnimation` (lazy) on a model's
  entity plays a clip on the GPU: the runtime bakes the clip once per
  skeleton, skin, clip and `rate` into joint palettes
  (`RenderScene::baked_clips`, shared by every entity that plays it), and
  each entity is a skinned instance at its own frame (`offset` + `speed` ×
  the world's time), blended between baked frames by the shader. No
  og-anim work per entity. The previous frame's clip, frame and transform
  give TAA its motion vectors. Drawing only, CPU frustum culling as other
  skinned instances; not GPU-culled or on the meshlet path yet.

## 0.7.7 — 2026-10-31

A patch with the simulation unchanged (the golden replays): IDL members
added off by default (mouse-look), none removed.

- `og cook` takes `.mp3`, `.ogg` (Vorbis) and `.flac` sounds beside `.wav`:
  decoded to 16-bit PCM in the cook (pure Rust, symphonia; no ffmpeg) and
  encoded to QOA like a WAV, chunked and streamed the same way, under the
  source's own asset id (`audio/theme.mp3`, `audio/theme.mp3#chunk3`). An
  undecodable file is refused with its name and format.
- Mouse-look: `og.Pointers.movement` (the mouse's relative movement this
  tick, CSS px: pointer 0's `movementX`/`movementY` summed, zero next tick)
  and `og.PointerLock` (`wanted`, which the game writes; `locked`, read
  only, the host's). The web shell asks for the lock on the next press on
  the canvas while `wanted`, exits it when not, reports
  `pointerlockchange` back (Escape releases it) and holds the absolute
  positions while locked. `PointerInput` and `og.PointerMove` gain
  `movementX`/`movementY` (0 when omitted), `InjectedInput` the
  `pointerLock` case, the command channel `mouse_move:dx,dy` and
  `pointer_lock:on|off`; native runs are headless and never lock.
  og.Pointers and og.PointerMove have new layouts, so a save
  restores with `migrate` (both are session state, which a restore keeps
  anyway); the simulation is unchanged (the golden replays).

- Fixed: `og` carries each built-in CJK font once (the look presets and
  style assets are statics), about 9.7 MB less per platform.
- Fixed: the og-core and og-game skills say the cited `examples/*` games
  are in the GitHub repository, not the installed package.

## 0.7.6 — 2026-10-31

A patch with the simulation unchanged (the golden replays): no IDL member
added or removed.

- `og api find <words>` and `og api show <name>` answer one API question at
  a time from the installed engine's wiki: find lists the matching entries
  (name, module, first sentence, then field and parameter docs) one line
  each, show prints one entry in full without its IDL source. og-core §7
  and og-game §25 teach them, so an agent looks an API up instead of
  reading a module's whole page (render.md is 440 KB).

## 0.7.5 — 2026-10-31

A patch with the simulation unchanged (the golden replays): no IDL member
added or removed (`AnchorDesc.margin`'s doc now says it insets every edge).
Games on a built-in style look different, as below.

- UI styles speak Chinese in their own voice: a style's `fonts` block takes
  `cjk` and `cjkDisplay`, Simplified Chinese companions that set a label's
  Chinese beside the role's Latin face, ahead of the locale's fonts. Every
  built-in style names one matched to its Latin (templates/styles/cjk:
  Noto Sans SC, Noto Serif SC, ZCOOL KuaiLe, Fusion Pixel; OFL), cooked in
  only for a game with a `zh` string table, rendered at run time.
- Button labels read on what is drawn: a style's `tone <part> r g b a` (the
  frame art's centre; `og cook` measures it from the atlas when a style
  does not set it) is what a button's label ink, and the ground any widget
  reads through it, are judged against, at 7:1. A game's `primaryColor`
  over a dark neon or scifi body keeps a light label, and a mid-scale
  colour (crimson) moves away from its ink as little as it takes. The
  accent `highlight` now follows the game's primary and reads 5:1 on its
  wash.
- A centred or end-aligned label in a column takes the column's width (it
  sat at the start in a box as wide as its text).
- `og.ui:` warns when a control is laid out past the screen's edge with no
  scroll container to bring it back, naming it and by how much.
- The neon style: its glow marks the button and the title only (no panel,
  HUD or number glow; tighter tube halos, a fainter grid), and its numbers
  are Chakra Petch SemiBold (Orbitron read 0 as Ø and 1 as 7).

## 0.7.4 — 2026-10-30

A patch with the simulation unchanged (the golden replays): no IDL member
changed, and without `ORIGIN_FRAMES_SOCKET` a game runs and draws as before.

- Native host, shared frames: with `ORIGIN_FRAMES_SOCKET` set (Origin
  Runner sets it when Origin Desktop draws the Session's engine), `og run`
  and `og dev` show their frames in the desktop window through shared GPU
  memory (IOSurfaces on macOS, Vulkan opaque fds on Linux, D3D12 shared
  handles on Windows; og-render's `shared_frames`, the desktop's
  `shared-frame` protocol) and render at the size the window asks for.
  Without the variable nothing changes; no IDL member changed.

## 0.7.3 — 2026-10-29

A knowledge-only patch with the simulation unchanged (the golden replays);
no IDL member and no crate changed.

- Knowledge, level-texturing: `texture-level.ts` paints with
  `gemini-nano-banana-2.1` by default (it was `qwen-image-2.1-pro`, still
  one `--model` away). On the village it matched Qwen's 34 calls and
  70.6 % coverage, and its fill views return the painted part nearly
  unchanged (mean pixel difference about 5 against 15–35). The `http`
  provider sends only the fields a model's route accepts: Qwen's `seed`
  and `prompt_extend`, otherwise the view's `size` (Gemini's route refused
  the old request with "does not support seed"); `dashscope` refuses a
  model it does not serve. A resumed run keeps the model it started with.

## 0.7.2 — 2026-10-29

A patch with the simulation unchanged (the golden replays): the IDL
members below are new and off by default, so a game that does not set them
draws and runs as before.

- render, `MaterialStyle.shadowDarkness` (new, off at 0): where the sun
  does not reach (the toon ramp's dark band, the side turned away, a cast
  shadow) a lit surface loses that share of the sun's light and of the
  ambient light (the environment map and the hemisphere light), so 1 draws
  every shadow as a solid black mass; point, spot and rect lights, the
  emissive and `shadowTint` still show inside it. `RenderCapabilities.stylise`
  lists `shadowDarkness`.
- looks, `heavy-ink-shadows`: shadows are solid black ink. Its
  `shadowTint "#000000"` at `shadowTintAmount 0` did nothing, and
  `posterize 3` never reached black because the environment's ambient
  light lifts every shadowed surface above the lowest step. The preset
  now sets `shadowDarkness 1`, six posterize steps, a near-black sky, a
  dark red fog and muted grading. Its frames change.
- looks, `low-resolution-3d`: a short dark fog (density 0.07) the colour
  of a matching flat sky, the clear colour the catalogue asks for, so the
  draw distance ends in it as on early consoles. Its `vertexSnap` and
  `affineTexturing` were already drawn (`retro_clip`, `stylised_uv`).
  Its frames change.
- cook, the mesh conformance: under a look with `Stylise` paint fields
  (and an art contract that conforms models) a model's own colour
  textures are painted in texel space as painted 2D sprites are (the
  Kuwahara along edges, bleed, pigment pooling, the canvas grain), after
  their `ArtTexels` resample, so the strokes stay on the surface. A
  texture no model claims, normal maps and data maps are not painted.
  Mesh textures under such a look re-cook (`paint 1` in their desc);
  painterly-3d's, sculpted-clay's and paper-craft's sample frames change.
- cook, the mesh conformance: a model that says `conform character` (at
  the top of its `type model` text, or in a glTF's import settings) is
  held to `ArtShape.characterFaces`, as a skinned one is, and `og style
  check` judges it the same way; a rigid or generator-built hero used to
  be held to `propFaces`. A `type model` past its face budget is built
  again with fewer segments (every generator's counts scaled by one
  factor, held at a sphere's 5 × 3 and a cylinder's, cone's or lathe's 5)
  before anything is decimated, and the report says `built at N % of its
  segments`. A conformed `type model` past its budget re-cooks; no IDL
  member changed.
- render, IDL: `MaterialStyle.surface` (`MaterialSurface`: none, clay,
  paper, plastic), `surfaceStrength` (default 1) and `surfaceScale` (m, 0
  the surface's own): a made surface on every lit standard material,
  drawn by the lit shader in each mesh's own space (normal detail by its
  screen-space surface gradient, colour grain, an energy-conserving
  wrapped light, roughness; clay and plastic take the base texture's
  coarse mip for its detail), fading where the pattern spans under four
  pixels. Unlit, `styled false` and physical materials keep their own.
  None by default: no frame moves without a look that sets it. Presets:
  `sculpted-clay` (surface clay, rim 0.1, bevel 0.05), `paper-craft`
  (surface paper, no posterize), `miniature-diorama` (surface plastic,
  bevel 0.05); their frames change. Where the shading normal lies in the
  face (a smooth mesh's silhouette, a rim turning away) the surface's bump
  fades out and keeps the normal: it had normalised a zero vector there,
  and the NaN spread through the post effects into black blots.
  `examples/rumpus-isle`'s look, which extends miniature-diorama, sets
  `surface none` to keep its own painted colours.

## 0.7.1 — 2026-10-28

A patch with the simulation unchanged (the golden replays); no IDL member
changed.

- cook, the mesh conformance (`ArtShape.normals flat`): faces thinner than
  the position grid are dropped, and flat normals hold on lathe poles and
  on voxelised parts turned by their placement (normals brought back
  through the placement's transpose). Conformed models re-cook (model desc
  `conform-mesh-v3`); voxel-worlds' turned parts shade differently.
- render, `RenderScene::set_shader_param`: a changed parameter is written
  into the material's uniform in place (`queue.write_buffer`) and keeps
  its bind group, where it used to rebuild both. On the web each new
  buffer and bind group cost about 1 ms in the browser's GPU process: the
  Grand View Garden's ~500 materials updated 15 times a second drew at
  1.5–2.5 fps in Chrome, 22–27 fps with this. Frames are unchanged; no
  IDL member changed.
- render and cook, `dither bayer4`/`bayer8` (`Stylise.dither`,
  `ArtPalette.dither`): the threshold matrix is the dispersed Bayer matrix
  (4 × 4: 0 8 2 10 / 12 4 14 6 / ...). It used to take the position's
  coarsest bit as the threshold's most significant, a quadrant-clustered
  matrix that drew a tone as blocks of the matrix's size (a 1-bit frame at
  240 lines turned into a 24 px checkerboard that swallowed every form).
  Every frame dithered with `bayer4` or `bayer8`, and every texture the
  cook conforms with them, changes; `bayer2` does not. No IDL member
  changed.
- looks: `rubber-hose-animation` (`posterize 4`) and `gothic-contour`
  (`posterize 5`) drop `Stylise.posterize`. Posterize rounds each channel
  on its own in display space, so a near-neutral colour whose channels
  sit closer than a step apart lands on unequal steps: the sepia print
  turned olive and pink, the blue-grey haze flat grey and violet (still
  visible at 8 tones). The cel bands stay the toon material's, the
  contours the ink pass's. Frames of games on these presets change.

## 0.7.0 — 2026-10-28

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
  At run time such a look also sets the camera (`ArtCamera`), the sprite
  outline (`ArtLine`), cleanEdge rotation and the animation step
  (`ArtMotion.stepFps`, which changes the ticks of a game whose
  animation drives play).
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
- `og style check [project]` (stream A6, docs/art.md §2.5): cooks, then
  measures every cooked asset against the look's art contract (colours
  and `maxColors`, the palette hit rate within ΔE00 2 where `strict`, the
  pixel grid and its half pixels, the line found, `maxSize`, `filter`,
  `mips`, faces against `propFaces`/`characterFaces`, the flat-normal
  share, texel density and its spread), lists every opt-out, and measures
  the title, play and results frames (project.ogt's new `screens` block
  or `--title`/`--play`/`--results`, captured by `og run --headless
  --commands`: colours, palette hit rate, edges). `--critic` prints only
  the catalogue's three checks and the frames' paths, `--json` one
  document; exit 0 within the contract or with no look, 3 off it.
  `og doctor --project` has a `style` line, and `og build --publish`
  prints the summary as warnings, never refusing a build.
- `og.Sprite.outline` (stream A4): a pixel outline this many logical
  pixels wide (0 to 4), drawn by the sprite shader from the texture's
  alpha; 0 is none, or, under the project's art contract, the look's
  `ArtLine.width` for a texture the cook did not line.
- `og.Sprite.outlineColor`: the outline's linear colour; at its default,
  the contract's `ArtLine.color`.
- `og.Sprite.palette`: a palette swap; the sprite draws its texture's
  `<id>#index` through a row of this palette texture of N rows; 0 is none.
- `og.Sprite.paletteRow`: the row of `Sprite.palette` drawn.
- `og.Tilemap.palette`: a palette swap of every tile layer; 0 is none.
- `og.Tilemap.paletteRow`: the row of `Tilemap.palette` drawn.
- Art contract at run time (stream A4, docs/art.md §2.4), only under
  project.ogt's `look`: `ArtPixels.rotation cleanEdge` draws pixel-art
  textures through the new cleanEdge sampler, `ArtLine` defaults
  `Sprite.outline`, `ArtCamera` sets the camera's projection, angle and
  field of view when a look is applied (`pixelPerfect` snaps a 2D camera
  by logical pixels), `ArtMotion.stepFps` defaults
  `AnimationPlayer.stepFps`, and a strict `ArtPalette` tints the UI's
  icons to the nearest palette colour. A game without the line draws as
  before; one with it changes frames (the release's breaking entry).

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
