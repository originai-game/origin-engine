<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StylePart"></a>

### `StylePart` (enum)

An element class a style dresses with a nine-slice frame.

A string: one of the variants below. The zero value is `"panel"`.

| Variant | Doc |
|---|---|
| `"panel"` | Dialogs and screen panels. |
| `"card"` | Cards. |
| `"button"` | Buttons (with hover, pressed and disabled states). |
| `"field"` | Text fields. |
| `"slot"` | Inventory and grid cells. |
| `"tooltip"` | Tooltips. |
| `"hud"` | HUD elements over the game (counters, timers) and toasts. |
| `"track"` | A progress bar's track. |

```ogt
enum StylePart {
  doc "An element class a style dresses with a nine-slice frame."
  variant panel "Dialogs and screen panels."
  variant card "Cards."
  variant button "Buttons (with hover, pressed and disabled states)."
  variant field "Text fields."
  variant slot "Inventory and grid cells."
  variant tooltip "Tooltips."
  variant hud "HUD elements over the game (counters, timers) and toasts."
  variant track "A progress bar's track."
}
```

<sub>`api.ogt` line 16167</sub>
