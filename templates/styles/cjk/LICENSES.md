# The styles' Chinese companions: licences

The six built-in UI styles name these as `cjk` and `cjkDisplay`
(`templates/ui-kit/tools/make-styles.py`): Simplified Chinese faces matched to
each pack's Latin voice. Each is a subset made with fontTools
(`pyftsubset`-equivalent: GB2312, ASCII, CJK and fullwidth punctuation; the
`kern`, `palt`, `halt`, `liga`, `locl` and `ccmp` features; no hinting); a
variable source is instanced at the weight named. The cook bakes only the CJK
punctuation into the atlas (`*.ttf.ogt`: `runtime true`); every other
character renders from the outlines when a game first shows it. Fusion Pixel
is also the pixel style's Latin face: its import cooks Latin-1 too, on its
12-pixel grid (`pixel 12`).

| File | Licence | Origin |
|---|---|---|
| `NotoSansSC-Regular.ttf` | SIL OFL 1.1 (`OFL-NotoSansSC.txt`) | Noto Sans SC (google/fonts `ofl/notosanssc`), the variable font at wght 400, subset. Copyright 2014-2021 Adobe, with Reserved Font Name 'Source' |
| `NotoSansSC-Bold.ttf` | SIL OFL 1.1 (`OFL-NotoSansSC.txt`) | the same at wght 700 |
| `NotoSerifSC-SemiBold.ttf` | SIL OFL 1.1 (`OFL-NotoSerifSC.txt`) | Noto Serif SC (google/fonts `ofl/notoserifsc`) at wght 600, subset. Copyright 2012 Google Inc. |
| `ZCOOLKuaiLe-Regular.ttf` | SIL OFL 1.1 (`OFL-ZCOOLKuaiLe.txt`) | ZCOOL KuaiLe (google/fonts `ofl/zcoolkuaile`), subset (the face covers GB2312 level 1). Copyright 2018 The ZCOOL KuaiLe Project Authors |
| `MaShanZheng-Regular.ttf` | SIL OFL 1.1 (`OFL-MaShanZheng.txt`) | Ma Shan Zheng (google/fonts `ofl/mashanzheng`), a brush-calligraphy face, subset (GB2312, ASCII, CJK punctuation). Copyright 2018 The Ma Shan Zheng Project Authors |
| `ZCOOLQingKeHuangYou-Regular.ttf` | SIL OFL 1.1 (`OFL-ZCOOLQingKeHuangYou.txt`) | ZCOOL QingKe HuangYou (google/fonts `ofl/zcoolqingkehuangyou`), a heavy display face, subset (GB2312, ASCII, CJK punctuation). Copyright 2018 The ZCOOL QingKe HuangYou Project Authors |
| `FusionPixel12-zhHans.ttf` | SIL OFL 1.1 (`OFL-FusionPixel.txt`) | Fusion Pixel Font 12px proportional zh_hans (github.com/TakWolf/fusion-pixel-font, release 2026.10.25), subset. Copyright (c) 2022, TakWolf |

| Pack | `cjk` (body, numbers) | `cjkDisplay` (titles) |
|---|---|---|
| pixel | Fusion Pixel 12 | Fusion Pixel 12 |
| cartoon | ZCOOL KuaiLe | ZCOOL KuaiLe |
| fantasy | Noto Serif SC SemiBold | Noto Serif SC SemiBold |
| scifi | Noto Sans SC Regular | Noto Sans SC Bold |
| paper | Noto Sans SC Regular | Noto Serif SC SemiBold |
| neon | Noto Sans SC Bold | Noto Sans SC Bold |
| jade | Noto Serif SC SemiBold | Ma Shan Zheng |
| midcore | Noto Sans SC Bold | ZCOOL QingKe HuangYou |
