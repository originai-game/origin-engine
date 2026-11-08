<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiPointer"></a>

### `UiPointer` (struct)

The primary pointer as the UI reads it this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Its x from the surface's left. |
| `y` | `number` | `0` | — | `cssPx` | — | Its y from the surface's top. |
| `inside` | `boolean` | `false` | — | — | — | It is over the surface: false for a mouse that left it, or a touch screen with no finger down. |
| `down` | `boolean` | `false` | — | — | — | Its button or finger is down. |
| `pressed` | `boolean` | `false` | — | — | — | Its button or finger went down this tick: where a drag a component runs itself (an inventory's item, a card) starts. |
| `released` | `boolean` | `false` | — | — | — | Its button or finger came up this tick: where such a drag ends. |
| `shift` | `boolean` | `false` | — | — | — | A Shift key is held (a split stack, a quick transfer, a range of a list). |
| `ctrl` | `boolean` | `false` | — | — | — | A Control key is held. |
| `alt` | `boolean` | `false` | — | — | — | An Alt (Option) key is held. |

```ogt
struct UiPointer {
  doc "The primary pointer as the UI reads it this tick."
  field x f32 "Its x from the surface's left." {
    unit cssPx
  }
  field y f32 "Its y from the surface's top." {
    unit cssPx
  }
  field inside bool "It is over the surface: false for a mouse that left it, or a touch screen with no finger down."
  field down bool "Its button or finger is down."
  field pressed bool "Its button or finger went down this tick: where a drag a component runs itself (an inventory's item, a card) starts."
  field released bool "Its button or finger came up this tick: where such a drag ends."
  field shift bool "A Shift key is held (a split stack, a quick transfer, a range of a list)."
  field ctrl bool "A Control key is held."
  field alt bool "An Alt (Option) key is held."
}
```

<sub>Module `ui` line 1089</sub>
