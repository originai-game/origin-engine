# Skyreach Meadow: generated assets

All made through the Origin platform's generation gateway (origin MCP,
OG_PLATFORM_DEVELOPMENT_KEY) on 2026-11-03. Original designs; no real
brands, logos or likenesses.

## The traveller (hero) and the second traveller (site-motion)

- Concepts: `generate_image` (gpt-image-2.5-sunburst), full-body A-pose
  reference sheets. Prompt: "Full-body character model reference for a 3D
  game, single figure, front view, symmetrical relaxed A-pose ... Stylised
  fantasy RPG in the spirit of Zelda: Breath of the Wild and Genshin Impact
  ... A young male (female) traveller: tousled chestnut-brown hair (dark
  auburn hair in a short braid), a vermilion-red hooded cloak ... a
  sage-green tunic with a brown leather belt ... cream linen trousers ...
  tall brown leather boots, a short sword in a scabbard on the left hip."
  - male: media 34a8a75cda15af731f840088a33ff45e
  - female: media 2baf9a03580a7dfcfa9a899e2d8af645
- Meshy image-to-3D (`generate_3d` operation image: meshy-6, a-pose,
  triangle topology, 60k target, remesh, textured, PBR):
  - male: operation skyreach-traveller-m-img-1, task 01a1a0c5-d7fb-727d-80c0-ae5a3db031df
  - female: operation skyreach-traveller-f-img-1, task 01a1a0c5-e3d8-76cc-a66a-61ddcfb085ac
- Meshy rig (`generate_3d` operation rig):
  - male: operation skyreach-traveller-m-rig-1, task 01a1a0ca-a89b-718b-95eb-478d5f4fc081, height 1.74 m
  - female: operation skyreach-traveller-f-rig-1, task 01a1a0ca-b7c8-7515-9c95-f2708636e3d5, height 1.66 m
  - Result: ~62k triangles, 24 joints, PBR (base colour, normal, metallic-roughness). Humanoid map: `rigs/meshy.ogt`.

## Terrain textures (`../textures/`)

`generate_image` (gpt-image-2.5-sunburst), "Seamless tileable square
texture for a stylised game terrain, viewed straight down ... hand-painted
painterly style ...", then desaturated toward sage and resized to 1024²
with ImageMagick:
- grass.png: media b7d7b4e085160d59a11d64fb6f7e351c (modulate 100,72,93)
- flowers.png: media 44eeeabd3ed5b658219c6b2ddad04029 (modulate 100,75,93)
- rock.png: media 4f9bec150075126ecc7c2daab1b3353b
- dirt.png: media df6b7b245d91d7aca389d91903d8a8e4 (modulate 100,80,100)
