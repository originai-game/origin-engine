# The scenes' set dressing

What `crates/og-tests/src/kit.rs` places around a scene's subject. Every
file here is sized for the 1280×720 reference frames: textures at 1024 px
(occlusion-roughness-metal maps at 512 px), JPEG quality 82 to 86.

## Models

Generated for origin-engine through the OriginGame gateway (2026-11-17):
a concept image by `generate_image` (gpt-image-2.6-sunburst), then Meshy
image-to-3D by `generate_3d` (meshy-6, triangles, PBR). The gateway returned
no price for any call; each Meshy task reports 30 credits. The sources (the
concept, the 2048 px model as Meshy made it, and the task receipt with its
prompt, IDs and checksums) are in the generated-sources archive
(`docs/ci.md` §Generated sources) under the key below.

| File | Triangles | Source key | Source sha256 (`model.glb`) |
|---|---|---|---|
| `column.glb` | 5,773 | `meshy/model/showcase-kit-column-v2/` | `c4ed6bac409af31ee74a62e5af97f80031537d9c137173efd1a5da96e07243ca` |
| `column_broken.glb` | 5,891 | `meshy/model/showcase-kit-column_broken-v2/` | `bbc9278032ae1d912cb6e0fb6ffd174fa29b0ac48b7f265a1b1b51b2d7acbc55` |
| `arch.glb` | 9,038 | `meshy/model/showcase-kit-arch-v1/` | `bdb1ec54f4ea7f7a46321531c2081c4b6278ad09d2b177fe0b38e924e2565425` |
| `watchtower.glb` | 14,170 | `meshy/model/showcase-kit-watchtower-v1/` | `68fd5be60fc89fd66e1e7edafdf77ce788ee2bce63e3ebf8c6a468de4e9b5fc6` |
| `boulder.glb` | 4,077 | `meshy/model/showcase-kit-boulder-v1/` | `c6943d632184979e4b114e65c4d6403e9515bb04375f2064bcb43d4379e06e3e` |
| `pine.glb` | 9,037 | `meshy/model/showcase-kit-pine-v1/` | `3b7220f86ca3e135e07d2b87f0fd621afbc8512b2d30b611b8db73c275fbacac` |
| `townhouse.glb` | 13,285 | `meshy/model/showcase-weather-townhouse-v1/` | `e5b8117dbbc225d4d6d82dae29b55c4b0c7cacf5250b7eb04e066e5639cb10d5` |
| `torii.glb` | 7,394 | `meshy/model/showcase-shadows-torii-v3/` | `2afccbe818735ffd528590662f0b6ddbe5df563e38942fc882bcba346e0f96cc` |
| `toro.glb` | 7,088 | `meshy/model/showcase-shadows-toro-v1/` | `ad620780579b0753bf44166609f116561481f88faac942fe3870c6c50898aca9` |
| `stone_bench.glb` | 8,000 | `meshy/model/showcase-shadows-stone_bench-v1/` | `730a185c72cd0f01f9e7bc3f0c0ce762a9115d53c0ea7a58abc6d516b2f69989` |
| `streetlamp.glb` | 3,000 | `meshy/model/showcase-weather-streetlamp-v1/` | `c5a2dd3a0f1544790517fb9d05ff35b8e41c79f3caab72421a86abca1f29c018` |
| `car.glb` | 11,598 | `meshy/model/showcase-weather-car-v1/` | `7f83959ae7d1156128d96488a07582525e38789557fa1136be694487fa6c8413` |
| `adobe.glb` | 8,941 | `meshy/model/showcase-weather-adobe-v2/` | `34c3eb207e6c69df6b34a465724b1dffd092bd487d407c4ea597433d07611782` |
| `counter.glb` | 11,820 | `meshy/model/showcase-weather-counter-v1/` | `33000305a33547bdadd9754052cc04e0e29b50cdb736662e2fa4527f373acf09` |
| `winebarrel.glb` | 3,730 | `meshy/model/showcase-lighting-winebarrel-v1/` | `e76e56fbbb562df7f03e3ee02dfd4a0717bf69c75a21e85d44632e0c4b69203f` |

The weather set's five (townhouse, street lamp, car, adobe house, a
market counter of produce) are
Meshy text-to-3D instead: `generate_3d` `text_preview` (meshy-6, triangles,
remeshed) then `text_refine` with PBR, no concept image; each bucket key
holds the model and the receipt (prompts, operation and task IDs). The
adobe is a second preview: the first stood on a round plinth. The street
lamp's textures are 512 px. `car.glb`'s material carries
`KHR_materials_clearcoat` (factor 1, roughness 0.05), added with glTF
Transform: the lacquer the rain beads on.

The lighting set's oak wine barrel is Meshy text-to-3D the same way
(`text_preview` at 4,000 triangles, then `text_refine` with PBR; 30
credits), its textures at 512 px: the market's and the cellar's barrels,
where the yard's blue steel drum (`barrel.glb`) would not belong.

The shrine set (torii, stone lantern, granite bench) is Meshy text-to-3D
the same way, its textures at 512 px: the lighting and shadows scenes'
court and avenue. The torii is a second version: the first came out with
four posts (a gazebo, not a gate) and stays in the bucket under `…-v1/`;
this one, from a reworded prompt, has two round posts, a kasagi
with upturned ends and a nuki.

The two columns are a second version: their first concepts were marbled in
large orange and grey patches, which read as camouflage on the shafts, so
each concept was repainted by `edit_image` as even pale limestone with dirt
in the flutes and moss at the base, and the model made again from it. The
first versions stay in the bucket under `…-v1/`. Their material's
`baseColorFactor` is 0.6 (linear): the limestone as painted was pale enough
to glow under a night's light and to look dry in rain, so its albedo is
two fifths lower (about a fifth as an sRGB value), a mid limestone; the
textures are unchanged.

Each committed Meshy model is its source with the emissive map dropped (Meshy
writes a near-black one), its textures resized to 1024 px and re-encoded as
JPEG, then pruned and deduplicated with glTF Transform 4.5.1.

`ladder.glb` (84 triangles) is from the platform's production asset catalog
(release `catalog-20261107-kenney-particles-v1`, group
`grp_840856f60c3e5156d0`): KayKit's Medieval Hexagon Pack 1.0 by Kay
Lousberg, CC0 1.0, no attribution required. It is the pack's
`ladder.gltf` (sha256
`de1f471e8c11615388a9caad79aef97414903390ff54924b33bff2547bc19e72`), its
buffer and its 16 KB palette texture, each checked against the catalog's
sha256, packed as one GLB with glTF Transform 4.5.1.

From the same catalog release (`catalog-20261107-kenney-particles-v1`),
CC0 1.0, no attribution required, each file checked against the
catalog's sha256 on download:

- `crate.glb` (6,576 triangles): Poly Haven's Wooden Crate 01, group
  `grp_4824f0a4cc69401f82`, the 1k glTF (sha256
  `fd9c6073acfc671e12b64666053f2805ae33aff5d3dbbcfe5c97c08c06644927`).
- `barrel.glb` (1,473 triangles): Poly Haven's Barrel 03, group
  `grp_175cd071a79cb720cb`, the 1k glTF (sha256
  `80e2faf48b7423bb522b573e459d99c5656a16763285ec8268e957fc7d41f9b8`).
- `palm.glb` (482 triangles): Kenney's Pirate Kit `palm-detailed-bend.glb`,
  group `grp_821b501ff5b37b9d37` (sha256
  `9bc339e2942d6fba9007caa9fa4301a6a08f4b134511bea65ce236ba2910e7f8`),
  its external `colormap.png` packed in.

The crate and barrel are packed as GLBs with their textures at 1024 px
JPEG, pruned and deduplicated with glTF Transform 4.5.1.

## Surfaces

From the platform's production asset catalog (release
`catalog-20261107-kenney-particles-v1`), each checked against the catalog's
sha256 on download. Poly Haven, CC0 1.0
(<https://creativecommons.org/publicdomain/zero/1.0/>); no attribution is
required.

| Files | Catalog group | Poly Haven asset | Source sha256 (the 2k colour map) |
|---|---|---|---|
| `ground/grass.{color,normal,orm}.jpg` | `grp_b9a4454916bfef3264` | Aerial Grass Rock | `1a2ddfa4652adbe00447b940c40019d46f4b88245bc1b341cdf622c2110b7bc5` |
| `ground/cobbles.{color,normal,orm}.jpg` | `grp_776009be378c7de71b` | Cobblestone 01 | `c224786e89cae7e2835a8dfafa6473c47d4d6deac0cca1db358002b8b6c4877e` |
| `ground/rock.{color,normal}.jpg` | `grp_28a4c1a58ffc4b45d1` | Rock Face 03 | `644ee79df7ffbf5f6afc330b64dd3f33794bab8820915fe61ffe0c88ecfd7ece` |
| `ground/planks.{color,normal,orm}.jpg` | `grp_5f733073766da2cdff` | Brown Planks 03 | `1140ceac531c2dfe2e714357e34cd81cce62878ced24fa711b87c4ca24416058` |
| `ground/stone.{color,normal,orm}.jpg` | `grp_65c1eae191c77733e1` | Broken Wall | `35615ed619b14ab8991205e887a24cadcf8edffdf15b36b153421604b9ba29ce` |

`color` is the diffuse map, `normal` the OpenGL normal map, `orm` the
occlusion (red), roughness (green) and metal (blue) map.

The stone's normal map is not Poly Haven's file as published: Broken Wall's
`nor_gl` is sRGB-encoded in every format (its 1k EXR, sha256
`1108d10ca3cdd4392020c679df6b880a68a740fa8d4a8ec65cc92b262c583a48`,
averages 0.71, 0.72, 0.95, a lean of about 35°). The kit's map is that EXR
decoded from sRGB to linear, each texel renormalised to unit length, then
written as an 8-bit JPEG (quality 86, 1024²). `kit::tests` holds every
kit normal map to within 4° of +Z on average. The rock is a
terrain layer only (`kit::layer_maps`), which takes no `orm`.

## Environment maps

`../env/kloofendal.hdr` and `../env/spruit.hdr` are Poly Haven skies from
the same catalog release (`catalog-20261107-kenney-particles-v1`), CC0 1.0,
no attribution required. Each is the catalog's 2k Radiance HDR (checked
against its sha256 on download), resized to 1024 × 512 with ImageMagick
and written as Radiance RGBE again; the 2k sources stay in the catalog.

| File | Catalog group | Poly Haven asset | Source sha256 (2k) | Committed sha256 |
|---|---|---|---|---|
| `env/kloofendal.hdr` | `grp_992b5f9ad847d66702` | Kloofendal Partly Cloudy (pure sky) | `5244534e9cf5b606f2ff513aa00ddb161b0a4826ffd88a0d3bd03ac29247d198` | `5a36c9ffafbf22beaeb738f55a035e00f428857e977c5a2046ab389ff2989da5` |
| `env/spruit.hdr` | `grp_2c118eb3d4bb794027` | Spruit Sunrise | `be35af1e825dc7506df48bf86568f9c05f8a8552cf57edf36beb16e54534ec57` | `7a287c6684bee62d4efdbe13b617048ecb076007ab787eb088290ee6ee6659b9` |
