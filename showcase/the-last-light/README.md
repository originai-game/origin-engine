# The Last Light

Hero showcase A: a cloaked wanderer with a staff and lantern on a cliff
path, a stone lighthouse on its headland sweeping its beam through the sea
mist, the sun going down into a swelling sea. An og game in TypeScript
through the public API; it runs on the native and web hosts.

```sh
og run .        # play it
og test .       # its playtests
og cook . --report
```

## Controls

| Key | Pad | What |
|---|---|---|
| Z | X | The next weather: clear, rain, storm, fog, snow |
| X | Y | The next season: spring, summer, autumn, winter |
| C | B | The next time of day: sunset, night, dawn |

Every switch blends in over a few seconds (`src/climate.ts`). The same
switches are commands, for captures and playtests: `weather:<name>`,
`season:<name>`, `time:<name>`, each with `:now` to skip the blend
(`og run . --commands "2:weather:storm;12:screenshot:storm.png"`).

## Layout

| Path | What |
|---|---|
| `src/layout.ts` | Where things stand, shared with `tools/terrain.ts` |
| `src/coast.ts` | The ground, the buildings and rocks, what grows, the sea |
| `src/wanderer.ts` | The wanderer, the staff, the lantern and the cloak |
| `src/lighthouse.ts` | The lamp and its sweeping beam |
| `src/sky.ts` | The sun, sky, clouds, wind and sea mist |
| `src/climate.ts` | Weather, season and time of day, and their switches |
| `tools/` | How the assets were fetched, generated and packed (never cooked) |

## Assets

No binary asset is in git. The models, the terrain's maps and textures and
the HUD font are pinned in `og-assets.json` (path, sha256, size,
visibility; docs/ci.md §The asset store), which also lists them in
`.gitignore`: 17 files, 50.9 MB.
Install them before the first cook:

```sh
bun scripts/assets.ts ensure showcase/the-last-light   # from the repository root
```

Everything came through the OriginGame asset gateway
(`tools/gateway.ts`, with `OG_PLATFORM_DEVELOPMENT_KEY`).
`tools/assets.json` is the ledger of the generation calls (operation, task
and media ids); `tools/catalogue.log` and `tools/pack.log` are the tools'
own output.

### Generated

Each model was generated for this project: a reference image by
gpt-image-2.6-sunburst from the prompt in `tools/refs/<name>.txt`, then a
mesh by Meshy 7.1 from that image. `tools/fetch.ts` downloads a finished
task and `tools/pack.ts` packs it for the game: the mesh welded and
simplified (meshoptimizer through `@gltf-transform/cli@4.1.1`, error
0.002, keeping the maps' seams), its 4k maps re-encoded as JPEG at the
size given. No third-party licence applies: the outputs belong to the
platform account under the gateway's generation terms.

| File | Gateway operation (Meshy task) | Reference media | Packed | sha256 (shipped) | sha256 (as generated) |
|---|---|---|---|---|---|
| `assets/models/lighthouse.glb` | `hero-a-lighthouse-v1` (task `01a1e8a3-e502-7361-8f58-8d69585bbb16`) | `9d83c712a19dea32a5a853ca06f0c1e6` | 98573 of 153671 triangles, maps at 2048 | `0bbd1aa6d12bb61af1757c3f31f41fdf34d77b81c73a6bc4aef4cf23493c75c1` | `31a269ba2eb78612d6aab55d236155b0ddc8d9efb8da65b5bf200bb7950678ab` |
| `assets/models/cottage.glb` | `hero-a-cottage-v1` (task `01a1e8a3-eb83-77bf-a193-76265cd910fb`) | `6d06814f3166c88bf63b27aadf84f8b9` | 79134 of 102714 triangles, maps at 1024 | `8d0a10cd94989e5c4a9f53c0b724a2a9b7c19f0d4ae3eb887ac276f39365598f` | `d33007a21c422e4acc4ab821af59259e3c604a54eb1f6c38a2d4258ac485ae72` |
| `assets/models/ruin.glb` | `hero-a-ruin-v1` (task `01a1e8a7-aa13-71c5-8526-eb087d8abdf4`) | `4f4384cf3811108b5e1721af06b14f6d` | 65342 of 82325 triangles, maps at 1024 | `9595e8ddfd99b64588f4a8a6a7f733493168dd3982cf43f03c290addd0fa3ffe` | `fc71fbd14cc777ed5bd4adea499666d6915dadc45469830c510e1fdeff937817` |
| `assets/models/rock-cliff.glb` | `hero-a-rock-cliff-v1` (task `01a1e8a7-94d6-7313-8352-60b5a7fd7233`) | `be7319b7dc9bc829a0e94677488bdde8` | 103323 of 123547 triangles, maps at 1024 | `717a9200f8539b4307b9b34223ec6023c840681cb1a8296963c70c02b3461a7a` | `e4d1f7971ef17be0f7381d8a5cab6ca6b1df37d6f09373d3d256ed0553f1a4da` |
| `assets/models/sea-stack.glb` | `hero-a-sea-stack-v1` (task `01a1e8a7-9f75-743e-ad81-2390114c23b3`) | `8f591a238b3a105a1944c8b309448388` | 36921 of 82185 triangles, maps at 1024 | `40d7a199399e54e5253ba6ac716c0eb47709048b6b47a2a3bf7d6a7383101e60` | `ee6b4e724e625671659acb6315a883c0ee3dc3db4b24593bb8635a7e426b0d4f` |
| `assets/models/boulder.glb` | `hero-a-boulder-v1` (task `01a1e8a7-b1ee-7746-a887-e75ebe577f71`) | `6743e88e526e2b47021b396822fadf0c` | 33190 of 40065 triangles, maps at 1024 | `e85e399df8f5c7c67bc0b845337a0bf61efd75705c1ca6465f9b8298f4276e67` | `4ea8c05167332ffd738fc1b7c229645cc27b847126ad6e0f44f080a3ee10e212` |
| `assets/models/wanderer.glb` | `hero-a-wanderer-v1b` (task `01a1e8a7-dfd2-70e3-a729-8746dfdbc122`), `hero-a-wanderer-rig-v1` (task `01a1e8c5-0bc3-7783-af0f-90b87fb60f01`), `hero-a-wanderer-idles-v1` (task `01a1e8c7-7cf4-769d-948e-ebca895e9744`) | `87816eadef36874dcee3324c44883bef`, `928b553c2339704026d35e0fb567dfdf` | 47301 of 92929 triangles, maps at 1024 | `5b9261b2d45e00366670d1a0978e4df69f76021425f82c4a94f25dc626a22c83` | `b182ebf6ceaa0522aa097749e855f6ab84071a920bd1e28c3ded6b6803b4eb12` |

The wanderer is three tasks: the mesh from a front and a back view
(multiview), its rig, and the rig's three idle clips (`Idle`, `Idle_02`,
`Long_Breathe_and_Look_Around`). After packing, `tools/cloak.ts` gives it
its cloak: a second mesh skinned to the spine, its cloth drawn by
ImageMagick, which og.Garment simulates. The shipped checksum is the
cloaked file's.

The sources as generated are kept as private objects in the asset store
by `tools/archive.ts` (docs/ci.md §Generated sources): `model.glb`, the
reference images and a `task.json` receipt (prompt, ids, sizes,
checksums), whose sha256 `tools/assets.json` records on each task as
`archive`. Those archived before 2026-11-19 keep their old key,
`meshy/model/<operation>/`, in the generated-sources index. The reference images
are not in git either (`tools/refs/*.png`); their prompts are.

### From the catalogue

Poly Haven, CC0 1.0 (https://creativecommons.org/publicdomain/zero/1.0/),
catalogue release `catalog-20261107-kenney-particles-v1`, fetched by
`tools/catalogue.ts`, which checks each file against the catalogue's hash
(`tools/catalogue.log`).

| Asset | Catalogue group |
|---|---|
| Aerial Grass Rock | `grp_b9a4454916bfef3264` |
| Rock Face | `grp_28a4c1a58ffc4b45d1` |
| Forest Ground | `grp_4cde219f95811c3035` |
| Lantern 01 | `grp_17d5a11495f4dfec62` |

| File | Asset | Made from | sha256 (shipped) | sha256 (source) |
|---|---|---|---|---|
| `assets/terrain/textures/aerial_grass_rock_diff_1k.jpg` | Aerial Grass Rock | `aerial_grass_rock_diff_2k.jpg` | ``802c867fa3ece0dc73b37c8fc053f492a2c43d405846d652edcc2663c03b3f32`` | `1a2ddfa4652adbe00447b940c40019d46f4b88245bc1b341cdf622c2110b7bc5` |
| `assets/terrain/textures/aerial_grass_rock_nor_gl_1k.jpg` | Aerial Grass Rock | `aerial_grass_rock_nor_gl_2k.jpg` | ``9b4061ab6b4351737efdab2910200b4c6d1723c7dcc0d66a3897931f6177b3fa`` | `c6aa14b042ce243f93a6ca807f46a085711c23ad316b104578e92df7770ee285` |
| `assets/terrain/textures/forest_ground_04_diff_1k.jpg` | Forest Ground | `forest_ground_04_diff_2k.jpg` | ``81bf21bf7d4075e08196580f1b4d3a72740b34a8992a2726b677297a6588cc3a`` | `24b8aaf4c8547305d80b0e029ec365219ab1b90748d4ca5651bdc9e75bfde5e6` |
| `assets/terrain/textures/forest_ground_04_nor_gl_1k.jpg` | Forest Ground | `forest_ground_04_nor_gl_2k.jpg` | ``571b3625ae4b8607b92bda08e708c3db5f9186cf5341fdb84b5ac084783f3d6e`` | `b37ac799ed410976541e0011c246fca254e74eaa3f096ae7a7d74178db5a6d9b` |
| `assets/terrain/textures/rock_face_03_grey_1k.jpg` | Rock Face, regraded | `rock_face_03_diff_2k.jpg` | ``f8d1789bb735784817cfef8f70b9278cd667a395664847cbb6f133e5ca6087a8`` | `644ee79df7ffbf5f6afc330b64dd3f33794bab8820915fe61ffe0c88ecfd7ece` |
| `assets/terrain/textures/rock_face_03_nor_gl_1k.jpg` | Rock Face | `rock_face_03_nor_gl_2k.jpg` | ``077fba3895cf8cc656b5a7c956f1a67a62116a69e897c1a92ca7dcab6967de24`` | `a498cef4ba6191f6959b6d03fb835263c77271948264a289af24497ab7111dec` |
| `assets/models/lantern.glb` | Lantern 01 | `Lantern_01_1k.gltf` with its buffer and three maps | ``0ac02152f06eb1640fb344448ca1f54290036eefbe113359d1b74c2fe38c8b97`` | `d3f8d7a7afec96d55d3b7f20c3e329c909d19d37047902a826bc157f77c27819` |

The terrain's textures are the catalogue's 2k maps taken down to 1024 with
ImageMagick: colour at JPEG quality 88, normals at 90 with full chroma.

```sh
magick aerial_grass_rock_diff_2k.jpg -resize 1024x1024 -strip -quality 88 aerial_grass_rock_diff_1k.jpg
magick aerial_grass_rock_nor_gl_2k.jpg -resize 1024x1024 -strip -sampling-factor 4:4:4 -quality 90 aerial_grass_rock_nor_gl_1k.jpg
```

The regraded rock is Rock Face's colour map with its rust taken down to a
lichen grey before the resize:

```sh
magick rock_face_03_diff_2k.jpg -modulate 104,30,100 -fill '#8a8f94' -colorize 12 -quality 92 rock_face_03_grey_2k.jpg
```

The lantern is packed by `tools/pack.ts` as the generated models are
(8266 of 33902 triangles, maps at 256: it hangs a hand high in the frame).

### Written here

`assets/plants/*.ogt` (the grasses, flowers and the hawthorn),
`assets/props/staff.ogt` and `assets/terrain/coast.ogt` are model and
terrain text; `tools/terrain.ts` generates `coast-height.png` and
`coast-control.png` (pinned with the rest) from `src/layout.ts`.
`assets/fonts/latin` came with `og create` (Noto Sans, OFL; its licence
files are beside it).
