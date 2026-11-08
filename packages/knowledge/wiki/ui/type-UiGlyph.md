<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiGlyph"></a>

### `UiGlyph` (struct)

An action's control on the device used last.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `icon` | `string` | `""` | — | — | — | The icon a style's set draws: `key.enter`, `xbox.a`, `ps.cross`, `switch.b`, `pad.south`, `touch.<action>`. |
| `text` | `string` | `""` | — | — | — | The text that names it: `Enter`, `A`, `×`. |
| `device` | `UiDevice` | `"keyboardMouse"` | — | — | — | The device it is for. |

```ogt
struct UiGlyph {
  doc "An action's control on the device used last."
  field icon String "The icon a style's set draws: `key.enter`, `xbox.a`, `ps.cross`, `switch.b`, `pad.south`, `touch.<action>`."
  field text String "The text that names it: `Enter`, `A`, `×`."
  field device UiDevice "The device it is for."
}
```

<sub>Module `ui` line 3643</sub>
