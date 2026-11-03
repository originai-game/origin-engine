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

## Sky props (meadow): replacing the procedural stand-ins

Concepts: `generate_image` (gpt-image-2.5-sunburst), "Single isolated 3D game
asset reference, three-quarter view ... Stylised fantasy in the spirit of
Zelda Tears of the Kingdom and Genshin Impact ... warm sandstone and cool
lilac-grey rock, sage-green grass. Original design." plus the subject. Then
Meshy image-to-3D (meshy-6, triangle topology, remesh, textured, PBR):
- island (hero floating island): media bf2da280e2f1cb1c69b4c08a97947409, operation skyreach-prop-island-img-1, task 01a1a0eb-8b0d-76c3-9ac9-8bd71b09827a, 40k
- arch (ruined archway): media db86bd8925a91ed32d8158c0de78342a, operation skyreach-prop-arch-img-1, task 01a1a0eb-9355-7486-a0db-50111071f0f7, 20k
- tower (far landmark): media 514d852c55fa15378099f19888ae2608, operation skyreach-prop-tower-img-1, task 01a1a0eb-9ffa-712e-b0dd-08f021c08b62, 12k
- islandSmall: media 4935e84ec6bac3656937d30805efc3a8, operation skyreach-prop-islandSmall-img-1, task 01a1a0eb-a70e-769f-b056-3d2b0969b59a, 15k

Results (Meshy GLBs, then gltf-transform optimize: meshopt geometry, WebP textures at 1024):
- island.glb: media cdb7543a6f50c0bf360097a69cca13fa, 16.4 MB to 1.25 MB
- arch.glb: media d8f3e76dc25c0a58c410125c123964e9, 14.7 MB to 0.94 MB
- tower.glb: media 47e4acc44f93b6d81903e317f7ee8cf3, 12.3 MB to 0.61 MB
- islandSmall.glb: media 828ef658f420e7eed8d149417fde1686, 12.8 MB to 0.66 MB

## Wildflowers (`flowers/`): replacing the code-built flowers

Concepts: `generate_image` (gpt-image-2.5-sunburst), "Single isolated 3D game
asset reference, three-quarter view, plain light grey background. Stylised
fantasy in the spirit of Zelda Tears of the Kingdom and Genshin Impact,
hand-painted look, clean readable shapes. A small clump of wild meadow
flowers about 30 cm tall growing from a tuft of sage-green leaves: ..."
plus the species. Then Meshy image-to-3D (meshy-6, triangle topology, 4k
target, remesh, textured, PBR):
- daisy (white, yellow centre): media 34fefd568cc61f790e30c788ff3e4397, operation skyreach-flower-daisy-img-1, task 01a1a236-6438-7288-99e2-5d34e97c3a66, GLB media bd5994f33dfb539b3acd054b37d25e89
- bell (nodding lilac bellflowers): media d3654988b65871f3f991f720bd1642a2, operation skyreach-flower-bell-img-1, task 01a1a236-6e43-72f2-9518-5dbb6346ceec, GLB media 6ded276d440a17b80ee9e3187ebcb777
- buttercup (golden cups): media 46c1a8644d3efd2d0c23f093cced784f, operation skyreach-flower-buttercup-img-1, task 01a1a236-75a2-72b0-873f-9c9bbb905e42, GLB media 245717e59e889385b98ccafa9892af0b

Each GLB (~3.9k triangles) was baked rooted at its base, centred in x and z,
to its height (daisy 0.34 m, bell 0.36 m, buttercup 0.30 m), for
og.TerrainScatter's sway, then gltf-transform optimize (meshopt, WebP at
512): about 250 KB each.
