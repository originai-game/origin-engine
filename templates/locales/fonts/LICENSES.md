# The locale faces: licences

`og new --lang` (packages/create, `bin/languages.ts`) copies the face a
chosen language needs into the game's `assets/fonts/`, with its licence
file and this table's row in the game's `assets/fonts/LICENSES.md`. Every
face is a Noto face from google/fonts at Regular (wght 400, wdth 100),
made by `../make-fonts.py` with fontTools: every layout feature kept, no
hinting. The CJK subsets hold a national standard's common set and no
Latin (a label's Latin comes from the style's face, run by run); a game
cuts its own subset of its tables before it ships (og-i18n §3).

| File | Languages | Characters | Licence | Origin |
|---|---|---|---|---|
| `NotoSansJP-Regular-subset.ttf` | ja | JIS X 0208 rows 1-8 and level 1 kanji (2,965), CJK and fullwidth punctuation | SIL OFL 1.1 (`OFL-NotoSansJP.txt`) | Noto Sans JP (google/fonts `ofl/notosansjp`). Copyright 2014-2021 Adobe, with Reserved Font Name 'Source' |
| `NotoSansKR-Regular-subset.ttf` | ko | KS X 1001 rows 1-12 and the 2,350 hangul syllables, compatibility jamo, CJK punctuation | SIL OFL 1.1 (`OFL-NotoSansKR.txt`) | Noto Sans KR (google/fonts `ofl/notosanskr`). Copyright 2014-2021 Adobe, with Reserved Font Name 'Source' |
| `NotoSansTC-Regular-subset.ttf` | zh-TW, zh-HK, zh-Hant | Big5 symbols and level 1 hanzi (5,401), bopomofo, CJK punctuation | SIL OFL 1.1 (`OFL-NotoSansTC.txt`) | Noto Sans TC (google/fonts `ofl/notosanstc`). Copyright 2014-2021 Adobe, with Reserved Font Name 'Source' |
| `NotoSansArabic-Regular.ttf` | ar, fa, ur, ps, sd, ug, ckb | the whole face (Arabic, its joining forms and digits, Basic Latin) | SIL OFL 1.1 (`OFL-NotoSansArabic.txt`) | Noto Sans Arabic (google/fonts `ofl/notosansarabic`). Copyright 2022 The Noto Project Authors |
| `NotoSansHebrew-Regular.ttf` | he, yi | the whole face (Hebrew with its points, Basic Latin) | SIL OFL 1.1 (`OFL-NotoSansHebrew.txt`) | Noto Sans Hebrew (google/fonts `ofl/notosanshebrew`). Copyright 2022 The Noto Project Authors |
| `NotoSans-CyrillicGreek-subset.ttf` | ru, uk, be, bg, sr, mk, kk, ky, mn, el | Cyrillic, Cyrillic Supplement, Greek and Extended Greek; no Latin | SIL OFL 1.1 (`OFL-NotoSans.txt`) | Noto Sans (google/fonts `ofl/notosans`). Copyright 2022 The Noto Project Authors |

Simplified Chinese (zh, zh-CN, zh-SG, zh-Hans) takes the styles' companion
`templates/styles/cjk/NotoSansSC-Regular.ttf` (GB2312, its row in
`templates/styles/cjk/LICENSES.md`), or the 3D template's own
`NotoSansSC-Regular-subset.ttf` when the game already has it.
