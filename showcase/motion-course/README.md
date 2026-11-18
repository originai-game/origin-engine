# Motion Course

og.motion's showcase: two characters run a course through a temple's
ruined forecourt on a clear morning. The runner walks, runs, brakes into a
running jump over a fallen drum, walks on, steps up onto the terrace under
the arch, turns to the camera and waves. The porter walks, hops, clears a
second drum, turns and cheers. The simulation owns every path and arc
(og.MotionAgent with `drive`); og.motion's model draws the motion between
the goals, on its default GPU backend.

An og game in TypeScript through the public API.

```sh
og run .          # play
og test .         # the footing playtest (tests/feet.og.ts)
tools/capture.sh  # the website's frame and the recording (see Captures)
```

## How it is built

| File | Holds |
|---|---|
| `src/main.ts` | The two characters, their courses of goals, the sun and the camera that follows the runner |
| `src/set.ts` | The forecourt: the meadow, the cobbled way, the terrace with its collider, the fallen drums, the columns, the arch, boulders and pines |
| `src/util.ts` | Small helpers: vectors, turns and loading |
| `tests/feet.og.ts` | Both characters' drawn feet over the whole course, held to the thresholds of `og api show animation` §2.1 |

## Captures

`tools/capture.sh` writes the settled frame (12.5 s: the runner waving on
the terrace, the porter cheering) and the recording of the whole course
(0 to 13.5 s at 60 fps), with the footing over the course, to
`og-out/captures/`, and their checksums to `SHA256SUMS`. Each is one
scripted run from tick 0. Each frame `og render` draws waits for og.motion's results
and for the textures being prepared, so the same GPU draws the same
frames every time. The frames of record come from the L4 host; anything
drawn on a software renderer is a preview. `SIZE`, `FPS`, `FRAME` and `TO`
change a run (the script's header lists them).

## Assets

The binary assets are not in git. `assets.json` pins each one (file,
sha256, bytes, key, url) in the model bucket (`docs/ci.md` §The model
bucket), and one command installs them, each checked against its sha256:

```sh
bun ../../scripts/showcase-assets.ts ensure .
```

| Files | Origin | Licence |
|---|---|---|
| `models/runner.glb`, `models/porter.glb` | Generated for origin-engine through the OriginGame gateway on 2026-11-18. A concept by `generate_image` (gpt-image-2.6-sunburst, A-pose), Meshy image-to-3D by `generate_3d` (meshy-6, PBR), then Meshy's auto-rig (1.70 m and 1.60 m) and its animate step for the Idle, Casual Walk and Run 2 clips, which are the fallback until the model is resident. `tools/shrink-models.sh` makes the committed files from Meshy's output (maps at 1024 px JPEG, meshes simplified and meshopt-compressed). Gateway task ids: runner `task_4l4fCAZdKuz5DwAyaWGnKdg2Gbe590Kj`, `task_QOTxiBKua6yED3wcRu1d2xzpp3sUHoGJ`, `task_9JzGwnrAk7VqXbPAWqpC01LzLuPWrONs`; porter `task_AYdzyLu171Xhg8RCwPTJ2OJnOKTOysTS`, `task_rghBjHcKDD0fAiwysDUgiFhVtD3kH7EP`, `task_mjIfVYKtCnbmXOKbpMzTvKQpq97xcSba`. | Generated work owned by OriginAI for origin-engine |
| `models/{arch,boulder,column,column_broken,pine}.glb` | The og-tests kit's files, byte for byte (`crates/og-tests/assets/kit/README.md` lists each one's gateway concept, Meshy task and checksum). | Generated work owned by OriginAI for origin-engine |
| `textures/grass.*`, `textures/cobbles.*`, `textures/stone.*` | The og-tests kit's surfaces, byte for byte: Poly Haven's Aerial Grass Rock, Cobblestone 01 and Broken Wall from the platform's asset catalog (the kit README describes Broken Wall's corrected normal map). | CC0 1.0 |

The motion itself comes from the engine's og.motion model
(`packages/engine/motion.json`), licensed by NVIDIA Corporation under the
NVIDIA Open Model Agreement.
