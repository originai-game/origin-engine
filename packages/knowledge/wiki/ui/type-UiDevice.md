<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiDevice"></a>

### `UiDevice` (enum)

The device a glyph set is for: the one the player used last.

A string: one of the variants below. The zero value is `"keyboardMouse"`.

| Variant | Doc |
|---|---|
| `"keyboardMouse"` | Keyboard and mouse. |
| `"xbox"` | An Xbox pad. |
| `"playstation"` | A PlayStation pad. |
| `"switch"` | A Nintendo Switch pad (south is B). |
| `"gamepad"` | A generic standard-mapping pad. |
| `"touch"` | A touch screen. |

```ogt
enum UiDevice {
  doc "The device a glyph set is for: the one the player used last."
  variant keyboardMouse "Keyboard and mouse."
  variant xbox "An Xbox pad."
  variant playstation "A PlayStation pad."
  variant switch "A Nintendo Switch pad (south is B)."
  variant gamepad "A generic standard-mapping pad."
  variant touch "A touch screen."
}
```

<sub>`api.ogt` line 23715</sub>
