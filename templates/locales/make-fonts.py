#!/usr/bin/env python3
"""The locale faces `og new --lang` installs (templates/locales/fonts/).

Each is a Noto face instanced at Regular (wght 400, wdth 100) and subset
with fontTools, keeping every layout feature (Arabic joining, Hebrew marks,
CJK `locl`, `palt`, `vert`) and dropping hinting. The CJK subsets hold a
national standard's common set, not one game's strings, so a game's tables
can grow without a new subset; a game cuts its own smaller one before it
ships (`subset-font.py`). They hold no Latin: a label mixing
scripts takes its Latin from the style's face, run by run.

    python3 templates/locales/make-fonts.py [--cache DIR]

The sources are fetched from google/fonts (SIL OFL 1.1) into the cache
(default .amp/in/fonts-src) when missing. Writes templates/locales/fonts/*
and prints each file's size and character count. LICENSES.md records the
sources; rerun this after changing a set below and update it.
"""
import argparse
import urllib.parse
import urllib.request
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

HERE = Path(__file__).resolve().parent
OUT = HERE / "fonts"
RAW = "https://github.com/google/fonts/raw/main/ofl/"


def rng(a: int, b: int) -> set[int]:
    return set(range(a, b + 1))


def codec(name: str, lead: range, trail: range) -> set[int]:
    """Every character a double-byte codec decodes from the lead and trail bytes given."""
    out: set[int] = set()
    for hi in lead:
        for lo in trail:
            try:
                out.update(ord(c) for c in bytes([hi, lo]).decode(name))
            except UnicodeDecodeError:
                pass
    return out


# CJK and fullwidth punctuation, shared by the three CJK faces.
CJK_PUNCT = rng(0x3000, 0x303F) | rng(0xFF01, 0xFF60) | rng(0xFFE0, 0xFFE6) | {0x2018, 0x2019, 0x201C, 0x201D, 0x2026, 0x2025, 0x30FB}

# JIS X 0208: rows 1-8 (symbols, digits, kana, Greek, Cyrillic, box drawing) and
# rows 16-47 (level 1 kanji, 2,965), through EUC-JP (row r is lead byte 0xA0 + r).
JIS = codec("euc_jp", range(0xA1, 0xA9), range(0xA1, 0xFF)) | codec("euc_jp", range(0xB0, 0xD0), range(0xA1, 0xFF))
# KS X 1001: rows 1-12 (symbols, compatibility jamo) and rows 16-40 (the 2,350 hangul syllables), no hanja.
KSX = codec("euc_kr", range(0xA1, 0xAD), range(0xA1, 0xFF)) | codec("euc_kr", range(0xB0, 0xC9), range(0xA1, 0xFF))
# Big5: the symbols (0xA140-0xA3BF) and level 1 hanzi (0xA440-0xC67E, 5,401), plus bopomofo.
BIG5 = codec("big5", range(0xA1, 0xA4), list(range(0x40, 0x7F)) + list(range(0xA1, 0xFF))) | codec("big5", range(0xA4, 0xC7), list(range(0x40, 0x7F)) + list(range(0xA1, 0xFF))) | rng(0x3100, 0x312F)
# Cyrillic (Russian, Ukrainian, Belarusian, Bulgarian, Serbian, Macedonian, Kazakh) and Greek, no Latin.
CYRILLIC_GREEK = rng(0x0370, 0x03FF) | rng(0x0400, 0x04FF) | rng(0x0500, 0x052F) | rng(0x1F00, 0x1FFF)

ASCII = rng(0x20, 0x7E)

# (output, source file in google/fonts, characters or None for the whole face)
FACES = [
    ("NotoSansJP-Regular-subset.ttf", "notosansjp/NotoSansJP[wght].ttf", (JIS | CJK_PUNCT) - ASCII),
    ("NotoSansKR-Regular-subset.ttf", "notosanskr/NotoSansKR[wght].ttf", (KSX | CJK_PUNCT | rng(0x3131, 0x318E)) - ASCII),
    ("NotoSansTC-Regular-subset.ttf", "notosanstc/NotoSansTC[wght].ttf", (BIG5 | CJK_PUNCT) - ASCII),
    ("NotoSansArabic-Regular.ttf", "notosansarabic/NotoSansArabic[wdth,wght].ttf", None),
    ("NotoSansHebrew-Regular.ttf", "notosanshebrew/NotoSansHebrew[wdth,wght].ttf", None),
    ("NotoSans-CyrillicGreek-subset.ttf", "notosans/NotoSans[wdth,wght].ttf", CYRILLIC_GREEK),
]


def fetch(cache: Path, source: str) -> Path:
    path = cache / Path(source).name
    if not path.exists():
        cache.mkdir(parents=True, exist_ok=True)
        url = RAW + urllib.parse.quote(source)
        print(f"fetch {url}")
        urllib.request.urlretrieve(url, path)
    return path


def regular(font: TTFont) -> TTFont:
    if "fvar" not in font:
        return font
    axes = {a.axisTag: (400 if a.axisTag == "wght" else 100 if a.axisTag == "wdth" else a.defaultValue) for a in font["fvar"].axes}
    return instancer.instantiateVariableFont(font, axes)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cache", type=Path, default=Path(".amp/in/fonts-src"))
    args = ap.parse_args()
    OUT.mkdir(exist_ok=True)
    for out, source, chars in FACES:
        font = regular(TTFont(fetch(args.cache, source)))
        opts = subset.Options()
        opts.layout_features = ["*"]
        opts.name_IDs = ["*"]
        opts.name_languages = ["*"]
        opts.notdef_outline = True
        opts.hinting = False
        opts.glyph_names = False
        sub = subset.Subsetter(opts)
        cmap = font.getBestCmap()
        sub.populate(unicodes=sorted(c for c in (chars if chars is not None else set(cmap)) if c in cmap))
        sub.subset(font)
        font.save(OUT / out)
        print(f"{out}: {(OUT / out).stat().st_size} bytes, {len(font.getBestCmap())} characters")


if __name__ == "__main__":
    main()
