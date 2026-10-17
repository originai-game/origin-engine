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
```

`og-cook <project> [--threads N] [--report out.md] [--json out.json] [--schemas schema.json]`
is a development binary; `og cook` (og-cli) is the user-facing command and
calls `cook_project`.

## Features

| Feature | Default | What it adds | Targets |
|---|---|---|---|
| `runtime` | via `cook` | `runtime` (Bundle, loaders), `ktx2`, `meshopt_codec`, `qoa`. Pure Rust: og-blob, og-core, og-render, glam, bytemuck, miniz_oxide | every target; `cargo check -p og-cook --no-default-features --features runtime --target wasm32-unknown-unknown` |
| `transcode` | yes | `CookedTexture::to_texture_data` for Basis textures (basisu's C++ transcoder, ruzstd) | native |
| `cook` | yes | `cook_project` and the importers (gltf, image, meshopt, mikktspace, fdsm, ttf-parser, hound, ruzstd) | native |
| `basis` | yes | the basis-universal crate (basisu 1.16) as the texture encoder; without it the cook uses `toktx` from PATH | native |

## Pipeline

1. **Walk** `assets/` (sorted, dot-files skipped). By extension: `.glb`/`.gltf`
   models, `.png`/`.jpg`/`.jpeg` textures, `.wav` sounds, `.ttf`/`.otf` fonts,
   `.ogt` scenes. `<file>.ogt` of `type import` holds `<file>`'s import
   settings. Files a `.gltf` references are part of that model. Anything else
   (licences, notes) is listed as ignored.
2. **Plan**: every output gets a cook stamp (below). An output whose stamp the
   manifest already holds, with its file present and its bytes' hash
   unchanged, is up to date and not cooked.
3. **Cook** the stale outputs on a thread pool (`CookOptions::threads`, 0 =
   all cores); results are collected by plan index, so scheduling never
   changes the bytes.
4. **Write** outputs whose bytes changed (unchanged files are not touched),
   delete outputs of removed or failed sources, write
   `cooked/.cook.ogt` when it changed. Return the `Report`.

### Importers

| Source | Output (`cooked/…`) | What happens |
|---|---|---|
| glTF 2.0 | `<path>.ogb` (`og.Model`) + `<path>.tex<n>.ogb` per texture use | Per primitive (glTF order): positions, normals (flat when missing), uv0, colour0, tangents (MikkTSpace for normal-mapped primitives that lack them, via unwelding); weld; quantise (position u16×3 over the bounds, normal oct snorm16×2, uv u16×2 over the uv range, colour u8×4, tangent oct snorm16×2 + sign); weld again; meshopt vertex-cache and overdraw (1.05) ordering; LOD chain by `meshopt_simplify` (3 levels, each half the previous, max error 5 % of the extent, stops when a level saves < 5 %); vertex-fetch reorder over all LODs; meshopt vertex codec v1 and index codec v1. Materials (metallic-roughness, emissive strength, alpha mode, double-sided) plus a default material, nodes in topological order, scenes and their roots. Animations, skins and morph targets are counted and reported, not cooked. |
| PNG/JPEG | `<path>.ogb` (`og.Texture`) | Decoded to RGBA8, then KTX2: colour → ETC1S/BasisLZ, sRGB, quality 128; `kind normal` → UASTC level 1 + RDO 1.0 + Zstandard, linear, tuned for normal maps; `kind linear` → the same without the normal-map tuning. Full mip chain. In a glTF, base colour and emissive are colour, metallic-roughness and occlusion linear, normal normal. |
| WAV | `<path>.ogb` (`og.Sound`) | 8–32-bit integer or 32-bit float PCM, 1–8 channels, to 16-bit, then QOA (in-crate encoder: 3.2 bits per sample). |
| TTF/OTF | `<path>.ogb` (`og.Font`) | The import settings' charset, rendered by fdsm as MTSDF (RGB multi-channel, A true SDF) at `em_px` with `px_range`, shelf-packed into pages, each page a lossless KTX2 (RGBA8 + zlib); a glyph table (advance, plane bounds in em, atlas rectangle) and ascender/descender/line gap. |
| scene `.ogt` | `<path>.ogb` (`og.Scene`) | Entities depth-first (parent before child), `model` references checked against the project's assets, one packed table per component type keyed by its reflected name. |

### Import settings

```text
type import
version 1
---
texture {
  kind normal          # color (default) | normal | linear
}
font {
  charset latin cjk-punct   # ascii | latin | punct | cjk-punct | kana | all
  chars "开始游戏"           # extra characters (UI strings)
  em_px 32
  px_range 4
  page_size 1024
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
| `og.Model` | `ModelMeta` (counts, default scene, skipped animations/skins, default-scene bounds) | `primitives` `PrimitiveRow` (material, attribute bits, vertex count/stride, byte ranges in `vertices`/`indices`, index count, LOD range, dequantisation offset/scale), `lods` `LodRow` (index range, simplify error), `meshes` `MeshRow` (primitive range, name), `materials` `MaterialRow`, `textures` `TextureRefRow` (asset id, kind), `nodes` `NodeRow` (parent, mesh, name, TRS), `scenes` `SceneRow`, `roots` (u32), `strings` (UTF-8 pool), `vertices`, `indices` (meshopt streams, 4-aligned per primitive) |
| `og.Texture` | `TextureMeta` (size, levels, kind, codec, colour space, alpha, supercompression) | `ktx2`: a complete KTX2 file, 16-aligned |
| `og.Sound` | `SoundMeta` (rate, channels, frames) | `qoa`: a complete QOA file |
| `og.Font` | `FontMeta` (units/em, em_px, px_range, ascender, descender, line gap, glyph and page counts, page size, uncooked code points) | `glyphs` `GlyphRow` sorted by code point, `atlas.<n>` KTX2 RGBA8/zlib |
| `og.Scene` | `SceneMeta` (counts, name) | `entities` `EntityRow` (name, parent, instance), `refs` `RefRow` (asset ids), `components` `ComponentRow` (name, layout hash, rows, inferred), `strings`, `c.<Type>` rows (`u32` entity index + packed fields), `s.<Type>` schema JSON |

KTX2 files are standard: ETC1S as BasisLZ with basisu's DFD and global-data
layout, UASTC with Zstandard (scheme 2), lossless RGBA8 with zlib (scheme 3).
The basis-universal crate only emits `.basis`, so `ktx2::from_basis` rebuilds
the KTX2 container from the `.basis` bits exactly as basisu's own
`create_ktx2_file` lays it out, and `ktx2::to_basis` is the inverse the
native transcoder needs (its Rust wrapper reads `.basis` only).

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
| glTF texture | texture options and encoder, the image's bytes, the texture kind |
| texture | texture options and encoder, the file and its import settings, the kind |
| sound | the file |
| font | the effective font settings, the file and its import settings |
| scene | the file, the schema set, each referenced id and whether it exists |

So a material edit in a glTF re-cooks its model blob and nothing else (its
textures keep their stamps); a texture option change re-cooks textures only;
a new cooker version re-cooks everything.

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
the template does; full CJK coverage needs runtime glyph generation into a
paged atlas (phase 2) or a smaller `em_px`.

## Determinism

The same sources, options and **cooker build** give byte-identical `cooked/`
(the test cooks the template twice, on all cores and on one thread, and
compares every file). Two caveats the integrator must decide on:

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
- `toktx` output depends on its version, which is part of every texture stamp.

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
| `cooking_twice_gives_identical_bytes` | two fresh cooks (all cores, one thread) write identical files; cooked ≤ 60 % of the sources; writes the report and the two hashes |
| `a_gltf_edit_recooks_exactly_its_outputs` | an unchanged project cooks nothing and writes nothing; renaming a glTF material re-cooks exactly `models/trickster.glb`, changes exactly its blob and the manifest |
| `a_stale_blob_is_refused_with_both_stamps` | an old blob behind a new manifest is refused with the path and both stamps (`og_core::ErrorKind::Schema` through the API); wrong type and unknown ids are refused |
| `meshes_keep_their_triangles_and_bounds` | per primitive: LOD 0 triangle count equals the source's, decoded vertex bounds within 1 % of the source accessor's, LODs shrink, normals unit length; per model: decoded default-scene bounds within 1 % of og-render's own glTF loader |
| `textures_decode_close_to_their_sources` | transcoded level 0 against the source pixels: ETC1S ≥ 30 dB (glTF base colour ≥ 27 dB), UASTC ≥ 35–38 dB; a row-flipped decode scores worse (orientation) |
| `sounds_fonts_and_scenes_load_as_authored` | QOA SNR > 20 dB and the WAV's length; the charset, the extra characters, inside/outside of a glyph's distance field; scene entities, nesting, instances, the Transform rows' values |
| `cooked_models_render_the_gltf_reference_scene` | og-tests' `gltf-reference` scene with the models swapped for cooked ones renders within RMSE 0.02 of `crates/og-tests/references/<tier>/gltf-reference.png` (0.0043 on Metal) |

Each covered path was broken once and its test failed (font glyph order by
thread count, over-broad glTF texture stamps, a skipped stamp check, the
Rust vertex decoder, the KTX2→.basis level order, the QOA decoder, materials
without base colour textures), then restored.

## What phase 2 owes

- **GPU-compressed textures**: og-render takes RGBA8 only, so the runtime
  transcodes to RGBA8 (native, `transcode`). The renderer should accept
  BC7/ASTC/ETC2 and the cooked mips; on the web the host transcodes with
  Binomial's wasm transcoder (the Rust runtime has no Basis transcoder on
  wasm32).
- **Animation**: skins, clips, morph targets (counted and reported today).
- **Fonts**: kerning and shaping (rustybuzz), runtime glyph generation for
  CJK, `msdfgen`-style error correction.
- **Hot reload**: the M4 row's "glTF edit → `og cook` → reload in ≤ 300 ms
  with world state kept" belongs with the host's bundle swap (og-cli /
  og-host); the cook's part (an incremental re-cook of one model in ~10–50 ms)
  is here.
- Archetype templates (`runner`, `match3`) and their assets.
- KTX2 orientation/swizzle metadata, texture arrays, cube maps (IBL
  prefiltering at cook time), audio streaming chunks, navmesh baking.
- `toktx` fallback is implemented but untested here (no `toktx` on this
  machine); with neither encoder the cook fails with a message naming both.
