<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-CaptureAccept"></a>

### `CaptureAccept` (enum)

Which devices a capture takes.

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | Keyboard, mouse and pads. |
| `"keyboardMouse"` | Keyboard and mouse. |
| `"gamepad"` | Pads. |

```ogt
enum CaptureAccept {
  doc "Which devices a capture takes."
  variant any "Keyboard, mouse and pads."
  variant keyboardMouse "Keyboard and mouse."
  variant gamepad "Pads."
}
```

<sub>Module `input` line 203</sub>
