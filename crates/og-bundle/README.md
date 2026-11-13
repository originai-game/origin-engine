# og-bundle

The cooked bundle's formats and the runtime loaders (`docs/design.md`
§2.5): the `.ogb` blobs og-cook writes into `cooked/` (`crates/og-cook`),
read into og-render's input types and plain scene data. `cooked/` is the
only thing the runtime reads. og-runtime, og-host-web and og-host-node
depend on this crate and never on the cook; og-cook depends on it and
re-exports its modules under the same paths (`og_cook::runtime`, ...).

```rust
let bundle = og_bundle::runtime::Bundle::open(&root.join("cooked"))?;          // native; Bundle::from_manifest(text) on the web
let (model, textures) = bundle.load_model_with_textures("assets/models/ship.glb")?;  // feature `transcode`
let parts = model.add_to(&mut render_scene, &textures);
parts.instance(&mut render_scene, transform);
let scene = bundle.load_scene("scenes/main.ogt")?;                             // plain data for the host to spawn
let anim = bundle.load_animation("assets/models/walker.glb")?;                 // with og_anim::cooked::BundleAnim in scope: SkeletonAsset, ClipAssets, skin and morph streams
let map = bundle.load_tilemap("maps/room.tmj")?;                              // og_tilemap::TilemapAsset; its TilesetRef::Asset ids through bundle.load_tileset
let terrain = bundle.load_terrain("terrain/island.ogt")?;                     // CookedTerrain: og_render::TerrainData, layers naming their textures
let env = bundle.load_environment("env/studio.ogt")?;                         // og_render::EnvironmentMapData, prefiltered at cook time
```

## Features

| Feature | Default | What it adds | Targets |
|---|---|---|---|
| (none) | | `runtime` (Bundle, loaders), `format`, `ktx2`, `meshopt_codec`, `qoa`, `fx`, `look`, `lightmap`, `scene_text`, `shader`. Pure Rust: og-blob, og-core, og-render-data, og-draw, og-tilemap, og-timeline, glam, bytemuck, miniz_oxide | every target; `cargo check -p og-bundle --target wasm32-unknown-unknown` |
| `transcode` | no | `CookedTexture::to_texture_data` (RGBA8 level 0) and `to_texture_source` (every level in the device's block format) for Basis textures (basisu's C++ transcoder, ruzstd) | native |
| `basis` | no | `ktx2::from_basis`, the KTX2 container rebuilt from basis-universal's `.basis` output (og-cook's encoder; ruzstd) | native |

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
| `og.Model` | `ModelMeta` (counts, default scene, skipped animations/skins, default-scene bounds) | `primitives` `PrimitiveRow` (material, attribute bits, vertex count/stride, byte ranges in `vertices`/`indices`, index count, LOD range, dequantisation offset/scale), `lods` `LodRow` (index range, simplify error), `meshes` `MeshRow` (primitive range, name), `materials` `MaterialRow`, `textures` `TextureRefRow` (asset id, kind), `nodes` `NodeRow` (parent, mesh, name, TRS, `visible`: 0 when `KHR_node_visibility` hides it at rest; schema 2, stamped `nodes-v2`), `scenes` `SceneRow`, `roots` (u32), `strings` (UTF-8 pool), `vertices`, `indices` (meshopt streams, 4-aligned per primitive) |
| `og.Texture` | `TextureMeta` (size, levels, kind, codec, colour space, alpha, supercompression) | `ktx2`: a complete KTX2 file, 16-aligned |
| `og.TextureArray` | `TextureArrayMeta` (size, layers, levels, kind) | `layers` `ArrayLayerRow` (the layers' texture ids in `strings`), `strings` |
| `og.Sound` | `SoundMeta` (rate, channels, frames, QOA frames per chunk, chunk count; 0 0 when whole) | `qoa`: a complete QOA file, or of a chunked sound its header and first chunk |
| `og.SoundChunk` | `SoundChunkMeta` (chunk, first QOA frame, frame count) | `qoa`: the chunk's QOA frames |
| `og.Font` | `FontMeta` (units/em, em_px, px_range, ascender, descender, line gap, glyph and page counts, page size, uncooked code points) | `glyphs` `GlyphRow` sorted by key: a character's code point, a shaped form at `GLYPH_ID_KEYS` (0x110000, past Unicode) + glyph id, `kerning` `KernRow` (left, right, adjust in em) sorted by pair, `atlas.<n>` KTX2 RGBA8/zlib, `color.<n>` KTX2 sRGB RGBA8/zlib colour glyph pages (`color true`; a row's page past the atlas pages), `foreground.<n>` the colour glyphs in the text's colour, each painted black and, half the page to the right, white (after the `color` pages), `color-field.<n>` and `foreground-field.<n>` their silhouettes as linear signed distance fields (`COLOR_FIELD_RANGE` 8 px; a foreground page's field is its left half) for shadows, glow and outlines, `ttf` (the font file, with a shaped script or `runtime true`) |
| `og.Animation` | `AnimMeta` (counts, `hidden_count`; schema 2) | `nodes` `NodeRow`, `hidden` (u32 nodes hidden at rest, ascending), `skins` `SkinRow`, `joints` (u32), `inverse_binds` (f32 ×16 per joint, column-major), `clips` `ClipRow` (name, duration, rate, source rate, frames, channel and event ranges), `channels` `ChannelRow` (node, path, source interpolation, lanes, sample offset, codec, keys, packed offset), `samples` (f32), `packed` (u16 quantised keys), `events` `EventRow`, `skin_streams` `SkinStreamRow` + `skin_data`, `morphs` `MorphRow` (moved-vertex count, data and index streams) + `targets` `TargetRow` (moved range) + `morph_data`, `strings` |
| `og.Scene` | `SceneMeta` (counts, name) | `entities` `EntityRow` (name, parent, instance), `refs` `RefRow` (asset ids), `components` `ComponentRow` (name, layout hash, rows, inferred), `strings`, `c.<Type>` rows (`u32` entity index + packed fields), `s.<Type>` schema JSON |
| `og.Tileset` | `TilesetMeta` (text bytes) | `text` (the canonical tileset `.ogt`) |
| `og.Tilemap` | `TilemapMeta` (width, height, tile px, unit, layer/object/tileset/prop counts) | `layers` `MapLayerRow` (name, tileset, terrain name, parallax, offset, opacity, flags visible/collision/repeat x/repeat y/terrain, z, first cell), `cells` (u32 LE per cell, layer after layer, og-tilemap's cell encoding), `objects` `MapObjectRow` (name, kind, rect, property range), `props` `MapPropRow`, `tilesets` `MapTilesetRow` (asset id, or embedded canonical text), `strings` |
| `og.EnvironmentMap` | `EnvironmentMeta` (size, levels, texel count, the source's width and height, intensity, rotation, samples; 0 for a KTX2's own prefiltered levels) | `texels` (og-render's `EnvironmentMapData::texels`: RGB9E5 u32 LE, level after level, each the six faces +x −x +y −y +z −z, rows from the top; zlib), `irradiance` `EnvironmentShRow` × 9 (r, g, b of the irradiance-over-π harmonics) |
| `og.Terrain` | `TerrainMeta` (size, samples, tile size and count, height range, layer count, control map size, height blend, lod, cast shadows, collision, the layer arrays' ids as ranges of `strings`; version 3) | `tiles` `TerrainTileRow` (first sample, size, range, byte range in `heights`), `heights` (per tile u16 LE, zlib), `layers` `TerrainLayerRow` (texture names as ranges of `strings`, og-render's layer fields, its layers of the arrays), `strings`, `control` (RGBA8, zlib) |
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
| `og.NavMesh` | `NavMeshMeta` (cell size and height, the agent's height, radius, climb and slope, tile size, max error, origin, grid width and depth, tile count) | `columns` (a meshopt vertex stream of `NavColumnRow`, one a grid column), `spans` (a meshopt vertex stream of `NavSpanRow`; the two are the walkable heightfield), `tiles` `NavTileRow`, `verts` `NavVertRow`, `polys` `NavPolyRow`, `poly_verts` (u32) |
| `og.StreamPack` (`cooked/stream/<group>.ogb`) | `StreamPackMeta` (file count) | `files` `StreamFileRow` (name and data ranges), `names` (paths under `cooked/`), `data` (the files, each 16-aligned) |

Besides the blobs, `cooked/` holds the manifest `.cook.ogt` (og-cook README §Asset ids)
and the cook times `.cook-times.ogt` (og-cook README §Cook times), both `.ogt` text. A
cube map an environment reads is not a blob of its own: it is decoded at
cook time (og-cook README §Importers), so the runtime never sees KTX2 cubes, BC6H or
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
`og_web_texture_target` names.

## Animation and navmesh loaders

The blob schemas of animations, flipbooks, rigs and navmeshes are here
(`format`); their loaders build og-anim's and og-ai's types, so they live
in those crates and og-bundle depends on neither: `og_anim::cooked`
(`BundleAnim`, `CookedAnimation`, `anim_codec`; og-anim's README §Cooked
samples) and `og_ai::nav::cooked` (`BundleNav`). Both add their loaders
to `Bundle` as a trait's methods.
