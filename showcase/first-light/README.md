# First Light

Hero showcase C of Origin Engine: a young shepherd and his dog under an old
oak on a hilltop look over a lake at first light. Below them lie a timber
village with its jetty and boat, a paddock of sheep behind dry stone walls,
a windmill on its hill, pine woods and a snow range. The player switches
the weather, the season and the time of day; nothing else is at stake.

It is a game project in TypeScript on the public API (`src/`), for the
native and web hosts.

## State

Milestone 1, a blockout: the composition under clear summer weather at
dawn. Weather and time of day switch with eased transitions (`src/env.ts`);
the season is chosen and shown but changes nothing until the engine's
`og.Season` lands. The HUD is mounted and draws no text or icons yet. Open
work is listed at the end.

## Running it

The binary assets are not in git. Install them first, from the repository
root (docs/ci.md §The asset store):

```sh
bun scripts/assets.ts ensure showcase/first-light
og run showcase/first-light
```

| Control | Does |
|---|---|
| 1 (pad X) | the next weather: clear, rain, storm, fog, snow |
| 2 (pad Y) | the next season: spring, summer, autumn, winter |
| 3 (pad B) | the next time of day: dawn, noon, dusk, night |
| H (pad Back) | show or hide the HUD |

Commands for scripts (`og run --commands`):

- `set_scene:<weather|->:<season|->:<time|->[:now]` chooses a state; `now`
  lands on it at once instead of easing to it.
- `set_hud:<on|off>` is H.
- `draft` turns the woods' shadows off and `draft:bare` leaves the woods
  out: for a quick look on a machine without a GPU, never for a frame that
  is kept.

A frame of the opening state:

```sh
og run showcase/first-light --headless --capture-only --size 1920x1080 \
  --commands "0:set_hud:off;0.1:screenshot:first-light.png;0.15:quit"
```

## How it is made

- `src/layout.ts` holds where everything stands, in scene metres with the
  knoll at the origin and the view down -z. The terrain tool and the game
  both read it.
- `tools/terrain.ts` writes the valley's 2049 x 2049 height map and its two
  control maps from the layout (`bun tools/terrain.ts`, then pin and upload
  them).
- `tools/models.ts` packs the raw generated models: true size, standing on
  the origin, meshes simplified and meshopt-compressed, maps WebP
  (`bun tools/models.ts <directory of raw .glb files>`).
- `src/land.ts` is the terrain, the lake, the grass and the scattered
  woods; `src/set.ts` the oak, the figures, the paddock, the village and
  the mill; `src/env.ts` the sky, light, fog, wind and the switching;
  `src/hud.ts` and `assets/ui/hud.svelte` the HUD.

## Assets

Every model was generated for this project through the OriginGame gateway
(origin MCP, `OG_PLATFORM_DEVELOPMENT_KEY`): a concept image from
`generate_image` or `edit_image` (gpt-image-2.6-sunburst), then Meshy
image-to-3D through `generate_3d` (`ai_model: latest`, PBR; from one image,
or from a front and a back view for the shepherd and the dog). Licence:
generated through the OriginGame gateway for origin-engine; no third-party
asset is included. The image prompts were not kept; the concept images are.

The raw GLBs, their concept images and a `task.json` receipt each are in
the private bucket `origin-generated-assets` under
`meshy/model/<operation>/`, so a model can be packed again or regenerated.
The shipped files are pinned in `og-assets.json` and stored under
`showcase/first-light/<sha256>.<ext>`.

| Shipped file | What | Meshy input | Operation (archive directory) | Gateway task | Size | sha256 (shipped) | sha256 (raw `model.glb`) |
|---|---|---|---|---|---|---|---|
| `assets/models/shepherd.glb` | the shepherd boy, standing with his back to the camera | multiview | `first-light-shepherd-mv-1` | `01a1e8a0-6822-72ac-b9e4-be097f5ff236` | 2.22 MB | `ea8bfd4089d2e6bcbc34e89f759e19766b48cd313cb760412e2979d587567f82` | `a78099f74ca3f5257ce934d53a659687a6eddac13dc2166eb774fc9542080013` |
| `assets/models/dog.glb` | the border collie sitting beside him | multiview | `first-light-dog-mv-1` | `01a1e8a2-5712-7344-8417-da54b74482e9` | 2.61 MB | `eed64e1beb73684c133d853d535fda5a7bf9133246240a012ae774883a30cad9` | `2a778c52a5ce13877b95bdfaee0018efb72824251689cebd409668539320dff5` |
| `assets/models/sheep-stand.glb` | a sheep standing | image | `first-light-sheep-stand-1` | `01a1e8a0-bd5d-7797-88f2-2224a773b71a` | 0.61 MB | `956f4677830ffd3a07657420a639d445325d94f840b066ba9e6b30a9de667636` | `511f1240987844c50f4dc65edbdf928d273cd125a21a157aef16da5f845bd2cd` |
| `assets/models/sheep-graze.glb` | a sheep grazing | image | `first-light-sheep-graze-1` | `01a1e8a0-bcfc-7431-a0bd-7abacaedcacc` | 0.61 MB | `2285e0ceee450923de25e366cd86228b9218a907090deae97418d17485100d31` | `916c368c38b1141c257761248035501255e517b9f28e7af083bb84ace8d90136` |
| `assets/models/house-a.glb` | a timber-framed house with a stone chimney | image | `first-light-house-a-1` | `01a1e8a0-bd87-7104-a831-b6a8713d7e21` | 1.49 MB | `6585941270cb58b9ef240a421ab7992cb7bfc71198af918d153a4da209eeb7e9` | `ff44aa8610e3fe48e8e29906dcdb87e6b6e6f6e1acb1357e74b04c1764bf75a9` |
| `assets/models/house-b.glb` | a small timber-framed cottage | image | `first-light-house-b-1` | `01a1e8a0-c4c1-7699-92e7-65cf9255e87d` | 1.00 MB | `ad876711af4dabf5be67f713c8c0b9c28720d3fce6b2b1873c086852de984957` | `8c18aa827de8c9be91a6a43aae9a0a3a9e51caa93d707eef35d1bd7d5e420800` |
| `assets/models/house-c.glb` | a long timber-framed house | image | `first-light-house-c-1` | `01a1e8a0-bd77-70cc-863d-5a495d802e7d` | 1.36 MB | `ede8eafe275c092b16ee22521a3e3746fbd256622076e6a9d8ded1b44e264033` | `5e56eb045b7537ec5c1b280dccb72035aa88deae647939fcc2cd9dcf02a18c96` |
| `assets/models/windmill-tower.glb` | the windmill's tower and cap | image | `first-light-windmill-tower-1` | `01a1e8a4-c0a1-74ae-ad14-d5cbaa819565` | 1.46 MB | `3cc9e2a123165728f495b7681f162f5a735500db1af35f449df044313d6dbdb0` | `5090aa65c80e17d99eb30f3f711bd856f0500dd1d2aa1f741d04427ab5113175` |
| `assets/models/windmill-sails.glb` | the windmill's sails, centred on their hub | image | `first-light-windmill-sails-1` | `01a1e8a4-c2a5-71e3-9ccd-1c3c3d619c72` | 0.52 MB | `dba86705a2d60d05eb5360ff23ab5e3fef772c786281893f3ae6146a62ca9446` | `014725792c132b7e9f644355d2fc9e597ea94095fba1f289cacedd7c79b8144e` |
| `assets/models/jetty.glb` | the jetty with its lantern post | image | `first-light-jetty-1` | `01a1e8a0-bd8c-749b-a0ed-eb066cf49e39` | 0.81 MB | `65538590757d35fa3bd8f52604b751a9e9eca53efbc7e07bd255e17e36894b02` | `41ba6311930649bf94c9895fb36d76418a74b8612cb75eb8c0ae860b9b4e88fb` |
| `assets/models/boat.glb` | the rowing boat | image | `first-light-boat-1` | `01a1e8a0-bd47-70a4-83bc-bc49414e2b4c` | 0.41 MB | `b6a22155363407f675e1faadc874bfabf7c53fb7e51a9606573c74c363a13b56` | `723f43fb85a77d412fb8bef2ec3c3dd86238c63954ccdfdea5fb2f53d1e11b9c` |
| `assets/models/gate.glb` | the paddock's gate | image | `first-light-gate-1` | `01a1e8a0-bd03-7193-ab71-1a8fb66ad3b7` | 0.26 MB | `4ae9634f0dfb6088a4e42eb2f0a6e55523f2249ba41af0f38ade6d90df58b1ec` | `eb605a135b52981d3d95753dc13a5451a516bcde992e69d668e6bf0ac10d6f13` |
| `assets/models/wall.glb` | a 4 m run of dry stone wall | image | `first-light-wall-1` | `01a1e8a0-bd92-7430-a4e9-cedc6685d138` | 0.40 MB | `23fa2253938f537ae2a1719d77332122a3940ec70f152b086611ab741db9f816` | `867fa101e7b0663e8381b27ca31cd499a37130183367ea7e672f31184c401d4a` |
| `assets/models/boulder.glb` | a mossy boulder | image | `first-light-boulder-1` | `01a1e8a4-c0fe-7430-95b7-11eb2343004f` | 0.91 MB | `a52e0be518b15fa840034298f8258084a5d353801eefc680da5f11b9879813e0` | `9c2b3d78bbc7e4a50a6006070e923b9d68dc6bc45ae13b04fe8fec4279804832` |
| `assets/models/mushrooms.glb` | a cluster of mushrooms at the oak's foot | image | `first-light-mushrooms-1` | `01a1e8a4-c0b5-7204-9499-9f17134b4e76` | 0.49 MB | `f8ed076f330ea157370fb9aa076ae3af8a6b34581c4503dc381fd04874fab048` | `223e34e3728b6231f0ef1520f0a250cfae4beee06d254104bc2cda87e6c04ab3` |

The terrain's maps are written by `tools/terrain.ts` (this project's own
work, no outside source):

| Shipped file | Size | sha256 |
|---|---|---|
| `assets/terrain/valley-height.png` | 8.00 MB | `8a0560f76a1eeb01ba2a323624d4c504199254125d128c31e32007f18c35a4f6` |
| `assets/terrain/valley-control-a.png` | 2.66 MB | `a0658d920c32b840e618be69574fe4b424e13354b47c89f610b0c412fc897aef` |
| `assets/terrain/valley-control-b.png` | 2.96 MB | `b1bf4034282668dd0febe4ca03d601ac40e2dc8763d7df589351d693be6b368f` |

In git:

- `assets/plants/*.ogt`: the old oak, the woods' oaks, birches and pines,
  the shrub, the heather and the daisy are engine plants written for this
  project; `assets/models/crook.ogt` is the shepherd's crook, a model of
  parts.
- `assets/fonts/latin/`: Noto Sans (the Noto Project Authors, SIL Open Font
  License 1.1, `OFL-notosans.txt`) and its hyphenation patterns
  (`LICENSE-hypher.txt`), as `og create` installs them.

## Open work

In order: rock and snow contrast on the range; terrain layer textures; the
oak's bark and leaf cards with an oak-leaf texture; the woods' species,
tone, density and clearings; broadleaf trees, footings and paths in the
village; the sheep and the path reading from the knoll; chimney smoke; god
rays through the oak; mist that thins and lifts with the light; the HUD's
font and icons; the lake's reflection (the engine's water); the seasons;
playtests of every state; the captures of every weather and season and the
recording of the switching.
