<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-InputGlyph"></a>

### `InputGlyph` (struct)

A control's glyph, for prompts and a controls screen.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `icon` | `string` | `""` | — | — | — | The icon a style's set draws: `key.space`, `mouse.left`, `xbox.a`, `ps.cross`, `switch.b`, `pad.south`, `xbox.stickL`, `key.wasd`, `touch.stick`. |
| `text` | `string` | `""` | — | — | — | The text that names it: `Space`, `LMB`, `A`, `×`, `L Stick`, `W A S D`. |
| `device` | `InputDevice` | `"keyboardMouse"` | — | — | — | The device it is drawn for. |
| `control` | `string` | `""` | — | — | — | The control it names (a composite's: its parts', space-separated). |

```ogt
struct InputGlyph {
  doc "A control's glyph, for prompts and a controls screen."
  field icon String "The icon a style's set draws: `key.space`, `mouse.left`, `xbox.a`, `ps.cross`, `switch.b`, `pad.south`, `xbox.stickL`, `key.wasd`, `touch.stick`."
  field text String "The text that names it: `Space`, `LMB`, `A`, `×`, `L Stick`, `W A S D`."
  field device InputDevice "The device it is drawn for."
  field control String "The control it names (a composite's: its parts', space-separated)."
}
```

<sub>`api.ogt` line 2211</sub>
