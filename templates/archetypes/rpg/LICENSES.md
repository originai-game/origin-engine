# Asset licences

Every asset here is CC0 1.0 (the KayKit and Kenney packs, through the
OriginGame asset catalogue), one of the engine's own, or self-made by the
Origin Engine authors.

| File | Licence | Origin |
|---|---|---|
| `assets/models/keep/*.gltf`, `*.bin`, `dungeon_texture.png` | CC0 1.0 | KayKit Dungeon Remastered 1.1 by Kay Lousberg (kaylousberg.com), through the OriginGame asset catalogue: walls, the pillar, floor tiles, the chest, barrels, crates, banners, the torch, a coin, a bottle and rubble, used as published |
| `assets/models/gear/sword_1handed.*`, `axe_1handed.*`, `knight_texture.png`, `barbarian_texture.png` | CC0 1.0 | KayKit Adventurers 2.0 by Kay Lousberg, through the OriginGame asset catalogue: the first-person view's blades |
| `assets/models/knight.glb`, `barbarian.glb`, `mage.glb` | CC0 1.0 | KayKit Adventurers 2.0 (the Knight with its sword and round shield, the Barbarian with its axe, the Mage with its staff), merged with their weapons by `tools/compose.py` |
| `assets/models/skeleton.glb`, `captain.glb` | CC0 1.0 | KayKit Skeletons 1.1 (Skeleton_Minion with its blade and small shield, Skeleton_Warrior with its axe), merged with their weapons by `tools/compose.py` |
| `assets/models/*.glb.ogt`, `assets/rigs/kaykit.ogt` | CC0 1.0 | self-made: the import settings retargeting the clip library onto KayKit's Rig_Medium, written by hand |
| `assets/anims/humanoid.glb` | CC0 1.0 | the engine's: the humanoid clip library of `examples/kestrel-stair` (idle, walk, run, attack, hurt, die, slam, with `hit` markers) |
| `assets/audio/swing.ogg`, `hit.ogg`, `clang.ogg`, `crush.ogg`, `bones.ogg`, `chest.ogg`, `step.ogg` | CC0 1.0 | Kenney (kenney.nl) sound packs, through the OriginGame asset catalogue (groups grp_922700f05eb47ba332, grp_e750966760b8b5fe9e, grp_bd2b673c7d2f704264, grp_eba07f82944cfb5f82, grp_5947fcaa08c1deb4a0, grp_4034b25c2c9c838fba, grp_c9bdf254d90371004e) |
| `assets/audio/keep.wav`, `won.wav`, `fallen.wav`, `quest.wav` | CC0 1.0 | self-made: the yard's music loop and the stingers, synthesised by `tools/make-assets.py` (standard library only, deterministic) |
| `assets/audio/*.ogt` | CC0 1.0 | `og sfx` presets (click, select, back, error, levelup, powerup, coin, hurt, laser) that `og cook` synthesises, written by `tools/make-assets.py` |
| `assets/textures/flame.png`, `smoke.png`, `spark.png` | CC0 1.0 | the engine's: `templates/fx-showcase` (`tools/make-assets.py`), by way of the third-person archetype |
| `assets/fx/fire.ogt`, `smoke.ogt` | CC0 1.0 | the engine's: `templates/fx-showcase`, written by hand |
| `assets/fx/sparks.ogt`, `heal.ogt`, `levelup.ogt` | CC0 1.0 | self-made: the blow's sparks, the draught's motes and the level-up's ring, written by hand |
| `assets/ai/*.ogt` | CC0 1.0 | self-made: the risen's behaviours, written by hand |
| `assets/nav/keep.ogt` | CC0 1.0 | self-made: the navmesh's bake file, written by `tools/make-nav.ts` from `src/layout.ts` |
| `assets/story/*.ogt` | CC0 1.0 | self-made: Maren's dialogue and the quest, written by hand |
| `assets/ui/*.ogt` | MIT | the engine's screen documents `main-menu` and `pause` (`og ui eject`), edited for this game |
| `assets/strings/*.ogt` | CC0 1.0 | self-made: the English and Chinese string tables |
| `assets/styles/greywatch/*` | CC0 1.0 art, SIL OFL 1.1 fonts | the engine's fantasy style pack by way of the third-person archetype's `ember`, `assets/styles/greywatch/LICENSES.md` |
| `references/*/*.png` | CC0 1.0 | frames of this game recorded by `tests/frames.og.ts` |
