# og-cook

The cook (`docs/design.md` §2.5): source assets in `assets/`
become `.ogb` blobs (`crates/og-blob`) in `cooked/`, incrementally by hash.
`cooked/` is the only thing the runtime reads, through og-bundle's loaders
(`crates/og-bundle/README.md`), which og-cook re-exports (`og_cook::runtime`).

```rust
let report = og_cook::cook_project(root, &og_cook::CookOptions::default())?;   // og cook
assert!(report.ok(), "{:?}", report.errors());
```

`og-cook <project> [--threads N] [--report out.md] [--json out.json] [--schemas schema.json]`
is a development binary; `og cook` (og-cli) is the user-facing command and
calls `cook_project` (`--report` for the slowest ten, `--watch` to cook each
change as it is saved: §Cook times).

## Features

| Feature | Default | What it adds | Targets |
|---|---|---|---|
| `transcode` | yes | og-bundle's `transcode` (native Basis transcoding in the loaders) | native |
| `cook` | yes | `cook_project` and the importers (gltf, image with PNG, JPEG, WebP, TGA, EXR and Radiance HDR, image_dds, the in-crate PSD reader, draco-core (pure-Rust Draco), meshopt, mikktspace, fdsm, ttf-parser, hound, ruzstd), og-render-data's `bake` (environment maps) | native |
| `basis` | yes | the basis-universal crate (basisu 1.16) as the texture encoder (and og-bundle's `basis`); without it the cook uses `toktx` from PATH | native |

## Pipeline

1. **Discover** checked `content/**/*.ts` declarations. External models,
   textures, sounds, fonts, Tiled/LDtk maps and Rive files use `file.import`
   with an explicit owning-module `assets/` or `sources/` path and native
   controls. Captured external model buffers, images and material libraries
   are part of that import. Raw files do not become assets by extension.
   The remaining look, materials and Yarn author formats are scanned until
   their domain cutovers; unrelated files (licences, notes) are ignored.
2. **Plan**: every output gets a cook stamp (below). An output whose stamp the
   manifest already holds, with its file present, its bytes' hash unchanged
   and its header holding the type hash and schema version the loader
   expects today (`format::expected_header`), is up to date and not cooked.
   The header check catches a schema that changed without an area
   revision bump: the manifest and the file agree, but the loader
   would refuse the blob ("holds type ..., expected ...").
3. **Cook** the stale outputs on a thread pool (`CookOptions::threads`, 0 =
   all cores); results are collected by plan index, so scheduling never
   changes the bytes. The outputs with the least work go first
   (`Job::cost`: a texture's texels, times 8 for UASTC), and each output
   starts with its share of the cook's threads by its share of the work
   that is left: a cook of many outputs keeps one output a core, and the
   large UASTC encodes at the end take the cores the others have freed
   (the block encode runs on them; the RDO pass after it is one job at
   every count, so the bytes are a single thread's). Planning cooks
   nothing: a model's LOD surfaces, whose ids its cook decides, are
   planned from the cooked blob that names them, else once the model has
   cooked, and then cooked in a second round (`Job::derived`, the report's
   `derived` stage).
4. **Write** outputs whose bytes changed (unchanged files are not touched),
   delete outputs of removed or failed sources, write
   `cooked/.cook.ogt` when it changed. Return the `Report`.

### Shared cook results

Native cooks use BLAKE3 action keys and verified contents in
`$HOME/.cache/origingame/cook-v1/{ac,cas}`. Without HOME, Windows uses
`$LOCALAPPDATA/origingame/cook-v1`. A texture's, sound's or font's key is
its source's bytes, its type and layout, and its stamp, which holds the
options and settings its cook reads and its areas' revisions: nothing of the
project, so the same file cooked the same way is one entry for every
project, checkout and engine build that would write the same bytes. Other
outputs' keys hold the project's inputs, the output's id and the cook's
options (no path among them) in place of the source's bytes. Prefracture keys cover the model and discovered dependency
contents, not its pathname. Video work keeps its existing encoder path.

`OG_COOK_CACHE=/path/to/cache og cook` selects an isolated store;
`OG_COOK_CACHE=off` (or an empty value) disables it. `OG_TEXTURE_CACHE` is a
fallback for existing callers. `OG_COOK_CACHE_MAX_BYTES` sets the size cap
(default 10 GiB). A cook trims the store at most once an hour (`gc.stamp`
in the store records the last pass); `og cache gc [--max-bytes N] [--json]`
trims it on demand.
Rust callers can use `cook::cook_project_with_texture_cache(root, options,
Some(path))`, or `None` to disable caching without reading or changing the
environment. This entry point retains its historical name but selects the whole
store. The cache tests use private temporary paths.

Reads validate BLAKE3 contents and the result's blob type, schema and cook stamp.
Missing, corrupt or unwritable entries fall back to normal cooking.
Writers exclusively create a sibling temporary file and rename the completed
entry into place; concurrent cooks may duplicate work but cannot expose a
partial result. A hit adds `shared texture cache hit` or `shared cook cache hit`
to the asset's notes. The output still counts as processed
(`cooked: true`), with cache lookup time, and enters the normal project write
and manifest path. Unchanged project outputs remain `cooked: false`.

The per-project `.og/cook-index` records file size, timestamps, file identity and
BLAKE3 content hash. An unchanged cook reuses its report without reading source
bytes. A stat proves a file unchanged only when the file last changed before the
cook's clock, the time the project's filesystem gives a file written at that
moment: file times advance in ticks, and a same-length rewrite in the tick of the
last change keeps the stat. A source or cooked output whose stat is not older
than the clock is racy. A racy source's content hash is taken as the cook opens,
before it is read, and a racy output's as the index is saved; the next cook
compares them, and drops what has settled since. A racy stat is never recorded
as a scanned source's entry. Changed projects are planned again; a source is
always read from its own file, and the index keeps its hash by its stat, never
a copy. Video projects and
models with dependencies outside `assets/` are replanned. The index and shared store are never shipped.

This is a disposable, trusted local cache, not an authenticated artifact store.
Eviction drops unreferenced contents first, then the oldest actions and their
unshared contents. Interrupted writes can leave `.tmp` files; GC removes them
after an hour, leaving recent writes alone even if they exceed the cap.

### Importers

| Source | Output (`cooked/…`) | What happens |
|---|---|---|
| glTF 2.0 | `<path>.ogb` (`og.Model`) + `<path>.tex<n>.ogb` per texture use | Per primitive (glTF order): positions, normals (flat when missing), uv0, colour0, tangents (MikkTSpace for normal-mapped primitives that lack them, via unwelding); weld; quantise (position u16×3 over the bounds, normal oct snorm16×2, uv u16×2 over the uv range, colour u8×4, tangent oct snorm16×2 + sign); weld again; meshopt vertex-cache and overdraw (1.05) ordering; LOD chain by `meshopt_simplify` (3 levels, each half the previous, max error 5 % of the extent, stops when a level saves < 5 %); vertex-fetch reorder over all LODs; meshopt vertex codec v1 and index codec v1. Materials (metallic-roughness, emissive strength, alpha mode, double-sided) plus a default material, nodes in topological order, scenes and their roots. Skins, animations and morph targets go to the `#anim` blob (next row). Welding keys skinned and morphed vertices by their skin and morph data, so it never merges vertices that deform apart. Before the gltf crate reads it the file is made plain (`src/cook/formats/model/plain.rs`, nothing renumbered): `EXT_meshopt_compression`/`KHR_meshopt_compression` views decoded by meshoptimizer (filters included) in place, `KHR_draco_mesh_compression` primitives decoded by draco-core into their accessors, `KHR_mesh_quantization` integer attributes read as floats (the file's own dequantising node scale and texture transform kept), a texture whose image is only in `KHR_texture_basisu`, `EXT_texture_webp` or `MSFT_texture_dds` given it as its source; a file requiring any other unknown extension is refused by name. Then (`src/cook/formats/model/ext.rs`): `KHR_texture_transform` baked into a primitive's uvs where every textured material it can draw with (its variants' too) shares one transform for all its textures, else carried per slot in the material row (base colour, metallic-roughness with a physical material's packed maps, normal, emissive, occlusion) to og-render's `UvTransform` uniforms; `KHR_materials_unlit` as the row's `unlit`; `KHR_lights_punctual` as `lights` rows by node (og-runtime spawns them with each instance); `EXT_mesh_gpu_instancing` as `instances` rows (the node draws its one mesh at each, as instances); `KHR_materials_variants` as `variants` and `variant_mappings` rows (og.Model.variant picks one). |
| MagicaVoxel `.vox` | as glTF 2.0 (`og.Model`) | `cook/formats/vox.rs`: `SIZE`/`XYZI` models (every one of a `PACK`), the `RGBA` palette (MagicaVoxel's default without one) and the `nTRN`/`nGRP`/`nSHP` graph (translations and the `_r` signed axis permutation; one with a negative determinant mirrors the model and its triangles' winding is flipped, so every face stays counter-clockwise from outside; without a graph each model sits at the origin about its pivot, exactly where a graph of one identity transform puts it, as MagicaVoxel and ogt_vox read such a file) become one glTF binary in memory: per placed model, a greedy mesh (per face direction and slice, maximal rectangles of one colour whose face is exposed; a 32³ block of one colour is 12 triangles) of unwelded quads with face normals and the palette colour as `COLOR_0` (linear), z-up turned to y-up (`(x, y, z)` → `(x, z, -y)`), `voxScale` metres a voxel (import settings `model { }`, default 1), the pivot MagicaVoxel's `floor(size / 2)` (ogt_vox's documented convention, which vengi matches: voxel `v` turned by `R` at `T` fills the cell at `floor(T + R (v + 1/2 - floor(size / 2)))`, odd sizes turned or mirrored included, checked voxel by voxel on the committed `tests/fixtures/vox/pivot.vox`), under a node at its translation; `voxOrigin bottom` lifts the whole file so its lowest voxel stands on y = 0 (`center`, the default, keeps the editor's origin); one white rough material. Animated files (MagicaVoxel's frames: an `nSHP` of several models, each shown from its `_f` key frame to the next; or several models without a graph, 0.98's `PACK` animation, model `i` frame `i`) become a glTF of mesh swaps: each animated model a plain mesh node at its translation (no joint: a frame that never moves needs no palette matrix and draws as a static instance when shown; a model a keyed transform moves is skinned, below), and per clip one visibility track per model on its mesh node (`OG_animation_visibility`, the row below; STEP, 1 on the frames its model shows and 0 elsewhere), a key per frame at `voxFps` (default 10) and one at the clip's end holding the last frame, so a frame folded away is not drawn (og-runtime skips a hidden node's primitives; before the visibility track it was a STEP scale of 0 on its joint, still drawn); the clips are `voxClip <name> <first> <last>` ranges (both frames shown), else one clip `frames` of every frame. Each frame's model sits about its own pivot as MagicaVoxel shows it (frames of different heights do not share a floor; `voxOrigin bottom` lifts the file by the lowest voxel of any frame). The rest pose shows frame 0 (the other frames' mesh nodes `KHR_node_visibility` hidden, which the model blob's node rows carry, so the draw without an `og.Skeleton` is frame 0 alone, and the model's bounds leave them out; a skinned one is also scaled to 0 for a reader that ignores the extension). `nTRN` key frames (each frame of a transform its own `_f`, `_t` and `_r`, held until the next key as ogt_vox samples them) make a file animated too: the graph is walked once per frame up to the last key of any shape or transform, and a model whose place or turn changes across frames is a mesh per run of frames at one turn (a turn is another set of voxel faces, swapped in like a frame's model); a run whose place changes is skinned whole (weight 1) to a joint of its own (scale 1, inverse bind the identity) carrying a STEP translation channel beside its mesh node's visibility track, its place on each frame (on a frame it is folded away, the place of its nearest shown frame); keys that change nothing leave the file static, byte for byte. A node's `_loop` (`nTRN` or `nSHP`, `1`) repeats its keys from its first to its last on every frame outside them, a frame before the first counting back from the last (ogt_vox's `compute_looped_frame_index`); the scene's playback range (`META`'s `_anim_range "<first> <last>"`, written by MagicaVoxel since 0.99.7.2) walks the frames up to its last (keys past it are not played) and is the default clip `frames` (frames before it stay frames of the file a `voxClip` may name); a range that ends before it starts is ignored, and a file whose range shows nothing move (the editor writes `0 30` into every file) stays static. Then the glTF row above, and the `#anim` row below for an animated file. |
| FBX, Wavefront OBJ | as glTF 2.0 (`og.Model`, and `#anim` when skinned, morphed or animated) | `cook/formats/fbx.rs`: read by ufbx (FBX binary and ASCII from 3000, OBJ with its `.mtl`; MIT or Unlicense, compiled into the native cook only) and written as one glTF binary in memory, so the glTF rows apply unchanged. Metres and +Y up right-handed (the file's unit and axes converted into the geometry); pivots, pre- and post-rotations fold into each node's TRS, geometric transforms bake into the mesh (a helper node when it is instanced), non-standard scale inheritance is compensated. Polygons triangulated, one primitive per material, a mesh several nodes show is one glTF mesh. Corners carry position, normal (made when missing), `TEXCOORD_0`/`_1` (v flipped) and `COLOR_0` (made linear). The first skin deformer is a glTF skin (cluster geometry-to-bone as the inverse bind, the 4 strongest weights normalised, noted when more), its mesh on a root node; blend shapes are morph targets named by their channels; every animation stack is a clip of its name baked by ufbx from 0, its blend channels' `DeformPercent` as a `weights` track. Materials from ufbx's PBR reading of every shading model and the `.mtl`: base colour and map, opacity (alpha blend; a separate opacity map packed into the colour's alpha), roughness (glossiness inverted) and metalness (separate maps packed into G and B at cook time), normal, emission (`KHR_materials_emissive_strength` above 1), occlusion, double-sided, unlit; a texture's UV transform is `KHR_texture_transform`. Embedded textures are packed into the glb; others are found by relative path, else by file name beside the model. Point, spot and directional lights become `KHR_lights_punctual`; cameras, area and volume lights are dropped with a note. Refused with the fix: an unreadable file, a texture neither embedded nor beside it, a `mtllib` not beside the OBJ, a file with no polygon mesh. |
| glTF skins, animations, morph targets | `<path>.anim.ogb` (`og.Animation`, id `<path>#anim`) | Only when the glTF has any. The node hierarchy in the model blob's order; skins (joints, inverse bind matrices, `skeleton`); per clip, every channel evaluated by the glTF sampler (STEP, LINEAR with shortest-arc slerp, CUBICSPLINE Hermite; f64; a key within 1 µs of a sample counts as reached) at `i / rate` and at the end, `rate` = the import settings' `animation { rate }` or the source's key rate rounded (1–120 Hz), the source rate kept; markers from `animations[i].extras."og.events"` (`[{"name", "time"}]`) and `animation { event "<clip>" "<name>" <s> }`; visibility: the nodes `KHR_node_visibility` hides (`visible: false`) are the blob's `hidden` table (og-anim's `SkeletonAsset::hidden`, the rest visibility), and an animation's visibility tracks, the engine's extension `animations[i].extensions.OG_animation_visibility.channels` (`[{"sampler": <the animation's sampler>, "node": <node>}]`, the sampler's output a float scalar per key, visible from 0.5), are STEP channels of 0 or 1 (`path` 4, `ChannelPath::Visibility`; glTF's own `KHR_animation_pointer` onto `/nodes/<n>/extensions/KHR_node_visibility/visible`, a channel path the gltf crate refuses, is rewritten in the JSON before the crate parses it into an entry of this extension, its sampler's output floats or unsigned bytes, 0 hidden; the crate then validates the file as any other. A pointer onto any other property is dropped with a warning, or refused naming it when the file requires `KHR_animation_pointer`; a file with no `pointer` path is parsed untouched); samples f32 by default (a channel whose frames are all the same bits stored once), or with `animation { compress true }` quantised with keyframe reduction within `tolerance` (og-bundle README §Animation samples); per skinned primitive joints u16×4 and weights unorm16×4 (summing to 65535) per cooked vertex; per morphed primitive, sparse, each target's moved vertices (a delta with any bit set) with their indices, position (and normal) deltas, names from `mesh.extras.targetNames`, default weights; streams meshopt-encoded. |
| PNG/JPEG/WebP/TGA/PSD/DDS | `<path>.ogb` (`og.Texture`) | Decoded to RGBA8 (`src/cook/formats/texture.rs` by its bytes: WebP lossy and lossless; TGA; DDS level 0 of BC1 to BC7 or uncompressed, image_dds; PSD/PSB the flattened composite Photoshop stores beside the layers, `src/cook/formats/psd.rs`: RGB, greyscale, CMYK, indexed and bitmap at 1 to 32 bits, raw, PackBits or zip, its transparency only when the layer count says so), then KTX2: colour → ETC1S/BasisLZ, sRGB, quality 128; `kind normal` → UASTC level 1 + Zstandard (no RDO), linear, tuned for normal maps, x y z in RGB and y again in alpha (Basis transcoders take BC5's and EAC RG11's second channel from alpha; `has_alpha` 1 says a map carries it, and the settings string's "normal y in alpha" recooks older maps); `kind linear` → the same without the normal-map tuning. Full mip chain. `kind pixel` → RGBA8 + zlib, sRGB, lossless, one level; the runtime samples it nearest (sprite sheets, tilesets). With `texture { stream true }` (a model's textures: `model { stream_textures true }`) a texture is encoded once and cooked as two outputs: `<path>.ogb` (id `<path>`) holds its mip tail, the whole chain's levels from the first whose longest side is at most 256 down, byte for byte, with a raw section `mips` giving the whole chain's level-0 width, height and level count; `<path>.mips.ogb` (id `<path>#mips`, og-bundle's `mips_id`) holds the whole chain, the ordinary blob. The runtime draws the tail and reads `#mips` when the texture is big on screen; the web host does not fetch `#mips` at boot and a publish build's service worker caches it when fetched. A texture whose header says it is at most 256 on its longest side, or in a streaming group (whose pack holds its whole chain), is cooked whole, with a note. In a glTF, base colour and emissive are colour, metallic-roughness and occlusion linear, normal normal. A material whose occlusion and metallic-roughness maps are separate images on one UV set with one transform cooks one packed texture in their place (R occlusion, G roughness, B metallic, at the larger map's size): packed when that texture cooks, never while the model loads, with the two source images' bytes as its hash, stamp input and shared-cache key (`model.rs` `ORM_PACK`). |
| KTX2 | `<path>.ogb` (`og.Texture`) | `src/cook/formats/ktx2_source.rs`. Basis ETC1S or UASTC, or RGBA8 (none or zlib), its `KTXorientation` and `KTXswizzle` honoured: a file that is `rd` and `rgba` (or says neither), of a colour or linear kind matching its transfer function, is taken byte for byte; any other is decoded (Basis through the native transcoder), flipped to rows top to bottom and columns left to right, swizzled (`r g b a 0 1` per channel) and cooked again as an image of its kind (`texture { kind }`, else sRGB colour or linear). The report says which. `toktx`'s own output goes through the same check. |
| texture array `.ogt` (`type texture-array`) | `<path>.ogb` (`og.TextureArray`) + `<path>.layer<n>.ogb` (`og.Texture`, id `<path>#layer<n>`) per layer | `src/cook/formats/texture_array.rs`: `kind color|normal|linear|pixel` and 1–256 `layer "<file>"` images (beside the file, else under `assets/`) of one size (a layer of another size is refused naming both); each layer cooked as a texture of the kind; the array blob holds the size, kind, level count and the layers' ids. The runtime uploads the layers as one `D2Array` texture (`og.load.textureArray`, og-render's `RenderScene::add_texture_array`). World and overlay sprites (`SpriteDesc.array`, `og.Sprite.array`), `array true` flipbooks and 2D particle emitters (an emitter's `array` line, og-bundle's `src/fx.rs`) draw from it, one draw per array whatever the layers; so do lit materials (`mapArray`, `normalMapArray`) and terrain layers (`layers`, `layerNormals`). |
| WAV, MP3, Ogg Vorbis, FLAC | `<path>.ogb` (`og.Sound`), and for a sound cooked in chunks `<path>.chunk<n>.ogb` (`og.SoundChunk`, id `<path>#chunk<n>`) | A WAV's 8–32-bit integer or 32-bit float PCM, 1–8 channels, to 16-bit; an MP3, Ogg Vorbis or FLAC decoded to 16-bit by symphonia (pure Rust, the first audio track, encoder delay and padding trimmed; Ogg Opus is refused); then QOA (in-crate encoder: 3.2 bits per sample). With `sound { stream true }`, or when the QOA file would pass 1 MiB (27 s of 48 kHz stereo) unless `stream false`, the file is encoded once and cut between QOA frames into chunks of about `chunk` seconds (default 4, whole frames): the sound's blob holds the QOA header and the first chunk, each other chunk a blob of its own, which the runtime reads ahead of the playheads and drops behind them (og-audio's README §Chunked sounds). A QOA frame carries its predictor's state, so the chunks decode to the whole file's samples exactly. A chunked sound in a stream group puts its head in the group's pack; its other chunks stay files of their own (never grouped), read ahead of the playheads once the group is resident. |
| TTF/OTF | `<path>.ogb` (`og.Font`) | The face's ascender, descender and line gap in em, its file whole (`file`), and a pixel font's grid (`pixel`); no glyph table, kerning or pages. |
| checked TS `file.rig` | `<module>/<content-stem>.ogb` (`og.Rig`) | A humanoid map (og-anim's README §Retargeting): naming scheme, standard bones mapped by name or explicitly unmapped with null, and optional per-foot heights; native fields loaded with `og.load.rig`. A model's `animation { rig }` names its canonical content ID for retargeting. |
| animation library (a glTF named by another model's `animation { library }`) | into the naming model's `#anim` | The library is cooked like any glTF (its own `#anim`, loadable with `og.load.animation`); the naming model's `#anim` gets the library's clips after its own, each rewritten by `og_anim::retarget` from the library's skeleton (its humanoid map from its own `animation { rig }` or detected) onto the model's (its `rig` or detected): rest corrections, the model's bone lengths, the hips' travel scaled by the leg ratio, the legs solved for the feet unless `footIk false`. A clip the model has of its own wins over a library's of the same name (a note says so); a library naming libraries is refused. Then each `mirror "<clip>" ["<name>"]`: the clip (the model's own or a library's, retargeted) mirrored on the model's skeleton by `og_anim::mirror` (sides paired by name, reflected in x = 0), written under the name (default `<clip>.mirrored`), so a mirrored clip costs nothing at run time; a clip it lacks, a name already taken or named twice is refused. |
| checked TS `file.flipbook` | `<id>.ogb` (`og.Flipbook`) | Native sheet fields from a canonical texture ref with grid, explicit frames or captured Aseprite/TexturePacker JSON; alternatively explicit module-relative animated GIF/WebP/APNG/still sources, decoded, composited, keyed, aligned, cropped, deduplicated and packed into `<id>#sheet`. Clips use frame names, positive timing and events. `array: true` also writes `<id>#array` and one `<id>#array#layerN` texture per frame (equally sized and inside the sheet, at most 256). No OGT author parser, text writer or sibling-path fallback. Input images, exchange data and helper imports participate in cook invalidation. |
| checked TS `file.scene`, `file.prefab`, `file.scatter` | `<module>/<content-stem>.ogb` (`og.Scene`) | Captured native entities depth-first, parents before children; reflected engine/game defaults and typed content relocations, sparse resource patches, native prefab expansion and ordered edits. No author text parser or generation wrapper. |
| checked TS `file.scene` with `hlod: { size, error, texels }` | `<id>#hlod` (`og.Hlod`) + per node `<id>#hlod-<n>` (`og.Model`) and its textures `#hlod-<n>-base`, `-surface`, `-emissive` | `src/cook/systems/hlod.rs` (docs/design.md §10.6): the scene's `static` models (its prefabs' too) in a quadtree over x and z, a leaf a `size` cell (default 64 m) of two or more, each level twice as wide, at most 10 levels. Per node one proxy, a leaf's from its models' source geometry and a higher node's from its children's proxies: welded, `meshopt_simplify` with pruning, the limit lowered (×0.6, at most 8 times) until the measured error is within `error × 2^level` (default `size` / 256), normals split at 45°, charted by xatlas at one texel per error within `texels` (default 1024), painted by og-bake's `transfer` and cooked as a model of one level, no collider and no raycast BVH. The node's stored error is the larger of the measured distance and a texel. Parts lying flush on a larger one within half the error (a window on its wall) are dropped first, and painted on it. Left out and counted in the report (`hlod` in cook-report.json) by reason: skinned, impostor, cut-out or translucent, fractured, GPU-instanced, tinted, flashed, variant, swaying, hidden, larger than a leaf, or with no other model near. The outputs stay in the first group whatever stream group the scene is in; a node's stamp is its models and where they stand (`tests/hlod.rs`). |
| checked TS `file.tileset` | `<id>.ogb` (`og.Tileset`) | Grid, solid/rect/polygon collision, animated tiles and blob47/bitmask16 terrains. Optional texture is a canonical content ref. Native fields are shared by bundles, embedded imports and snapshots; no text parser or fallback. |
| checked TS `file.tilemap`, Tiled `.tmj`, LDtk `.ldtk` | `<id>.ogb` (`og.Tilemap`) | One `og_tilemap::TilemapAsset` whatever the source (`src/cook/systems/tilemap.rs`, native only). **TS**: direct checked layers, packed u32 cells (0 empty, else tile ID + 1 with optional X/Y flips; terrain 0/1), canonical tileset refs and world-space objects; no text parser or generate wrapper. **Tiled**: orthogonal, finite maps; layer data as a JSON array or uncompressed base64 (zlib/gzip/zstd refused: save as CSV or Base64); flip bits kept (diagonal refused); `offsetx/y`, `parallaxx/y`, `opacity`, `visible`, `repeatx/y`, group layers flattened; custom properties `collision` (bool), `z` (int), `autotile` (a terrain of the layer's tileset: nonzero gids mean filled), `repeatx`/`repeaty`; an external tileset `x.tsx`/`x.tsj` needs an explicit `bindTileset "x.tsx" "module/tileset-id"` import setting (or the all-tileset override), never a guessed sibling; an embedded one (single image) is imported with its tiles' `animation`, first `objectgroup` rectangle (collision) and boolean `solid` property; one tileset per layer; object layers become objects (`type` or `class` as kind, string-valued properties; tile objects anchored at their bottom). **LDtk**: one level (the first or `level`; separate level files refused), every layer on one grid size; `Tiles`/`AutoLayer`/IntGrid auto tiles become tile layers (tile id `t` or from `src`; flips from `f`; one tile per cell, the last of a stack wins), drawn in reverse list order (LDtk's first layer on top); IntGrid values listed in `solid` (default 1) become an invisible collision layer `<name>.collision` over an embedded one-tile solid tileset; an explicit image-path binding or all-tileset override uses a checked TS tileset, otherwise one is embedded from `tileGridSize`, `__cWid`, `__cHei`, `padding`, `spacing`; `Entities` become objects at their top-left (pivot applied), kind the identifier, name the `name` field or the `iid`, fields as properties. Units: `unit` world units per tile width (default 1); object positions and offsets in map-local world units, y up from the top-left corner. Referenced tilesets and images must be known cooked outputs. |
| environment `.ogt` (`type environment`) | `<path>.ogb` (`og.EnvironmentMap`) | `src/cook/systems/environment.rs`: its `source` (an equirectangular Radiance `.hdr` or OpenEXR `.exr`, linear RGB, twice as wide as tall, decoded by the `image` crate; or a KTX2 cube map, `src/cook/formats/cube_source.rs`: six square faces +x −x +y −y +z −z with rows from the top, level 0 of RGBA16F, RGB16F, RGBA32F, RGB32F, RGB9E5, B10G11R11 or BC6H, unsigned or signed, decoded at cook time by `src/cook/formats/bc6h.rs`, stored plain, Zstandard or zlib, `KTXorientation` `rd` or none; with `prefiltered true` every level, its own mip chain already GGX-prefiltered (cmgen's, IBLBaker's, the glTF IBL sampler's), taken as the map's levels by `bake::from_prefiltered`: level m read at lod m/4 · (L − 1) of the file's L levels, a chain of five texel for texel, at the file's own size, no `samples`) or its `faces` (six square `.hdr`/`.exr` images of one size, in that order); read beside the file, else under `assets/`; part of the environment like a terrain's heightmap: not cooked or listed on its own, their bytes in the stamp) prefiltered by og-render's `bake` (`bake::bake`, or `bake::bake_cube` for a cube: the same radiance cube, read from the source's box-filtered level of at most twice the bake's face): `size` (level 0's face, a power of two 16..2048, default 128), `intensity` (default 1), `rotation` (degrees about +y, what was at +x moving towards -z; default 0) and `samples` (GGX samples per texel, 16..4096, default 256) into five cube levels of RGB9E5 texels, level m GGX-prefiltered at roughness m/4 (og-render's IBL convention; level 0 the radiance), and the diffuse irradiance as nine order-2 harmonics. fx-showcase's 256 × 128 studio at size 128 cooks in 0.52 s (a dev build, six threads) into a 132 KB blob (511 KiB of texels, zlib'd to 128 KiB). Loaded by `og.load.environment` (`Bundle::environment`), lit by through `og.Environment`. The studio given as a 64² cube (RGBA16F) bakes within 0.2 % mean per texel and 0.17 % in irradiance of the equirectangular studio it was cut from (`tests/environment.rs`). |
| `file.terrain` in `content/**/*.ts` | `<module>/<content-id>.ogb` (`og.Terrain`) | Checked image, evaluated heightfield or seeded noise, processed by the existing terrain kernels and ordered brush/stamp edits. Explicit module-relative image/control/hole inputs enter the cache key. Texture/array/model/preserve operands are canonical refs. Compressed terrain tiles retain controls, layers, holes, horizon, LOD and collision data. Terrain, navigation and projected scatter share one checked surface; no author terrain parser or generate wrapper remains. |
| `file.navmesh` in `content/**/*.ts` | `<module>/<content-id>.ogb` (`og.NavMesh`) | Captured scene/model geometry, explicit boxes and shared terrain surfaces feed the native bake. Model placement, scene collider approximations and dynamic exclusion are retained. Terrain sampling uses renderer diagonals, holes and optional bounds/stride. Agent profiles, ordered discriminated areas and unique full-width link IDs retain their runtime blob layout. Planning and execution never reopen geometry author files; no author navigation text parser remains. |
| checked TS `file.look` | `<module>/<content-id>.ogb` (`og.Look`) | Checked reflected resource patches, composed before asset discovery and conformance. Built-in preset slugs are explicit bases; project bases and resource handles use typed canonical refs. Embedded TS preset, mood and variant layers use the same closed evaluator. Cycles and chains deeper than eight are refused; fields, enums, ranges, writable state and target kinds are validated. Colours accept linear triples or sRGB hex. The native `data` section stores flattened values and the resolved chain, never author text. Startup comes from og-project.json; `.og/cook/look.json` captures the successful startup look, palettes and SHA384 input fingerprints for `readLook`. No look author parser, serializer, extension alias or same-slug project fallback remains. |
| palette `.hex`, GIMP `.gpl` | `<path>.ogb` (`og.Palette`) | Up to 256 sRGB colours in file order: `.hex` one `rrggbb` per line (`#` optional, blank lines and `;` comments skipped), `.gpl` after its `GIMP Palette` header (`Name:`/`Columns:` and `#` lines skipped, `r g b [name]` per colour). Loaded by `og.load.palette` (`Bundle::palette`) for `Stylise.palette`. |

### Built-in look assets

The engine ships a few assets a look can name without the game owning a
copy (`BUILTINS` in `src/cook/systems/look.rs`, compiled into the cook from
`templates/looks/assets/`, made by `generate.ts` there and CC0):

| Name in a look | Bundle id | What it is |
|---|---|---|
| `"@canvas/paper"` | `@canvas/paper.png` | a fine, light paper grain with scattered fibres (256², 39 KB) |
| `"@canvas/linen"` | `@canvas/linen.png` | a rough woven canvas, a plain weave of 4 px threads (256², 48 KB) |
| `"@canvas/watercolor"` | `@canvas/watercolor.png` | cold-pressed watercolour paper: soft bumps, shallow pits (256², 37 KB) |
| `"@palettes/lcd2"` | `@palettes/lcd2.hex` | two greens of a monochrome LCD |
| `"@palettes/handheld4"` | `@palettes/handheld4.hex` | four greens of a green-screen handheld |
| `"@palettes/dusk16"` | `@palettes/dusk16.hex` | sixteen dusk colours and a cool sea ramp |
| `"@palettes/bright32"` | `@palettes/bright32.hex` | thirty-two bright colours for 16-bit-era pixel art |
| `"@palettes/1bit"` | `@palettes/1bit.hex` | black and white |

The canvases are tileable grey (`Stylise.paintCanvas` reads the red channel
as the grain), the palettes `.hex`. A handle field of a look
(`Stylise.paintCanvas "@canvas/paper"`, `Stylise.palette
"@palettes/handheld4"`; the extension may be left out), itself or through
the looks it extends, names one: the cooked look stores the bundle id, and
`cook_project` adds every built-in its looks name to the bundle under that
id as a texture or palette source of its own (once, whatever names it), so
the runtime loads it like any other asset and `og.load.texture("@canvas/paper.png")`
finds it too in a game whose look names it. A built-in no look names is not
cooked. An `@` name that is not a built-in is refused with the list of the
kind's built-ins; a game's own path under `assets/` starting with `@` is
refused (the namespace is the engine's). Tests: `look_builtins.rs`, and
`cook::look`'s `the_built_ins_are_the_template_assets` (the table lists the
directory, PNGs within 64 KB, palettes of their named size).

### Art conformance

The cook makes the assets meet the project's art contract (docs/art.md
§2.3): the canonical look ID og-project.json names, flattened,
its art blocks (`cook::look::contract`). `src/cook/conform/` holds one pass
per kind of asset, each called once from `cook_project` before planning;
2D textures are `conform/texture2d.rs` (3D textures, models and clips are
`conform/mesh.rs`, stream A3). A pass never writes a source: it rewrites
the in-memory sources the rest of the cook reads (the conformed image in
`Source::conformed`, a sheet's re-laid-out frames or tiles in its text,
each extra output as a source of its own), so stamps, the manifest and the
shared texture cache see a conformed asset as any other, its stamp carrying
the contract's hash (the look's text, the palette's and canvas's bytes, the
pass's version). Without a look, or with a look whose art blocks are all at
zero, nothing is conformed and `cooked/` is byte-identical to a cook
without the pass (`tests/conform_2d.rs`).

**Which textures are 2D.** Those a flipbook or a tile set names (their
cells are the frames or the tiles), those a 2D rig, a UI style (`atlas`,
`icons`) or a map's own embedded tileset names (a layout the cook cannot
move: palette and 1-bit alpha only, no resample, no line), and of the
rest, `kind pixel` textures and, in a 2D project (`create { form "2d" }`), every PNG or
JPEG. An image's import settings say `conform sprite` or `conform ui` for
one the cook would not find (a HUD image in a 3D game), and `conform off`
to keep it as drawn. Normal and linear textures are data and never
conformed; `.ktx2` sources are cooked as they are.

**What, cell by cell, in order** (each step only when the contract asks):

1. *Painted 2D*: `Stylise`'s paint fields in the texture's texels
   (Kuwahara with `paintAnisotropy`, `paintBleed`, `paintEdgeDarkening`,
   the canvas grain at `paintCanvasScale` canvas texels per texel), the
   frame's paint pass ported (`texture2d/paint.rs`). Only with an art
   contract: a look that sets the paint fields and no art block keeps them
   a frame treatment.
2. *The pixel grid* (`ArtPixels` sets `unit`, `logicalHeight` or
   `spriteSizes`): the cell size is, of the candidate longer sides not past
   the cell's own (`spriteSizes`; a tile set's tiles `unit`; an import's
   `conform { size N }`), the smallest whose downscale draws the cell back
   within 0.02 mean Oklab distance of the largest's, so art drawn at 2x or
   3x finds its own grid and detail no smaller size holds takes the
   largest. The downscale is a dominant-colour box filter
   (`texture2d/resample.rs`): every source texel counts for the one target
   texel its centre falls in; the target takes the colour most of its
   box's opaque texels share (texels within 8 per channel count as one,
   their mean taken), and is opaque when at least half its box is. Never
   upscaled. Alpha becomes 1 bit (128 and up opaque; a clear texel is
   0 0 0 0). A sheet's cells keep their grid (margins and spacing scaled,
   a spacing at least 1) or, off a grid, pack in rows one texel apart; the
   flipbook's frames and size and the tile set's tile size, margin,
   spacing and colliders are written to follow. A cell whose sides are not
   of `spriteSizes` is a warning in the report.
3. *The palette* (`ArtPalette`): with `colors` and `strict`, every opaque
   texel goes to the nearest colour by Oklab distance (fixed point, so the
   same on every machine), mixed with the next nearest where it falls
   between them by the palette's `dither` (the frame's thresholds:
   `bayer2/4/8`, the engine's blue-noise tile, interleaved gradient noise;
   read from each cell's corner, so a frame carries its pattern); then
   `rampSteps` keeps the most used shades of each hue family (greys, and
   the twelve hues of the colour wheel by nearest Oklch hue) and
   `maxColors` the most used overall, and the texels map again onto what
   is kept. Tiles keep off `accents`. Without `strict`, `colors` holds the
   hues only (each texel takes its nearest colour's hue, its own lightness
   and chroma). Without `colors`, `maxColors` and `rampSteps` make the
   asset's own palette: its colours exactly when it has few enough, else a
   median cut in Oklab; ramps merge their two nearest shades until they
   fit.
4. *The line* (`ArtLine.width`, `color`, `place`, `brush`): drawn into each
   cell, never across into a neighbour, by a port of Pixelorama's
   OutlineInline (`texture2d/outline.rs`, MIT, `third_party/pixelorama/`):
   a clear texel (outside) or an opaque one (inside; a sprite cell's edge
   counts as clear, a tile's does not, so tiles never seam) takes the line when a texel of the other kind lies under the
   brush round it, a diamond (|dx| + |dy| <= width), a circle (radius
   width + 0.5) or a square of the line's width; `both` draws the larger
   half inside. With a palette the line takes its nearest colour (an
   asset's own palette gains the line's colour).
5. *Rotations* (`ArtPixels.rotation prerotated`, `rotationSteps` N of 2 or
   more): each cell before its line, turned by k x 360 / N degrees
   counter-clockwise as seen, by a port of torcado's cleanEdge
   (`texture2d/cleanedge.rs`, MIT, `third_party/cleanedge/`), which never
   blends, then lined.

**What it writes beside the texture** (each an output of its own,
`og.load.texture` / `og.load.palette` by id):

| Id | When | Format |
|---|---|---|
| `<id>` | conformed | the conformed image; `kind pixel` (RGBA8, lossless, no mips) when the pixel grid or a palette applied, else its own kind |
| `<id>#index` | a palette mapped every texel (strict `colors`, or the asset's own palette) and holds at most 255 colours | **the index texture**, for runtime palette swaps (docs/art.md §2.4): the conformed texture's size and layout; `kind pixel` (RGBA8 sRGB, sampled nearest); RGB is the conformed colour (so it draws as the texture does), **alpha is the palette index plus one**, 0 a clear texel. Alpha is never sRGB-decoded, so a shader reads `index = round(a * 255) - 1` exactly. The palette is `ArtPalette.colors` in file order, or `<id>#palette` |
| `<id>#palette` | the index is to the asset's own palette | that palette (`og.Palette`), sorted dark to light by Oklab L, the line's colour last when it was added |
| `<id>#rotations` | `rotation prerotated` | one row per cell (in the sheet's frame order; one row for a sprite), N columns, column k turned by k x 360 / N degrees counter-clockwise; every cell `w` x `h` texels: the turned cell's diagonal, rounded up to the parity of the cell's own side, the art centred; `kind pixel` |
| `<id>#rotations-index` | rotations and an index | the rotations' index texture, as `#index` |

**Opt-outs and the report.** `conform off` in an image's import settings,
or `conform: {kind: 'off'}` in a flipbook, tile set or 2D rig TS declaration (its texture), and
project.ogt's block

```text
conform {
  off "sprites/photos/*" "ui/logo.png"   # asset ids; * within a path segment, ** across, a trailing / a folder
}
```

keep an asset as drawn. `Report::conform` lists every asset the contract
changed (with what: sizes, colours before and after, the line, the extra
outputs), every one it left as it was and every opt-out with its reason;
each is also a note on its asset. `og cook --report` prints them, the
Markdown report (`--report out.md`) tables them, `--json` and
`cook-report.json` carry them as `conform`.

### Import settings

```ts
import {file, ref} from '@origingame/engine/author';
export default file.import({
  kind: 'model', source: 'assets/models/hero.glb',
  front: '-x', up: '+y', streamTextures: true,
  animation: {rate: 30, rig: ref('base/rigs/strider'),
    libraries: [ref('base/anims/humanoid-clips')], footIk: true,
    events: [{clip: 'walk', marker: 'footL', time: 0.26}]},
});
```

`og api show file:import` is the generated schema for all six native import
variants. Texture controls include `usage`, `stream`, explicit `normal` and
`sway` sources and `conform`; sound uses `stream` and `chunk`; font uses
`pixel`; model carries processing, fracture, orientation, VOX and animation
controls. Map imports use canonical `ref` bindings for `tileset`, `tilesets`
and embedded-image `textures`, never guessed sibling paths. Content entry
identity supplies the cooked ID. Shared settings are ordinary TS imports;
OGT sidecars, directory defaults and their merge/parser path are removed.

### Scenes

```ts
import { file, ref } from "@origingame/engine/author";

export default file.scene({
  name: "Main",
  entities: [{
    name: "ship",
    model: ref("base/models/dropship"),
    components: { "og.Transform": { position: [0, 1.5, 0] } },
    children: [{ name: "light" }],
  }],
});
```

Project-listed pure TS schema modules supply game component/resource records;
engine records come from the IDL. Every authored field is checked against
that bound schema and omitted fields retain canonical record defaults.
Unknown records are refused, never inferred from numeric values. Each packed
table carries its reflected layout. Content references stay as typed
relocations until the runtime calls the registered column loader. The manifest
records direct content dependencies so runtime-bundle fetches include their
transitive files before spawn. Inspector/Tune changes memory only; explicit
checked TS edits belong to `og set`.

### Streaming groups

A scene's `streams: { name: { assets: [ref("module/id")] } }` declarations (`src/cook/pipeline/stream.rs`)
group glTF models without skins or morph targets, textures, texture
arrays, flipbooks (an `array true` sheet's `#array` and its layers go with
it), sounds (not chunked), environment maps, palettes, looks and colour
LUTs (`.cube`); any other kind is refused naming the scene. Each group's
outputs are packed into `cooked/stream/<name>.ogb`, and the manifest lists
the group with the memory its assets take once loaded (`memory_of`: a
texture's RGBA8 with mips, a model's vertices and indices, a sound's
samples, an environment map's RGB9E5 texels, a palette's 256 x 1 RGBA8, a
LUT's 32³ RGBA8). The runtime loads a grouped texture array, flipbook,
environment map, palette, look or LUT once its group is resident and lets
it go when the group unloads (og-runtime's `stream`): a sheet's array is
kept while a sprite of the sheet draws from it, its sprites draw nothing
while it is away, and they follow it back under its new renderer handle
(og-runtime's `tests/stream_release.rs`).

A particle emitter (`type particles`, og-bundle's `src/fx.rs`) may name a texture
array instead of a texture: `array "<id>" [<layers>] [off | life <cycles>
| fps <n> | random]` draws a sprite-mode emitter's particles from the
array's layers (by age, at a rate, or one at random per particle), and
`round true` gives them the soft dot's round edge; the cook checks the
array exists like a texture.

## Formats

The blob formats (`format.rs`) and how textures reach the GPU are
og-bundle's, with the loaders that read them
(`crates/og-bundle/README.md` §Formats); how animation channels hold
their samples is og-anim's (`cooked/anim_codec.rs`,
`crates/og-anim/README.md` §Cooked samples). og-cook writes them and
re-exports the modules under the same paths (`og_cook::format`,
`og_cook::runtime`, `og_cook::anim_codec`, ...).

### Asset ids, the manifest and stamps

An asset id is the path under `assets/` (`models/ship.glb`; loaders accept an
`assets/` prefix); a glTF's textures are `models/ship.glb#tex<n>` in
first-use order over its materials. The output file is the id with `.ogb`
appended (`#` becomes `.`).

`cooked/.cook.ogt` (an `.ogt` document of type `cook`) records the cooker,
the option text of each output kind, and per source: kind, source hash, size,
and per output: id, type, file, **stamp**, hash and size of the bytes. The
runtime reads it as its index: `Bundle::blob` refuses a blob whose type hash,
schema version or cook stamp differs from it, with the path and both stamps.

A stamp is FNV-1a 64 (`og_core::hash`) over the revisions of the areas
whose code writes that output (`format::Area`, `format::revisions`: the
blob container's always, then e.g. Texture for a texture, Model and Texture
for a model's LOD surfaces) and exactly what that output depends on. A
change to what an area writes bumps that area's `Area::revision` in the
same commit, and only that area's outputs cook again; the package version
is in no stamp and no cache key, so a release with no revision moved keeps
every cooked file and every cached result:

| Output | Stamp inputs |
|---|---|
| model | mesh options (every one its blob follows: LODs, clusters, whether it carries a raycast BVH, which `og cook` decides from the game's code, a collider, an impostor), the glTF bytes (and the files it references), its texture ids |
| animation | mesh options (it re-cooks the primitives for their vertex order), the animation settings, the glTF bytes, and with `library` or `rig` the bytes of the rig file, each library, its import settings and its rig |
| rig | the file |
| look | the file, each base look's id and text hash, and whether each asset it names exists |
| palette | the file |
| glTF texture | texture options and encoder, the image's bytes, the texture kind |
| texture | texture options and encoder, the file and its import settings, the kind |
| texture array, and each layer | texture options and encoder, the file, every layer image's bytes (and the layer's index) |
| sound | the file, its import settings, and the chunk count it is cut into |
| sound chunk | the same, and the chunk's index |
| font | the effective font settings, the file and its import settings |
| scene | the file, the schema set, each referenced id and whether it exists |

So a material edit in a glTF re-cooks its model blob and nothing else (its
textures keep their stamps); a texture option change re-cooks textures only;
a new cooker version re-cooks everything.

### Cook times

Beside the manifest, `cooked/.cook-times.ogt` (type `cook-times`, `asset
"<source>" <ms>` lines, `CookTimes`) records what each asset's last cook
took: its outputs' times summed, to the microsecond. An up-to-date asset
keeps the time of the cook that made it, so the record describes the whole
bundle however little the last run cooked; the file is written only when
an asset cooked or went. The manifest itself holds no times, so it stays
the same bytes for the same sources (§Determinism); the times are the one
file two cooks write differently, and `og build` leaves them out of the
export. `Report::slowest(n)` and `AssetReport::cook_ms` read them:

- `og cook --report [out.md]` prints the ten slowest assets by their last
  cook (and writes the whole report, which leads with the same table, to
  `out.md`);
- `og cook --watch [--exit-after N] [--json]` cooks, then cooks again on
  every change under `assets/` (only what the change reaches), printing
  each cooked asset with its time and the run's time against the 300 ms
  reload budget (`--json`: one line per cook, `{"event":"cook","changed",
  "assets":[{source,ms,outputs}],"ms","withinBudget"}`); `og dev` prints
  the same lines after its re-cooks.

## Options

`CookOptions { threads, mesh: MeshOptions, texture: TextureOptions, font: FontSettings, schemas }`.
Defaults: LOD levels 3, ratio 0.5, max error 0.05, overdraw 1.05, tangents on;
ETC1S quality 128, UASTC level 1, RDO off, Zstandard on, mips on; fonts
outline fonts. Thread count never changes the output.

### Fonts

A font blob is the face's metrics and its file, whole (and a pixel font's
grid): no atlas is cooked. Text is laid out from the file at run time
(og-text's Parley layout) and every glyph rastered at the size it is
drawn, COLR glyphs in colour, so a CJK face costs its file's size and a
character a game never declared draws like any other. OpenType features
and palettes are CSS's (`font-feature-settings`, `font-variant-*`,
`font-palette`). A `.riv`'s fonts cook the same way, each checked for the
characters its runs show.

## Determinism

The same sources, options and **cooker build** give byte-identical `cooked/`
whatever the thread count and the machine's load (`cooking_twice_gives_identical_bytes`
cooks the template on all cores and on one thread;
`every_thread_count_cooks_the_same_bytes` cooks a small copy on 1, 2 and 8
threads, three cooks at a time: once each in the quick and functional
lanes, ten times each under `--all`). Both compare every file, with
`.cook-times.ogt`, the wall-clock record of §Cook times, compared by its
assets only: the manifest holds no times, so it is compared byte for byte.
Results land by plan index, the font's glyphs by index, and no hash map's
order reaches a file.

basisu's UASTC RDO pass (`uastc_rdo: Some(_)`, off by default) hashes a key
struct with four uninitialised padding bytes, so a lookup could miss and
the chosen blocks depend on what the thread ran before. Upstream fixed it
(`selector_bitsequence::m_pad`, zeroed); until a basis-universal-sys
release bundles that fix, the workspace patches in
`third_party/basis-universal-sys` (the crate with that one change, only the
files its build compiles). The same crate holds the RDO pass to one job:
basisu splits it into up to four by its thread count, each matching
selectors within its own blocks, so the bytes followed the count. With one
job a UASTC texture encoded on several threads (`execute` gives a large
encode the cores other outputs leave idle; each block is encoded alone) is
byte for byte the one a single thread encodes. ETC1S encodes on one thread. `-ftrivial-auto-var-init=zero` is no
substitute: gcc leaves the temporaries the map is called with
uninitialised.

BasisU uses non-contracting scalar arithmetic: `-ffp-contract=off` with
GCC/Clang, `/fp:precise` without `/fp:contract` with MSVC. Otherwise an ARM64
FMA rounds once where x86-64 rounds a multiply and add separately, changing
the selected UASTC blocks. The unchanged UASTC mip goldens check the same
bytes on Linux x86-64 and macOS ARM64. The encoder identity includes this
policy, invalidating older texture results in both the project and shared
cache. The native build still leaves BasisU's optional SSE path disabled.

Limits:

- C++ encoders use the profile's optimisation level; an `-O0` basisu
  writes different UASTC bytes. The workspace pins them at `opt-level = 3`
  in the dev profile, so debug and release agree. Meshoptimizer and fdsm
  retain their compiler's floating-point defaults: the BasisU contraction
  policy does not establish cross-host byte identity for every cooked
  format. The general guarantee remains per cooker build.
- `toktx` output depends on its version (toktx 4 prints it on stderr),
  which is part of every texture stamp.

## Tests

`crates/og-cook/tests/template.rs` (integration, on the template: the
cook's determinism and one end-to-end load), `tests/shader.rs` and
`tests/scene_string.rs` (validation), and the unit tests that check the
shipped presets, built-ins, themes and string tables against the cook;
`crates/og-blob/tests/blob.rs`.
Run them in release: a debug build compiles basisu at `-O0` and one cook
takes about a minute.

```sh
cargo test --release -p og-blob -p og-cook
cargo check -p og-bundle --target wasm32-unknown-unknown   # the loaders, no C
cargo run --release -p og-cook --bin og-cook -- templates/game --report report.md   # cooks into templates/game/cooked
```

| Test | Asserts |
|---|---|
| `every_thread_count_cooks_the_same_bytes` | the template made small (trickster.glb out, the two textures at 128², 13 outputs of every kind the template has, both texture encoders, the font's own glyph pool) cooked on 1, 2 and 8 threads with the three series running at once (one cook a series in the quick and functional lanes, ten under `--all`, `OG_GATE_ALL`), so each cook shares the machine with two others: every file of every cook is the first one-thread cook's bytes, the cook times masked as above |
| `sounds_fonts_and_scenes_load_as_authored` | QOA SNR > 20 dB and the WAV's length; the font's file whole with its metrics and no atlas; scene entities, nesting, instances, the Transform rows' values |

## `.vox` frame rate

`voxFps` gives a `.vox` animation's rate, because the file carries none:
MagicaVoxel's format (`MagicaVoxel-file-format-vox.txt` and its extension,
ephtracy/voxel-model) defines no rate in any chunk; `nTRN` and `nSHP`
frames carry only a frame index (`_f`), and the one animation setting the
editor writes is the playback range (`META`'s `_anim_range`, which the cook
reads), as ogt_vox (opengametools, whose reading this cook follows) does.
