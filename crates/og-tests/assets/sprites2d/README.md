# The 2D scenes' art

Painted backdrops and sprites for the 2D scenes (`fluid-2d`, `fluid-groups`
and the rest of the sprites family), sized for the 1280×720 frames:
backdrops 1280×720 JPEG at quality 84, sprites a few hundred pixels PNG.

Generated for origin-engine through the OriginGame gateway (2026-11-18) by
`generate_image` (gpt-image-2.6-sunburst), generated work owned by
OriginAI for origin-engine. The gateway draws sprites on a near-magenta key
with no alpha: opaque sprites are keyed by distance from the key colour
(a narrow soft edge, magenta spill pulled off the rim), the glass flask by
un-mixing the key into straight alpha, its outside cleared by a flood fill
from the border and its clear body set to a faint even glass tint. The
sources (the gateway's PNGs) are in the `origin-generated-assets` bucket
(`docs/ci.md` §The model bucket) under `gateway/image/showcase-2d/`.

| File | Gateway media id | Source sha256 | Made from it by |
|---|---|---|---|
| `cistern.jpg` | `60fa59a9f760d9e3e2f9ef5f91768db8` | `7e9e81cae9da40ceffc24f66ca905d5fe89a08505551e746603e443d5d513f92` | cover-crop to 1280×720 |
| `workshop.jpg` | `3aefba5b3aa5bbf7ad9b9876370471df` | `6257bea5621c4b56e415936e94dc84693ca66fef5c3f83f61b7aecb21e6ca868` | cover-crop to 1280×720 |
| `pipe.png` | `93cba67d3d2ada88579f914382507bd0` | `888e346b5060aac0d3065632aa7be03fff3627320ec57ea2a5903137cedada76` | hard key, cropped, 384 px tall |
| `flask.png` | `f637c7547595b277da1db1a23eeb470f` | `fa4941840ef745ca731e46d14afa36cc43c336e00a6eb090113f8e0b45f7dcaf` | glass key, cropped, 256 px |
| `stone-block.png` | `1fe8e74a9b5c2188d53817cc5200ee06` | `7789f47b716bd782698c17b5e432b72c0bfcaf00bf1ce55f030000c7891db0f6` | 128×128 |

Prompts: a pixel-art cistern chamber (mossy arches, a rusted pipe, a light
shaft from a grate, torches) with an empty lower third; an alchemist's
workshop at night (shelves of jars, a furnace, a moonlit window) over an
empty workbench; a rusted riveted pipe ending in a flanged outlet; an empty
corked round-bottomed flask; one mossy teal cistern stone block.
