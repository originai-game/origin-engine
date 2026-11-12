# Vegetation sources

Licences checked against these exact revisions on 2026-11-03 (ez-tree on
2026-11-12). Original
copyright and permission notices are alongside this file. Headers in adapted
code identify the source actually used; this list does not claim that every
reference's implementation has been ported.

| Source | Revision | Licence | Use |
| --- | --- | --- | --- |
| [SeedThree](https://github.com/SkyeShark/SeedThree) | `cf2eaad39d9b7355ed086aca2a95fb785e539242` | MIT, SkyeShark 2026 | Anchor-coherent wind, canopy dome normals and leaf translucency reference |
| [threejs-nanites](https://github.com/VictorZakharov/threejs-nanites) | `500113d7b651d2eb96392ce87c96330bdec25e15` | MIT, Victor Zakharov 2026 | Deformation-aware culling, motion vectors, canopy shadows and impostors reference |
| [RedGPU](https://github.com/redcamel/RedGPU) | `95aaf9a9ecb3c5ccfcfc824ccad1a6489392cf68` (`4.4.0-Alpha`) | MIT, Redcamel 2019– | Foliage LOD and octahedral impostors reference |
| [ez-tree](https://github.com/dgreenheck/ez-tree) | `dcf309bd86bd521083d9c70f01f2de45fdc7c457` | MIT, Daniel Greenheck 2024 | The parameter model and growth rules of og-geom's `plant` (levels of branches, gnarliness, twist, force, stratified children and leaves), rewritten in Rust; no texture or preset file is copied |

[GPU Gems 3 chapter 16](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-16-vegetation-procedural-animation-and-shading-crysis)
and Unreal Pivot Painter are technique references only. No Epic or NVIDIA
implementation is copied under these MIT notices.

The newer-source search also found SleepinWei/Scene-Renderer's October 2026
hierarchical tree-wind work and Helio's August 2026 shared foliage wind design.
Neither is vendored. Scene-Renderer documents shared template deformation,
which sacrifices per-instance phase. The implementation here retains
per-instance wind, explicit attachment metadata, and current/previous drawn
states; the shared gust-field integration follows the engine's grass path.
