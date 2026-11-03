#!/usr/bin/env python3
"""The UI kit's locale fonts: Noto subsets holding exactly the characters
its string tables (assets/strings/*.ogt) use, plus printable ASCII, plus
every character of the kit's language lists (the settings screen's
LANGUAGES in src/main.ts and the gallery's select in src/controls.ts, read
from the sources: each language is named in its own script, 한국어 and
العربية whatever the locale, so every subset whose face has a name's
characters carries them, and a name draws in every locale).

    python3 tools/make-locale-fonts.py <NotoSansCJK-Regular.ttc> <NotoSansArabic.ttf>
    python3 tools/make-locale-fonts.py --check

--check (and every run, after writing) compares each locale's strings with
the face that draws them: a character its subset maps to an empty glyph or
lacks, one its import does not cook, a cooked atlas cell with no ink
(cooked/fonts, when the kit is cooked), or one a style pack's CJK companion
maps to an empty glyph is a gap, and the tool exits 1.

Sources (SIL OFL 1.1, assets/fonts/OFL.txt): Noto Sans CJK (notofonts/noto-cjk,
the Regular TTC: face 0 is Japanese, face 1 Korean) and Noto Sans Arabic
(notofonts/arabic). Writes assets/fonts/NotoSansJP-subset.otf,
NotoSansKR-subset.otf and NotoSansArabic-subset.ttf with fontTools'
subsetter, keeping the layout features (the Arabic joining forms and
ligatures og-ui shapes with), and each font's import settings (.ogt) with
the same characters. zh-CN uses the kit's existing NotoSansSC subset; its
import settings gain the table's characters and `runtime true`: the strings
in RUNTIME_KEYS (the main menu's greeting) are left out of zh-CN's
characters, so the Chinese font renders them from its outlines at run time
(og-ui's CJK strategy, crates/og-ui/README.md §Text); ja's subset carries
them (its 旅 is not in the Chinese subset's cooked cells).
"""
import re
import sys
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTCollection, TTFont

KIT = Path(__file__).resolve().parent.parent
STRINGS = KIT / "assets" / "strings"
# The language lists: the settings screen's ({ code, label } rows) and the gallery's select (plain names).
LISTS = [(KIT / "src" / "main.ts", r'label: "([^"]*)"'), (KIT / "src" / "controls.ts", r'"([^"]*)"')]
FONTS = KIT / "assets" / "fonts"
ASCII = "".join(chr(c) for c in range(0x20, 0x7F))
# Strings drawn with run-time glyphs in zh-CN (not cooked).
RUNTIME_KEYS = {"menu.greeting"}
# Strings mixing scripts, drawn a run per face from the other locales' fonts
# (og-ui's font per run): no locale's subset takes their characters.
MIXED_KEYS = {"menu.mixed"}


def chars_of(locale: str) -> str:
    text = (STRINGS / f"{locale}.ogt").read_text(encoding="utf-8")
    skip = (RUNTIME_KEYS if locale == "zh-CN" else set()) | MIXED_KEYS
    used = "".join(
        m.group(2) for m in re.finditer(r'^string "([^"]*)" "(.*)"$', text, re.M) if m.group(1) not in skip
    )
    return "".join(sorted(set(used + ASCII)))


def endonyms() -> str:
    """The characters of every language list's names (each file's `const LANGUAGES = [...]`)."""
    out = ""
    for path, name in LISTS:
        block = re.search(r"^const LANGUAGES = \[(.*?)\];", path.read_text(encoding="utf-8"), re.M | re.S)
        if not block:
            sys.exit(f"{path}: no `const LANGUAGES = [...]` (a language list)")
        out += "".join(re.findall(name, block.group(1)))
    return out


def covered(font: TTFont, text: str) -> str:
    """The characters of `text` the font has."""
    cmap = font.getBestCmap()
    return "".join(c for c in text if ord(c) in cmap)


def write_subset(font: TTFont, text: str, out: Path) -> None:
    opts = subset.Options()
    opts.layout_features = ["*"]
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    opts.glyph_names = False
    sub = subset.Subsetter(opts)
    sub.populate(text=text)
    sub.subset(font)
    font.save(out)


def settings(name: str, charset: str, chars: str, runtime: bool = False) -> None:
    body = "".join(c for c in chars if ord(c) > 0x7E and c not in '"\\')
    extra = "  runtime true\n" if runtime else ""
    (FONTS / f"{name}.ogt").write_text(
        "type import\nversion 1\n---\n"
        "# The characters the UI kit's string tables use (tools/make-locale-fonts.py).\n"
        f"font {{\n  charset {charset}\n  chars \"{body}\"\n  em_px 32\n  px_range 11\n{extra}}}\n",
        encoding="utf-8",
    )


# Each locale's face, and the faces of every style pack's CJK companion
# (templates/styles/*/style.ogt `cjk`/`cjkDisplay`, rendered at run time):
# under a pack a CJK string draws its characters from the companion first.
LOCALE_FACES = {"ja": "NotoSansJP-subset.otf", "ko": "NotoSansKR-subset.otf", "zh-CN": "NotoSansSC-Regular-subset.ttf", "ar": "NotoSansArabic-subset.ttf"}
COMPANIONS = KIT.parent / "styles" / "cjk"
# The cook's charset words (og_bundle::format::charset).
CHARSETS = {
    "ascii": [(0x20, 0x7E)], "latin": [(0x20, 0x7E), (0xA0, 0xFF)], "punct": [(0x2000, 0x206F)],
    "cjk-punct": [(0x3000, 0x303F), (0xFF00, 0xFFEF)], "kana": [(0x3040, 0x30FF)],
    "arabic": [(0x0600, 0x06FF), (0x0750, 0x077F), (0xFB50, 0xFDFF), (0xFE70, 0xFEFF)],
}
# Characters no face draws ink for.
INKLESS = {"\u00a0", "\u200c", "\u200d", "\u2009", "\u202f", "\u3000", "\u0640"}


def inked(font: TTFont) -> set:
    """The code points the font maps to a glyph with an outline (a cmap hit drawing nothing is not one)."""
    from fontTools.pens.boundsPen import BoundsPen
    glyphs = font.getGlyphSet()
    out = set()
    for cp, name in font.getBestCmap().items():
        pen = BoundsPen(glyphs)
        glyphs[name].draw(pen)
        if pen.bounds:
            out.add(cp)
    return out


def import_chars(name: str) -> tuple:
    """A font import's cooked code points (its `chars` and charset words) and whether it renders the rest at run time."""
    text = (FONTS / f"{name}.ogt").read_text(encoding="utf-8")
    chars = {ord(c) for c in (re.search(r'^\s*chars "(.*)"$', text, re.M) or [None, ""])[1]}
    for word in (re.search(r"^\s*charset (.*)$", text, re.M) or [None, ""])[1].split():
        for lo, hi in CHARSETS.get(word, []):
            chars |= set(range(lo, hi + 1))
    return chars, re.search(r"^\s*runtime true$", text, re.M) is not None


def cooked_cells(name: str) -> dict:
    """A cooked font's atlas cells with ink, by code point: the cook's blob (cooked/fonts/<name>.ogb) read
    with its glyph rows and atlas pages; empty when the kit is not cooked."""
    import struct
    import zlib
    path = KIT / "cooked" / "fonts" / f"{name}.ogb"
    if not path.exists():
        return {}
    b = path.read_bytes()
    count, names_size = struct.unpack_from("<I", b, 12)[0], struct.unpack_from("<I", b, 48)[0]
    pool = b[64 + count * 48: 64 + count * 48 + names_size]
    sections = {}
    for i in range(count):
        e = 64 + i * 48
        no, nl = struct.unpack_from("<II", b, e)
        off, size = struct.unpack_from("<QQ", b, e + 8)
        sections[pool[no:no + nl].decode()] = (b[off:off + size], struct.unpack_from("<I", b, e + 32)[0])
    pages = []
    while f"atlas.{len(pages)}" in sections:
        ktx = sections[f"atlas.{len(pages)}"][0]
        w, h = struct.unpack_from("<II", ktx, 20)
        off, length = struct.unpack_from("<QQ", ktx, 80)
        px = ktx[off:off + length]
        pages.append((w, h, zlib.decompress(px) if struct.unpack_from("<I", ktx, 44)[0] == 3 else px))
    rows, stride = sections["glyphs"]
    cells = {}
    for i in range(len(rows) // stride):
        cp, page, *_ = struct.unpack_from("<II", rows, i * stride)
        ax, ay, aw, ah = struct.unpack_from("<4I", rows, i * stride + 28)
        ink = False
        if aw and page < len(pages):
            w, h, px = pages[page]
            ink = ay + ah <= h and any(sorted(px[o:o + 3])[1] > 127 for y in range(ay, ay + ah) for o in range((y * w + ax) * 4, (y * w + ax + aw) * 4, 4))
        cells[cp] = ink
    return cells


def check() -> list:
    """Every character each locale's strings draw, against the face that draws it: in its subset's cmap with an
    outline, in its import's characters (or rendered at run time), with an inked cell in the cooked atlas; and
    in each style pack's CJK companion either absent or drawn (a companion that maps a character to an empty
    glyph would win over the locale's face and draw a blank). Returns the gaps."""
    gaps = []
    companions = {p.name: (inked(TTFont(p)), set(TTFont(p).getBestCmap())) for p in sorted(COMPANIONS.glob("*.ttf"))}
    # The kit's body face (NotoSansSC), a fallback in every locale and under every pack, and the one its
    # run-time keys render from.
    body = LOCALE_FACES["zh-CN"]
    faces = {name: (inked(TTFont(FONTS / name)), set(TTFont(FONTS / name).getBestCmap()), *import_chars(name), cooked_cells(name)) for name in LOCALE_FACES.values()}

    def draws(name: str, cp: int, runtime_only: bool = False) -> str:
        """Why `name` does not draw `cp` ("" when it does)."""
        ink, cmap, cooked, runtime, cells = faces[name]
        if cp not in ink:
            return f"{name} draws no ink for it ({'an empty glyph' if cp in cmap else 'not in its cmap'})"
        if runtime_only:
            return "" if runtime else f"{name} renders nothing at run time"
        if cp not in cooked:
            return "" if runtime else f"{name}'s import does not cook it"
        if cells and not cells.get(cp, False):
            return f"cooked/fonts/{name}.ogb has no inked atlas cell for it"
        return ""

    for locale, name in LOCALE_FACES.items():
        table = (STRINGS / f"{locale}.ogt").read_text(encoding="utf-8")
        for m in re.finditer(r'^string "([^"]*)" "(.*)"$', table, re.M):
            key, text = m.group(1), m.group(2)
            if key in MIXED_KEYS:
                continue
            for c in sorted({c for c in text if not c.isspace() and c not in INKLESS and ord(c) > 0x7E}):
                cp = ord(c)
                if key in RUNTIME_KEYS and locale in ("zh-CN", "ja"):
                    why = draws(body, cp, runtime_only=True)
                else:
                    # The locale's face draws it when its atlas has it (a blank cell too); else the body face it
                    # falls back to.
                    why = draws(name, cp)
                    if why and cp not in faces[name][2] and name != body and not draws(body, cp):
                        why = ""
                if why:
                    gaps.append(f"{locale} {key}: {c} U+{cp:04X}: {why}")
                for comp, (ink, cmap) in companions.items():
                    if cp in cmap and cp not in ink:
                        gaps.append(f"{locale} {key}: {c} U+{cp:04X}: the style companion {comp} maps it to an empty glyph")
    return gaps


def main() -> None:
    if sys.argv[1:] == ["--check"]:
        gaps = check()
        print("\n".join(gaps) or "every locale string's characters are drawn by their faces")
        sys.exit(1 if gaps else 0)
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    cjk = TTCollection(sys.argv[1])
    names = endonyms()
    for face, locale, name in [(0, "ja", "NotoSansJP-subset.otf"), (1, "ko", "NotoSansKR-subset.otf")]:
        text = "".join(sorted(set(chars_of(locale) + covered(cjk.fonts[face], names))))
        write_subset(cjk.fonts[face], text, FONTS / name)
        settings(name, "ascii cjk-punct", text)
    arabic = TTFont(sys.argv[2])
    text = chars_of("ar") + covered(arabic, names) + "ـ‌‍"
    write_subset(arabic, text, FONTS / "NotoSansArabic-subset.ttf")
    settings("NotoSansArabic-subset.ttf", "ascii arabic", "")
    # The SC subset is not re-made here: its import takes the names its file has.
    sc = "".join(sorted(set(chars_of("zh-CN") + covered(TTFont(FONTS / "NotoSansSC-Regular-subset.ttf"), names))))
    settings("NotoSansSC-Regular-subset.ttf", "latin cjk-punct", "简体中文原点引擎设置" + sc, runtime=True)
    for p in sorted(FONTS.glob("Noto*-subset.*")):
        if p.suffix in (".otf", ".ttf"):
            print(f"{p.name}: {p.stat().st_size} bytes")
    gaps = check()
    if gaps:
        sys.exit("\n".join(gaps))


if __name__ == "__main__":
    main()
