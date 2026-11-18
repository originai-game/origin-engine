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
| `hero-knight.png` | `290173466e9314b710430e9534aecb36` | `9b0ad4ab53b7451925365c5054f198e5d0341073dcb4e2d251e3ae64753a8017` | hard key, cropped, 256 px tall |
| `hero-mage.png` | `2aa0f10784e88477b0b49f6c19ff441c` | `29bc40710c1af6cadf09c9e997f3a1e58ff91bc610becc253855d1ecd46675f1` | hard key, cropped, 256 px tall |
| `hero-ranger.png` | `5162cb4778ad132ac8934a3827afc7df` | `2634d69313c0924de7d430c5dc1c03a5c89eb5612bd3c9360ea68f8cefe63d18` | hard key, cropped, 256 px tall |
| `tree-oak.png` | `e464a2b21dbef178276e319e6dc3f59a` | `daa38937c45818767c0f2140625be2dcd773b252921333669758564bef487bb3` | hard key, cropped, 256 px tall |
| `sign-shield.png` | `c465908610d57a08185e3baf840df7b8` | `cffed06b09bc40623f385a267b28530f16782d41cc3c9e3a9b4af80c26e35cc5` | hard key, cropped, 128 px |
| `vault.jpg` | `42da3ce7e8ef3df71e1dae523d63b35f` | `87cfeac3b371911fe71897a33c8e86d8076e5a63f38e5f25af54cc977eea4907` | cover-crop to 1280×720 |
| `chest.png` | `5f6741bf4e0f500ad70d798162412860` | `5b0bea46c39bb105c7a0ccb8182fe68d15fa41e213758a556e15290407a50de8` | its own alpha, cropped, 256 px tall |
| `cauldron.png` | `cc20609e10c71db9f964573d9e548ff2` | `3ae5c638ce04e71f4e54baa962c294297686333d5e84a622743e9380d026b709` | its own alpha, cropped, 256 px tall |
| `vault-block.png` | `e3c5cead3ac83c0bbdaf259f37afd265` | `581898f822c90b6c3674f6ff21fa8912f96e99613557f9a933de1cf6b18a1aa9` | 128×128 |
| `cave-tiles.png` | `ea371d6dbf0365e14f0fc510fd851170`, `b65ddf48da64fa5d0040200b8bb50b7d`, `b1ea1f5e28c53058fef4d773d0c947ba`, `0ba8038e130d48e95329ee5bc3716a8a` | `fd2704be257eb17ce7f3127755287c78929706619f3c584be6c31d09f4c8788b`, `a6015e5c8bbf51341a338fe68b8bd4858cc8e1dd1bb335f208cb4a4352eb9456`, `13c924bc5f473ddde80de5285d4a14b2ef8a6ea11d0dbd97c625727afe02f04c`, `ffb6274e3b1982a0bfea19ec2c05ee75446faa47a80ca595d58de8f945926d7c` | four tiles (rock, mossy rock, brick, sky) each 40×40 side by side; the rock's colour matrix (1.03, 1.16, 1.27) to the mossy rock's grey; the sky's stars kept by taking each block's brightest pixel where it is above 0.85 luma over the mean of its dim pixels |
| `torch.png` | `76583f42572152fdf95ccb394f3c655c` | `aa0fdd42ef18077570be73b6618667e1a07ea419cda21dfee13db340811b65d9` | hard key, cropped, 128 px tall |
| `crystal.png` | `13c7f12f197d451e9f090d2ac8f1f457` | `c587e71901d932f5a37a73a1511fa82c15ba837839de4ac29e10528fdd6d455f` | hard key, cropped, 128 px tall |

Prompts: a pixel-art cistern chamber (mossy arches, a rusted pipe, a light
shaft from a grate, torches) with an empty lower third; an alchemist's
workshop at night (shelves of jars, a furnace, a moonlit window) over an
empty workbench; a rusted riveted pipe ending in a flanged outlet; an empty
corked round-bottomed flask; one mossy teal cistern stone block; three HD-2D pixel-art RPG
heroes (a knight in a red tabard, a mage in a blue hooded robe, a ranger in a
green cloak), full body, three-quarter front view, on a magenta key; a round leafy oak; a
round heraldic shield with a red rim and a dark diagonal bar; a dwarven
treasure vault (torch-lit pillars, iron doors, hoards in the corners) over
an empty floor; an open treasure chest full of gold; an iron cauldron of
teal brew on embers; one carved slate dwarven block with a gold knotwork
band. The chest and the cauldron came from the gateway with straight alpha
(`transparent`), so they needed no key; the torch and the crystal, asked for
the same way, came back on the key. The cave tiles: seamless pixel-art
cave rock, the same rock under a band of moss, an old red-brick wall and
a starry night sky; a wall torch; a cluster of pale-blue crystals on a
rock.
