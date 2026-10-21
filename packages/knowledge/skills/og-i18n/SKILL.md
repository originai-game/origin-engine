---
name: og-i18n
description: Localizing a game on Origin Engine with og.i18n — string tables per locale as .ogt, messages with arguments, plurals and select (ICU), the host's locale and the player's choice, font chains for Chinese, Japanese, Korean and Arabic, CJK line breaking (kinsoku), shaping and right-to-left layout in og.ui, the cook-time check for missing keys, and how to prove each locale with frames. Use when a game shows text to players in more than one language, adds a language, or shows boxes or broken Arabic.
---

Every string a player reads comes from a table, never from a literal in a
system. `og.i18n` picks the table for the locale the host asks for (the
browser's language, `og run --locale`) or the player chooses, formats the
message, and hands og.ui the fonts and the direction for it. The API is in
`reference/i18n.md`; the platform's `localization` skill covers store
listings and the catalog's languages.

## 1. One table per locale

```text
type strings
version 1
---
locale "ja"
fonts "fonts/NotoSansJP-game.otf"        # this locale's font chain, first first
string "menu.play" "プレイ"
string "hud.coins" "{n, plural, other {コイン#枚}}"
string "results.time" "タイム {time}"
```

- One file per locale under `assets/strings/` (`en.ogt`, `zh-CN.ogt`,
  `ja.ogt`, `ko.ogt`, `ar.ogt`); keys are dotted by screen
  (`menu.play`, `pause.resume`, `shop.buy`). Mark the base table `base
  true` (else `en` is the base).
- Load every table in `start`: `og.i18n.load(world, "strings/ja.ogt")`;
  the active locale is negotiated again after each load (`ja-JP` takes
  `ja`, `zh-TW` takes `zh-Hant` or `zh-TW`, else the base).
- A key a table lacks falls back to its `fallback` table, then the base, and
  `og cook` warns about it. Treat a cook warning as a bug: `og polish`
  scores missing keys (og-build §5).

## 2. Messages

`og.i18n.t(world, "hud.coins", { n: coins })`. The forms:

| Form | Example |
|---|---|
| an argument | `"Hello, {name}"` |
| a number, grouped for the locale | `"{score, number} pts"` |
| plural (CLDR categories `zero one two few many other`, `=N` exact, `#` the number) | `"{n, plural, =0 {No coins} one {# coin} other {# coins}}"` |
| select | `"{g, select, f {She won} m {He won} other {They won}}"` |
| a literal brace or quote | `'{'`, `''` |

- Big numbers in a locale's own units (K, M, B in English; 万, 亿 in
  Chinese) are table data too: a key for how many digits a unit spans and
  a key per unit, read by one formatter (`templates/archetypes/idle`'s
  `big` in `src/numbers.ts`).
- Whole sentences per key, never concatenated pieces: word order differs
  (Japanese puts the verb last, Arabic reads right to left).
- Plurals in every table, even where the language has one form
  (`{n, plural, other {コイン#枚}}`): the argument names stay the same in
  every locale, and `og cook` checks they do.
- Numbers, times and prices go through arguments, not string building.
- og.ui's templates have words of their own (Retry, Share, "Best 1,200",
  "New best", Cancel): each is a field of the template's desc with the
  English default (`ResultsDesc.bestLabel`, `newBestLabel`, `shareLabel`,
  `actions`; `ConfirmDesc.confirmLabel`, `cancelLabel`), so a localised
  game passes its table's: `bestLabel: og.i18n.t(world, "results.best")`
  with `"Best {best}"` / `"最佳 {best}"` in the tables (the template fills
  `{best}`, formatted).
- Emphasis inside a sentence is rich-text markup in the message itself
  (`"Your progress in slot [b]{slot}[/b] will be lost."`, shown with
  `og.ui.richText`): each translation bolds or colours its own words, in
  its own order, instead of the code splitting the sentence into styled
  pieces.

## 3. Fonts for every script

- Latin and Cyrillic: the skin's font. Every other script: a subset of a
  Noto face for that script holding exactly the characters your tables
  use, named first in that table's `fonts` line.
- What the engine ships: `fonts/NotoSansSC-Regular-subset.ttf`, in every
  project (Simplified Chinese: GB2312's 6,763 hanzi, kana and Latin-1,
  2.5 MB), the source for a Chinese subset and the skins' CJK `fallback`.
  The ui-kit's `NotoSansJP-subset.otf`, `NotoSansKR-subset.otf` and its
  Arabic, Hebrew and Devanagari subsets hold only the ui-kit's own strings
  (the Japanese one 177 characters, 28 of them kanji): they are no source
  for a game's text. A Japanese, Korean, Traditional Chinese or Arabic
  game brings the whole Noto face (notofonts, SIL OFL; Noto Sans CJK's
  Regular `.ttc` holds Japanese as face 0, Korean 1, Simplified 2,
  Traditional 3) into a folder outside `assets/` (`tools/fonts/`), so the
  cook never sees it, and records its licence.
- Make the subset with `templates/ui-kit/tools/subset-font.py`, copied into
  the project's `tools/`: it reads every `string` of `assets/strings/`
  (`--locale ja` for one table, `--extra` for what players or playtests
  type), writes the subset and its import settings (`chars` listing every
  character) and exits 1 naming each character the source lacks. Run it
  again after a string changes, then `og cook`:

  ```sh
  python3 tools/subset-font.py tools/fonts/NotoSansCJK-Regular.ttc assets/fonts/NotoSansJP-game.otf --face 0 --locale ja
  python3 tools/subset-font.py assets/fonts/NotoSansSC-Regular-subset.ttf assets/fonts/NotoSansSC-game.ttf --locale zh-CN --cjk-only
  ```

  `--cjk-only` leaves Latin out, so a label mixing scripts takes its Latin
  from the skin's face run by run; with Latin kept, the whole label is
  drawn in the subset (og-ui §7 has the rule).
- Never take Japanese from the Chinese face: it lacks the Japanese forms of
  common kanji (鐘 鈴 険 続 設 語 楽 効 are not in it) and draws the kanji
  both scripts share in their Chinese shapes. The fallback chain below is a
  safety net for a character a subset missed, not a font for a language:
  a Japanese string that reaches the skin's Chinese `fallback` is drawn in
  Chinese forms, and one no font has is a box with a warning on the first
  frame that draws it.
- Each font's import settings (`<font>.ogt`) list its characters (`chars`)
  or a charset: `ascii`, `latin`, `kana`, `hangul`, `arabic`. A font
  cooked with the `arabic` charset also carries its joining forms and its
  outlines, which og-ui's shaper needs.
- og-ui looks a glyph up in the active locale's fonts first, then every
  other loaded table's, then `UiDesc.fallbackFonts` (fonts for scripts no
  table names: a Hebrew or Devanagari face), then the skin's. A box (tofu)
  in a frame means a font lacks the character: add it to the subset and
  to `chars`. One label is drawn in one font, so the font that covers a
  Hebrew word must also have the digits beside it.
- Player-typed text (a name, a chat line) is not in any table: cook the
  CJK face with `runtime true` in its import settings and the characters
  outside its atlas are drawn at run time (about 1 to 2 ms each, once per
  character), with a warning the first time. Keep `chars` for everything
  the tables hold, so a frame never waits on them; `charset all` for CJK
  is about 20 MB and never the answer.
- Complex scripts are shaped from the font file the cook keeps: Arabic
  joins, Devanagari conjuncts and pre-base vowel signs, marks over their
  bases. Cook such a font with the script's charset (`arabic`) or its
  `chars`; check a frame of every shaped string (a conjunct drawn as
  separate letters means the font or its file is missing). Kerning pairs
  apply in every font that has them: nothing to set.
- Chinese, Japanese and Korean break between characters under kinsoku:
  closing brackets and quotes, 、。，？！, small kana, ー and the iteration
  marks never start a line, opening brackets never end one, and a katakana
  word stays whole (オリジン・ / クエスト, not オリジン・クエス / ト);
  Latin breaks at spaces. Nothing to configure: write the string as it
  reads and check the frame.

## 4. Right to left

Arabic, Hebrew, Persian and Urdu locales set the UI's direction to
right-to-left: og.ui mirrors the whole layout about the surface's centre
(a corner anchored top-left moves top-right, rows run right to left),
flips start and end alignment, and orders words by direction while numbers
and Latin words inside stay left to right (by the Unicode bidirectional
algorithm, over each whole line of `og.ui.richText`; a label takes the
direction of its first strong character). Do not mirror the play field or
the camera. Icons are drawn as they are: give a direction icon (back, next)
its mirrored twin for right to left, and leave icons of objects alone.

## 5. The player's choice

The settings screen's language list calls `og.i18n.setLocale(world, tag)`
and saves the choice with the other settings (`og.platform.saveSettings`,
og-platform-sdk): restore it on the first tick. `og.i18n.locale(world)`
answers the requested and active locale, the direction and the font chain.

## 6. Prove it

- A playtest per locale that matters (`templates/ui-kit/tests/locales.og.ts`):
  `locale:<tag>` in the command script switches the locale, then
  `screenshot` each main screen and compare with references; read each
  frame for boxes, clipped words and wrong direction.
- Check text fits: the longest translation (German, Russian) of each
  button in `dump_ui` stays inside its element.
- `og cook` prints `warning:` lines for missing or mismatched keys; the
  build is not done while any remain.
