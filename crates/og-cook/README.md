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

1. **Walk** `assets/` (sorted, dot-files skipped). By extension: `.glb`/`.gltf`/`.vox`/`.fbx`/`.obj`
   models, `.tmj` (Tiled) and `.ldtk` (LDtk) maps, `.png`/`.jpg`/`.jpeg`/`.webp`/`.tga`/`.psd`/`.dds`/`.ktx2` textures, `.wav`/`.mp3`/`.ogg`/`.flac` sounds, `.ttf`/`.otf` fonts,
   `.ogt` scenes, particle emitters, tilesets, tilemaps, terrains, environments and looks (by `type`), `.hex` and `.gpl` palettes. `<file>.ogt` of `type import` holds `<file>`'s import
   settings. Files a `.gltf` references are part of that model, as are the textures an FBX or OBJ names and an OBJ's `.mtl`. Anything else
   (licences, notes) is listed as ignored.
2. **Plan**: every output gets a cook stamp (below). An output whose stamp the
   manifest already holds, with its file present, its bytes' hash unchanged
   and its header holding the type hash and schema version the loader
   expects today (`format::expected_header`), is up to date and not cooked.
   The header check catches a schema that changed without a
   `COOKER_VERSION` bump: the manifest and the file agree, but the loader
   would refuse the blob ("holds type ..., expected ...").
3. **Cook** the stale outputs on a thread pool (`CookOptions::threads`, 0 =
   all cores); results are collected by plan index, so scheduling never
   changes the bytes.
4. **Write** outputs whose bytes changed (unchanged files are not touched),
   delete outputs of removed or failed sources, write
   `cooked/.cook.ogt` when it changed. Return the `Report`.

### Shared cook results

Native cooks use BLAKE3 action keys and verified contents in
`$HOME/.cache/origingame/cook-v1/{ac,cas}`. Without HOME, Windows uses
`$LOCALAPPDATA/origingame/cook-v1`. Texture and sound keys cover their source
bytes, settings and output stamp. Other output keys also cover project inputs
and ids. Prefracture keys cover the model and discovered dependency contents,
not its pathname. Video work keeps its existing encoder path.

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
as a scanned source's entry. Changed projects are planned again, with unchanged
scanned sources read from the content store when available. Video projects and
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
| PNG/JPEG/WebP/TGA/PSD/DDS | `<path>.ogb` (`og.Texture`) | Decoded to RGBA8 (`src/cook/formats/texture.rs` by its bytes: WebP lossy and lossless; TGA; DDS level 0 of BC1 to BC7 or uncompressed, image_dds; PSD/PSB the flattened composite Photoshop stores beside the layers, `src/cook/formats/psd.rs`: RGB, greyscale, CMYK, indexed and bitmap at 1 to 32 bits, raw, PackBits or zip, its transparency only when the layer count says so), then KTX2: colour → ETC1S/BasisLZ, sRGB, quality 128; `kind normal` → UASTC level 1 + RDO 1.0 + Zstandard, linear, tuned for normal maps, x y z in RGB and y again in alpha (Basis transcoders take BC5's and EAC RG11's second channel from alpha; `has_alpha` 1 says a map carries it, and the settings string's "normal y in alpha" recooks older maps); `kind linear` → the same without the normal-map tuning. Full mip chain. `kind pixel` → RGBA8 + zlib, sRGB, lossless, one level; the runtime samples it nearest (sprite sheets, tilesets). With `texture { stream true }` (a model's textures: `model { stream_textures true }`) a texture is encoded once and cooked as two outputs: `<path>.ogb` (id `<path>`) holds its mip tail, the whole chain's levels from the first whose longest side is at most 256 down, byte for byte, with a raw section `mips` giving the whole chain's level-0 width, height and level count; `<path>.mips.ogb` (id `<path>#mips`, og-bundle's `mips_id`) holds the whole chain, the ordinary blob. The runtime draws the tail and reads `#mips` when the texture is big on screen; the web host does not fetch `#mips` at boot and a publish build's service worker caches it when fetched. A texture whose header says it is at most 256 on its longest side, or in a streaming group (whose pack holds its whole chain), is cooked whole, with a note. In a glTF, base colour and emissive are colour, metallic-roughness and occlusion linear, normal normal. |
| KTX2 | `<path>.ogb` (`og.Texture`) | `src/cook/formats/ktx2_source.rs`. Basis ETC1S or UASTC, or RGBA8 (none or zlib), its `KTXorientation` and `KTXswizzle` honoured: a file that is `rd` and `rgba` (or says neither), of a colour or linear kind matching its transfer function, is taken byte for byte; any other is decoded (Basis through the native transcoder), flipped to rows top to bottom and columns left to right, swizzled (`r g b a 0 1` per channel) and cooked again as an image of its kind (`texture { kind }`, else sRGB colour or linear). The report says which. `toktx`'s own output goes through the same check. |
| texture array `.ogt` (`type texture-array`) | `<path>.ogb` (`og.TextureArray`) + `<path>.layer<n>.ogb` (`og.Texture`, id `<path>#layer<n>`) per layer | `src/cook/formats/texture_array.rs`: `kind color|normal|linear|pixel` and 1–256 `layer "<file>"` images (beside the file, else under `assets/`) of one size (a layer of another size is refused naming both); each layer cooked as a texture of the kind; the array blob holds the size, kind, level count and the layers' ids. The runtime uploads the layers as one `D2Array` texture (`og.load.textureArray`, og-render's `RenderScene::add_texture_array`). World and overlay sprites (`SpriteDesc.array`, `og.Sprite.array`), `array true` flipbooks and 2D particle emitters (an emitter's `array` line, og-bundle's `src/fx.rs`) draw from it, one draw per array whatever the layers; so do lit materials (`mapArray`, `normalMapArray`) and terrain layers (`layers`, `layerNormals`). |
| WAV, MP3, Ogg Vorbis, FLAC | `<path>.ogb` (`og.Sound`), and for a sound cooked in chunks `<path>.chunk<n>.ogb` (`og.SoundChunk`, id `<path>#chunk<n>`) | A WAV's 8–32-bit integer or 32-bit float PCM, 1–8 channels, to 16-bit; an MP3, Ogg Vorbis or FLAC decoded to 16-bit by symphonia (pure Rust, the first audio track, encoder delay and padding trimmed; Ogg Opus is refused); then QOA (in-crate encoder: 3.2 bits per sample). With `sound { stream true }`, or when the QOA file would pass 1 MiB (27 s of 48 kHz stereo) unless `stream false`, the file is encoded once and cut between QOA frames into chunks of about `chunk` seconds (default 4, whole frames): the sound's blob holds the QOA header and the first chunk, each other chunk a blob of its own, which the runtime reads ahead of the playheads and drops behind them (og-audio's README §Chunked sounds). A QOA frame carries its predictor's state, so the chunks decode to the whole file's samples exactly. A chunked sound in a stream group puts its head in the group's pack; its other chunks stay files of their own (never grouped), read ahead of the playheads once the group is resident. |
| TTF/OTF | `<path>.ogb` (`og.Font`) | The import settings' charset, rendered by fdsm as MTSDF (RGB multi-channel, A true SDF) at `em_px` with `px_range`, generated in f32 and passed through msdfgen's error correction (fdsm's `correct_error_mtsdf`: edge priority, distance checks at edges, before the sign correction) so channels that would interpolate into a false edge at a sharp corner become single-channel; shelf-packed into pages, each page a lossless KTX2 (RGBA8 + zlib); a glyph table (advance, plane bounds in em, atlas rectangle), ascender/descender/line gap and a kerning table: the pair adjustments of every `kern` feature's GPOS lookups (pair positioning formats 1 and 2, every script's; the first subtable of a lookup that takes a pair decides it, lookups add), else the legacy `kern` table, between the cooked characters, in em. A charset with a shaped script (Arabic, Devanagari and og-i18n's other `needs_shaping` scripts) adds every glyph the font's substitutions reach (joining forms, ligatures, conjuncts) by glyph id (keyed 0x110000 + id, past Unicode, so a font's own private-use characters never alias one), and the font file; so does `runtime true`, for characters outside the charset that og-ui renders from the outlines when a text needs them. |
| rig `.ogt` (`type rig`) | `<path>.ogb` (`og.Rig`) | A humanoid map (og-anim's README §Retargeting): `scheme` (auto, mixamo, vrm, unity, rigify, ue, blender, soma) and standard bones named outright or `none`; validated and stored as canonical text, loaded with `og.load.rig`. A model's `animation { rig }` names it for the cook's retargeting. |
| animation library (a glTF named by another model's `animation { library }`) | into the naming model's `#anim` | The library is cooked like any glTF (its own `#anim`, loadable with `og.load.animation`); the naming model's `#anim` gets the library's clips after its own, each rewritten by `og_anim::retarget` from the library's skeleton (its humanoid map from its own `animation { rig }` or detected) onto the model's (its `rig` or detected): rest corrections, the model's bone lengths, the hips' travel scaled by the leg ratio, the legs solved for the feet unless `footIk false`. A clip the model has of its own wins over a library's of the same name (a note says so); a library naming libraries is refused. Then each `mirror "<clip>" ["<name>"]`: the clip (the model's own or a library's, retargeted) mirrored on the model's skeleton by `og_anim::mirror` (sides paired by name, reflected in x = 0), written under the name (default `<clip>.mirrored`), so a mirrored clip costs nothing at run time; a clip it lacks, a name already taken or named twice is refused. |
| flipbook `.ogt` (`type flipbook`) | `<path>.ogb` (`og.Flipbook`) | The canonical sheet text (og-anim's README): grid and `frame` lines, an `atlas` (Aseprite frames and tags, TexturePacker frames and pivots) resolved into frames and clips, the texture's size read from the image, paths relative to the sheet when the file exists. The stamp covers the texture's and the atlas's bytes. With `array true` the frames, cut out of the texture, also cook into a texture array: `<path>#array` (`og.TextureArray`) with a layer `<path>#array#layer<n>` (`og.Texture`, pixel art under `filter nearest`, colour otherwise) per frame, frame `n` being layer `n`; every frame must then have one size (two sizes are refused naming both) and lie inside the texture, at most 256 of them. The runtime draws an `og.Flipbook` of such a sheet from the array (one draw per sheet whichever frames its sprites show). A sheet with `source` lines (animated GIF, WebP or APNG; `src/cook/systems/animated.rs`) is expanded before anything is hashed: its frames decoded, composited, keyed, aligned, cropped to one size, shared when identical and packed into a texture source `<path>#sheet` (`og.Texture`, pixel art under `filter nearest`), its text replaced by the canonical sheet over that texture; the files it names are consumed. |
| scene `.ogt` | `<path>.ogb` (`og.Scene`) | Entities depth-first (parent before child), `model` references checked against the project's assets, one packed table per component type keyed by its reflected name. |
| tileset `.ogt` (`type tileset`) | `<path>.ogb` (`og.Tileset`) | Parsed and validated by og-tilemap (grid, solid and partial collision, animated tiles, blob47/bitmask16 terrains); its `texture` must be an asset. The canonical text is stored and parsed again at load. |
| tilemap `.ogt` (`type tilemap`), Tiled `.tmj`, LDtk `.ldtk` | `<path>.ogb` (`og.Tilemap`) | One `og_tilemap::TilemapAsset` whatever the source (`src/cook/systems/tilemap.rs`, native only). **Tiled**: orthogonal, finite maps; layer data as a JSON array or uncompressed base64 (zlib/gzip/zstd refused: save as CSV or Base64); flip bits kept (diagonal refused); `offsetx/y`, `parallaxx/y`, `opacity`, `visible`, `repeatx/y`, group layers flattened; custom properties `collision` (bool), `z` (int), `autotile` (a terrain of the layer's tileset: nonzero gids mean filled), `repeatx`/`repeaty`; an external tileset `x.tsx`/`x.tsj` is the sibling `x.ogt` (refused when missing, naming the path), an embedded one (single image) is imported with its tiles' `animation`, first `objectgroup` rectangle (collision) and boolean `solid` property; one tileset per layer; object layers become objects (`type` or `class` as kind, string-valued properties; tile objects anchored at their bottom). **LDtk**: one level (the first or `level`; separate level files refused), every layer on one grid size; `Tiles`/`AutoLayer`/IntGrid auto tiles become tile layers (tile id `t` or from `src`; flips from `f`; one tile per cell, the last of a stack wins), drawn in reverse list order (LDtk's first layer on top); IntGrid values listed in `solid` (default 1) become an invisible collision layer `<name>.collision` over an embedded one-tile solid tileset; a tileset image with a sibling `<stem>.ogt` uses that tileset, else one is embedded from `tileGridSize`, `__cWid`, `__cHei`, `padding`, `spacing`; `Entities` become objects at their top-left (pivot applied), kind the identifier, name the `name` field or the `iid`, fields as properties. Units: `unit` world units per tile width (default 1); object positions and offsets in map-local world units, y up from the top-left corner. Referenced tilesets and images must be assets. |
| environment `.ogt` (`type environment`) | `<path>.ogb` (`og.EnvironmentMap`) | `src/cook/systems/environment.rs`: its `source` (an equirectangular Radiance `.hdr` or OpenEXR `.exr`, linear RGB, twice as wide as tall, decoded by the `image` crate; or a KTX2 cube map, `src/cook/formats/cube_source.rs`: six square faces +x −x +y −y +z −z with rows from the top, level 0 of RGBA16F, RGB16F, RGBA32F, RGB32F, RGB9E5, B10G11R11 or BC6H, unsigned or signed, decoded at cook time by `src/cook/formats/bc6h.rs`, stored plain, Zstandard or zlib, `KTXorientation` `rd` or none; with `prefiltered true` every level, its own mip chain already GGX-prefiltered (cmgen's, IBLBaker's, the glTF IBL sampler's), taken as the map's levels by `bake::from_prefiltered`: level m read at lod m/4 · (L − 1) of the file's L levels, a chain of five texel for texel, at the file's own size, no `samples`) or its `faces` (six square `.hdr`/`.exr` images of one size, in that order); read beside the file, else under `assets/`; part of the environment like a terrain's heightmap: not cooked or listed on its own, their bytes in the stamp) prefiltered by og-render's `bake` (`bake::bake`, or `bake::bake_cube` for a cube: the same radiance cube, read from the source's box-filtered level of at most twice the bake's face): `size` (level 0's face, a power of two 16..2048, default 128), `intensity` (default 1), `rotation` (degrees about +y, what was at +x moving towards -z; default 0) and `samples` (GGX samples per texel, 16..4096, default 256) into five cube levels of RGB9E5 texels, level m GGX-prefiltered at roughness m/4 (og-render's IBL convention; level 0 the radiance), and the diffuse irradiance as nine order-2 harmonics. fx-showcase's 256 × 128 studio at size 128 cooks in 0.52 s (a dev build, six threads) into a 132 KB blob (511 KiB of texels, zlib'd to 128 KiB). Loaded by `og.load.environment` (`Bundle::environment`), lit by through `og.Environment`. The studio given as a 64² cube (RGBA16F) bakes within 0.2 % mean per texel and 0.17 % in irradiance of the equirectangular studio it was cut from (`tests/environment.rs`). |
| terrain `.ogt` (`type terrain`) | `<path>.ogb` (`og.Terrain`) | `src/cook/systems/terrain.rs`: its `heightmap` (a 16- or 8-bit grey PNG, its first channel at 16 bits; or an EXR, its first channel, through the `image` crate's `exr` feature) mapped by `height <min> <max>` (PNG default 0 100; EXR default 0 1, metres as stored), its `size` in metres (default one per sample), rows from -z (the image's top) and columns from -x; cut into `tile` × `tile` sample tiles (default 128; the last ones smaller), each quantised to 16 bits over its own range (finer than a 16-bit source's step over the whole range) and zlib'd; the `control` map (RGBA, one channel per layer) zlib'd; up to four `layer { albedo normal color roughness tile triplanar normalStrength slope slopeFade altitude altitudeFade }`, their textures assets of their own (give a normal map `texture { kind normal }`), or their maps layers of two texture arrays (`layers "<array>.ogt"`, `layerNormals "<array>.ogt"`, assets of their own of `type texture-array`, beside the terrain or under `assets/`; each layer's `albedoLayer` and `normalLayer`, by default its place: the terrain then samples two textures instead of up to eight, og-render's `Terrain::layer_arrays`); `heightBlend`, `castShadows`, `collision`, `lod { spacing ring }`. The heightmap and control map are read beside the terrain file (else under `assets/`) and are the terrain's, not textures: the walk does not cook them, and the terrain's stamp covers their bytes. A navmesh `.ogt`'s `terrain "<file>" { position x y z step n }` bakes the terrain's surface (every `step`th sample, each cell split along the renderer's diagonal, only the cells inside the navmesh's `bounds`) with the agent's slope limit. |
| look `.ogt` (`type look`) | `<path>.ogb` (`og.Look`) | og-bundle's `src/look.rs` (the grammar, merging, hex colours) and `src/cook/systems/look.rs`: a frame's art direction as data. Top-level `name`, `extends`, `ui` (a UI style: a built-in slug or the game's own asset id) and `antialias` (tier, off, msaa4, taa), then one block per resource it sets (`RenderSettings`, `PostEffects`, `Stylise`, `MaterialStyle`, `Fog`, `Sky`, `Environment`, and the art contract's `ArtPalette`, `ArtPixels`, `ArtTexels`, `ArtShape`, `ArtLine`, `ArtMotion`, `ArtCamera`) with its fields by reflected name. `extends "<slug>"` resolves to the game's own `assets/looks/<slug>.ogt` (never the file itself), else the built-in preset `templates/looks/<slug>.ogt` (`PRESETS`, compiled in); `extends "looks/base.ogt"` to another look; chains up to 8 deep, cycles refused; the look is laid over its base field by field, its name its own. Every field is checked against the IDL's records (`og_records::RECORDS`): it exists and is writable, a number is in range and whole for u32/i32, an enum takes one of its variants, a colour is three linear numbers or an sRGB `"#rrggbb"` (stored linear), a handle field (`Texture`, `Lut`, `Palette`, `EnvironmentMap`) names an asset of its kind in the bundle, or one of the engine's built-in assets (below) as `"@canvas/<name>"` / `"@palettes/<name>"`. `PostEffects.flash` and `shakeTrauma`, animated by game code, are refused. The blob holds the flattened canonical text with `extends` listing the chain; the stamp covers every base's text and whether each named asset exists. Loaded by `og.load.look` (`Bundle::look`), applied by `og.render.applyLook` and `GameDesc.look`. project.ogt's `look "looks/<id>.ogt"` names the project's look: the cook reads its art contract (the flattened look's art blocks) before any asset cooks and notes in the look's report what the contract sets; a line naming no look of the bundle is that asset's error, a malformed line fails the cook. |
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
§2.3): the look project.ogt's `look "looks/<id>.ogt"` names, flattened,
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
rest, `kind pixel` textures and, in a `profile "2d"` project, every PNG or
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
or in a flipbook's, tile set's or 2D rig's own `.ogt` (its texture), and
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

```text
type import
version 1
---
conform off            # the art contract leaves this image as drawn; or `conform sprite` / `conform ui`
                       # (its role when the cook would not find it), with an optional block: conform { size 32 }
texture {
  kind normal          # color (default) | normal | linear | pixel (sprite sheets and tilesets: RGBA8 lossless, sRGB, no mips, sampled nearest)
  stream true          # stream the levels above 256 texels: `<id>` holds the mip tail, `<id>#mips` the whole chain (default false; refused with kind pixel)
}
animation {
  rate 30                     # resampling rate for every clip; default: the source's key rate
  event "walk" "footL" 0.26   # a marker: clip, name, seconds (also glTF extras."og.events")
  rig "rigs/strider.ogt"      # this model's humanoid map (`type rig`); default: detected from its node names
  library "anims/humanoid-clips.glb"  # an animation library: its clips retargeted onto this model (repeatable)
  footIk true                 # retargeting solves the legs so planted feet stay planted (default true)
  mirror "turnLeft" "turnRight"  # a clip (its own or a library's) written again mirrored, under that name (default "<clip>.mirrored"; repeatable)
  compress true               # quantised samples and keyframe reduction (default false: f32)
  tolerance 0.0001            # the most compression may move a sampled value (default 1e-4)
}
font {
  charset latin cjk-punct   # ascii | latin | punct | cjk-punct | kana | hangul | arabic | all
  chars "开始游戏"           # extra characters (UI strings)
  em_px 32
  px_range 11               # the field's reach in atlas px; it bounds outlines, glows and weight at 0.5 * px_range * size / em_px - 0.5 CSS px (a third of em_px: a 2 px outline at 16 px; default 4)
  page_size 1024            # the last page is cut to the rows it uses
  runtime true              # ship the font file: characters outside the charset render at run time (default false)
  color true                # COLR v0/v1 colour glyphs drawn in colour onto colour pages (default false; not with runtime)
  color_px 64               # the colour images' pixels per em, 8..=256 (default 64)
}
sound {                     # a .wav, .mp3, .ogg or .flac
  stream true               # cook in chunks: true | false (default: when the QOA file passes 1 MiB)
  chunk 4                   # seconds per chunk, 1..=60 (rounded up to whole 5120-sample QOA frames)
}
model {                     # a MagicaVoxel .vox
  voxScale 0.1              # metres per voxel (default 1)
  voxOrigin bottom          # center (the editor's origin, default) | bottom (the lowest voxel on y = 0)
  voxFps 8                  # an animated .vox's frames per second, 1..=120 (default 10)
  voxClip walk 0 3          # a clip of frames 0 to 3, both shown (repeatable; default one clip `frames` of every frame)
}
model {                     # any model
  stream_textures true      # its glTF textures stream their levels above 256 texels, as `texture { stream true }` (default false)
}
tilemap {                   # a .tmj, .ldtk or tilemap .ogt
  unit 1                    # world units per tile width
  level "Level_0"           # LDtk: the level, by identifier (default the first)
  solid 1 2                 # LDtk: the IntGrid values that are solid (default 1)
  tileset "tiles/x.ogt"     # use this tileset for every tileset of the map
}
```

### Scenes

```text
type scene
version 1
---
name "Main"
entity "ship" {
  model "models/dropship.glb"        # the host spawns the model's default scene under this entity
  Transform {
    position 0 1.5 0
  }
  entity "light" { … }               # children nest
}
```

A component block's entries are fields (numbers, `true`/`false`). With the
component's schema in `CookOptions::schemas` (the `og schema` JSON, which
`og-cook --schemas` reads) fields are checked by name and lane count,
ordered, typed (integers must be integral) and defaulted from the schema;
the key resolves exactly as written. Without a schema the cook infers one
(f32 lanes; every entity must set the same fields), marks the table
`inferred` and the report says so. Each table stores its schema JSON next to
its rows, so a host migrates rows to its registered layout by field name.

### Streaming groups

A scene's `stream "<name>" { asset ... }` blocks (`src/cook/pipeline/stream.rs`)
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

A stamp is FNV-1a 64 (`og_core::hash`) over the cooker version
(`format::COOKER_VERSION`) and exactly what that output depends on:

| Output | Stamp inputs |
|---|---|
| model | mesh options, the glTF bytes (and the files it references), its texture ids |
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
ETC1S quality 128, UASTC level 1, RDO 1.0, Zstandard on, mips on; fonts
`latin`, 32 px/em, range 4, 1024-pixel pages. Thread count never changes the
output.

### CJK fonts

`charset all` on a CJK font cooks to many atlas pages, several times the
TTF's size. Games should declare the characters they show (`chars`), and
add `runtime true` for text they cannot know (a player's name, chat): the
blob then carries the font file and og-ui renders what the charset left
out from its outlines into pages of the font's own
(`crates/og-ui/README.md` §Text).

### Colour glyphs

`color true` draws each cooked character that has a COLR paint (v0
layers, v1 paint graphs: solid, linear, radial and sweep brushes with pad,
repeat and reflect, transforms, glyph and box clips, every composite mode)
into an sRGB image at `color_px` (`crates/og-ui-core/src/colr.rs`: skrifa walks the
graph, font-rs's accumulation rasterizer covers each clip). The images are
shelf-packed onto the blob's `color.<n>` pages, after the field's pages,
and the glyph rows point at them; the field keeps their advances. og-render
draws a colour page (`MsdfFont::distance_range` 0) as sprites, untinted but
for the text's alpha and without glow, outline or shadow, on both hosts.
Only fonts that ask carry colour pages, and an emoji font is its own asset,
loaded when a game loads it. A font none of whose cooked characters is
coloured refuses the setting, as does `runtime true` (run-time glyphs have
no colour). The images are painted with CPAL palette 0, or the one
`palette` names: an index (`palette 2`), or `light` or `dark`, the first
palette the font marks for that background; a palette the font lacks is
refused. One palette is cooked per font: a game that shows the same glyphs
in two palettes imports the font twice (a copy of the file with its own
`.ogt`), so each palette costs its pages only where it is used.

A glyph whose paint uses the text's colour (palette entry 0xFFFF) is
painted twice, with a black foreground and a white one, side by side on the
blob's `foreground.<n>` pages (after the colour pages; the row points at the
black copy, the white one is half the page to the right). og-render mixes
the two channel by channel by the text's colour in sRGB, where the cook
painted them, at the text's alpha: exact for fills, gradients and
Porter-Duff composites, whose result is affine in the foreground. A blend
can bend it: a separable blend that branches on the foreground (overlay
with it as the backdrop, hard-light as the source, soft-light, darken,
lighten, dodge, burn, difference), the foreground multiplied or screened
by itself, a clamping plus, or a hue, saturation, colour or luminosity
blend. The cook paints each such glyph again at three text colours (two
greys and one colour) and warns in the font's note, naming the characters
and the worst difference from the mix in 8-bit levels; the glyph still
draws as the mix. Glyphs that do not use it stay on the colour pages, byte
for byte.

Each colour page and foreground page also gets its glyphs' silhouette as a
distance field (`color-field.<n>`, `foreground-field.<n>`: the exact
Euclidean distance to the alpha's half-coverage edge, range 8 px, twice
the 4 px border round each image, so it never reaches a neighbour). og-runtime adds
each as a font page before the colour pages that draws only the shadows,
glow and outline of a text (never a fill), so they lie under the images.
The fields cost about a tenth of the colour pages (the whole COLR test
font: 67 KB beside 672 KB).

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

basisu's UASTC RDO pass (`uastc_rdo: Some(_)`, the default) hashes a key
struct with four uninitialised padding bytes, so a lookup could miss and
the chosen blocks depend on what the thread ran before. Upstream fixed it
(`selector_bitsequence::m_pad`, zeroed); until a basis-universal-sys
release bundles that fix, the workspace patches in
`third_party/basis-universal-sys` (the crate with that one change, only the
files its build compiles). `-ftrivial-auto-var-init=zero` is no
substitute: gcc leaves the temporaries the map is called with
uninitialised.

Limits:

- The C++ encoders (basisu, meshoptimizer, fdsm) are compiled by `cc` with
  the profile's optimisation level and clang's default `-ffp-contract=on`;
  an `-O0` basisu writes different UASTC bytes, and an arm64 host (FMA) can
  differ from x86-64. The workspace pins them at `opt-level = 3` in the dev
  profile, so debug and release agree. Stamps stay correct (they describe
  inputs), but "same bytes" holds per cooker build, not across hosts;
  cross-host identity would need `-ffp-contract=off` for the C++ crates,
  which is not set.
- `toktx` output depends on its version (toktx 4 prints it on stderr),
  which is part of every texture stamp.

## Tests

`crates/og-cook/tests/template.rs` (integration, on the template; writes
evidence under `$OG_EVIDENCE_DIR`), `crates/og-blob/tests/blob.rs`.
Run them in release: a debug build compiles basisu at `-O0` and one cook
takes about a minute.

```sh
cargo test --release -p og-blob -p og-cook
cargo check -p og-bundle --target wasm32-unknown-unknown   # the loaders, no C
cargo run --release -p og-cook --bin og-cook -- templates/game --report report.md   # cooks into templates/game/cooked
```

| Test | Asserts |
|---|---|
| `cooking_twice_gives_identical_bytes` | two fresh cooks (all cores, one thread) write identical files (the cook times aside: the same assets, their times masked); a difference names every file that differs, each with its size and first differing byte, and the manifest's differing lines; cooked ≤ 60 % of the sources; writes the report and the two hashes |
| `every_thread_count_cooks_the_same_bytes` | the template made small (trickster.glb out, the two textures at 128², the font's charset ASCII and its characters: 13 outputs of every kind the template has, both texture encoders, the font's own glyph pool) cooked on 1, 2 and 8 threads with the three series running at once (one cook a series in the quick and functional lanes, ten under `--all`, `OG_GATE_ALL`), so each cook shares the machine with two others: every file of every cook is the first one-thread cook's bytes, the cook times masked as above |
| `a_gltf_edit_recooks_exactly_its_outputs` | an unchanged project cooks nothing and writes nothing, each asset reporting its recorded cook time and the slowest ten in order; renaming a glTF material re-cooks exactly `models/trickster.glb`, changes exactly its blob, the manifest and the cook times, where only its time moved (to this run's) |
| `a_stale_blob_is_refused_with_both_stamps` | an old blob behind a new manifest is refused with the path and both stamps (`og_core::ErrorKind::Schema` through the API); wrong type and unknown ids are refused |
| `meshes_keep_their_triangles_and_bounds` | per primitive: LOD 0 triangle count equals the source's, decoded vertex bounds within 1 % of the source accessor's, LODs shrink, normals unit length; per model: decoded default-scene bounds within 1 % of og-render's own glTF loader |
| `textures_decode_close_to_their_sources` | transcoded level 0 against the source pixels: ETC1S ≥ 30 dB (glTF base colour ≥ 27 dB), UASTC ≥ 35–38 dB; a row-flipped decode scores worse (orientation) |
| `sounds_fonts_and_scenes_load_as_authored` | QOA SNR > 20 dB and the WAV's length; the charset, the extra characters, inside/outside of a glyph's distance field; scene entities, nesting, instances, the Transform rows' values |
| `animation.rs` (separate file) | every cooked clip sample of `walker.glb` (source rates and 60 Hz) and `trickster.glb` equals an independent glTF sampler within 1e-5; skins, inverse binds, joints, weights and morph deltas follow the cooked vertices; markers from extras and import settings, bad ones refused; a cooked walker plays in og-anim |
| `anim_compress.rs` (separate file) | every committed animated model cooked f32 and compressed: every frame of every channel within the tolerance, every node within 0.5 mm per metre of the skeleton, smaller samples, the same skins and morphs; the trickster posed mid-clip by each cook within og-tests' default frame tolerance; the settings parse and enter the stamp. Resident: a compressed model's loaded clips take exactly the bytes of its blob's sample sections (less than the frames as f32), every frame read from the keys has the bits of the frames written out from the format's description, and og.anim playing every clip at speeds 1, 0.37 and −1 gives the same world hash tick for tick from the keys as from those frames |
| `sound_formats.rs` (separate file) | an MP3 (1 s, 44.1 kHz stereo) and an Ogg Vorbis (0.5 s, 22.05 kHz mono) fixture cook to `og.Sound` blobs with the source's rate, channels and length (within 3 %), whose QOA decodes to the tone (RMS, zero crossings of the first channel); an undecodable `.mp3` is refused naming the file and the format |
| `sound_chunks.rs` (separate file) | a 12.5 s WAV with `sound { stream true chunk 1.5 }` cooks to 8 chunks whose head and chunk blobs laid end to end are the bytes of the same WAV cooked whole, and go into og-audio's bank; nothing re-cooks unchanged, a chunk length edit re-cooks only that sound and removes the old chunks; the 1 MiB threshold and `stream false`; bad settings refused by line; a chunked sound in a stream group packs its head, its chunks stay files of their own, and the group carries the whole sound's shape |
| `vox.rs` (separate file; and `cook::vox`'s unit tests) | a `.vox` with `model { voxScale 0.25 voxOrigin bottom }` cooks to those bounds (its settings file sorts after it, so the conversion waits for it); an unchanged project cooks nothing, a settings edit re-cooks the model; a bad `voxOrigin` fails it naming the line; every one of the 48 signed axis permutations an `nTRN` frame holds keeps every triangle counter-clockwise about its normal; `voxScale` and `voxOrigin bottom` place one and two models; the committed `tests/fixtures/vox/pivot.vox` (the test writer's: a 3 x 2 x 5 model of five isolated voxels placed as is, turned about z and y, mirrored in x, its axes cycled) puts every voxel in MagicaVoxel's cell (a ceiling pivot fails it); the same voxels with and without a scene graph cook to the same bounds, centred and with `voxOrigin bottom` (the unit test: the same glTF bytes, for even, odd and one-voxel sizes); an animated file (three graph-less frames, and an `nSHP` keyed by `_f` beside a static part) cooks to plain mesh nodes, no skin (no frame moves, so none has a joint), `voxClip`/`voxFps` clips or the default `frames`, visibility channels showing one frame's mesh node at a time (no transform channel, every node at rest scale 1, the frames after 0 hidden at rest; compressed at the loosest tolerance, the same flags), no primitive skinned, the bind-pose bounds frame 0's (the model cook's bounds leave out the nodes hidden at rest); a keyed transform (a block moved between frames 0 and 2 beside a static base) cooks to one joint with a STEP translation channel at `voxFps` and a visibility channel on its mesh node, its places the keys' in glTF's frame, bound to the block's joint (the unit tests: the keys sorted by `_f`, a turn at frame 1 splitting the bar into two runs, the run before the turn, which never moves, a plain mesh node without a joint, a folded frame holding its nearest shown place, `voxOrigin bottom` over every frame's place, keys that change nothing the same glTF as one key); `voxClip` past the last frame or on a static file, a bad `voxFps` and a repeated clip are refused by line |
| `flipbook.rs` (separate file) | animated GIF and APNG sources keyed, aligned to the feet, cropped to one size, deduplicated and packed into `#sheet` with their delays and events, the sources consumed, an edited GIF recooking, a texture beside `source` refused; atlases into sheets, and an `array true` sheet's frames cut into its texture array's layers texel for texel, frames of two sizes refused; such a sheet and a texture array in a streaming group pack the sheet, the array and every layer (each naming the group, the pack's bytes the files'), the group's memory counting the layers, the sheet's texture staying in the first group |
| `fx.rs` (separate file) | a `.cube` and an emitter cook and load with every line reaching its field; an emitter's `array` line (its layers and mode, `round`), a missing array refused; a streaming group holding a LUT, its memory the LUT's 32³ RGBA8 |
| `textures.rs` (separate file) | an RGBA8 KTX2 tagged `ru`/`bgra` cooks to the source texels exactly; an `rd` RGBA8 or Basis KTX2 is taken byte for byte; a Basis KTX2 tagged `ru` is re-encoded the right way up; `texture { stream true }` cooks a large texture's mip tail as `<id>` and its whole chain as `<id>#mips` from one encode, a texture within the tail is not split, and stream on pixel art or a non-boolean is refused |
| `retarget.rs` (separate file) | fx-showcase's animation library retargeted at cook time into `strider.glb` (Mixamo names, armature scale 0.01, a rig file) and `stout.glb` (Unreal names, detected): the clips and markers arrive, and on every frame the library's walk plants a foot the character's foot is on its own ground within 2 mm and does not slide; an edit of the rig file re-cooks only the model naming it, an edit of the library's settings both; a rig naming a missing node fails with the file and line |
| `cooked_models_render_the_gltf_reference_scene` | og-tests' `gltf-reference` scene with the models swapped for cooked ones renders within RMSE 0.02 of `crates/og-tests/references/<tier>/gltf-reference.png` |
| `compressed.rs` (separate file) | the template's Basis textures reach each block family the GPU takes (BC7/BC5, ASTC 4x4, ETC2/EAC RG11) by kind, with sRGB only on colour and every cooked level at its exact size; RGBA8 without a renderer or with compression off; stone boards near and far (small levels) draw within RMSE 0.012 and 0.5 % outliers of their RGBA8 frame, which the normal map read flipped exceeds; the cooked models with compressed textures stay within RMSE 0.02 and 1 % outliers of `gltf-reference`; `FrameStats::texture_bytes` shrinks |
| `environment.rs` (separate file; and og-render's `a_prefiltered_chain_is_taken_as_is`) | `a_prefiltered_ktx2_chain_is_taken_as_it_is`: the studio's own baked five levels written as an RGB9E5 KTX2 chain with `prefiltered true` cook back texel for texel (without it they are filtered again), its irradiance within 0.5 %; a one-level file, another size, `samples`, a non-KTX2 source and a bad flag are refused. Then: a `type environment` with og-tests' 64 × 32 studio `.hdr` (size 32, intensity 1.5, rotation 30, 64 samples) cooks into exactly og-render's `bake` of the same image and settings and loads back through `Bundle::load_environment`; the image is not listed as ignored; an unchanged project cooks nothing, an edited image recooks the map; a missing or non-HDR source, a bad size and an unknown key are refused by name. The studio cut into a 64² cube and given as an RGBA16F KTX2 (Zstandard) cooks into exactly `bake_cube` of its texels and within 0.5 % mean, 3 % worst prefiltered texel and 0.5 % irradiance of the equirectangular map; the same cube as RGB9E5, BC6H (zlib) and six `.hdr` faces within their precision of it; one face edited re-cooks only its environment; a KTX2 of one face, of RGBA8, oriented `ru`, short of its faces or holding a NaN, five faces, faces of two sizes, and a source with faces are refused by what is wrong. `cook::bc6h`'s unit tests: every mode, unsigned and signed, decodes 256 random blocks to a reference decoder's texels (bcdec) |
| `terrain.rs` (separate file) | a terrain and its heightmap, control map and layers cook into tiles that load back sample for sample, deterministically and incrementally; an EXR's metres as stored; refusals by name; a navmesh walks a terrain's ramp but not its cliff; `terrain_layers_from_texture_arrays`: `layers` and `layerNormals` cook into the arrays' ids and each layer's layers (its place by default) and load back, a terrain of separate textures has none, and a layer's own maps beside arrays, an index without its array, a non-`.ogt` or missing array, a fractional index are refused |
| `look_builtins.rs` (separate file) | a look naming `@canvas/paper` and `@palettes/handheld4`, and one extending `watercolor-wash` (which names `@canvas/watercolor`), cook with those three built-ins in the bundle under their ids and no other, loadable as a texture and a palette; the cooked look holds the ids; a second cook re-cooks nothing; a game's own `assets/@canvas/…` is refused |
| `tilemap.rs` (separate file) | a tileset `.ogt`, a hand-written map, a Tiled `.tmj` (external and embedded tilesets, flips, base64 data, offsets, parallax, properties, an `autotile` layer, objects) and an LDtk project (tiles with flips, IntGrid collision, an entity at its pivot) cook and load into og-tilemap assets equal to the sources, with resolved terrain and merged collision; a compressed Tiled layer, a missing sibling tileset `.ogt` and an unknown LDtk level are refused by name; import settings pick the unit, the level and the solid IntGrid values |

## `.vox` frame rate

`voxFps` gives a `.vox` animation's rate, because the file carries none:
MagicaVoxel's format (`MagicaVoxel-file-format-vox.txt` and its extension,
ephtracy/voxel-model) defines no rate in any chunk; `nTRN` and `nSHP`
frames carry only a frame index (`_f`), and the one animation setting the
editor writes is the playback range (`META`'s `_anim_range`, which the cook
reads), as ogt_vox (opengametools, whose reading this cook follows) does.
