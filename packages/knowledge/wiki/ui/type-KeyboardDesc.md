<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-KeyboardDesc"></a>

### `KeyboardDesc` (desc struct)

An on-screen keyboard: what it edits and how it types.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `string` | `""` | — | — | — | The text it edits, from game state. |
| `layout` | `KeyboardLayout` | `"letters"` | — | — | — | The keys it opens with (the player switches between letters and symbols, and kana when allowed). |
| `kana` | `boolean` | `false` | — | — | — | The kana layout is offered. |
| `compose` | `boolean` | `false` | — | — | — | Letters compose (an IME: pinyin, romaji) instead of typing: the composition shows underlined and candidates take it. |
| `candidates` | `readonly string[]` | `[]` | — | — | — | Conversions of the composition the game offers (a dictionary's, for KeyboardResult.composition); the kana layout adds its katakana itself. |
| `maxChars` | `number` | `0` | — | — | — | The most characters (0: no limit). |
| `title` | `string` | `""` | — | — | — | What is being typed (`Name your hero`); empty for none. |
| `placeholder` | `string` | `""` | — | — | — | Shown while the text is empty. |
| `submitLabel` | `string` | `""` | — | — | — | The enter key's label; empty is `Done`. |
| `password` | `boolean` | `false` | — | — | — | Show the text as dots (the game keeps it); a numeric layout then takes digits only (a PIN: no sign, point or caret keys). |

```ogt
struct KeyboardDesc {
  doc "An on-screen keyboard: what it edits and how it types."
  desc true
  field value String "The text it edits, from game state."
  field layout KeyboardLayout "The keys it opens with (the player switches between letters and symbols, and kana when allowed)."
  field kana bool "The kana layout is offered."
  field compose bool "Letters compose (an IME: pinyin, romaji) instead of typing: the composition shows underlined and candidates take it."
  field candidates List.String "Conversions of the composition the game offers (a dictionary's, for KeyboardResult.composition); the kana layout adds its katakana itself."
  field maxChars u32 "The most characters (0: no limit)."
  field title String "What is being typed (`Name your hero`); empty for none."
  field placeholder String "Shown while the text is empty."
  field submitLabel String "The enter key's label; empty is `Done`."
  field password bool "Show the text as dots (the game keeps it); a numeric layout then takes digits only (a PIN: no sign, point or caret keys)."
}
```

<sub>`api.ogt` line 19901</sub>
