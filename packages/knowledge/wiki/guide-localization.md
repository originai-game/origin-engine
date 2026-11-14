<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `localization` guide: Localization

How a game speaks the player's language: string tables per locale, ICU messages, locale selection and fallback, the font chain and right-to-left layout, and formatting numbers, money, dates and durations without reading a clock. Fonts per script and checking a screen in other locales are in `screens §6`; dialogue lines are in `yarn §2`.

The API: `og api show i18n`.

<a id="guide-localization-1"></a>
### `localization §1` String tables

One `.ogt` of `type strings` per locale, under `assets/strings/`:

```text
type strings
version 1
---
locale "ja"
fonts "fonts/NotoSansJP.ttf"
string "shop.title" "ショップ"
string "coins" "{n, plural, =0 {コインなし} other {# コイン}}"
```

- Messages are a subset of ICU MessageFormat: `{name}`, `{n, number}`,
  `{n, plural, =0 {..} one {..} other {..}}` with `#`, and
  `{g, select, a {..} other {..}}`. Plurals follow CLDR's rules.
- `og cook` warns about every key a locale lacks, or takes other arguments
  for, than the base table. Fix each warning; a missing key reads the
  fallback silently in play.
- Never concatenate translated fragments: one message with arguments
  keeps word order translatable.

<a id="guide-localization-2"></a>
### `localization §2` Which locale is active

- The host asks for a locale: the browser's `navigator.language` on the
  web, `og run --locale` natively.
- The active locale is the best loaded table for the request (ja-JP takes
  ja; zh-TW takes zh-Hant or zh-TW), else the base table (`base true`,
  else en, else the first).
- A key the active table lacks is read from its `fallback` table, then
  the base.

<a id="guide-localization-3"></a>
### `localization §3` Fonts and direction

- The active locale sets the UI's font chain: its own `fonts` first, then
  the locale's installed font pack, then packs a frame needed for
  characters only they cover, then the other tables' fonts.
- A right-to-left locale (ar, he, fa, ur) mirrors og.ui's layout. og-ui
  shapes Arabic words with the font's own tables, which a font cooked with
  the `arabic` charset carries.

<a id="guide-localization-4"></a>
### `localization §4` Formatting

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
