<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-InputDevice"></a>

### `InputDevice` (enum)

A kind of input device: the set a glyph is drawn from, the device used last, and what a pad is. og.ui's UiDevice has the same values.

A string: one of the variants below. The zero value is `"keyboardMouse"`.

| Variant | Doc |
|---|---|
| `"keyboardMouse"` | Keyboard and mouse. |
| `"xbox"` | An Xbox pad (and the family a pad whose kind the host could not tell is drawn as). |
| `"playstation"` | A PlayStation pad. |
| `"switch"` | A Nintendo Switch pad (its south button is B). |
| `"gamepad"` | A generic pad: glyphs name positions (South, East). |
| `"touch"` | A touch screen. |

```ogt
enum InputDevice {
  doc "A kind of input device: the set a glyph is drawn from, the device used last, and what a pad is. og.ui's UiDevice has the same values."
  variant keyboardMouse "Keyboard and mouse."
  variant xbox "An Xbox pad (and the family a pad whose kind the host could not tell is drawn as)."
  variant playstation "A PlayStation pad."
  variant switch "A Nintendo Switch pad (its south button is B)."
  variant gamepad "A generic pad: glyphs name positions (South, East)."
  variant touch "A touch screen."
}
```

<sub>`api.ogt` line 2026</sub>
