# Effekseer (file formats; sample effects)

Source: https://github.com/effekseer/Effekseer, revision
`b5700082b49bc5f919610880dafddc90211a9a4e`. Licence: MIT, Copyright (c)
2011 Effekseer Project (`LICENSE` here).

`crates/og-cook/src/cook/systems/efk/` reads Effekseer's files as that source
writes them; it ports format knowledge, not code, and embeds no Effekseer
runtime:

- `.efkefc`: the `EFKE` header and chunk layout and the `EDIT` chunk's
  compressed element tree (`Dev/Editor/EffekseerCore/IO/EfkEfc.cs`,
  `Utils/BinaryReader.cs`).
- `.efkproj` parameters, their defaults and enums
  (`Dev/Editor/EffekseerCore/Data/*Values.cs`, `Data/Define.cs`,
  `Data/IO.cs`, `Data/Value/*.cs`, `Utils/ProjectVersionUpdater.cs`), the
  editor's easing cubic (`Utils/Math.cs`), spawn shapes
  (`Dev/Cpp/Effekseer/Effekseer/Parameter/Effekseer.SpawnMethod.h`) and the
  ring's geometry and UVs
  (`Dev/Cpp/EffekseerRendererCommon/.../EffekseerRenderer.RingRendererBase.h`).
- `.efkmodel` vertices and faces (`Dev/Cpp/Effekseer/Effekseer/Model/Effekseer.Model.cpp`).

## Sample effects

The effects Effekseer distributes (https://effekseer.github.io/en/contribute.html)
are CC0: "These effects are distributed under CC-0 License. We'd appreciate
it if you could credit Effekseer and a contributor." The importer's tests
read a few of them, copied unchanged into
`crates/og-cook/tests/fixtures/effekseer/`:

| File | Contributor | Pack |
| --- | --- | --- |
| `Simple_Sprite_BillBoard.efkproj`, `Simple_Ring_Shape1.efkproj`, `Simple_Ribbon_Sword.efkproj`, `Texture/Particle01.png`, `Texture/SwordLine01.png` | Effekseer (Effekseer Project) | `Effekseer01.zip` |
| `iwai-ume.efkefc` | HATO | `HATO01.zip` (HATO-Springhascome/celebration) |

Credit: effects by Effekseer and its contributors, CC0.

## Effect packs

The engine's Effekseer effect packs are converted from the same CC0 sample
zips by `og vfx import`: the bundled `packages/engine/extensions/effekseer-starter/`
(a curated set) and the downloadable `packages/vfx-packs/effekseer-<contributor>/`
(every sample of one contributor). Their textures are the zips' files,
PNGs losslessly recompressed (decoded pixels unchanged); each file's zip
entry, licence (`CC0-1.0`) and credit ("Effekseer and <contributor>") are
in its pack's `assets.json`.

| Contributor | Zip | Source |
| --- | --- | --- |
| AndrewFM | `AndrewFM01.zip` | https://effekseer.github.io/en/contributes/AndrewFM01/index.html |
| Effekseer Project | `Effekseer01.zip` | https://effekseer.github.io/en/contributes/Effekseer01/index.html |
| HATO | `HATO01.zip` | https://effekseer.github.io/en/contributes/HATO01/index.html |
| MAGICAL x SPIRAL | `MAGICALxSPIRAL.zip` | https://effekseer.github.io/en/contributes/MAGICALxSPIRAL/index.html |
| NextSoft | `NextSoft01.zip` | https://effekseer.github.io/en/contributes/NextSoft01/index.html |
| Pierre | `Pierre01.zip` | https://effekseer.github.io/en/contributes/Pierre01/index.html |
| Pierre | `Pierre02.zip` | https://effekseer.github.io/en/contributes/Pierre02/index.html |
| Suzuki | `Suzuki01.zip` | https://effekseer.github.io/en/contributes/Suzuki01/index.html |
| tktk | `tktk01.zip` | https://effekseer.github.io/en/contributes/tktk01/index.html |
| tktk | `tktk02.zip` | https://effekseer.github.io/en/contributes/tktk02/index.html |
