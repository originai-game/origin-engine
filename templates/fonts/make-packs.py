#!/usr/bin/env python3
"""The font packs (docs/ui.md §3.4): one directory per script under
templates/fonts/, each with its faces, their import settings, an
`extension.json` of kind `font`, its licence and LICENSES.md.

Each face is a Noto face from google/fonts instanced at Regular (wght
400, wdth 100) and subset with fontTools, keeping every layout feature
(Arabic joining, Devanagari conjuncts, Thai marks, CJK `locl`, `palt`,
`vert`) and dropping hinting. The CJK subsets hold a national standard's
common set, not one game's strings. A face cooks no atlas (its space or ideographic space alone)
and draws every character from its outlines when first shown (`runtime
true`), so a game pays per character it shows and a pack cooks to about
its face's size. A pack takes no Latin a style's face already draws, except
`latin`, the fallback for the Latin a style face lacks (Vietnamese,
Latin Extended).

    python3 templates/fonts/make-packs.py [--cache DIR]

Sources are fetched into the cache (default .amp/in/fonts-src) when
missing. Rerun after changing a pack below; it rewrites every pack.
"""
import argparse
import json
import urllib.parse
import urllib.request
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

HERE = Path(__file__).resolve().parent
RAW = "https://github.com/google/fonts/raw/main/ofl/"


def rng(a: int, b: int) -> set[int]:
    return set(range(a, b + 1))


def codec(name: str, lead, trail) -> set[int]:
    """Every character a double-byte codec decodes from the lead and trail bytes given."""
    out: set[int] = set()
    for hi in lead:
        for lo in trail:
            try:
                out.update(ord(c) for c in bytes([hi, lo]).decode(name))
            except UnicodeDecodeError:
                pass
    return out


ASCII = rng(0x20, 0x7E)
CJK_PUNCT = rng(0x3000, 0x303F) | rng(0xFF01, 0xFF60) | rng(0xFFE0, 0xFFE6) | {0x2018, 0x2019, 0x201C, 0x201D, 0x2026, 0x2025, 0x30FB}
# JIS X 0208 rows 1-8 and level 1 kanji (2,965).
JIS = codec("euc_jp", range(0xA1, 0xA9), range(0xA1, 0xFF)) | codec("euc_jp", range(0xB0, 0xD0), range(0xA1, 0xFF))
# KS X 1001 rows 1-12 and the 2,350 hangul syllables, no hanja.
KSX = codec("euc_kr", range(0xA1, 0xAD), range(0xA1, 0xFF)) | codec("euc_kr", range(0xB0, 0xC9), range(0xA1, 0xFF))
# Big5 symbols and level 1 hanzi (5,401), plus bopomofo.
BIG5_TRAIL = list(range(0x40, 0x7F)) + list(range(0xA1, 0xFF))
BIG5 = codec("big5", range(0xA1, 0xA4), BIG5_TRAIL) | codec("big5", range(0xA4, 0xC7), BIG5_TRAIL) | rng(0x3100, 0x312F)
# GB 2312 rows 1-9 (symbols) and level 1 hanzi (3,755).
GB = codec("gb2312", range(0xA1, 0xAA), range(0xA1, 0xFF)) | codec("gb2312", range(0xB0, 0xD8), range(0xA1, 0xFF))
LATIN = ASCII | rng(0xA0, 0x24F) | rng(0x0300, 0x036F) | rng(0x1E00, 0x1EFF) | rng(0x2000, 0x206F) | rng(0x20A0, 0x20CF) | {0x2122, 0x2190, 0x2192}
CYRILLIC_GREEK = rng(0x0370, 0x03FF) | rng(0x0400, 0x04FF) | rng(0x0500, 0x052F) | rng(0x1F00, 0x1FFF)


def chars(*ranges) -> str:
    return "".join(chr(c) for r in ranges for c in range(r[0], r[1] + 1))


# name: (description, scripts, languages, [(output, google/fonts source, characters or None)])
PACKS = {
    "latin": (
        "Noto Sans for the Latin a style's face lacks: Latin-1, Latin Extended A and B, Vietnamese, combining marks",
        ["Latn"],
        "en es fr de it pt nl sv da no nb nn fi is pl cs sk sl hr bs ro hu tr az vi id ms tl fil sw et lv lt sq mt ga cy eu ca gl af ha yo ig zu xh uz tk la eo".split(),
        [("NotoSans-Latin.ttf", "notosans/NotoSans[wdth,wght].ttf", LATIN)],
    ),
    "cyrillic-greek": (
        "Noto Sans Cyrillic, Cyrillic Supplement, Greek and Extended Greek",
        ["Cyrl", "Grek"],
        "ru uk be bg sr mk kk ky mn tg el".split(),
        [("NotoSans-CyrillicGreek.ttf", "notosans/NotoSans[wdth,wght].ttf", CYRILLIC_GREEK)],
    ),
    "arabic": (
        "Noto Sans Arabic: Arabic with its joining forms and digits, Persian, Urdu",
        ["Arab"],
        "ar fa ur ps sd ug ckb".split(),
        [("NotoSansArabic-Regular.ttf", "notosansarabic/NotoSansArabic[wdth,wght].ttf", None)],
    ),
    "hebrew": (
        "Noto Sans Hebrew with its points",
        ["Hebr"],
        "he iw yi".split(),
        [("NotoSansHebrew-Regular.ttf", "notosanshebrew/NotoSansHebrew[wdth,wght].ttf", None)],
    ),
    "devanagari": (
        "Noto Sans Devanagari with its conjuncts: Hindi, Marathi, Nepali, Sanskrit",
        ["Deva"],
        "hi mr ne sa".split(),
        [("NotoSansDevanagari-Regular.ttf", "notosansdevanagari/NotoSansDevanagari[wdth,wght].ttf", None)],
    ),
    "thai": (
        "Noto Sans Thai with its marks",
        ["Thai"],
        ["th"],
        [("NotoSansThai-Regular.ttf", "notosansthai/NotoSansThai[wdth,wght].ttf", None)],
    ),
    "japanese": (
        "Noto Sans JP: JIS X 0208 rows 1-8 and level 1 kanji (2,965), kana, CJK and fullwidth punctuation",
        ["Jpan"],
        ["ja"],
        [("NotoSansJP-Regular.ttf", "notosansjp/NotoSansJP[wght].ttf", (JIS | CJK_PUNCT) - ASCII)],
    ),
    "korean": (
        "Noto Sans KR: KS X 1001 rows 1-12 and the 2,350 hangul syllables, compatibility jamo",
        ["Kore"],
        ["ko"],
        [("NotoSansKR-Regular.ttf", "notosanskr/NotoSansKR[wght].ttf", (KSX | CJK_PUNCT | rng(0x3131, 0x318E)) - ASCII)],
    ),
    "chinese-traditional": (
        "Noto Sans TC: Big5 symbols and level 1 hanzi (5,401), bopomofo",
        ["Hant"],
        "zh-Hant zh-TW zh-HK zh-MO".split(),
        [("NotoSansTC-Regular.ttf", "notosanstc/NotoSansTC[wght].ttf", (BIG5 | CJK_PUNCT) - ASCII)],
    ),
    "chinese-simplified": (
        "Noto Sans SC: GB 2312 symbols and level 1 hanzi (3,755)",
        ["Hans"],
        "zh zh-Hans zh-CN zh-SG".split(),
        [("NotoSansSC-Regular.ttf", "notosanssc/NotoSansSC[wght].ttf", (GB | CJK_PUNCT) - ASCII)],
    ),
}


def fetch(cache: Path, source: str) -> Path:
    path = cache / Path(source).name
    if not path.exists():
        cache.mkdir(parents=True, exist_ok=True)
        print(f"fetch {RAW + source}")
        urllib.request.urlretrieve(RAW + urllib.parse.quote(source), path)
    return path


def licence(cache: Path, source: str) -> str:
    family = source.split("/")[0]
    path = cache / f"OFL-{family}.txt"
    if not path.exists():
        urllib.request.urlretrieve(RAW + family + "/OFL.txt", path)
    return path.read_text()


def regular(font: TTFont) -> TTFont:
    if "fvar" not in font:
        return font
    axes = {a.axisTag: (400 if a.axisTag == "wght" else 100 if a.axisTag == "wdth" else a.defaultValue) for a in font["fvar"].axes}
    return instancer.instantiateVariableFont(font, axes)


SIDECAR = """type import
version 1
---
# Made by templates/fonts/make-packs.py: no atlas cooked (its space alone);
# every character is drawn from the face's outlines when first shown, so
# a game pays only for what it shows. px_range is a third of em_px, so a
# 2 px outline reaches 16 px text.
font {{
  charset
  chars " 　"
  em_px 48
  px_range 16
  runtime true
}}
"""


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cache", type=Path, default=Path(".amp/in/fonts-src"))
    args = ap.parse_args()
    rows = []
    for name, (description, scripts, languages, faces) in PACKS.items():
        out = HERE / name
        out.mkdir(exist_ok=True)
        for old in out.iterdir():
            old.unlink()
        files = []
        for file, source, wanted in faces:
            font = regular(TTFont(fetch(args.cache, source)))
            opts = subset.Options()
            opts.layout_features = ["*"]
            opts.name_IDs = ["*"]
            opts.name_languages = ["*"]
            opts.notdef_outline = True
            opts.hinting = False
            opts.glyph_names = False
            cmap = font.getBestCmap()
            sub = subset.Subsetter(opts)
            sub.populate(unicodes=sorted(c for c in (wanted if wanted is not None else set(cmap)) if c in cmap))
            sub.subset(font)
            font.save(out / file)
            (out / f"{file}.ogt").write_text(SIDECAR.format())
            ofl = f"OFL-{source.split('/')[0]}.txt"
            (out / ofl).write_text(licence(args.cache, source))
            count = len(font.getBestCmap())
            size = (out / file).stat().st_size
            files.append(file)
            rows.append(f"| `{name}/{file}` | {', '.join(languages)} | {count:,} characters, {size:,} bytes | SIL OFL 1.1 (`{ofl}`) | google/fonts `ofl/{source}` |")
            print(f"{name}/{file}: {size} bytes, {count} characters")
        manifest = {
            "format": 1,
            "kind": "font",
            "name": name,
            "version": "1.0.0",
            "description": description,
            "scripts": scripts,
            "languages": languages,
            "faces": files,
        }
        (out / "extension.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    table = "\n".join(rows)
    (HERE / "LICENSES.md").write_text(f"""# Font packs: licences

Made by `make-packs.py` from google/fonts: each a Noto face at Regular (wght
400, wdth 100), every layout feature kept, no hinting. Copyright The Noto
Project Authors; the CJK faces Copyright 2014-2021 Adobe, with Reserved Font
Name 'Source'. Each pack carries its licence file, which `og font add` and
`og create --lang` install with it.

| File | Languages | Holds | Licence | Source |
|---|---|---|---|---|
{table}
""")


if __name__ == "__main__":
    main()
