# Night Shift

Hero showcase B: a courier with a glowing pack beside a hover-bike at the
mouth of a market alley, at night, in the rain. Neon signs and paper
lanterns lie in the wet street, a food stall steams, pedestrians walk under
umbrellas and the towers fade into the city's haze.

An og game in TypeScript through the public API. It runs on the native and
the web hosts.

```sh
og run .          # play
og test .         # the playtests
tools/capture.sh  # the website's frames and the recording (see Captures)
```

## Controls

| Key | Pad | Does |
|---|---|---|
| R | X | The next weather: rain, storm, fog, clear and wet, snow |
| F | Y | The next season: spring, summer, autumn, winter |
| T | B | Night or dawn |
| H | Back | Show or hide the HUD |

Each switch ramps over a few seconds. The same switches are commands, for
scripts and captures: `weather:<name>`, `season:<name>`, `time:<name>`,
`hud`.

## How it is built

| File | Holds |
|---|---|
| `src/set.ts` | The street, the stall, the tenements, the signs and lanterns, the gate, the towers |
| `src/cast.ts` | The courier (posed on the CPU), the hover-bike, the pedestrians (a GPU crowd) |
| `src/air.ts` | Weather, season and time of day: the targets, the ramps, and what they write to the sky, the fog, the weather and the lights |
| `src/flora.ts` | The cherry tree and what it sheds in spring and autumn |
| `src/hud.ts`, `assets/ui/hud.svelte` | The delivery HUD, from the engine's built-in Minimap, HudTimer, RpgQuestTracker, StatBar and ActionHints in the default style |

Seasons and the ramps between weathers are the game's own for now
(`src/air.ts`); they move to `og.Season` and `og.Weather.change` as the
engine gains them.

## Captures

`tools/capture.sh` writes every weather in every season, at night and at
dawn (40 frames), and one recording of the switching, to
`og-out/captures/`, with their checksums in `SHA256SUMS`. Each season is
one scripted run from tick 0, so the same GPU draws the same frames every
time. The frames of record come from the L4 host; anything drawn on a
software renderer is a preview. `SIZE`, `HOLD`, `TIMES` and `SEASONS`
narrow a run (the script's header lists them).

## Assets

The binary assets are not in git. `og-assets.json` pins each one (path,
sha256, size, visibility) in the platform's asset store (`docs/ci.md`
§The asset store), and one command installs them, each checked against its sha256:

```sh
bun ../../scripts/assets.ts ensure .
```

`tools/capture.sh` runs it first. After changing a pinned file, `pin` it
and `push` with the same script, and commit `og-assets.json`.

Generated assets were made for this project through the Origin gateway
(concept images: `gpt-image-2.6-sunburst`; meshes, rigs and clips: Meshy
image-to-3D, rigging and animation); they carry no third-party licence.
Every gateway call is in `tools/gateway-journal.md` with its ids. The raw
Meshy outputs and their concept images are kept in the generated-sources
archive (docs/ci.md §Generated sources) under `meshy/model/<operation>/`
(`model.glb`, `concept.png`, `task.json`), and the sign and window images
under `scenes/night-shift/textures/`.

`tools/shrink-models.sh` makes the shipped models from `raw/models/` (not
committed: each asset's last gateway output, the animate task's for a
rigged one, repacked once by `tools/pack-model.py` with 2048 JPEG maps,
the lantern with its glow): maps capped, meshes simplified to their size
in the frame, quantized and meshopt-compressed (gltfpack 1.3). The courier and the bike
keep 2048 maps and every triangle; the rest have 1024 maps. The Poly Haven
sets were fetched through the gateway's catalog at 2k and replaced by
Poly Haven's own 1k files of the same sets (`dl.polyhaven.org`).

### Pinned (`og-assets.json`)

| File under `assets/` | Source and gateway ids | Shipped as | Bytes | sha256 |
|---|---|---|---|---|
| `models/courier.glb` | concept image media `aed1983bbf2de2ca758b416fc0dad1bf` (generation `c77071fd-e8df-45b3-900b-0d02dec995e1`); mesh `ns-courier-image-v1`, task `01a1e8a1-954c-7722-9225-e0b556d729d4`, media `854cc7f0cd867f1e3634bdf24c9491c4`; rig `ns-courier-rig-v1`, task `01a1e8b1-97f8-76d1-8eab-683478128d8b`, media `330ba6bb2d9c7bc5bcf025090bf87cdb`; clips (Idle, Alert) `ns-courier-anim-v1`, task `01a1e8b5-42a2-732d-a2d8-7eb189e02cd4`, media `f141befd4743917bc7189a5381b3bd9a` | 2048 maps, every triangle (123k) | 5672248 | `e48b1ffe47d0b4c425f85e26b84ea09e30c0649a4132b52e49c013ea5f820e02` |
| `models/bike.glb` | concept image media `9fbaeb47c736af43ef460505ee540689`; mesh `ns-bike-image-v1`, task `01a1e8a4-2246-75a5-8ecc-70c1a472e3a7`, media `12d79351b68200cb1ae96c72b2b87cf0` | 2048 maps, every triangle (151k) | 6033216 | `8fa2953a1628816888194186d88ab8b68bd587050341576dc3c4d60c58730551` |
| `models/ped_a.glb` | concept image media `4a5cb3986d2a9d9a2d65ae447e38c441`; mesh `ns-ped-a-image-v1`, task `01a1e8a4-1cd3-7262-be4a-a49eb52b3dfc`, media `ed26b5f2f81deeb492624e2c874a7f30`; rig `ns-ped-a-rig-v1`, task `01a1e8b1-9805-7392-8245-4b477bd64287`, media `59a588996118624a864dd167cef43962`; clips (Walk with Umbrella, Idle) `ns-ped-a-anim-v1`, task `01a1e8b5-429a-727e-93b2-5424c180ddad`, media `96ccf805e0cbdec88f3e3e2e0b4f63d1` | 1024 maps, 15k triangles of 46k | 1158204 | `054e3cb6d07adc1441794ffeada392ee1f42fbcf7849cc4cabd68808342bbb8f` |
| `models/ped_b.glb` | concept image media `087e4847222b860fc2e1d106ae860a75`; mesh `ns-ped-b-image-v1`, task `01a1e8a4-1d01-745c-837c-d471477a5a33`, media `2874ea37245c3acd0261b9a80331082b`; rig `ns-ped-b-rig-v1`, task `01a1e8b1-980c-741c-a353-8f05790007a0`, media `8d07144d1f663eef657660964775d9b1`; clips (Walk with Umbrella, Idle) `ns-ped-b-anim-v1`, task `01a1e8b5-429e-716b-b5a8-221ad698df22`, media `338099433a44441dc33b5d0b7eddfe82` | 1024 maps, 18k of 47k | 1430600 | `7e1884d5b21bd1446023760c6e0988e9570e224603985fe60613f40330843c5d` |
| `models/ped_c.glb` | concept image media `4a3c17e84f5aae2b2cf8477b2c6610f4`; mesh `ns-ped-c-image-v1`, task `01a1e8a4-1d02-72a5-b745-a4c2cafba937`, media `8a47eabcece3e858ac12832babf9d7d0`; rig `ns-ped-c-rig-v1`, task `01a1e8b1-9806-7287-9085-dbadae98e864`, media `3d7cdd8b942ac31a6118bdf1f4b10eb1`; clips (Walk with Umbrella, Stand and Chat) `ns-ped-c-anim-v1`, task `01a1e8b5-4325-753f-89d1-638bcc312fdb`, media `0d13e30151f3a3d9621cf7d613665038` | 1024 maps, 16k of 46k | 1378664 | `950d11788165b788724d37b0df7b14016b3ba4d9cc173b35a70979e8202a57c9` |
| `models/stall.glb` | concept image generation `3d43b403-8be5-4b9f-bdc0-9a54d7d23abd`; mesh `ns-stall-image-v1`, task `01a1e8c1-044b-75fb-9425-fc5522f61ed6`, media `bbd8e2e0ef0d9196c23944aeba121370` | 1024 maps, 122k of 195k | 3590584 | `d397f9e3016ff6abeeff0fc940f0e2d4dc0097c3ca4813c37ddc1b19c42a037c` |
| `models/block-a.glb` | concept image generation `5d010d5b-f5f5-4112-a60f-a7f7d60c2e76`; mesh `ns-block-a-image-v1`, task `01a1e8c0-fb64-766f-aa3f-e8181822e0e2`, media `a53d76ff8b6b9ab0ac912a89aa7bb900` | 1024 maps, 101k of 142k | 3370952 | `6e98563897f38aa592dd54af56a5b921f3c4884c8e0a454f10cac3aa120b0f54` |
| `models/block-b.glb` | concept image generation `6fd9bbdd-cdba-43f2-a3ce-204a64714c57`; mesh `ns-block-b-image-v1`, task `01a1e8c0-fb64-70de-a213-8542215d4401`, media `f741d812d035167ab3982df7363be3f8` | 1024 maps, 102k of 147k | 3382480 | `81ea5a690f5c9927025d3bef590111376f62111d0f727ca20420d08634f9bf9c` |
| `models/gate.glb` | concept image generation `f5ca3c42-055a-48b4-97c2-23537a0733bb`; mesh `ns-gate-image-v1`, task `01a1e8c0-fe74-747a-8762-0b03b759f6ee`, media `c6ab1f36ce6e451997e15bb5c898f60b` | 1024 maps, 29k of 59k | 1864760 | `65875206a05b1c3daf46c78b92600b5628c0e742b18874a4f9a8fcd5f8e680a4` |
| `models/lantern.glb` | concept image generation `97ce8116-1867-488c-9a98-ac89ca9cad64`; mesh `ns-lantern-image-v1`, task `01a1e8fb-2b8b-7133-8380-b70427d8eac8`, media `9dca0e1a2229f220749aec466da72ee7` | 1024 maps, 12k of 20k; a glow of 2.5 | 1473176 | `2cfa9007cf8c4bb6f96814a237cc24d7b7c5ca3abdc03395c1a59ceb0802c579` |
| `textures/cobblestone_01_diff_1k.jpg` | Poly Haven, Cobblestone 01, 1k (gateway catalog group `grp_776009be378c7de71b`), CC0 | | 487258 | `aab330227974aadbc477b2022351112f5696fa7048d99dc776c2e6f23f4e0d2a` |
| `textures/cobblestone_01_nor_gl_1k.jpg` | Poly Haven, Cobblestone 01, 1k (gateway catalog group `grp_776009be378c7de71b`), CC0 | | 543090 | `4b88ed092d4391506b4760f454996f633c171332b8a68557593d6e80627fc8c0` |
| `textures/cobblestone_01_arm_1k.jpg` | Poly Haven, Cobblestone 01, 1k (gateway catalog group `grp_776009be378c7de71b`), CC0 | | 224969 | `71328ab5c403dac927af691ade4255944d2c22936bd4575895d835f96fc4f5a1` |
| `textures/brushed_concrete_diff_1k.jpg` | Poly Haven, Brushed Concrete, 1k (gateway catalog group `grp_1474aca64e69a44dc0`), CC0 | | 847278 | `4054ed6eb65cd063356ffb4a88706b7d7b78c5435051c48bf0ee64290f04bc8c` |
| `textures/brushed_concrete_nor_gl_1k.jpg` | Poly Haven, Brushed Concrete, 1k (gateway catalog group `grp_1474aca64e69a44dc0`), CC0 | | 849228 | `f4953a881f34627cefcc22f780d392314635e31bb41f75cf3888a5af55c21820` |
| `textures/brushed_concrete_arm_1k.jpg` | Poly Haven, Brushed Concrete, 1k (gateway catalog group `grp_1474aca64e69a44dc0`), CC0 | | 783702 | `374e4408acfb0bb4512f904f6fd3ae1006195ed12f2e258625b5b8b3a91f6528` |
| `textures/signs/chevrons.png` | Generated image, generation `6678b61b-1193-45a5-b1e4-c682c04be28f`, resized | | 471766 | `6cf48aac59a19c02a4c2474c83ec1de48d4c7eb3ad55a5cc6bcb9f776e6dee35` |
| `textures/signs/ring.png` | Generated image, generation `313ac34c-3248-4e90-bb50-d2f16d97d891`, resized | | 339598 | `c48feb099bfec723de0025a1cc9ea981f8b9542d52fdb696f5957f3662803291` |
| `textures/signs/triangle.png` | Generated image, generation `e9cd686b-0d54-4575-a733-a636c691b21b`, resized | | 211491 | `150a66c72bf2b7c3d1d26f39052407477acede55c3f933d764b743e6561db779` |
| `textures/signs/bowl.png` | Generated image, generation `3bb9d47b-967d-4303-8042-acdddcd094ba`, resized | | 262843 | `73418ba4463e2050b5edb064d100ead5ce043410a425429c17388604cd19988d` |
| `textures/tower-windows.jpg` | Generated image, generation `5b6d3372-fab9-4860-acbb-43f7af40de49`, resized | | 181334 | `8fa23a8276a9ea86dcec286429689ca5b519d85d9d5939ade6caba88e4d038cc` |

### In git

| File under `assets/` | Source | Licence |
|---|---|---|
| `models/cherry.ogt`, `models/cherry-bare.ogt` | Written for this project (a `plant` model) | The repository's |
| `textures/map.png` | Drawn by `tools/draw-map.sh` | The repository's |
| `ui/icons/ink-icons.png` | The engine's shared ink icons (`templates/styles/shared/`) | The repository's |
| `fonts/latin/` | Noto Sans, the font pack `og create` installed | SIL OFL 1.1 (`OFL-notosans.txt`); hyphenation patterns as `LICENSE-hypher.txt` |
| `fx/*.ogt`, `ui/hud.svelte`, `looks/night-shift.ogt` | Written for this project | The repository's |
