<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MotionToken"></a>

### `MotionToken` (enum)

A named motion of the style (docs/ui.md §3.1): components use these, so a style's personality changes every motion at once.

A string: one of the variants below. The zero value is `"ui"`.

| Variant | Doc |
|---|---|
| `"ui"` | The control spring: hover, press, toggles. |
| `"panel"` | Panels, sheets and drawers. |
| `"emphasis"` | Rewards, results and level-ups. |
| `"playful"` | Bouncy toys: chips, stickers, mascots. |
| `"enter"` | The entrance ease and duration. |
| `"exit"` | The exit ease and duration. |

```ogt
enum MotionToken {
  doc "A named motion of the style (docs/ui.md §3.1): components use these, so a style's personality changes every motion at once."
  variant ui "The control spring: hover, press, toggles."
  variant panel "Panels, sheets and drawers."
  variant emphasis "Rewards, results and level-ups."
  variant playful "Bouncy toys: chips, stickers, mascots."
  variant enter "The entrance ease and duration."
  variant exit "The exit ease and duration."
}
```

<sub>Module `ui` line 816</sub>
