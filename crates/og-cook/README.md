# og-cook

The cook (`docs/design.md` §2.5, phase 1 M4): source assets in `assets/`
become `.ogb` blobs (`crates/og-blob`) in `cooked/`, incrementally by hash;
and the runtime loaders that read `cooked/` into og-render's input types and
plain scene data. `cooked/` is the only thing the runtime reads.

```rust
let report = og_cook::cook_project(root, &og_cook::CookOptions::default())?;   // og cook
assert!(report.ok(), "{:?}", report.errors());

let bundle = og_cook::runtime::Bundle::open(&root.join("cooked"))?;             // native; Bundle::from_manifest(text) on the web
let (model, textures) = bundle.load_model_with_textures("assets/models/ship.glb")?;  // feature `transcode`
let parts = model.add_to(&mut render_scene, &textures);
parts.instance(&mut render_scene, transform);
let scene = bundle.load_scene("scenes/main.ogt")?;                             // plain data for the host to spawn
let anim = bundle.load_animation("assets/models/walker.glb")?;                 // og_anim::SkeletonAsset, ClipAssets, skin and morph streams
let map = bundle.load_tilemap("maps/room.tmj")?;                              // og_tilemap::TilemapAsset; its TilesetRef::Asset ids through bundle.load_tileset
let terrain = bundle.load_terrain("terrain/island.ogt")?;                     // CookedTerrain: og_render::TerrainData, layers naming their textures
let env = bundle.load_environment("env/studio.ogt")?;                         // og_render::EnvironmentMapData, prefiltered at cook time
```

`og-cook <project> [--threads N] [--report out.md] [--json out.json] [--schemas schema.json]`
is a development binary; `og cook` (og-cli) is the user-facing command and
calls `cook_project` (`--report` for the slowest ten, `--watch` to cook each
change as it is saved: §Cook times).

## Features

| Feature | Default | What it adds | Targets |
|---|---|---|---|
| `runtime` | via `cook` | `runtime` (Bundle, loaders), `ktx2`, `meshopt_codec`, `qoa`. Pure Rust: og-blob, og-core, og-render, glam, bytemuck, miniz_oxide | every target; `cargo check -p og-cook --no-default-features --features runtime --target wasm32-unknown-unknown` |
| `transcode` | yes | `CookedTexture::to_texture_data` (RGBA8 level 0) and `to_texture_source` (every level in the device's block format) for Basis textures (basisu's C++ transcoder, ruzstd) | native |
| `cook` | yes | `cook_project` and the importers (gltf, image with PNG, JPEG, EXR and Radiance HDR, meshopt, mikktspace, fdsm, ttf-parser, hound, ruzstd), og-render's `bake` (environment maps) | native |
| `basis` | yes | the basis-universal crate (basisu 1.16) as the texture encoder; without it the cook uses `toktx` from PATH | native |

## Pipeline

1. **Walk** `assets/` (sorted, dot-files skipped). By extension: `.glb`/`.gltf`/`.vox`
   models, `.tmj` (Tiled) and `.ldtk` (LDtk) maps, `.png`/`.jpg`/`.jpeg` textures, `.wav`/`.mp3`/`.ogg`/`.flac` sounds, `.ttf`/`.otf` fonts,
   `.ogt` scenes, particle emitters, tilesets, tilemaps, terrains, environments and looks (by `type`), `.hex` and `.gpl` palettes. `<file>.ogt` of `type import` holds `<file>`'s import
   settings. Files a `.gltf` references are part of that model. Anything else
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

### Shared texture results

After the per-project manifest check, ordinary standalone and glTF textures
can reuse a shared on-disk result. Compression settings and quality are
unchanged. The key is the existing cook stamp (input content, options,
encoder identity and cooker version), plus the expected type hash and schema
version. The result is the exact `.ogb` the uncached cook would write.
Standalone texture stamps also include their asset-relative path, so a
renamed standalone texture is a miss; a fresh project at another root is not.
Arrays, video frames and other asset processing do not use this cache.

`OG_TEXTURE_CACHE=/path/to/cache og cook` selects an isolated cache directory;
`OG_TEXTURE_CACHE=off` (or an empty value) disables it. Otherwise it uses
`$XDG_CACHE_HOME/origingame/texture-results-v1`, falling back to
`$LOCALAPPDATA/origingame/texture-results-v1` on Windows, then
`$HOME/.cache/origingame/texture-results-v1`; without these, it is disabled.
Rust callers can use `cook::cook_project_with_texture_cache(root, options,
Some(path))`, or `None` to disable caching without reading or changing the
environment. The cache tests use this entry point and private temporary paths.

Entries are `.ogb` containers holding the result and its FNV-1a content hash.
Reads validate that hash, the container, the result's type/schema and its cook
stamp. Missing, corrupt or unwritable entries fall back to normal cooking.
Writers exclusively create a sibling temporary file and rename the completed
entry into place; concurrent cooks may duplicate work but cannot expose a
partial result. A hit adds `shared texture cache hit` to the asset's notes
instead of reporting encoder work. The output still counts as processed
(`cooked: true`), with cache lookup time, and enters the normal project write
and manifest path. Unchanged project outputs remain `cooked: false`.

This is a disposable, trusted local cache, not an authenticated artifact store.
There is no eviction policy: remove its directory to reclaim space. Interrupted
writes may leave unused `.tmp` files; they are never read as entries.

### Importers

| Source | Output (`cooked/…`) | What happens |
|---|---|---|
| glTF 2.0 | `<path>.ogb` (`og.Model`) + `<path>.tex<n>.ogb` per texture use | Per primitive (glTF order): positions, normals (flat when missing), uv0, colour0, tangents (MikkTSpace for normal-mapped primitives that lack them, via unwelding); weld; quantise (position u16×3 over the bounds, normal oct snorm16×2, uv u16×2 over the uv range, colour u8×4, tangent oct snorm16×2 + sign); weld again; meshopt vertex-cache and overdraw (1.05) ordering; LOD chain by `meshopt_simplify` (3 levels, each half the previous, max error 5 % of the extent, stops when a level saves < 5 %); vertex-fetch reorder over all LODs; meshopt vertex codec v1 and index codec v1. Materials (metallic-roughness, emissive strength, alpha mode, double-sided) plus a default material, nodes in topological order, scenes and their roots. Skins, animations and morph targets go to the `#anim` blob (next row). Welding keys skinned and morphed vertices by their skin and morph data, so it never merges vertices that deform apart. |
| MagicaVoxel `.vox` | as glTF 2.0 (`og.Model`) | `cook/vox.rs`: `SIZE`/`XYZI` models (every one of a `PACK`), the `RGBA` palette (MagicaVoxel's default without one) and the `nTRN`/`nGRP`/`nSHP` graph (translations and the `_r` signed axis permutation; one with a negative determinant mirrors the model and its triangles' winding is flipped, so every face stays counter-clockwise from outside; without a graph each model sits at the origin about its pivot, exactly where a graph of one identity transform puts it, as MagicaVoxel and ogt_vox read such a file) become one glTF binary in memory: per placed model, a greedy mesh (per face direction and slice, maximal rectangles of one colour whose face is exposed; a 32³ block of one colour is 12 triangles) of unwelded quads with face normals and the palette colour as `COLOR_0` (linear), z-up turned to y-up (`(x, y, z)` → `(x, z, -y)`), `voxScale` metres a voxel (import settings `model { }`, default 1), the pivot MagicaVoxel's `floor(size / 2)` (ogt_vox's documented convention, which vengi matches: voxel `v` turned by `R` at `T` fills the cell at `floor(T + R (v + 1/2 - floor(size / 2)))`, odd sizes turned or mirrored included, checked voxel by voxel on the committed `tests/fixtures/vox/pivot.vox`), under a node at its translation; `voxOrigin bottom` lifts the whole file so its lowest voxel stands on y = 0 (`center`, the default, keeps the editor's origin); one white rough material. Animated files (MagicaVoxel's frames: an `nSHP` of several models, each shown from its `_f` key frame to the next; or several models without a graph, 0.98's `PACK` animation, model `i` frame `i`) become a glTF of mesh swaps: each animated model a plain mesh node at its translation (no joint: a frame that never moves needs no palette matrix and draws as a static instance when shown; a model a keyed transform moves is skinned, below), and per clip one visibility track per model on its mesh node (`OG_animation_visibility`, the row below; STEP, 1 on the frames its model shows and 0 elsewhere), a key per frame at `voxFps` (default 10) and one at the clip's end holding the last frame, so a frame folded away is not drawn (og-runtime skips a hidden node's primitives; before the visibility track it was a STEP scale of 0 on its joint, still drawn); the clips are `voxClip <name> <first> <last>` ranges (both frames shown), else one clip `frames` of every frame. Each frame's model sits about its own pivot as MagicaVoxel shows it (frames of different heights do not share a floor; `voxOrigin bottom` lifts the file by the lowest voxel of any frame). The rest pose shows frame 0 (the other frames' mesh nodes `KHR_node_visibility` hidden, which the model blob's node rows carry, so the draw without an `og.Skeleton` is frame 0 alone, and the model's bounds leave them out; a skinned one is also scaled to 0 for a reader that ignores the extension). `nTRN` key frames (each frame of a transform its own `_f`, `_t` and `_r`, held until the next key as ogt_vox samples them; owed row 63) make a file animated too: the graph is walked once per frame up to the last key of any shape or transform, and a model whose place or turn changes across frames is a mesh per run of frames at one turn (a turn is another set of voxel faces, swapped in like a frame's model); a run whose place changes is skinned whole (weight 1) to a joint of its own (scale 1, inverse bind the identity) carrying a STEP translation channel beside its mesh node's visibility track, its place on each frame (on a frame it is folded away, the place of its nearest shown frame); keys that change nothing leave the file static, byte for byte. A node's `_loop` (`nTRN` or `nSHP`, `1`) repeats its keys from its first to its last on every frame outside them, a frame before the first counting back from the last (ogt_vox's `compute_looped_frame_index`); the scene's playback range (`META`'s `_anim_range "<first> <last>"`, written by MagicaVoxel since 0.99.7.2) walks the frames up to its last (keys past it are not played) and is the default clip `frames` (frames before it stay frames of the file a `voxClip` may name); a range that ends before it starts is ignored, and a file whose range shows nothing move (the editor writes `0 30` into every file) stays static. Then the glTF row above, and the `#anim` row below for an animated file. |
| glTF skins, animations, morph targets | `<path>.anim.ogb` (`og.Animation`, id `<path>#anim`) | Only when the glTF has any. The node hierarchy in the model blob's order; skins (joints, inverse bind matrices, `skeleton`); per clip, every channel evaluated by the glTF sampler (STEP, LINEAR with shortest-arc slerp, CUBICSPLINE Hermite; f64; a key within 1 µs of a sample counts as reached) at `i / rate` and at the end, `rate` = the import settings' `animation { rate }` or the source's key rate rounded (1–120 Hz), the source rate kept; markers from `animations[i].extras."og.events"` (`[{"name", "time"}]`) and `animation { event "<clip>" "<name>" <s> }`; visibility: the nodes `KHR_node_visibility` hides (`visible: false`) are the blob's `hidden` table (og-anim's `SkeletonAsset::hidden`, the rest visibility), and an animation's visibility tracks, the engine's extension `animations[i].extensions.OG_animation_visibility.channels` (`[{"sampler": <the animation's sampler>, "node": <node>}]`, the sampler's output a float scalar per key, visible from 0.5), are STEP channels of 0 or 1 (`path` 4, `ChannelPath::Visibility`; glTF's own `KHR_animation_pointer` onto `/nodes/<n>/extensions/KHR_node_visibility/visible`, a channel path the gltf crate refuses, is rewritten in the JSON before the crate parses it into an entry of this extension, its sampler's output floats or unsigned bytes, 0 hidden; the crate then validates the file as any other. A pointer onto any other property is dropped with a warning, or refused naming it when the file requires `KHR_animation_pointer`; a file with no `pointer` path is parsed untouched); samples f32 by default (a channel whose frames are all the same bits stored once), or with `animation { compress true }` quantised with keyframe reduction within `tolerance` (§Animation samples); per skinned primitive joints u16×4 and weights unorm16×4 (summing to 65535) per cooked vertex; per morphed primitive, sparse, each target's moved vertices (a delta with any bit set) with their indices, position (and normal) deltas, names from `mesh.extras.targetNames`, default weights; streams meshopt-encoded. |
| PNG/JPEG | `<path>.ogb` (`og.Texture`) | Decoded to RGBA8, then KTX2: colour → ETC1S/BasisLZ, sRGB, quality 128; `kind normal` → UASTC level 1 + RDO 1.0 + Zstandard, linear, tuned for normal maps, x y z in RGB and y again in alpha (Basis transcoders take BC5's and EAC RG11's second channel from alpha; `has_alpha` 1 says a map carries it, and the settings string's "normal y in alpha" recooks older maps); `kind linear` → the same without the normal-map tuning. Full mip chain. `kind pixel` → RGBA8 + zlib, sRGB, lossless, one level; the runtime samples it nearest (sprite sheets, tilesets). In a glTF, base colour and emissive are colour, metallic-roughness and occlusion linear, normal normal. |
| KTX2 | `<path>.ogb` (`og.Texture`) | `src/cook/ktx2_source.rs`. Basis ETC1S or UASTC, or RGBA8 (none or zlib), its `KTXorientation` and `KTXswizzle` honoured: a file that is `rd` and `rgba` (or says neither), of a colour or linear kind matching its transfer function, is taken byte for byte; any other is decoded (Basis through the native transcoder), flipped to rows top to bottom and columns left to right, swizzled (`r g b a 0 1` per channel) and cooked again as an image of its kind (`texture { kind }`, else sRGB colour or linear). The report says which. `toktx`'s own output goes through the same check. |
| texture array `.ogt` (`type texture-array`) | `<path>.ogb` (`og.TextureArray`) + `<path>.layer<n>.ogb` (`og.Texture`, id `<path>#layer<n>`) per layer | `src/cook/texture_array.rs`: `kind color|normal|linear|pixel` and 1–256 `layer "<file>"` images (beside the file, else under `assets/`) of one size (a layer of another size is refused naming both); each layer cooked as a texture of the kind; the array blob holds the size, kind, level count and the layers' ids. The runtime uploads the layers as one `D2Array` texture (`og.load.textureArray`, og-render's `RenderScene::add_texture_array`; padded with the last layer when WebGL2 would take the count for a 2D or cube texture). World and overlay sprites (`SpriteDesc.array`, `og.Sprite.array`), `array true` flipbooks and 2D particle emitters (an emitter's `array` line, `src/fx.rs`) draw from it, one draw per array whatever the layers; so do lit materials (`mapArray`, `normalMapArray`) and terrain layers (`layers`, `layerNormals`) since cook round four. |
| WAV, MP3, Ogg Vorbis, FLAC | `<path>.ogb` (`og.Sound`), and for a sound cooked in chunks `<path>.chunk<n>.ogb` (`og.SoundChunk`, id `<path>#chunk<n>`) | A WAV's 8–32-bit integer or 32-bit float PCM, 1–8 channels, to 16-bit; an MP3, Ogg Vorbis or FLAC decoded to 16-bit by symphonia (pure Rust, the first audio track, encoder delay and padding trimmed; Ogg Opus is refused); then QOA (in-crate encoder: 3.2 bits per sample). With `sound { stream true }`, or when the QOA file would pass 1 MiB (27 s of 48 kHz stereo) unless `stream false`, the file is encoded once and cut between QOA frames into chunks of about `chunk` seconds (default 4, whole frames): the sound's blob holds the QOA header and the first chunk, each other chunk a blob of its own, which the runtime reads ahead of the playheads and drops behind them (og-audio's README §Chunked sounds). A QOA frame carries its predictor's state, so the chunks decode to the whole file's samples exactly. A chunked sound in a stream group puts its head in the group's pack; its other chunks stay files of their own (never grouped), read ahead of the playheads once the group is resident. |
| TTF/OTF | `<path>.ogb` (`og.Font`) | The import settings' charset, rendered by fdsm as MTSDF (RGB multi-channel, A true SDF) at `em_px` with `px_range`, generated in f32 and passed through msdfgen's error correction (fdsm's `correct_error_mtsdf`: edge priority, distance checks at edges, before the sign correction) so channels that would interpolate into a false edge at a sharp corner become single-channel; shelf-packed into pages, each page a lossless KTX2 (RGBA8 + zlib); a glyph table (advance, plane bounds in em, atlas rectangle), ascender/descender/line gap and a kerning table: the pair adjustments of every `kern` feature's GPOS lookups (pair positioning formats 1 and 2, every script's; the first subtable of a lookup that takes a pair decides it, lookups add), else the legacy `kern` table, between the cooked characters, in em. A charset with a shaped script (Arabic, Devanagari and og-i18n's other `needs_shaping` scripts) adds every glyph the font's substitutions reach (joining forms, ligatures, conjuncts) under U+F0000 + glyph id, and the font file; so does `runtime true`, for characters outside the charset that og-ui renders from the outlines when a text needs them. |
| rig `.ogt` (`type rig`) | `<path>.ogb` (`og.Rig`) | A humanoid map (og-anim's README §Retargeting): `scheme` (auto, mixamo, vrm, unity, rigify, ue, blender) and standard bones named outright or `none`; validated and stored as canonical text, loaded with `og.load.rig`. A model's `animation { rig }` names it for the cook's retargeting. |
| animation library (a glTF named by another model's `animation { library }`) | into the naming model's `#anim` | The library is cooked like any glTF (its own `#anim`, loadable with `og.load.animation`); the naming model's `#anim` gets the library's clips after its own, each rewritten by `og_anim::retarget` from the library's skeleton (its humanoid map from its own `animation { rig }` or detected) onto the model's (its `rig` or detected): rest corrections, the model's bone lengths, the hips' travel scaled by the leg ratio, the legs solved for the feet unless `footIk false`. A clip the model has of its own wins over a library's of the same name (a note says so); a library naming libraries is refused. Then each `mirror "<clip>" ["<name>"]`: the clip (the model's own or a library's, retargeted) mirrored on the model's skeleton by `og_anim::mirror` (sides paired by name, reflected in x = 0), written under the name (default `<clip>.mirrored`), so a mirrored clip costs nothing at run time; a clip it lacks, a name already taken or named twice is refused. |
| flipbook `.ogt` (`type flipbook`) | `<path>.ogb` (`og.Flipbook`) | The canonical sheet text (og-anim's README): grid and `frame` lines, an `atlas` (Aseprite frames and tags, TexturePacker frames and pivots) resolved into frames and clips, the texture's size read from the image, paths relative to the sheet when the file exists. The stamp covers the texture's and the atlas's bytes. With `array true` the frames, cut out of the texture, also cook into a texture array: `<path>#array` (`og.TextureArray`) with a layer `<path>#array#layer<n>` (`og.Texture`, pixel art under `filter nearest`, colour otherwise) per frame, frame `n` being layer `n`; every frame must then have one size (two sizes are refused naming both) and lie inside the texture, at most 256 of them. The runtime draws an `og.Flipbook` of such a sheet from the array (one draw per sheet whichever frames its sprites show). |
| scene `.ogt` | `<path>.ogb` (`og.Scene`) | Entities depth-first (parent before child), `model` references checked against the project's assets, one packed table per component type keyed by its reflected name. |
| tileset `.ogt` (`type tileset`) | `<path>.ogb` (`og.Tileset`) | Parsed and validated by og-tilemap (grid, solid and partial collision, animated tiles, blob47/bitmask16 terrains); its `texture` must be an asset. The canonical text is stored and parsed again at load. |
| tilemap `.ogt` (`type tilemap`), Tiled `.tmj`, LDtk `.ldtk` | `<path>.ogb` (`og.Tilemap`) | One `og_tilemap::TilemapAsset` whatever the source (`src/cook/tilemap.rs`, native only). **Tiled**: orthogonal, finite maps; layer data as a JSON array or uncompressed base64 (zlib/gzip/zstd refused: save as CSV or Base64); flip bits kept (diagonal refused); `offsetx/y`, `parallaxx/y`, `opacity`, `visible`, `repeatx/y`, group layers flattened; custom properties `collision` (bool), `z` (int), `autotile` (a terrain of the layer's tileset: nonzero gids mean filled), `repeatx`/`repeaty`; an external tileset `x.tsx`/`x.tsj` is the sibling `x.ogt` (refused when missing, naming the path), an embedded one (single image) is imported with its tiles' `animation`, first `objectgroup` rectangle (collision) and boolean `solid` property; one tileset per layer; object layers become objects (`type` or `class` as kind, string-valued properties; tile objects anchored at their bottom). **LDtk**: one level (the first or `level`; separate level files refused), every layer on one grid size; `Tiles`/`AutoLayer`/IntGrid auto tiles become tile layers (tile id `t` or from `src`; flips from `f`; one tile per cell, the last of a stack wins), drawn in reverse list order (LDtk's first layer on top); IntGrid values listed in `solid` (default 1) become an invisible collision layer `<name>.collision` over an embedded one-tile solid tileset; a tileset image with a sibling `<stem>.ogt` uses that tileset, else one is embedded from `tileGridSize`, `__cWid`, `__cHei`, `padding`, `spacing`; `Entities` become objects at their top-left (pivot applied), kind the identifier, name the `name` field or the `iid`, fields as properties. Units: `unit` world units per tile width (default 1); object positions and offsets in map-local world units, y up from the top-left corner. Referenced tilesets and images must be assets. |
| environment `.ogt` (`type environment`, phase 4) | `<path>.ogb` (`og.EnvironmentMap`) | `src/cook/environment.rs`: its `source` (an equirectangular Radiance `.hdr` or OpenEXR `.exr`, linear RGB, twice as wide as tall, decoded by the `image` crate; or a KTX2 cube map, `src/cook/cube_source.rs`: six square faces +x −x +y −y +z −z with rows from the top, level 0 of RGBA16F, RGB16F, RGBA32F, RGB32F, RGB9E5, B10G11R11 or BC6H, unsigned or signed, decoded at cook time by `src/cook/bc6h.rs`, stored plain, Zstandard or zlib, `KTXorientation` `rd` or none; with `prefiltered true` every level, its own mip chain already GGX-prefiltered (cmgen's, IBLBaker's, the glTF IBL sampler's), taken as the map's levels by `bake::from_prefiltered`: level m read at lod m/4 · (L − 1) of the file's L levels, a chain of five texel for texel, at the file's own size, no `samples`) or its `faces` (six square `.hdr`/`.exr` images of one size, in that order); read beside the file, else under `assets/`; part of the environment like a terrain's heightmap: not cooked or listed on its own, their bytes in the stamp) prefiltered by og-render's `bake` (`bake::bake`, or `bake::bake_cube` for a cube: the same radiance cube, read from the source's box-filtered level of at most twice the bake's face): `size` (level 0's face, a power of two 16..2048, default 128), `intensity` (default 1), `rotation` (degrees about +y, what was at +x moving towards -z; default 0) and `samples` (GGX samples per texel, 16..4096, default 256) into five cube levels of RGB9E5 texels, level m GGX-prefiltered at roughness m/4 (og-render's IBL convention; level 0 the radiance), and the diffuse irradiance as nine order-2 harmonics. fx-showcase's 256 × 128 studio at size 128 cooks in 0.52 s (a dev build, six threads) into a 132 KB blob (511 KiB of texels, zlib'd to 128 KiB). Loaded by `og.load.environment` (`Bundle::environment`), lit by through `og.Environment`. The studio given as a 64² cube (RGBA16F) bakes within 0.2 % mean per texel and 0.17 % in irradiance of the equirectangular studio it was cut from (`tests/environment.rs`). |
| terrain `.ogt` (`type terrain`, phase 2) | `<path>.ogb` (`og.Terrain`) | `src/cook/terrain.rs`: its `heightmap` (a 16- or 8-bit grey PNG, its first channel at 16 bits; or an EXR, its first channel, through the `image` crate's `exr` feature) mapped by `height <min> <max>` (PNG default 0 100; EXR default 0 1, metres as stored), its `size` in metres (default one per sample), rows from -z (the image's top) and columns from -x; cut into `tile` × `tile` sample tiles (default 128; the last ones smaller), each quantised to 16 bits over its own range (finer than a 16-bit source's step over the whole range) and zlib'd; the `control` map (RGBA, one channel per layer) zlib'd; up to four `layer { albedo normal color roughness tile triplanar normalStrength slope slopeFade altitude altitudeFade }`, their textures assets of their own (give a normal map `texture { kind normal }`), or their maps layers of two texture arrays (`layers "<array>.ogt"`, `layerNormals "<array>.ogt"`, assets of their own of `type texture-array`, beside the terrain or under `assets/`; each layer's `albedoLayer` and `normalLayer`, by default its place: the terrain then samples two textures instead of up to eight, og-render's `Terrain::layer_arrays`); `heightBlend`, `castShadows`, `collision`, `lod { spacing ring }`. The heightmap and control map are read beside the terrain file (else under `assets/`) and are the terrain's, not textures: the walk does not cook them, and the terrain's stamp covers their bytes. A navmesh `.ogt`'s `terrain "<file>" { position x y z step n }` bakes the terrain's surface (every `step`th sample, each cell split along the renderer's diagonal, only the cells inside the navmesh's `bounds`) with the agent's slope limit. |
| look `.ogt` (`type look`) | `<path>.ogb` (`og.Look`) | `src/look.rs` (the grammar, merging, hex colours) and `src/cook/look.rs`: a frame's art direction as data. Top-level `name`, `extends`, `ui` (a UI style: a built-in slug or the game's own asset id) and `antialias` (tier, off, msaa4, taa), then one block per resource it sets (`RenderSettings`, `PostEffects`, `Stylise`, `MaterialStyle`, `Fog`, `Sky`, `Environment`, and the art contract's `ArtPalette`, `ArtPixels`, `ArtTexels`, `ArtShape`, `ArtLine`, `ArtMotion`, `ArtCamera`) with its fields by reflected name. `extends "<slug>"` resolves to the game's own `assets/looks/<slug>.ogt` (never the file itself), else the built-in preset `templates/looks/<slug>.ogt` (`PRESETS`, compiled in); `extends "looks/base.ogt"` to another look; chains up to 8 deep, cycles refused; the look is laid over its base field by field, its name its own. Every field is checked against the IDL's records (`og_api::host::RECORDS`): it exists and is writable, a number is in range and whole for u32/i32, an enum takes one of its variants, a colour is three linear numbers or an sRGB `"#rrggbb"` (stored linear), a handle field (`Texture`, `Lut`, `Palette`, `EnvironmentMap`) names an asset of its kind in the bundle, or one of the engine's built-in assets (below) as `"@canvas/<name>"` / `"@palettes/<name>"`. `PostEffects.flash` and `shakeTrauma`, animated by game code, are refused. The blob holds the flattened canonical text with `extends` listing the chain; the stamp covers every base's text and whether each named asset exists. Loaded by `og.load.look` (`Bundle::look`), applied by `og.render.applyLook` and `GameDesc.look`. project.ogt's `look "looks/<id>.ogt"` names the project's look: the cook reads its art contract (the flattened look's art blocks) before any asset cooks and notes in the look's report what the contract sets; a line naming no look of the bundle is that asset's error, a malformed line fails the cook. |
| palette `.hex`, GIMP `.gpl` | `<path>.ogb` (`og.Palette`) | Up to 256 sRGB colours in file order: `.hex` one `rrggbb` per line (`#` optional, blank lines and `;` comments skipped), `.gpl` after its `GIMP Palette` header (`Name:`/`Columns:` and `#` lines skipped, `r g b [name]` per colour). Loaded by `og.load.palette` (`Bundle::palette`) for `Stylise.palette`. |

### Built-in look assets

The engine ships a few assets a look can name without the game owning a
copy (`BUILTINS` in `src/cook/look.rs`, compiled into the cook from
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
   a frame treatment, as before.
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
| `<id>#index` | a palette mapped every texel (strict `colors`, or the asset's own palette) and holds at most 255 colours | **the index texture**, for runtime palette swaps (docs/art.md §2.4, stream A4): the conformed texture's size and layout; `kind pixel` (RGBA8 sRGB, sampled nearest); RGB is the conformed colour (so it draws as the texture does), **alpha is the palette index plus one**, 0 a clear texel. Alpha is never sRGB-decoded, so a shader reads `index = round(a * 255) - 1` exactly. The palette is `ArtPalette.colors` in file order, or `<id>#palette` |
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
  px_range 4
  page_size 1024
  runtime true              # ship the font file: characters outside the charset render at run time (default false)
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

A scene's `stream "<name>" { asset ... }` blocks (`src/cook/stream.rs`)
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

A particle emitter (`type particles`, `src/fx.rs`) may name a texture
array instead of a texture: `array "<id>" [<layers>] [off | life <cycles>
| fps <n> | random]` draws a sprite-mode emitter's particles from the
array's layers (by age, at a rate, or one at random per particle), and
`round true` gives them the soft dot's round edge; the cook checks the
array exists like a texture.

## Formats

Every blob is an `.ogb` container (`crates/og-blob/README.md`). Its root is a
reflected struct (`#[derive(Reflect)]`, 4-byte scalar fields only, so the
packed layout equals the `#[repr(C)]` layout) in a one-row `meta` table; its
tables are rows of reflected structs tagged with their layout hash. The
header's type hash is FNV-1a over the layout hashes of the root and every row
type (`format::type_hash`); the schema version is the root's. All in
`src/format.rs`.

| Blob | Root | Sections |
|---|---|---|
| `og.Model` | `ModelMeta` (counts, default scene, skipped animations/skins, default-scene bounds) | `primitives` `PrimitiveRow` (material, attribute bits, vertex count/stride, byte ranges in `vertices`/`indices`, index count, LOD range, dequantisation offset/scale), `lods` `LodRow` (index range, simplify error), `meshes` `MeshRow` (primitive range, name), `materials` `MaterialRow`, `textures` `TextureRefRow` (asset id, kind), `nodes` `NodeRow` (parent, mesh, name, TRS, `visible`: 0 when `KHR_node_visibility` hides it at rest; schema 2 added it, and the stamps of every model, proxy and `#anim` carry `nodes-v2`, so each re-cooks once), `scenes` `SceneRow`, `roots` (u32), `strings` (UTF-8 pool), `vertices`, `indices` (meshopt streams, 4-aligned per primitive) |
| `og.Texture` | `TextureMeta` (size, levels, kind, codec, colour space, alpha, supercompression) | `ktx2`: a complete KTX2 file, 16-aligned |
| `og.TextureArray` | `TextureArrayMeta` (size, layers, levels, kind) | `layers` `ArrayLayerRow` (the layers' texture ids in `strings`), `strings` |
| `og.Sound` | `SoundMeta` (rate, channels, frames, QOA frames per chunk, chunk count; 0 0 when whole) | `qoa`: a complete QOA file, or of a chunked sound its header and first chunk |
| `og.SoundChunk` | `SoundChunkMeta` (chunk, first QOA frame, frame count) | `qoa`: the chunk's QOA frames |
| `og.Font` | `FontMeta` (units/em, em_px, px_range, ascender, descender, line gap, glyph and page counts, page size, uncooked code points) | `glyphs` `GlyphRow` sorted by code point (shaped forms at U+F0000 + glyph id), `kerning` `KernRow` (left, right, adjust in em) sorted by pair, `atlas.<n>` KTX2 RGBA8/zlib, `ttf` (the font file, with a shaped script or `runtime true`) |
| `og.Animation` | `AnimMeta` (counts; schema 2 added `hidden_count`, so every `#anim` cooked before the visibility track re-cooks) | `nodes` `NodeRow`, `hidden` (u32 nodes hidden at rest, ascending), `skins` `SkinRow`, `joints` (u32), `inverse_binds` (f32 ×16 per joint, column-major), `clips` `ClipRow` (name, duration, rate, source rate, frames, channel and event ranges), `channels` `ChannelRow` (node, path, source interpolation, lanes, sample offset, codec, keys, packed offset), `samples` (f32), `packed` (u16 quantised keys), `events` `EventRow`, `skin_streams` `SkinStreamRow` + `skin_data`, `morphs` `MorphRow` (moved-vertex count, data and index streams) + `targets` `TargetRow` (moved range) + `morph_data`, `strings` |
| `og.Scene` | `SceneMeta` (counts, name) | `entities` `EntityRow` (name, parent, instance), `refs` `RefRow` (asset ids), `components` `ComponentRow` (name, layout hash, rows, inferred), `strings`, `c.<Type>` rows (`u32` entity index + packed fields), `s.<Type>` schema JSON |
| `og.Tileset` | `TilesetMeta` (text bytes) | `text` (the canonical tileset `.ogt`) |
| `og.Tilemap` | `TilemapMeta` (width, height, tile px, unit, layer/object/tileset/prop counts) | `layers` `MapLayerRow` (name, tileset, terrain name, parallax, offset, opacity, flags visible/collision/repeat x/repeat y/terrain, z, first cell), `cells` (u32 LE per cell, layer after layer, og-tilemap's cell encoding), `objects` `MapObjectRow` (name, kind, rect, property range), `props` `MapPropRow`, `tilesets` `MapTilesetRow` (asset id, or embedded canonical text), `strings` |
| `og.EnvironmentMap` | `EnvironmentMeta` (size, levels, texel count, the source's width and height, intensity, rotation, samples; 0 for a KTX2's own prefiltered levels) | `texels` (og-render's `EnvironmentMapData::texels`: RGB9E5 u32 LE, level after level, each the six faces +x −x +y −y +z −z, rows from the top; zlib), `irradiance` `EnvironmentShRow` × 9 (r, g, b of the irradiance-over-π harmonics) |
| `og.Terrain` | `TerrainMeta` (size, samples, tile size and count, height range, layer count, control map size, height blend, lod, cast shadows, collision, the layer arrays' ids as ranges of `strings`; version 3) | `tiles` `TerrainTileRow` (first sample, size, range, byte range in `heights`), `heights` (per tile u16 LE, zlib), `layers` `TerrainLayerRow` (texture names as ranges of `strings`, og-render's layer fields, its layers of the arrays; version 2), `strings`, `control` (RGBA8, zlib) |
| `og.Look` | `LookMeta` (text bytes) | `text` (the canonical, flattened look) |
| `og.Palette` | `PaletteMeta` (colour count) | `rgb` (3 bytes per colour, sRGB) |
| `og.Lut` | `LutMeta` (side, 32) | `rgba` (size³ RGBA8, red fastest) |
| `og.Flipbook` | `FlipbookMeta` (text bytes) | `text` (the canonical sheet: frames resolved from the grid and an Aseprite or TexturePacker atlas, clips with frame indices and durations, the texture's size, `array`) |
| `og.Rig` | `RigMeta` (text bytes) | `text` (the canonical humanoid map: its scheme and the bones it names) |
| `og.ParticleEmitter` | `EmitterMeta` (text bytes) | `text` (the validated emitter file) |
| `og.UiStyle` | `StyleMeta` (text bytes) | `text` (the validated style file) |
| `og.MaterialShader` | `ShaderMeta` (text bytes) | `text` (the canonical shader file, its code inlined) |
| `og.Behaviour` | `BehaviourMeta` (text bytes) | `text` (the validated behaviour file) |
| `og.Strings` | `StringsMeta` (text bytes) | `text` (the validated string table) |
| `og.NavMesh` | `NavMeshMeta` (cell size and height, the agent's height, radius, climb and slope, tile size, max error, origin, grid width and depth, tile count) | `columns` `NavColumnRow`, `spans` `NavSpanRow` (the walkable heightfield), `tiles` `NavTileRow`, `verts` `NavVertRow`, `polys` `NavPolyRow`, `poly_verts` (u32) |
| `og.StreamPack` (`cooked/stream/<group>.ogb`) | `StreamPackMeta` (file count) | `files` `StreamFileRow` (name and data ranges), `names` (paths under `cooked/`), `data` (the files, each 16-aligned) |

Besides the blobs, `cooked/` holds the manifest `.cook.ogt` (§Asset ids)
and the cook times `.cook-times.ogt` (§Cook times), both `.ogt` text. A
cube map an environment reads is not a blob of its own: it is decoded at
cook time (§Importers), so the runtime never sees KTX2 cubes, BC6H or
half floats, only the `og.EnvironmentMap`'s RGB9E5 levels.

KTX2 files are standard: ETC1S as BasisLZ with basisu's DFD and global-data
layout, UASTC with Zstandard (scheme 2), lossless RGBA8 with zlib (scheme 3).
The basis-universal crate only emits `.basis`, so `ktx2::from_basis` rebuilds
the KTX2 container from the `.basis` bits exactly as basisu's own
`create_ktx2_file` lays it out, and `ktx2::to_basis` is the inverse the
native transcoder needs (its Rust wrapper reads `.basis` only).

At runtime a texture reaches the GPU in the encoding the renderer takes for
its kind (`CookedTexture::target_format` over og-render's
`RenderInfo::texture_formats`): colour and linear data as BC7, ASTC 4x4 or
ETC2, normal maps as BC5, ASTC 4x4 or EAC RG11 (BC7 or ETC2 for a map cooked
without y in alpha), with every cooked mip level
(`CookedTexture::to_texture_source`, `CookedModel::add_to_with`). RGBA8
level 0, with mips the GPU builds, is the last resort: pixel art, distance
fields and other lossless cooks, sizes not in whole 4x4 blocks, a device
with no block format, or no renderer attached yet. og-runtime's loaders
pass the attached backend's formats to the host's texture hook; natively
basisu transcodes here, on the web the engine Worker transcodes with
Binomial's transcoder once the renderer is attached, to the format
`og_web_texture_target` names. On the template's five Basis textures the
GPU holds 2.5 MB instead of 10.1 MB (`tests/evidence/P2/render/compressed-textures.md`).

### Animation samples

`src/anim_codec.rs` holds a channel one of three ways (`ChannelRow::codec`):

| Codec | When | Stored |
|---|---|---|
| f32 | the default | `frames × lanes` f32, as resampled |
| constant | every frame the same bits (always), or within `tolerance` of the first (compressed) | one sample |
| quantised | `animation { compress true }` | rotations as 48-bit smallest-three (the largest component's index in 2 bits, the others in 15 bits over ±1/√2); translations, scales and morph weights as unorm16 per lane over the channel's own `min`/`extent` (in `samples`); then keyframe reduction: the first and last frame kept, and every frame dropped that the interpolation between its kept neighbours (lerp, normalised lerp on the shorter arc, a hold for STEP) rebuilds within `tolerance` of the resampled value; the kept frames' indices as u16 when any was dropped |

A visibility channel (0 or 1, STEP) is never approximated: its tolerance
is 0 whatever the setting, so a compressed one keeps exactly the frames
where the flag changes and the ends, and its unorm over [0, 1] rebuilds 0
and 1 bit for bit (`anim_codec`'s unit test, `tests/vox.rs`).

The loader keeps each channel as stored (`anim_codec::samples`, og-anim's
`Samples`: f32, one constant sample, or the quantised keys with their
kept frames' indices and ranges), so a resident clip costs what its
cooked form costs (`ClipAsset::sample_bytes`, the blob's `samples` and
`packed` sections). og-anim samples the keys where they lie: a dropped
frame is rebuilt when it is read with the same interpolation the
reduction measured, scalar and in a fixed order, so every host gets the
same floats and a pose is bit for bit the one the frames written out as
f32 give (`tests/anim_compress.rs`: the world hash of every clip played
forwards, slowed and backwards is the same tick for tick, and was the
same before this change). A channel whose 16-bit quantisation alone would
miss the tolerance stays f32. The tolerance is per lane in the channel's units
(metres, unit-quaternion components, scale, weight); its default, 1e-4, is
0.1 mm and 0.011°. Retargeting reads the clips uncompressed, then the
model's `#anim` compresses the result.

On fx-showcase's animated models (the game template, third-person and
og-tests carry copies of the same files), the `#anim` blobs in bytes
(`tests/evidence/P4/cook/anim-compress.md`):

| Model | Before (r3, f32) | f32 + constants + sparse morphs | Compressed (1e-4) | Samples f32 → compressed |
|---|---:|---:|---:|---:|
| `walker.glb` (6 clips, 4 morph targets) | 9,002 | 9,051 | 6,203 | 4,892 → 2,028 |
| `trickster.glb` (4 clips) | 121,879 | 106,967 | 99,479 | 25,200 → 2,012 |
| `strider.glb` (3 retargeted clips) | 21,285 | 20,501 | 10,293 | 15,940 → 4,448 |
| `stout.glb` (no clips) | 4,136 | 4,200 | 4,200 | — |
| `puppet2d.glb` (1 clip) | 8,427 | 8,603 | 5,899 | 4,836 → 2,132 |
| `anims/humanoid-clips.glb` (3 clips) | 14,736 | 12,576 | 7,648 | 11,636 → 4,150 |
| **total** | **179,465** | **161,898** | **133,722** | **62,504 → 14,770** |

The trickster's blob is mostly its skin stream (93 KB of joints and
weights per vertex). Every compressed lane is within the tolerance (worst
9.97e-5), every node within 0.18 mm in model space, and the trickster posed
mid-clip by either cook renders within og-tests' default frame tolerance
of the other (RMSE 0.0028, 73 of 921,600 pixels past 32 levels, all on
silhouette edges). f32 stays the default, so every committed reference
frame is unchanged.

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

## The template, measured

`templates/game/assets` on an M4 Pro (release build, all cores), evidence in
`tests/evidence/M4/`:

| Class | Sources | Cooked | Cooked / source |
|---|---:|---:|---:|
| font (Latin-1 + CJK punctuation + 30 UI hanzi: 508 glyphs, one 1024×408 page) | 2,514,063 | 555,854 | 22.1 % |
| model (2 glTF, incl. their 3 textures) | 1,982,444 | 984,408 | 49.7 % |
| scene | 530 | 1,174 | 221 % |
| sound | 44,144 | 9,144 | 20.7 % |
| texture (2 standalone PNG) | 325,545 | 185,158 | 56.9 % |
| **total** | **4,866,726** | **1,735,738** | **35.7 %** |

A full cook takes about 3.1 s (UASTC with RDO dominates: 2–3 s per 512²–1024²
texture on one thread each); an up-to-date cook about 10 ms. Texture PSNR
(RGB, level 0, transcoded by basisu): ETC1S `stone.png` 41.3 dB, glTF base
colour 1024² 28.9 dB; UASTC `stone_n.png` 47.6 dB, glTF normal 47.2 dB, glTF
metallic-roughness 43.1 dB. Without the animation data the cook drops,
trickster.glb's geometry is still a fair comparison: 18,641 triangles cook to
a 387 KB model blob (LODs 18,641 / 9,320 / 6,724 triangles).

**CJK fonts.** `charset all` on the platform's subset (7,831 glyphs: GB2312,
kana, punctuation) cooks to 10 pages of 1024² in 4.9 s and **19.8 MB** — eight
times the TTF. Games should declare the characters they show (`chars`), as
the template does, and add `runtime true` for text they cannot know (a
player's name, chat): the blob then carries the 2.5 MB font file and og-ui
renders what the charset left out from its outlines into pages of the
font's own (`crates/og-ui/README.md` §Text). The UI kit's Chinese font does.

## Determinism

The same sources, options and **cooker build** give byte-identical `cooked/`
whatever the thread count and the machine's load (`cooking_twice_gives_identical_bytes`
cooks the template on all cores and on one thread;
`every_thread_count_cooks_the_same_bytes` cooks a small copy thirty times on
1, 2 and 8 threads, three cooks at a time). Both compare every file, with
`.cook-times.ogt`, the wall-clock record of §Cook times, compared by its
assets only: the manifest holds no times, so it is compared byte for byte.
Results land by plan index, the font's glyphs by index, and no hash map's
order reaches a file.

The one cause found that broke this was in basisu: its UASTC RDO pass
(`uastc_rdo: Some(_)`, the default) keeps a hash map of selector bit
sequences keyed on a struct of a u64 and a u32, and hashed all sixteen
bytes, four of them padding that was never written. The padding held
whatever the stack held, so a lookup could miss a key the pass had
inserted, and the blocks it chose followed what the thread had run
before. It showed on CI's two-vCPU Linux runner (gcc) as a one-thread cook
differing from an all-cores one in 2 of 5 runs; on the M4 Pro (clang) the
garbage happened to repeat, and 45 cooks at 2, 8 and 12 threads, 26 under
x86-64 Linux and 9 runs of the test binary under load all matched.
valgrind named it (`uastc_rdo_blocks`, "uninitialised value ... created
by a stack allocation", 8,282 errors from 12 contexts, all that map).
Upstream fixed it by giving the padding a name and a zero
(`selector_bitsequence::m_pad`); basis-universal-sys 0.3.1 bundles basisu
1.16, from before, so the workspace patches in
`third_party/basis-universal-sys` (the crate with that one change, only
the files its build compiles) until a release carries it; valgrind then
reports no error, and none before or after reached a written byte (the
encodes written to files). Building the C++
with `-ftrivial-auto-var-init=zero` is no substitute: gcc leaves the
temporaries the map is called with uninitialised (valgrind still reports
every context). The fix does not move the M4 Pro's bytes.

Two caveats the integrator must decide on:

- The C++ encoders (basisu, meshoptimizer) are compiled by `cc` with the
  profile's optimisation level and clang's default `-ffp-contract=on`. A debug
  build of basisu (`-O0`) writes different UASTC bytes than a release build,
  and an arm64 host (FMA) can differ from x86-64. Stamps stay correct (they
  describe inputs), but "same bytes" holds per cooker build, not across
  hosts. Pinning `[profile.dev.package.basis-universal-sys] opt-level = 3`
  (and the same for `meshopt`, `fdsm`) in the workspace makes debug and
  release agree and makes the debug test run ~10× faster; cross-host
  identity would need `-ffp-contract=off` for the C++ crates
  (`.cargo/config.toml` `[env]`) and a check on Linux x86-64.
- `toktx` output depends on its version, which is part of every texture stamp
  (toktx 4 prints it on stderr; the stamp read stdout only and held an
  empty version until P4).

## Tests

`crates/og-cook/tests/template.rs` (integration, on the template; write
evidence to `$OG_EVIDENCE_DIR/M4/`), `crates/og-blob/tests/blob.rs`.
Run them in release: a debug build compiles basisu at `-O0` and one cook
takes about a minute.

```sh
export CARGO_TARGET_DIR=$PWD/target/agent-assets
cargo test --release -p og-blob -p og-cook
cargo check -p og-cook --no-default-features --features runtime --target wasm32-unknown-unknown
cargo run --release -p og-cook --bin og-cook -- templates/game --report report.md   # cooks into templates/game/cooked
```

| Test | Asserts |
|---|---|
| `cooking_twice_gives_identical_bytes` | two fresh cooks (all cores, one thread) write identical files (the cook times aside: the same assets, their times masked); a difference names every file that differs, each with its size and first differing byte, and the manifest's differing lines; cooked ≤ 60 % of the sources; writes the report and the two hashes |
| `every_thread_count_cooks_the_same_bytes` | the template made small (trickster.glb out, the two textures at 128², the font's charset ASCII and its characters: 13 outputs of every kind the template has, both texture encoders, the font's own glyph pool) cooked 30 times, ten each on 1, 2 and 8 threads with the three series running at once, so each cook shares the machine with two others: every file of every cook is the first one-thread cook's bytes, the cook times masked as above (9 s in a dev build on the M4 Pro; a cook whose font stamp moved with the thread count failed it on the first two-thread run, naming the manifest's hash line) |
| `a_gltf_edit_recooks_exactly_its_outputs` | an unchanged project cooks nothing and writes nothing, each asset reporting its recorded cook time and the slowest ten in order; renaming a glTF material re-cooks exactly `models/trickster.glb`, changes exactly its blob, the manifest and the cook times, where only its time moved (to this run's) |
| `a_stale_blob_is_refused_with_both_stamps` | an old blob behind a new manifest is refused with the path and both stamps (`og_core::ErrorKind::Schema` through the API); wrong type and unknown ids are refused |
| `meshes_keep_their_triangles_and_bounds` | per primitive: LOD 0 triangle count equals the source's, decoded vertex bounds within 1 % of the source accessor's, LODs shrink, normals unit length; per model: decoded default-scene bounds within 1 % of og-render's own glTF loader |
| `textures_decode_close_to_their_sources` | transcoded level 0 against the source pixels: ETC1S ≥ 30 dB (glTF base colour ≥ 27 dB), UASTC ≥ 35–38 dB; a row-flipped decode scores worse (orientation) |
| `sounds_fonts_and_scenes_load_as_authored` | QOA SNR > 20 dB and the WAV's length; the charset, the extra characters, inside/outside of a glyph's distance field; scene entities, nesting, instances, the Transform rows' values |
| `animation.rs` (separate file) | every cooked clip sample of `walker.glb` (source rates and 60 Hz) and `trickster.glb` equals an independent glTF sampler within 1e-5; skins, inverse binds, joints, weights and morph deltas follow the cooked vertices; markers from extras and import settings, bad ones refused; a cooked walker plays in og-anim; evidence `anim-cook-resample.json` |
| `anim_compress.rs` (separate file) | every committed animated model cooked f32 and compressed: every frame of every channel within the tolerance, every node within 0.5 mm per metre of the skeleton, smaller samples, the same skins and morphs; the trickster posed mid-clip by each cook within og-tests' default frame tolerance; the settings parse and enter the stamp; evidence `P4/cook/anim-compress.json` and the two frames. Resident: a compressed model's loaded clips take exactly the bytes of its blob's sample sections (less than the frames as f32), every frame read from the keys has the bits of the frames written out from the format's description, and og.anim playing every clip at speeds 1, 0.37 and −1 gives the same world hash tick for tick from the keys as from those frames |
| `sound_formats.rs` (separate file) | an MP3 (1 s, 44.1 kHz stereo) and an Ogg Vorbis (0.5 s, 22.05 kHz mono) fixture cook to `og.Sound` blobs with the source's rate, channels and length (within 3 %), whose QOA decodes to the tone (RMS, zero crossings of the first channel); an undecodable `.mp3` is refused naming the file and the format |
| `sound_chunks.rs` (separate file) | a 12.5 s WAV with `sound { stream true chunk 1.5 }` cooks to 8 chunks whose head and chunk blobs laid end to end are the bytes of the same WAV cooked whole, and go into og-audio's bank; nothing re-cooks unchanged, a chunk length edit re-cooks only that sound and removes the old chunks; the 1 MiB threshold and `stream false`; bad settings refused by line; a chunked sound in a stream group packs its head, its chunks stay files of their own, and the group carries the whole sound's shape |
| `vox.rs` (separate file; and `cook::vox`'s unit tests) | a `.vox` with `model { voxScale 0.25 voxOrigin bottom }` cooks to those bounds (its settings file sorts after it, so the conversion waits for it); an unchanged project cooks nothing, a settings edit re-cooks the model; a bad `voxOrigin` fails it naming the line; every one of the 48 signed axis permutations an `nTRN` frame holds keeps every triangle counter-clockwise about its normal (the 24 mirrors failed with the flip removed); `voxScale` and `voxOrigin bottom` place one and two models; the committed `tests/fixtures/vox/pivot.vox` (the test writer's: a 3 x 2 x 5 model of five isolated voxels placed as is, turned about z and y, mirrored in x, its axes cycled) puts every voxel in MagicaVoxel's cell (a ceiling pivot fails it); the same voxels with and without a scene graph cook to the same bounds, centred and with `voxOrigin bottom` (the unit test: the same glTF bytes, for even, odd and one-voxel sizes; the old reading put a graph-less model's corner at the origin, FINDINGS #27 of `examples/lantern-vale`); an animated file (three graph-less frames, and an `nSHP` keyed by `_f` beside a static part) cooks to plain mesh nodes, no skin (no frame moves, so none has a joint), `voxClip`/`voxFps` clips or the default `frames`, visibility channels showing one frame's mesh node at a time (no transform channel, every node at rest scale 1, the frames after 0 hidden at rest; compressed at the loosest tolerance, the same flags), no primitive skinned, the bind-pose bounds frame 0's (the model cook's bounds leave out the nodes hidden at rest); a keyed transform (a block moved between frames 0 and 2 beside a static base) cooks to one joint with a STEP translation channel at `voxFps` and a visibility channel on its mesh node, its places the keys' in glTF's frame, bound to the block's joint (the unit tests: the keys sorted by `_f`, a turn at frame 1 splitting the bar into two runs, the run before the turn, which never moves, a plain mesh node without a joint, a folded frame holding its nearest shown place, `voxOrigin bottom` over every frame's place, keys that change nothing the same glTF as one key); `voxClip` past the last frame or on a static file, a bad `voxFps` and a repeated clip are refused by line |
| `flipbook.rs` (separate file) | atlases into sheets, and an `array true` sheet's frames cut into its texture array's layers texel for texel (broken once by cutting every layer from the texture's top rows), frames of two sizes refused; such a sheet and a texture array in a streaming group pack the sheet, the array and every layer (each naming the group, the pack's bytes the files'), the group's memory counting the layers, the sheet's texture staying in the first group |
| `fx.rs` (separate file) | a `.cube` and an emitter cook and load with every line reaching its field; an emitter's `array` line (its layers and mode, `round`), a missing array refused; a streaming group holding a LUT, its memory the LUT's 32³ RGBA8 |
| `textures_p4.rs` (separate file) | an RGBA8 KTX2 tagged `ru`/`bgra` cooks to the source texels exactly; an `rd` RGBA8 or Basis KTX2 is taken byte for byte; a Basis KTX2 tagged `ru` is re-encoded the right way up (53.9 dB against 23.7 as stored); a texture array's layers cook in order, each also a texture, upload as one `D2Array` texture whose layers read back exactly (RGBA8) or in the device's block format (BC7 on Metal), counted in the frame's texture bytes; layers of two sizes refused; with `toktx` on PATH (else skipped with the reason printed), the template's two textures cook through it (40.8 and 45.9 dB, `rd`, its version in the stamp) and toktx's own `--lower_left_maps_to_s0t0` (`ru`) output is flipped back (40.3 dB); evidence `P4/cook/textures-*.json` |
| `retarget.rs` (separate file) | fx-showcase's animation library retargeted at cook time into `strider.glb` (Mixamo names, armature scale 0.01, a rig file) and `stout.glb` (Unreal names, detected): the clips and markers arrive, and on every frame the library's walk plants a foot the character's foot is on its own ground within 2 mm and does not slide (0 m measured; 2.9 cm without foot IK); an edit of the rig file re-cooks only the model naming it, an edit of the library's settings both; a rig naming a missing node fails with the file and line; evidence `retarget-contacts.json` |
| `cooked_models_render_the_gltf_reference_scene` | og-tests' `gltf-reference` scene with the models swapped for cooked ones renders within RMSE 0.02 of `crates/og-tests/references/<tier>/gltf-reference.png` (0.0043 on Metal) |
| `compressed.rs` (separate file) | the template's Basis textures reach each block family the GPU takes (BC7/BC5, ASTC 4x4, ETC2/EAC RG11) by kind, with sRGB only on colour and every cooked level at its exact size; RGBA8 without a renderer or with compression off; stone boards near and far (small levels) draw within RMSE 0.012 and 0.5 % outliers of their RGBA8 frame, which the normal map read flipped exceeds; the cooked models with compressed textures stay within RMSE 0.02 and 1 % outliers of `gltf-reference`; `FrameStats::texture_bytes` shrinks (evidence under `tests/evidence/P2/render/`) |
| `environment.rs` (separate file; and og-render's `a_prefiltered_chain_is_taken_as_is`) | `a_prefiltered_ktx2_chain_is_taken_as_it_is`: the studio's own baked five levels written as an RGB9E5 KTX2 chain with `prefiltered true` cook back texel for texel (without it they are filtered again), its irradiance within 0.5 %; a one-level file, another size, `samples`, a non-KTX2 source and a bad flag are refused. Then: a `type environment` with og-tests' 64 × 32 studio `.hdr` (size 32, intensity 1.5, rotation 30, 64 samples) cooks into exactly og-render's `bake` of the same image and settings and loads back through `Bundle::load_environment`; the image is not listed as ignored; an unchanged project cooks nothing, an edited image recooks the map; a missing or non-HDR source, a bad size and an unknown key are refused by name. The studio cut into a 64² cube and given as an RGBA16F KTX2 (Zstandard) cooks into exactly `bake_cube` of its texels and within 0.5 % mean, 3 % worst prefiltered texel and 0.5 % irradiance of the equirectangular map; the same cube as RGB9E5, BC6H (zlib) and six `.hdr` faces within their precision of it; one face edited re-cooks only its environment; a KTX2 of one face, of RGBA8, oriented `ru`, short of its faces or holding a NaN, five faces, faces of two sizes, and a source with faces are refused by what is wrong. `cook::bc6h`'s unit tests: every mode, unsigned and signed, decodes 256 random blocks to a reference decoder's texels (bcdec; it agreed on 2,000,000 random blocks; a swapped header bit in one mode fails) |
| `terrain.rs` (separate file) | a terrain and its heightmap, control map and layers cook into tiles that load back sample for sample, deterministically and incrementally; an EXR's metres as stored; refusals by name; a navmesh walks a terrain's ramp but not its cliff; `terrain_layers_from_texture_arrays`: `layers` and `layerNormals` cook into the arrays' ids and each layer's layers (its place by default) and load back, a terrain of separate textures has none, and a layer's own maps beside arrays, an index without its array, a non-`.ogt` or missing array, a fractional index are refused |
| `look_builtins.rs` (separate file) | a look naming `@canvas/paper` and `@palettes/handheld4`, and one extending `watercolor-wash` (which names `@canvas/watercolor`), cook with those three built-ins in the bundle under their ids and no other, loadable as a texture and a palette; the cooked look holds the ids; a second cook re-cooks nothing; a game's own `assets/@canvas/…` is refused |
| `tilemap.rs` (separate file) | a tileset `.ogt`, a hand-written map, a Tiled `.tmj` (external and embedded tilesets, flips, base64 data, offsets, parallax, properties, an `autotile` layer, objects) and an LDtk project (tiles with flips, IntGrid collision, an entity at its pivot) cook and load into og-tilemap assets equal to the sources, with resolved terrain and merged collision; a compressed Tiled layer, a missing sibling tileset `.ogt` and an unknown LDtk level are refused by name; import settings pick the unit, the level and the solid IntGrid values |

Each covered path was broken once and its test failed (font glyph order by
thread count, over-broad glTF texture stamps, a skipped stamp check, the
Rust vertex decoder, the KTX2→.basis level order, the QOA decoder, materials
without base colour textures), then restored.

## What phase 2 owes

- (Fonts: kerning, shaped forms keyed by glyph id, `runtime true` for
  characters outside the charset and msdfgen's error correction landed in
  the phase 4 UI text round; `tests/template.rs` checks the template
  font's pairs and bounds its corner clashes.)
- **`.vox` frame rate**: `voxFps` gives it, and stays: the file does not
  carry one. MagicaVoxel's format documents (`MagicaVoxel-file-format-vox.txt`
  and its extension, ephtracy/voxel-model) define no rate in any chunk;
  `nTRN` and `nSHP` frames carry only a frame index (`_f`), and the one
  animation setting 0.99.7.2 writes is the playback range (`META`'s
  `_anim_range`, which the cook reads). The editor's playback time step
  is a setting of the editor, not of the file: ogt_vox (opengametools,
  whose reading this cook follows) reads `_anim_range` and no rate.

Done since phase 1 (kept here for the record): the measured M4 "glTF edit
→ reload ≤ 300 ms" run is filed (`tests/evidence/P2/tooling/m4-m5-tooling.md`:
8 reloads, worst 112.8 ms in `og dev`, 251 ms from the write; owed row 34);
terrains stream tile by tile (og.Terrain schema 2, the phase 3 streaming
round: the runtime decodes the wanted tiles off the frame within a budget);
the `runner` and `match3` archetypes exist (`templates/archetypes/`);
animation sample compression and sparse morph targets, audio streaming
chunks, KTX2 orientation and swizzle, texture arrays and the `toktx`
fallback's test landed in P4 (§Animation samples, the importers above,
`tests/evidence/P4/cook/`). The cook's third round: environments from KTX2
cube maps (RGBA16F, RGB9E5, BC6H decoded at cook time and the other HDR
formats) and from six faces, baked like the equirectangular path
(`tests/environment.rs`); sound chunks fetched nearest the playhead first,
ahead of streaming packs by priority, four reads in flight (og-audio's
§Chunked sounds, og-runtime's `tests/stream_fetch.rs`); flipbooks and
their arrays in streaming groups (§Streaming groups); each asset's cook
time recorded, `og cook --report` printing the slowest ten and `og cook
--watch` timing each change against the 300 ms reload budget (§Cook
times); `.vox` mirrors flipping their winding and `model { voxScale
voxOrigin }`. The cook's fourth round: materials and terrain layers
sample texture arrays (`StandardMaterialDesc.mapArray`/`normalMapArray`
with `og.Mesh.materialLayer`, one draw per material whatever the layers;
`type terrain`'s `layers`/`layerNormals`, two sampled textures instead of
up to eight: og-render's §Array maps); a KTX2 cube's own prefiltered mip
chain taken as it is (`prefiltered true`); the `.vox` pivot checked
against MagicaVoxel's convention for odd sizes turned and mirrored (it
held: a committed fixture, the formula above).
