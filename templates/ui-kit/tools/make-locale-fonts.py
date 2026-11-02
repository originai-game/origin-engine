#!/usr/bin/env python3
"""The UI kit's locale fonts: Noto subsets holding exactly the characters
its string tables (assets/strings/*.ogt) use, plus printable ASCII, plus
every character of the kit's language lists (the settings screen's
LANGUAGES in src/main.ts and the gallery's select in src/controls.ts, read
from the sources: each language is named in its own script, 한국어 and
العربية whatever the locale, so every subset whose face has a name's
characters carries them, and a name draws in every locale).

    python3 tools/make-locale-fonts.py <NotoSansCJK-Regular.ttc> <NotoSansArabic.ttf>

Sources (SIL OFL 1.1, assets/fonts/OFL.txt): Noto Sans CJK (notofonts/noto-cjk,
the Regular TTC: face 0 is Japanese, face 1 Korean) and Noto Sans Arabic
(notofonts/arabic). Writes assets/fonts/NotoSansJP-subset.otf,
NotoSansKR-subset.otf and NotoSansArabic-subset.ttf with fontTools'
subsetter, keeping the layout features (the Arabic joining forms and
ligatures og-ui shapes with), and each font's import settings (.ogt) with
the same characters. zh-CN uses the kit's existing NotoSansSC subset; its
import settings gain the table's characters and `runtime true`: the strings
in RUNTIME_KEYS (the main menu's greeting) are left out of zh-CN's and ja's
characters, so the Chinese font renders them from its outlines at run time
(og-ui's CJK strategy, crates/og-ui/README.md §Text).
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
# Strings drawn with run-time glyphs in zh-CN and ja (not cooked).
RUNTIME_KEYS = {"menu.greeting"}
# Strings mixing scripts, drawn a run per face from the other locales' fonts
# (og-ui's font per run): no locale's subset takes their characters.
MIXED_KEYS = {"menu.mixed"}


def chars_of(locale: str) -> str:
    text = (STRINGS / f"{locale}.ogt").read_text(encoding="utf-8")
    skip = (RUNTIME_KEYS if locale in ("zh-CN", "ja") else set()) | MIXED_KEYS
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


def main() -> None:
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


if __name__ == "__main__":
    main()
