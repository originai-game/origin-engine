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

- **2D destructible pixel terrain.** og.PixelTerrain is ground of cells
  (Worms, Noita): og.destruction.carve2d digs, fill2d builds (ground or
  bedrock), explode2d blows a hole and throws the ring round it as Voronoi
  pieces, fracture2d breaks a piece or a terrain into Voronoi pieces, and
  cell2d reads a cell. Each chunk collides as its smoothed outline (a
  static SHAPE_TRIMESH collider on a generated mesh, rebuilt only where a
  change touched it); ground no longer joined to bedrock or the bottom row
  falls as an og.PixelPiece (a dynamic convex body), reported by
  og.PixelDetached (cause CAUSE_CARVE, CAUSE_EXPLOSION or CAUSE_FRACTURE).
  The cells, pieces and queued changes are world state (snapshots,
  restores, replays, the world hash); the drawn texture is remade from the
  cells on a frame after they change. The records are lazy: a world that
  never names them saves and hashes as before.
- **Emitter expressiveness.** `type particles` emitters gain curves over
  life for the speed (`speedOverLife`) and the spin (`spinOverLife`), a
  curve over each cycle for the rate (`rateOverTime`), up to four
  attractors (`attractor`), a vector field (`field curl ...`, or
  `field "<texture>" ...` that the cook bakes into a grid), emission
  along a Catmull-Rom spline (`shape spline ... [closed]`) and a share of
  the emitter's own velocity (`inheritVelocity`). Compute and the CPU
  path agree on every one; emitters without them simulate as before.
- **Flipbooks from animated images.** A `type flipbook` sheet names
  animated GIF, WebP or APNG files with `source "hero_run.gif" { ... }`,
  a clip per file, instead of a texture. The cook keeps the delays as clip
  durations, keys a flat background (`key auto`), aligns the frames
  (`align feet|bbox|centroid|none`), resamples (`scale`), crops them to one
  size with the pivot at the anchor, shares identical frames and packs
  them into `<sheet>#sheet`: the canonical sheet an Aseprite atlas makes.
  `og sprite inspect` measures a source or a built sheet: delays, drawn
  bounds and centroid drift, duplicates, the loop seam and the edge
  fringe, numbers only.
- **Sprite material shaders.** `og.Sprite.material` draws a sprite
  through a material shader's hooks (vertex_offset, surface_color,
  surface_emissive) with the mesh's shader assets and parameters, so a
  dissolve, freeze, shimmer or palette effect is a shader. Shaders of the
  new `mode sprite` run on sprites only and may read the texture anywhere
  with `sprite_texel(uv)`. dump_render reports `spriteShaderErrors`.
- **Animation graphs on 2D.** `og.anim.graph({ sprite: true, ... })`
  makes a graph of flipbook or rig2d clips; an og.AnimationGraph beside
  an og.Flipbook cuts between states on frame boundaries, and beside an
  og.Skeleton2D cross-fades and blends two clips (new
  `Skeleton2D.blendClip`, `blendTime`, `blend`). Deterministic, in the
  world snapshot.
- **Breaking: the engine ships no skills.** The 25 skill packs under
  `packages/knowledge/skills/` (`og-core`, `og-game`, `og-build`, `og-ui`,
  `og-taste` and the rest) and `packages/knowledge/guides/` are deleted,
  with their `skill:<name>` pack names and the knowledge mount of
  `game/skills/`. The mount writes only `game-scaffold/`; `payload`,
  `defaultRoots`, `installed`, `mount`, `summary`, `artBrief`, `readLook`
  and `presetLook` keep their signatures. An agent reads the engine through
  `og`: `og api find`/`show` over the API and the engine's guides.
- **Breaking: the UI packs carry no skills.** The kits' `building-*-ui`,
  the style packs' `styling-*` and `materials/using-ui-materials` folders
  are deleted; `extension.json` `skill` is optional. Read
  `og api show kits`, `ui-materials` and `screens` instead. Style-pack
  concept images now sit in each pack's `concepts/` folder.
- **IDL `aliases`.** One line of synonyms and other engines' names on a
  declaration, printed in the wiki as "Also known as:" and scored by
  `og api find` as its own field. 94 entries carry them. Recall on the
  160-query set goes from 100 to 132 at @5; a 40-query held-out set
  (`api-find-heldout.tsv`) stays at 21 at @5.
- **`og api find` adds static embeddings**: potion-base-8M (model2vec,
  MIT) at 128 dimensions in int8, fused with BM25F by reciprocal rank. Each
  entry's vector ships in `wiki/search.tsv`; the token table
  (`crates/og-api/search/words.bin`) is compiled into `og`. On the 160-query
  evaluation set recall@1 goes from 46 to 53 and recall@5 from 90 to 100
  over BM25F alone. `--json` adds `bm25` and `cosine`.
- **`og api find` ranks with BM25F**, from an index `og api gen` ships as
  `wiki/search.tsv`, over each entry's name, summary, member names and
  body, with camelCase and dot splitting, light stemming and one- or
  two-edit typo tolerance. On the 96-query evaluation set
  (`crates/og-cli/tests/fixtures/api-find-queries.tsv`) recall@1 goes from
  18 to 30 and recall@5 from 38 to 56. `--json` adds scores; `og api show
  <module>` lists the module's members. The wiki is split into a module
  page with a contents table plus one page per function, record, interface
  and type under `wiki/<module>/` (ui.md 1.1 MB → 116 KB).
- **The engine's guides are entries of `og api`.** `crates/og-api/guides/`
  holds `start`, `testing`, `determinism`, `looks`, `models`, `screens`,
  `animation`, `art-review` and `three-to-og`, rendered into the wiki as
  `guide-<name>.md`; `og api show <guide>` prints one and `og api find`
  returns their sections (kind `guide`) beside the API. The IDL's module
  `skill` key is removed.
- **Breaking: no gameplay templates or UI gallery.** All gameplay starter
  trees, gallery scenes/art/frames and template-only checks are removed.
  Core feature tests use small fixtures; reusable kits, styles, looks and
  locales remain. The distribution's `ui-sheet/` is a neutral control sheet,
  not the gallery. Knowledge mount signatures stay unchanged, but starter
  assets and gameplay paths are absent from the payload. Generated evidence
  is no longer committed. The template replay and standalone wasm-determinism
  tools are removed; core replay and native/wasm hash tests remain.
- **Jumps the simulation owns.** `og.motion.jump` (`JumpGoalDesc`:
  `height`, `distance`, `heading`, `prepare`, `recover`, `pose`) queues a
  jump: after its preparation the og.motion system takes off in the tick,
  through an og.CharacterController's own jump (its `jumpSpeed` set for the
  height, held on the takeoff tick) or, without a controller, a fixed
  ballistic arc back to the takeoff height, and the goal ends `recover`
  seconds after the landing tick. `MotionGoalKind` gains `jump` and
  `MotionStatus` gains `airborne`. The motion model never decides takeoff,
  height or landing; its jump is drawn in place on the simulation's flight,
  retimed to the landing each tick predicts from the arc and the ground
  along it (a ledge up lands early, a drop late). The air pose is the
  model's: no lean, lean-rate or leg bound is applied to any drawn frame.
  `dump_motion` reports the drawn feet's distance ahead of the hips
  (`feetAheadM`).
- **A retrained motion model.** og.motion ships the retrain from run3
  (og-motion-train `weights/geo8`, sha256 `ee488c6d…`): beside the jump
  curriculum it distils rollouts the student drives itself and the decoded
  feet where the teacher's are planted. Closed loop it reaches its goals as
  well as before with less skate (9.11 cm, 1.19 cm; was 9.13, 1.20); held-out
  jumps lean 9° at most and drift 12 cm (was 14° and 28 cm); on the jump
  review planted feet slide a third as much (0.35 mm mean). The library's
  `jump` pose is a standing jump's tucked apex, upright with the feet under
  the hips. Same size, same API; drawn motion changes.
- **Fixed: a running jump no longer dives.** The model's goals now keep the
  simulation's time and place: the rest of a walk, run or pose the model
  is already in merges into the next goal when less of it is left than the
  model's shortest generation, instead of costing a whole generation behind
  the simulation. A running jump's goal had started 3.3 m behind its apex,
  and the model lunged toward horizontal.
- **Fixed: planted feet outrank `og.motion.react`.** The reaction's clip was
  blended over the feet the planter had placed, legs and hips included, so a
  reaction that lowered or turned the hips slid planted feet (33 cm in the
  regression test). The feet are now planted after the reaction, the look and
  the hand contact; a foot the reaction's own clip steps follows the clip,
  and a reaction turned more than 30° from the facing steps both.
- **Fixed: a world restored after terrain edits steps as the run did.**
  A restore replays og.terrain's edit log on the next tick; handing
  physics the cooked heights and then each brush rebuilt every collider of
  the field, so a restored run left the uninterrupted one on its first
  tick. The replay now hands physics the final heights once (the
  snapshot's change nothing), and a field re-registered with other heights
  before the first step after a restore (a snapshot taken before a later
  edit) reloads the restored simulation over them instead of rebuilding
  its colliders. World hashes of runs without a restore do not move.
- **Fixed: a falling og.CharacterController lands where it meets the
  ground.** Ground snapping now holds only a controller that stood on the
  ground at the start of its move (slopes and steps down), as rapier's own
  controller does; a fall onto ground within `snapDistance` was pulled down
  that far in one tick, a drop of up to 0.2 m at every landing. Replays
  with a controller falling onto the ground step differently.
- **Breaking: creation writes one empty skeleton.** `og create` records
  genre, view, look, mood, UI style, platform, input, players, languages,
  and music in `project.ogt`'s `create` block. It also configures the chosen
  look, mood and UI in `assets/looks/<id>.ogt`, named by the manifest's
  top-level `look`; the entry stays empty. It installs no gameplay,
  art, controls, UI, or translations. `--archetype`, `--profile`, and the
  `create()` options `template`, `archetype`, and `profile` are removed;
  use `--view` for the intended view and build the game with the engine API.
  `Created` returns `choices`, not `look`, `profile`, or `notes`.
- **Breaking: the release archive has no archetypes or 2D starter.**
  `create/template/` and the knowledge scaffold carry the same empty
  skeleton; `archetypes/`, `create/archetypes/`, and `create/template-2d/`
  are no longer shipped. The scaffold's binary distribution list is empty.
- **Unchecked checks have an opt-in failing exit.** `--require-checked`
  makes check-bearing commands exit 1 when a GPU check was not executed.
  The default remains a warning. JSON summaries use `uncheckedCount`;
  `og test` keeps its aggregate `unchecked` as a deprecated alias.
- **Music clocks can start before world time zero.** `Music.start` no
  longer declares a minimum of zero, so starting partway through a song
  passes reflected field validation and default invariants.
- **Playtests check properties every tick.** `og.assert` (`always`,
  `sometimes`, `reachable`, `unreachable`, `each`) records a game's
  properties beside the world, never in it. Outside publish builds every
  tick also checks the default invariants from reflection: no NaN, every
  number inside its field's range, no dangling Entity, roots inside
  `og.WorldBounds`. A playtest's `invariants` and `require`, the
  `invariant:` action and the probe's `assertions` report name the first
  failing tick, the entity, the field and the value, and list every
  sometimes that never fired.
- **UI text uses cooked-font paragraph layout.** Parley, HarfRust and
  Skrifa replace rustybuzz and the space-only breaker. Unicode break
  opportunities, dictionary breaks and grapheme-safe overflow replace
  the handwritten rules; plain labels and styled rich spans draw soft
  hyphens only at taken breaks, reserving their width. Rich-text atoms
  participate as inline boxes. Fonts recook to retain their OpenType
  layout tables without enabling runtime rasterization or expanding a
  declared charset; atlas-only legacy fonts keep table metrics.
  The MSDF renderer and text effects remain unchanged. No IDL change.

- **Posterize can keep hues: `Stylise.posterizeMode`.** `channel` (the
  default, as before) rounds each display channel to its own step;
  `tone` steps a colour's value and its tint apart, so light, exposure
  and a mood move the value and never the hue. render: `PosterizeMode`,
  `Quantize.mode`. The minimal-geometric and heavy-ink-shadows presets
  name `tone`: their own frames move (minimal-geometric's ground reads
  its grey-green, not a snapped grey), and their moods no longer turn
  grounds green or hills pastel. Mood layers of their own for
  minimal-geometric and bold-comic-illustration (dark and high-contrast
  without a vignette; bold-comic's dark without the shadow balance that
  ringed its sky) and muted-cinematic-3d (bright and soft that read);
  `@palettes/handheld4-high-contrast`'s two light greens drawn closer, so
  a flat sky no longer dithers into a dot grid. The previews of
  layered-silhouette, luminous-pixel, monochrome-pixel and
  minimal-geometric are redrawn in their looks' own palettes.

- **The layer stack follows Unreal CommonUI's activatable screens.** The
  stack is ordered by layer kind, then push order, so a modal pushed before
  a menu stays over it and a toast stays over both. Back, focus, `pop`
  (og.ui.pop and a document's) and og.ui.replace act on the active layer,
  the top one whose input is menu or all, not the topmost: a toast over a
  pause menu no longer swallows Back. A document's `pop` pops its own
  layer. ui: `UiLayer.active` added; `UiDocEvent.kind` gains `activate` and
  `deactivate`, sent as a layer becomes and stops being the active one;
  `push`, `pop` and `replace` documented as acting by kind and on the
  active layer (a break for a game that popped a toast over a menu;
  changelog.json says how to fix it).

- **Native accessibility has a shared-frame transport.** An opt-in,
  versioned socket extension carries incremental AccessKit trees with
  stable ids and frame-pixel bounds. Returned actions reach controls on
  the UI tick. Origin Desktop still needs the window's OS adapter
  (platform ask 25); the engine never takes a window handle. No IDL change.

- **The style catalogue goes to schema 3.** `look.family` (an internal
  sample-scene name) leaves the catalogue for `tools/art-samples/scenes.json`.
  `category` now groups the 37 styles in plain terms (pixel art, retro 3D,
  hand-drawn and painted, ink and comic, toon and stylised 3D, low-poly,
  toy and handmade 3D, flat and graphic, realistic). New: `look.views` (the
  creation views each suits), `look.mood` (its preset's mood) and, for the
  14 `content` styles, `look.authored` (each brief kind they need made and
  what it must be). Slugs are unchanged. The art brief gains a `Mood` line
  and, for authored kinds, an `Authored` line; `presetLook(slug, { mood })`,
  `authoredArt(look)`, `MOODS` and `BriefLook.mood` are additive. `og new
  --mood` writes the game's look's mood line, and the premise's style
  match keeps to the chosen view. A reader of `look.family` falls back to
  `category`.
- **A look takes a mood.** `mood "<word>"` (bright, dark, warm, cold,
  soft, high-contrast) in a look file: every built-in preset names the
  mood it is drawn in, and a game's look naming another has the mood's
  layer (`templates/looks/moods/`: grading, exposure, ambient light, and
  for monochrome-pixel and monochrome-dither-3d a palette of the mood)
  laid over its preset before its own fields (docs/art.md §2.10). Ten
  built-in palettes join (`@palettes/handheld4-<mood>`,
  `@palettes/1bit-<mood>`). render: `LookInfo.mood` added. A game whose
  look names no mood draws as before.

- **A script's frames draw at the configured quality.** `og run` without
  `--realtime` and the web probe's `frame` (the browser reference tiers)
  step frames for a script, and automatic quality read their time as a
  display's: on a loaded machine the same script stepped a render-scale
  rung at a different tick each run, so its TAA history and outlines
  differed (six web looks at the second wave's end). Those
  frames now draw at the configured render scale and preset and are no
  sample of automatic quality; a game's display frames adapt as before.
  No IDL change.

- **A flat look's mesh textures keep their patches again.** The flat
  abstraction of a 3D look's colour textures (two Kuwahara passes, then
  the asset's own palette) counts every texel alike in the palette's cut
  again: the activity weighting db74d4ea4 gave the 2D cut, applied after
  the filter, weighed down the borders of the shapes it keeps, so a
  lawn's darker patches merged into the field (anime-cel-3d's ground drew
  one olive, bold-comic-illustration's moved 12.6 %). 2D textures keep the
  weighting. Frames of flat 3D looks move back towards those before
  db74d4ea4; no IDL change.

- **Particles meet the scene's colliders on the GPU.** An emitter's
  `collision` line takes what its particles meet: `collision 0.4 0.2
  colliders` bounces compute particles off the game's og.Collider boxes,
  spheres, capsules and terrains (and scene planes) through the shared
  scene-collider kernel, on or off screen, with nothing read back; `depth`
  (the default) keeps the depth-prepass bounce. Sprite-mode emitters,
  which already simulated in compute, now meet the colliders instead of
  passing through and being counted as skipped (no shipped emitter had
  sprite-mode collision, so no frame moves). Events and sub-emitters stay
  on the CPU, which now also bounces sprite-mode particles off the
  colliders. og-render: `ParticleCollision::against`, `CollisionTarget`.
  IDL doc changes only.
- **Physics: a hull's and a compound's mass properties are the same bits
  on every host.** A convex hull's, a triangle mesh's and a compound
  body's inertia is diagonalised by parry through glamx's eigensolver,
  whose closed form called the host's `acos` and `cos`, so a world of
  such bodies, destruction's loose islands above all, stepped apart on
  native and on wasm from the first tick after a fracture (the wave's end
  of 2026-11-05: the yard's wall split into 112 islands on native and 17
  on wasm). glamx is now carried under `third_party/glamx` (the root
  `[patch.crates-io]`, as basis-universal-sys is) with that solver
  replaced by a cyclic Jacobi iteration in f64 made of `+ - * / sqrt`
  only, so both hosts give one hash; the principal frame of a hull comes
  out as another valid rotation of the same tensor, so a world with
  hulls or compounds hashes differently from before (its motion is the
  same to rounding; `tests/sleep.rs`'s pen took a new seed). A primitive
  shape and every 2D shape never reach the solver and keep their bits.
  The call was inside a dependency, out of clippy's reach, so the guard
  is the determinism tool: it steps 60 compounds of hulls on every build
  (`tools/wasm-determinism`), and `packages/engine`'s destruction test
  compares the yard's hash across hosts.

- **TerrainLayerDesc.cellVariation.** Each cell of a layer's pattern (a
  sett, a flag, a tile) takes its own tone: the layer's normal map's alpha
  names the cell, and each cell's brightness moves by up to 25 % and its
  warmth by up to 20 % at 1. Opaque normal maps and 0, the default, change
  nothing. The terrain blob's layer row is v5, so terrains re-cook.
  Displaced layers' sides (a sett's wall) take their stone's own face,
  unfolded down from its arris, where they streaked into the one row of
  texels along the edge. The setts of terrain-displaced and
  weather-cobbles-wet vary from stone to stone.

- **A lake glitters in pinpoints.** The lake's glints are counted: each
  pixel-sized cell of the surface holds its capillary facets, and a
  hash per cell draws how many of them face the sun (Zirr and
  Kaplanyan 2016, Deliot and Belcour 2023), so a near pixel shows a hard
  glint now and then and a far one averages to the lobe. Each glint is
  soft-capped at eight times the sunlit white. A lake's reflection no
  longer mirrors the sky's sun disc, which every ripple face turned to it
  drew as a soft disc twice the sun's width that bloomed (water-3d, the
  fx-showcase water station).

- **A gale's spume streams off its crests.** The spume's sheets skim the
  sea instead of settling into it behind the next wave, so they read at a
  storm's distance. Past 17 m/s they tear off any crest within a third of
  folding, not only breaking ones. Each sheet is 1 to 2.6 m, three to six
  times as long as wide along its flight, and comes in from 12 m away.
  Droplets are seen to 130 m and spume to 320 m (water-ocean-storm,
  water-shore-storm).

- **Destruction: broken joints shed clay and mortar grit.** Every bond
  that breaks in a prop with og.Destructible.debris sheds three or four
  small chips through the GPU debris layer: one centimetre to five and
  a half across, knocked out of the joint along its normal and moving with the
  blow or blast that broke it, drawn with the prop's cut-face material
  and, for one of them, with a new `fracture.mortar` material, which no
  face draws. At most 48 joints a tick (spread over all that broke) and
  384 at once shed grit; they last og.Destruction.debrisLifetime. The
  crumbs are world state (a restored world throws the same grit), so
  a fracture's world hash changes, and saves of the og.destruction
  state from before are refused (`DESTRUC2`); the cooked fracture asset
  is version 2, and the cook now recooks one of an older version.
  New IDL member: `DestructibleInfo.crumbs`; doc
  change on `og.Destructible.debris`. The yard's wall has a mortar.

- **Breaking: lit mesh particles blend alpha by default.** An emitter
  drawn as lit meshes (`render mesh` with `lit true`) and no `blend`
  line now blends alpha, where it blended additive: a chip, shard or
  splinter covers what is behind it instead of being added onto it,
  where a lit grey chip glowed white over the ground, as paper confetti.
  Unlit meshes, billboards and sprites still blend additive by default,
  and a `blend` line always wins. Frames of such emitters change. Fix:
  nothing for solid debris; an emitter meant to glow as lit meshes
  (lit embers, magic shards) adds `blend additive`. `og upgrade` lists
  every lit mesh emitter without a `blend` line (changelog.json's
  breaking entry). No IDL change.
- **Visual 3D fluids on the GPU.** `og.Fluid3D` gains `simulation`:
  `SIMULATION_VISUAL` (1) has the renderer simulate it with MLS-MPM
  (ported from matsuoka-601/WebGPU-Ocean, MIT; third_party/webgpu-ocean)
  against the scene's colliders, up to 262 144 particles, drawn by the
  same screen-space surface, with nothing read back. The tick holds no
  particles for it (`fluid3dParticles` answers an empty list, buoyancy and
  groups ignore it), so gameplay stays on the CPU fluid. `count` and
  `maxCount` reach 262 144 (the tick still simulates at most 32 768).
  og.Fluid3D is schema version 2; the default keeps every game as it was.

- **3D fluid surface: rounded, refracting, absorbing.** Every og.Fluid3D
  (CPU or visual) is smoothed with the narrow-range filter (Truong and
  Yuksel 2018, as matsuoka-601/Splash runs it, MIT), so free surfaces and
  silhouettes round off instead of showing the particles' flat faces. The
  scene behind is seen along the ray refracted through the body's
  thickness and absorbed per channel (Beer-Lambert), so shallow water is
  clear and deep water darkens to its colour. A visual fluid also turns
  hard-stopped or thrown-loose water to white foam, and a pour narrows and
  breaks up as it falls. Frames of 3D fluids change; no IDL change.
- **The probe's triangles count GPU-culled and meshlet draws.** Both hosts
  ask for GPU culling's counts every frame, so `state().timings.triangles`
  and `dump_render`'s `triangles` hold the camera's drawn triangles a frame
  or two late instead of 0, and the meshlet path counts its clusters'
  triangles at the levels it picked instead of its instances' whole meshes.

- **Every IDL column factory is exported.** `Upscaler`, `GiMethod`,
  `OutlinePlace`, `HalftoneMode`, `HalftoneRegion` and `WindMotorKind`
  were declared but not named exports. The column factories and constants
  are now generated from the IDL (`src/generated/exports.ts`, drift-checked
  by `og api check`), so a new one is exported with its declaration.

- **An old save's og.Wind restores as lazy.** og.Wind is at version 2
  (turbulence and gustSize); a version 1 save, written when every world
  held the resource, drops it at its defaults, so a world that never set a
  wind restores as one without it (the hash this build gives that game);
  a set wind keeps every saved field.

- **Stylise.hatchingPaper.** The paper hatching lies on: one-ink hatching
  lifts its ground towards it, and under `hatchingKeepColor` the wash lies
  on the paper's colour at its own lightness. Under `hatchingKeepColor` a
  weak tint now thins at twice the lift, so a lit floor's faint cast from
  the sky goes to the paper (detailed-doodle and inked-comic-3d). White,
  the default paper, changes nothing else; detailed-doodle draws on a warm
  sketchbook white (`#f8f2e4`), where a cool grey floor read as
  lavender-grey. Moves every raw world hash (the Stylise record gains a
  field).

- **A lake's sun glitter is the sea's.** The lake's power-220 Blinn-Phong
  highlight (some 5° wide, twenty suns) lit whole ripple faces and clipped
  them into flat white fields near the sun's mirror angle; it is now
  web-ocean's Gaussian of the facets' slopes, widened only by the slopes a
  pixel leaves out (LEAN) and their change across it, and bounded by the
  sun's disc: a glitter path of sparkles.

- **Visual simulation on the GPU for cloth, soft bodies and garments.**
  `og.Cloth`, `og.SoftBody` and `og.Garment` gain `simulation`:
  `SIMULATION_GAMEPLAY` (0, the default, unchanged) or
  `SIMULATION_VISUAL` (1). A visual body is simulated by the renderer
  with XPBD in small steps, one compute workgroup per body, against the
  scene's colliders (spheres, boxes, capsules, height fields), and
  nothing is read back. The tick then carries the rest shape with the
  entity (cloth points and garment points stay on their rest shape or
  skin, a soft body keeps `volume` 1 and its rest `center`), so gameplay
  never reads a looks-only motion. A frame steps the ticks since the one
  it last drew (eight at most); a world that went back or jumped further
  starts again with a 60-tick preroll. Where the device has no compute
  or vertex storage (`og.render.capabilities().visualSimulation` false)
  a visual body draws its rest shape (a garment its skin). New fields;
  the default keeps every existing game as it was.

- **Destruction: fine debris throws rubble chips in its own material.**
  Each fine piece the debris layer flies (og.Destructible.debris) now
  throws four to six chips of its own as GPU rigid bodies: thick convex
  lumps (at least about half as thick as wide), most of them small and
  a few near the piece's size, drawn with the prop's cut-face material
  (one in four with its outer face's). Brick throws clay, concrete grey
  aggregate, wood its grain and glass clear glass, with no tint to
  author. They are visual only, seeded by the piece, and caught up like
  the pieces. Doc change on `og.Destructible.debris`. The example
  yard's mesh emitters now say `blend alpha`: emitters then blended
  additive by default (the break above), so a lit grey tetrahedron added onto the ground glowed
  white, as paper confetti. Its wood and glass props now have debris
  too.

- **A watercolour wash over the painted frame.** `Stylise` gains
  `watercolorEdgeDarkening`, `watercolorGranulation`,
  `watercolorTurbulence`, `watercolorDryBrush`, `watercolorWobble` and
  `watercolorPaper`: a pass after the painterly strokes that ports
  Bousseau et al. 2006 and MNPR (Montesdeoca et al. 2018,
  `third_party/mnpr`). A pigment density about one, weighed by the
  pigment load so the paper and the palest washes stay clean, thins each
  colour towards the paper or thickens it, C(1 − (1 − C)(d − 1)), so the
  wash keeps its mean value where `paintCanvasStrength`'s multiply only
  darkened: granulation settles the pigment into the paper's pits and
  off its peaks (the canvas's relief about its mean, in units of its
  spread, so any canvas reads as paper), turbulence drifts it with a
  slow noise about a hundred texels across seeded per object; pigment
  pools in a rim two to three pixels wide at a wash's outer boundary (a
  colour step wider than a line, or a silhouette; a seam, a crease or a
  like-coloured neighbour is none, so a tiled floor is one wash); the
  dry brush leaves the paper's peaks bare in that rim and on the
  light-catching tones; and the frame is read displaced by the relief
  and a waver. The cook applies the same wash in texel space on painted
  2D sprites and painted 3D colour textures (`PAINT_VERSION` 4, so
  painted textures recook); on painted 2D (a `profile "2d"` project under
  the art contract, docs/art.md §2.3) the frame then draws neither the
  paint nor the watercolour pass, so a sprite is painted and washed once
  and the look is tuned once for both. Every field is off at zero, so no existing
  look moves; `watercolor-wash` uses them and now reads as translucent
  washes on white paper rather than oil paint.

- **Whole-turn primitives are closed solids; fractured cylinders break
  into lumps.** `cylinderGeometry`, `coneGeometry`, `capsuleGeometry`,
  `torusGeometry`, `ringGeometry`, `circleGeometry`, `tubeGeometry` and
  the built-in sphere computed the last column of a whole turn at 2π,
  where f32 sine is not 0, and the sphere's and capsule's poles missed
  the axis by sin π, so the seam never welded. A fracture cut through
  that slit leaked: a third of a cylinder's cells came out as open
  skins with no cut faces and no bonds (the thin shells and flat cards
  a concrete pillar broke into). The seam now reuses the first column's
  angle and the poles sit on the axis; the pillar's pieces are all
  closed lumps (least to greatest width at least 0.47, where 13 of 44
  were under 0.35). Vertex positions move by under 1e-7; a cooked
  fracture of a cylinder changes and is recooked.

- **Scatter placement on the GPU.** New `og.TerrainScatter.gpuPlacement`
  (on by default; `gpuPlacement: false` places every chunk on the CPU
  as before): the scatter's chunks are placed by a compute pass
  when they come into range, by the same rules (layer weight, holes,
  `noiseThreshold`, `avoidWater` and the footprints of the scatters
  placed before it) from the terrain's finest tiles, with nothing per
  copy on the CPU. On the lake-shore test scene (three chained scatters,
  6 copies per m², about 3.5 new chunks a frame) a frame's CPU time
  drops from 60.3 ms to 14.3 ms (dev build) for 0.21 ms of GPU; the
  frame matches the CPU placement's pixel for pixel. Without compute,
  the CPU places them as before. Not a break: every scatter scene's
  reference frame (terrain-grass, terrain-grass-wind, terrain-hidden,
  terrain-scatter-unshadowed, impostor-scatter, probe-outdoor) holds
  within its tolerance with the GPU placing it.

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

- **Image-based light, shadows and textures as three.js draws them (PR #8:
  s42, s55, s14, s18, s36).** The environment's specular is Karis's split
  sum, F0 times the LUT's scale plus its bias, in every lit shader (the
  standard, physical, scatter, terrain, grass, crowd-card, impostor and
  custom-lit paths): the bias already carries the Fresnel rise, so rough
  surfaces seen edge-on no longer reflect three to four times the sky. The
  standard material reads its reflection bent towards the normal by
  roughness squared (three.js's getIBLRadiance), so a rough back face no
  longer mirrors the sky behind it. Alpha-tested materials cast cut-out
  shadows (leaf and grass cards no longer cast solid quads). The material
  sampler's anisotropy follows the sprites' rule, so a software
  rasterizer stays at 1. The cook fills a mipmapped colour texture's
  transparent texels from their visible neighbours (push-pull) before its
  mips, so cut-out foliage no longer darkens with distance;
  COOKER_VERSION r10 -> r11, so every game re-cooks once (the release's
  breaking entry). No IDL change. Behaviour change: frames with rough
  surfaces at grazing angles, alpha-tested casters or cut-out textures
  move.

- **Models share identical textures and meshes (PR #8: s24, s72).** The
  cook writes a glTF texture outside a streaming group and an art contract
  once per image and use (`_gltf/<image hash>-<kind>#tex`, `-stream` for a
  `stream_textures` model's), whichever models name it, instead of once per
  model (`<model>#tex<n>`): a scene exported as many glTFs (cells, tiles)
  cooks, downloads, transcodes and uploads each image once (Grand View
  Garden: cooked bytes 1.5 GB -> 791 MB). At load, unstreamed models share
  those textures and meshes of the same vertices, indices and foliage (and
  their levels of detail), and a model's materials, still its own, draw in
  the instanced batch of an identical material loaded before
  (`RenderScene::alias_material`, copy-on-write: an edit of either ends
  it), so one tree in many cells draws in one batch while
  `og.render.updateMaterial` on one model's material never changes another
  model. A shared mesh's key is a word-at-a-time hash (SipHash over every
  mesh was 1.5 s of a 1072-model web boot). `og dev` decodes a shared
  texture again in place when its output changes. No IDL change. Breaking:
  a glTF texture's `<model>#tex<n>` id is gone after the next cook (the
  release's breaking entry; `og upgrade` lists the lines naming `#tex`).
  Frames do not change.

- **Web: frame pipelines `auto` by default.** project.ogt
  `publish { pipelines }` takes `auto`, now the default: the Worker picks
  `async` on a mobile GPU (`GPUAdapterInfo.vendor` `img-tec`, `arm` or
  `qualcomm`, or `navigator.userAgentData.mobile` when the browser names
  no adapter) and `sync` elsewhere, Apple GPUs included until iOS is
  measured. On a Pixel 10 `sync` left the page without a frame for about
  23 s behind the gate (live at about 27.5 s from navigation, against
  about 18 s with `async`); on a Mac `async` made a cold boot 2.5-5x
  later, and `auto` runs `sync` there (docs/web-export.md §Pipelines).
  `sync`, `async` and `prewarm` stay as explicit choices. The mode run
  and why are `mode` and `reason` on the probe's `boot.pipelines` and
  `pipelines { mode, reason }` in the boot telemetry sample, and
  `boot.pipelines` is reported again when the tap stops the warm-up.
  Behaviour change: a game that set no `pipelines` runs `async` on a
  phone's PowerVR, Mali or Adreno GPU; `pipelines sync` keeps the old
  default.

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
  pier feet it barely reaches keep their footing. The og-destruction
  skill has breakables lit with og.GlobalIllumination `both`: a stump's
  overhanging cut faces face the ground and read black without bounce.
  The debris layer: a shattered chunk's pieces given to `debris` fly on
  the GPU as rigid bodies (og-render `DebrisPiece`, XPBD rigid bodies of
  Müller et al. 2020, `third_party/gpu-debris.md`). Each is drawn as an
  ordinary instance of its own cut piece, starting where and how fast it
  broke off, landing on the scene's box, sphere and terrain colliders.
  Nothing is read back. `DestructibleInfo.debris` counts them, and
  `DestructionCapabilities` gains `maxVisualDebris` and
  `visualDebrisContacts`. The scene's collider packing for compute
  (`scene_colliders`) is shared with the GPU visual-body solver. A loose run of
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
- **Changed: a missing GPU warns and leaves frames unchecked.** Native
  `og run` and `og test` continue as renderer `none` when no adapter is
  available. Warnings name wgpu's error; JSON includes `renderer`,
  `adapterError` and unchecked counts. Unchecked suites are not passed;
  executed checks still fail normally. Rust and Bun GPU tests register as
  skipped without an adapter. `OG_RENDERER=none` remains optional and
  removes only the warning's "unexpected" wording (platform ask 20).
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

## Earlier releases

Releases before 0.7.6 are in `packages/engine/changelog.json` (every
version, machine-readable) and in this file's git history.
