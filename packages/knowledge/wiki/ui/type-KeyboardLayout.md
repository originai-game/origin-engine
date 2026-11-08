<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-KeyboardLayout"></a>

### `KeyboardLayout` (enum)

An on-screen keyboard's keys.

A string: one of the variants below. The zero value is `"letters"`.

| Variant | Doc |
|---|---|
| `"letters"` | Latin letters (QWERTY), shift and caps, digits on a long press of the top row (the default). |
| `"symbols"` | Digits and punctuation. |
| `"numeric"` | A number pad: digits, a decimal point and a minus. |
| `"kana"` | The Japanese kana table (gojūon) with the voicing marks, small kana and the long-vowel mark; typed kana compose until committed. |

```ogt
enum KeyboardLayout {
  doc "An on-screen keyboard's keys."
  variant letters "Latin letters (QWERTY), shift and caps, digits on a long press of the top row (the default)."
  variant symbols "Digits and punctuation."
  variant numeric "A number pad: digits, a decimal point and a minus."
  variant kana "The Japanese kana table (gojūon) with the voicing marks, small kana and the long-vowel mark; typed kana compose until committed."
}
```

<sub>Module `ui` line 4037</sub>
