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
prompt, IDs and checksums) are in the `origin-generated-assets` bucket
(`docs/ci.md` §The model bucket) under the key below.

| File | Triangles | Source key | Source sha256 (`model.glb`) |
|---|---|---|---|
| `column.glb` | 5,863 | `meshy/model/showcase-kit-column-v1/` | `7d9875c4c750cdb7a7a3714e4704e426bd556be67409cbf8e98a3735fe292285` |
| `column_broken.glb` | 5,927 | `meshy/model/showcase-kit-column_broken-v1/` | `864fe2ef66d80659369e9172ef5d1ada9b3f441d1bd912981355cd291538da90` |
| `arch.glb` | 9,038 | `meshy/model/showcase-kit-arch-v1/` | `bdb1ec54f4ea7f7a46321531c2081c4b6278ad09d2b177fe0b38e924e2565425` |
| `watchtower.glb` | 14,170 | `meshy/model/showcase-kit-watchtower-v1/` | `68fd5be60fc89fd66e1e7edafdf77ce788ee2bce63e3ebf8c6a468de4e9b5fc6` |
| `boulder.glb` | 4,077 | `meshy/model/showcase-kit-boulder-v1/` | `c6943d632184979e4b114e65c4d6403e9515bb04375f2064bcb43d4379e06e3e` |
| `pine.glb` | 9,037 | `meshy/model/showcase-kit-pine-v1/` | `3b7220f86ca3e135e07d2b87f0fd621afbc8512b2d30b611b8db73c275fbacac` |

Each committed model is its source with the emissive map dropped (Meshy
writes a near-black one), its textures resized to 1024 px and re-encoded as
JPEG, then pruned and deduplicated with glTF Transform 4.5.1.

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

`color` is the diffuse map, `normal` the OpenGL normal map, `orm` the
occlusion (red), roughness (green) and metal (blue) map.
