<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NavIntent"></a>

### `NavIntent` (enum)

A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give.

A string: one of the variants below. The zero value is `"up"`.

| Variant | Doc |
|---|---|
| `"up"` | Up. |
| `"down"` | Down. |
| `"left"` | Left. |
| `"right"` | Right. |
| `"next"` | The next control in reading order (Tab). |
| `"prev"` | The previous one (Shift+Tab). |
| `"activate"` | Press the focused control (Enter, Space, A). |
| `"back"` | Leave: the topmost dialog or menu closes, else og.ui.back() answers true (Escape, B). |

```ogt
enum NavIntent {
  doc "A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give."
  variant up "Up."
  variant down "Down."
  variant left "Left."
  variant right "Right."
  variant next "The next control in reading order (Tab)."
  variant prev "The previous one (Shift+Tab)."
  variant activate "Press the focused control (Enter, Space, A)."
  variant back "Leave: the topmost dialog or menu closes, else og.ui.back() answers true (Escape, B)."
}
```

<sub>Module `ui` line 823</sub>
