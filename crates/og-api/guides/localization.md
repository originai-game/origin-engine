# Localization

How a game speaks the player's language: string tables per locale, ICU messages, locale selection and fallback, the font chain and right-to-left layout, ruby, vertical text and emphasis marks, and formatting numbers, money, dates and durations without reading a clock. Fonts per script and checking a screen in other locales are in `screens §6`; dialogue lines are in `yarn §2`.

The API: `og api show i18n`.

## 1. String tables

One typed declaration per locale, under a module's `content/strings/`:

```ts
import { file, ref } from '@origingame/engine/author';
export default file.strings({
  locale: 'ja',
  fonts: [ref('base/fonts/NotoSansJP')],
  entries: {
    'shop.title': 'ショップ',
    coins: '{n, plural, =0 {コインなし} other {# コイン}}',
  },
});
```

The output of `content/strings/ja.ts` in the base module is `base/strings/ja`.
Font refs name actual font declarations or scanned binary fonts, never guessed
runtime paths. Import the TS table for UI registration; the game bundle carries
only its output ID, not author code. Helpers stay in `lib/`, outside `content/`.
Use `og api show file:strings` for generated fields and `og api gen` for project
types; `og check` checks those types without running the game.

`og i18n tables --json` reads all module tables in the restricted cook evaluator;
`--builtins` reads the installed engine's UI tables. Creation, pseudo-locales and
preview tools use this output instead of parsing TS or importing it in Bun.
`--index` returns project table IDs and paths without evaluating content; the
bundler uses that index without transferring every locale's messages.
`og i18n extract --write` adds missing story keys to direct `entries` literals
with source-bound edits. A computed/shared expression is read-only: edit its
source. With no table, it creates `content/strings/en.ts` without overwriting.

- Messages are a subset of ICU MessageFormat: `{name}`, `{n, number}`,
  `{n, plural, =0 {..} one {..} other {..}}` with `#`, and
  `{g, select, a {..} other {..}}`. Plurals follow CLDR's rules.
- `og cook` warns about every key a locale lacks, or takes other arguments
  for, than the base table. Fix each warning; a missing key reads the
  fallback silently in play.
- Never concatenate translated fragments: one message with arguments
  keeps word order translatable.

## 2. Which locale is active

- The host asks for a locale: the browser's `navigator.language` on the
  web, `og run --locale` natively.
- The active locale is the best loaded table for the request (ja-JP takes
  ja; zh-TW takes zh-Hant or zh-TW), else the base table (`base: true`,
  else en, else the first).
- A key the active table lacks is read from its `fallback` table, then
  the base.

## 3. Fonts and direction

Author a pack in `content/fonts/<name>/index.ts` with `file['font-extension']`.
Its ordered faces are canonical font refs; its `.hyph` inputs are paths relative
to the module root. Binary fonts and dictionaries live under `assets/fonts/`.
Use `og api show file:font-extension` for generated types. `og font add` and
`og create --lang` install both roots; `extension.json` holds only discovery
metadata and language/script search tags, never runtime membership. Editing
the TS declaration changes the cooked index, not another font's own settings.

- The active locale sets the UI's font chain: its own `fonts` first, then
  the locale's installed font pack, then packs a frame needed for
  characters only they cover, then the other tables' fonts.
- Every font ships whole: text that exists only at run time (a player's
  name, chat, a server's text) draws like any other. A style pack's
  faces and font packs' faces are not in a web build's first download;
  each is fetched after the boot, and until it lands its characters draw
  from the next face (the default face, then the locale's). A character
  a pack's faces lack takes the default face before the locale's.
- Every UI draws Chinese with nothing installed: the bundle carries the
  `chinese-simplified` pack and fetches it the first time a frame shows a
  character only it has. A game in Japanese, Korean or Traditional
  Chinese adds that language's pack (`og font add japanese`) for its
  forms.
- Chinese, Japanese, Thai, Lao, Khmer and Burmese are written without
  spaces, so their words and line breaks come from ICU's dictionaries.
  The cook ships a dictionary in the font pack of a locale that needs one
  (`og font add thai` for a `th` table), so a game pays only for its own
  locales; a locale whose language no installed pack names is warned of
  at cook, and its text breaks by the rules alone (whole runs, not words).
- A right-to-left locale (ar, he, fa, ur) mirrors og.ui's layout. Arabic
  words are shaped with the font's own tables: every font ships its file
  whole.

## 4. Chinese and Japanese typesetting

- Furigana and pinyin are HTML ruby inside a text: `<ruby>漢字<rt>かんじ</rt></ruby>`,
  one `<rt>` after each base it annotates (`<rb>` optional, `<rp>` never
  shown). A ruby is one unbreakable box; annotate word by word, not a whole
  sentence, or the line cannot break inside it.
- A second level goes in `<rtc>`: with `ruby-position: alternate` (the
  default) it sits under the base, a translation under furigana
  (`<ruby>東京<rt>とうきょう</rt><rtc>Tokyo</rtc></ruby>`); words directly
  in an `<rtc>` span all its bases. Zhuyin beside Traditional Chinese is
  `ruby-position: inter-character`.
- Leave room for ruby and emphasis marks in `line-height`. An annotation or
  mark fits a line whose leading holds half an em on its side; a tighter
  line grows by what it lacks, so a dialogue box whose height is its
  lines' must allow for the taller line.
- `ruby-overhang: auto` (the default) lets a long annotation hang over the
  text beside it by up to one of its characters; use `none` where the text
  beside a ruby is a number or a button label that must not be covered.
- Vertical text: `writing-mode: vertical-rl` with the face's own vertical
  forms (`vmtx`, `vert`), which the cooked CJK font packs keep. Set dates
  and two-digit numbers upright with `text-combine-upright: digits 2`;
  longer numbers lie sideways. A short Latin abbreviation or a number in a
  run of its own takes one upright em with `text-combine-upright: all` on
  its `::part` (`<span part="n">ABC</span>`, `p::part(n) { … }`). Latin words lie sideways by default;
  `text-orientation: upright` stacks them, which reads well only for short
  initialisms.
- Emphasis marks (`text-emphasis: filled`) take their side from the
  content language, as CSS Text Decoration 4's default sheet does: under
  horizontal Chinese text (`lang="zh"` on an element, or a Chinese locale
  at the root), over Japanese and Korean, right of vertical text in all
  three. Set `lang` on mixed-language text; `text-emphasis-position`
  overrides it.

## 5. Formatting

- formatCompact, formatCurrency, formatDate, formatTime, formatDuration,
  and the message styles `{n, number, compact}`,
  `{p, number, currency/EUR}`, `{d, date, short}` and `{d, time}`, write
  from a CLDR subset built into the engine: the languages the plural rules
  cover and zh-Hant. Any other language is written as en.
- Formatting follows the active locale, else the requested one, else en.
- A date or time is milliseconds since 1970-01-01 00:00 UTC that the game
  passes in (og.platform.now, PlatformResult.time, a save's timestamp, a
  server's event time), in a message's arguments too. Nothing reads a
  clock, so the same value gives the same text on every host and in a
  replay. A message's dates are UTC; formatDate and formatTime take an
  offset for the player's zone.
